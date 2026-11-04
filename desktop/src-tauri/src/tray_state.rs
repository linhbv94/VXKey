//! Snapshot locks end before native UI calls; busy state skips until the next tick.
use std::sync::Mutex;
use vxkey_runtime::service::Service;
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum State {
    Live,
    Suspended,
    #[cfg_attr(windows,allow(dead_code))]
    Bypassed,
    Disconnected,
    Inactive,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Signature {
    pub style: String,
    pub mode: String,
    pub visible: bool,
    pub locale: String,
    pub state: State,
}
impl Signature {
    pub fn dimmed(&self) -> bool {
        self.state != State::Live
    }
    pub fn asset(&self) -> &'static [u8] {
        match (self.style.as_str(), self.mode.as_str()) {
            ("playful", "vi") => include_bytes!("../assets/menubar_playful_vi.rgba"),
            ("playful", _) => include_bytes!("../assets/menubar_playful_en.rgba"),
            (_, "vi") => include_bytes!("../assets/menubar_serious_vi.rgba"),
            _ => include_bytes!("../assets/menubar_serious_en.rgba"),
        }
    }
    pub fn tooltip(&self) -> String {
        let vi = self.locale == "vi";
        let mode = if self.mode == "vi" {
            if vi { "Tiếng Việt" } else { "Vietnamese" }
        } else {
            "English"
        };
        let state = match (&self.state, vi) {
            (State::Live, _) => "",
            (State::Suspended, true) => " — đã tạm ngừng",
            (State::Bypassed, true) => " — bỏ qua ứng dụng này",
            (State::Disconnected, true) => " — chưa kết nối",
            (State::Inactive, true) => " — chờ ứng dụng nhập liệu",
            (State::Suspended, false) => " — suspended",
            (State::Bypassed, false) => " — app bypassed",
            (State::Disconnected, false) => " — disconnected",
            (State::Inactive, false) => " — waiting for input app",
        };
        format!("VXKey — {mode}{state}")
    }
}
#[cfg_attr(windows,allow(dead_code))]
pub fn snapshot(service: &Mutex<Service>) -> Option<Signature> {
    let service = service.try_lock().ok()?;
    let c = &service.store.config;
    let native = service.native.as_ref().filter(|n| n.fresh());
    let mode = native
        .map(|n| n.status.mode.clone())
        .unwrap_or_else(|| c.typing.language_mode.clone());
    let state = match native {
        None => State::Disconnected,
        Some(n) if n.status.suspended => State::Suspended,
        Some(n)
            if n.status.target.as_ref().is_some_and(|t| {
                c.apps
                    .policies
                    .get(&t.app_id)
                    .is_some_and(|p| p.input_bypass)
            }) =>
        {
            State::Bypassed
        }
        Some(n) if !n.status.active || !n.status.driver.attached => State::Inactive,
        Some(_) => State::Live,
    };
    Some(Signature {
        style: c.ui.tray_icon_style.clone(),
        mode,
        visible: c.system.show_tray_icon,
        locale: c.ui.locale.clone(),
        state,
    })
}
#[cfg_attr(windows,allow(dead_code))]
pub fn render(
    service: &Mutex<Service>,
    previous: &Mutex<Option<Signature>>,
    apply: impl FnOnce(&Signature) -> bool,
) {
    let Some(signature) = snapshot(service) else {
        return;
    };
    {
        let Ok(old) = previous.try_lock() else { return };
        if old.as_ref() == Some(&signature) {
            return;
        }
    }
    // Native renderer can synchronously dispatch/re-enter main. Neither mutex is held.
    if apply(&signature) {
        if let Ok(mut old) = previous.try_lock() {
            *old = Some(signature)
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn with_service(run: impl FnOnce(Mutex<Service>)) {
        let path = std::env::temp_dir().join(format!(
            "vxkey_tray_{}",
            vxkey_runtime::bridge::fresh_id().unwrap()
        ));
        let store = vxkey_runtime::storage::ConfigStore::open(
            path.join("config.json"),
            vxkey_runtime::config::Platform::Macos,
        )
        .unwrap();
        run(Mutex::new(Service::new(store)));
        let _ = std::fs::remove_dir_all(path);
    }
    #[test]
    fn renderer_can_reenter_both_mutexes() {
        with_service(|service| {
            let cache = Mutex::new(None);
            let mut called = false;
            render(&service, &cache, |_| {
                assert!(service.try_lock().is_ok());
                assert!(cache.try_lock().is_ok());
                true
            });
            let empty = Mutex::new(None);
            render(&service, &empty, |_| false);
            assert!(empty.lock().unwrap().is_none());
            render(&service, &cache, |_| {
                called = true;
                true
            });
            assert!(!called);
        })
    }
    #[test]
    fn busy_runtime_never_blocks_ui() {
        with_service(|service| {
            let _guard = service.lock().unwrap();
            assert!(snapshot(&service).is_none())
        })
    }
    #[test]
    fn busy_icon_cache_skips_without_waiting() {
        with_service(|service| {
            let cache = Mutex::new(None);
            let _guard = cache.lock().unwrap();
            render(&service, &cache, |_| panic!("busy cache must skip"))
        })
    }
    #[test]
    fn disconnected_preserves_selected_svg_and_cannot_look_live() {
        with_service(|service| {
            let mut s = service.lock().unwrap();
            s.store.config.typing.language_mode = "en".into();
            s.store.config.ui.tray_icon_style = "playful".into();
            drop(s);
            let state = snapshot(&service).unwrap();
            assert!(state.dimmed());
            assert_eq!(state.state, State::Disconnected);
            assert_eq!(
                state.asset(),
                include_bytes!("../assets/menubar_playful_en.rgba")
            );
            assert!(state.tooltip().contains("English"));
        })
    }
}
