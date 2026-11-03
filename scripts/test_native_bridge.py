"""Ad-hoc pinned, two-process IPC/fault fixture. Never installs/selects an IME."""
import hashlib, json, os, pathlib, shutil, socket, subprocess, threading, time, uuid
ROOT = pathlib.Path(__file__).resolve().parents[1]
OUT = ROOT / 'build' / 'bridge_fixture'
DATA = OUT / 'data'
OUT.mkdir(parents=True, exist_ok=True)
DATA.mkdir(mode=0o700, exist_ok=True)
os.chmod(DATA, 0o700)
# This root contains only this script's generated fixtures.
for name in ['config.json', 'recovery_state.json']:
    path = DATA / name
    if path.exists(): path.unlink()
source = ROOT / 'target/debug/vxkey_bridge_fixture'
paths = {}
def sign(role, identifier, extra=()):
    path = OUT / role
    shutil.copyfile(source, path); path.chmod(0o700)
    subprocess.run(['codesign', '--force', '--sign', '-', '--identifier', identifier, *extra, str(path)], check=True, capture_output=True)
    info = subprocess.run(['codesign', '-d', '--verbose=3', str(path)], check=True, capture_output=True, text=True).stderr
    return path, next(line.split('=', 1)[1] for line in info.splitlines() if line.startswith('CDHash='))
paths['host'], host_hash = sign('host', 'com.vxkey.desktop')
paths['client'], client_hash = sign('client', 'com.vxkey.engine')
paths['intruder'], intruder_hash = sign('intruder', 'com.vxkey.engine', ['--options', 'runtime'])
assert intruder_hash != client_hash
pair = DATA / 'bridge_pair.json'
pair.write_text(json.dumps({'companion': host_hash, 'engine': client_hash})); pair.chmod(0o600)
env = dict(os.environ, VXKEY_DATA_DIR=str(DATA), VXKEY_BRIDGE_DEV_TRUST='1')
processes = []
checks = []
def spawn(role, args_role=None):
    p = subprocess.Popen([str(paths[role]), args_role or role], stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True, env=env)
    processes.append(p)
    return p

def ask(p, request):
    p.stdin.write(json.dumps(request) + '\n'); p.stdin.flush()
    # Each subprocess has finite wire deadlines; the outer process is bounded too.
    import select
    if not select.select([p.stdout], [], [], 5)[0]:
        raise AssertionError('fixture response deadline')
    line = p.stdout.readline()
    if not line: raise AssertionError('fixture exited: ' + p.stderr.read()[:200])
    return json.loads(line)

def wait_for(fn, seconds=8):
    end = time.monotonic() + seconds
    while time.monotonic() < end:
        value = fn()
        if value: return value
        time.sleep(0.05)
    raise AssertionError('condition deadline')

lock = threading.Lock()
model = {'driver': {'accessibility': True, 'listen_access': True, 'post_access': True, 'attached': False}, 'engine_session_id': str(uuid.uuid4()), 'loop_sequence': '0', 'active': True,
    'target': None, 'composing': False, 'mode': 'vi', 'suspended': True,
    'applied_revision': '0', 'result': None, 'memory': None}
model['target'] = {'app_id': 'fixture.editor', 'pid': 4242, 'engine_session_id': model['engine_session_id'], 'focus_generation': '1'}
stop = threading.Event(); freeze = threading.Event(); delivery = {}; pending_config = None
client = None

def pump():
    global delivery, pending_config
    while not stop.is_set():
        if freeze.is_set(): time.sleep(0.02); continue
        with lock:
            model['loop_sequence'] = str(int(model['loop_sequence']) + 1)
            model['driver']['attached'] = not model['suspended']
            status = dict(model)
        d = ask(client, status)
        with lock:
            delivery = d
            if d.get('config'): pending_config = d['config']
            if pending_config and not model['composing']:
                model['applied_revision'] = pending_config['revision']; pending_config = None
            model['suspended'] = not d.get('connected') or d.get('latched', True) or model['applied_revision'] == '0'
            control = d.get('control')
            if control:
                outcome = 'stale_target'
                if control['action'] == 'set_language' and control['target'] == model['target'] and not model['composing'] and not model['suspended']:
                    model['mode'] = control['mode']; outcome = 'applied'
                    model['memory'] = {'app_id': model['target']['app_id'], 'mode': model['mode'], 'base_revision': model['applied_revision']}
                elif control['action'] == 'suspend' and control['engine_session_id'] == model['engine_session_id']:
                    outcome = 'applied'
                    if control['suspended']: model['suspended'] = True
                model['result'] = {'id': control['id'], 'outcome': outcome}
        time.sleep(0.05)

try:
    # An untrusted same-UID server must not receive even the IME handshake.
    runtime = DATA / 'runtime'; runtime.mkdir(mode=0o700, exist_ok=True); runtime.chmod(0o700)
    endpoint = runtime / 'engine.sock'
    if endpoint.exists(): endpoint.unlink()
    fake = socket.socket(socket.AF_UNIX); fake.settimeout(4); fake.bind(str(endpoint)); fake.listen(1)
    rejected_client = spawn('client')
    assert json.loads(rejected_client.stdout.readline()) == {'ready': True}
    conn, _ = fake.accept(); conn.settimeout(3); assert conn.recv(4) == b''; conn.close(); fake.close(); endpoint.unlink()
    rejected_client.terminate(); rejected_client.wait(timeout=5)
    checks.append('IME rejects unpinned same-UID server before sending handshake or status')
    host = spawn('host')
    assert json.loads(host.stdout.readline()) == {'ready': True}
    intruder = spawn('intruder', 'client')
    assert intruder.wait(timeout=5) != 0
    checks.append('same-ID different CDHash process rejected at own identity check')
    # A different same-UID executable reaches the real endpoint. Host must close
    # before any config/handshake data is disclosed, not merely reject its status.
    s = socket.socket(socket.AF_UNIX); s.settimeout(3); s.connect(str(DATA / 'runtime/engine.sock'))
    assert s.recv(4) == b''; s.close()
    assert not (DATA / 'recovery_state.json').exists()
    checks.append('same-UID unsigned/unpinned socket peer gets EOF, no config and no fault-latch DoS')
    client = spawn('client')
    assert json.loads(client.stdout.readline()) == {'ready': True}
    thread = threading.Thread(target=pump, daemon=True); thread.start()
    wait_for(lambda: delivery.get('connected') and not model['suspended'])
    snapshot = wait_for(lambda: (x if (x := ask(host, {'command': 'get_snapshot'}))['data']['runtime']['applied_revision'] == '1' else None))
    assert snapshot['data']['capabilities']['native_engine'] is True
    assert snapshot['data']['runtime']['effective_mode'] == 'vi'
    checks.append('mutual audit-token/CDHash handshake, applied revision and live mode')
    with lock: model['composing'] = True
    revision = snapshot['data']['revision']
    patch = ask(host, {'command': 'patch_config', 'expected_revision': revision, 'changes': {'typing': {'input_method': 'vni'}}})
    assert patch['ok'] and patch['data']['engine_ack_state'] == 'pending'
    wait_for(lambda: pending_config is not None)
    assert model['applied_revision'] == '1'
    with lock: model['composing'] = False
    wait_for(lambda: model['applied_revision'] == patch['data']['persisted_revision'])
    wait_for(lambda: ask(host, {'command': 'get_snapshot'})['data']['runtime']['applied_revision'] == model['applied_revision'])
    checks.append('whole native config deferred while composing, ack after safe boundary')
    with lock: target = dict(model['target'])
    wrong = dict(target, focus_generation='0')
    stale = ask(host, {'fixture': 'control', 'control': {'action': 'set_language', 'id': str(uuid.uuid4()), 'mode': 'en', 'target': wrong, 'expected_loop_sequence': model['loop_sequence']}})
    assert not stale['ok'] and stale['error'] == 'stale_target'
    mode = ask(host, {'fixture': 'control', 'control': {'action': 'set_language', 'id': str(uuid.uuid4()), 'mode': 'en', 'target': target, 'expected_loop_sequence': model['loop_sequence']}})
    assert mode['ok'] and mode['status']['mode'] == 'en'
    wait_for(lambda: ask(host, {'command': 'get_snapshot'})['data']['config']['apps']['language_memory'].get('fixture.editor') == 'en')
    checks.append('session/focus-bound mode action, stale rejection and revision-bound memory writeback')
    # Stop ONLY main-loop ticks. Rust worker is alive and still exchanges packets.
    freeze.set()
    wait_for(lambda: (DATA / 'recovery_state.json').exists(), seconds=6)
    marker = json.loads((DATA / 'recovery_state.json').read_text())
    assert marker['suspended'] and marker['reason_code'] in ['event_loop_stalled', 'bridge_disconnected']
    wait_for(lambda: ask(host, {'command': 'get_snapshot'})['data']['capabilities']['native_engine'] is False)
    checks.append('main-loop freeze detected despite live background worker; durable suspension and stale UI invalidation')
    with lock: delivery = {}
    freeze.clear()
    wait_for(lambda: delivery.get('connected') and delivery.get('latched') and model['suspended'], seconds=10)
    checks.append('reconnect retains safe-start latch and never auto-resumes')
    stale_resume = ask(host, {'fixture': 'control', 'control': {'action': 'suspend', 'id': str(uuid.uuid4()), 'suspended': False, 'engine_session_id': str(uuid.uuid4())}})
    assert not stale_resume['ok'] and (DATA / 'recovery_state.json').exists()
    resume = ask(host, {'fixture': 'control', 'control': {'action': 'suspend', 'id': str(uuid.uuid4()), 'suspended': False, 'engine_session_id': model['engine_session_id']}})
    assert resume['ok']
    wait_for(lambda: not model['suspended'])
    assert not (DATA / 'recovery_state.json').exists()
    checks.append('explicit same-session healthy Resume clears marker; stale session cannot')
    host.kill(); host.wait(timeout=5)
    wait_for(lambda: (DATA / 'recovery_state.json').exists())
    wait_for(lambda: model['suspended'])
    checks.append('companion crash suspends client without blocking main-loop tick')
    stop.set(); thread.join(timeout=5); client.terminate(); client.wait(timeout=5)
    host = spawn('host'); assert json.loads(host.stdout.readline()) == {'ready': True}
    assert ask(host, {'fixture': 'latch'})['latched']
    assert ask(host, {'command': 'get_snapshot'})['data']['config']['typing']['input_method'] == 'vni'
    checks.append('process restart retains persisted config and fault marker')
    report = {'status': 'pass', 'scope': 'macOS signed two-process transport and simulated main-loop fixture; no IMK activation',
        'checks': checks, 'host_cdhash': host_hash, 'client_cdhash': client_hash,
        'source_binary_sha256': hashlib.sha256(source.read_bytes()).hexdigest(),
        'ime_installed': False, 'ime_activated': False, 'source_selection_tested': False,
        'production_team_identity_tested': False}
    (ROOT / 'docs/qa/native_bridge_evidence.json').write_text(json.dumps(report, ensure_ascii=False, indent=2) + '\n')
    print(json.dumps(report, ensure_ascii=False, indent=2))
finally:
    stop.set()
    for p in processes:
        if p.poll() is None:
            p.terminate()
            try: p.wait(timeout=5)
            except subprocess.TimeoutExpired: p.kill(); p.wait(timeout=5)
