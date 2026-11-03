use super::*;
use std::{
    fs::{self, File, OpenOptions},
    os::unix::{
        fs::{DirBuilderExt, FileTypeExt, MetadataExt, OpenOptionsExt, PermissionsExt},
        net::{UnixListener, UnixStream},
    },
    path::{Path, PathBuf},
    sync::{Arc, Mutex, mpsc},
    thread,
};

// Development metadata only, on bridge workers, bounded and never on key callbacks.
fn dev_trace(latch: &Latch, event: &str, detail: serde_json::Value) {
    #[cfg(any(debug_assertions, feature = "development-bridge"))]
    if std::env::var("VXKEY_BRIDGE_DEV_TRUST").as_deref() == Ok("1") {
        let path = latch.root.join("bridge_trace.jsonl");
        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .append(true)
            .mode(0o600)
            .custom_flags(libc::O_NOFOLLOW)
            .open(path)
        {
            if file
                .metadata()
                .is_ok_and(|m| m.is_file() && m.len() < 65536)
            {
                let _ = writeln!(
                    file,
                    "{}",
                    serde_json::json!({"event":event,"detail":detail})
                );
            }
        }
    }
    #[cfg(not(any(debug_assertions, feature = "development-bridge")))]
    let _ = (latch, event, detail);
}

/// Shared latch serialized across companion/native worker. A corrupt or unreadable
/// marker always means suspended. Never consulted by the keyboard callback.
#[derive(Clone)]
pub struct Latch {
    root: PathBuf,
}
impl Latch {
    pub fn new(root: &Path) -> Result<Self, String> {
        secure_directory(root)?;
        Ok(Self { root: root.into() })
    }
    pub fn is_set(&self) -> bool {
        match self.root.join("recovery_state.json").symlink_metadata() {
            Ok(_) => true,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => false,
            Err(_) => true,
        }
    }
    fn lock(&self) -> Result<File, String> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .mode(0o600)
            .custom_flags(libc::O_NOFOLLOW)
            .open(self.root.join("recovery.lock"))
            .map_err(|_| "recovery_unavailable")?;
        file.lock().map_err(|_| "recovery_unavailable")?;
        Ok(file)
    }
    pub fn set(&self, reason: &str) -> Result<(), String> {
        let _lock = self.lock()?;
        if self.is_set() {
            return Ok(());
        }
        let id = fresh_id()?;
        let temp = self.root.join(format!("recovery_tmp_{id}"));
        let bytes=serde_json::to_vec(&serde_json::json!({"schema_version":1,"suspended":true,"reason_code":reason,"failed_component_version":env!("CARGO_PKG_VERSION"),"incident_id":id})).map_err(|_|"recovery_unavailable")?;
        let result = (|| {
            let mut f = OpenOptions::new()
                .write(true)
                .create_new(true)
                .mode(0o600)
                .open(&temp)
                .map_err(|_| "recovery_unavailable")?;
            f.write_all(&bytes)
                .and_then(|_| f.sync_all())
                .map_err(|_| "recovery_unavailable")?;
            fs::rename(&temp, self.root.join("recovery_state.json"))
                .map_err(|_| "recovery_unavailable")?;
            File::open(&self.root)
                .and_then(|f| f.sync_all())
                .map_err(|_| "recovery_unavailable")
        })();
        if result.is_err() {
            let _ = fs::remove_file(temp);
        }
        Ok(result?)
    }
    /// Only the supervisor's explicit Resume transaction calls this with a live,
    /// session-bound main-loop acknowledgement. A changed incident cannot be cleared.
    pub fn incident(&self) -> Option<Vec<u8>> {
        let file = OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_NOFOLLOW)
            .open(self.root.join("recovery_state.json"))
            .ok()?;
        let mut bytes = vec![];
        file.take(4097).read_to_end(&mut bytes).ok()?;
        if bytes.len() > 4096 {
            return None;
        }
        Some(bytes)
    }
    pub fn clear(&self, expected: Option<&[u8]>) -> Result<(), String> {
        let _lock = self.lock()?;
        if self.is_set() && (expected.is_none() || self.incident().as_deref() != expected) {
            return Err("incident_changed".into());
        }
        match fs::remove_file(self.root.join("recovery_state.json")) {
            Ok(()) => File::open(&self.root)
                .and_then(|f| f.sync_all())
                .map_err(|_| "recovery_unavailable".into()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(_) => Err("recovery_unavailable".into()),
        }
    }
}
fn secure_directory(root: &Path) -> Result<(), String> {
    if !root.is_absolute() {
        return Err("invalid_path".into());
    }
    fs::DirBuilder::new()
        .recursive(true)
        .mode(0o700)
        .create(root)
        .map_err(|_| "runtime_unavailable")?;
    let m = fs::symlink_metadata(root).map_err(|_| "runtime_unavailable")?;
    // SAFETY: geteuid has no arguments or borrowed memory.
    if !m.is_dir() || m.uid() != unsafe { libc::geteuid() } || m.mode() & 0o077 != 0 {
        return Err("unsafe_runtime_directory".into());
    }
    Ok(())
}
fn socket_path(root: &Path) -> Result<PathBuf, String> {
    let dir = root.join("runtime");
    secure_directory(&dir)?;
    let path = dir.join("engine.sock");
    if path.as_os_str().as_encoded_bytes().len() > 103 {
        return Err("socket_path_too_long".into());
    }
    Ok(path)
}
fn deadline(stream: &UnixStream) -> Result<(), String> {
    stream
        .set_read_timeout(Some(Duration::from_secs(1)))
        .and_then(|_| stream.set_write_timeout(Some(Duration::from_secs(1))))
        .map_err(|_| "disconnected".into())
}

struct Pending {
    control: Control,
    reply: mpsc::SyncSender<Result<NativeStatus, String>>,
    expires: Instant,
}
#[derive(Clone)]
pub struct Host {
    queue: mpsc::SyncSender<Pending>,
    pub latch: Latch,
}
impl Host {
    #[cfg(target_os = "macos")]
    pub fn start(
        root: &Path,
        service: Arc<Mutex<crate::service::Service>>,
    ) -> Result<Self, String> {
        let latch = Latch::new(root)?;
        let trust = super::trust::Trust::load(root, true)?;
        let path = socket_path(root)?;
        let owner = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .mode(0o600)
            .custom_flags(libc::O_NOFOLLOW)
            .open(path.with_extension("lock"))
            .map_err(|_| "runtime_unavailable")?;
        owner.try_lock().map_err(|_| "bridge_in_use")?;
        match fs::symlink_metadata(&path) {
            Ok(m) if m.file_type().is_socket() => {
                fs::remove_file(&path).map_err(|_| "runtime_unavailable")?;
            }
            Ok(_) => return Err("unsafe_socket_path".into()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(_) => return Err("runtime_unavailable".into()),
        }
        let listener = UnixListener::bind(&path).map_err(|_| "runtime_unavailable")?;
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600))
            .map_err(|_| "runtime_unavailable")?;
        let (tx, rx) = mpsc::sync_channel(8);
        let worker_latch = latch.clone();
        thread::Builder::new()
            .name("vxkey_bridge".into())
            .spawn(move || {
                let _owner = owner;
                for stream in listener.incoming() {
                    let Ok(mut stream) = stream else { continue };
                    if deadline(&stream).is_err() || !trust.check(&stream) {
                        continue;
                    }
                    let authenticated = host_session(&mut stream, &rx, &service, &worker_latch);
                    // Failed unauthenticated handshakes must not suspend another session.
                    if authenticated {
                        let _ = worker_latch.set("bridge_disconnected");
                        if let Ok(mut s) = service.lock() {
                            s.native = None;
                            s.suspended = true;
                        }
                        // Background driver detaches on its lease expiry. Never select
                        // another keyboard layout as a recovery mechanism.
                    }
                    while let Ok(p) = rx.try_recv() {
                        let _ = p.reply.try_send(Err("disconnected".into()));
                    }
                }
            })
            .map_err(|_| "runtime_unavailable")?;
        Ok(Self { queue: tx, latch })
    }
    pub fn request(&self, control: Control) -> Result<NativeStatus, String> {
        control.validate()?;
        let (tx, rx) = mpsc::sync_channel(1);
        self.queue
            .try_send(Pending {
                control,
                reply: tx,
                expires: Instant::now() + Duration::from_secs(2),
            })
            .map_err(|_| "busy")?;
        rx.recv_timeout(Duration::from_secs(2))
            .map_err(|_| "outcome_unknown")?
    }
}
#[cfg(target_os = "macos")]
fn host_session(
    stream: &mut UnixStream,
    queue: &mpsc::Receiver<Pending>,
    service: &Mutex<crate::service::Service>,
    latch: &Latch,
) -> bool {
    let Ok(mut wire) = Wire::accept(stream) else {
        return false;
    };
    if wire
        .send(
            stream,
            Message::Handshake(Handshake {
                component: "companion".into(),
                app_version: env!("CARGO_PKG_VERSION").into(),
            }),
        )
        .is_err()
    {
        return false;
    }
    let mut progress = Progress::default();
    let mut revision = String::new();
    let mut pending: Option<(Pending, Option<Vec<u8>>)> = None;
    let mut seen_controls = std::collections::VecDeque::<String>::new();
    let mut traced = String::new();
    loop {
        let status = match wire.read(stream) {
            Ok(Message::Status(status)) => status,
            Ok(_) => {
                dev_trace(latch, "host_message_type", serde_json::json!({}));
                break;
            }
            Err(error) => {
                dev_trace(latch, "host_read_error", serde_json::json!({"error":error}));
                break;
            }
        };
        let signature = format!(
            "{:?}/{}/{:?}/{:?}",
            status.driver, status.suspended, status.fault, status.result
        );
        if traced != signature {
            dev_trace(
                latch,
                "status",
                serde_json::json!({"sequence":status.loop_sequence,"driver":status.driver,"suspended":status.suspended,"fault":status.fault,"result":status.result,"diagnostics":status.diagnostics}),
            );
            traced = signature;
        }
        if let Some(reason) = &status.fault {
            let _ = latch.set(reason);
        }
        if status.validate().is_err()
            || progress
                .observe(&status.loop_sequence, Instant::now())
                .is_err()
        {
            dev_trace(
                latch,
                "loop_stalled",
                serde_json::json!({"sequence":status.loop_sequence}),
            );
            let _ = latch.set("event_loop_stalled");
            break;
        }
        if let Some((p, incident)) = pending.take() {
            if status
                .result
                .as_ref()
                .is_some_and(|r| r.id == p.control.id())
            {
                let result = status.result.as_ref().unwrap();
                if result.outcome == "applied" {
                    let matches_effect = match &p.control {
                        Control::RequestPermissions { .. } => true,
                        Control::SetLanguage { mode, target, .. } => {
                            status.mode == *mode
                                && status.active
                                && !status.suspended
                                && status.target.as_ref() == Some(target)
                        }
                        Control::Suspend {
                            suspended: true, ..
                        } => status.suspended,
                        Control::Suspend {
                            suspended: false, ..
                        } => status.fault.is_none(),
                    };
                    if !matches_effect {
                        dev_trace(
                            latch,
                            "control_effect_mismatch",
                            serde_json::json!({"control":p.control,"fault":status.fault}),
                        );
                        let _ = p.reply.try_send(Err("outcome_unknown".into()));
                        break;
                    }
                    let resume = matches!(
                        &p.control,
                        Control::Suspend {
                            suspended: false,
                            ..
                        }
                    );
                    // Clear durable fault only after main loop accepted the SAME session.
                    if resume && latch.clear(incident.as_deref()).is_err() {
                        let _ = p.reply.try_send(Err("recovery_unavailable".into()));
                    } else {
                        let _ = p.reply.try_send(Ok(status.clone()));
                    }
                } else {
                    let _ = p.reply.try_send(Err(result.outcome.clone()));
                }
            } else if Instant::now() >= p.expires {
                let _ = p.reply.try_send(Err("outcome_unknown".into()));
            } else {
                pending = Some((p, incident));
            }
        }
        let Ok(mut s) = service.lock() else { break };
        s.accept_native(status.clone(), wire.session.clone());
        let config = if revision != s.store.config.revision {
            revision = s.store.config.revision.clone();
            Some(NativeConfig::from_config(&s.store.config))
        } else {
            None
        };
        drop(s);
        let mut control = None;
        if pending.is_none() {
            if let Ok(p) = queue.try_recv() {
                if Instant::now() >= p.expires {
                    let _ = p.reply.try_send(Err("timeout".into()));
                } else {
                    let unique = !seen_controls.iter().any(|id| id == p.control.id());
                    let valid = unique
                        && match &p.control {
                            Control::SetLanguage {
                                target,
                                expected_loop_sequence,
                                ..
                            } => {
                                status.active
                                    && !status.suspended
                                    && status.target.as_ref() == Some(target)
                                    && config::counter(expected_loop_sequence).is_ok_and(|n| {
                                        config::counter(&status.loop_sequence)
                                            .is_ok_and(|current| n <= current)
                                    })
                            }
                            Control::RequestPermissions {
                                engine_session_id, ..
                            }
                            | Control::Suspend {
                                engine_session_id, ..
                            } => *engine_session_id == status.engine_session_id,
                        };
                    if !valid {
                        let _ = p.reply.try_send(Err("stale_target".into()));
                    } else {
                        if matches!(
                            &p.control,
                            Control::Suspend {
                                suspended: true,
                                ..
                            }
                        ) && latch.set("user_suspended").is_err()
                        {
                            let _ = p.reply.try_send(Err("recovery_unavailable".into()));
                            continue;
                        }
                        seen_controls.push_back(p.control.id().to_string());
                        if seen_controls.len() > 64 {
                            seen_controls.pop_front();
                        }
                        control = Some(p.control.clone());
                        pending = Some((p, latch.incident()));
                    }
                }
            }
        }
        if wire
            .send(
                stream,
                Message::Sync(SyncPayload {
                    config,
                    control,
                    latched: latch.is_set(),
                }),
            )
            .is_err()
        {
            break;
        }
    }
    if let Some((p, _)) = pending {
        let _ = p.reply.try_send(Err("outcome_unknown".into()));
    }
    true
}
#[derive(Default)]
struct ClientState {
    status: Option<NativeStatus>,
    tick: Option<Instant>,
    connected: bool,
    exchange: Option<Instant>,
    latched: bool,
    config: Option<NativeConfig>,
    control: Option<Control>,
}
#[derive(Serialize)]
pub struct Delivery {
    pub connected: bool,
    pub latched: bool,
    pub config: Option<NativeConfig>,
    pub control: Option<Control>,
}
pub struct Client {
    state: Arc<Mutex<ClientState>>,
}
impl Client {
    #[cfg(target_os = "macos")]
    pub fn start(root: &Path) -> Result<Self, String> {
        let latch = Latch::new(root)?;
        let trust = super::trust::Trust::load(root, false)?;
        let path = socket_path(root)?;
        let state = Arc::new(Mutex::new(ClientState {
            latched: latch.is_set(),
            ..Default::default()
        }));
        let shared = state.clone();
        thread::Builder::new()
            .name("vxkey_native_bridge".into())
            .spawn(move || {
                let mut delay = 1;
                let mut ever_connected = false;
                loop {
                    let connected = UnixStream::connect(&path)
                        .ok()
                        .filter(|s| deadline(s).is_ok() && trust.check(s));
                    if let Some(mut stream) = connected {
                        let until = Instant::now() + HEALTH_TIMEOUT;
                        while shared.lock().map(|s| s.status.is_none()).unwrap_or(false)
                            && Instant::now() < until
                        {
                            thread::sleep(Duration::from_millis(10));
                        }
                        let session = client_session(&mut stream, &shared, &latch);
                        ever_connected |= session;
                        if session {
                            delay = 1;
                        }
                    }
                    if ever_connected {
                        let _ = latch.set("bridge_disconnected");
                    }
                    if let Ok(mut s) = shared.lock() {
                        s.connected = false;
                        s.latched = true;
                        s.config = None;
                        s.control = None;
                    }
                    thread::sleep(Duration::from_secs(delay));
                    delay = (delay * 2).min(30);
                }
            })
            .map_err(|_| "runtime_unavailable")?;
        Ok(Self { state })
    }
    /// Main-loop timer only. Never blocks on the worker, disk, socket or fsync.
    pub fn tick(&self, status: NativeStatus) -> Result<Delivery, String> {
        status.validate()?;
        let mut s = self.state.try_lock().map_err(|_| "busy")?;
        s.status = Some(status);
        s.tick = Some(Instant::now());
        Ok(Delivery {
            connected: s.connected && s.exchange.is_some_and(|at| at.elapsed() < HEALTH_TIMEOUT),
            latched: s.latched,
            config: s.config.take(),
            control: s.control.take(),
        })
    }
}
#[cfg(target_os = "macos")]
fn client_session(stream: &mut UnixStream, shared: &Mutex<ClientState>, latch: &Latch) -> bool {
    let Ok(id) = fresh_id() else { return false };
    let Ok(mut wire) = Wire::new(id) else {
        return false;
    };
    if wire
        .send(
            stream,
            Message::Handshake(Handshake {
                component: "engine".into(),
                app_version: env!("CARGO_PKG_VERSION").into(),
            }),
        )
        .is_err()
    {
        return false;
    }
    if !matches!(wire.read(stream),Ok(Message::Handshake(h))if h.component=="companion" && h.app_version==env!("CARGO_PKG_VERSION"))
    {
        return false;
    }
    loop {
        let status = {
            let Ok(s) = shared.lock() else { break };
            if s.tick.is_none_or(|t| t.elapsed() >= HEALTH_TIMEOUT) {
                drop(s);
                let _ = latch.set("event_loop_stalled");
                break;
            }
            s.status.clone()
        };
        let Some(status) = status else { break };
        if let Some(reason) = &status.fault {
            let _ = latch.set(reason);
        }
        if wire.send(stream, Message::Status(status.clone())).is_err() {
            break;
        }
        let Ok(Message::Sync(sync)) = wire.read(stream) else {
            break;
        };
        if sync.config.as_ref().is_some_and(|c| c.validate().is_err()) {
            let _ = latch.set("invalid_native_config");
            break;
        }
        if sync.control.as_ref().is_some_and(|c| c.validate().is_err()) {
            let _ = latch.set("invalid_native_control");
            break;
        }
        let Ok(mut s) = shared.lock() else { break };
        s.connected = true;
        s.exchange = Some(Instant::now());
        s.latched = sync.latched || latch.is_set();
        if let Some(config) = sync.config {
            s.config = Some(config);
        }
        if let Some(control) = sync.control {
            // A control is delivered once. Never replay it on reconnect or timeout.
            if s.control.is_some() {
                drop(s);
                let _ = latch.set("control_queue_overflow");
                break;
            }
            s.control = Some(control);
        }
        drop(s);
        thread::sleep(Duration::from_millis(100));
    }
    true
}

#[cfg(test)]
mod contention_tests {
    use super::*;
    #[test]
    fn main_tick_reports_busy_without_waiting_or_replacing_cached_status() {
        let shared = Arc::new(Mutex::new(ClientState::default()));
        let client = Client {
            state: shared.clone(),
        };
        let status:NativeStatus=serde_json::from_value(serde_json::json!({
            "engine_session_id":fresh_id().unwrap(),"loop_sequence":"1","active":false,"target":null,
            "driver":{"accessibility":true,"listen_access":true,"post_access":true,"attached":false},
            "composing":false,"mode":"vi","suspended":true,"applied_revision":"1","result":null,"memory":null,"fault":null
        })).unwrap();
        let held = shared.lock().unwrap();
        assert_eq!(client.tick(status.clone()).err().as_deref(), Some("busy"));
        assert!(held.status.is_none());
        drop(held);
        assert!(client.tick(status).is_ok());
        assert_eq!(
            shared
                .lock()
                .unwrap()
                .status
                .as_ref()
                .unwrap()
                .loop_sequence,
            "1"
        );
    }
}
