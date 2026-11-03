#!/bin/sh
# Isolated SDK27 candidate only: never install/register/activate an input source.
set -eu
root="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
cd "$root"
python3 scripts/check_macos_sdk.py >/dev/null
cargo +stable build -p funput-ffi --features convert,vxkey-native-dev --release
out="${VXKEY_CANDIDATE_ROOT:-$root/build}/native_candidate/VXKeyEngine.app"
mkdir -p "$out/Contents/MacOS" "$out/Contents/Resources" "$root/build/swift_module_cache"
cp docs/spec/config_example.json "$out/Contents/Resources/default_config.json"
cp icon.icns "$out/Contents/Resources/icon.icns"
python3 scripts/write_native_info.py "$out/Contents/Info.plist"
printf APPLVXKY > "$out/Contents/PkgInfo"
swiftc -sdk "$(xcrun --show-sdk-path)" -target arm64-apple-macos26.0 -module-cache-path build/swift_module_cache \
  -import-objc-header native/macos/bridging_vxkey.h -I crates/funput-ffi/include \
  native/macos_background/*.swift platforms/macos/Funput/Model/InputMethod.swift \
  platforms/macos/Funput/ExtraOnsets/ExtraOnsetLetters.swift \
  platforms/macos/Funput/IME/FunputComposer.swift platforms/macos/Funput/IME/ComposerConfiguration.swift \
  platforms/macos/Funput/IME/FunputComposer+Text.swift \
  target/release/libfunput_ffi.a -framework AppKit -framework Carbon \
  -o "$out/Contents/MacOS/VXKeyEngine"
python3 scripts/check_macos_sdk.py --artifact "$out"
echo "Candidate: $out (not installed or activated)"
