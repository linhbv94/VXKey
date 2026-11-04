//! Metadata worker: image path, process-start identity, integrity and known native
//! password controls. No UIA text/range lookup. Custom password controls remain a
//! native coverage gate; absence of ES_PASSWORD is not proof of non-sensitive text.
use super::TargetCache;
use crate::{
    config::canonical_app_id,
    session::{Access, Target},
};
use std::{
    mem::{size_of, zeroed},
    ptr,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::Duration,
};
use windows_sys::Win32::{
    Foundation::*,
    Security::*,
    System::{StationsAndDesktops::*, Threading::*},
    UI::{Input::KeyboardAndMouse::GetKeyboardLayout, WindowsAndMessaging::*},
};

struct Handle(HANDLE);
impl Drop for Handle {
    fn drop(&mut self) {
        unsafe {
            CloseHandle(self.0);
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct Focus {
    window: usize,
    focus: usize,
    pid: u32,
    layout: usize,
}

fn focus() -> Option<Focus> {
    let window = unsafe { GetForegroundWindow() };
    if window.is_null() {
        return None;
    }
    let mut pid = 0;
    let thread = unsafe { GetWindowThreadProcessId(window, &mut pid) };
    if pid == 0 || thread == 0 {
        return None;
    }
    let mut gui: GUITHREADINFO = unsafe { zeroed() };
    gui.cbSize = size_of::<GUITHREADINFO>() as u32;
    if unsafe { GetGUIThreadInfo(thread, &mut gui) } == 0 {
        return None;
    }
    // Exclude menus/move-size/system UI, but do not require a text caret or class.
    if gui.flags & (GUI_INMENUMODE | GUI_INMOVESIZE | GUI_SYSTEMMENUMODE | GUI_POPUPMENUMODE) != 0 {
        return None;
    }
    // Hosted editors may focus a child owned by another process/thread. Scope
    // image identity, integrity and keyboard layout to that actual input owner,
    // rather than accidentally grouping every child under ApplicationFrameHost.
    let mut input_pid = pid;
    let input_thread = if gui.hwndFocus.is_null() {
        thread
    } else {
        let id = unsafe { GetWindowThreadProcessId(gui.hwndFocus, &mut input_pid) };
        if id == 0 || input_pid == 0 {
            return None;
        }
        id
    };
    let layout = unsafe { GetKeyboardLayout(input_thread) } as usize;
    if layout == 0 {
        return None;
    }
    Some(Focus {
        window: window as usize,
        focus: gui.hwndFocus as usize,
        pid: input_pid,
        layout,
    })
}

pub(super) fn matches_current(target: &Target) -> bool {
    focus().is_some_and(|f| {
        f.window == target.window
            && f.focus == target.focus
            && f.pid == target.pid
            && f.layout == target.layout
    })
}

fn integrity(process: HANDLE) -> Option<u32> {
    let mut token = ptr::null_mut();
    if unsafe { OpenProcessToken(process, TOKEN_QUERY, &mut token) } == 0 {
        return None;
    }
    let token = Handle(token);
    let mut required = 0;
    unsafe {
        GetTokenInformation(
            token.0,
            TokenIntegrityLevel,
            ptr::null_mut(),
            0,
            &mut required,
        );
    }
    if required < size_of::<TOKEN_MANDATORY_LABEL>() as u32 || required > 65536 {
        return None;
    }
    // Pointer-aligned storage for TOKEN_MANDATORY_LABEL and the SID it references.
    let mut words = vec![0usize; (required as usize).div_ceil(size_of::<usize>())];
    if unsafe {
        GetTokenInformation(
            token.0,
            TokenIntegrityLevel,
            words.as_mut_ptr().cast(),
            required,
            &mut required,
        )
    } == 0
    {
        return None;
    }
    let label = unsafe { &*words.as_ptr().cast::<TOKEN_MANDATORY_LABEL>() };
    if unsafe { IsValidSid(label.Label.Sid) } == 0 {
        return None;
    }
    let count = unsafe { *GetSidSubAuthorityCount(label.Label.Sid) };
    if count == 0 {
        return None;
    }
    Some(unsafe { *GetSidSubAuthority(label.Label.Sid, count as u32 - 1) })
}

fn input_desktop_is_default() -> bool {
    let desktop = unsafe { OpenInputDesktop(0, 0, DESKTOP_READOBJECTS) };
    if desktop.is_null() {
        return false;
    }
    let mut name = [0u16; 256];
    let mut required = 0;
    let ok = unsafe {
        GetUserObjectInformationW(
            desktop,
            UOI_NAME,
            name.as_mut_ptr().cast(),
            (name.len() * 2) as u32,
            &mut required,
        )
    } != 0;
    unsafe {
        CloseDesktop(desktop);
    }
    ok && String::from_utf16_lossy(&name[..name.iter().position(|u| *u == 0).unwrap_or(name.len())])
        .eq_ignore_ascii_case("default")
}

fn known_password(hwnd: usize) -> bool {
    if hwnd == 0 {
        return false;
    }
    let mut class = [0u16; 128];
    let len = unsafe { GetClassNameW(hwnd as _, class.as_mut_ptr(), class.len() as i32) };
    if len <= 0 {
        return false;
    }
    let class = String::from_utf16_lossy(&class[..len as usize]).to_lowercase();
    (class == "edit" || class.starts_with("richedit"))
        && unsafe { GetWindowLongPtrW(hwnd as _, GWL_STYLE) } & 0x20 != 0 // ES_PASSWORD
}

fn resolve(f: Focus, own_integrity: Option<u32>) -> Option<Target> {
    let process = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, f.pid) };
    if process.is_null() {
        return None;
    }
    let process = Handle(process);
    let mut image = vec![0u16; 32768];
    let mut len = image.len() as u32;
    if unsafe { QueryFullProcessImageNameW(process.0, 0, image.as_mut_ptr(), &mut len) } == 0 {
        return None;
    }
    let app_id = canonical_app_id(&String::from_utf16_lossy(&image[..len as usize]))?;
    let mut created: FILETIME = unsafe { zeroed() };
    let mut exit: FILETIME = unsafe { zeroed() };
    let mut kernel: FILETIME = unsafe { zeroed() };
    let mut user: FILETIME = unsafe { zeroed() };
    if unsafe { GetProcessTimes(process.0, &mut created, &mut exit, &mut kernel, &mut user) } == 0 {
        return None;
    }
    let access = if !input_desktop_is_default() || known_password(f.focus) {
        Access::Secure
    } else {
        match (own_integrity, integrity(process.0)) {
            (Some(ours), Some(theirs)) if theirs <= ours => Access::Allowed,
            (Some(_), Some(_)) => Access::Elevated,
            _ => Access::Unknown,
        }
    };
    // Revalidate after cross-process work: never publish metadata for an old focus.
    if focus() != Some(f) {
        return None;
    }
    Some(Target {
        window: f.window,
        focus: f.focus,
        pid: f.pid,
        process_start: (created.dwHighDateTime as u64) << 32 | created.dwLowDateTime as u64,
        layout: f.layout,
        app_id,
        access,
    })
}

pub(super) fn monitor(cache: TargetCache, stop: Arc<AtomicBool>) {
    let ours = integrity(unsafe { GetCurrentProcess() });
    while !stop.load(Ordering::Acquire) {
        let target = focus()
            .and_then(|f| resolve(f, ours))
            .map(|t| (Arc::new(t), std::time::Instant::now()));
        if let Ok(mut cached) = cache.lock() {
            *cached = target;
        }
        // Only the metadata worker waits. Never the native hook or engine.
        thread::sleep(Duration::from_millis(20));
    }
}
