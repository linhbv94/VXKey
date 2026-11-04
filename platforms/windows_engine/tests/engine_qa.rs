//! Portable client/transport fixtures. These do NOT run a Windows hook/SendInput.
use vxkey_windows_engine::{
    InputMethod, KeySource,
    config::{self, Binding, CTRL, META, Settings},
    delivery::{Batch, Edit, Packet, Receipt},
    session::*,
};

struct Editor {
    text: String,
    caret: usize,
    selection: Option<(usize, usize)>,
    down: Vec<(bool, u16)>,
}
impl Editor {
    fn new(text: &str, caret: usize) -> Self {
        Self {
            text: text.into(),
            caret,
            selection: None,
            down: Vec::new(),
        }
    }
    fn delete(&mut self) {
        if let Some((a, b)) = self.selection.take() {
            self.text.replace_range(a..b, "");
            self.caret = a;
        } else if self.caret > 0 {
            let prev = self.text[..self.caret]
                .char_indices()
                .next_back()
                .unwrap()
                .0;
            self.text.replace_range(prev..self.caret, "");
            self.caret = prev;
        }
    }
    fn insert(&mut self, text: &str) {
        if let Some((a, b)) = self.selection.take() {
            self.text.replace_range(a..b, "");
            self.caret = a;
        }
        self.text.insert_str(self.caret, text);
        self.caret += text.len();
    }
    fn batch(&mut self, batch: &Batch) -> Delivery {
        let mut units = Vec::new();
        for p in &batch.packets {
            match *p {
                Packet::Key { vk: 8, up: false } => self.delete(),
                Packet::Unicode { unit, up: false } => units.push(unit),
                _ => {}
            }
            let (id, up) = match *p {
                Packet::Key { vk, up } => ((false, vk), up),
                Packet::Unicode { unit, up } => ((true, unit), up),
            };
            if up {
                if let Some(i) = self.down.iter().position(|p| *p == id) {
                    self.down.remove(i);
                }
            } else {
                self.down.push(id);
            }
        }
        self.insert(&String::from_utf16(&units).unwrap());
        Delivery::Sent(Receipt {
            expected: batch.packets.len(),
            accepted: batch.packets.len(),
            os_error: 0,
            repair: None,
        })
    }
}

fn target(path: &str) -> Target {
    Target {
        window: 1,
        focus: 2,
        pid: 3,
        process_start: 4,
        layout: 0x409,
        app_id: config::canonical_app_id(path).unwrap(),
        access: Access::Allowed,
    }
}
fn setup(mut config: Settings) -> (Session, Editor) {
    config.revision = 1;
    let mut session = Session::new(config).unwrap();
    session.set_target(Some(target(r"C:\Apps\Editor.exe")));
    session.resume();
    (session, Editor::new("", 0))
}
fn key(vk: u16, down: bool, ch: Option<char>, time: u64) -> Key {
    Key {
        vk,
        down,
        ch,
        time_ms: time,
        source: KeySource::Standard,
        origin: Origin::Physical,
    }
}
fn press(session: &mut Session, editor: &mut Editor, k: Key) -> Disposition {
    let disposition = session.handle(k, |batch| editor.batch(batch));
    if disposition == Disposition::Pass && k.down {
        if k.vk == 8 {
            editor.delete();
        } else if !config::is_modifier(k.vk)
            && session.modifiers() & (config::CTRL | config::ALT | config::META) == 0
        {
            if let Some(ch) = k.ch {
                editor.insert(&ch.to_string());
            }
        }
    }
    disposition
}
fn type_text(session: &mut Session, editor: &mut Editor, text: &str) {
    for (n, ch) in text.chars().enumerate() {
        let vk = if ch.is_ascii_alphabetic() {
            ch.to_ascii_uppercase() as u16
        } else {
            ch as u16
        };
        press(session, editor, key(vk, true, Some(ch), n as u64 * 20));
        press(session, editor, key(vk, false, None, n as u64 * 20 + 1));
    }
}

#[test]
fn telex_user_regressions() {
    for (raw, expected) in [
        (
            "as af aa ngaan hangf casch cachs ddwojwc w",
            "á à â ngân hàng cách cách được ư",
        ),
        ("w thwr nhwng chwx vwaf casi", "ư thử nhưng chữ vừa cái"),
        ("ddwojwc mwojwt", "được mượt"),
        ("aaa", "aa"),
    ] {
        let (mut s, mut e) = setup(Settings::default());
        type_text(&mut s, &mut e, raw);
        assert_eq!(e.text, expected, "{raw}");
        assert!(e.down.is_empty());
        assert!(s.diagnostics().fault.is_none());
    }
}
#[test]
fn vni_and_numpad_are_different() {
    let mut settings = Settings::default();
    settings.engine.method = InputMethod::Vni;
    let (mut s, mut e) = setup(settings.clone());
    type_text(&mut s, &mut e, "a6");
    assert_eq!(e.text, "â");
    let (mut s, mut e) = setup(settings);
    type_text(&mut s, &mut e, "a");
    let mut k = key(0x66, true, Some('6'), 30);
    k.source = KeySource::Numpad;
    press(&mut s, &mut e, k);
    assert_eq!(e.text, "a6");
}
#[test]
fn macro_is_not_truncated_at_ffi_capacity() {
    let mut config = Settings::default();
    config.engine.shortcut_smart_case = false;
    let expansion = format!("{} 🧪", "Kiểm tra macro dài hơn hai mươi ký tự ".repeat(8));
    config.macros.push(("vxqa".into(), expansion.clone()));
    let (mut s, mut e) = setup(config);
    type_text(&mut s, &mut e, "vxqa ");
    assert_eq!(e.text, format!("{expansion} "));
    assert!(e.down.is_empty());
}
#[test]
fn first_word_after_every_observed_boundary() {
    let (mut s, mut e) = setup(Settings::default());
    for vk in [13, 0x25, 0x2e, 9] {
        type_text(&mut s, &mut e, "nhaaps");
        press(&mut s, &mut e, key(vk, true, None, 0));
        press(&mut s, &mut e, key(vk, false, None, 1));
        e.text.clear();
        e.caret = 0;
        type_text(&mut s, &mut e, "ddwojwc casi");
        assert_eq!(e.text, "được cái");
        s.reset();
        e.text.clear();
        e.caret = 0;
    }
    type_text(&mut s, &mut e, "nhaaps");
    press(&mut s, &mut e, key(0xa2, true, None, 0));
    press(&mut s, &mut e, key(0x41, true, Some('a'), 1));
    press(&mut s, &mut e, key(0x41, false, None, 2));
    press(&mut s, &mut e, key(0xa2, false, None, 3));
    e.selection = Some((0, e.text.len()));
    press(&mut s, &mut e, key(8, true, None, 4));
    press(&mut s, &mut e, key(8, false, None, 5));
    type_text(&mut s, &mut e, "ddwojwc casi");
    assert_eq!(e.text, "được cái");
}
#[test]
fn sentinel_and_selected_word_stay_bounded() {
    let (mut s, _) = setup(Settings::default());
    let mut e = Editor::new("PRE 🧪  SUF", "PRE 🧪 ".len());
    type_text(&mut s, &mut e, "cachs");
    assert_eq!(e.text, "PRE 🧪 cách SUF");
    s.reset();
    let a = "PRE 🧪 ".len();
    e.selection = Some((a, a + "cách".len()));
    type_text(&mut s, &mut e, "ddwojwc");
    assert_eq!(e.text, "PRE 🧪 được SUF");
}
#[test]
fn backspace_retones_and_z_removes_tone() {
    let (mut s, mut e) = setup(Settings::default());
    type_text(&mut s, &mut e, "chaof");
    press(&mut s, &mut e, key(8, true, None, 0));
    press(&mut s, &mut e, key(8, false, None, 1));
    type_text(&mut s, &mut e, "s");
    assert_eq!(e.text, "chá");
    s.reset();
    e = Editor::new("", 0);
    type_text(&mut s, &mut e, "phur ");
    press(&mut s, &mut e, key(8, true, None, 0));
    press(&mut s, &mut e, key(8, false, None, 1));
    type_text(&mut s, &mut e, "s");
    assert_eq!(e.text, "phú");
    type_text(&mut s, &mut e, "z");
    assert_eq!(e.text, "phu");
}
#[test]
fn windows_period_default_and_cancellation() {
    for (raw, expected) in [
        ("aa  ", "â. "),
        ("  ", "  "),
        ("a.  ", "a.  "),
        ("1  ", "1. "),
    ] {
        let (mut s, mut e) = setup(Settings::default());
        type_text(&mut s, &mut e, raw);
        assert_eq!(e.text, expected);
    }
    let (mut s, mut e) = setup(Settings::default());
    type_text(&mut s, &mut e, "a ");
    s.reset();
    type_text(&mut s, &mut e, " ");
    assert_eq!(e.text, "a  ");
}
#[test]
fn ctrl_win_release_toggle_and_third_key_cancel() {
    let (mut s, mut e) = setup(Settings::default());
    for (vk, down) in [(0xa2, true), (0x5b, true), (0xa2, false)] {
        press(&mut s, &mut e, key(vk, down, None, 0));
    }
    assert_eq!(s.mode_revision(), 0);
    press(&mut s, &mut e, key(0x5b, false, None, 1));
    assert_eq!(s.mode_revision(), 1);
    type_text(&mut s, &mut e, "aa");
    assert_eq!(e.text, "aa");
    for (vk, down) in [
        (0xa2, true),
        (0x5b, true),
        (0x56, true),
        (0x56, false),
        (0x5b, false),
        (0xa2, false),
    ] {
        assert_eq!(
            press(&mut s, &mut e, key(vk, down, None, 10)),
            Disposition::Pass
        );
    }
    assert_eq!(s.mode_revision(), 1);
}
#[test]
fn hard_exclusion_precedes_shortcuts_macro_and_period() {
    let mut config = Settings::default();
    config
        .excluded_apps
        .insert(config::canonical_app_id(r"C:\Apps\Editor.exe").unwrap());
    config.macros.push(("vxqa".into(), "EXPANDED".into()));
    let (mut s, mut e) = setup(config);
    type_text(&mut s, &mut e, "aa vxqa  ");
    assert_eq!(e.text, "aa vxqa  ");
    for (vk, down) in [(0xa2, true), (0x5b, true), (0x5b, false), (0xa2, false)] {
        press(&mut s, &mut e, key(vk, down, None, 0));
    }
    assert_eq!(s.mode_revision(), 0);
    s.set_target(Some(target(r"D:\Apps\Editor.exe")));
    type_text(&mut s, &mut e, "aa");
    assert!(e.text.ends_with("â"));
}
#[test]
fn secure_elevated_unknown_and_paused_never_inject() {
    for access in [Access::Secure, Access::Elevated, Access::Unknown] {
        let (mut s, mut e) = setup(Settings::default());
        let mut t = target(r"C:\Apps\Editor.exe");
        t.access = access;
        s.set_target(Some(t));
        type_text(&mut s, &mut e, "aa");
        assert_eq!(e.text, "aa");
        assert_eq!(s.diagnostics().replacements, 0);
    }
    let (mut s, mut e) = setup(Settings::default());
    s.pause();
    type_text(&mut s, &mut e, "aa");
    assert_eq!(e.text, "aa");
    s.control_disconnected();
    assert!(!s.diagnostics().running);
    s.resume();
    type_text(&mut s, &mut e, "aa");
    assert_eq!(e.text, "aaâ");
}
#[test]
fn control_loss_does_not_disable_running_session() {
    let (mut s, mut e) = setup(Settings::default());
    type_text(&mut s, &mut e, "ddw");
    s.control_disconnected();
    type_text(&mut s, &mut e, "ojwc");
    assert_eq!(e.text, "được");
}
#[test]
fn delivery_zero_passes_partial_consumes_and_fault_latches() {
    for accepted in [0, 1, 2, 3] {
        let (mut s, mut e) = setup(Settings::default());
        type_text(&mut s, &mut e, "a");
        let mut calls = 0;
        let d = s.handle(key(0x41, true, Some('a'), 0), |batch| {
            calls += 1;
            Delivery::Sent(Receipt {
                expected: batch.packets.len(),
                accepted,
                os_error: 5,
                repair: None,
            })
        });
        assert_eq!(
            d,
            if accepted == 0 {
                Disposition::Pass
            } else {
                Disposition::Consume
            }
        );
        assert_eq!(calls, 1);
        assert_eq!(s.diagnostics().fault, Some(Fault::Delivery));
        s.control_disconnected();
        s.handle(key(0x53, true, Some('s'), 1), |_| {
            panic!("fault must bypass")
        });
        let mut config = Settings::default();
        config.revision = 2;
        s.configure(config).unwrap();
        assert_eq!(s.diagnostics().fault, Some(Fault::Delivery));
    }
}
#[test]
fn target_race_cancels_before_any_delivery() {
    let (mut s, mut e) = setup(Settings::default());
    type_text(&mut s, &mut e, "a");
    assert_eq!(
        s.handle(key(0x41, true, Some('a'), 0), |_| Delivery::Cancelled),
        Disposition::Pass
    );
    assert!(s.diagnostics().fault.is_none());
    s.handle(key(0x41, false, None, 1), |_| panic!());
    type_text(&mut s, &mut e, "s");
    assert_eq!(e.text, "as");
}
#[test]
fn own_injection_never_reenters_and_third_party_resets() {
    let (mut s, mut e) = setup(Settings::default());
    type_text(&mut s, &mut e, "a");
    let mut k = key(0x41, true, Some('a'), 0);
    k.origin = Origin::Own;
    assert_eq!(s.handle(k, |_| panic!()), Disposition::Pass);
    type_text(&mut s, &mut e, "s");
    assert_eq!(e.text, "á");
    k.origin = Origin::OtherInjected;
    s.handle(k, |_| panic!());
    type_text(&mut s, &mut e, "s");
    assert_eq!(e.text, "ás");
}
#[test]
fn held_key_up_pair_survives_autorepeat_transform() {
    let (mut s, mut e) = setup(Settings::default());
    assert_eq!(
        press(&mut s, &mut e, key(0x41, true, Some('a'), 0)),
        Disposition::Pass
    );
    press(&mut s, &mut e, key(0x41, true, Some('a'), 1));
    assert_eq!(
        press(&mut s, &mut e, key(0x41, false, None, 2)),
        Disposition::Pass
    );
    assert!(e.down.is_empty());
}
#[test]
fn flip_waits_for_modifier_release_without_leaking_f5() {
    let mut config = Settings::default();
    config.flip = Some(Binding {
        modifiers: CTRL | config::ALT,
        key: Some(0x74),
    });
    let (mut s, mut e) = setup(config);
    type_text(&mut s, &mut e, "pass");
    for vk in [0xa2, 0xa4] {
        press(&mut s, &mut e, key(vk, true, None, 0));
    }
    assert_eq!(
        press(&mut s, &mut e, key(0x74, true, None, 1)),
        Disposition::Consume
    );
    assert_eq!(
        press(&mut s, &mut e, key(0x74, true, None, 2)),
        Disposition::Consume
    );
    assert_eq!(
        press(&mut s, &mut e, key(0x74, false, None, 3)),
        Disposition::Consume
    );
    press(&mut s, &mut e, key(0xa2, false, None, 4));
    press(&mut s, &mut e, key(0xa4, false, None, 5));
    assert_eq!(e.text, "pass");
}
#[test]
fn fresh_config_is_validated_and_revision_monotonic() {
    let (mut s, mut e) = setup(Settings::default());
    type_text(&mut s, &mut e, "a");
    let config = Settings::default();
    assert_eq!(s.configure(config), Err("stale_config_revision"));
    type_text(&mut s, &mut e, "s");
    assert_eq!(e.text, "á");
    let mut config = Settings::default();
    config.toggle = Some(Binding {
        modifiers: META,
        key: Some(0x56),
    });
    assert_eq!(config.validate(), Err("native_history_reserved"));
}
#[test]
fn all_batch_prefixes_repair_only_balance_and_modifiers() {
    let batch = Batch::prepare(
        &Edit {
            backspaces: 2,
            text: "ộ🧪".into(),
        },
        &[0xa0],
        None,
    )
    .unwrap();
    for accepted in 0..=batch.packets.len() {
        let repair = batch.repair(accepted);
        assert!(!repair.iter().any(|p| matches!(
            p,
            Packet::Unicode { up: false, .. } | Packet::Key { vk: 8, up: false }
        )));
        let mut held = Vec::new();
        for &p in batch.packets[..accepted].iter().chain(repair.iter()) {
            let (id, up) = match p {
                Packet::Key { vk, up } => ((false, vk), up),
                Packet::Unicode { unit, up } => ((true, unit), up),
            };
            if up {
                if let Some(i) = held.iter().position(|p| *p == id) {
                    held.remove(i);
                }
            } else {
                held.push(id);
            }
        }
        assert!(
            held.iter().all(|id| *id == (false, 0xa0)),
            "prefix={accepted}"
        );
    }
}
#[test]
fn full_utf16_and_bounds_before_deletion() {
    let text = "🧪".repeat(4096);
    let batch = Batch::prepare(
        &Edit {
            backspaces: 128,
            text,
        },
        &[],
        None,
    )
    .unwrap();
    assert_eq!(batch.packets.len(), (128 + 8192) * 2);
    assert!(
        Batch::prepare(
            &Edit {
                backspaces: 129,
                text: "a".into()
            },
            &[],
            None
        )
        .is_err()
    );
    assert!(
        Batch::prepare(
            &Edit {
                backspaces: 1,
                text: "🧪".repeat(4097)
            },
            &[],
            None
        )
        .is_err()
    );
    assert!(
        Batch::prepare(
            &Edit {
                backspaces: 1,
                text: "\0".into()
            },
            &[],
            None
        )
        .is_err()
    );
}
#[test]
fn long_literal_word_does_not_delete_unobserved_text() {
    let (mut s, mut e) = setup(Settings::default());
    let raw = format!("{}s ", "b".repeat(300));
    type_text(&mut s, &mut e, &raw);
    assert_eq!(e.text, raw);
    assert!(s.diagnostics().fault.is_none());
}

#[test]
fn seeded_typing_streams_never_erase_prefix_or_suffix() {
    // Independent text client invariant across 40,000 varied keystrokes. Only
    // typed text is editable; no physical Backspace requests a sentinel deletion.
    let mut seed = 0x6b65795f77696e31u64;
    let alphabet = b"aadeiosfrxjwznbch 123.,";
    for _ in 0..500 {
        let (mut s, _) = setup(Settings::default());
        let mut e = Editor::new("PRE 🧪  SUF", "PRE 🧪 ".len());
        for _ in 0..80 {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            let ch = alphabet[((seed >> 32) as usize) % alphabet.len()] as char;
            type_text(&mut s, &mut e, &ch.to_string());
            assert!(e.text.starts_with("PRE 🧪 "), "prefix changed");
            assert!(e.text.ends_with(" SUF"), "suffix changed");
            assert!(e.down.is_empty());
            assert!(s.diagnostics().fault.is_none());
        }
    }
}

#[test]
fn macros_in_english_follow_the_snapshot_switch() {
    for in_en in [true, false] {
        let mut config = Settings::default();
        config.vietnamese = false;
        config.engine.shortcut_smart_case = false;
        config.engine.shortcuts_in_english = in_en;
        config.macros.push(("vxqa".into(), "DONE".into()));
        let (mut s, mut e) = setup(config);
        type_text(&mut s, &mut e, "vxqa ");
        assert_eq!(e.text, if in_en { "DONE " } else { "vxqa " });
    }
}

#[test]
fn period_eligibility_uses_rendered_macro_output() {
    let mut config = Settings::default();
    config.engine.shortcut_smart_case = false;
    config.macros.push(("vxqa".into(), "DONE!".into()));
    let (mut s, mut e) = setup(config);
    type_text(&mut s, &mut e, "vxqa  ");
    assert_eq!(e.text, "DONE!  ");
}

#[test]
fn period_drives_sentence_capitalization() {
    let mut config = Settings::default();
    config.engine.auto_capitalize = true;
    let (mut s, mut e) = setup(config);
    type_text(&mut s, &mut e, "a  b");
    assert_eq!(e.text, "a. B");
}

#[test]
fn period_timeout_and_repeat_do_not_transform() {
    let (mut s, mut e) = setup(Settings::default());
    type_text(&mut s, &mut e, "a ");
    press(&mut s, &mut e, key(32, true, Some(' '), 1000));
    press(&mut s, &mut e, key(32, true, Some(' '), 1010));
    press(&mut s, &mut e, key(32, false, None, 1020));
    assert_eq!(e.text, "a   ");
}

#[test]
fn flip_is_cancelled_on_target_change() {
    let mut config = Settings::default();
    config.flip = Some(Binding {
        modifiers: CTRL | config::ALT,
        key: Some(0x74),
    });
    let (mut s, mut e) = setup(config);
    type_text(&mut s, &mut e, "cas");
    for vk in [0xa2, 0xa4, 0x74] {
        press(&mut s, &mut e, key(vk, true, None, 0));
    }
    s.set_target(Some(target(r"D:\Other\Editor.exe")));
    for vk in [0x74, 0xa4, 0xa2] {
        press(&mut s, &mut e, key(vk, false, None, 1));
    }
    assert_eq!(e.text, "cá");
    assert_eq!(s.diagnostics().replacements, 1);
}

#[test]
fn left_right_modifier_release_order_is_current_event_aware() {
    for ctrl in [0xa2, 0xa3] {
        for meta in [0x5b, 0x5c] {
            for reverse in [false, true] {
                let (mut s, mut e) = setup(Settings::default());
                for vk in [ctrl, meta] {
                    press(&mut s, &mut e, key(vk, true, None, 0));
                }
                let release = if reverse { [ctrl, meta] } else { [meta, ctrl] };
                for vk in release {
                    press(&mut s, &mut e, key(vk, false, None, 1));
                }
                assert_eq!(s.modifiers(), 0);
                assert_eq!(s.mode_revision(), 1);
            }
        }
    }
}

#[test]
fn per_app_language_memory_and_disable_remember() {
    let mut config = Settings::default();
    config.language_memory.insert(
        config::canonical_app_id(r"C:\Apps\Editor.exe").unwrap(),
        false,
    );
    let (mut s, mut e) = setup(config.clone());
    type_text(&mut s, &mut e, "aa");
    assert_eq!(e.text, "aa");
    s.set_target(Some(target(r"D:\Other\Editor.exe")));
    type_text(&mut s, &mut e, "aa");
    assert_eq!(e.text, "aaâ");
    config.remember_per_app = false;
    let (mut s, mut e) = setup(config);
    type_text(&mut s, &mut e, "aa");
    assert_eq!(e.text, "â");
}

#[test]
fn injected_modifier_cannot_turn_a_shortcut_into_text() {
    let (mut s, mut e) = setup(Settings::default());
    let mut ctrl = key(0xa2, true, None, 0);
    ctrl.origin = Origin::OtherInjected;
    s.handle(ctrl, |_| panic!("modifier must not inject"));
    type_text(&mut s, &mut e, "aa");
    assert_eq!(e.text, "");
    assert_eq!(s.mode_revision(), 0);
    ctrl.down = false;
    s.handle(ctrl, |_| panic!());
    type_text(&mut s, &mut e, "aa");
    assert_eq!(e.text, "â");
}

#[test]
fn shifted_batch_releases_and_restores_only_held_shift() {
    let batch = Batch::prepare(
        &Edit {
            backspaces: 1,
            text: "Á".into(),
        },
        &[0xa1],
        None,
    )
    .unwrap();
    assert_eq!(
        batch.packets.first(),
        Some(&Packet::Key { vk: 0xa1, up: true })
    );
    assert_eq!(
        batch.packets.last(),
        Some(&Packet::Key {
            vk: 0xa1,
            up: false
        })
    );
    assert!(
        !batch
            .packets
            .iter()
            .any(|p| matches!(p, Packet::Key { vk: 0xa0, .. }))
    );
}

#[test]
fn pid_start_layout_and_focus_each_reset_composition() {
    for field in 0..4 {
        let (mut s, mut e) = setup(Settings::default());
        type_text(&mut s, &mut e, "a");
        let mut t = target(r"C:\Apps\Editor.exe");
        match field {
            0 => t.pid += 1,
            1 => t.process_start += 1,
            2 => t.layout += 1,
            _ => t.focus += 1,
        }
        s.set_target(Some(t));
        type_text(&mut s, &mut e, "s");
        assert_eq!(e.text, "as");
    }
}

#[test]
fn shift_enter_allows_the_first_word_immediately() {
    let (mut s, mut e) = setup(Settings::default());
    type_text(&mut s, &mut e, "nhaaps");
    press(&mut s, &mut e, key(0xa0, true, None, 0));
    press(&mut s, &mut e, key(13, true, None, 1));
    press(&mut s, &mut e, key(13, false, None, 2));
    press(&mut s, &mut e, key(0xa0, false, None, 3));
    e.insert("\n");
    type_text(&mut s, &mut e, "ddwojwc casi");
    assert!(e.text.ends_with("\nđược cái"));
}

#[test]
fn rejected_config_keeps_the_last_known_good_snapshot() {
    let (mut s, mut e) = setup(Settings::default());
    type_text(&mut s, &mut e, "a");
    let mut config = Settings::default();
    config.revision = 2;
    config.macros.push(("".into(), "bad".into()));
    assert_eq!(s.configure(config), Err("invalid_macro"));
    assert_eq!(s.diagnostics().config_revision, 1);
    type_text(&mut s, &mut e, "s");
    assert_eq!(e.text, "á");
}

#[test]
fn delivery_probe_is_engine_independent_and_self_stops() {
    let (mut s, mut e) = setup(Settings::default());
    s.begin_probe();
    type_text(&mut s, &mut e, "azx aa azx aa");
    assert_eq!(e.text, "aX aa aX aa");
    assert!(!s.probe_active());
    assert!(!s.diagnostics().running);
    type_text(&mut s, &mut e, " aa");
    assert!(e.text.ends_with(" aa"));
    s.resume();
    type_text(&mut s, &mut e, " aa");
    assert!(e.text.ends_with(" â"));
}

#[test]
fn repair_failure_is_counted_and_never_retried_as_text() {
    let (mut s, mut e) = setup(Settings::default());
    type_text(&mut s, &mut e, "a");
    let mut calls = 0;
    s.handle(key(0x41, true, Some('a'), 0), |batch| {
        calls += 1;
        Delivery::Sent(Receipt {
            expected: batch.packets.len(),
            accepted: 1,
            os_error: 0,
            repair: Some(vxkey_windows_engine::delivery::RepairReceipt {
                expected: 1,
                accepted: 0,
                os_error: 5,
            }),
        })
    });
    assert_eq!(calls, 1);
    assert_eq!(s.diagnostics().failed_repairs, 1);
    assert_eq!(s.diagnostics().last_repair_os_error, 5);
    assert_eq!(s.diagnostics().fault, Some(Fault::Delivery));
}

#[test]
fn startup_held_modifiers_do_not_leave_generic_aliases_stuck() {
    let (mut s, mut e) = setup(Settings::default());
    let mut held = [false; 256];
    for vk in [1, 0x10, 0x11, 0x12, 0xa0, 0xa2, 0xa4] {
        held[vk] = true;
    }
    s.seed_pressed(held);
    s.resume();
    for vk in [0xa0, 0xa2, 0xa4] {
        press(&mut s, &mut e, key(vk, false, None, 0));
    }
    assert_eq!(s.modifiers(), 0);
    assert_eq!(s.mode_revision(), 0);
    type_text(&mut s, &mut e, "aa");
    assert_eq!(e.text, "â");
}

#[test]
fn third_key_then_additional_modifier_does_not_rearm_toggle() {
    let (mut s, mut e) = setup(Settings::default());
    for (vk, down) in [
        (0xa2, true),
        (0x5b, true),
        (0x56, true),
        (0x56, false),
        (0xa3, true),
        (0xa3, false),
        (0x5b, false),
        (0xa2, false),
    ] {
        press(&mut s, &mut e, key(vk, down, None, 0));
    }
    assert_eq!(s.mode_revision(), 0);
}
