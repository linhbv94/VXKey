//! Tauri adapter owned by Codex. UI agent supplies frontend + Tauri app context.
//! Caller passes a trusted, platform-resolved data directory, never a JS path.
use serde_json::{Value, json};
use std::{path::PathBuf, sync::Mutex};
use vxkey_runtime::{
    config::Platform,
    service::{Service, Surface},
    storage::ConfigStore,
};

pub struct Backend(Mutex<Service>);
impl Backend {
    pub fn open(data_dir: PathBuf) -> Result<Self, String> {
        let store = ConfigStore::open(data_dir.join("config.json"), Platform::current())?;
        store.save_initial()?;
        Ok(Self(Mutex::new(Service::new(store))))
    }
}
#[tauri::command]
fn vxkey_command(
    window: tauri::WebviewWindow,
    state: tauri::State<'_, Backend>,
    request: Value,
) -> Value {
    let surface = match window.label() {
        "settings" => Surface::Settings,
        "tray" => Surface::Tray,
        "history" => Surface::History,
        "onboarding" => Surface::Onboarding,
        "conversion" => Surface::Conversion,
        _ => return failure("permission_denied"),
    };
    // Never unwrap a poisoned UI mutex or panic from IPC; keyboard lives separately.
    match state.0.lock() {
        Ok(mut service) => service.dispatch_from(surface, request),
        Err(_) => failure("unavailable"),
    }
}
fn failure(code: &str) -> Value {
    json!({"ok":false,"request_id":null,"data":null,"error":{"code":code,"message_key":format!("errors.{code}"),"retryable":false}})
}
/// Install on exactly one app owner. No global hooks, autostart or native permissions.
pub fn builder(data_dir: PathBuf) -> Result<tauri::Builder<tauri::Wry>, String> {
    let state = Backend::open(data_dir)?;
    Ok(tauri::Builder::default()
        .manage(state)
        .invoke_handler(tauri::generate_handler![vxkey_command]))
}
