#!/bin/sh
set -eu
echo "Retired IMK adapter fixture; use scripts/test_background_driver.sh" >&2
exit 2
root="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
cd "$root"
mkdir -p build/bridge_fixture build/swift_module_cache
cargo +stable build -p funput-ffi --features convert,vxkey-native-dev --release --offline
swiftc -sdk "$(xcrun --show-sdk-path)" -target arm64-apple-macos26.0 -module-cache-path build/swift_module_cache \
  -import-objc-header native/macos/bridging_vxkey.h -I crates/funput-ffi/include \
  native/tests/bridge_adapter/main.swift native/macos/runtime_settings.swift \
  platforms/macos/Funput/Model/InputMethod.swift platforms/macos/Funput/ExtraOnsets/ExtraOnsetLetters.swift \
  platforms/macos/Funput/IME/FunputComposer.swift platforms/macos/Funput/IME/FunputComposer+Text.swift \
  platforms/macos/Funput/IME/ComposerConfiguration.swift \
  target/release/libfunput_ffi.a -framework AppKit -framework InputMethodKit -framework Carbon \
  -o build/bridge_fixture/swift_adapter
