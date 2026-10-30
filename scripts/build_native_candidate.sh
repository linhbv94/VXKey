#!/bin/sh
# Isolated SDK27 candidate only: never install/register/activate an input source.
set -eu
root="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
cd "$root"
python3 scripts/check_macos_sdk.py >/dev/null
cargo +stable build -p funput-ffi --features convert --release
out="$root/build/native_candidate/VXKeyInput.app"
mkdir -p "$out/Contents/MacOS" "$out/Contents/Resources" "$root/build/swift_module_cache"
cp docs/spec/config_example.json "$out/Contents/Resources/default_config.json"
cp icon.icns "$out/Contents/Resources/icon.icns"
python3 scripts/write_native_info.py "$out/Contents/Info.plist"
swiftc -sdk "$(xcrun --show-sdk-path)" -target arm64-apple-macos26.0 -module-cache-path build/swift_module_cache \
  -import-objc-header platforms/macos/Bridging-Funput.h -I crates/funput-ffi/include \
  native/macos/*.swift platforms/macos/Funput/Model/InputMethod.swift \
  platforms/macos/Funput/ExtraOnsets/ExtraOnsetLetters.swift \
  platforms/macos/Funput/IME/FunputComposer.swift platforms/macos/Funput/IME/ComposerConfiguration.swift \
  platforms/macos/Funput/IME/InputEventPolicy.swift platforms/macos/Funput/IME/RetoneDocument.swift \
  platforms/macos/Funput/IME/RetoneScan.swift platforms/macos/Funput/IME/EnglishComposition.swift \
  platforms/macos/Funput/IME/FunputComposer+Text.swift \
  platforms/macos/Funput/IME/Composition/*.swift \
  target/release/libfunput_ffi.a -framework AppKit -framework InputMethodKit -framework Carbon \
  -o "$out/Contents/MacOS/VXKeyInput"
python3 scripts/check_macos_sdk.py --artifact "$out"
echo "Candidate: $out (not installed or activated)"
