#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Health {
    Healthy,
    Degraded,
    Suspended,
    Unresponsive,
}
/// Heartbeat is supplied by the actual input event loop, not a live background thread.
pub struct Recovery {
    pub health: Health,
    last_sequence: u64,
    last_progress_ms: u64,
    pub incident: Option<&'static str>,
}
impl Default for Recovery {
    fn default() -> Self {
        Self {
            health: Health::Suspended,
            last_sequence: 0,
            last_progress_ms: 0,
            incident: Some("unverified_start"),
        }
    }
}
impl Recovery {
    pub fn acknowledge(&mut self, sequence: u64, now_ms: u64) {
        if sequence > self.last_sequence {
            self.last_sequence = sequence;
            self.last_progress_ms = now_ms;
        }
    }
    pub fn observe(&mut self, now_ms: u64) {
        if self.health == Health::Healthy && now_ms.saturating_sub(self.last_progress_ms) >= 3000 {
            self.health = Health::Unresponsive;
            self.incident = Some("event_loop_stalled");
        }
    }
    pub fn fault(&mut self, reason: &'static str) {
        self.health = Health::Suspended;
        self.incident = Some(reason);
    }
    pub fn resume(&mut self, healthy_ack: bool, now_ms: u64) -> bool {
        if !healthy_ack {
            return false;
        }
        self.health = Health::Healthy;
        self.incident = None;
        self.last_progress_ms = now_ms;
        true
    }
    /// Never replay failed injections: uncertain delivery is a safety suspension.
    pub fn injection_result(&mut self, expected: usize, actual: usize) -> bool {
        if expected == actual {
            return true;
        }
        self.fault(if actual == 0 {
            "injection_failed"
        } else {
            "injection_partial"
        });
        false
    }
}
