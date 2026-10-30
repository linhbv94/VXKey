use crate::{config::Config, input_policy::InputPolicy};
use funput_core::{ExtraOnsets, InputMethod, SyllableRules, ToneStyle};
use funput_engine::{Action, Engine, ImeResult};
/// Real upstream engine plus VXKey hard bypass. No platform injection or I/O here.
pub struct InputSession {
    engine: Engine,
    pub policy: InputPolicy,
    pending: Option<Config>,
}
impl Default for InputSession {
    fn default() -> Self {
        Self {
            engine: Engine::new(),
            policy: InputPolicy::default(),
            pending: None,
        }
    }
}
impl InputSession {
    pub fn configure(&mut self, c: &Config) -> bool {
        if !self.engine.buffer().is_empty() {
            self.pending = Some(c.clone());
            return false;
        }
        self.apply(c);
        true
    }
    fn apply(&mut self, c: &Config) {
        self.engine.update_config(|e| {
            e.method = match c.typing.input_method.as_str() {
                "vni" => InputMethod::Vni,
                "telex_advanced" => InputMethod::TelexAdvanced,
                _ => InputMethod::Telex,
            };
            e.tone_style = if c.typing.tone_style == "traditional" {
                ToneStyle::Traditional
            } else {
                ToneStyle::Modern
            };
            e.smart_restore = c.typing.smart_restore;
            e.eager_restore = c.typing.eager_restore;
            e.spell_check = c.typing.spell_check;
            e.auto_capitalize = c.typing.auto_capitalize;
            e.shortcuts_enabled = c.macros.enabled;
            e.shortcuts_in_english = c.macros.in_en;
            e.shortcut_smart_case = c.macros.smart_case;
            let mut extra = ExtraOnsets::NONE;
            for letter in &c.typing.extra_onsets {
                extra = extra.union(match letter.as_str() {
                    "z" => ExtraOnsets::Z,
                    "f" => ExtraOnsets::F,
                    "w" => ExtraOnsets::W,
                    "j" => ExtraOnsets::J,
                    _ => ExtraOnsets::NONE,
                });
            }
            e.syllable_rules = SyllableRules::STANDARD.with_extra_onsets(extra);
        });
        self.engine.clear_shortcuts();
        for m in &c.macros.entries {
            self.engine.add_shortcut(&m.trigger, &m.replacement);
        }
        self.policy.global_mode = c.typing.language_mode.clone();
        self.engine.set_enabled(self.policy.mode(c) == "vi");
    }
    pub fn process(&mut self, key: char, c: &Config) -> ImeResult {
        if self.policy.blocked(c) {
            self.engine.clear();
            return ImeResult {
                action: Action::None,
                backspace: 0,
                output: String::new(),
            };
        }
        self.engine.set_enabled(self.policy.mode(c) == "vi");
        let result = self.engine.process_char(key);
        if self.engine.buffer().is_empty() {
            if let Some(pending) = self.pending.take() {
                self.apply(&pending);
            }
        }
        result
    }
    pub fn buffer(&self) -> &str {
        self.engine.buffer()
    }
    pub fn clear(&mut self) {
        self.engine.clear();
        self.apply_pending_at_boundary();
    }
    fn apply_pending_at_boundary(&mut self) {
        if self.engine.buffer().is_empty() {
            if let Some(pending) = self.pending.take() {
                self.apply(&pending);
            }
        }
    }
    pub fn backspace(&mut self, c: &Config) -> ImeResult {
        if self.policy.blocked(c) {
            self.clear();
            return ImeResult {
                action: Action::None,
                backspace: 0,
                output: String::new(),
            };
        }
        let result = self.engine.on_backspace();
        self.apply_pending_at_boundary();
        result
    }
    pub fn flip(&mut self, c: &Config) -> ImeResult {
        if self.policy.blocked(c) || self.policy.mode(c) != "vi" {
            return ImeResult {
                action: Action::None,
                backspace: 0,
                output: String::new(),
            };
        }
        self.engine.flip_composing()
    }
}
