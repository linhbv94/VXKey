//! Standalone Test GUI owner, prepared for later Windows packaging/native QA.
//! This is not the production Tauri app or an installer. No terminal is needed
//! when built on Windows as `cargo build --release --example engine_test`.
#![cfg_attr(windows, windows_subsystem = "windows")]

#[cfg(not(windows))]
fn main() {
    eprintln!("Windows Test GUI: available on Windows only; native QA is not run.");
}

#[cfg(windows)]
fn main() {
    gui::run();
}

#[cfg(windows)]
mod gui {
    use std::{
        cell::RefCell,
        mem::zeroed,
        panic::{AssertUnwindSafe, catch_unwind},
        ptr,
    };
    use vxkey_windows_engine::{
        config::Settings,
        native::{Command, EngineHandle},
    };
    use windows_sys::Win32::{
        Foundation::*, System::LibraryLoader::GetModuleHandleW, UI::WindowsAndMessaging::*,
    };
    thread_local! { static ENGINE: RefCell<Option<EngineHandle>> = const { RefCell::new(None) }; }
    fn wide(text: &str) -> Vec<u16> {
        text.encode_utf16().chain(Some(0)).collect()
    }
    unsafe fn child(
        parent: HWND,
        class: &str,
        title: &str,
        id: usize,
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        style: u32,
    ) {
        unsafe {
            CreateWindowExW(
                0,
                wide(class).as_ptr(),
                wide(title).as_ptr(),
                WS_CHILD | WS_VISIBLE | style,
                x,
                y,
                w,
                h,
                parent,
                id as _,
                GetModuleHandleW(ptr::null()),
                ptr::null(),
            );
        }
    }
    fn status(window: HWND, text: &str) {
        unsafe {
            SetWindowTextW(GetDlgItem(window, 10), wide(text).as_ptr());
        }
    }
    unsafe extern "system" fn procedure(window: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
        let handled = catch_unwind(AssertUnwindSafe(|| match msg {
            WM_CREATE => {
                unsafe {
                    child(
                        window,
                        "STATIC",
                        "VXKey Test / Windows engine — native QA chưa nghiệm thu",
                        0,
                        16,
                        16,
                        640,
                        22,
                        0,
                    );
                    child(
                        window,
                        "STATIC",
                        "Start để bật; Pause để trả phím nguyên bản. Đóng cửa sổ = thoát engine.\nTạm ngừng bộ gõ Việt khác trước khi Start. Chỉ gõ vào tài liệu nháp.",
                        0,
                        16,
                        45,
                        640,
                        42,
                        0,
                    );
                    child(
                        window,
                        "BUTTON",
                        "Start / Bật",
                        1,
                        16,
                        98,
                        132,
                        30,
                        WS_TABSTOP,
                    );
                    child(
                        window,
                        "BUTTON",
                        "Pause / Dừng",
                        2,
                        160,
                        98,
                        132,
                        30,
                        WS_TABSTOP,
                    );
                    child(
                        window,
                        "BUTTON",
                        "Diagnostics",
                        3,
                        304,
                        98,
                        132,
                        30,
                        WS_TABSTOP,
                    );
                    child(
                        window,
                        "BUTTON",
                        "Probe azx → aX",
                        4,
                        448,
                        98,
                        190,
                        30,
                        WS_TABSTOP,
                    );
                    child(
                        window,
                        "STATIC",
                        "Paused. Hook health: chưa xác nhận.",
                        10,
                        16,
                        142,
                        640,
                        68,
                        0,
                    );
                    child(
                        window,
                        "STATIC",
                        "Probe: azx → aX; tự dừng sau 12 phím hoặc 30 giây. Start = Funput thật.\nGõ tay: as af aa ngaan hangf casch cachs ddwojwc w",
                        0,
                        16,
                        222,
                        640,
                        42,
                        0,
                    );
                    child(
                        window,
                        "EDIT",
                        "",
                        20,
                        16,
                        278,
                        640,
                        130,
                        WS_TABSTOP
                            | WS_BORDER
                            | WS_VSCROLL
                            | ES_MULTILINE as u32
                            | ES_AUTOVSCROLL as u32,
                    );
                }
                match EngineHandle::start(Settings::default()) {
                    Ok(handle) => ENGINE.with(|engine| *engine.borrow_mut() = Some(handle)),
                    Err(error) => status(window, &format!("Không khởi tạo được: {error}")),
                }
                Some(0)
            }
            WM_COMMAND => {
                let id = wp & 0xffff;
                if (1..=4).contains(&id) {
                    let result=ENGINE.with(|engine|{
                            let borrowed=engine.borrow();let handle=borrowed.as_ref().ok_or("engine_unavailable".to_owned())?;
                            if id==1 {handle.command(Command::Resume).map_err(str::to_owned)?;}
                            if id==2 {handle.command(Command::Pause).map_err(str::to_owned)?;}
                            if id==4 {handle.command(Command::DeliveryProbe).map_err(str::to_owned)?;}
                            let snapshot=handle.snapshot().map_err(str::to_owned)?;
                            Ok::<_,String>(format!("{} | mode={}/{} | native QA={} | hook health={}\nbatches={} | failed={} | accepted={} | max callback={}us | fault={:?}",
                                if snapshot.diagnostics.running{"Running / Đang bật"}else{"Paused / Tạm dừng"},
                                snapshot.input_mode,if snapshot.vietnamese{"VI"}else{"EN"},snapshot.native_acceptance,snapshot.hook_health,
                                snapshot.diagnostics.replacements,snapshot.diagnostics.failed_batches,snapshot.diagnostics.accepted_inputs,
                                snapshot.diagnostics.max_callback_us,snapshot.diagnostics.fault))
                        });
                    status(
                        window,
                        &result.unwrap_or_else(|error| format!("Chưa xác nhận: {error}")),
                    );
                }
                Some(0)
            }
            WM_DESTROY => {
                ENGINE.with(|engine| drop(engine.borrow_mut().take()));
                unsafe {
                    PostQuitMessage(0);
                }
                Some(0)
            }
            _ => None,
        }));
        match handled {
            Ok(Some(result)) => result,
            _ => unsafe { DefWindowProcW(window, msg, wp, lp) },
        }
    }
    pub fn run() {
        unsafe {
            let instance = GetModuleHandleW(ptr::null());
            let class = wide("VXKeyWindowsEngineTest");
            let mut wc: WNDCLASSW = zeroed();
            wc.hInstance = instance;
            wc.lpfnWndProc = Some(procedure);
            wc.lpszClassName = class.as_ptr();
            wc.hCursor = LoadCursorW(ptr::null_mut(), IDC_ARROW);
            wc.hbrBackground = 6usize as _;
            if RegisterClassW(&wc) == 0 {
                return;
            }
            let window = CreateWindowExW(
                0,
                class.as_ptr(),
                wide("VXKey Test — Windows engine").as_ptr(),
                WS_OVERLAPPEDWINDOW,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                700,
                480,
                ptr::null_mut(),
                ptr::null_mut(),
                instance,
                ptr::null(),
            );
            if window.is_null() {
                return;
            }
            ShowWindow(window, SW_SHOW);
            let mut msg: MSG = zeroed();
            while GetMessageW(&mut msg, ptr::null_mut(), 0, 0) > 0 {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }
    }
}
