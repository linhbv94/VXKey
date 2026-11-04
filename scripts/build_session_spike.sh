#!/bin/sh
# Explicit isolated, ad-hoc DEV experiment, never a production updater artifact.
# Refuse overwriting the bundle: granting it freezes its bytes for that trial.
set -eu
root="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
cd "$root"
python3 scripts/check_macos_sdk.py >/dev/null
out="${VXKEY_SESSION_SPIKE_ROOT:-$root/build/session_spike_candidate_v6}/vxkey_session_spike.app"
if [ -e "$out" ]; then
  echo 'Refusing to overwrite existing spike. Preserve its permission identity; archive/remove manually before a new trial.' >&2
  exit 2
fi
cargo +stable build -p funput-ffi --features convert,vxkey-native-dev --release --offline
mkdir -p "$out/Contents/MacOS" "$out/Contents/Resources" build/swift_module_cache
swiftc -O -sdk "$(xcrun --show-sdk-path)" -target arm64-apple-macos26.0 -module-cache-path build/swift_module_cache \
  -import-objc-header native/macos/bridging_vxkey.h -I crates/funput-ffi/include \
  native/macos_background/edit_plan.swift native/macos_session_spike/*.swift \
  platforms/macos/Funput/ExtraOnsets/ExtraOnsetLetters.swift \
  platforms/macos/Funput/IME/FunputComposer.swift platforms/macos/Funput/IME/FunputComposer+Text.swift \
  target/release/libfunput_ffi.a -framework AppKit -framework Carbon \
  -o "$out/Contents/MacOS/vxkey_session_spike"
python3 - "$out" <<'PY'
from pathlib import Path
import plistlib, shutil, sys
app = Path(sys.argv[1])
info = dict(CFBundleName='VXKey Session Spike', CFBundleDisplayName='VXKey Session Spike',
    CFBundleIdentifier='com.vxkey.spike.session', CFBundleExecutable='vxkey_session_spike',
    CFBundlePackageType='APPL', CFBundleShortVersionString='0.1.5', CFBundleVersion='6',
    LSUIElement=True, LSMinimumSystemVersion='26.0', NSHighResolutionCapable=True,
    CFBundleIconFile='icon.icns')
(app/'Contents/Info.plist').write_bytes(plistlib.dumps(info))
shutil.copyfile('icon.icns', app/'Contents/Resources/icon.icns')
PY
codesign --force --sign - --identifier com.vxkey.spike.session "$out"
codesign --verify --deep --strict "$out"
python3 scripts/check_macos_sdk.py --artifact "$out"
printf '%s\n' "$out"
