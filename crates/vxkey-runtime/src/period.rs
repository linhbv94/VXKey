/// Decision-only model. Native adapter must verify the actual UTF-16 replacement
/// range and atomic edit/Undo support before consuming the second Space.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum NativePeriod {
    Enabled,
    Disabled,
    Unknown,
    NotApplicable,
}
#[derive(Clone, Copy, Debug)]
pub struct SpaceContext {
    pub focus_generation: u64,
    pub caret_utf16: usize,
    pub edit_revision: u64,
    pub unmodified: bool,
    pub repeat: bool,
    pub eligible_prefix: bool,
    pub safe: bool,
}
#[derive(Default)]
pub struct PeriodGesture {
    first: Option<(u64, SpaceContext)>,
}
impl PeriodGesture {
    pub fn cancel(&mut self) {
        self.first = None;
    }
    pub fn space(
        &mut self,
        now_ms: u64,
        c: SpaceContext,
        enabled: bool,
        native: NativePeriod,
    ) -> bool {
        if !enabled
            || matches!(native, NativePeriod::Enabled | NativePeriod::Unknown)
            || !c.safe
            || !c.unmodified
            || c.repeat
        {
            self.cancel();
            return false;
        }
        if let Some((time, first)) = self.first.take() {
            // Exactly one owned first-space insert must separate snapshots.
            return now_ms.checked_sub(time).is_some_and(|d| d <= 500)
                && c.focus_generation == first.focus_generation
                && c.caret_utf16 == first.caret_utf16 + 1
                && c.edit_revision == first.edit_revision + 1;
        }
        if c.eligible_prefix {
            self.first = Some((now_ms, c));
        }
        false
    }
}
