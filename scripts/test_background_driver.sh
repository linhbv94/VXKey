#!/bin/sh
# Real Rust composer + pure edit policy. No hooks, permissions or UI automation.
set -eu
root="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
cd "$root"
python3 scripts/check_macos_sdk.py >/dev/null
cargo +stable build -p funput-ffi --features convert,vxkey-native-dev --release --offline
mkdir -p build/background_fixture build/swift_module_cache
swiftc -sdk "$(xcrun --show-sdk-path)" -target arm64-apple-macos26.0 -module-cache-path build/swift_module_cache \
  -import-objc-header native/macos/bridging_vxkey.h -I crates/funput-ffi/include \
  native/macos_background/context_guard.swift native/macos_background/permission_cache.swift native/macos_background/stream_window.swift native/macos_background/edit_plan.swift \
  native/macos_background/event_driver.swift native/macos_background/runtime_settings.swift \
  platforms/macos/Funput/Model/InputMethod.swift platforms/macos/Funput/ExtraOnsets/ExtraOnsetLetters.swift \
  platforms/macos/Funput/IME/FunputComposer.swift platforms/macos/Funput/IME/ComposerConfiguration.swift \
  platforms/macos/Funput/IME/FunputComposer+Text.swift native/tests/background_driver/main.swift \
  target/release/libfunput_ffi.a -framework AppKit -framework Carbon -o build/background_fixture/background_fixture
build/background_fixture/background_fixture > docs/qa/background_edit_evidence.json
