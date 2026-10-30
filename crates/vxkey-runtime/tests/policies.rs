use std::cell::Cell;
use vxkey_runtime::{
    config::*, engine::InputSession, history::*, input_policy::Target, period::*, recovery::*,
    shortcut::*, storage::ConfigStore,
};
#[test]
fn defaults() {
    for p in [Platform::Macos, Platform::Windows] {
        let c = Config::fresh(p);
        c.validate().unwrap();
        assert!(!c.system.launch_companion_at_login);
        assert_eq!(c.ui.tray_icon_style, "serious");
        assert_eq!(c.typing.double_space_period, p == Platform::Windows);
        assert!(matches!(
            c.shortcuts.toggle,
            Some(Shortcut::ModifierChord { .. })
        ));
    }
}
#[test]
fn invalid_config() {
    let mut c = Config::fresh(Platform::Macos);
    c.schema_version = 2;
    assert!(c.validate().is_err());
    let mut v = serde_json::to_value(Config::fresh(Platform::Macos)).unwrap();
    v["typing"]["injected"] = true.into();
    assert!(serde_json::from_value::<Config>(v).is_err());
}
#[test]
fn counter_precision() {
    for s in ["0", "9007199254740993", "18446744073709551615"] {
        assert!(counter(s).is_ok());
    }
    for s in ["01", "-1", "1.5", "18446744073709551616"] {
        assert!(counter(s).is_err());
    }
}
#[test]
fn shortcut_validation() {
    let mut c = Config::fresh(Platform::Macos);
    c.shortcuts.history = c.shortcuts.toggle.clone();
    assert!(c.validate().is_err());
    c.shortcuts.history = None;
    c.shortcuts.toggle = Some(Shortcut::ModifierChord {
        modifiers: vec!["control".into(), "control".into()],
    });
    assert!(c.validate().is_err());
}
#[test]
fn macro_case_collision() {
    let mut c = Config::fresh(Platform::Macos);
    for (i, t) in ["vn", "VN"].iter().enumerate() {
        c.macros.entries.push(MacroEntry {
            id: format!("00000000-0000-4000-8000-{:012x}", i),
            trigger: t.to_string(),
            replacement: "Việt Nam".into(),
        });
    }
    assert!(c.validate().is_err());
    c.macros.smart_case = false;
    assert!(c.validate().is_ok());
}
#[test]
fn unicode_byte_cap() {
    let mut c = Config::fresh(Platform::Macos);
    c.macros.entries.push(MacroEntry {
        id: "00000000-0000-4000-8000-000000000001".into(),
        trigger: "vn".into(),
        replacement: "ư".repeat(4097),
    });
    assert!(c.validate().is_err());
}
#[test]
fn actual_engine_parity() {
    for (method, keys, want) in [
        ("telex", "vuwng", "vưng"),
        ("telex_advanced", "vwng", "vưng"),
        ("vni", "vu7ng", "vưng"),
    ] {
        let mut c = Config::fresh(Platform::Macos);
        c.typing.input_method = method.into();
        c.typing.smart_restore = false;
        c.typing.eager_restore = false;
        let mut s = InputSession::default();
        s.configure(&c);
        for k in keys.chars() {
            s.process(k, &c);
        }
        assert_eq!(s.buffer(), want, "{method}");
    }
}
#[test]
fn hard_bypass_english_macros() {
    let mut c = Config::fresh(Platform::Macos);
    c.typing.language_mode = "en".into();
    c.macros.entries.push(MacroEntry {
        id: "00000000-0000-4000-8000-000000000001".into(),
        trigger: "vn".into(),
        replacement: "Việt Nam".into(),
    });
    c.apps.policies.insert(
        "fixture.editor".into(),
        AppPolicy {
            input_bypass: true,
            clipboard_blocked: false,
        },
    );
    let mut s = InputSession::default();
    s.configure(&c);
    s.policy.target = Some(Target {
        app_id: "fixture.editor".into(),
        pid: 1,
        generation: 1,
    });
    for k in "vn ".chars() {
        let r = s.process(k, &c);
        assert!(r.output.is_empty());
        assert_eq!(r.backspace, 0);
    }
    assert!(s.buffer().is_empty());
    assert!(!c.apps.policies["fixture.editor"].clipboard_blocked);
}
#[test]
fn secure_and_suspended() {
    for secure in [true, false] {
        let c = Config::fresh(Platform::Macos);
        let mut s = InputSession::default();
        s.configure(&c);
        s.policy.secure = secure;
        s.policy.suspended = !secure;
        for k in "tieengs".chars() {
            assert!(s.process(k, &c).output.is_empty());
        }
        assert!(s.buffer().is_empty());
    }
}
#[test]
fn target_and_memory() {
    let mut c = Config::fresh(Platform::Macos);
    c.apps
        .language_memory
        .insert("fixture.editor".into(), "en".into());
    let mut s = InputSession::default();
    s.configure(&c);
    let t = Target {
        app_id: "fixture.editor".into(),
        pid: 1,
        generation: 4,
    };
    s.policy.target = Some(t.clone());
    assert_eq!(s.policy.mode(&c), "en");
    let mut stale = t.clone();
    stale.generation = 3;
    assert!(!s.policy.validate_target(&stale));
    assert!(s.policy.validate_target(&t));
}
#[test]
fn modifier_orders() {
    for order in [
        [Modifier::Control, Modifier::Alt],
        [Modifier::Alt, Modifier::Control],
    ] {
        for reverse in [false, true] {
            let mut g = ModifierGesture::new([Modifier::Control, Modifier::Alt]);
            assert!(!g.changed(order[0], false, true));
            assert!(!g.changed(order[1], true, true));
            let release = if reverse { [order[1], order[0]] } else { order };
            assert!(!g.changed(release[0], release[0] == order[1], false));
            assert!(g.changed(release[1], release[1] == order[1], false));
            assert!(!g.changed(release[1], false, false));
        }
    }
}
#[test]
fn third_key_cancels() {
    let mut g = ModifierGesture::new([Modifier::Control, Modifier::Meta]);
    g.changed(Modifier::Control, false, true);
    g.changed(Modifier::Meta, false, true);
    g.non_modifier();
    assert!(!g.changed(Modifier::Meta, false, false));
    assert!(!g.changed(Modifier::Control, false, false));
}
#[test]
fn physical_sides() {
    let mut g = ModifierGesture::new([Modifier::Control, Modifier::Meta]);
    g.changed(Modifier::Control, false, true);
    g.changed(Modifier::Control, true, true);
    g.changed(Modifier::Meta, false, true);
    assert!(!g.changed(Modifier::Control, false, false));
    assert!(!g.changed(Modifier::Meta, false, false));
    assert!(g.changed(Modifier::Control, true, false));
}
#[test]
fn extra_modifier_cancels() {
    let mut g = ModifierGesture::new([Modifier::Control, Modifier::Alt]);
    g.changed(Modifier::Control, false, true);
    g.changed(Modifier::Alt, false, true);
    g.changed(Modifier::Shift, false, true);
    for m in [Modifier::Shift, Modifier::Alt, Modifier::Control] {
        assert!(!g.changed(m, false, false));
    }
}
fn space(caret: usize, rev: u64) -> SpaceContext {
    SpaceContext {
        focus_generation: 1,
        caret_utf16: caret,
        edit_revision: rev,
        unmodified: true,
        repeat: false,
        eligible_prefix: true,
        safe: true,
    }
}
#[test]
fn two_spaces() {
    let mut p = PeriodGesture::default();
    assert!(!p.space(100, space(5, 1), true, NativePeriod::Disabled));
    assert!(p.space(600, space(6, 2), true, NativePeriod::Disabled));
    assert!(!p.space(650, space(7, 3), false, NativePeriod::Disabled));
}
#[test]
fn native_period_lock() {
    for native in [NativePeriod::Enabled, NativePeriod::Unknown] {
        let mut p = PeriodGesture::default();
        assert!(!p.space(100, space(5, 1), true, native));
        assert!(!p.space(200, space(6, 2), true, native));
    }
}
#[test]
fn period_stale_context() {
    for variant in 0..4 {
        let mut p = PeriodGesture::default();
        p.space(100, space(5, 1), true, NativePeriod::Disabled);
        let mut c = space(6, 2);
        let time = if variant == 0 { 601 } else { 200 };
        if variant == 1 {
            c.focus_generation = 2;
        }
        if variant == 2 {
            c.caret_utf16 = 8;
        }
        if variant == 3 {
            c.edit_revision = 4;
        }
        assert!(!p.space(time, c, true, NativePeriod::Disabled));
    }
}
fn meta(n: u64) -> CaptureMeta {
    CaptureMeta {
        count: n,
        after_count: n,
        text_only: true,
        concealed: false,
        transient: false,
        generated: false,
        own_write: false,
        remote: false,
        source_known: true,
        source_blocked: false,
    }
}
#[test]
fn filter_before_payload() {
    for variant in 0..8 {
        let mut h = History::default();
        h.enabled = true;
        let mut m = meta(1);
        match variant {
            0 => m.concealed = true,
            1 => m.transient = true,
            2 => m.generated = true,
            3 => m.own_write = true,
            4 => m.remote = true,
            5 => m.source_known = false,
            6 => m.source_blocked = true,
            _ => m.text_only = false,
        };
        let reads = Cell::new(0);
        assert!(!h.capture(m, 0, || {
            reads.set(reads.get() + 1);
            Some(("secret".into(), 1))
        }));
        assert_eq!(reads.get(), 0);
        assert!(h.is_empty());
    }
}
#[test]
fn race_after_payload() {
    let mut h = History::default();
    h.enabled = true;
    assert!(!h.capture(meta(1), 0, || Some(("raced".into(), 2))));
    assert!(h.is_empty());
}
#[test]
fn pause_baseline() {
    let mut h = History::default();
    h.enabled = true;
    h.paused = true;
    assert!(!h.capture(meta(1), 0, || panic!("read")));
    h.baseline(1);
    h.paused = false;
    assert!(!h.capture(meta(1), 0, || panic!("backfill")));
    assert!(h.capture(meta(2), 0, || Some(("new".into(), 2))));
}
#[test]
fn history_unicode_quota_expiry_delete() {
    let mut h = History::default();
    h.enabled = true;
    h.set_limits(1, 1).unwrap();
    let text = "  tiếng Việt\n🙂 \n";
    assert!(h.capture(meta(1), 0, || Some((text.into(), 1))));
    assert_eq!(h.search("Việt", 0, 50, 0)[0].text, text);
    h.capture(meta(2), 1, || Some(("second".into(), 2)));
    assert_eq!(h.len(), 1);
    let e = h.search("", 0, 50, 1).remove(0);
    assert!(h.get(&e.id, h.revision + 1, 1).is_err());
    assert!(h.delete(&e.id, h.revision).unwrap());
    h.capture(meta(3), 1, || Some(("expires".into(), 3)));
    h.prune(86400002);
    assert!(h.is_empty());
}
#[test]
fn oversized_history() {
    let mut h = History::default();
    h.enabled = true;
    assert!(!h.capture(meta(1), 0, || Some(("a".repeat(MAX_ITEM + 1), 1))));
    assert!(h.is_empty());
}
#[test]
fn atomic_config() {
    let d = tempfile::tempdir().unwrap();
    let path = d.path().join("config.json");
    let mut s = ConfigStore::open(&path, Platform::Macos).unwrap();
    s.save_initial().unwrap();
    let mut new = s.config.clone();
    new.ui.tray_icon_style = "playful".into();
    s.replace(new, "1").unwrap();
    assert_eq!(s.config.revision, "2");
    let read = ConfigStore::open(&path, Platform::Macos).unwrap();
    assert_eq!(read.config.ui.tray_icon_style, "playful");
    assert!(s.replace(s.config.clone(), "1").is_err());
}
#[test]
fn corrupt_not_overwritten() {
    let d = tempfile::tempdir().unwrap();
    let path = d.path().join("config.json");
    std::fs::write(&path, b"corrupt").unwrap();
    assert!(ConfigStore::open(&path, Platform::Macos).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), b"corrupt");
}
#[test]
fn validation_keeps_revision() {
    let d = tempfile::tempdir().unwrap();
    let mut s = ConfigStore::open(d.path().join("config.json"), Platform::Macos).unwrap();
    s.save_initial().unwrap();
    let mut invalid = s.config.clone();
    invalid.ui.theme = "made_up".into();
    assert!(s.replace(invalid, "1").is_err());
    assert_eq!(s.config.revision, "1");
}
#[test]
fn partial_injection_suspend() {
    let mut r = Recovery::default();
    assert!(!r.resume(false, 0));
    assert!(r.resume(true, 0));
    assert!(!r.injection_result(4, 2));
    assert_eq!(r.health, Health::Suspended);
    r.acknowledge(9, 200);
    assert_eq!(r.health, Health::Suspended);
}
#[test]
fn actual_loop_heartbeat() {
    let mut r = Recovery::default();
    r.resume(true, 0);
    r.acknowledge(1, 100);
    for now in [1000, 2000, 3200] {
        r.acknowledge(1, now);
        r.observe(now);
    }
    assert_eq!(r.health, Health::Unresponsive);
}
#[test]
fn config_is_deferred_until_boundary() {
    let mut c = Config::fresh(Platform::Macos);
    c.typing.smart_restore = false;
    c.typing.eager_restore = false;
    let mut s = InputSession::default();
    s.configure(&c);
    s.process('a', &c);
    s.process('s', &c);
    assert_eq!(s.buffer(), "á");
    let mut next = c.clone();
    next.typing.input_method = "vni".into();
    assert!(!s.configure(&next));
    assert_eq!(s.buffer(), "á");
    s.process(' ', &c);
    assert!(s.buffer().is_empty());
    s.process('a', &next);
    s.process('1', &next);
    assert_eq!(s.buffer(), "á");
}
#[test]
fn io_failure_does_not_advance_config() {
    let d = tempfile::tempdir().unwrap();
    let parent = d.path().join("parent");
    let mut s = ConfigStore::open(parent.join("config.json"), Platform::Macos).unwrap();
    std::fs::write(&parent, b"file blocks parent directory").unwrap();
    let mut next = s.config.clone();
    next.ui.theme = "dark".into();
    assert!(s.replace(next, "1").is_err());
    assert_eq!(s.config.revision, "1");
    assert_eq!(s.config.ui.theme, "system");
}
#[test]
fn service_persist_readback_and_surface_authority() {
    use serde_json::json;
    use vxkey_runtime::service::{Service, Surface};
    let d = tempfile::tempdir().unwrap();
    let store = ConfigStore::open(d.path().join("config.json"), Platform::Macos).unwrap();
    let mut service = Service::new(store);
    let request =
        json!({"command":"patch_config","expected_revision":"1","changes":{"ui":{"theme":"dark"}}});
    assert_eq!(
        service.dispatch_from(Surface::History, request.clone())["error"]["code"],
        "permission_denied"
    );
    assert_eq!(
        service.dispatch_from(Surface::Settings, request.clone())["ok"],
        true
    );
    assert_eq!(
        service.dispatch(request)["error"]["code"],
        "revision_conflict"
    );
    assert_eq!(
        service.dispatch(json!({"command":"get_snapshot"}))["data"]["config"]["ui"]["theme"],
        "dark"
    );
}
#[test]
fn service_native_failure_is_not_reported_success() {
    use serde_json::json;
    use vxkey_runtime::service::Service;
    let d = tempfile::tempdir().unwrap();
    let mut service =
        Service::new(ConfigStore::open(d.path().join("config.json"), Platform::Macos).unwrap());
    for command in [
        "copy_history_entry",
        "set_launch_at_login",
        "set_windows_history_provider",
    ] {
        assert_eq!(service.dispatch(json!({"command":command}))["ok"], false);
    }
    let snapshot = service.dispatch(json!({"command":"get_snapshot"}));
    assert_eq!(snapshot["data"]["capabilities"]["native_engine"], false);
    assert_eq!(snapshot["data"]["capabilities"]["native_capture"], false);
}
#[test]
fn converter_uses_real_core_and_order() {
    use serde_json::json;
    use vxkey_runtime::service::{Service, Surface};
    let d = tempfile::tempdir().unwrap();
    let mut service =
        Service::new(ConfigStore::open(d.path().join("config.json"), Platform::Macos).unwrap());
    let r=service.dispatch_from(Surface::Conversion,json!({"command":"convert_text","text":"ĐẸP Việt Nam","from_charset":"unicode","to_charset":"unicode","transforms":["lower","no_diacritics"],"keep_d":false,"flatten_caps":false}));
    assert_eq!(r["ok"], true);
    assert_eq!(r["data"]["text"], "dep viet nam");
}

#[test]
fn shortcut_collision_ignores_modifier_order() {
    let mut c = vxkey_runtime::config::Config::fresh(vxkey_runtime::config::Platform::Macos);
    c.shortcuts.flip = Some(vxkey_runtime::config::Shortcut::KeyChord {
        modifiers: vec!["meta".into(), "control".into()],
        key_code: "KeyV".into(),
    });
    assert_eq!(c.validate().unwrap_err(), "duplicate_shortcut");
}

#[test]
fn pending_config_applies_after_cancel_and_backspace() {
    for cancel in [true, false] {
        let mut c = Config::fresh(Platform::Macos);
        c.typing.smart_restore = false;
        c.typing.eager_restore = false;
        let mut session = InputSession::default();
        session.configure(&c);
        session.process('a', &c);
        let mut next = c.clone();
        next.typing.input_method = "vni".into();
        assert!(!session.configure(&next));
        if cancel {
            session.clear();
        } else {
            session.backspace(&c);
        }
        assert!(session.buffer().is_empty());
        session.process('a', &next);
        session.process('1', &next);
        assert_eq!(session.buffer(), "á");
    }
}
