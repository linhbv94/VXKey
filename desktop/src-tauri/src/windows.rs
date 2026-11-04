//! Windows owner/control boundary. Anonymous inherited pipes are private process
//! capabilities: no public listener, named-pipe discovery, per-key IPC or secrets
//! in argv. The exact child joins a non-inherited kill-on-close Job BEFORE the
//! first config frame may install hooks. Renderer/control stalls never lease typing.
use serde_json::{Value, json};
use std::{
    io::{Read, Write},
    os::windows::{io::AsRawHandle, process::CommandExt},
    process::{Child, Command, Stdio},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    time::{Duration, Instant},
};
use vxkey_runtime::{
    config::{Config, Shortcut},
    service::{Service, Surface},
};
use vxkey_windows_engine::{
    config::{Binding, Settings, canonical_app_id},
    native::{Command as EngineCommand, EngineHandle},
};
use windows_sys::Win32::{
    Foundation::*,
    System::{JobObjects::*, Threading::CreateMutexW},
    UI::WindowsAndMessaging::*,
};

pub const APP_ID: &str = if cfg!(feature = "windows-production") {
    "com.vxkey.desktop"
} else {
    "com.vxkey.desktop.test.windows"
};
pub const DATA_NAME: &str = if cfg!(feature = "windows-production") {
    "VXKey"
} else {
    "VXKeyTest"
};
pub const RUN_VALUE: &str = DATA_NAME;
const INSTANCE_NAME: &str = if cfg!(feature = "windows-production") {
    r"Local\VXKeyCompanion"
} else {
    r"Local\VXKeyTestCompanion"
};

pub struct Instance(HANDLE);
impl Drop for Instance {
    fn drop(&mut self) {
        unsafe {
            CloseHandle(self.0);
        }
    }
}
pub fn instance() -> Result<Option<Instance>, String> {
    let name: Vec<u16> = INSTANCE_NAME.encode_utf16().chain(Some(0)).collect();
    let handle = unsafe { CreateMutexW(std::ptr::null(), 0, name.as_ptr()) };
    if handle.is_null() {
        return Err("single_instance_failed".into());
    }
    let instance = Instance(handle);
    if unsafe { GetLastError() } == ERROR_ALREADY_EXISTS {
        let title: Vec<u16> = "VXKey Settings".encode_utf16().chain(Some(0)).collect();
        let window = unsafe { FindWindowW(std::ptr::null(), title.as_ptr()) };
        if !window.is_null() && !std::env::args().any(|arg| arg == "--startup") {
            unsafe {
                ShowWindow(window, SW_RESTORE);
                SetForegroundWindow(window);
            }
        }
        return Ok(None);
    }
    Ok(Some(instance))
}

const LIMIT: usize = 2 * 1024 * 1024;
fn read_frame(input: &mut impl Read) -> Result<Value, String> {
    let mut head = [0; 4];
    input.read_exact(&mut head).map_err(|_| "disconnected")?;
    let len = u32::from_le_bytes(head) as usize;
    if len == 0 || len > LIMIT {
        return Err("frame_limit".into());
    }
    let mut bytes = vec![0; len];
    input.read_exact(&mut bytes).map_err(|_| "disconnected")?;
    serde_json::from_slice(&bytes).map_err(|_| "invalid_frame".into())
}
fn write_frame(output: &mut impl Write, value: &Value) -> Result<(), String> {
    let bytes = serde_json::to_vec(value).map_err(|_| "invalid_frame")?;
    if bytes.len() > LIMIT {
        return Err("frame_limit".into());
    }
    output
        .write_all(&(bytes.len() as u32).to_le_bytes())
        .and_then(|_| output.write_all(&bytes))
        .and_then(|_| output.flush())
        .map_err(|_| "disconnected".into())
}
fn binding(value: &Option<Shortcut>) -> Result<Option<Binding>, String> {
    let Some(value) = value else { return Ok(None) };
    let (mods, key) = match value {
        Shortcut::ModifierChord { modifiers } => (modifiers, None),
        Shortcut::KeyChord {
            modifiers,
            key_code,
        } => (modifiers, Some(key_code.as_str())),
    };
    let mut modifiers = 0;
    for m in mods {
        modifiers |= match m.as_str() {
            "control" => 1,
            "alt" => 2,
            "shift" => 4,
            "meta" => 8,
            _ => return Err("invalid_shortcut".into()),
        };
    }
    let key = match key {
        None => None,
        Some(k) => Some(
            if let Some(l) = k
                .strip_prefix("Key")
                .filter(|s| s.len() == 1 && s.as_bytes()[0].is_ascii_uppercase())
            {
                l.as_bytes()[0] as u16
            } else if let Some(d) = k
                .strip_prefix("Digit")
                .filter(|s| s.len() == 1 && s.as_bytes()[0].is_ascii_digit())
            {
                d.as_bytes()[0] as u16
            } else if let Some(f) = k
                .strip_prefix('F')
                .and_then(|s| s.parse::<u16>().ok())
                .filter(|n| (1..=24).contains(n))
            {
                0x70 + f - 1
            } else {
                match k {
                    "Space" => 0x20,
                    "Enter" => 13,
                    "Tab" => 9,
                    "Backspace" => 8,
                    "Escape" => 27,
                    "Home" => 36,
                    "End" => 35,
                    "PageUp" => 33,
                    "PageDown" => 34,
                    "Delete" => 46,
                    "ArrowLeft" => 37,
                    "ArrowUp" => 38,
                    "ArrowRight" => 39,
                    "ArrowDown" => 40,
                    "Minus" => 189,
                    "Equal" => 187,
                    "Comma" => 188,
                    "Period" => 190,
                    "Slash" => 191,
                    "Semicolon" => 186,
                    "Quote" => 222,
                    "BracketLeft" => 219,
                    "BracketRight" => 221,
                    "Backslash" => 220,
                    "Backquote" => 192,
                    _ => return Err("unsupported_shortcut".into()),
                }
            },
        ),
    };
    Ok(Some(Binding { modifiers, key }))
}
pub fn settings(c: &Config) -> Result<Settings, String> {
    c.validate()?;
    let mut s = Settings::default();
    s.revision = c.counter()?;
    s.vietnamese = c.typing.language_mode == "vi";
    s.retone = c.typing.retone;
    s.double_space_period = c.typing.double_space_period;
    s.engine.method = match c.typing.input_method.as_str() {
        "vni" => funput_core::InputMethod::Vni,
        "telex_advanced" => funput_core::InputMethod::TelexAdvanced,
        _ => funput_core::InputMethod::Telex,
    };
    s.engine.tone_style = if c.typing.tone_style == "traditional" {
        funput_core::ToneStyle::Traditional
    } else {
        funput_core::ToneStyle::Modern
    };
    s.engine.smart_restore = c.typing.smart_restore;
    s.engine.eager_restore = c.typing.eager_restore;
    s.engine.spell_check = c.typing.spell_check;
    s.engine.auto_capitalize = c.typing.auto_capitalize;
    let mut extra = funput_core::ExtraOnsets::NONE;
    for l in &c.typing.extra_onsets {
        extra = extra.union(match l.as_str() {
            "z" => funput_core::ExtraOnsets::Z,
            "f" => funput_core::ExtraOnsets::F,
            "w" => funput_core::ExtraOnsets::W,
            "j" => funput_core::ExtraOnsets::J,
            _ => return Err("invalid_extra_onsets".into()),
        });
    }
    s.engine.syllable_rules = funput_core::SyllableRules::STANDARD.with_extra_onsets(extra);
    s.engine.shortcuts_enabled = c.macros.enabled;
    s.engine.shortcuts_in_english = c.macros.in_en;
    s.engine.shortcut_smart_case = c.macros.smart_case;
    s.macros = c
        .macros
        .entries
        .iter()
        .map(|m| (m.trigger.clone(), m.replacement.clone()))
        .collect();
    s.toggle = binding(&c.shortcuts.toggle)?;
    s.flip = binding(&c.shortcuts.flip)?;
    s.remember_per_app = c.apps.remember_per_app;
    for (app, mode) in &c.apps.language_memory {
        s.language_memory.insert(
            canonical_app_id(app).ok_or("invalid_app_identity")?,
            mode == "vi",
        );
    }
    for (app, policy) in &c.apps.policies {
        if policy.input_bypass {
            s.excluded_apps
                .insert(canonical_app_id(app).ok_or("invalid_app_identity")?);
        }
    }
    if let Some(ours) = std::env::current_exe()
        .ok()
        .and_then(|p| canonical_app_id(&p.to_string_lossy()))
    {
        s.excluded_apps.insert(ours);
    }
    s.validate().map_err(str::to_owned)?;
    Ok(s)
}

/// Invoked before Tauri initialization; only the parent-created pipe can reach it.
pub fn helper_main() -> Result<(), String> {
    let mut input = std::io::stdin().lock();
    let mut output = std::io::stdout().lock();
    let bootstrap = read_frame(&mut input)?;
    if bootstrap["protocol"] != 1
        || bootstrap["sequence"] != 1
        || bootstrap["command"] != "bootstrap"
    {
        return Err("invalid_bootstrap".into());
    }
    let session = bootstrap["session"]
        .as_str()
        .filter(|s| vxkey_runtime::config::valid_uuid(s))
        .ok_or("invalid_session")?
        .to_owned();
    let config: Config =
        serde_json::from_value(bootstrap["config"].clone()).map_err(|_| "invalid_config")?;
    let engine = EngineHandle::start(settings(&config)?)?;
    let mut sequence = 1u64;
    let mut request = bootstrap;
    loop {
        let result: Result<(), String> = (|| {
            let allowed: &[&str] = match request["command"].as_str() {
                Some("bootstrap" | "configure") => {
                    &["protocol", "session", "sequence", "command", "config"]
                }
                _ => &["protocol", "session", "sequence", "command"],
            };
            if request.as_object().is_none_or(|o| {
                o.len() != allowed.len() || o.keys().any(|k| !allowed.contains(&k.as_str()))
            }) {
                return Err("invalid_fields".into());
            }
            match request["command"].as_str().ok_or("invalid_command")? {
                "bootstrap" if sequence == 1 => {}
                "configure" => {
                    let c: Config = serde_json::from_value(request["config"].clone())
                        .map_err(|_| "invalid_config")?;
                    engine
                        .command(EngineCommand::Configure(settings(&c)?))
                        .map_err(str::to_owned)?;
                }
                "resume" => engine
                    .command(EngineCommand::Resume)
                    .map_err(str::to_owned)?,
                "pause" => engine
                    .command(EngineCommand::Pause)
                    .map_err(str::to_owned)?,
                "snapshot" => {}
                "shutdown" => return Ok(()),
                _ => return Err("invalid_command".into()),
            };
            Ok(())
        })();
        let response = match engine.snapshot() {
            Ok(s) => {
                let target=s.target.map(|t|json!({"app_id":t.app_id,"pid":t.pid,"process_start":t.process_start.to_string(),"focus":t.focus.to_string(),"layout":t.layout.to_string(),"access":format!("{:?}",t.access)}));
                json!({"protocol":1,"session":session,"sequence":sequence,"ok":result.is_ok() && s.last_config_error.is_none(),"error":result.err().or(s.last_config_error.map(str::to_owned)),"pid":std::process::id(),"running":s.diagnostics.running,"vi":s.vietnamese,"revision":s.diagnostics.config_revision.to_string(),"mode_revision":s.mode_revision,"memory":s.language_memory,"target":target,"hook_registered":s.hook_registered_at_start,"hook_health":s.hook_health,"fault":s.diagnostics.fault.map(|f|format!("{f:?}")),"batches":s.diagnostics.replacements,"failed":s.diagnostics.failed_batches,"accepted":s.diagnostics.accepted_inputs,"max_callback_us":s.diagnostics.max_callback_us})
            }
            Err(e) => {
                json!({"protocol":1,"session":session,"sequence":sequence,"ok":false,"error":e,"pid":std::process::id()})
            }
        };
        // A broken control channel does not change an authenticated run/Pause.
        if write_frame(&mut output, &response).is_err() {
            loop {
                std::thread::park();
            }
        }
        if request["command"] == "shutdown" {
            break;
        }
        request = match read_frame(&mut input) {
            Ok(r) => r,
            Err(_) => loop {
                std::thread::park();
            },
        };
        sequence = sequence.checked_add(1).ok_or("sequence_overflow")?;
        if request["protocol"] != 1
            || request["session"] != session
            || request["sequence"] != sequence
        {
            return Err("stale_session_or_sequence".into());
        }
    }
    drop(engine);
    Ok(())
}

struct Job(HANDLE);
unsafe impl Send for Job {}
unsafe impl Sync for Job {}
impl Drop for Job {
    fn drop(&mut self) {
        unsafe {
            CloseHandle(self.0);
        }
    }
}
struct Call {
    value: Value,
    reply: mpsc::SyncSender<Result<Value, String>>,
}
#[derive(Clone)]
pub struct Observation {
    pub value: Value,
    pub seen: Instant,
    pub target: Option<Value>,
}
pub struct Host {
    tx: mpsc::SyncSender<Call>,
    pub observation: Mutex<Option<Observation>>,
    child: Mutex<Option<Child>>,
    job: Mutex<Option<Job>>,
    stopped: AtomicBool,
    pub session: String,
}
impl Host {
    pub fn start(config: &Config) -> Result<Arc<Self>, String> {
        settings(config)?;
        let job = unsafe { CreateJobObjectW(std::ptr::null(), std::ptr::null()) };
        if job.is_null() {
            return Err("job_create_failed".into());
        }
        let job = Job(job);
        let mut limits: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = unsafe { std::mem::zeroed() };
        limits.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
        if unsafe {
            SetInformationJobObject(
                job.0,
                JobObjectExtendedLimitInformation,
                (&limits as *const JOBOBJECT_EXTENDED_LIMIT_INFORMATION).cast(),
                std::mem::size_of_val(&limits) as u32,
            )
        } == 0
        {
            return Err("job_config_failed".into());
        }
        let mut child = Command::new(std::env::current_exe().map_err(|_| "exe_unavailable")?)
            .arg("--windows-engine-helper")
            .creation_flags(0x08000000)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|_| "engine_spawn_failed")?;
        if unsafe { AssignProcessToJobObject(job.0, child.as_raw_handle().cast()) } == 0 {
            let _ = child.kill();
            let _ = child.wait();
            return Err("job_assign_failed".into());
        }
        let pid = child.id();
        let mut input = child.stdin.take().ok_or("pipe_missing")?;
        let mut output = child.stdout.take().ok_or("pipe_missing")?;
        let session = vxkey_runtime::bridge::fresh_id()?;
        let (tx, rx) = mpsc::sync_channel::<Call>(8);
        let (read_tx, read_rx) = mpsc::sync_channel(1);
        std::thread::spawn(move || {
            loop {
                let result = read_frame(&mut output);
                let failed = result.is_err();
                if read_tx.send(result).is_err() || failed {
                    break;
                }
            }
        });
        let wire_session = session.clone();
        std::thread::spawn(move || {
            let mut seq = 0u64;
            while let Ok(call) = rx.recv() {
                seq += 1;
                let mut value = call.value;
                value["protocol"] = json!(1);
                value["session"] = json!(wire_session);
                value["sequence"] = json!(seq);
                let result = write_frame(&mut input, &value)
                    .and_then(|_| {
                        read_rx
                            .recv_timeout(Duration::from_secs(2))
                            .map_err(|_| "engine_unresponsive".to_owned())?
                    })
                    .and_then(|v| {
                        if v["protocol"] == 1
                            && v["session"] == wire_session
                            && v["sequence"] == seq
                            && v["pid"] == pid
                        {
                            Ok(v)
                        } else {
                            Err("invalid_peer_or_sequence".into())
                        }
                    });
                let failed = result.is_err();
                let _ = call.reply.try_send(result);
                if failed {
                    break;
                }
            }
        });
        let host = Arc::new(Self {
            tx,
            observation: Mutex::new(None),
            child: Mutex::new(Some(child)),
            job: Mutex::new(Some(job)),
            stopped: AtomicBool::new(false),
            session,
        });
        if let Err(e) = host.request(json!({"command":"bootstrap","config":config})) {
            host.stop();
            return Err(e);
        }
        Ok(host)
    }
    pub fn request(&self, value: Value) -> Result<Value, String> {
        if self.stopped.load(Ordering::Acquire) {
            return Err("engine_stopped".into());
        }
        let (reply, rx) = mpsc::sync_channel(1);
        self.tx
            .try_send(Call { value, reply })
            .map_err(|_| "engine_busy")?;
        let value = rx
            .recv_timeout(Duration::from_secs(3))
            .map_err(|_| "engine_unresponsive")??;
        let mut slot = self.observation.lock().map_err(|_| "unavailable")?;
        let ours = std::env::current_exe()
            .ok()
            .and_then(|p| canonical_app_id(&p.to_string_lossy()));
        let external = value["target"]
            .as_object()
            .filter(|t| t.get("app_id").and_then(Value::as_str) != ours.as_deref())
            .map(|_| value["target"].clone());
        let target = external.or_else(|| slot.as_ref().and_then(|s| s.target.clone()));
        *slot = Some(Observation {
            value: value.clone(),
            seen: Instant::now(),
            target,
        });
        if value["ok"] != true {
            return Err(value["error"].as_str().unwrap_or("engine_rejected").into());
        }
        Ok(value)
    }
    pub fn stop(&self) {
        if self.stopped.swap(true, Ordering::AcqRel) {
            return;
        }
        if let Ok(mut job) = self.job.lock() {
            job.take();
        }
        if let Ok(mut child) = self.child.lock() {
            if let Some(mut child) = child.take() {
                let _ = child.wait();
            }
        }
    }
    pub fn observed(&self) -> Option<Observation> {
        if self.stopped.load(Ordering::Acquire) {
            return None;
        }
        self.observation
            .try_lock()
            .ok()?
            .clone()
            .filter(|s| s.seen.elapsed() < Duration::from_secs(3))
    }
    pub fn set_paused(&self, paused: bool) -> Result<Value, String> {
        let result = self.request(json!({"command":if paused{"pause"}else{"resume"}}));
        if paused && result.is_err() {
            // Explicit emergency Pause is stronger than control continuity.
            // A broken pipe must not leave the user's requested Pause unapplied.
            // Kill only this owned Job; resuming now requires restarting the app.
            self.stop();
            return Ok(json!({"running":false,"restart_required":true}));
        }
        result
    }
}
impl Drop for Host {
    fn drop(&mut self) {
        self.stop();
    }
}

pub fn overlay(host: &Host, data: &mut Value) {
    data["capabilities"]["native_engine"] = json!(true);
    data["capabilities"]["native_shortcuts"] = json!(true);
    data["capabilities"]["windows_history_provider"] = json!(true);
    data["capabilities"]["updater"] = json!(cfg!(feature = "windows-production"));
    data["capabilities"]["autostart"] = json!(true);
    let Some(o) = host.observed() else { return };
    let s = &o.value;
    let running = s["running"] == true;
    let fault = !s["fault"].is_null();
    data["runtime"]["connection_status"] = json!("connected");
    data["runtime"]["engine_session_id"] = json!(host.session);
    data["runtime"]["driver_kind"] = json!("windows_hook_stream");
    data["runtime"]["hook_health"] = s["hook_health"].clone();
    data["runtime"]["driver_status"] = json!(if running && !fault {
        "active"
    } else {
        "inactive"
    });
    data["runtime"]["engine_status"] = json!(if fault {
        "degraded"
    } else if running {
        "running"
    } else {
        "suspended"
    });
    data["runtime"]["health_state"] = json!(if fault {
        "degraded"
    } else if running {
        "healthy"
    } else {
        "suspended"
    });
    data["runtime"]["driver_fault"] = s["fault"].clone();
    data["runtime"]["applied_revision"] = s["revision"].clone();
    let app = o.target.as_ref().and_then(|t| t["app_id"].as_str());
    let excluded = app
        .and_then(|a| data["config"]["apps"]["policies"][a]["input_bypass"].as_bool())
        .unwrap_or(false);
    data["runtime"]["effective_mode"] = json!(if !running {
        "suspended"
    } else if excluded {
        "bypass"
    } else if s["vi"] == true {
        "vi"
    } else {
        "en"
    });
    data["runtime"]["period_substitution"]["effective_enabled"] =
        json!(running && data["config"]["typing"]["double_space_period"] == true);
    data["runtime"]["period_substitution"]["reason"] = json!(if running {
        "engine_enabled"
    } else {
        "suspended"
    });
    if let Some(t) = o.target {
        let app = t["app_id"].as_str().unwrap_or("");
        let name = app
            .rsplit('\\')
            .next()
            .unwrap_or(app)
            .trim_end_matches(".exe");
        data["runtime"]["current_app"] = json!({"app_id":app,"display_name":name,"pid":t["pid"],"engine_session_id":host.session,"focus_generation":t["process_start"],"target_token":format!("{}:{}:{}",t["pid"],t["process_start"],app),"input_bypass":excluded});
    }
}

pub fn dispatch(host: &Host, service: &Mutex<Service>, surface: Surface, request: Value) -> Value {
    let run = || -> Result<Value, String> {
        let mut service = service.lock().map_err(|_| "unavailable")?;
        let command = request["command"]
            .as_str()
            .ok_or("invalid_argument")?
            .to_owned();
        let expected: Option<&[&str]> = match command.as_str() {
            "set_language" => Some(&["command", "mode", "target"]),
            "set_current_app_bypass" => Some(&[
                "command",
                "target_token",
                "expected_revision",
                "input_bypass",
            ]),
            "set_input_suspended" => Some(&["command", "target_session_id", "suspended", "reason"]),
            "test_keystroke" => Some(&["command", "action", "key"]),
            _ => None,
        };
        if let Some(fields) = expected {
            if request.as_object().is_none_or(|o| {
                o.len() != fields.len() || o.keys().any(|k| !fields.contains(&k.as_str()))
            }) {
                return Err("invalid_argument".into());
            }
        }
        if command == "test_keystroke" {
            if !matches!(surface, Surface::Settings | Surface::Onboarding) {
                return Err("permission_denied".into());
            }
            let o = host.observed().ok_or("unavailable")?;
            if o.value["running"] != true || !o.value["fault"].is_null() {
                return Err("unavailable".into());
            }
            let mut config = service.store.config.clone();
            config.typing.language_mode = if o.value["vi"] == true {
                "vi".into()
            } else {
                "en".into()
            };
            let action = request["action"].as_str().ok_or("invalid_argument")?;
            let (replace, insert) = match action {
                "reset" => {
                    service.test_session.clear();
                    service.test_session.configure(&config);
                    (0, String::new())
                }
                "key" => {
                    let text = request["key"].as_str().ok_or("invalid_argument")?;
                    let mut chars = text.chars();
                    let ch = chars.next().ok_or("invalid_argument")?;
                    if chars.next().is_some() {
                        return Err("invalid_argument".into());
                    }
                    service.test_session.configure(&config);
                    let result = service.test_session.process(ch, &config);
                    if result.action == funput_engine::Action::Send {
                        (result.backspace, result.output)
                    } else {
                        (0, ch.to_string())
                    }
                }
                "backspace" => {
                    let result = service.test_session.backspace(&config);
                    if result.action == funput_engine::Action::Send {
                        (result.backspace, result.output)
                    } else {
                        (1, String::new())
                    }
                }
                _ => return Err("invalid_argument".into()),
            };
            return Ok(
                json!({"replace_chars":replace,"insert":insert,"buffer":service.test_session.buffer(),"mode":config.typing.language_mode,"input_method":config.typing.input_method}),
            );
        }
        if command == "set_input_suspended" {
            if !matches!(surface, Surface::Tray | Surface::Settings) {
                return Err("permission_denied".into());
            }
            if request["target_session_id"] != host.session || request["reason"] != "user" {
                return Err("stale_target".into());
            }
            let paused = request["suspended"].as_bool().ok_or("invalid_argument")?;
            host.set_paused(paused)?;
            return Ok(json!({"suspended":paused,"engine_session_id":host.session}));
        }
        let before = service.store.config.revision.clone();
        if command == "patch_config"
            && request["changes"]["system"]
                .get("launch_companion_at_login")
                .is_some()
        {
            if surface != Surface::Settings {
                return Err("permission_denied".into());
            }
            if request["expected_revision"] != before {
                return Err("revision_conflict".into());
            }
            let enabled = request["changes"]["system"]["launch_companion_at_login"]
                .as_bool()
                .ok_or("invalid_argument")?;
            // This command handles one explicit setting; never silently ignore
            // other changes bundled with an autostart mutation.
            if request["changes"].as_object().is_none_or(|o| o.len() != 1)
                || request["changes"]["system"]
                    .as_object()
                    .is_none_or(|o| o.len() != 1)
            {
                return Err("invalid_argument".into());
            }
            let key = windows_registry::CURRENT_USER
                .create(r"Software\Microsoft\Windows\CurrentVersion\Run")
                .map_err(|_| "autostart_failed")?;
            let previous = match key.get_value(RUN_VALUE) {
                Ok(value) => Some(value),
                Err(e) if e.code().0 as u32 == 0x80070002 => None,
                Err(_) => return Err("autostart_failed".into()),
            };
            if enabled {
                let exe = std::env::current_exe().map_err(|_| "exe_unavailable")?;
                key.set_string(RUN_VALUE, format!("\"{}\" --startup", exe.display()))
                    .map_err(|_| "autostart_failed")?;
            } else if previous.is_some() {
                key.remove_value(RUN_VALUE)
                    .map_err(|_| "autostart_failed")?;
            }
            let mut c = service.store.config.clone();
            c.system.launch_companion_at_login = enabled;
            if let Err(error) = service.store.replace(c, &before) {
                let restored = if let Some(value) = previous {
                    key.set_value(RUN_VALUE, &value)
                } else {
                    key.remove_value(RUN_VALUE)
                };
                return Err(if restored.is_err() {
                    "autostart_rollback_failed".into()
                } else {
                    error
                });
            }
            host.request(json!({"command":"configure","config":service.store.config}))?;
            return Ok(
                json!({"revision":service.store.config.revision,"applied_revision":service.store.config.revision,"engine_ack_state":"applied"}),
            );
        }
        let mut response = if command == "set_language" || command == "set_current_app_bypass" {
            if surface != Surface::Tray {
                return Err("permission_denied".into());
            }
            let o = host.observed().ok_or("unavailable")?;
            let t = o.target.ok_or("stale_target")?;
            let app = t["app_id"].as_str().ok_or("stale_target")?;
            let mut c = service.store.config.clone();
            if command == "set_language" {
                if request["target"]["app_id"] != app
                    || request["target"]["engine_session_id"] != host.session
                    || request["target"]["pid"] != t["pid"]
                    || request["target"]["focus_generation"] != t["process_start"]
                {
                    return Err("stale_target".into());
                }
                let mode = request["mode"]
                    .as_str()
                    .filter(|m| ["vi", "en"].contains(m))
                    .ok_or("invalid_argument")?;
                c.typing.language_mode = mode.into();
                if c.apps.remember_per_app {
                    c.apps.language_memory.insert(app.into(), mode.into());
                }
            } else {
                let token = format!("{}:{}:{}", t["pid"], t["process_start"], app);
                if request["target_token"] != token || request["expected_revision"] != before {
                    return Err("stale_target".into());
                }
                c.apps
                    .policies
                    .entry(app.into())
                    .or_insert(vxkey_runtime::config::AppPolicy {
                        input_bypass: false,
                        clipboard_blocked: false,
                    })
                    .input_bypass = request["input_bypass"]
                    .as_bool()
                    .ok_or("invalid_argument")?;
            }
            settings(&c)?;
            service.store.replace(c, &before)?;
            json!({"ok":true,"data":{"revision":service.store.config.revision},"error":null})
        } else {
            // Validate physical Windows bindings before persistence; unsupported
            // functionality keeps the existing explicit capability/error contract.
            if command == "patch_config" {
                let mut c =
                    serde_json::to_value(&service.store.config).map_err(|_| "invalid_config")?;
                if let Some(changes) = request["changes"].as_object() {
                    for (group, values) in changes {
                        if let Some(values) = values.as_object() {
                            for (key, value) in values {
                                c[group][key] = value.clone();
                            }
                        }
                    }
                }
                let c: Config = serde_json::from_value(c).map_err(|_| "invalid_config")?;
                settings(&c)?;
            }
            service.native_period = vxkey_runtime::period::NativePeriod::NotApplicable;
            service.dispatch_from(surface, request)
        };
        if response["ok"] == true && before != service.store.config.revision {
            host.request(json!({"command":"configure","config":service.store.config}))?;
            response["data"]["applied_revision"] = json!(service.store.config.revision);
            response["data"]["engine_ack_state"] = json!("applied");
        }
        if command == "get_snapshot" && response["ok"] == true {
            overlay(host, &mut response["data"]);
            response["data"]["version"] = json!(env!("CARGO_PKG_VERSION"));
        }
        Ok(response)
    };
    match run() {
        Ok(v) if v.get("ok").is_some() => v,
        Ok(data) => json!({"ok":true,"data":data,"error":null,"request_id":null}),
        Err(e) => super::failure(&e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn disconnected_host() -> Host {
        let (tx, rx) = mpsc::sync_channel(1);
        drop(rx);
        Host {
            tx,
            observation: Mutex::new(Some(Observation {
                value: json!({"running":true}),
                seen: Instant::now(),
                target: None,
            })),
            child: Mutex::new(None),
            job: Mutex::new(None),
            stopped: AtomicBool::new(false),
            session: vxkey_runtime::bridge::fresh_id().unwrap(),
        }
    }
    #[test]
    fn explicit_pause_stops_owned_host_if_control_is_broken() {
        let host = disconnected_host();
        assert!(host.observed().is_some());
        assert!(host.set_paused(true).is_ok());
        assert!(host.stopped.load(Ordering::Acquire));
        assert!(host.observed().is_none());
    }
    #[test]
    fn failed_resume_does_not_change_existing_run_state() {
        let host = disconnected_host();
        assert!(host.set_paused(false).is_err());
        assert!(!host.stopped.load(Ordering::Acquire));
    }
    #[test]
    fn namespace_and_updater_profile_are_explicit() {
        if cfg!(feature = "windows-production") {
            assert_eq!(APP_ID, "com.vxkey.desktop");
            assert_eq!(DATA_NAME, "VXKey");
        } else {
            assert_eq!(APP_ID, "com.vxkey.desktop.test.windows");
            assert_eq!(DATA_NAME, "VXKeyTest");
        }
    }
    #[test]
    fn windows_config_is_durable_across_reopen() {
        let root = std::env::temp_dir().join(format!(
            "vxkey_config_fixture_{}",
            vxkey_runtime::bridge::fresh_id().unwrap()
        ));
        assert_eq!(root.parent(), Some(std::env::temp_dir().as_path()));
        {
            let mut store = vxkey_runtime::storage::ConfigStore::open(
                root.join("config.json"),
                vxkey_runtime::config::Platform::Windows,
            )
            .unwrap();
            store.save_initial().unwrap();
            let mut next = store.config.clone();
            next.typing.input_method = "vni".into();
            next.apps
                .language_memory
                .insert(r"c:\fixture\editor.exe".into(), "en".into());
            let revision = store.config.revision.clone();
            store.replace(next, &revision).unwrap();
        }
        {
            let store = vxkey_runtime::storage::ConfigStore::open(
                root.join("config.json"),
                vxkey_runtime::config::Platform::Windows,
            )
            .unwrap();
            assert_eq!(store.config.typing.input_method, "vni");
            assert_eq!(store.config.revision, "2");
            assert_eq!(
                store.config.apps.language_memory[r"c:\fixture\editor.exe"],
                "en"
            );
        }
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn windows_mapping_preserves_core_and_native_history() {
        let c = Config::fresh(vxkey_runtime::config::Platform::Windows);
        let s = settings(&c).unwrap();
        assert_eq!(
            s.toggle,
            Some(Binding {
                modifiers: 9,
                key: None
            })
        );
        assert!(s.double_space_period);
        assert!(s.flip.is_none());
    }
    #[test]
    fn bounded_frame_rejects_oversize_before_body() {
        assert!(read_frame(&mut ((LIMIT as u32 + 1).to_le_bytes().as_slice())).is_err());
    }
    #[test]
    fn app_path_policy_is_not_a_mac_bundle_id() {
        let mut c = Config::fresh(vxkey_runtime::config::Platform::Windows);
        c.apps.policies.insert(
            "c:\\apps\\editor.exe".into(),
            vxkey_runtime::config::AppPolicy {
                input_bypass: true,
                clipboard_blocked: false,
            },
        );
        assert!(
            settings(&c)
                .unwrap()
                .excluded_apps
                .contains("c:\\apps\\editor.exe")
        );
    }
}
