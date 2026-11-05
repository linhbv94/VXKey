//! Login-item preference transaction; no keyboard control or implicit Resume.
use serde_json::{Value, json};
use vxkey_runtime::{service::Surface, storage::ConfigStore};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    Disabled,
    NotFound,
    Enabled,
    RequiresApproval,
    Unavailable,
}
impl Status {
    pub fn registered(self) -> bool {
        matches!(self, Self::Enabled | Self::RequiresApproval)
    }
    pub fn name(self) -> &'static str {
        match self {
            Self::Disabled => "disabled",
            Self::NotFound => "not_found",
            Self::Enabled => "enabled",
            Self::RequiresApproval => "requires_approval",
            Self::Unavailable => "unavailable",
        }
    }
}
pub trait OsLoginItem {
    fn status(&self) -> Result<Status, String>;
    fn set(&self, enabled: bool) -> Result<(), String>;
}
fn restore(os: &impl OsLoginItem, before: Status) -> bool {
    let _ = os.set(before.registered());
    os.status().is_ok_and(|actual| {
        actual == before
            || (matches!(before, Status::Disabled | Status::NotFound)
                && matches!(actual, Status::Disabled | Status::NotFound))
    })
}
pub fn handles(request: &Value) -> bool {
    request["command"] == "patch_config"
        && request["changes"]["system"]
            .get("launch_companion_at_login")
            .is_some()
}
pub fn patch(
    store: &mut ConfigStore,
    surface: Surface,
    request: &Value,
    os: &impl OsLoginItem,
) -> Result<Value, String> {
    if surface != Surface::Settings {
        return Err("permission_denied".into());
    }
    if request["command"] != "patch_config"
        || request.as_object().is_none_or(|o| {
            o.len() != 3
                || !["command", "expected_revision", "changes"]
                    .iter()
                    .all(|k| o.contains_key(*k))
        })
        || request["changes"].as_object().is_none_or(|o| o.len() != 1)
        || request["changes"]["system"]
            .as_object()
            .is_none_or(|o| o.len() != 1)
    {
        return Err("invalid_argument".into());
    }
    let expected = request["expected_revision"]
        .as_str()
        .ok_or("invalid_argument")?;
    if expected != store.config.revision {
        return Err("revision_conflict".into());
    }
    let enabled = request["changes"]["system"]["launch_companion_at_login"]
        .as_bool()
        .ok_or("invalid_argument")?;
    let mut config = store.config.clone();
    config.system.launch_companion_at_login = enabled;
    config.validate()?;
    let before = os.status()?;
    if before == Status::Unavailable {
        return Err("autostart_unavailable".into());
    }
    let changed = before.registered() != enabled;
    let operation = if changed { os.set(enabled) } else { Ok(()) };
    let actual = match os.status() {
        Ok(actual) if actual != Status::Unavailable && actual.registered() == enabled => actual,
        _ => {
            if changed && !restore(os, before) {
                return Err("autostart_rollback_failed".into());
            }
            return Err(operation.err().unwrap_or_else(|| "autostart_failed".into()));
        }
    };
    if let Err(error) = store.replace(config, expected) {
        // Rename may have committed even when directory fsync was uncertain.
        // Keep OS and committed memory aligned, never roll it back blindly.
        if store.config.revision != expected {
            return Err(error);
        }
        if changed && !restore(os, before) {
            return Err("autostart_rollback_failed".into());
        }
        return Err(error);
    }
    Ok(json!({"revision":store.config.revision,"launch_at_login_state":actual.name()}))
}

#[cfg(target_os = "macos")]
pub mod macos {
    use super::*;
    use objc2::{
        msg_send,
        rc::autoreleasepool,
        runtime::{AnyClass, AnyObject, Bool},
    };
    #[link(name = "ServiceManagement", kind = "framework")]
    unsafe extern "C" {}
    pub struct MainApp;
    pub fn startup_latch(
        root: &std::path::Path,
    ) -> Result<vxkey_runtime::bridge::local::Latch, String> {
        let latch = vxkey_runtime::bridge::local::Latch::new(root)?;
        latch.set("startup_paused")?;
        Ok(latch)
    }
    fn with_service<T>(call: impl FnOnce(&AnyObject) -> Result<T, String>) -> Result<T, String> {
        autoreleasepool(|_| {
            let class = AnyClass::get(c"SMAppService").ok_or("autostart_unavailable")?;
            // SMAppService.mainAppService refers to this companion's app bundle,
            // not the keyboard helper. OS registration happens only on user action.
            let service: *mut AnyObject = unsafe { msg_send![class, mainAppService] };
            let service = unsafe { service.as_ref() }.ok_or("autostart_unavailable")?;
            call(service)
        })
    }
    impl OsLoginItem for MainApp {
        fn status(&self) -> Result<Status, String> {
            with_service(|s| {
                let status: isize = unsafe { msg_send![s, status] };
                Ok(match status {
                    0 => Status::Disabled,
                    1 => Status::Enabled,
                    2 => Status::RequiresApproval,
                    3 => Status::NotFound,
                    _ => Status::Unavailable,
                })
            })
        }
        fn set(&self, enabled: bool) -> Result<(), String> {
            with_service(|s| {
                let error = std::ptr::null_mut::<*mut AnyObject>();
                let ok: Bool = unsafe {
                    if enabled {
                        msg_send![s,registerAndReturnError:error]
                    } else {
                        msg_send![s,unregisterAndReturnError:error]
                    }
                };
                if ok.as_bool() {
                    Ok(())
                } else {
                    Err("autostart_failed".into())
                }
            })
        }
    }
    pub fn open_settings() -> Result<(), String> {
        let class = AnyClass::get(c"SMAppService").ok_or("autostart_unavailable")?;
        unsafe {
            let _: () = msg_send![class, openSystemSettingsLoginItems];
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        cell::{Cell, RefCell},
        path::PathBuf,
    };
    struct Fake {
        state: Cell<Status>,
        writes: RefCell<Vec<bool>>,
        deny: bool,
    }
    impl OsLoginItem for Fake {
        fn status(&self) -> Result<Status, String> {
            Ok(self.state.get())
        }
        fn set(&self, on: bool) -> Result<(), String> {
            self.writes.borrow_mut().push(on);
            if self.deny {
                return Err("autostart_failed".into());
            }
            self.state.set(if on {
                Status::Enabled
            } else {
                Status::Disabled
            });
            Ok(())
        }
    }
    fn fixture(run: impl FnOnce(&mut ConfigStore, &Fake, &PathBuf)) {
        let root = std::env::temp_dir().join(format!(
            "vxkey_login_{}",
            vxkey_runtime::bridge::fresh_id().unwrap()
        ));
        let mut store = ConfigStore::open(
            root.join("config.json"),
            vxkey_runtime::config::Platform::Macos,
        )
        .unwrap();
        store.save_initial().unwrap();
        let fake = Fake {
            state: Cell::new(Status::Disabled),
            writes: RefCell::new(vec![]),
            deny: false,
        };
        run(&mut store, &fake, &root);
        drop(store);
        let _ = std::fs::remove_dir_all(root);
    }
    fn request(revision: &str, on: bool) -> Value {
        json!({"command":"patch_config","expected_revision":revision,"changes":{"system":{"launch_companion_at_login":on}}})
    }
    #[test]
    fn enables_disables_and_persists() {
        fixture(|s, os, _| {
            patch(s, Surface::Settings, &request("1", true), os).unwrap();
            assert!(s.config.system.launch_companion_at_login);
            patch(s, Surface::Settings, &request("2", false), os).unwrap();
            assert!(!s.config.system.launch_companion_at_login);
            assert_eq!(*os.writes.borrow(), vec![true, false]);
        });
    }
    #[test]
    fn stale_wrong_surface_and_mixed_patch_have_no_os_effect() {
        fixture(|s, os, _| {
            assert!(patch(s, Surface::Tray, &request("1", true), os).is_err());
            assert!(patch(s, Surface::Settings, &request("0", true), os).is_err());
            let mut r = request("1", true);
            r["changes"]["system"]["show_tray_icon"] = json!(false);
            assert!(patch(s, Surface::Settings, &r, os).is_err());
            assert!(os.writes.borrow().is_empty());
            assert_eq!(s.config.revision, "1");
        });
    }
    #[test]
    fn os_denial_does_not_persist() {
        fixture(|s, _, _| {
            let os = Fake {
                state: Cell::new(Status::Disabled),
                writes: RefCell::new(vec![]),
                deny: true,
            };
            assert!(patch(s, Surface::Settings, &request("1", true), &os).is_err());
            assert!(!s.config.system.launch_companion_at_login);
        });
    }
    #[test]
    fn persistence_failure_restores_os_registration() {
        fixture(|s, os, root| {
            std::fs::remove_file(root.join("config.json")).unwrap();
            std::fs::create_dir(root.join("config.json")).unwrap();
            assert!(patch(s, Surface::Settings, &request("1", true), os).is_err());
            assert_eq!(os.state.get(), Status::Disabled);
            assert_eq!(*os.writes.borrow(), vec![true, false]);
            assert!(!s.config.system.launch_companion_at_login);
        });
    }
    #[test]
    fn approval_is_honest_and_never_re_registered_on_snapshot() {
        fixture(|s, os, _| {
            os.state.set(Status::RequiresApproval);
            let value = patch(s, Surface::Settings, &request("1", true), os).unwrap();
            assert_eq!(value["launch_at_login_state"], "requires_approval");
            assert!(os.writes.borrow().is_empty());
        });
    }
    #[test]
    fn missing_os_entry_can_be_registered_again() {
        fixture(|s, os, _| {
            os.state.set(Status::NotFound);
            patch(s, Surface::Settings, &request("1", true), os).unwrap();
            assert_eq!(os.state.get(), Status::Enabled);
            assert!(s.config.system.launch_companion_at_login);
        });
    }
    #[cfg(target_os = "macos")]
    #[test]
    fn every_launch_latches_and_keeps_existing_fault() {
        fixture(|_, _, root| {
            let first = macos::startup_latch(root).unwrap();
            assert!(first.is_set());
            let incident = first.incident().unwrap();
            first.clear(Some(&incident)).unwrap();
            assert!(!first.is_set());
            assert!(macos::startup_latch(root).unwrap().is_set());
            let incident = first.incident().unwrap();
            first.clear(Some(&incident)).unwrap();
            first.set("driver_fault").unwrap();
            let incident = first.incident().unwrap();
            macos::startup_latch(root).unwrap();
            assert_eq!(first.incident().unwrap(), incident);
        });
    }
}
