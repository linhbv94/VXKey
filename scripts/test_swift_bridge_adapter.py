"""Actual Swift AppSettings timer/cache + Rust composer; no IMK registration."""
import hashlib, json, os, pathlib, select, shutil, subprocess, time, uuid
ROOT = pathlib.Path(__file__).resolve().parents[1]
OUT = ROOT / 'build/bridge_fixture'
DATA = OUT / 'swift_data'; DATA.mkdir(mode=0o700, exist_ok=True); DATA.chmod(0o700)
for name in ['config.json', 'recovery_state.json']:
    path = DATA / name
    if path.exists(): path.unlink()

def signed(source, name, identifier):
    path=OUT/name;shutil.copyfile(source,path);path.chmod(0o700)
    subprocess.run(['codesign','--force','--sign','-','--identifier',identifier,str(path)],capture_output=True,check=True)
    text=subprocess.run(['codesign','-d','--verbose=3',str(path)],capture_output=True,text=True,check=True).stderr
    return path,next(l.split('=',1)[1] for l in text.splitlines() if l.startswith('CDHash='))

hostpath,hosthash=signed(ROOT/'target/debug/vxkey_bridge_fixture','swift_host','com.vxkey.desktop')
clientpath,clienthash=signed(OUT/'swift_adapter','swift_client','com.vxkey.inputmethod.vxkey')
trust=DATA/'bridge_pair.json';trust.write_text(json.dumps({'companion':hosthash,'ime':clienthash}));trust.chmod(0o600)
env=dict(os.environ,VXKEY_DATA_DIR=str(DATA),VXKEY_BRIDGE_DEV_TRUST='1')
processes=[]
def spawn(path,args=()):
    p=subprocess.Popen([str(path),*args],stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=subprocess.PIPE,text=True,env=env);processes.append(p)
    assert json.loads(p.stdout.readline())=={'ready':True};return p

def ask(p,r):
    p.stdin.write(json.dumps(r)+'\n');p.stdin.flush()
    assert select.select([p.stdout],[],[],7)[0], 'response timeout'
    line=p.stdout.readline()
    assert line, 'process exit: '+p.stderr.read()[:300]
    return json.loads(line)
def wait(fn):
    end=time.monotonic()+10
    while time.monotonic()<end:
        value=fn()
        if value:return value
        time.sleep(.1)
    raise AssertionError('state timeout')
def snapshot():return ask(host,{'command':'get_snapshot'})['data']
checks=[]
try:
    host=spawn(hostpath,['host']);client=spawn(clientpath)
    wait(lambda:snapshot()['runtime']['applied_revision']=='1')
    state=ask(client,{'fixture':'state'});assert state['input_method']=='telex' and not state['suspended']
    checks.append('real Swift AppSettings connects, maps strict native cache and acknowledges revision')
    macros=ask(host,{'command':'replace_macros','expected_revision':'1','enabled':True,'in_en':True,'smart_case':True,'entries':[{'id':str(uuid.uuid4()),'trigger':'fixture','replacement':'Không phải văn bản người dùng'}]})
    assert macros['ok'];wait(lambda:ask(client,{'fixture':'state'})['macro_count']==1)
    old=snapshot()['revision'];wait(lambda:snapshot()['runtime']['applied_revision']==old)
    assert ask(client,{'fixture':'compose'})['composing']
    patch=ask(host,{'command':'patch_config','expected_revision':old,'changes':{'typing':{'input_method':'vni'}}})
    assert patch['ok'];time.sleep(.35)
    assert snapshot()['runtime']['applied_revision']==old
    assert ask(client,{'fixture':'state'})['input_method']=='telex'
    ask(client,{'fixture':'clear'})
    wait(lambda:snapshot()['runtime']['applied_revision']==patch['data']['persisted_revision'])
    assert ask(client,{'fixture':'state'})['input_method']=='vni'
    checks.append('real Rust composer buffer delays Swift config/macro apply until boundary; then VNI and revision agree')
    ask(client,{'fixture':'stall'})
    wait(lambda:(DATA/'recovery_state.json').exists())
    wait(lambda:ask(client,{'fixture':'state'})['suspended'])
    checks.append('blocking actual Swift main RunLoop stops health progress while bridge worker remains alive; transforms remain suspended after reconnect')
    wait(lambda:snapshot()['runtime']['connection_status']=='connected')
    state=snapshot();session=state['runtime']['engine_session_id']
    resume=ask(host,{'fixture':'control','control':{'action':'suspend','id':str(uuid.uuid4()),'suspended':False,'ime_session_id':session}})
    assert resume['ok'];wait(lambda:not ask(client,{'fixture':'state'})['suspended'])
    assert not (DATA/'recovery_state.json').exists()
    checks.append('explicit same-session Resume requires real Swift main-loop ACK before removing durable marker')
    report={'status':'pass','scope':'real Swift AppSettings/main timer and Rust FFI composer; stub controller only, no IMK/document edits',
        'checks':checks,'host_cdhash':hosthash,'client_cdhash':clienthash,'swift_sha256':hashlib.sha256(clientpath.read_bytes()).hexdigest(),
        'ime_installed':False,'ime_activated':False,'native_document_typing_tested':False}
    (ROOT/'docs/qa/swift_bridge_evidence.json').write_text(json.dumps(report,ensure_ascii=False,indent=2)+'\n');print(json.dumps(report,ensure_ascii=False,indent=2))
finally:
    for p in processes:
        if p.poll() is None:
            p.terminate()
            try:p.wait(timeout=5)
            except subprocess.TimeoutExpired:p.kill();p.wait(timeout=5)
