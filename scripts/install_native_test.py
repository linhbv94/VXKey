"""Reversible per-user development IME install; never overwrites an existing app."""
from pathlib import Path
import hashlib,json,os,plistlib,shutil,subprocess,sys,uuid,ctypes
root=Path(__file__).resolve().parents[1]
source=root/'build/native_candidate/VXKeyInput.app'
target=Path.home()/'Library/Input Methods/VXKeyInput.app'
receipt=root/'build/native_activation/install_receipt.json'
expected_id='com.vxkey.inputmethod.vxkey'
def digest(app):
    return hashlib.sha256((app/'Contents/MacOS/VXKeyInput').read_bytes()).hexdigest()
def verify(app):
    info=plistlib.loads((app/'Contents/Info.plist').read_bytes())
    assert info['CFBundleIdentifier']==expected_id
    subprocess.run(['codesign','--verify','--deep','--strict',str(app)],check=True,capture_output=True)
    return digest(app)
mode=sys.argv[1] if len(sys.argv)>1 else 'install'
if mode=='install':
    raise SystemExit('Retired: VXKey is a background utility. Do not install an Input Source; build the paired VXKey.app instead.')
    assert not target.exists() and not target.is_symlink(), 'Existing install: preserve and review first'
    assert all(not p.is_symlink() for p in source.rglob('*')), 'Unexpected bundle links'
    sha=verify(source)
    data={'status':'preparing','target':str(target),'source':str(source),'binary_sha256':sha,'overwrote_existing':False}
    receipt.write_text(json.dumps(data,indent=2)+'\n')
    # Publish a complete bundle atomically, without briefly exposing an empty .app
    # to LaunchServices and without overwriting a concurrently-created destination.
    stage=target.parent/('.vxkey_stage_'+uuid.uuid4().hex)
    stage.mkdir(mode=0o700)
    staged=stage/'VXKeyInput.app'
    try:
        shutil.copytree(source,staged)
        assert verify(staged)==sha
        lib=ctypes.CDLL('/usr/lib/libSystem.B.dylib',use_errno=True)
        rename=lib.renamex_np;rename.argtypes=[ctypes.c_char_p,ctypes.c_char_p,ctypes.c_uint];rename.restype=ctypes.c_int
        if rename(os.fsencode(staged),os.fsencode(target),4)!=0:
            raise OSError(ctypes.get_errno(),'Atomic exclusive install failed')
        target.touch();data['status']='installed';receipt.write_text(json.dumps(data,indent=2)+'\n')
    finally:
        if staged.exists():
            quarantine=root/'build/native_activation'/('failed_install_'+uuid.uuid4().hex)
            shutil.move(str(staged),str(quarantine))
        stage.rmdir()
    print(json.dumps(data,indent=2))
elif mode=='remove':
    data=json.loads(receipt.read_text());assert data['status']=='installed' and data['target']==str(target)
    assert verify(target)==data['binary_sha256'], 'Installed bytes changed: preserve for review'
    # Source recovery/disable must already have succeeded; check OS readback.
    control=root/'build/native_activation/source_control'
    state=json.loads(subprocess.check_output([str(control),'status'],text=True))
    assert not state['selected'].startswith('com.vxkey.')
    assert not any(x['enabled'] for x in state['sources'] if x['id']==expected_id)
    quarantine=root/'build/native_activation'/('removed_'+uuid.uuid4().hex)
    quarantine.mkdir();shutil.move(str(target),str(quarantine/'VXKeyInput.app'))
    data.update(status='removed',quarantine=str(quarantine/'VXKeyInput.app'));receipt.write_text(json.dumps(data,indent=2)+'\n');print(json.dumps(data,indent=2))
else:raise SystemExit('install/remove only')
