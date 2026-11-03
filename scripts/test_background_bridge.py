"""Actual Swift cache/timer and Rust transport/composer; no OS hooks or grant."""
from pathlib import Path
import subprocess,os,json,time,select
r=Path(__file__).resolve().parents[1];out=r/'build/background_adapter';out.mkdir(exist_ok=True,parents=True)
data=out/'data';data.mkdir(mode=0o700,exist_ok=True);data.chmod(0o700)
for name in ['config.json','recovery_state.json']:
 p=data/name
 if p.exists():p.unlink()
subprocess.run(['cargo','+stable','build','-p','vxkey-runtime','--bin','vxkey_bridge_fixture','--offline'],cwd=r,check=True,capture_output=True)
files=['native/macos_background/runtime_settings.swift','native/tests/background_bridge/fake_driver.swift','native/tests/background_bridge/main.swift','platforms/macos/Funput/Model/InputMethod.swift','platforms/macos/Funput/ExtraOnsets/ExtraOnsetLetters.swift','platforms/macos/Funput/IME/FunputComposer.swift','platforms/macos/Funput/IME/ComposerConfiguration.swift','platforms/macos/Funput/IME/FunputComposer+Text.swift']
sdk=subprocess.check_output(['xcrun','--show-sdk-path'],text=True).strip()
subprocess.run(['swiftc','-sdk',sdk,'-target','arm64-apple-macos26.0','-module-cache-path','build/swift_module_cache','-import-objc-header','native/macos/bridging_vxkey.h','-I','crates/funput-ffi/include',*files,'target/release/libfunput_ffi.a','-framework','AppKit','-framework','Carbon','-o',str(out/'engine')],cwd=r,check=True)
import shutil
shutil.copyfile(r/'target/debug/vxkey_bridge_fixture',out/'host')
pins={}
for name,identity in [('host','com.vxkey.desktop'),('engine','com.vxkey.engine')]:
 p=out/name;p.chmod(0o700)
 subprocess.run(['codesign','--force','--sign','-','--identifier',identity,str(p)],check=True,capture_output=True)
 signed=subprocess.run(['codesign','-d','--verbose=3',str(p)],capture_output=True,text=True,check=True).stderr
 pins['companion' if name=='host' else 'engine']=next(x.split('=',1)[1] for x in signed.splitlines()if x.startswith('CDHash='))
p=data/'bridge_pair.json';p.write_text(json.dumps(pins)+'\n');p.chmod(0o600)
env=dict(os.environ,VXKEY_DATA_DIR=str(data),VXKEY_BRIDGE_DEV_TRUST='1')
host=subprocess.Popen([str(out/'host'),'host'],env=env,stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=subprocess.PIPE,text=True)
engine=None
checks=[]
def ask(req):
 host.stdin.write(json.dumps(req)+'\n');host.stdin.flush()
 assert select.select([host.stdout],[],[],5)[0], 'host deadline'
 line=host.stdout.readline();assert line, 'host exited'
 return json.loads(line)
def snap():return ask({'command':'get_snapshot'})['data']
def until(pred,seconds=6):
 end=time.monotonic()+seconds
 while time.monotonic()<end:
  v=pred()
  if v:return v
  time.sleep(.05)
 raise AssertionError('deadline')
try:
 ready_line=host.stdout.readline()
 assert ready_line, f'fixture host exited {host.poll()}: {host.stderr.read(2000)}'
 assert json.loads(ready_line)['ready']
 engine=subprocess.Popen([str(out/'engine')],env=env,stdout=subprocess.DEVNULL,stderr=subprocess.PIPE)
 s=until(lambda: (x if (x:=snap())['runtime']['applied_revision']=='1' else None))
 checks.append('real Swift cache timer applies native revision')
 session=s['runtime']['engine_session_id']
 import uuid
 for index in range(3):
  for suspended in [False,True]:
   result=ask({'fixture':'control','control':{'action':'suspend','id':str(uuid.uuid4()),'suspended':suspended,'engine_session_id':session}})
   assert result['ok'],result
   until(lambda:snap()['runtime']['health_state']==('suspended'if suspended else'healthy'))
 checks.append('three real cache Resume/Pause transactions acknowledged and reflected')
 print(json.dumps({'status':'pass','checks':checks,'os_hook_mocked':True,'native_typing_tested':False},indent=2))
finally:
 if engine:
  if engine.poll() is None:engine.terminate()
  engine.wait(timeout=5)
  error=engine.stderr.read()
  if error:print(error[:2000])
 host.terminate();host.wait(timeout=5)
