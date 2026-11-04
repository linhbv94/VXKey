//! In-process worker API for the future Windows companion. Starting a worker
//! installs hooks but remains PAUSED until an explicit Resume command. No service,
//! elevation, registry changes, UIA text reads or helper authentication claims.
mod target;

use crate::{
    config::{Settings, is_modifier},
    delivery::{Batch, Packet, Receipt, RepairReceipt},
    session::{Delivery, Diagnostics, Disposition, Fault, Key, Origin, Session, Target},
};
use std::{
    cell::{Cell, RefCell},
    mem::{size_of, zeroed},
    panic::{AssertUnwindSafe, catch_unwind},
    ptr,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver, SyncSender},
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};
use windows_sys::Win32::{
    Foundation::*,
    Security::Cryptography::*,
    System::{LibraryLoader::GetModuleHandleW, Threading::*},
    UI::{Input::KeyboardAndMouse::*, WindowsAndMessaging::*},
};

type TargetCache = Arc<Mutex<Option<(Arc<Target>, Instant)>>>;
static ENGINE_RUNNING: AtomicBool = AtomicBool::new(false);
struct WorkerLease;
impl Drop for WorkerLease {
    fn drop(&mut self) {
        ENGINE_RUNNING.store(false, Ordering::Release);
    }
}
thread_local! {
    static HOST: RefCell<Option<Host>> = const { RefCell::new(None) };
    static TAG: Cell<usize> = const { Cell::new(0) };
}

struct Host {
    session: Session,
    targets: TargetCache,
    last_target: Option<(Arc<Target>, Instant)>,
    caps: bool,
    dead_pending: bool,
    probe_deadline: Option<Instant>,
}

pub enum Command {
    Resume,
    Pause,
    DeliveryProbe,
    Configure(Settings),
    Snapshot(SyncSender<Snapshot>),
}

#[derive(Clone, Debug)]
pub struct Snapshot {
    pub target: Option<Target>,
    pub driver_kind: &'static str,
    pub diagnostics: Diagnostics,
    pub vietnamese: bool,
    pub language_memory: std::collections::BTreeMap<String, bool>,
    pub mode_revision: u64,
    /// Registration is not a hook-health query. Windows can silently remove it.
    pub hook_registered_at_start: bool,
    pub native_acceptance: &'static str,
    pub hook_health: &'static str,
    pub input_mode: &'static str,
    pub last_config_error: Option<&'static str>,
}

pub struct EngineHandle {
    tx: SyncSender<Command>,
    thread_id: u32,
    thread: Option<JoinHandle<()>>,
}

impl EngineHandle {
    pub fn start(settings: Settings) -> Result<Self, String> {
        settings.validate().map_err(str::to_owned)?;
        if ENGINE_RUNNING
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_err()
        {
            return Err("engine_already_started_in_process".into());
        }
        let lease = WorkerLease;
        let (tx, rx) = mpsc::sync_channel(32);
        let (ready_tx, ready_rx) = mpsc::sync_channel(1);
        let cancelled = Arc::new(AtomicBool::new(false));
        let worker_cancelled = cancelled.clone();
        let worker = thread::Builder::new()
            .name("vxkey_windows_hook".into())
            .spawn(move || {
                let _lease = lease;
                run(settings, rx, ready_tx, worker_cancelled);
            })
            .map_err(|e| e.to_string())?;
        match ready_rx.recv_timeout(Duration::from_secs(3)) {
            Ok(Ok(thread_id)) => Ok(Self {
                tx,
                thread_id,
                thread: Some(worker),
            }),
            Ok(Err(error)) => {
                let _ = worker.join();
                Err(error)
            }
            Err(error) => {
                cancelled.store(true, Ordering::Release);
                // Dropping the sender also prevents a late worker from running.
                drop(tx);
                Err(format!("worker_start_timeout: {error}"))
            }
        }
    }
    pub fn command(&self, command: Command) -> Result<(), &'static str> {
        self.tx
            .try_send(command)
            .map_err(|_| "engine_busy_or_stopped")
    }
    /// Metadata only; no raw keystrokes, composed text, clipboard or document data.
    pub fn snapshot(&self) -> Result<Snapshot, &'static str> {
        let (tx, rx) = mpsc::sync_channel(1);
        self.command(Command::Snapshot(tx))?;
        rx.recv_timeout(Duration::from_secs(1))
            .map_err(|_| "snapshot_unconfirmed")
    }
    pub fn shutdown(mut self) {
        self.stop();
    }
    fn stop(&mut self) {
        if let Some(worker) = self.thread.take() {
            unsafe {
                PostThreadMessageW(self.thread_id, WM_QUIT, 0, 0);
            }
            let _ = worker.join();
        }
    }
}
impl Drop for EngineHandle {
    fn drop(&mut self) {
        self.stop();
    }
}

fn run(
    settings: Settings,
    commands: Receiver<Command>,
    ready: SyncSender<Result<u32, String>>,
    cancelled: Arc<AtomicBool>,
) {
    let mut session = match Session::new(settings) {
        Ok(s) => s,
        Err(e) => {
            let _ = ready.send(Err(e.into()));
            return;
        }
    };
    let mut pressed = [false; 256];
    for (vk, down) in pressed.iter_mut().enumerate() {
        *down = unsafe { GetAsyncKeyState(vk as i32) } < 0;
    }
    session.seed_pressed(pressed);
    let mut nonce = 0usize;
    let random_ok = unsafe {
        BCryptGenRandom(
            ptr::null_mut(),
            (&mut nonce as *mut usize).cast(),
            size_of::<usize>() as u32,
            BCRYPT_USE_SYSTEM_PREFERRED_RNG,
        )
    } >= 0;
    if !random_ok || nonce == 0 {
        let _ = ready.send(Err("injection_tag_unavailable".into()));
        return;
    }
    TAG.set(nonce);
    let targets = Arc::new(Mutex::new(None));
    HOST.with(|cell| {
        *cell.borrow_mut() = Some(Host {
            session,
            targets: targets.clone(),
            last_target: None,
            caps: unsafe { GetKeyState(VK_CAPITAL as i32) } & 1 != 0,
            dead_pending: false,
            probe_deadline: None,
        })
    });
    let module = unsafe { GetModuleHandleW(ptr::null()) };
    let keyboard = unsafe { SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard_proc), module, 0) };
    let mouse = unsafe { SetWindowsHookExW(WH_MOUSE_LL, Some(mouse_proc), module, 0) };
    let timer = unsafe { SetTimer(ptr::null_mut(), 0, 10, None) };
    if keyboard.is_null() || mouse.is_null() || timer == 0 {
        let error = unsafe { GetLastError() };
        unsafe {
            if !keyboard.is_null() {
                UnhookWindowsHookEx(keyboard);
            }
            if !mouse.is_null() {
                UnhookWindowsHookEx(mouse);
            }
            if timer != 0 {
                KillTimer(ptr::null_mut(), timer);
            }
        }
        HOST.with(|cell| *cell.borrow_mut() = None);
        let _ = ready.send(Err(format!("hook_install_failed:{error}")));
        return;
    }
    let stop = Arc::new(AtomicBool::new(false));
    let worker_stop = stop.clone();
    let metadata = thread::Builder::new()
        .name("vxkey_windows_target".into())
        .spawn(move || target::monitor(targets, worker_stop));
    let metadata = match metadata {
        Ok(worker) => worker,
        Err(e) => {
            unsafe {
                UnhookWindowsHookEx(keyboard);
                UnhookWindowsHookEx(mouse);
                KillTimer(ptr::null_mut(), timer);
            }
            HOST.with(|cell| *cell.borrow_mut() = None);
            let _ = ready.send(Err(format!("metadata_worker_failed:{e}")));
            return;
        }
    };
    let thread_id = unsafe { GetCurrentThreadId() };
    let mut msg: MSG = unsafe { zeroed() };
    // Ensure PostThreadMessage always has a destination queue before readiness.
    unsafe {
        PeekMessageW(&mut msg, ptr::null_mut(), 0, 0, PM_NOREMOVE);
    }
    let mut active = !cancelled.load(Ordering::Acquire) && ready.send(Ok(thread_id)).is_ok();
    let mut config_error = None;
    while active && !cancelled.load(Ordering::Acquire) {
        let got = unsafe { GetMessageW(&mut msg, ptr::null_mut(), 0, 0) };
        if got <= 0 {
            break;
        }
        if msg.message == WM_TIMER {
            HOST.with(|cell| {
                if let Some(host) = cell.borrow_mut().as_mut() {
                    if host
                        .probe_deadline
                        .is_some_and(|deadline| Instant::now() >= deadline)
                    {
                        if host.session.probe_active() {
                            host.session.pause();
                        }
                        host.probe_deadline = None;
                    }
                }
            });
            // Control never runs in the keyboard callback. Bounded queue/drain.
            for _ in 0..32 {
                match commands.try_recv() {
                    Ok(command) => HOST.with(|cell| {
                        let mut borrowed = cell.borrow_mut();
                        if let Some(host) = borrowed.as_mut() {
                            match command {
                                Command::Resume => {
                                    host.session.resume();
                                    host.dead_pending = false;
                                    host.probe_deadline = None;
                                }
                                Command::Pause => host.session.pause(),
                                Command::DeliveryProbe => {
                                    host.session.begin_probe();
                                    host.probe_deadline =
                                        Some(Instant::now() + Duration::from_secs(30));
                                    host.dead_pending = false;
                                }
                                Command::Configure(settings) => {
                                    config_error = host.session.configure(settings).err();
                                }
                                Command::Snapshot(tx) => {
                                    let _ = tx.try_send(Snapshot {
                                        target: host.targets.try_lock().ok().and_then(|cache| cache.as_ref().filter(|(_, at)| at.elapsed() < Duration::from_millis(250)).map(|(target, _)| (**target).clone())),
                                        driver_kind: "windows_hook_stream",
                                        diagnostics: host.session.diagnostics(),
                                        vietnamese: host.session.effective_vietnamese(),
                                        language_memory: host
                                            .session
                                            .settings()
                                            .language_memory
                                            .clone(),
                                        mode_revision: host.session.mode_revision(),
                                        hook_registered_at_start: true,
                                        hook_health: "unconfirmed",
                                        input_mode: if host.session.probe_active() {
                                            "delivery_probe"
                                        } else {
                                            "funput"
                                        },
                                        native_acceptance: "not_tested",
                                        last_config_error: config_error,
                                    });
                                }
                            }
                        }
                    }),
                    Err(mpsc::TryRecvError::Empty) => break,
                    Err(mpsc::TryRecvError::Disconnected) => {
                        active = false;
                        break;
                    }
                }
            }
        }
        unsafe {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
    unsafe {
        UnhookWindowsHookEx(keyboard);
        UnhookWindowsHookEx(mouse);
        KillTimer(ptr::null_mut(), timer);
    }
    HOST.with(|cell| *cell.borrow_mut() = None);
    TAG.set(0);
    stop.store(true, Ordering::Release);
    let _ = metadata.join();
}

unsafe extern "system" fn keyboard_proc(code: i32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    if code != HC_ACTION as i32 {
        return unsafe { CallNextHookEx(ptr::null_mut(), code, wp, lp) };
    }
    // Windows owns this pointer for the duration of the callback.
    let kbd = unsafe { *(lp as *const KBDLLHOOKSTRUCT) };
    let own = TAG.get() != 0 && kbd.dwExtraInfo == TAG.get() && kbd.flags & LLKHF_INJECTED != 0;
    if own {
        return unsafe { CallNextHookEx(ptr::null_mut(), code, wp, lp) };
    }
    let started = Instant::now();
    let result = catch_unwind(AssertUnwindSafe(|| {
        HOST.with(|cell| {
            let Ok(mut borrowed) = cell.try_borrow_mut() else {
                return Disposition::Pass;
            };
            let Some(host) = borrowed.as_mut() else {
                return Disposition::Pass;
            };
            if let Ok(cached) = host.targets.try_lock() {
                host.last_target = cached.clone();
            }
            let current = host
                .last_target
                .as_ref()
                .filter(|(_, sampled)| sampled.elapsed() <= Duration::from_millis(250))
                .map(|(t, _)| t.clone());
            // No token/path/UIA queries inside the callback. A stale/unknown target
            // is refreshed by the worker; it never blocks or swallows a physical key.
            let current = current.filter(|target| target::matches_current(target));
            if host.session.target() != current.as_deref() {
                host.session.set_target(current.as_deref().cloned());
                host.dead_pending = false;
            }
            let down = matches!(wp as u32, WM_KEYDOWN | WM_SYSKEYDOWN);
            let mut ch = None;
            if down && kbd.flags & LLKHF_INJECTED == 0 && !is_modifier(kbd.vkCode as u16) {
                if kbd.vkCode == VK_CAPITAL as u32 && !host.session.physical_down(VK_CAPITAL) {
                    host.caps = !host.caps;
                }
                if let Some(target) = current.as_ref() {
                    let mut state = host.session.keyboard_state(host.caps);
                    if (kbd.vkCode as usize) < state.len() {
                        state[kbd.vkCode as usize] |= 0x80;
                    }
                    let mut units = [0u16; 8];
                    // Bit 2 avoids modifying the system dead-key buffer. Foreground
                    // layout, rather than the worker's layout, is authoritative.
                    let count = unsafe {
                        ToUnicodeEx(
                            kbd.vkCode,
                            kbd.scanCode,
                            state.as_ptr(),
                            units.as_mut_ptr(),
                            units.len() as i32,
                            4,
                            target.layout as _,
                        )
                    };
                    if count < 0 {
                        host.dead_pending = true;
                    } else if count > 0 && host.dead_pending {
                        host.dead_pending = false;
                    } else if count == 1 {
                        ch = char::from_u32(units[0] as u32);
                    }
                }
            }
            let vk = match kbd.vkCode as u16 {
                VK_SHIFT => {
                    if kbd.scanCode == 0x36 {
                        VK_RSHIFT
                    } else {
                        VK_LSHIFT
                    }
                }
                VK_CONTROL => {
                    if kbd.flags & LLKHF_EXTENDED != 0 {
                        VK_RCONTROL
                    } else {
                        VK_LCONTROL
                    }
                }
                VK_MENU => {
                    if kbd.flags & LLKHF_EXTENDED != 0 {
                        VK_RMENU
                    } else {
                        VK_LMENU
                    }
                }
                vk => vk,
            };
            let key = Key {
                vk,
                down,
                ch,
                source: if (0x60..=0x69).contains(&kbd.vkCode) {
                    crate::KeySource::Numpad
                } else {
                    crate::KeySource::Standard
                },
                origin: if kbd.flags & LLKHF_INJECTED != 0 {
                    Origin::OtherInjected
                } else {
                    Origin::Physical
                },
                time_ms: unsafe { windows_sys::Win32::System::SystemInformation::GetTickCount64() },
            };
            let outcome = host.session.handle(key, |batch| {
                if current
                    .as_ref()
                    .is_none_or(|target| !target::matches_current(target))
                {
                    return Delivery::Cancelled;
                }
                Delivery::Sent(send(batch))
            });
            host.session
                .callback_time(started.elapsed().as_micros().min(u64::MAX as u128) as u64);
            outcome
        })
    }));
    match result {
        Ok(Disposition::Consume) => 1,
        Ok(Disposition::Pass) => unsafe { CallNextHookEx(ptr::null_mut(), code, wp, lp) },
        Err(_) => {
            HOST.with(|cell| {
                if let Ok(mut borrowed) = cell.try_borrow_mut() {
                    if let Some(host) = borrowed.as_mut() {
                        host.session.fault(Fault::CallbackPanic);
                    }
                }
            });
            unsafe { CallNextHookEx(ptr::null_mut(), code, wp, lp) }
        }
    }
}

unsafe extern "system" fn mouse_proc(code: i32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    if code == HC_ACTION as i32
        && matches!(
            wp as u32,
            WM_LBUTTONDOWN
                | WM_RBUTTONDOWN
                | WM_MBUTTONDOWN
                | WM_XBUTTONDOWN
                | WM_MOUSEWHEEL
                | WM_MOUSEHWHEEL
        )
    {
        let _ = catch_unwind(AssertUnwindSafe(|| {
            HOST.with(|cell| {
                if let Ok(mut borrowed) = cell.try_borrow_mut() {
                    if let Some(host) = borrowed.as_mut() {
                        host.session.reset();
                        host.dead_pending = false;
                    }
                }
            })
        }));
    }
    unsafe { CallNextHookEx(ptr::null_mut(), code, wp, lp) }
}

fn input(packet: Packet) -> INPUT {
    let (vk, scan, flags) = match packet {
        Packet::Unicode { unit, up } => (
            0,
            unit,
            KEYEVENTF_UNICODE | if up { KEYEVENTF_KEYUP } else { 0 },
        ),
        Packet::Key { vk, up } => (
            vk,
            if vk == VK_BACK { 0x0e } else { 0 },
            if up { KEYEVENTF_KEYUP } else { 0 }
                | if matches!(vk, 0x5b | 0x5c | 0xa3 | 0xa5) {
                    KEYEVENTF_EXTENDEDKEY
                } else {
                    0
                },
        ),
    };
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: vk,
                wScan: scan,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: TAG.get(),
            },
        },
    }
}

fn send(batch: &Batch) -> Receipt {
    let inputs: Vec<_> = batch.packets.iter().copied().map(input).collect();
    unsafe {
        SetLastError(0);
    }
    let accepted = unsafe {
        SendInput(
            inputs.len() as u32,
            inputs.as_ptr(),
            size_of::<INPUT>() as i32,
        )
    } as usize;
    let error = if accepted == inputs.len() {
        0
    } else {
        unsafe { GetLastError() }
    };
    let mut repair_receipt = None;
    if accepted < inputs.len() && accepted > 0 {
        let repair: Vec<_> = batch.repair(accepted).into_iter().map(input).collect();
        if !repair.is_empty() {
            // Best effort key/modifier balance only; never retry text. The main
            // session latches fault even if this cleanup succeeds.
            unsafe {
                SetLastError(0);
            }
            let repaired = unsafe {
                SendInput(
                    repair.len() as u32,
                    repair.as_ptr(),
                    size_of::<INPUT>() as i32,
                )
            } as usize;
            repair_receipt = Some(RepairReceipt {
                expected: repair.len(),
                accepted: repaired,
                os_error: if repaired == repair.len() {
                    0
                } else {
                    unsafe { GetLastError() }
                },
            });
        }
    }
    Receipt {
        expected: inputs.len(),
        accepted,
        os_error: error,
        repair: repair_receipt,
    }
}
