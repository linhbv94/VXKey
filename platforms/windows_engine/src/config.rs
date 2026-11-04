use funput_engine::EngineConfig;
use std::collections::{BTreeMap, BTreeSet};

pub const CTRL: u8 = 1;
pub const ALT: u8 = 2;
pub const SHIFT: u8 = 4;
pub const META: u8 = 8;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Binding {
    pub modifiers: u8,
    /// Windows virtual key. None means a modifier-release gesture.
    pub key: Option<u16>,
}

#[derive(Clone, Debug)]
pub struct Settings {
    pub revision: u64,
    pub engine: EngineConfig,
    pub vietnamese: bool,
    pub retone: bool,
    pub double_space_period: bool,
    pub toggle: Option<Binding>,
    pub flip: Option<Binding>,
    pub macros: Vec<(String, String)>,
    /// Canonical full executable paths, resolved off the keyboard thread.
    pub excluded_apps: BTreeSet<String>,
    pub remember_per_app: bool,
    pub language_memory: BTreeMap<String, bool>,
}

impl Default for Settings {
    fn default() -> Self {
        let engine = EngineConfig::default();
        Self {
            revision: 0,
            engine,
            vietnamese: true,
            retone: true,
            double_space_period: true,
            toggle: Some(Binding {
                modifiers: CTRL | META,
                key: None,
            }),
            flip: None,
            macros: Vec::new(),
            excluded_apps: BTreeSet::new(),
            remember_per_app: true,
            language_memory: BTreeMap::new(),
        }
    }
}

impl Settings {
    /// Run outside the hook; invalid snapshots never replace the running config.
    pub fn validate(&self) -> Result<(), &'static str> {
        for (binding, allow_gesture) in [(&self.toggle, true), (&self.flip, false)] {
            if let Some(b) = binding {
                if b.modifiers == 0 || b.modifiers & !15 != 0 {
                    return Err("invalid_modifiers");
                }
                match b.key {
                    None if !allow_gesture || b.modifiers.count_ones() < 2 => {
                        return Err("invalid_gesture");
                    }
                    Some(k) if k == 0 || k > 254 || is_modifier(k) => return Err("invalid_key"),
                    _ => {}
                }
                // Native clipboard history belongs to Windows in this engine.
                if b.modifiers & META != 0 && b.key == Some(0x56) {
                    return Err("native_history_reserved");
                }
            }
        }
        if self.toggle.is_some() && self.toggle == self.flip {
            return Err("duplicate_binding");
        }
        if self.excluded_apps.len() > 2000 || self.language_memory.len() > 2000 {
            return Err("policy_limit");
        }
        if self
            .excluded_apps
            .iter()
            .chain(self.language_memory.keys())
            .any(|p| canonical_app_id(p).as_deref() != Some(p))
        {
            return Err("invalid_app_identity");
        }
        if self.macros.len() > 1000 {
            return Err("macro_limit");
        }
        let bytes: usize = self
            .macros
            .iter()
            .map(|(k, v)| k.len() + v.len())
            .sum::<usize>()
            + self.excluded_apps.iter().map(String::len).sum::<usize>()
            + self.language_memory.keys().map(String::len).sum::<usize>();
        if bytes > 1024 * 1024 {
            return Err("config_too_large");
        }
        let mut triggers = BTreeSet::new();
        for (key, text) in &self.macros {
            let identity = if self.engine.shortcut_smart_case {
                key.to_lowercase()
            } else {
                key.clone()
            };
            if key.is_empty()
                || key.chars().count() > 32
                || key.chars().any(|c| c.is_whitespace() || c.is_control())
                || text.is_empty()
                || text.encode_utf16().count() > 8192
                || text
                    .chars()
                    .any(|c| c == '\0' || (c.is_control() && c != '\n' && c != '\t' && c != '\r'))
                || !triggers.insert(identity)
            {
                return Err("invalid_macro");
            }
        }
        Ok(())
    }
}

/// No basename identities: two installs named editor.exe must remain distinct.
pub fn canonical_app_id(path: &str) -> Option<String> {
    let path = path
        .strip_prefix(r"\\?\")
        .unwrap_or(path)
        .replace('/', "\\");
    let bytes = path.as_bytes();
    if path.len() > 32768
        || path.chars().any(char::is_control)
        || !(bytes.len() > 3 && bytes[0].is_ascii_alphabetic() && bytes[1..3] == *b":\\"
            || path.starts_with(r"\\"))
        || path.split('\\').any(|part| part == "." || part == "..")
    {
        return None;
    }
    Some(path.to_lowercase())
}

pub fn is_modifier(vk: u16) -> bool {
    matches!(vk, 0x10..=0x12 | 0x5b..=0x5c | 0xa0..=0xa5)
}

pub fn modifier(vk: u16) -> u8 {
    match vk {
        0x10 | 0xa0 | 0xa1 => SHIFT,
        0x11 | 0xa2 | 0xa3 => CTRL,
        0x12 | 0xa4 | 0xa5 => ALT,
        0x5b | 0x5c => META,
        _ => 0,
    }
}
