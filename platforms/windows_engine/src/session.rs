//! Serialized native state. No disk, clipboard, IPC, UIA, wait or UI locks here.
use crate::{
    config::{self, ALT, CTRL, META, SHIFT, Settings},
    delivery::{Batch, Edit, Receipt},
    stream::Stream,
};
use funput_engine::KeySource;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Access {
    Allowed,
    Elevated,
    Secure,
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Target {
    pub window: usize,
    pub focus: usize,
    pub pid: u32,
    pub process_start: u64,
    pub layout: usize,
    pub app_id: String,
    pub access: Access,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Origin {
    Physical,
    Own,
    OtherInjected,
}

#[derive(Clone, Copy, Debug)]
pub struct Key {
    pub vk: u16,
    pub down: bool,
    pub ch: Option<char>,
    pub source: KeySource,
    pub origin: Origin,
    pub time_ms: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Disposition {
    Pass,
    Consume,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fault {
    Delivery,
    InvalidEdit,
    CallbackPanic,
}

#[derive(Clone, Debug, Default)]
pub struct Diagnostics {
    pub physical_events: u64,
    pub replacements: u64,
    pub accepted_inputs: u64,
    pub failed_batches: u64,
    pub failed_repairs: u64,
    pub target_resets: u64,
    pub config_revision: u64,
    pub running: bool,
    pub fault: Option<Fault>,
    pub last_os_error: u32,
    pub last_repair_os_error: u32,
    pub last_callback_us: u64,
    pub max_callback_us: u64,
}

pub enum Delivery {
    Cancelled,
    Sent(Receipt),
}

pub struct Session {
    settings: Settings,
    stream: Stream,
    target: Option<Target>,
    pressed: [bool; 256],
    visible: [bool; 256],
    gesture_armed: bool,
    gesture_cancelled: bool,
    pending_flip: bool,
    first_space: Option<u64>,
    diagnostics: Diagnostics,
    /// In-memory hotkey changes for the owner to persist outside the callback.
    mode_revision: u64,
    probe_remaining: u8,
}

impl Session {
    pub fn new(settings: Settings) -> Result<Self, &'static str> {
        settings.validate()?;
        let stream = Stream::new(&settings);
        let diagnostics = Diagnostics {
            config_revision: settings.revision,
            ..Diagnostics::default()
        };
        Ok(Self {
            settings,
            stream,
            target: None,
            pressed: [false; 256],
            visible: [false; 256],
            gesture_armed: false,
            gesture_cancelled: false,
            pending_flip: false,
            first_space: None,
            diagnostics,
            mode_revision: 0,
            probe_remaining: 0,
        })
    }
    pub fn configure(&mut self, settings: Settings) -> Result<(), &'static str> {
        settings.validate()?;
        if settings.revision <= self.settings.revision {
            return Err("stale_config_revision");
        }
        self.stream = Stream::new(&settings);
        self.diagnostics.config_revision = settings.revision;
        self.settings = settings;
        self.reset();
        Ok(())
    }
    pub fn diagnostics(&self) -> Diagnostics {
        self.diagnostics.clone()
    }
    pub fn settings(&self) -> &Settings {
        &self.settings
    }
    pub fn mode_revision(&self) -> u64 {
        self.mode_revision
    }
    pub fn target(&self) -> Option<&Target> {
        self.target.as_ref()
    }
    pub fn resume(&mut self) {
        self.probe_remaining = 0;
        self.reset();
        self.diagnostics.fault = None;
        self.diagnostics.running = true;
    }
    pub fn pause(&mut self) {
        self.probe_remaining = 0;
        self.diagnostics.running = false;
        self.reset();
    }
    /// Control/UI disconnect intentionally does nothing. The owner owns lifetime.
    pub fn control_disconnected(&mut self) {}
    pub fn fault(&mut self, fault: Fault) {
        self.probe_remaining = 0;
        self.diagnostics.fault = Some(fault);
        self.diagnostics.running = false;
        self.reset();
    }
    pub fn callback_time(&mut self, micros: u64) {
        self.diagnostics.last_callback_us = micros;
        self.diagnostics.max_callback_us = self.diagnostics.max_callback_us.max(micros);
    }
    /// Explicit, bounded delivery-only probe. No Funput key processing. The
    /// native owner also expires this mode after 30 seconds, including idle.
    pub fn begin_probe(&mut self) {
        self.resume();
        self.probe_remaining = 12;
    }
    pub fn probe_active(&self) -> bool {
        self.probe_remaining > 0
    }
    pub fn set_target(&mut self, target: Option<Target>) {
        if self.target != target {
            self.target = target;
            self.diagnostics.target_resets = self.diagnostics.target_resets.saturating_add(1);
            self.reset();
        }
    }
    /// Mouse click, focus/caret change, paste, layout change or unknown boundary.
    pub fn reset(&mut self) {
        self.stream.reset();
        self.first_space = None;
        self.pending_flip = false;
        self.gesture_armed = false;
        self.gesture_cancelled = self.modifiers() != 0;
    }
    /// Initial physical state must be sampled before installing the hook. No
    /// phantom gesture is armed if the engine starts with modifiers already held.
    pub fn seed_pressed(&mut self, mut keys: [bool; 256]) {
        // Async snapshots include generic modifier aliases and mouse buttons.
        // Only sided keyboard events release the corresponding entries later.
        for vk in [0, 1, 2, 3, 4, 5, 6, 7, 0x10, 0x11, 0x12] {
            keys[vk] = false;
        }
        self.pressed = keys;
        self.visible = keys;
    }
    pub fn physical_down(&self, vk: u16) -> bool {
        self.pressed.get(vk as usize).copied().unwrap_or(false)
    }
    pub fn modifiers(&self) -> u8 {
        self.pressed
            .iter()
            .enumerate()
            .filter(|(_, down)| **down)
            .fold(0, |mask, (vk, _)| mask | config::modifier(vk as u16))
    }
    pub fn keyboard_state(&self, caps: bool) -> [u8; 256] {
        let mut state = self.pressed.map(|v| if v { 0x80 } else { 0 });
        for (vk, mask) in [(0x10, SHIFT), (0x11, CTRL), (0x12, ALT)] {
            state[vk] = if self.modifiers() & mask != 0 {
                0x80
            } else {
                0
            };
        }
        state[0x14] = u8::from(caps);
        state
    }
    fn allowed(&self) -> bool {
        self.diagnostics.running
            && self.diagnostics.fault.is_none()
            && self.target.as_ref().is_some_and(|t| {
                t.access == Access::Allowed && !self.settings.excluded_apps.contains(&t.app_id)
            })
    }
    fn vi(&self) -> bool {
        if self.settings.remember_per_app {
            if let Some(on) = self
                .target
                .as_ref()
                .and_then(|t| self.settings.language_memory.get(&t.app_id))
            {
                return *on;
            }
        }
        self.settings.vietnamese
    }
    fn toggle(&mut self) {
        let on = !self.vi();
        self.settings.vietnamese = on;
        if self.settings.remember_per_app {
            if let Some(t) = &self.target {
                self.settings.language_memory.insert(t.app_id.clone(), on);
            }
        }
        self.mode_revision = self.mode_revision.saturating_add(1);
        self.reset();
    }
    fn emit(
        &mut self,
        edit: Result<Option<Edit>, &'static str>,
        pending_up: Option<u16>,
        deliver: &mut impl FnMut(&Batch) -> Delivery,
    ) -> Disposition {
        let edit = match edit {
            Ok(Some(e)) => e,
            Ok(None) => return Disposition::Pass,
            Err(_) => {
                self.fault(Fault::InvalidEdit);
                return Disposition::Pass;
            }
        };
        let shifts: Vec<_> = [0xa0, 0xa1]
            .into_iter()
            .filter(|k| self.pressed[*k as usize])
            .collect();
        let batch = match Batch::prepare(&edit, &shifts, pending_up) {
            Ok(batch) => batch,
            Err(_) => {
                self.fault(Fault::InvalidEdit);
                return Disposition::Pass;
            }
        };
        // A valid empty edit may still swallow a duplicate tone/shape key.
        if batch.packets.is_empty() {
            return Disposition::Consume;
        }
        match deliver(&batch) {
            Delivery::Cancelled => {
                self.reset();
                Disposition::Pass
            }
            Delivery::Sent(receipt) => {
                if receipt.repair.is_some_and(|r| r.accepted != r.expected) {
                    self.diagnostics.failed_repairs =
                        self.diagnostics.failed_repairs.saturating_add(1);
                    self.diagnostics.last_repair_os_error =
                        receipt.repair.map_or(0, |r| r.os_error);
                }
                self.diagnostics.accepted_inputs = self
                    .diagnostics
                    .accepted_inputs
                    .saturating_add(receipt.accepted as u64);
                if receipt.expected == batch.packets.len()
                    && receipt.complete()
                    && receipt.repair.is_none_or(|r| r.accepted == r.expected)
                {
                    self.diagnostics.replacements = self.diagnostics.replacements.saturating_add(1);
                    Disposition::Consume
                } else {
                    self.diagnostics.failed_batches =
                        self.diagnostics.failed_batches.saturating_add(1);
                    self.diagnostics.last_os_error = receipt.os_error;
                    self.fault(Fault::Delivery);
                    // Zero input accepted: original key may safely pass. Partial:
                    // never replay text or let the trigger append a second edit.
                    if receipt.accepted == 0 {
                        Disposition::Pass
                    } else {
                        Disposition::Consume
                    }
                }
            }
        }
    }
    pub fn handle(
        &mut self,
        mut key: Key,
        mut deliver: impl FnMut(&Batch) -> Delivery,
    ) -> Disposition {
        // Native uses scan/extended flags to supply sided keys. Generic external
        // callers still get a balanced left-side alias.
        key.vk = match key.vk {
            0x10 => 0xa0,
            0x11 => 0xa2,
            0x12 => 0xa4,
            vk => vk,
        };
        if key.origin == Origin::Own {
            return Disposition::Pass;
        }
        if key.origin == Origin::OtherInjected {
            self.reset();
            // Remappers/OSK may supply modifiers. They never arm our shortcuts,
            // but their state must still prevent composition under Ctrl/Alt/Win.
            if key.vk <= 255 && config::is_modifier(key.vk) {
                self.pressed[key.vk as usize] = key.down;
                self.visible[key.vk as usize] = key.down;
            }
            return Disposition::Pass;
        }
        if key.vk > 255 {
            self.reset();
            return Disposition::Pass;
        }
        self.diagnostics.physical_events = self.diagnostics.physical_events.saturating_add(1);
        let i = key.vk as usize;
        let repeat = key.down && self.pressed[i];
        if key.down && !repeat {
            self.visible[i] = false;
        }
        let visible_up = self.visible[i];
        let was_pressed = self.pressed[i];
        self.pressed[i] = key.down;
        let outcome = self.decide(key, repeat, &mut deliver);
        if key.down {
            if outcome == Disposition::Pass {
                self.visible[i] = true;
            }
            outcome
        } else {
            self.visible[i] = false;
            // Preserve the up of a down already passed (including autorepeat).
            if !was_pressed || visible_up {
                Disposition::Pass
            } else {
                Disposition::Consume
            }
        }
    }
    fn decide(
        &mut self,
        key: Key,
        repeat: bool,
        deliver: &mut impl FnMut(&Batch) -> Delivery,
    ) -> Disposition {
        // Hard app/secure/integrity/pause policy BEFORE every shortcut/transform.
        if !self.allowed() {
            self.reset();
            return Disposition::Pass;
        }
        let mods = self.modifiers();
        if self.probe_remaining > 0 {
            if !key.down || config::is_modifier(key.vk) || mods != 0 {
                return Disposition::Pass;
            }
            self.probe_remaining -= 1;
            let result = match key.ch {
                Some('z') => Disposition::Consume,
                Some('x') => self.emit(
                    Ok(Some(Edit {
                        backspaces: 0,
                        text: "X".into(),
                    })),
                    None,
                    deliver,
                ),
                _ => Disposition::Pass,
            };
            if self.probe_remaining == 0 {
                self.pause();
            }
            return result;
        }
        if let Some(binding) = self.settings.toggle.clone() {
            if let Some(vk) = binding.key {
                if key.down && key.vk == vk && mods == binding.modifiers {
                    if !repeat {
                        self.toggle();
                    }
                    return Disposition::Consume;
                }
            } else {
                if key.down && (!config::is_modifier(key.vk) || mods & !binding.modifiers != 0) {
                    self.gesture_cancelled = true;
                }
                if key.down
                    && !repeat
                    && config::is_modifier(key.vk)
                    && mods == binding.modifiers
                    && !self.gesture_cancelled
                    && !self
                        .pressed
                        .iter()
                        .enumerate()
                        .any(|(vk, down)| *down && !config::is_modifier(vk as u16))
                {
                    self.gesture_armed = true;
                }
                if !key.down && mods == 0 {
                    let toggle = self.gesture_armed && !self.gesture_cancelled;
                    self.gesture_armed = false;
                    self.gesture_cancelled = false;
                    if toggle {
                        self.toggle();
                        return Disposition::Pass;
                    }
                }
            }
        }
        if self.pending_flip {
            if key.down && self.settings.flip.as_ref().and_then(|b| b.key) == Some(key.vk) {
                return Disposition::Consume;
            }
            if key.down
                && !config::is_modifier(key.vk)
                && self.settings.flip.as_ref().and_then(|b| b.key) != Some(key.vk)
            {
                self.reset();
            } else if !key.down && mods == 0 {
                self.pending_flip = false;
                let edit = self.stream.flip();
                let up = config::is_modifier(key.vk).then_some(key.vk);
                return self.emit(edit, up, deliver);
            } else {
                return Disposition::Pass;
            }
        }
        if self.vi()
            && self
                .settings
                .flip
                .as_ref()
                .is_some_and(|b| b.key == Some(key.vk) && b.modifiers == mods)
            && key.down
        {
            self.pending_flip = true;
            self.first_space = None;
            return Disposition::Consume;
        }
        if !key.down {
            return Disposition::Pass;
        }
        if config::is_modifier(key.vk) {
            // Do not destroy composition before a configured flip. Other
            // modifier shortcuts reset on the first non-modifier key below.
            self.first_space = None;
            return Disposition::Pass;
        }
        if mods & (CTRL | ALT | META) != 0 {
            self.reset();
            return Disposition::Pass;
        }
        if key.vk == 8 {
            self.first_space = None;
            let edit = self.stream.backspace(self.settings.retone);
            return self.emit(edit, None, deliver);
        }
        if matches!(key.vk, 9 | 13 | 27 | 0x21..=0x28 | 0x2d..=0x2e | 0x70..=0x87) {
            self.reset();
            if key.vk == 13 {
                self.stream.line_start();
            }
            return Disposition::Pass;
        }
        let Some(ch) = key.ch else {
            self.reset();
            return Disposition::Pass;
        };
        if self.settings.double_space_period && ch == ' ' && mods == 0 && !repeat {
            if self
                .first_space
                .take()
                .is_some_and(|time| key.time_ms.checked_sub(time).is_some_and(|d| d <= 500))
            {
                let outcome = self.emit(
                    Ok(Some(Edit {
                        backspaces: 1,
                        text: ". ".into(),
                    })),
                    None,
                    deliver,
                );
                if outcome == Disposition::Consume && self.diagnostics.running {
                    self.stream.period_committed();
                }
                return outcome;
            }
        } else {
            self.first_space = None;
        }
        let edit = self.stream.feed(ch, key.source, self.vi());
        let eligible = self.stream.space_eligible();
        let replacing = edit.as_ref().is_ok_and(Option::is_some);
        let outcome = self.emit(edit, None, deliver);
        if self.diagnostics.running
            && self.settings.double_space_period
            && ch == ' '
            && mods == 0
            && !repeat
            && eligible
            && (!replacing || outcome == Disposition::Consume)
        {
            self.first_space = Some(key.time_ms);
        }
        outcome
    }
}
