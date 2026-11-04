"""VXKey macOS Silicon production pair. No install, dev pins or QA data roots."""
from pathlib import Path
import os,json,subprocess,shutil,plistlib,sys
root=Path(__file__).resolve().parents[1]
os.chdir(root)
identity=json.loads((root/'scripts/release_identity.json').read_text())
assert json.loads((root/'desktop/src-tauri/tauri.conf.json').read_text())['version']==identity['version']
if os.environ.get('GITHUB_ACTIONS')=='true':assert os.environ.get('GITHUB_REF_NAME')=='v'+identity['version']
env=dict(os.environ,VXKEY_SIGNER_SHA1=identity['certificate_sha1'],MACOSX_DEPLOYMENT_TARGET='26.0')
def run(args,cwd=root):subprocess.run(args,cwd=cwd,env=env,check=True)
run(['python3','scripts/check_macos_sdk.py'])
run(['cargo','+stable','build','-p','funput-ffi','--features','convert,vxkey-native','--release','--locked'])
app=root/'build/release_helper/VXKeyEngine.app';resources=app/'Contents/Resources';resources.mkdir(parents=True,exist_ok=True);(app/'Contents/MacOS').mkdir(exist_ok=True)
shutil.copy2('docs/spec/config_example.json',resources/'default_config.json');shutil.copy2('icon.icns',resources/'icon.icns')
run(['python3','scripts/write_native_info.py',str(app/'Contents/Info.plist')])
p=app/'Contents/Info.plist';info=plistlib.loads(p.read_bytes());info['CFBundleShortVersionString']=identity['version'];info['CFBundleVersion']=identity['version'];p.write_bytes(plistlib.dumps(info))
sdk=subprocess.check_output(['xcrun','--show-sdk-path'],text=True).strip();cache=root/'build/release_swift_cache';cache.mkdir(exist_ok=True)
sources=sorted(str(x) for x in (root/'native/macos_background').glob('*.swift'))
sources += sorted(str(x) for x in (root/'native/macos_session').glob('*.swift'))
sources += [str(root/x) for x in ['platforms/macos/Funput/Model/InputMethod.swift','platforms/macos/Funput/ExtraOnsets/ExtraOnsetLetters.swift','platforms/macos/Funput/IME/FunputComposer.swift','platforms/macos/Funput/IME/ComposerConfiguration.swift','platforms/macos/Funput/IME/FunputComposer+Text.swift']]
run(['swiftc','-O','-sdk',sdk,'-target','arm64-apple-macos26.0','-module-cache-path',str(cache),'-D','VXKEY_SESSION_ADAPTER','-import-objc-header','native/macos/bridging_vxkey.h','-I','crates/funput-ffi/include',*sources,'target/release/libfunput_ffi.a','-framework','AppKit','-framework','Carbon','-o',str(app/'Contents/MacOS/VXKeyEngine')])
if '--compile-only' in sys.argv:
 print('Helper compiled only; not a signed or installable release candidate');sys.exit(0)
run(['codesign','--force','--sign',identity['certificate_sha1'],'--options','runtime','--timestamp=none',str(app)])
run(['codesign','--verify','--deep','--strict',str(app)])
run(['python3','scripts/check_macos_sdk.py','--artifact',str(app)])
run(['python3','scripts/render_tray_assets.py'])
run(['npm','exec','--','tauri','build','--bundles','app,dmg','--config',os.environ['VXKEY_SIGNER_CONFIG']],root/'desktop')
