use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub schema_version: u32,
    pub revision: String,
    pub ui: Ui,
    pub typing: Typing,
    pub apps: Apps,
    pub macros: Macros,
    pub shortcuts: Shortcuts,
    pub clipboard: Clipboard,
    pub system: System,
}
macro_rules! model {
    ($name:ident { $($field:ident : $ty:ty),* $(,)? }) => {
        #[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
        #[serde(deny_unknown_fields)]
        pub struct $name { $(pub $field: $ty),* }
    };
}
model!(Ui {
    locale: String,
    theme: String,
    tray_icon_style: String
});
model!(Typing { input_method: String, tone_style: String, language_mode: String,
    composition_mode: String, smart_restore: bool, retone: bool, auto_capitalize: bool,
    eager_restore: bool, spell_check: bool, extra_onsets: Vec<String>, double_space_period: bool });
model!(Apps { remember_per_app: bool, language_memory: BTreeMap<String,String>, policies: BTreeMap<String,AppPolicy> });
model!(AppPolicy {
    input_bypass: bool,
    clipboard_blocked: bool
});
model!(Macros { enabled: bool, in_en: bool, entries: Vec<MacroEntry>, smart_case: bool });
model!(MacroEntry {
    id: String,
    trigger: String,
    replacement: String
});
model!(Shortcuts { toggle: Option<Shortcut>, history: Option<Shortcut>, flip: Option<Shortcut> });
model!(Clipboard {
    enabled: bool,
    history_mode: String,
    max_entries: usize,
    ttl_days: u32,
    windows_history_provider: String
});
model!(System {
    launch_companion_at_login: bool,
    show_tray_icon: bool
});
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Shortcut {
    ModifierChord {
        modifiers: Vec<String>,
    },
    KeyChord {
        modifiers: Vec<String>,
        key_code: String,
    },
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Platform {
    Macos,
    Windows,
    Other,
}
impl Platform {
    pub fn current() -> Self {
        if cfg!(target_os = "macos") {
            Self::Macos
        } else if cfg!(target_os = "windows") {
            Self::Windows
        } else {
            Self::Other
        }
    }
}
impl Config {
    pub fn fresh(platform: Platform) -> Self {
        let source = if platform == Platform::Windows {
            include_str!("../../../docs/spec/config_example_windows.json")
        } else {
            include_str!("../../../docs/spec/config_example.json")
        };
        // Checked-in examples are compiled into the binary; user input never reaches expect.
        serde_json::from_str(source).expect("VXKey default config is valid")
    }
    pub fn counter(&self) -> Result<u64, String> {
        counter(&self.revision)
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.schema_version != 1 {
            return Err("unsupported_schema".into());
        }
        self.counter()?;
        choice(&self.ui.locale, &["vi", "en"])?;
        choice(&self.ui.theme, &["system", "dark", "light", "pure_black"])?;
        choice(&self.ui.tray_icon_style, &["serious", "playful"])?;
        choice(
            &self.typing.input_method,
            &["telex", "telex_advanced", "vni"],
        )?;
        choice(&self.typing.tone_style, &["modern", "traditional"])?;
        choice(&self.typing.language_mode, &["vi", "en"])?;
        choice(
            &self.typing.composition_mode,
            &["preedit", "direct_verified"],
        )?;
        choice(&self.clipboard.history_mode, &["session", "encrypted"])?;
        choice(
            &self.clipboard.windows_history_provider,
            &["native", "vxkey"],
        )?;
        if !(1..=200).contains(&self.clipboard.max_entries)
            || !(1..=7).contains(&self.clipboard.ttl_days)
        {
            return Err("invalid_history_limit".into());
        }
        let extra: BTreeSet<_> = self.typing.extra_onsets.iter().collect();
        if extra.len() != self.typing.extra_onsets.len()
            || extra
                .iter()
                .any(|v| !["z", "f", "w", "j"].contains(&v.as_str()))
        {
            return Err("invalid_extra_onsets".into());
        }
        let app_ids: BTreeSet<_> = self
            .apps
            .policies
            .keys()
            .chain(self.apps.language_memory.keys())
            .collect();
        if app_ids.len() > 2000 || app_ids.iter().any(|id| !valid_app_id(id)) {
            return Err("invalid_app_policy".into());
        }
        for mode in self.apps.language_memory.values() {
            choice(mode, &["vi", "en"])?;
        }
        if self.macros.entries.len() > 1000 {
            return Err("macro_limit".into());
        }
        let mut ids = BTreeSet::new();
        let mut triggers = BTreeSet::new();
        for m in &self.macros.entries {
            let trigger = if self.macros.smart_case {
                m.trigger.to_lowercase()
            } else {
                m.trigger.clone()
            };
            if !valid_uuid(&m.id)
                || !ids.insert(&m.id)
                || !triggers.insert(trigger)
                || m.trigger.is_empty()
                || m.trigger.chars().count() > 32
                || m.trigger
                    .chars()
                    .any(|c| c.is_whitespace() || c.is_control())
                || m.replacement.is_empty()
                || m.replacement.len() > 8192
            {
                return Err("invalid_macro".into());
            }
        }
        for (s, modifier_ok) in [
            (&self.shortcuts.toggle, true),
            (&self.shortcuts.flip, false),
            (&self.shortcuts.history, false),
        ] {
            if let Some(shortcut) = s {
                shortcut.validate(modifier_ok)?;
            }
        }
        let shortcuts = [
            &self.shortcuts.toggle,
            &self.shortcuts.flip,
            &self.shortcuts.history,
        ];
        for (i, a) in shortcuts.iter().enumerate() {
            for b in &shortcuts[i + 1..] {
                if let (Some(a), Some(b)) = (a, b)
                    && a.identity() == b.identity()
                {
                    return Err("duplicate_shortcut".into());
                }
            }
        }
        if serde_json::to_vec(self)
            .map_err(|_| "invalid_config")?
            .len()
            > 1024 * 1024
        {
            return Err("config_too_large".into());
        }
        Ok(())
    }
}
impl Shortcut {
    fn identity(&self) -> (BTreeSet<&str>, Option<&str>) {
        match self {
            Self::ModifierChord { modifiers } => {
                (modifiers.iter().map(String::as_str).collect(), None)
            }
            Self::KeyChord {
                modifiers,
                key_code,
            } => (
                modifiers.iter().map(String::as_str).collect(),
                Some(key_code.as_str()),
            ),
        }
    }
    pub fn validate(&self, modifier_ok: bool) -> Result<(), String> {
        let (modifiers, key) = match self {
            Self::ModifierChord { modifiers } => (modifiers, None),
            Self::KeyChord {
                modifiers,
                key_code,
            } => (modifiers, Some(key_code)),
        };
        let set: BTreeSet<_> = modifiers.iter().collect();
        if modifiers.is_empty()
            || modifiers.len() > 4
            || set.len() != modifiers.len()
            || modifiers
                .iter()
                .any(|s| !["control", "alt", "shift", "meta"].contains(&s.as_str()))
        {
            return Err("invalid_shortcut".into());
        }
        match key {
            None if !modifier_ok || modifiers.len() < 2 => Err("unsupported_shortcut".into()),
            Some(k)
                if k.is_empty()
                    || k.len() > 32
                    || !k.chars().all(|c| c.is_ascii_alphanumeric()) =>
            {
                Err("invalid_shortcut".into())
            }
            _ => Ok(()),
        }
    }
}
pub fn choice(value: &str, allowed: &[&str]) -> Result<(), String> {
    if allowed.contains(&value) {
        Ok(())
    } else {
        Err("invalid_argument".into())
    }
}
pub fn valid_app_id(id: &str) -> bool {
    id.len() <= 255
        && id.split('.').count() > 1
        && id.split('.').all(|p| {
            !p.is_empty()
                && p.chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
        })
}
pub fn valid_uuid(id: &str) -> bool {
    id.len() == 36
        && id.char_indices().all(|(i, c)| {
            if [8, 13, 18, 23].contains(&i) {
                c == '-'
            } else {
                c.is_ascii_hexdigit()
            }
        })
}
pub fn counter(value: &str) -> Result<u64, String> {
    if value.is_empty()
        || value.len() > 20
        || (value.len() > 1 && value.starts_with('0'))
        || !value.bytes().all(|b| b.is_ascii_digit())
    {
        return Err("invalid_revision".into());
    }
    value.parse().map_err(|_| "invalid_revision".into())
}
