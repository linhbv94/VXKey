#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    #[cfg(windows)]
    if std::env::args().nth(1).as_deref()==Some("--windows-engine-helper") {
        if vxkey_desktop_backend::windows::helper_main().is_err(){std::process::exit(1);}
        return;
    }
    #[cfg(windows)]
    let _instance=match vxkey_desktop_backend::windows::instance(){Ok(Some(instance))=>instance,Ok(None)=>return,Err(_)=>std::process::exit(1)};
    #[cfg(windows)]
    if std::env::var_os("VXKEY_DATA_DIR").is_none() {
        if let Some(root)=std::env::var_os("APPDATA") {
            // Candidate namespace; production Mac identity/data are unchanged.
            unsafe {std::env::set_var("VXKEY_DATA_DIR",std::path::PathBuf::from(root).join(vxkey_desktop_backend::windows::DATA_NAME));}
        }
    }
    let result = vxkey_runtime::paths::data_dir().and_then(vxkey_desktop_backend::builder);
    match result {
        Ok(builder) => {
            use tauri::Manager;
            let app = match builder.build(tauri::generate_context!()) {
                Ok(app) => app,
                Err(err) => {
                    eprintln!("VXKey companion failed: {err}");
                    std::process::exit(1);
                }
            };
            app.run(|handle, event| {
                if matches!(event,tauri::RunEvent::Ready) && vxkey_desktop_backend::start_in_tray(handle) {
                    for window in handle.webview_windows().values(){let _=window.hide();}
                }
                if let tauri::RunEvent::Exit = event {
                    vxkey_desktop_backend::stop_owned_engine(handle);
                }
                if let tauri::RunEvent::ExitRequested { ref api, .. } = event {
                    if !vxkey_desktop_backend::may_exit(handle) {
                        api.prevent_exit();
                        let _ = vxkey_desktop_backend::request_quit(handle);
                    }
                }
                // Development-only visual fixture entry. No JS command can select it;
                // production builds always launch Settings and expose no QA override.
                #[cfg(debug_assertions)]
                if let tauri::RunEvent::Ready = event {
                    if let Some(surface) = qa_surface() {
                        for window in handle.webview_windows().values() {
                            let _ = window.hide();
                        }
                        if let Some(window) = handle.get_webview_window(surface) {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                }
                #[cfg(target_os = "macos")]
                if let tauri::RunEvent::Reopen { .. } = event {
                    if let Some(window) =
                        handle.get_webview_window(qa_surface().unwrap_or("settings"))
                    {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
                #[cfg(not(target_os = "macos"))]
                let _ = (handle, event);
            });
        }
        Err(err) => {
            eprintln!("VXKey initialization failed: {err}");
            std::process::exit(1);
        }
    }
}

#[cfg_attr(all(windows,not(debug_assertions)),allow(dead_code))]
fn qa_surface() -> Option<&'static str> {
    #[cfg(debug_assertions)]
    if std::env::var_os("VXKEY_DATA_DIR").is_some() {
        return match std::env::var("VXKEY_QA_SURFACE").ok().as_deref() {
            Some("tray") => Some("tray"),
            Some("history") => Some("history"),
            Some("conversion") => Some("conversion"),
            Some("onboarding") => Some("onboarding"),
            _ => None,
        };
    }
    None
}
