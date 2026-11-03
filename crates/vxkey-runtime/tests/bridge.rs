use std::{
    io::Cursor,
    time::{Duration, Instant},
};
use vxkey_runtime::{
    bridge::*,
    config::{Config, Platform},
};
fn frame(message: Message) -> Vec<u8> {
    let mut wire = Wire::new(fresh_id().unwrap()).unwrap();
    let mut bytes = vec![];
    wire.send(&mut bytes, message).unwrap();
    bytes
}
fn hello() -> Message {
    Message::Handshake(Handshake {
        component: "engine".into(),
        app_version: "0.1.0".into(),
    })
}
#[test]
fn framed_handshake_roundtrip_and_replay_rejected() {
    let bytes = frame(hello());
    let mut input = Cursor::new([bytes.clone(), bytes].concat());
    let mut receiver = Wire::accept(&mut input).unwrap();
    assert!(receiver.read(&mut input).is_err());
}
#[test]
fn frame_bounds_truncation_and_unknown_fields() {
    assert!(Wire::accept(&mut Cursor::new((FRAME_LIMIT as u32 + 1).to_be_bytes())).is_err());
    let bytes = frame(hello());
    assert!(Wire::accept(&mut Cursor::new(&bytes[..bytes.len() - 1])).is_err());
    let mut doc: serde_json::Value = serde_json::from_slice(&bytes[4..]).unwrap();
    doc["untrusted"] = serde_json::json!(true);
    let payload = serde_json::to_vec(&doc).unwrap();
    let mut bad = (payload.len() as u32).to_be_bytes().to_vec();
    bad.extend(payload);
    assert!(Wire::accept(&mut Cursor::new(bad)).is_err());
}
#[test]
fn session_binding_and_counter_canonicality() {
    let bytes = frame(hello());
    let mut receiver = Wire::new(fresh_id().unwrap()).unwrap();
    assert!(receiver.read(&mut Cursor::new(bytes)).is_err());
    assert!(Wire::new("not-a-session".into()).is_err());
    for bad in ["01", "-1", "18446744073709551616"] {
        assert!(vxkey_runtime::config::counter(bad).is_err());
    }
}
#[test]
fn projection_does_not_include_ui_clipboard_or_history_shortcut() {
    let c = Config::fresh(Platform::Macos);
    let projected = NativeConfig::from_config(&c);
    projected.validate().unwrap();
    let json = serde_json::to_value(&projected).unwrap();
    for key in ["ui", "clipboard", "system", "history"] {
        assert!(json.get(key).is_none());
    }
    let mut bad = projected;
    bad.macros.entries.push(vxkey_runtime::config::MacroEntry {
        id: fresh_id().unwrap(),
        trigger: "bad trigger".into(),
        replacement: "x".into(),
    });
    assert!(bad.validate().is_err());
}
#[test]
fn progress_requires_actual_increasing_main_loop_sequence() {
    let now = Instant::now();
    let mut p = Progress::default();
    assert!(p.observe("0", now).is_err());
    p.observe("1", now).unwrap();
    p.observe("1", now + Duration::from_secs(2)).unwrap();
    assert!(p.observe("1", now + Duration::from_secs(3)).is_err());
    assert!(p.observe("0", now + Duration::from_secs(4)).is_err());
    p.observe("2", now + Duration::from_secs(4)).unwrap();
}
#[cfg(unix)]
#[test]
fn latch_survives_restart_and_rejects_changed_incident() {
    use vxkey_runtime::bridge::local::Latch;
    let d = tempfile::tempdir().unwrap();
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(d.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
    let latch = Latch::new(d.path()).unwrap();
    assert!(!latch.is_set());
    latch.set("event_loop_stalled").unwrap();
    let incident = latch.incident().unwrap();
    assert!(Latch::new(d.path()).unwrap().is_set());
    latch.set("other").unwrap();
    assert_eq!(latch.incident().unwrap(), incident);
    assert!(latch.clear(Some(b"different")).is_err());
    assert!(latch.is_set());
    latch.clear(Some(&incident)).unwrap();
    assert!(!latch.is_set());
    std::fs::write(d.path().join("recovery_state.json"), b"corrupt").unwrap();
    assert!(latch.is_set());
}
#[cfg(unix)]
#[test]
fn unsafe_directory_and_marker_symlink_fail_closed() {
    use std::os::unix::fs::{PermissionsExt, symlink};
    use vxkey_runtime::bridge::local::Latch;
    let d = tempfile::tempdir().unwrap();
    std::fs::set_permissions(d.path(), std::fs::Permissions::from_mode(0o755)).unwrap();
    assert!(Latch::new(d.path()).is_err());
    std::fs::set_permissions(d.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
    let latch = Latch::new(d.path()).unwrap();
    symlink("missing", d.path().join("recovery_state.json")).unwrap();
    assert!(latch.is_set());
}
fn native_status() -> NativeStatus {
    let session = fresh_id().unwrap();
    NativeStatus {
        diagnostics: None,
        driver: vxkey_runtime::bridge::DriverStatus {
            accessibility: true,
            listen_access: true,
            post_access: true,
            attached: false,
        },
        engine_session_id: session.clone(),
        loop_sequence: "1".into(),
        active: true,
        target: Some(Target {
            app_id: "fixture.editor".into(),
            pid: 4242,
            engine_session_id: session,
            focus_generation: "1".into(),
        }),
        composing: false,
        mode: "vi".into(),
        suspended: false,
        applied_revision: "1".into(),
        result: None,
        memory: None,
        fault: None,
    }
}
#[test]
fn target_token_binds_focus_session_and_preserves_clipboard_policy() {
    use vxkey_runtime::{
        service::{Service, Surface},
        storage::ConfigStore,
    };
    let d = tempfile::tempdir().unwrap();
    let mut store = ConfigStore::open(d.path().join("config.json"), Platform::Macos).unwrap();
    store.config.apps.policies.insert(
        "fixture.editor".into(),
        vxkey_runtime::config::AppPolicy {
            input_bypass: false,
            clipboard_blocked: true,
        },
    );
    let mut service = Service::new(store);
    let status = native_status();
    service.accept_native(status.clone(), fresh_id().unwrap());
    let snapshot = service.dispatch(serde_json::json!({"command":"get_snapshot"}));
    let token = snapshot["data"]["runtime"]["current_app"]["target_token"]
        .as_str()
        .unwrap();
    let response=service.dispatch_from(Surface::Tray,serde_json::json!({"command":"set_current_app_bypass","target_token":token,"expected_revision":"1","input_bypass":true}));
    assert_eq!(response["ok"], true);
    assert!(service.store.config.apps.policies["fixture.editor"].clipboard_blocked);
    let mut changed = status;
    changed.target.as_mut().unwrap().focus_generation = "2".into();
    service.accept_native(changed, fresh_id().unwrap());
    let response=service.dispatch_from(Surface::Tray,serde_json::json!({"command":"set_current_app_bypass","target_token":token,"expected_revision":"2","input_bypass":false}));
    assert_eq!(response["error"]["code"], "stale_target");
}
#[test]
fn stale_runtime_cannot_publish_live_mode_or_accept_bypass() {
    use vxkey_runtime::{service::Service, storage::ConfigStore};
    let d = tempfile::tempdir().unwrap();
    let store = ConfigStore::open(d.path().join("config.json"), Platform::Macos).unwrap();
    let mut service = Service::new(store);
    service.accept_native(native_status(), fresh_id().unwrap());
    service.native.as_mut().unwrap().seen = Instant::now() - Duration::from_secs(4);
    let response = service.dispatch(serde_json::json!({"command":"get_snapshot"}));
    assert_eq!(response["data"]["capabilities"]["native_engine"], false);
    assert!(response["data"]["runtime"]["current_app"].is_null());
    assert_ne!(response["data"]["runtime"]["effective_mode"], "vi");
}
#[test]
fn memory_delta_requires_exact_active_app_and_current_config_revision() {
    use vxkey_runtime::{service::Service, storage::ConfigStore};
    let d = tempfile::tempdir().unwrap();
    let store = ConfigStore::open(d.path().join("config.json"), Platform::Macos).unwrap();
    let mut service = Service::new(store);
    let mut status = native_status();
    status.memory = Some(MemoryDelta {
        app_id: "fixture.other".into(),
        mode: "en".into(),
        base_revision: "1".into(),
    });
    service.accept_native(status.clone(), fresh_id().unwrap());
    assert!(service.store.config.apps.language_memory.is_empty());
    status.memory.as_mut().unwrap().app_id = "fixture.editor".into();
    status.memory.as_mut().unwrap().base_revision = "0".into();
    service.accept_native(status.clone(), fresh_id().unwrap());
    assert!(service.store.config.apps.language_memory.is_empty());
    status.memory.as_mut().unwrap().base_revision = "1".into();
    service.accept_native(status, fresh_id().unwrap());
    assert_eq!(
        service.store.config.apps.language_memory["fixture.editor"],
        "en"
    );
}

#[test]
fn native_projection_rejects_unknown_physical_key_instead_of_false_apply_ack() {
    let mut c = NativeConfig::from_config(&Config::fresh(Platform::Macos));
    c.toggle = Some(vxkey_runtime::config::Shortcut::KeyChord {
        modifiers: vec!["control".into()],
        key_code: "FutureKey".into(),
    });
    assert_eq!(c.validate().unwrap_err(), "unsupported_shortcut");
    c.toggle = None;
    c.validate().unwrap();
}

#[test]
fn background_driver_permission_and_attachment_invariants() {
    let mut status = native_status();
    status.driver.attached = true;
    status.suspended = true;
    assert!(
        status.validate().is_err(),
        "suspended tap must never be reported attached"
    );
    status.suspended = false;
    status.driver.accessibility = false;
    assert!(
        status.validate().is_err(),
        "attached tap requires actual access"
    );
    status.driver.attached = false;
    status.suspended = true;
    status.fault = Some("permission_required".into());
    assert!(status.validate().is_ok());
    let control = Control::RequestPermissions {
        id: fresh_id().unwrap(),
        engine_session_id: status.engine_session_id,
    };
    assert!(control.validate().is_ok());
}
