use std::collections::BTreeSet;
/// Physical modifier IDs (left/right independent) avoid release imbalance.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Modifier {
    Control,
    Alt,
    Shift,
    Meta,
}
#[derive(Default)]
pub struct ModifierGesture {
    down: BTreeSet<(Modifier, bool)>,
    wanted: BTreeSet<Modifier>,
    armed: bool,
    cancelled: bool,
}
impl ModifierGesture {
    pub fn new(wanted: impl IntoIterator<Item = Modifier>) -> Self {
        Self {
            wanted: wanted.into_iter().collect(),
            ..Self::default()
        }
    }
    pub fn cancel(&mut self) {
        self.armed = false;
        self.cancelled = true;
    }
    pub fn reset(&mut self) {
        self.down.clear();
        self.armed = false;
        self.cancelled = false;
    }
    /// Return toggle intent only; caller MUST NOT swallow physical modifier events.
    pub fn changed(&mut self, modifier: Modifier, right: bool, pressed: bool) -> bool {
        let had_down = !self.down.is_empty();
        if pressed {
            self.down.insert((modifier, right));
        } else {
            self.down.remove(&(modifier, right));
        }
        let families: BTreeSet<_> = self.down.iter().map(|(m, _)| *m).collect();
        if !self.wanted.contains(&modifier) {
            self.cancel();
        }
        if families == self.wanted && !self.cancelled {
            self.armed = true;
        }
        if had_down && self.down.is_empty() {
            let fire = self.armed && !self.cancelled;
            self.reset();
            return fire;
        }
        false
    }
    pub fn non_modifier(&mut self) {
        if !self.down.is_empty() {
            self.cancel();
        }
    }
}
