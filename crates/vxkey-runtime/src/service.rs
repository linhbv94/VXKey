//! Embeddable command service for the future Tauri owner. No UI or global hooks.
use crate::{
    config::{Config, Shortcut, choice, valid_app_id},
    history::History,
    storage::ConfigStore,
};
use serde_json::{Value, json};
use std::time::{SystemTime, UNIX_EPOCH};
pub struct Service {
    pub store: ConfigStore,
    pub history: History,
    pub suspended: bool,
    sequence: u64,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Surface {
    Settings,
    Tray,
    History,
    Onboarding,
    Conversion,
    Developer,
}
impl Service {
    pub fn dispatch_from(&mut self, surface: Surface, request: Value) -> Value {
        let command = request.get("command").and_then(Value::as_str).unwrap_or("");
        let allowed = match surface {
            Surface::Developer => true,
            Surface::Settings => !matches!(command, "convert_text"),
            Surface::Tray => [
                "get_snapshot",
                "set_language",
                "set_current_app_bypass",
                "set_input_suspended",
            ]
            .contains(&command),
            Surface::History => [
                "get_snapshot",
                "list_history",
                "get_history_entry",
                "copy_history_entry",
                "delete_history_entry",
                "clear_history",
            ]
            .contains(&command),
            Surface::Onboarding => ["get_snapshot"].contains(&command),
            Surface::Conversion => ["get_snapshot", "convert_text"].contains(&command),
        };
        if !allowed {
            return json!({"ok":false,"request_id":null,"data":null,"error":{"code":"permission_denied","message_key":"errors.permission_denied","retryable":false}});
        }
        self.dispatch(request)
    }

    pub fn new(store: ConfigStore) -> Self {
        Self {
            store,
            history: History::default(),
            suspended: true,
            sequence: 0,
        }
    }
    pub fn dispatch(&mut self, request: Value) -> Value {
        self.sequence += 1;
        let id = format!("00000000-0000-4000-8000-{:012x}", self.sequence);
        let result = self.execute(&request);
        match result {
            Ok(data) => json!({"ok":true,"request_id":id,"data":data,"error":null}),
            Err(code) => {
                json!({"ok":false,"request_id":id,"data":null,"error":{"code":code,"message_key":format!("errors.{code}"),"retryable":matches!(code.as_str(),"revision_conflict"|"persist_failed"|"unavailable")}})
            }
        }
    }
    fn execute(&mut self, r: &Value) -> Result<Value, String> {
        let command = r
            .get("command")
            .and_then(Value::as_str)
            .ok_or("invalid_argument")?;
        match command {
            "get_snapshot" => {
                fields(r, &["command"])?;
                let config = &self.store.config;
                let platform = if cfg!(target_os = "macos") {
                    "macos"
                } else if cfg!(target_os = "windows") {
                    "windows"
                } else {
                    "linux"
                };
                let mut redacted = serde_json::to_value(config).map_err(|_| "invalid_config")?;
                redacted["macros"]["entries"] = json!([]);
                Ok(
                    json!({"config":redacted,"revision":config.revision,"version":"0.1.0","platform":platform,"capabilities":{"settings":true,"session_history":true,"native_capture":false,"encrypted_history":false,"native_engine":false,"native_shortcuts":false,"autostart":false,"windows_history_provider":false,"conversion_text":true,"conversion_files":false,"updater":false},"runtime":{"engine_status":"unavailable","source_status":"unknown","effective_mode":if self.suspended{"suspended"}else{"unavailable"},"connection_status":"disconnected","health_state":"suspended","applied_revision":"0","period_substitution":{"native_state":"unknown","preference":config.typing.double_space_period,"effective_enabled":false,"control_enabled":false,"reason":"checking"},"current_app":null,"capture_state":if self.history.paused{"paused"}else{"off"}}}),
                )
            }
            "patch_config" => {
                fields(r, &["command", "expected_revision", "changes"])?;
                let expected = string(r, "expected_revision")?;
                let changes = r
                    .get("changes")
                    .and_then(Value::as_object)
                    .ok_or("invalid_argument")?;
                if changes.is_empty() {
                    return Err("invalid_argument".into());
                }
                let mut value =
                    serde_json::to_value(&self.store.config).map_err(|_| "invalid_config")?;
                for (group, items) in changes {
                    let allowed: &[&str] = match group.as_str() {
                        "ui" => &["locale", "theme", "tray_icon_style"],
                        "typing" => &[
                            "input_method",
                            "tone_style",
                            "language_mode",
                            "composition_mode",
                            "smart_restore",
                            "eager_restore",
                            "spell_check",
                            "retone",
                            "auto_capitalize",
                            "extra_onsets",
                            "double_space_period",
                        ],
                        "apps" => &["remember_per_app"],
                        "clipboard" => &["max_entries", "ttl_days"],
                        "system" => &["show_tray_icon"],
                        _ => return Err("invalid_argument".into()),
                    };
                    let object = items.as_object().ok_or("invalid_argument")?;
                    if object.is_empty() {
                        return Err("invalid_argument".into());
                    }
                    for (key, val) in object {
                        if !allowed.contains(&key.as_str()) {
                            return Err("invalid_argument".into());
                        }
                        if key == "double_space_period" && val == &json!(true) {
                            return Err("capability_locked".into());
                        }
                        value[group][key] = val.clone();
                    }
                }
                let config: Config =
                    serde_json::from_value(value).map_err(|_| "invalid_argument")?;
                self.store.replace(config, expected)?;
                self.apply_limits();
                Ok(self.persisted())
            }
            "set_app_policy" => {
                fields(
                    r,
                    &[
                        "command",
                        "expected_revision",
                        "app_id",
                        "input_bypass",
                        "clipboard_blocked",
                    ],
                )?;
                let id = string(r, "app_id")?;
                if !valid_app_id(id) || id.starts_with("com.vxkey.") {
                    return Err("invalid_argument".into());
                }
                let mut c = self.store.config.clone();
                c.apps.policies.insert(
                    id.into(),
                    crate::config::AppPolicy {
                        input_bypass: boolean(r, "input_bypass")?,
                        clipboard_blocked: boolean(r, "clipboard_blocked")?,
                    },
                );
                self.store.replace(c, string(r, "expected_revision")?)?;
                Ok(json!({"revision":self.store.config.revision,"apply_state":"unavailable"}))
            }
            "forget_app_language" => {
                fields(r, &["command", "expected_revision", "app_id"])?;
                let mut c = self.store.config.clone();
                let removed = c
                    .apps
                    .language_memory
                    .remove(string(r, "app_id")?)
                    .is_some();
                self.store.replace(c, string(r, "expected_revision")?)?;
                Ok(json!({"revision":self.store.config.revision,"removed":removed}))
            }
            "list_macros" => {
                fields(r, &["command", "expected_revision", "offset", "limit"])?;
                if string(r, "expected_revision")? != self.store.config.revision {
                    return Err("revision_conflict".into());
                }
                let offset = number(r, "offset", 0, 1000)?;
                let limit = number(r, "limit", 1, 50)?;
                Ok(
                    json!({"items":self.store.config.macros.entries.iter().skip(offset).take(limit).collect::<Vec<_>>(),"revision":self.store.config.revision}),
                )
            }
            "replace_macros" => {
                fields(
                    r,
                    &[
                        "command",
                        "expected_revision",
                        "enabled",
                        "in_en",
                        "smart_case",
                        "entries",
                    ],
                )?;
                let mut c = self.store.config.clone();
                c.macros.enabled = boolean(r, "enabled")?;
                c.macros.in_en = boolean(r, "in_en")?;
                c.macros.smart_case = boolean(r, "smart_case")?;
                c.macros.entries =
                    serde_json::from_value(r.get("entries").cloned().ok_or("invalid_argument")?)
                        .map_err(|_| "invalid_argument")?;
                self.store.replace(c, string(r, "expected_revision")?)?;
                Ok(self.persisted())
            }
            "register_hotkey" => {
                fields(r, &["command", "expected_revision", "action", "shortcut"])?;
                let action = string(r, "action")?;
                choice(action, &["toggle", "flip", "history"])?;
                let shortcut: Option<Shortcut> =
                    serde_json::from_value(r.get("shortcut").cloned().ok_or("invalid_argument")?)
                        .map_err(|_| "invalid_argument")?;
                if let Some(s) = shortcut {
                    s.validate(action == "toggle")?;
                }
                Err("unsupported".into())
            }
            "list_history" => {
                fields(r, &["command", "query", "offset", "limit"])?;
                let query = string(r, "query")?;
                if query.chars().count() > 256 {
                    return Err("invalid_argument".into());
                }
                let offset = number(r, "offset", 0, 200)?;
                let limit = number(r, "limit", 1, 50)?;
                let items = self.history.search(query, offset, limit, now());
                Ok(
                    json!({"items":items.iter().map(|e|json!({"id":e.id,"preview":e.text.chars().take(200).collect::<String>(),"created_at_ms":e.created_ms,"byte_length":e.text.len()})).collect::<Vec<_>>(),"total":self.history.len(),"store_revision":self.history.revision.to_string()}),
                )
            }
            "get_history_entry" => {
                fields(r, &["command", "entry_id", "expected_store_revision"])?;
                let rev = crate::config::counter(string(r, "expected_store_revision")?)?;
                let e = self
                    .history
                    .get(string(r, "entry_id")?, rev, now())
                    .map_err(str::to_string)?;
                Ok(
                    json!({"id":e.id,"text":e.text,"byte_length":e.text.len(),"store_revision":self.history.revision.to_string()}),
                )
            }
            "delete_history_entry" => {
                fields(r, &["command", "entry_id", "expected_store_revision"])?;
                let rev = crate::config::counter(string(r, "expected_store_revision")?)?;
                let removed = self
                    .history
                    .delete(string(r, "entry_id")?, rev)
                    .map_err(str::to_string)?;
                Ok(json!({"removed":removed,"store_revision":self.history.revision.to_string()}))
            }
            "set_capture_state" => {
                fields(r, &["command", "action", "consent_id"])?;
                match string(r, "action")? {
                    "disable" => {
                        self.history.enabled = false;
                        Ok(json!({"capture_state":"off","permission_status":"unknown"}))
                    }
                    "pause" => {
                        self.history.paused = true;
                        Ok(json!({"capture_state":"paused","permission_status":"unknown"}))
                    }
                    "enable" | "resume" | "retry" => Err("unsupported".into()),
                    _ => Err("invalid_argument".into()),
                }
            }
            "convert_text" => {
                fields(
                    r,
                    &[
                        "command",
                        "text",
                        "from_charset",
                        "to_charset",
                        "transforms",
                        "keep_d",
                        "flatten_caps",
                    ],
                )?;
                use funput_core::{
                    charset::{Charset, convert},
                    textcase::{Options, Transform, apply},
                };
                fn charset(name: &str) -> Result<Charset, String> {
                    match name {
                        "unicode" => Ok(Charset::Unicode),
                        "tcvn3" => Ok(Charset::Tcvn3),
                        "vni_windows" => Ok(Charset::VniWindows),
                        "unicode_combining" => Ok(Charset::UnicodeCombining),
                        _ => Err("invalid_argument".into()),
                    }
                }
                let text = string(r, "text")?;
                if text.len() > 1024 * 1024 {
                    return Err("quota_exceeded".into());
                }
                let source = charset(string(r, "from_charset")?)?;
                let target = charset(string(r, "to_charset")?)?;
                let pivot = convert(text, source, Charset::Unicode);
                let mut value = pivot.text;
                let options = Options {
                    d_to_ascii: !boolean(r, "keep_d")?,
                    keep_all_caps: !boolean(r, "flatten_caps")?,
                };
                let transforms = r
                    .get("transforms")
                    .and_then(Value::as_array)
                    .ok_or("invalid_argument")?;
                if transforms.len() > 5 {
                    return Err("invalid_argument".into());
                }
                let mut seen = std::collections::BTreeSet::new();
                for item in transforms {
                    let name = item.as_str().ok_or("invalid_argument")?;
                    if !seen.insert(name) {
                        return Err("invalid_argument".into());
                    }
                    let transform = match name {
                        "upper" => Transform::Upper,
                        "lower" => Transform::Lower,
                        "no_diacritics" => Transform::NoDiacritics,
                        "sentence" => Transform::Sentence,
                        "title" => Transform::Title,
                        _ => return Err("invalid_argument".into()),
                    };
                    value = apply(&value, transform, options);
                }
                let output = convert(&value, Charset::Unicode, target);
                Ok(
                    json!({"text":output.text,"unmapped":pivot.unmapped+output.unmapped,"normalized":pivot.normalized+output.normalized}),
                )
            }
            "update_action"
            | "set_windows_history_provider"
            | "set_launch_at_login"
            | "set_history_mode"
            | "prepare_import"
            | "commit_import"
            | "export_config"
            | "copy_history_entry"
            | "clear_history"
            | "reset_preferences"
            | "set_input_suspended" => Err("unsupported".into()),
            "set_language" | "set_current_app_bypass" => Err("stale_target".into()),
            _ => Err("unsupported".into()),
        }
    }
    fn persisted(&self) -> Value {
        json!({"persisted_revision":self.store.config.revision,"applied_revision":"0","engine_ack_state":"unavailable"})
    }
    fn apply_limits(&mut self) {
        let c = &self.store.config.clipboard;
        let _ = self.history.set_limits(c.max_entries, c.ttl_days);
    }
}
fn fields(r: &Value, allowed: &[&str]) -> Result<(), String> {
    let o = r.as_object().ok_or("invalid_argument")?;
    if o.keys().any(|k| !allowed.contains(&k.as_str()))
        || allowed.iter().any(|k| !o.contains_key(*k))
    {
        return Err("invalid_argument".into());
    }
    Ok(())
}
fn string<'a>(r: &'a Value, k: &str) -> Result<&'a str, String> {
    r.get(k)
        .and_then(Value::as_str)
        .ok_or_else(|| "invalid_argument".into())
}
fn boolean(r: &Value, k: &str) -> Result<bool, String> {
    r.get(k)
        .and_then(Value::as_bool)
        .ok_or_else(|| "invalid_argument".into())
}
fn number(r: &Value, k: &str, min: usize, max: usize) -> Result<usize, String> {
    r.get(k)
        .and_then(Value::as_u64)
        .and_then(|n| usize::try_from(n).ok())
        .filter(|n| *n >= min && *n <= max)
        .ok_or_else(|| "invalid_argument".into())
}
fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}
