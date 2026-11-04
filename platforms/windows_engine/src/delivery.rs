//! Bounded, prebuilt SendInput batches: no lead glyph, extra deletion or sleep.

pub const MAX_BACKSPACES: usize = 128;
pub const MAX_UNITS: usize = 8192;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Edit {
    pub backspaces: usize,
    pub text: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Packet {
    Key { vk: u16, up: bool },
    Unicode { unit: u16, up: bool },
}

#[derive(Clone, Debug)]
pub struct Batch {
    pub packets: Vec<Packet>,
    /// Only modifiers temporarily released by this batch may be restored.
    restore: Vec<u16>,
}

impl Batch {
    /// `held_shift` is a physical snapshot on the callback thread. `pending_up`
    /// is a modifier release currently in the hook, not yet visible to the client.
    pub fn prepare(
        edit: &Edit,
        held_shift: &[u16],
        pending_up: Option<u16>,
    ) -> Result<Self, &'static str> {
        if edit.backspaces > MAX_BACKSPACES
            || edit.text.encode_utf16().count() > MAX_UNITS
            || edit.text.contains('\0')
            || held_shift.iter().any(|k| !matches!(k, 0xa0 | 0xa1))
        {
            return Err("invalid_delivery_plan");
        }
        let mut packets =
            Vec::with_capacity(2 * (edit.backspaces + edit.text.encode_utf16().count() + 3));
        if let Some(vk) = pending_up {
            packets.push(Packet::Key { vk, up: true });
        }
        let mut restore = Vec::new();
        for &vk in held_shift {
            if !restore.contains(&vk) {
                restore.push(vk);
                packets.push(Packet::Key { vk, up: true });
            }
        }
        for _ in 0..edit.backspaces {
            packets.extend([
                Packet::Key { vk: 8, up: false },
                Packet::Key { vk: 8, up: true },
            ]);
        }
        for unit in edit.text.encode_utf16() {
            packets.extend([
                Packet::Unicode { unit, up: false },
                Packet::Unicode { unit, up: true },
            ]);
        }
        for &vk in &restore {
            packets.push(Packet::Key { vk, up: false });
        }
        Ok(Self { packets, restore })
    }

    /// Repair unmatched synthetic presses and our temporary Shift releases only.
    /// Never replays deletion/text. Prefix is the exact SendInput accepted count.
    pub fn repair(&self, accepted: usize) -> Vec<Packet> {
        let accepted = accepted.min(self.packets.len());
        let mut open = Vec::new();
        for &p in &self.packets[..accepted] {
            let (id, up) = match p {
                Packet::Key { vk, up } => ((false, vk), up),
                Packet::Unicode { unit, up } => ((true, unit), up),
            };
            if self.restore.contains(&id.1) && !id.0 {
                continue;
            }
            if up {
                if let Some(i) = open.iter().position(|x| *x == id) {
                    open.remove(i);
                }
            } else {
                open.push(id);
            }
        }
        let mut repair: Vec<_> = open
            .into_iter()
            .map(|(unicode, value)| {
                if unicode {
                    Packet::Unicode {
                        unit: value,
                        up: true,
                    }
                } else {
                    Packet::Key {
                        vk: value,
                        up: true,
                    }
                }
            })
            .collect();
        for &vk in &self.restore {
            let released = self.packets[..accepted].contains(&Packet::Key { vk, up: true });
            let restored = self.packets[..accepted].contains(&Packet::Key { vk, up: false });
            if released && !restored {
                repair.push(Packet::Key { vk, up: false });
            }
        }
        repair
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Receipt {
    pub expected: usize,
    pub accepted: usize,
    pub os_error: u32,
    pub repair: Option<RepairReceipt>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RepairReceipt {
    pub expected: usize,
    pub accepted: usize,
    pub os_error: u32,
}
impl Receipt {
    pub fn complete(self) -> bool {
        self.expected == self.accepted
    }
}
