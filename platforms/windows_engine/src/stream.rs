use crate::{
    config::Settings,
    delivery::{Edit, MAX_BACKSPACES, MAX_UNITS},
};
use funput_engine::{Action, Engine, ImeResult, KeySource};

/// Observed local stream, not proof of a document range or grapheme count.
pub struct Stream {
    engine: Engine,
    rendered: String,
    completed: Option<String>,
    bypass_word: bool,
    space_eligible: bool,
    tail_char: Option<char>,
}

impl Stream {
    pub fn new(config: &Settings) -> Self {
        let mut engine = Engine::new();
        engine.update_config(|e| *e = config.engine.clone());
        for (key, text) in &config.macros {
            engine.add_shortcut(key, text);
        }
        Self {
            engine,
            rendered: String::new(),
            completed: None,
            bypass_word: false,
            space_eligible: false,
            tail_char: None,
        }
    }
    pub fn reset(&mut self) {
        self.engine.clear();
        self.engine.disarm_capitalization();
        self.rendered.clear();
        self.completed = None;
        self.bypass_word = false;
        self.space_eligible = false;
        self.tail_char = None;
    }
    pub fn line_start(&mut self) {
        self.reset();
        self.engine.arm_capitalization();
    }
    fn result(&mut self, result: ImeResult) -> Result<Option<Edit>, &'static str> {
        if result.action == Action::None {
            return Ok(None);
        }
        let count = self.rendered.chars().count();
        if result.backspace > count
            || result.backspace > MAX_BACKSPACES
            || result.output.encode_utf16().count() > MAX_UNITS
            || result.output.contains('\0')
        {
            self.reset();
            return Err("unowned_or_oversized_edit");
        }
        let keep = count - result.backspace;
        self.rendered.truncate(
            self.rendered
                .char_indices()
                .nth(keep)
                .map_or(self.rendered.len(), |(i, _)| i),
        );
        self.rendered.push_str(&result.output);
        Ok(Some(Edit {
            backspaces: result.backspace,
            text: result.output,
        }))
    }
    pub fn feed(
        &mut self,
        ch: char,
        source: KeySource,
        vi: bool,
    ) -> Result<Option<Edit>, &'static str> {
        self.space_eligible = false;
        if !ch.is_ascii() || ch.is_control() {
            self.reset();
            return Ok(None);
        }
        if self.bypass_word {
            if ch.is_whitespace() {
                self.reset();
            }
            return Ok(None);
        }
        if self.rendered.chars().count() >= MAX_BACKSPACES {
            self.reset();
            self.bypass_word = !ch.is_whitespace();
            return Ok(None);
        }
        self.completed = None;
        let previous = self.tail_char;
        self.engine.set_enabled(vi);
        let result = self.engine.process_key(ch, source);
        let action = result.action;
        let edit = self.result(result)?;
        if action == Action::None {
            self.rendered.push(ch);
        }
        self.tail_char = self.rendered.chars().last();
        if self.engine.buffer().is_empty() {
            if ch == ' ' && self.rendered.ends_with(' ') {
                self.space_eligible = self
                    .rendered
                    .chars()
                    .rev()
                    .nth(1)
                    .or_else(|| (action == Action::None).then_some(previous).flatten())
                    .is_some_and(char::is_alphanumeric);
                let word = self.rendered.trim_end_matches(' ');
                if !word.is_empty()
                    && word.chars().count() <= MAX_BACKSPACES
                    && word.chars().all(char::is_alphabetic)
                {
                    self.completed = Some(word.to_owned());
                }
            }
            self.rendered.clear();
        }
        Ok(edit)
    }
    pub fn space_eligible(&self) -> bool {
        self.space_eligible
    }
    pub fn period_committed(&mut self) {
        self.reset();
        // Keep the engine's sentence-capitalization trace consistent with the
        // transform just emitted; these known punctuation keys cannot expand a macro.
        self.engine.process_char('.');
        self.engine.process_char(' ');
    }
    pub fn backspace(&mut self, retone: bool) -> Result<Option<Edit>, &'static str> {
        if !retone || self.bypass_word {
            self.reset();
            return Ok(None);
        }
        if let Some(word) = self.completed.take() {
            if self.engine.adopt(&word) {
                self.rendered = word;
                self.tail_char = self.rendered.chars().last();
            } else {
                self.reset();
            }
            return Ok(None);
        }
        if self.rendered.is_empty() {
            self.reset();
            return Ok(None);
        }
        let result = self.engine.on_backspace();
        if result.action != Action::None {
            return self.result(result);
        }
        self.rendered.pop();
        self.tail_char = self.rendered.chars().last();
        Ok(None)
    }
    pub fn flip(&mut self) -> Result<Option<Edit>, &'static str> {
        if self.rendered.is_empty() {
            return Ok(None);
        }
        let result = self.engine.flip_composing();
        self.result(result)
    }
}
