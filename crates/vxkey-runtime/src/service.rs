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
    pub native: Option<crate::bridge::Observation>,
    native_target_token: Option<(String, crate::bridge::Target)>,
    pub native_period: crate::period::NativePeriod,
    reset_confirmation: Option<(String, String, std::time::Instant)>,
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
    pub fn accept_native(&mut self, mut status: crate::bridge::NativeStatus, session: String) {
        if status.validate().is_err() || !crate::config::valid_uuid(&session) {
            return;
        }
        if status.fault.is_some() {
            status.suspended = true;
        }
        // No content enters logs/events. Persist only explicit mode deltas, with a
        // matching base revision and exact active external target.
        if let Some(delta) = &status.memory {
            if status.active
                && !status.suspended
                && self.store.config.apps.remember_per_app
                && delta.base_revision == self.store.config.revision
                && status
                    .target
                    .as_ref()
                    .is_some_and(|t| t.app_id == delta.app_id)
            {
                let mut config = self.store.config.clone();
                if config.apps.language_memory.get(&delta.app_id) != Some(&delta.mode) {
                    config
                        .apps
                        .language_memory
                        .insert(delta.app_id.clone(), delta.mode.clone());
                    let revision = self.store.config.revision.clone();
                    let _ = self.store.replace(config, &revision);
                }
            }
        }
        let target = status.target.as_ref().filter(|_| status.active);
        let same_session = self.native.as_ref().is_some_and(|n| n.session == session);
        if !same_session || self.native_target_token.as_ref().map(|(_, t)| t) != target {
            self.native_target_token =
                target.and_then(|t| crate::bridge::fresh_id().ok().map(|id| (id, t.clone())));
        }
        self.suspended = status.suspended;
        self.native = Some(crate::bridge::Observation {
            status,
            seen: std::time::Instant::now(),
            session,
        });
    }
    pub fn native_snapshot(&self, value: &mut Value) {
        let Some(observation) = self.native.as_ref().filter(|n| n.fresh()) else {
            return;
        };
        let status = &observation.status;
        value["capabilities"]["native_engine"] = json!(true);
        value["runtime"]["connection_status"] = json!("connected");
        value["runtime"]["driver_fault"] = json!(status.fault);
        value["runtime"]["driver_status"] = json!(if status.driver.attached {
            "active"
        } else {
            "inactive"
        });
        value["runtime"]["driver"] = json!(status.driver);
        #[cfg(debug_assertions)]
        {
            value["runtime"]["driver_diagnostics"] = json!(status.diagnostics);
        }
        value["runtime"]["engine_status"] = json!(if status.suspended {
            "suspended"
        } else if !status.driver.attached {
            "unavailable"
        } else {
            "ready"
        });
        value["runtime"]["health_state"] = json!(if status.suspended {
            "suspended"
        } else if !status.driver.attached {
            "degraded"
        } else {
            "healthy"
        });
        value["runtime"]["effective_mode"] = json!(if status.suspended {
            "suspended"
        } else if !status.active || !status.driver.attached {
            "unavailable"
        } else if status.target.as_ref().is_some_and(|t| self
            .store
            .config
            .apps
            .policies
            .get(&t.app_id)
            .is_some_and(|p| p.input_bypass))
        {
            "bypass"
        } else {
            &status.mode
        });
        value["runtime"]["applied_revision"] = json!(status.applied_revision);
        value["runtime"]["engine_session_id"] = json!(status.engine_session_id);
        value["runtime"]["runtime_sequence"] = json!(status.loop_sequence);
        // No OS-frontmost inference. The target originates from the native foreground-app cache.
        value["runtime"]["current_app"] = if status.active {
            status
                .target
                .as_ref()
                .map(|target| {
                    let mut value = serde_json::to_value(target).unwrap_or(Value::Null);
                    value["display_name"] = json!(target.app_id);
                    value["input_bypass"] = json!(
                        self.store
                            .config
                            .apps
                            .policies
                            .get(&target.app_id)
                            .is_some_and(|p| p.input_bypass)
                    );
                    value["target_token"] = self
                        .native_target_token
                        .as_ref()
                        .map(|(id, _)| json!(id))
                        .unwrap_or(Value::Null);
                    value
                })
                .unwrap_or(Value::Null)
        } else {
            Value::Null
        };
    }
    pub fn dispatch_from(&mut self, surface: Surface, request: Value) -> Value {
        let command = request.get("command").and_then(Value::as_str).unwrap_or("");
        let allowed = match surface {
            Surface::Developer => true,
            Surface::Settings => [
                "get_snapshot",
                "patch_config",
                "set_app_policy",
                "forget_app_language",
                "list_macros",
                "replace_macros",
                "register_hotkey",
                "set_launch_at_login",
                "set_capture_state",
                "set_history_mode",
                "set_windows_history_provider",
                "prepare_import",
                "commit_import",
                "export_config",
                "reset_preferences",
                "update_action",
            ]
            .contains(&command),
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

    /// Only a trusted Settings host can mint a short-lived, revision-bound token.
    pub fn prepare_reset(&mut self, expected: &str) -> Result<String, String> {
        if expected != self.store.config.revision {
            return Err("revision_conflict".into());
        }
        let id = fresh_id()?;
        self.reset_confirmation = Some((
            id.clone(),
            expected.into(),
            std::time::Instant::now()
                .checked_add(std::time::Duration::from_secs(30))
                .ok_or("unavailable")?,
        ));
        Ok(id)
    }
    pub fn new(store: ConfigStore) -> Self {
        Self {
            store,
            history: History::default(),
            suspended: true,
            native: None,
            native_target_token: None,
            native_period: crate::period::NativePeriod::Unknown,
            reset_confirmation: None,
        }
    }
    pub fn dispatch(&mut self, request: Value) -> Value {
        let id = match fresh_id() {
            Ok(id) => id,
            Err(_) => {
                return json!({"ok":false,"request_id":null,"data":null,"error":{"code":"unavailable","message_key":"errors.unavailable","retryable":true}});
            }
        };
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
                let (native_state, control_enabled, period_reason) = match self.native_period {
                    crate::period::NativePeriod::Enabled => ("enabled", false, "native_enabled"),
                    crate::period::NativePeriod::Disabled => {
                        ("disabled", true, "engine_unavailable")
                    }
                    crate::period::NativePeriod::NotApplicable => {
                        ("not_applicable", true, "engine_unavailable")
                    }
                    crate::period::NativePeriod::Unknown => ("unknown", false, "checking"),
                };
                let mut redacted = serde_json::to_value(config).map_err(|_| "invalid_config")?;
                redacted["macros"]["entries"] = json!([]);
                let mut snapshot = json!({"config":redacted,"revision":config.revision,"version":"0.1.0","platform":platform,"capabilities":{"settings":true,"session_history":true,"native_capture":false,"encrypted_history":false,"native_engine":false,"native_shortcuts":false,"autostart":false,"windows_history_provider":false,"conversion_text":true,"conversion_files":false,"updater":false},"runtime":{"engine_status":"unavailable","driver_status":"unknown","effective_mode":if self.suspended{"suspended"}else{"unavailable"},"connection_status":"disconnected","health_state":"suspended","applied_revision":"0","period_substitution":{"native_state":native_state,"preference":config.typing.double_space_period,"effective_enabled":false,"control_enabled":control_enabled,"reason":period_reason},"current_app":null,"capture_state":if self.history.paused{"paused"}else{"off"}}});
                self.native_snapshot(&mut snapshot);
                Ok(snapshot)
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
                        if key == "double_space_period"
                            && val == &json!(true)
                            && !matches!(
                                self.native_period,
                                crate::period::NativePeriod::Disabled
                                    | crate::period::NativePeriod::NotApplicable
                            )
                        {
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
                let policy = crate::config::AppPolicy {
                    input_bypass: boolean(r, "input_bypass")?,
                    clipboard_blocked: boolean(r, "clipboard_blocked")?,
                };
                if !policy.input_bypass && !policy.clipboard_blocked {
                    c.apps.policies.remove(id);
                } else {
                    c.apps.policies.insert(id.into(), policy);
                }
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
            "reset_preferences" => {
                fields(r, &["command", "expected_revision", "confirmation_id"])?;
                let expected = string(r, "expected_revision")?;
                let token = string(r, "confirmation_id")?;
                let valid =
                    self.reset_confirmation
                        .as_ref()
                        .is_some_and(|(id, revision, expiry)| {
                            id == token
                                && revision == expected
                                && std::time::Instant::now() <= *expiry
                        });
                if !valid {
                    return Err("permission_denied".into());
                }
                self.reset_confirmation = None;
                let defaults = Config::fresh(crate::config::Platform::current());
                let mut config = self.store.config.clone();
                config.ui.theme = defaults.ui.theme;
                config.typing = defaults.typing;
                self.store.replace(config, expected)?;
                Ok(
                    json!({"revision":self.store.config.revision,"preserved_fields":["ui.locale","ui.tray_icon_style","apps","macros","shortcuts","clipboard","system"],"engine_ack_state":"unavailable"}),
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
            | "set_input_suspended" => Err("unsupported".into()),
            "set_current_app_bypass" => {
                fields(
                    r,
                    &[
                        "command",
                        "target_token",
                        "expected_revision",
                        "input_bypass",
                    ],
                )?;
                let token = string(r, "target_token")?;
                let Some((expected, target)) = &self.native_target_token else {
                    return Err("stale_target".into());
                };
                if token != expected
                    || !self.native.as_ref().is_some_and(|n| {
                        n.fresh() && n.status.active && n.status.target.as_ref() == Some(target)
                    })
                {
                    return Err("stale_target".into());
                }
                let mut config = self.store.config.clone();
                let policy = config.apps.policies.entry(target.app_id.clone()).or_insert(
                    crate::config::AppPolicy {
                        input_bypass: false,
                        clipboard_blocked: false,
                    },
                );
                policy.input_bypass = boolean(r, "input_bypass")?;
                if !policy.input_bypass && !policy.clipboard_blocked {
                    config.apps.policies.remove(&target.app_id);
                }
                self.store
                    .replace(config, string(r, "expected_revision")?)?;
                Ok(
                    json!({"revision":self.store.config.revision,"input_bypass":boolean(r,"input_bypass")?,"apply_state":"pending"}),
                )
            }
            "set_language" => Err("stale_target".into()),
            _ => Err("unsupported".into()),
        }
    }
    fn persisted(&self) -> Value {
        let applied = self
            .native
            .as_ref()
            .filter(|n| n.fresh())
            .map(|n| n.status.applied_revision.as_str())
            .unwrap_or("0");
        json!({"persisted_revision":self.store.config.revision,"applied_revision":applied,"engine_ack_state":if applied==self.store.config.revision {"applied"}else if self.native.as_ref().is_some_and(|n|n.fresh()) {"pending"}else{"unavailable"}})
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

fn fresh_id() -> Result<String, String> {
    let mut bytes = [0u8; 16];
    getrandom::fill(&mut bytes).map_err(|_| "unavailable")?;
    bytes[6] = (bytes[6] & 0x0f) | 0x40;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    let hex = bytes.iter().map(|b| format!("{b:02x}")).collect::<String>();
    let id = format!(
        "{}-{}-{}-{}-{}",
        &hex[..8],
        &hex[8..12],
        &hex[12..16],
        &hex[16..20],
        &hex[20..]
    );
    Ok(id)
}
