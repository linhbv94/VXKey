"""Embed and pin an isolated development pair. No input source or TCC mutation."""
from pathlib import Path
import hashlib,json,plistlib,subprocess,shutil,os
root=Path(__file__).resolve().parents[1]
candidate=Path(os.environ.get('VXKEY_CANDIDATE_ROOT',str(root/'build')))
data=candidate/'qa_integration_data';data.mkdir(mode=0o700,exist_ok=True);data.chmod(0o700)
companion=candidate/'desktop_candidate/VXKey.app'
engine=candidate/'native_candidate/VXKeyEngine.app'
embedded=companion/'Contents/Resources/VXKeyEngine.app'
if embedded.exists():shutil.rmtree(embedded)
shutil.copytree(engine,embedded)
apps={'engine':(embedded,'com.vxkey.engine','VXKeyEngine'), 'companion':(companion,'com.vxkey.desktop','VXKey')}
pins={};artifacts={}
for role,(app,identifier,exe) in apps.items():
    path=app/'Contents/Info.plist';info=plistlib.loads(path.read_bytes())
    assert info['CFBundleIdentifier']==identifier
    info['LSEnvironment']={'VXKEY_DATA_DIR':str(data),'VXKEY_BRIDGE_DEV_TRUST':'1'}
    path.write_bytes(plistlib.dumps(info))
    subprocess.run(['codesign','--force','--sign','-','--identifier',identifier,str(app)],check=True,capture_output=True)
    subprocess.run(['codesign','--verify','--deep','--strict',str(app)],check=True,capture_output=True)
    signed=subprocess.run(['codesign','-d','--verbose=3',str(app)],check=True,text=True,capture_output=True).stderr
    pins[role]=next(x.split('=',1)[1]for x in signed.splitlines()if x.startswith('CDHash='))
    artifacts[role]={'app':str(app.relative_to(root)),'bundle_id':identifier,'cdhash':pins[role],
       'binary_sha256':hashlib.sha256((app/'Contents/MacOS'/exe).read_bytes()).hexdigest()}
manifest=data/'bridge_pair.json';manifest.write_text(json.dumps(pins)+'\n');manifest.chmod(0o600)
report={'status':'built_background_development_pair','signing':'explicit ad-hoc pins; NOT production identity or TCC retention',
        'artifacts':artifacts,'data_root':str(data.relative_to(root)),'input_source_required':False,
        'runtime_typing_tested':False,'autostart_enabled':False}
Path(os.environ.get('VXKEY_PAIR_REPORT',str(root/'docs/qa/native_pair_artifacts.json'))).write_text(json.dumps(report,ensure_ascii=False,indent=2)+'\n')
print(json.dumps(report,ensure_ascii=False,indent=2))
