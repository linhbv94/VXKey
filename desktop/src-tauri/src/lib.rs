//! Companion owns persistence and windows; keyboard processing remains a separate process.
#[cfg(not(windows))]
mod about;
mod clipboard;
mod tray_state;
#[cfg(windows)]
pub mod windows;
use serde_json::{Value, json};
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};
use tauri::{Emitter, Manager};
use vxkey_runtime::{
    config::Platform,
    service::{Service, Surface},
    storage::ConfigStore,
};
type TraySignature = tray_state::Signature;
pub struct Backend(
    Arc<Mutex<Service>>,
    #[cfg_attr(windows,allow(dead_code))] Mutex<Option<TraySignature>>,
    std::sync::atomic::AtomicBool,
    #[cfg(windows)] Arc<windows::Host>,
    #[cfg(target_os = "macos")] Option<vxkey_runtime::bridge::local::Host>,
    #[cfg(target_os = "macos")] Option<vxkey_runtime::bridge::local::Latch>,
    #[cfg(target_os = "macos")] Arc<Mutex<Option<std::process::Child>>>,
);
impl Backend {
    pub fn open(data_dir: PathBuf) -> Result<Self, String> {
        let store = ConfigStore::open(data_dir.join("config.json"), Platform::current())?;
        store.save_initial()?;
        let service = Arc::new(Mutex::new(Service::new(store)));
        #[cfg(windows)]
        let windows = windows::Host::start(&service.lock().map_err(|_| "unavailable")?.store.config)?;
        #[cfg(target_os = "macos")]
        let recovery = vxkey_runtime::bridge::local::Latch::new(&data_dir).ok();
        #[cfg(target_os = "macos")]
        let host = vxkey_runtime::bridge::local::Host::start(&data_dir, service.clone()).ok();
        Ok(Self(
            service,
            Mutex::new(None),
            std::sync::atomic::AtomicBool::new(false),
            #[cfg(windows)]
            windows,
            #[cfg(target_os = "macos")]
            host,
            #[cfg(target_os = "macos")]
            recovery,
            #[cfg(target_os = "macos")]
            Arc::new(Mutex::new(None)),
        ))
    }
}
// Bounded development fixture evidence. No user content or production logging.
fn probe_visibility(label: &str, visible: Option<bool>, event: &str) {
    #[cfg(debug_assertions)]
    if std::env::var_os("VXKEY_QA_SURFACE").is_some() {
        use std::io::Write;
        if let Some(root) = std::env::var_os("VXKEY_DATA_DIR") {
            let path = PathBuf::from(root).join("ui_window_probe.jsonl");
            if std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0) < 65536 {
                if let Ok(mut file) = std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(path)
                {
                    let _ = writeln!(
                        file,
                        "{}",
                        json!({"event": event, "label": label, "visible": visible})
                    );
                }
            }
        }
    }
    #[cfg(not(debug_assertions))]
    let _ = (label, visible, event);
}
#[tauri::command]
fn vxkey_fit_tray(window: tauri::WebviewWindow, height: f64) -> Result<(), String> {
    if window.label() != "tray" {
        return Err("permission_denied".into());
    }
    if !height.is_finite() || !(280.0..=640.0).contains(&height) {
        return Err("invalid_argument".into());
    }
    window
        .set_size(tauri::LogicalSize::new(320.0, height.ceil()))
        .map_err(|_| "unavailable".into())
}
#[tauri::command]
async fn vxkey_request_permissions(
    window: tauri::WebviewWindow,
    app: tauri::AppHandle,
) -> Result<(), String> {
    if window.label() != "settings" {
        return Err("permission_denied".into());
    }
    tauri::async_runtime::spawn_blocking(move || {
        #[cfg(target_os = "macos")]
        {
            let state = app.state::<Backend>();
            let session = state
                .0
                .lock()
                .map_err(|_| "unavailable")?
                .native
                .as_ref()
                .filter(|n| n.fresh())
                .map(|n| n.status.engine_session_id.clone())
                .ok_or("unavailable")?;
            let id = vxkey_runtime::bridge::fresh_id()?;
            let status = state.3.as_ref().ok_or("unavailable")?.request(
                vxkey_runtime::bridge::Control::RequestPermissions {
                    id,
                    engine_session_id: session,
                },
            )?;
            if status
                .result
                .as_ref()
                .is_some_and(|r| r.outcome == "applied")
            {
                Ok(())
            } else {
                Err("unavailable".into())
            }
        }
        #[cfg(not(target_os = "macos"))]
        {
            let _ = app;
            Err("unsupported".into())
        }
    })
    .await
    .map_err(|_| "unavailable".to_string())?
}
fn show_about(app: &tauri::AppHandle) -> Result<(), String> {
    #[cfg(windows)]
    {
        let window = app.get_webview_window("settings").ok_or("unavailable")?;
        window.show().map_err(|_| "unavailable")?;
        window.set_focus().map_err(|_| "unavailable")?;
        return window.emit("settings_tab", "about").map_err(|_| "unavailable".into());
    }
    #[cfg(not(windows))]
    {
    let vi = app
        .state::<Backend>()
        .0
        .lock()
        .map_err(|_| "unavailable")?
        .store
        .config
        .ui
        .locale
        == "vi";
    about::show(vi)
    }
}
#[tauri::command]
fn vxkey_show_about(window: tauri::WebviewWindow, app: tauri::AppHandle) -> Result<(), String> {
    if !matches!(window.label(), "settings" | "tray") {
        return Err("permission_denied".into());
    }
    // Hide before presenting AppKit's panel: a hidden WebView may suspend the
    // JavaScript continuation, so the host owns both operations atomically.
    if window.label() == "tray" {
        window.hide().map_err(|_| "unavailable")?;
        probe_visibility(window.label(), window.is_visible().ok(), "before_about");
    }
    show_about(&app)
}
#[tauri::command]
fn vxkey_prepare_reset(
    window: tauri::WebviewWindow,
    state: tauri::State<'_, Backend>,
    expected_revision: String,
) -> Result<String, String> {
    if window.label() != "settings" {
        return Err("permission_denied".into());
    }
    state
        .0
        .lock()
        .map_err(|_| "unavailable")?
        .prepare_reset(&expected_revision)
}
#[tauri::command]
fn vxkey_copy_text(window: tauri::WebviewWindow, text: String) -> Result<(), String> {
    if window.label() != "conversion" {
        return Err("permission_denied".into());
    }
    #[cfg(windows)]
    {return clipboard::write_windows(&text,window.hwnd().map_err(|_|"unavailable")?.0 as _).map_err(str::to_string);}
    #[cfg(not(windows))]
    clipboard::write(&text).map_err(str::to_string)
}
#[tauri::command]
fn vxkey_prepare_update(window: tauri::WebviewWindow, app: tauri::AppHandle) -> Result<(), String> {
    if window.label() != "settings" { return Err("permission_denied".into()); }
    stop_owned_engine(&app);
    app.state::<Backend>().2.store(true, std::sync::atomic::Ordering::Release);
    Ok(())
}
#[tauri::command]
fn vxkey_restart_app(window: tauri::WebviewWindow, app: tauri::AppHandle, confirmed: Option<bool>) -> Result<(), String> {
    if !matches!(window.label(), "settings" | "tray") {
        return Err("permission_denied".into());
    }
    if confirmed == Some(true) {
        if window.label() != "settings" { return Err("permission_denied".into()); }
        stop_owned_engine(&app);
        app.state::<Backend>().2.store(true, std::sync::atomic::Ordering::Release);
        app.restart();
    }
    // Request only: do not stop the helper until Settings resolves its draft guard.
    let settings = app.get_webview_window("settings").ok_or("unavailable")?;
    settings.show().map_err(|_| "unavailable")?;
    settings.set_focus().map_err(|_| "unavailable")?;
    settings.emit("request_restart", ()).map_err(|_| "unavailable".into())
}

fn known_window(label: &str) -> bool {
    matches!(
        label,
        "settings" | "tray" | "history" | "onboarding" | "conversion"
    )
}
#[tauri::command]
async fn vxkey_command(
    window: tauri::WebviewWindow,
    app: tauri::AppHandle,
    request: Value,
) -> Value {
    let state = app.state::<Backend>();
    let surface = match window.label() {
        "settings" => Surface::Settings,
        "tray" => Surface::Tray,
        "history" => Surface::History,
        "onboarding" => Surface::Onboarding,
        "conversion" => Surface::Conversion,
        _ => return failure("permission_denied"),
    };
    #[cfg(windows)]
    {
        let update_menu=request["changes"]["ui"].get("locale").is_some();
        let host=state.3.clone(); let service=state.0.clone();
        let response=tauri::async_runtime::spawn_blocking(move || windows::dispatch(&host,&service,surface,request)).await.unwrap_or_else(|_| failure("unavailable"));
        sync_tray(&app);
        if update_menu && response["ok"]==true {let _=install_menu(&app);}
        return response;
    }
    #[cfg(not(windows))]
    {
    #[cfg(target_os = "macos")]
    if matches!(
        request.get("command").and_then(Value::as_str),
        Some("set_language" | "set_input_suspended")
    ) {
        if surface != Surface::Tray && !(surface == Surface::Settings && request["command"] == "set_input_suspended") {
            return failure("permission_denied");
        }
        let host = state.3.clone();
        let recovery = state.4.clone();
        let service = state.0.clone();
        let response = tauri::async_runtime::spawn_blocking(move || {
            native_control(host, recovery, service, request)
        })
        .await
        .unwrap_or_else(|_| failure("unavailable"));
        sync_tray(&app);
        return response;
    }
    let update_menu = request
        .get("changes")
        .and_then(|c| c.get("ui"))
        .and_then(|c| c.get("locale"))
        .is_some();
    let copy = request.get("command").and_then(Value::as_str) == Some("copy_history_entry");
    let mut read_request = request.clone();
    if copy {
        if surface != Surface::History {
            return failure("permission_denied");
        }
        read_request["command"] = json!("get_history_entry");
    }
    // OS/AppKit calls can dispatch or block: read before taking the runtime lock.
    #[cfg(target_os = "macos")]
    let native_period = if objc2_app_kit::NSSpellChecker::isAutomaticPeriodSubstitutionEnabled() {
        vxkey_runtime::period::NativePeriod::Enabled
    } else {
        vxkey_runtime::period::NativePeriod::Disabled
    };
    #[cfg(not(target_os = "macos"))]
    let native_period = vxkey_runtime::period::NativePeriod::NotApplicable;
    let mut response = match state.0.lock() {
        Ok(mut service) => {
            service.native_period = native_period;
            service.dispatch_from(surface, read_request)
        }
        Err(_) => failure("unavailable"),
    };
    if copy && response["ok"] == true {
        if let Some(text) = response["data"]["text"].as_str() {
            match clipboard::write(text) {
                Ok(()) => {
                    response["data"] = json!({"copied":true,"clipboard_write_id":response["request_id"],"store_revision":response["data"]["store_revision"]})
                }
                Err(code) => response = failure(code),
            }
        } else {
            response = failure("invalid_response")
        }
    }
    if response["ok"] == true && response["data"].get("capabilities").is_some() {
        response["data"]["version"] = json!(env!("CARGO_PKG_VERSION"));
        response["data"]["capabilities"]["updater"] = json!(true);
    }
    sync_tray(&app);
    if update_menu && response["ok"] == true {
        let _ = install_menu(&app);
    }

    response
    }
}
#[tauri::command]
fn vxkey_open_surface(
    window: tauri::WebviewWindow,
    app: tauri::AppHandle,
    surface: String,
    tab: Option<String>,
) -> Result<(), String> {
    if !known_window(window.label()) || !known_window(&surface) || surface == "tray" {
        return Err("permission_denied".into());
    }
    if let Some(ref tab) = tab {
        if surface != "settings"
            || !matches!(
                tab.as_str(),
                "general"
                    | "typing"
                    | "apps"
                    | "macros"
                    | "shortcuts"
                    | "clipboard"
                    | "system"
                    | "updates"
                    | "about"
                    | "support"
            )
        {
            return Err("invalid_argument".into());
        }
    }
    let target = app.get_webview_window(&surface).ok_or("unavailable")?;
    target.show().map_err(|_| "unavailable")?;
    target.set_focus().map_err(|_| "unavailable")?;
    if let Some(tab) = tab {
        target
            .emit("settings_tab", tab)
            .map_err(|_| "unavailable")?;
    }
    if window.label() == "tray" {
        let _ = window.hide();
    }
    Ok(())
}
#[tauri::command]
fn vxkey_close_surface(window: tauri::WebviewWindow) -> Result<(), String> {
    if !known_window(window.label()) {
        return Err("permission_denied".into());
    }
    window.hide().map_err(|_| "unavailable")?;
    probe_visibility(window.label(), window.is_visible().ok(), "close_surface");
    Ok(())
}
#[tauri::command]
fn vxkey_quit(
    window: tauri::WebviewWindow,
    app: tauri::AppHandle,
    confirmed: Option<bool>,
) -> Result<(), String> {
    if !matches!(window.label(), "settings" | "tray") {
        return Err("permission_denied".into());
    }
    if confirmed == Some(true) && window.label() == "settings" {
        app.state::<Backend>()
            .2
            .store(true, std::sync::atomic::Ordering::Release);
        app.exit(0);
        return Ok(());
    }
    request_quit(&app)
}
pub fn may_exit(app: &tauri::AppHandle) -> bool {
    app.state::<Backend>()
        .2
        .load(std::sync::atomic::Ordering::Acquire)
}
pub fn request_quit(app: &tauri::AppHandle) -> Result<(), String> {
    let settings = app.get_webview_window("settings").ok_or("unavailable")?;
    settings.show().map_err(|_| "unavailable")?;
    settings.set_focus().map_err(|_| "unavailable")?;
    settings
        .emit("request_quit", ())
        .map_err(|_| "unavailable".into())
}

fn failure(code: &str) -> Value {
    json!({"ok":false,"request_id":null,"data":null,"error":{"code":code,"message_key":format!("errors.{code}"),"retryable":false}})
}
fn sync_tray(app: &tauri::AppHandle) {
    let target = app.clone();
    let _ = app.run_on_main_thread(move || apply_tray(&target));
}
fn apply_tray(app: &tauri::AppHandle) {
    let Some(tray) = app.tray_by_id("vxkey") else {
        return;
    };
    let state = app.state::<Backend>();
    #[cfg(windows)]
    {
        let Ok(service)=state.0.try_lock() else {return};
        let config=&service.store.config;
        let observed=state.3.observed();
        let signature=tray_state::Signature{style:config.ui.tray_icon_style.clone(),mode:if observed.as_ref().is_some_and(|s|s.value["vi"]==true){"vi".into()}else{"en".into()},visible:config.system.show_tray_icon,locale:config.ui.locale.clone(),state:match observed {None=>tray_state::State::Disconnected,Some(ref o) if o.value["running"]!=true=>tray_state::State::Suspended,Some(ref o) if !o.value["fault"].is_null()=>tray_state::State::Inactive,_=>tray_state::State::Live}};
        drop(service);
        let mut bytes=signature.asset().to_vec();
        // Windows does not tint template icons: use a stable blue glyph on both
        // light and dark taskbars, with lower opacity while paused/disconnected.
        for p in bytes.chunks_exact_mut(4){p[0]=0;p[1]=150;p[2]=220;if signature.dimmed(){p[3]=(u16::from(p[3])*45/100) as u8;}}
        let _=tray.set_icon(Some(tauri::image::Image::new_owned(bytes,36,36)));
        let _=tray.set_tooltip(Some(signature.tooltip()));let _=tray.set_visible(signature.visible);
        return;
    }
    #[cfg(not(windows))]
    tray_state::render(&state.0, &state.1, |signature| {
        let mut bytes = signature.asset().to_vec();
        if signature.dimmed() {
            for pixel in bytes.chunks_exact_mut(4) {
                pixel[3] = ((u16::from(pixel[3]) * 45) / 100) as u8
            }
        }
        let image = tauri::image::Image::new_owned(bytes, 36, 36);
        let image_ok = tray.set_title(None::<&str>).is_ok() && tray.set_icon(Some(image)).is_ok();
        #[cfg(target_os = "macos")]
        let image_ok = image_ok && tray.set_icon_as_template(true).is_ok();
        image_ok
            && tray.set_tooltip(Some(signature.tooltip())).is_ok()
            && tray.set_visible(signature.visible).is_ok()
    });
}
pub fn builder(data_dir: PathBuf) -> Result<tauri::Builder<tauri::Wry>, String> {
    let state = Backend::open(data_dir)?;
    Ok(tauri::Builder::default()
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .manage(state)
        .setup(|app| {
            #[cfg(windows)]
            {
                if app.config().identifier!=windows::APP_ID {return Err("Windows build feature/identity mismatch".into());}
                let service=app.state::<Backend>().0.clone();let host=app.state::<Backend>().3.clone();
                std::thread::spawn(move || {
                    let mut mode_revision=0u64;
                    while let Ok(snapshot)=host.request(json!({"command":"snapshot"})) {
                        let next=snapshot["mode_revision"].as_u64().unwrap_or(0);
                        if next!=mode_revision {
                            let config=if let Ok(mut service)=service.lock(){
                                if snapshot["revision"]!=service.store.config.revision {continue;}
                                mode_revision=next;
                                let mut c=service.store.config.clone();
                                c.typing.language_mode=if snapshot["vi"]==true{"vi".into()}else{"en".into()};
                                if let Some(memory)=snapshot["memory"].as_object(){c.apps.language_memory=memory.iter().filter_map(|(app,on)|on.as_bool().map(|on|(app.clone(),if on{"vi".into()}else{"en".into()}))).collect();}
                                let expected=service.store.config.revision.clone();
                                if service.store.replace(c,&expected).is_ok(){
                                    let c=service.store.config.clone();
                                    let _=host.request(json!({"command":"configure","config":c}));
                                    Some(c)
                                }else{None}
                            }else{None};
                            let _=config;
                        }
                        std::thread::sleep(std::time::Duration::from_millis(500));
                    }
                });
            }
            // Menu-bar utility: closing Settings must not leave a Dock app behind.
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);
            #[cfg(target_os = "macos")]
            {
                let helper = app.path().resource_dir()?.join("VXKeyEngine.app/Contents/MacOS/VXKeyEngine");
                // Exact embedded child only; never search PATH or launch another IME.
                if helper.is_file() {
                    let mut command = std::process::Command::new(helper);
                    command.env("VXKEY_DATA_DIR", vxkey_runtime::paths::data_dir()?)
                        .stdin(std::process::Stdio::null()).stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null());
                    if let Ok(child) = command.spawn() {
                        *app.state::<Backend>().5.lock().map_err(|_| "engine_lock")? = Some(child);
                        let state = app.state::<Backend>();
                        let owned = state.5.clone(); let service = state.0.clone(); let latch = state.4.clone();
                        std::thread::spawn(move || {
                            let began = std::time::Instant::now();
                            loop {
                                std::thread::sleep(std::time::Duration::from_millis(250));
                                let Ok(state) = service.try_lock() else { continue };
                                let healthy = state.native.as_ref().is_some_and(|n| n.fresh());
                                let independent = state.native_session_stream;
                                drop(state);
                                if independent {
                                    // A missing UI/control heartbeat cannot kill a live
                                    // keyboard owner. Still detect a real child exit;
                                    // Quit/update remove and reap this exact child below.
                                    let exited = if let Ok(mut slot) = owned.lock() {
                                        let Some(child) = slot.as_mut() else { break };
                                        match child.try_wait() {
                                            Ok(Some(_)) => { slot.take(); true },
                                            _ => false,
                                        }
                                    } else { false };
                                    if exited {
                                        if let Some(latch) = &latch { let _ = latch.set("driver_exited"); }
                                        if let Ok(mut state) = service.lock() { state.native = None; state.suspended = true; }
                                        break;
                                    }
                                    continue;
                                }
                                if healthy { continue; }
                                if began.elapsed() < std::time::Duration::from_secs(8) { continue; }
                                if let Some(latch) = &latch { let _ = latch.set("driver_unresponsive"); }
                                if let Ok(mut child) = owned.lock() {
                                    if let Some(mut child) = child.take() { let _ = child.kill(); let _ = child.wait(); }
                                }
                                break; // no automatic respawn after a keyboard fault
                            }
                        });
                    }
                }
            }
            use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
            let tray = TrayIconBuilder::with_id("vxkey")
                .show_menu_on_left_click(false);
            #[cfg(not(target_os = "macos"))]
            let tray = if let Some(icon) = app.default_window_icon() {
                tray.icon(icon.clone())
            } else {
                tray
            };
            tray.on_tray_icon_event(|tray, event| {
                if let TrayIconEvent::Click {
                    position,
                    button: MouseButton::Left,
                    button_state: MouseButtonState::Up,
                    ..
                } = event
                {
                    if let Some(panel) = tray.app_handle().get_webview_window("tray") {
                        if panel.is_visible().unwrap_or(false) {
                            let _ = panel.hide();
                            return;
                        }
                        let monitor = tray
                            .app_handle()
                            .monitor_from_point(position.x, position.y)
                            .ok()
                            .flatten();
                        let scale = monitor.as_ref().map(|m| m.scale_factor()).unwrap_or(1.0);
                        let width = 320.0 * scale;
                        let height = 420.0 * scale;
                        let mut x = position.x - width / 2.0;
                        let mut y = position.y + 12.0 * scale;
                        if let Some(monitor) = monitor {
                            let origin = monitor.position();
                            let size = monitor.size();
                            x = x.clamp(
                                origin.x as f64,
                                (origin.x as f64 + size.width as f64 - width).max(origin.x as f64),
                            );
                            if y + height > origin.y as f64 + size.height as f64 {
                                y = position.y - height - 12.0 * scale;
                            }
                            y = y.max(origin.y as f64);
                        }
                        let _ =
                            panel.set_position(tauri::PhysicalPosition::new(x as i32, y as i32));
                        let _ = panel.show();
                        let _ = panel.set_focus();
                    }
                }
            })
            .build(app)?;
            install_menu(app.handle())?;
            sync_tray(app.handle());
            let handle = app.handle().clone();
            let queued=Arc::new(std::sync::atomic::AtomicBool::new(false));
            std::thread::spawn(move || loop {
                std::thread::sleep(std::time::Duration::from_millis(500));
                if queued.swap(true,std::sync::atomic::Ordering::AcqRel) {continue;}
                let target = handle.clone(); let done=queued.clone();
                if handle.run_on_main_thread(move || {sync_tray(&target);done.store(false,std::sync::atomic::Ordering::Release);}).is_err() { break; }
            });
            Ok(())
        })
        .on_menu_event(|app, event| {
            #[cfg(windows)]
            if matches!(event.id().as_ref(), "suspend_input"|"resume_input") {
                let host=app.state::<Backend>().3.clone();let pause=event.id().as_ref()=="suspend_input";
                std::thread::spawn(move||{let _=host.set_paused(pause);});
            }
            #[cfg(target_os = "macos")]
            if matches!(event.id().as_ref(), "suspend_input" | "resume_input") {
                let state = app.state::<Backend>();
                let suspended = event.id().as_ref() == "suspend_input";
                let session = state.0.lock().ok().and_then(|s| s.native.as_ref().filter(|n|n.fresh()).map(|n|n.status.engine_session_id.clone()));
                let request = json!({"command":"set_input_suspended","target_session_id":session,"suspended":suspended,"reason":"user"});
                let host=state.3.clone(); let recovery=state.4.clone(); let service=state.0.clone();
                let vi=state.0.lock().map(|s|s.store.config.ui.locale=="vi").unwrap_or(true);
                let handle=app.clone();
                std::thread::spawn(move || {
                    let response=native_control(host,recovery,service,request);
                    let success=response["ok"]==true;
                    let _=handle.emit("native_recovery_result",response);
                    let _=handle.run_on_main_thread(move || native_recovery_notice(vi,suspended,success));
                });
            }
            if event.id().as_ref() == "about" {
                let _ = show_about(app);
            }
            if event.id().as_ref() == "quit" {
                let _ = request_quit(app);
            }
            if event.id().as_ref() == "settings" {
                if let Some(window) = app.get_webview_window("settings") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
        })
        .on_window_event(|window, event| match event {
            tauri::WindowEvent::CloseRequested { api, .. } => {
                api.prevent_close();
                if window.label() == "settings" {
                    let _ = window.emit("request_close", ());
                } else {
                    let _ = window.hide();
                }
            }
            tauri::WindowEvent::Focused(false) if window.label() == "tray" => {
                let _ = window.hide();
                probe_visibility(window.label(), window.is_visible().ok(), "focus_lost");
            }
            _ => {}
        })
        .invoke_handler(tauri::generate_handler![
            vxkey_prepare_update,
            vxkey_restart_app,
            vxkey_request_permissions,
            vxkey_command,
            vxkey_prepare_reset,
            vxkey_show_about,
            vxkey_fit_tray,
            vxkey_open_surface,
            vxkey_close_surface,
            vxkey_quit,
            vxkey_copy_text
        ]))
}


#[cfg(not(windows))]
fn install_menu(app: &tauri::AppHandle) -> tauri::Result<()> {
    use tauri::menu::{Menu, MenuItem, PredefinedMenuItem as P, Submenu};
    let state = app.state::<Backend>();
    let vi = state
        .0
        .lock()
        .map(|s| s.store.config.ui.locale == "vi")
        .unwrap_or(false);
    let about = MenuItem::with_id(
        app,
        "about",
        if vi {
            "Giới thiệu VXKey"
        } else {
            "About VXKey"
        },
        true,
        None::<&str>,
    )?;
    let settings = MenuItem::with_id(
        app,
        "settings",
        if vi { "Cài đặt…" } else { "Settings…" },
        true,
        Some("CmdOrCtrl+,"),
    )?;
    // AppKit's predefined terminate: bypasses Wry ExitRequested. Route normal Quit
    // through the same draft guard as close; the final confirmed action calls exit.
    let quit = MenuItem::with_id(
        app,
        "quit",
        if vi { "Thoát VXKey" } else { "Quit VXKey" },
        true,
        Some("CmdOrCtrl+Q"),
    )?;
    let pause = MenuItem::with_id(
        app,
        "suspend_input",
        if vi {
            "Tạm ngừng xử lý phím"
        } else {
            "Suspend input processing"
        },
        true,
        None::<&str>,
    )?;
    let resume = MenuItem::with_id(
        app,
        "resume_input",
        if vi {
            "Khôi phục xử lý phím"
        } else {
            "Resume input processing"
        },
        true,
        None::<&str>,
    )?;
    let app_menu = Submenu::with_items(
        app,
        "VXKey",
        true,
        &[
            &about,
            &P::separator(app)?,
            &settings,
            &pause,
            &resume,
            &P::separator(app)?,
            &P::services(app, None)?,
            &P::separator(app)?,
            &P::hide(app, None)?,
            &P::hide_others(app, None)?,
            &P::show_all(app, None)?,
            &P::separator(app)?,
            &quit,
        ],
    )?;
    let edit = Submenu::with_items(
        app,
        if vi { "Sửa" } else { "Edit" },
        true,
        &[
            &P::undo(app, None)?,
            &P::redo(app, None)?,
            &P::separator(app)?,
            &P::cut(app, None)?,
            &P::copy(app, None)?,
            &P::paste(app, None)?,
            &P::select_all(app, None)?,
        ],
    )?;
    app.set_menu(Menu::with_items(app, &[&app_menu, &edit])?)?;
    Ok(())
}

#[cfg(target_os = "macos")]
fn native_control(
    host: Option<vxkey_runtime::bridge::local::Host>,
    recovery: Option<vxkey_runtime::bridge::local::Latch>,
    service: Arc<Mutex<Service>>,
    request: Value,
) -> Value {
    use vxkey_runtime::bridge::{Control, Target, fresh_id};
    #[derive(serde::Deserialize)]
    #[serde(tag = "command", rename_all = "snake_case", deny_unknown_fields)]
    enum Request {
        SetLanguage {
            mode: String,
            target: Target,
        },
        SetInputSuspended {
            target_session_id: Option<String>,
            suspended: bool,
            reason: String,
        },
    }
    let required: &[&str] = match request.get("command").and_then(Value::as_str) {
        Some("set_language") => &["command", "mode", "target"],
        Some("set_input_suspended") => &["command", "target_session_id", "suspended", "reason"],
        _ => return failure("invalid_argument"),
    };
    if !request.as_object().is_some_and(|o| {
        o.len() == required.len() && required.iter().all(|key| o.contains_key(*key))
    }) {
        return failure("invalid_argument");
    }
    let Ok(request) = serde_json::from_value::<Request>(request) else {
        return failure("invalid_argument");
    };
    let Ok(id) = fresh_id() else {
        return failure("unavailable");
    };
    let native = service
        .lock()
        .ok()
        .and_then(|s| s.native.clone())
        .filter(|n| n.fresh());
    let control = match request {
        Request::SetLanguage { mode, target } => {
            if !["vi", "en"].contains(&mode.as_str()) || !target.valid() {
                return failure("invalid_argument");
            }
            let Some(native) = native else {
                return failure("unavailable");
            };
            if native.status.target.as_ref() != Some(&target)
                || native.status.suspended
                || !native.status.active
            {
                return failure("stale_target");
            }
            Control::SetLanguage {
                id: id.clone(),
                mode,
                target,
                expected_loop_sequence: native.status.loop_sequence,
            }
        }
        Request::SetInputSuspended {
            target_session_id,
            suspended,
            reason,
        } => {
            if !["user", "fault_recovery"].contains(&reason.as_str())
                || (!suspended && reason != "user")
            {
                return failure("invalid_argument");
            }
            if target_session_id
                .as_ref()
                .is_some_and(|id| !vxkey_runtime::config::valid_uuid(id))
            {
                return failure("invalid_argument");
            }
            if target_session_id.as_ref().is_some_and(|id| {
                native
                    .as_ref()
                    .is_none_or(|n| *id != n.status.engine_session_id)
            }) {
                return failure("stale_target");
            }
            if suspended {
                let Some(latch) = recovery else {
                    return failure("recovery_unavailable");
                };
                if latch.set("user_suspended").is_err() {
                    return failure("recovery_unavailable");
                }
                if let Ok(mut s) = service.lock() {
                    s.suspended = true;
                }
            }
            let Some(native) = native else {
                return failure("unavailable");
            };
            if (!suspended
                && target_session_id.as_deref() != Some(native.status.engine_session_id.as_str()))
                || target_session_id
                    .as_ref()
                    .is_some_and(|id| *id != native.status.engine_session_id)
            {
                return failure("stale_target");
            }
            Control::Suspend {
                id: id.clone(),
                suspended,
                engine_session_id: native.status.engine_session_id,
            }
        }
    };
    let Some(host) = host else {
        return failure("unavailable");
    };
    let result = host.request(control);
    match result {
        Ok(mut status) => {
            // Resume readiness ACK clears the marker first. Wait for the following
            // main-loop status rather than claim resumed from that readiness ACK.
            if status.suspended && !host.latch.is_set() {
                let until = std::time::Instant::now() + std::time::Duration::from_millis(500);
                while std::time::Instant::now() < until {
                    if let Ok(s) = service.lock() {
                        if let Some(n) = s.native.as_ref().filter(|n| {
                            n.fresh()
                                && n.status.engine_session_id == status.engine_session_id
                                && !n.status.suspended
                        }) {
                            status = n.status.clone();
                            break;
                        }
                    }
                    std::thread::sleep(std::time::Duration::from_millis(20));
                }
                if status.suspended {
                    return failure("outcome_unknown");
                }
            }
            json!({"ok":true,"request_id":id,"data":{"language_mode":status.mode,"suspended":status.suspended,"engine_session_id":status.engine_session_id,"health_state":if status.suspended{"suspended"}else if status.driver.attached{"healthy"}else{"degraded"},"runtime_sequence":status.loop_sequence,"persistence":"pending"},"error":null})
        }
        Err(code) => failure(&code),
    }
}

#[cfg(target_os = "macos")]
fn native_recovery_notice(vi: bool, suspended: bool, success: bool) {
    use objc2::MainThreadMarker;
    use objc2_app_kit::NSAlert;
    use objc2_foundation::NSString;
    let Some(mtm) = MainThreadMarker::new() else {
        return;
    };
    let alert = NSAlert::new(mtm);
    alert.setMessageText(&NSString::from_str("VXKey"));
    let text = match (vi, suspended, success) {
        (true, true, true) => {
            "Đã tạm ngừng xử lý phím. VXKey giữ trạng thái này khi khởi động lại."
        }
        (true, false, true) => "Bộ gõ đã xác nhận khôi phục xử lý phím.",
        (true, _, false) => {
            "Chưa khôi phục được bộ gõ. Xem trạng thái trong Cài đặt → Hệ thống. Nếu đã đủ quyền nhưng mất kết nối, hãy thoát và mở lại VXKey."
        }
        (false, true, true) => {
            "Input processing is suspended. VXKey retains this state after restart."
        }
        (false, false, true) => "The input method acknowledged that processing has resumed.",
        (false, _, false) => {
            "The driver could not resume. Check Settings → System. If permissions are granted but the driver is disconnected, quit and reopen VXKey."
        }
    };
    alert.setInformativeText(&NSString::from_str(text));
    alert.addButtonWithTitle(&NSString::from_str("OK"));
    let _ = alert.runModal();
}

/// Stop and reap only the helper spawned by this companion. Child retains PID
/// ownership until wait, preventing accidental signalling of a reused PID.
pub fn stop_owned_engine(app: &tauri::AppHandle) {
    #[cfg(windows)]
    app.state::<Backend>().3.stop();
    #[cfg(target_os = "macos")]
    if let Ok(mut child) = app.state::<Backend>().5.lock() {
        if let Some(mut child) = child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
    #[cfg(not(target_os = "macos"))]
    let _ = app;
}

#[cfg(windows)]
fn install_menu(app:&tauri::AppHandle)->tauri::Result<()> {
    use tauri::menu::{Menu,MenuItem};
    let vi=app.state::<Backend>().0.lock().map(|s|s.store.config.ui.locale=="vi").unwrap_or(true);
    let settings=MenuItem::with_id(app,"settings",if vi{"Cài đặt…"}else{"Settings…"},true,None::<&str>)?;
    let pause=MenuItem::with_id(app,"suspend_input",if vi{"Tạm ngừng xử lý phím"}else{"Pause input processing"},true,None::<&str>)?;
    let resume=MenuItem::with_id(app,"resume_input",if vi{"Bắt đầu gõ tiếng Việt"}else{"Start typing Vietnamese"},true,None::<&str>)?;
    let quit=MenuItem::with_id(app,"quit",if vi{"Thoát VXKey"}else{"Quit VXKey"},true,None::<&str>)?;
    if let Some(tray)=app.tray_by_id("vxkey"){tray.set_menu(Some(Menu::with_items(app,&[&settings,&pause,&resume,&quit])?))?;}
    Ok(())
}
