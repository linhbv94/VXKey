#!/bin/sh
# Local QA bundle only; no installer, input-source registration, signing or release.
set -eu
root="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
cd "$root"
python3 scripts/check_macos_sdk.py >/dev/null
python3 scripts/render_tray_assets.py
npm run build --prefix desktop
MACOSX_DEPLOYMENT_TARGET=26.0 cargo +stable build --manifest-path desktop/src-tauri/Cargo.toml --features custom-protocol --offline
python3 - <<'PY'
from pathlib import Path
import shutil, plistlib, os
root=Path.cwd()
candidate=Path(os.environ.get('VXKEY_CANDIDATE_ROOT',str(root/'build')))
app=candidate/'desktop_candidate/VXKey.app'
(app/'Contents/MacOS').mkdir(parents=True,exist_ok=True)
(app/'Contents/Resources').mkdir(exist_ok=True)
shutil.copy2(root/'desktop/src-tauri/target/debug/vxkey-desktop',app/'Contents/MacOS/VXKey')
shutil.copy2(root/'icon.icns',app/'Contents/Resources/icon.icns')
qa_dir=candidate/'qa_integration_data'
qa_dir.mkdir(parents=True,exist_ok=True,mode=0o700)
os.chmod(qa_dir,0o700)
info={'LSEnvironment':{'VXKEY_DATA_DIR':str(qa_dir)},'CFBundleName':'VXKey','CFBundleDisplayName':'VXKey','CFBundleIdentifier':'com.vxkey.desktop','CFBundleExecutable':'VXKey','CFBundlePackageType':'APPL','CFBundleShortVersionString':'0.1.0','CFBundleVersion':'1','CFBundleIconFile':'icon.icns','LSMinimumSystemVersion':'26.0','NSHighResolutionCapable':True}
(app/'Contents/Info.plist').write_bytes(plistlib.dumps(info))
print(app)
PY
python3 scripts/check_macos_sdk.py --artifact "${VXKEY_CANDIDATE_ROOT:-$root/build}/desktop_candidate/VXKey.app"
