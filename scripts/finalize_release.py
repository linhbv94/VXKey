"""Verify the exact signed packages and prepare updater assets. No publishing."""
from pathlib import Path
import json,subprocess,plistlib,tarfile,tempfile,hashlib,shutil
root=Path(__file__).resolve().parents[1];cfg=json.loads((root/'scripts/release_identity.json').read_text());bundle=root/'desktop/src-tauri/target/release/bundle';out=root/'build/release_assets';out.mkdir(parents=True,exist_ok=True);version=cfg['version']
def verify(app):
 for target,identifier in [(app,cfg['app_id']),(app/'Contents/Resources/VXKeyEngine.app',cfg['helper_id'])]:
  info=plistlib.loads((target/'Contents/Info.plist').read_bytes());assert info['CFBundleIdentifier']==identifier and info['CFBundleShortVersionString']==version;assert 'LSEnvironment' not in info
  icon=info['CFBundleIconFile'];icon=icon if icon.endswith('.icns') else icon+'.icns'
  assert (target/'Contents/Resources'/icon).read_bytes()==(root/'icon.icns').read_bytes(), 'Bundled icon differs from approved ICNS'
  subprocess.run(['codesign','--verify','--deep','--strict',str(target)],check=True)
  r=subprocess.run(['codesign','-d','-r-',str(target)],capture_output=True,text=True,check=True);dr=r.stdout+r.stderr
  assert ('certificate leaf = H"'+cfg['certificate_sha1'].lower()+'"') in dr and 'cdhash' not in dr
  subprocess.run(['python3',str(root/'scripts/check_macos_sdk.py'),'--artifact',str(target)],check=True)
  binary=target/'Contents/MacOS'/info['CFBundleExecutable'];assert subprocess.check_output(['lipo','-archs',str(binary)],text=True).strip()=='arm64'
app=bundle/'macos/VXKey.app';verify(app)
archive=next((bundle/'macos').glob('*.app.tar.gz'));sig=Path(str(archive)+'.sig');assert sig.is_file() and len(sig.read_text().strip())>20
with tempfile.TemporaryDirectory() as d:
 with tarfile.open(archive) as t:t.extractall(d,filter='data')
 verify(Path(d)/'VXKey.app')
archive_name=f'vxkey_{version}_macos_silicon.app.tar.gz';dmg_name=f'vxkey_{version}_macos_silicon.dmg';shutil.copy2(archive,out/archive_name);shutil.copy2(next((bundle/'dmg').glob('*.dmg')),out/dmg_name)
manifest={'version':version,'notes':'VXKey '+version,'platforms':{'darwin-aarch64':{'signature':sig.read_text().strip(),'url':f'https://github.com/linhbv94/VXKey/releases/download/v{version}/'+archive_name}}}
(out/'latest.json').write_text(json.dumps(manifest,indent=2))
# Same app bytes must be present in the first-install DMG and updater archive.
with tempfile.TemporaryDirectory() as mount:
 subprocess.run(['hdiutil','attach','-readonly','-nobrowse','-mountpoint',mount,str(out/dmg_name)],check=True)
 try:
  target=Path(mount)/'VXKey.app';verify(target)
  for rel in ['Contents/MacOS/vxkey-desktop','Contents/Resources/VXKeyEngine.app/Contents/MacOS/VXKeyEngine']:
   assert (app/rel).read_bytes()==(target/rel).read_bytes()
 finally:subprocess.run(['hdiutil','detach',mount],check=True)
print('Signed app/helper, archive and DMG identity/SDK/arch/version gates PASS')
