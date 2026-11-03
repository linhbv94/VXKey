//! Native-only protocol. No typed keys, surrounding text, clipboard or UI data.
use crate::config::{self, Config, Macros, Platform, Shortcut, Typing};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    io::{Read, Write},
    time::{Duration, Instant},
};
pub const FRAME_LIMIT: usize = 2 * 1024 * 1024;
pub const HEALTH_TIMEOUT: Duration = Duration::from_secs(3);
macro_rules! dto {
    ($name:ident { $($field:ident : $ty:ty),* $(,)? }) => {
        #[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
        #[serde(deny_unknown_fields)]
        pub struct $name { $(pub $field: $ty),* }
    };
}
dto!(NativeConfig { revision: String, typing: Typing, macros: Macros,
    remember_per_app: bool, language_memory: BTreeMap<String,String>, input_policies: BTreeMap<String,bool>,
    toggle: Option<Shortcut>, flip: Option<Shortcut> });
impl NativeConfig {
    pub fn from_config(c: &Config) -> Self {
        Self {
            revision: c.revision.clone(),
            typing: c.typing.clone(),
            macros: c.macros.clone(),
            remember_per_app: c.apps.remember_per_app,
            language_memory: c.apps.language_memory.clone(),
            input_policies: c
                .apps
                .policies
                .iter()
                .map(|(k, v)| (k.clone(), v.input_bypass))
                .collect(),
            toggle: c.shortcuts.toggle.clone(),
            flip: c.shortcuts.flip.clone(),
        }
    }
    pub fn validate(&self) -> Result<(), String> {
        let mut c = Config::fresh(Platform::Macos);
        c.revision = self.revision.clone();
        c.typing = self.typing.clone();
        c.macros = self.macros.clone();
        c.apps.remember_per_app = self.remember_per_app;
        c.apps.language_memory = self.language_memory.clone();
        c.apps.policies = self
            .input_policies
            .iter()
            .map(|(k, v)| {
                (
                    k.clone(),
                    config::AppPolicy {
                        input_bypass: *v,
                        clipboard_blocked: false,
                    },
                )
            })
            .collect();
        c.shortcuts.toggle = self.toggle.clone();
        c.shortcuts.flip = self.flip.clone();
        // Match the physical codes supported by this Mac adapter. A syntactically
        // valid future Windows binding must not receive a false Mac apply ACK.
        for binding in [&self.toggle, &self.flip].into_iter().flatten() {
            if let Shortcut::KeyChord { key_code, .. } = binding {
                let letter = key_code
                    .strip_prefix("Key")
                    .is_some_and(|s| s.len() == 1 && s.bytes().all(|b| b.is_ascii_uppercase()));
                let digit = key_code
                    .strip_prefix("Digit")
                    .is_some_and(|s| s.len() == 1 && s.bytes().all(|b| b.is_ascii_digit()));
                let function = key_code.strip_prefix('F').is_some_and(|s| {
                    s.parse::<u8>().is_ok_and(|n| (1..=12).contains(&n)) && !s.starts_with('0')
                });
                let named = [
                    "Equal",
                    "Minus",
                    "BracketRight",
                    "BracketLeft",
                    "Enter",
                    "Quote",
                    "Semicolon",
                    "Backslash",
                    "Comma",
                    "Slash",
                    "Period",
                    "Tab",
                    "Space",
                    "Backquote",
                    "Backspace",
                    "Escape",
                    "Home",
                    "PageUp",
                    "Delete",
                    "End",
                    "PageDown",
                    "ArrowLeft",
                    "ArrowRight",
                    "ArrowDown",
                    "ArrowUp",
                ]
                .contains(&key_code.as_str());
                if !(letter || digit || function || named) {
                    return Err("unsupported_shortcut".into());
                }
            }
        }
        c.validate()
    }
}
dto!(Target {
    app_id: String,
    pid: i32,
    engine_session_id: String,
    focus_generation: String
});
impl Target {
    pub fn valid(&self) -> bool {
        config::valid_app_id(&self.app_id)
            && !self.app_id.starts_with("com.vxkey.")
            && self.pid > 0
            && config::valid_uuid(&self.engine_session_id)
            && config::counter(&self.focus_generation).is_ok()
    }
}
dto!(ControlResult {
    id: String,
    outcome: String
});
dto!(MemoryDelta {
    app_id: String,
    mode: String,
    base_revision: String
});
dto!(DriverStatus {
    accessibility: bool,
    listen_access: bool,
    post_access: bool,
    attached: bool
});
dto!(DriverDiagnostics {
    seen: u32,
    bypassed: u32,
    missing_target: u32,
    unverified: u32,
    rewrites: u32,
    verified: u32,
    failed: u32
});
dto!(NativeStatus { diagnostics: Option<DriverDiagnostics>, driver: DriverStatus, engine_session_id: String, loop_sequence: String, active: bool, target: Option<Target>,
    composing: bool, mode: String, suspended: bool, applied_revision: String,
    result: Option<ControlResult>, memory: Option<MemoryDelta>, fault: Option<String> });
impl NativeStatus {
    pub fn validate(&self) -> Result<(), String> {
        if self.driver.attached
            && (!self.driver.accessibility || !self.driver.post_access || self.suspended)
        {
            return Err("invalid_driver_status".into());
        }
        if self.fault.as_ref().is_some_and(|f| {
            ![
                "engine_fault",
                "bridge_stale",
                "permission_required",
                "driver_fault",
            ]
            .contains(&f.as_str())
        }) {
            return Err("invalid_fault".into());
        }
        if !config::valid_uuid(&self.engine_session_id)
            || config::counter(&self.loop_sequence).is_err()
            || config::counter(&self.applied_revision).is_err()
            || (self.active && self.target.is_none())
            || !["vi", "en"].contains(&self.mode.as_str())
            || self
                .target
                .as_ref()
                .is_some_and(|t| !t.valid() || t.engine_session_id != self.engine_session_id)
        {
            return Err("invalid_status".into());
        }
        if self.result.as_ref().is_some_and(|r| {
            !config::valid_uuid(&r.id)
                || !["applied", "stale_target", "unavailable", "rejected"]
                    .contains(&r.outcome.as_str())
        }) {
            return Err("invalid_result".into());
        }
        if self.memory.as_ref().is_some_and(|m| {
            !config::valid_app_id(&m.app_id)
                || m.app_id.starts_with("com.vxkey.")
                || !["vi", "en"].contains(&m.mode.as_str())
                || config::counter(&m.base_revision).is_err()
        }) {
            return Err("invalid_memory".into());
        }
        Ok(())
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "action", rename_all = "snake_case", deny_unknown_fields)]
pub enum Control {
    RequestPermissions {
        id: String,
        engine_session_id: String,
    },
    SetLanguage {
        id: String,
        mode: String,
        target: Target,
        expected_loop_sequence: String,
    },
    Suspend {
        id: String,
        suspended: bool,
        engine_session_id: String,
    },
}
impl Control {
    pub fn validate(&self) -> Result<(), String> {
        let valid = match self {
            Self::SetLanguage {
                id,
                mode,
                target,
                expected_loop_sequence,
            } => {
                config::valid_uuid(id)
                    && ["vi", "en"].contains(&mode.as_str())
                    && target.valid()
                    && config::counter(expected_loop_sequence).is_ok()
            }
            Self::RequestPermissions {
                id,
                engine_session_id,
            }
            | Self::Suspend {
                id,
                engine_session_id,
                ..
            } => config::valid_uuid(id) && config::valid_uuid(engine_session_id),
        };
        if valid {
            Ok(())
        } else {
            Err("invalid_argument".into())
        }
    }
    pub fn id(&self) -> &str {
        match self {
            Self::SetLanguage { id, .. }
            | Self::Suspend { id, .. }
            | Self::RequestPermissions { id, .. } => id,
        }
    }
}
dto!(SyncPayload { config: Option<NativeConfig>, control: Option<Control>, latched: bool });
dto!(Handshake {
    component: String,
    app_version: String
});
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(
    tag = "type",
    content = "payload",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum Message {
    Handshake(Handshake),
    Status(NativeStatus),
    Sync(SyncPayload),
}
dto!(Frame {
    protocol_version: u32,
    request_id: String,
    session_id: String,
    sequence: String,
    message: Message
});
pub fn fresh_id() -> Result<String, String> {
    let mut b = [0u8; 16];
    getrandom::fill(&mut b).map_err(|_| "unavailable")?;
    b[6] = (b[6] & 15) | 64;
    b[8] = (b[8] & 63) | 128;
    Ok(format!(
        "{:08x}-{:04x}-{:04x}-{:04x}-{:012x}",
        u32::from_be_bytes(b[0..4].try_into().unwrap()),
        u16::from_be_bytes(b[4..6].try_into().unwrap()),
        u16::from_be_bytes(b[6..8].try_into().unwrap()),
        u16::from_be_bytes(b[8..10].try_into().unwrap()),
        u64::from_be_bytes([0, 0, b[10], b[11], b[12], b[13], b[14], b[15]])
    ))
}
pub struct Wire {
    pub session: String,
    sent: u64,
    received: u64,
}
impl Wire {
    pub fn new(session: String) -> Result<Self, String> {
        if !config::valid_uuid(&session) {
            return Err("invalid_session".into());
        }
        Ok(Self {
            session,
            sent: 0,
            received: 0,
        })
    }
    pub fn send(&mut self, stream: &mut impl Write, message: Message) -> Result<(), String> {
        self.sent = self.sent.checked_add(1).ok_or("sequence_overflow")?;
        let frame = Frame {
            protocol_version: 2,
            request_id: fresh_id()?,
            session_id: self.session.clone(),
            sequence: self.sent.to_string(),
            message,
        };
        let bytes = serde_json::to_vec(&frame).map_err(|_| "invalid_frame")?;
        if bytes.len() > FRAME_LIMIT {
            return Err("oversized_frame".into());
        }
        stream
            .write_all(&(bytes.len() as u32).to_be_bytes())
            .and_then(|_| stream.write_all(&bytes))
            .map_err(|_| "disconnected".into())
    }
    pub fn read(&mut self, stream: &mut impl Read) -> Result<Message, String> {
        let frame = read_frame(stream)?;
        if frame.session_id != self.session
            || config::counter(&frame.sequence)?
                != self.received.checked_add(1).ok_or("sequence_overflow")?
        {
            return Err("stale_session_or_sequence".into());
        }
        self.received += 1;
        Ok(frame.message)
    }
    /// Listener accepts exactly one sequence-1 IME handshake before reading data.
    pub fn accept(stream: &mut impl Read) -> Result<Self, String> {
        let frame = read_frame(stream)?;
        if frame.sequence != "1"
            || frame.message
                != Message::Handshake(Handshake {
                    component: "engine".into(),
                    app_version: env!("CARGO_PKG_VERSION").into(),
                })
        {
            return Err("incompatible_protocol".into());
        }
        let mut wire = Self::new(frame.session_id)?;
        wire.received = 1;
        Ok(wire)
    }
}
fn read_frame(stream: &mut impl Read) -> Result<Frame, String> {
    let mut head = [0; 4];
    stream.read_exact(&mut head).map_err(|_| "disconnected")?;
    let length = u32::from_be_bytes(head) as usize;
    if length == 0 || length > FRAME_LIMIT {
        return Err("oversized_frame".into());
    }
    let mut bytes = vec![0; length];
    stream.read_exact(&mut bytes).map_err(|_| "disconnected")?;
    let frame: Frame = serde_json::from_slice(&bytes).map_err(|_| "invalid_frame")?;
    if frame.protocol_version != 2
        || !config::valid_uuid(&frame.request_id)
        || !config::valid_uuid(&frame.session_id)
    {
        return Err("incompatible_protocol".into());
    }
    Ok(frame)
}
#[derive(Default)]
pub struct Progress {
    sequence: u64,
    at: Option<Instant>,
}
impl Progress {
    pub fn observe(&mut self, sequence: &str, now: Instant) -> Result<(), String> {
        let sequence = config::counter(sequence)?;
        if sequence < self.sequence {
            return Err("sequence_regressed".into());
        }
        if sequence > self.sequence {
            self.sequence = sequence;
            self.at = Some(now);
        }
        if self
            .at
            .is_none_or(|at| now.duration_since(at) >= HEALTH_TIMEOUT)
        {
            return Err("event_loop_stalled".into());
        }
        Ok(())
    }
}
#[cfg(unix)]
pub mod local;
#[cfg(target_os = "macos")]
pub mod trust;

#[derive(Clone)]
pub struct Observation {
    pub status: NativeStatus,
    pub seen: Instant,
    pub session: String,
}
impl Observation {
    pub fn fresh(&self) -> bool {
        self.seen.elapsed() < HEALTH_TIMEOUT
    }
}
