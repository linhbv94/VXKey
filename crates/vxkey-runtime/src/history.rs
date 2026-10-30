use std::collections::VecDeque;
pub const MAX_ITEM: usize = 128 * 1024;
pub const MAX_TOTAL: usize = 10 * 1024 * 1024;
#[derive(Clone, Debug)]
pub struct CaptureMeta {
    pub count: u64,
    pub after_count: u64,
    pub text_only: bool,
    pub concealed: bool,
    pub transient: bool,
    pub generated: bool,
    pub own_write: bool,
    pub remote: bool,
    pub source_known: bool,
    pub source_blocked: bool,
}
#[derive(Clone, Debug)]
pub struct Entry {
    pub id: String,
    pub text: String,
    pub created_ms: u64,
    pub expires_ms: u64,
}
/// Session-only store. Encryption is deliberately not faked with plaintext disk storage.
pub struct History {
    entries: VecDeque<Entry>,
    pub revision: u64,
    pub enabled: bool,
    pub paused: bool,
    pub locked: bool,
    last_count: u64,
    max_entries: usize,
    ttl_ms: u64,
    next_id: u64,
}
impl Default for History {
    fn default() -> Self {
        Self {
            entries: VecDeque::new(),
            revision: 0,
            enabled: false,
            paused: false,
            locked: false,
            last_count: 0,
            max_entries: 200,
            ttl_ms: 7 * 86400000,
            next_id: 1,
        }
    }
}
impl History {
    pub fn baseline(&mut self, count: u64) {
        self.last_count = count;
    }
    pub fn set_limits(&mut self, n: usize, days: u32) -> Result<(), &'static str> {
        if !(1..=200).contains(&n) || !(1..=7).contains(&days) {
            return Err("invalid_argument");
        }
        self.max_entries = n;
        self.ttl_ms = days as u64 * 86400000;
        Ok(())
    }
    pub fn eligible(&self, m: &CaptureMeta) -> bool {
        self.enabled
            && !self.paused
            && !self.locked
            && m.count != self.last_count
            && m.count == m.after_count
            && m.text_only
            && !m.concealed
            && !m.transient
            && !m.generated
            && !m.own_write
            && !m.remote
            && m.source_known
            && !m.source_blocked
    }
    /// read_payload is called only after metadata/source policy passed.
    pub fn capture(
        &mut self,
        m: CaptureMeta,
        now: u64,
        read_payload: impl FnOnce() -> Option<(String, u64)>,
    ) -> bool {
        if !self.eligible(&m) {
            self.last_count = m.after_count;
            return false;
        }
        self.last_count = m.after_count;
        let Some((text, after_read_count)) = read_payload() else {
            return false;
        };
        if after_read_count != m.count {
            self.last_count = after_read_count;
            return false;
        }
        if text.is_empty() || text.len() > MAX_ITEM {
            return false;
        }
        self.prune(now);
        if self.entries.front().is_some_and(|e| e.text == text) {
            self.entries.pop_front();
        }
        let id = format!("00000000-0000-4000-8000-{:012x}", self.next_id);
        self.next_id += 1;
        self.entries.push_front(Entry {
            id,
            text,
            created_ms: now,
            expires_ms: now.saturating_add(self.ttl_ms),
        });
        while self.entries.len() > self.max_entries
            || self.entries.iter().map(|e| e.text.len()).sum::<usize>() > MAX_TOTAL
        {
            self.entries.pop_back();
        }
        self.revision += 1;
        true
    }
    pub fn prune(&mut self, now: u64) {
        let before = self.entries.len();
        self.entries.retain(|e| e.expires_ms > now);
        if self.entries.len() != before {
            self.revision += 1;
        }
    }
    pub fn search(&mut self, query: &str, offset: usize, limit: usize, now: u64) -> Vec<Entry> {
        self.prune(now);
        if self.locked {
            return vec![];
        }
        let q = query.to_lowercase();
        self.entries
            .iter()
            .filter(|e| e.text.to_lowercase().contains(&q))
            .skip(offset)
            .take(limit.min(50))
            .cloned()
            .collect()
    }
    pub fn get(&mut self, id: &str, revision: u64, now: u64) -> Result<Entry, &'static str> {
        self.prune(now);
        if self.locked {
            return Err("locked");
        }
        if self.revision != revision {
            return Err("revision_conflict");
        }
        self.entries
            .iter()
            .find(|e| e.id == id)
            .cloned()
            .ok_or("not_found")
    }
    pub fn delete(&mut self, id: &str, revision: u64) -> Result<bool, &'static str> {
        if self.revision != revision {
            return Err("revision_conflict");
        }
        let before = self.entries.len();
        self.entries.retain(|e| e.id != id);
        let removed = self.entries.len() != before;
        if removed {
            self.revision += 1;
        }
        Ok(removed)
    }
    pub fn clear(&mut self) -> usize {
        let n = self.entries.len();
        self.entries.clear();
        if n > 0 {
            self.revision += 1;
        }
        n
    }
    pub fn len(&self) -> usize {
        self.entries.len()
    }
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}
