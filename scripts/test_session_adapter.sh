#!/bin/sh
# Production session module with real core/config, no OS taps or event posting.
set -eu
root="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
cd "$root"
python3 scripts/check_macos_sdk.py >/dev/null
cargo +stable build -p funput-ffi --features convert,vxkey-native-dev --release --offline
mkdir -p build/session_adapter_fixture build/swift_module_cache
swiftc -O -D VXKEY_SESSION_ADAPTER -sdk "$(xcrun --show-sdk-path)" -target arm64-apple-macos26.0 \
  -module-cache-path build/swift_module_cache -import-objc-header native/macos/bridging_vxkey.h \
  -I crates/funput-ffi/include native/macos_background/edit_plan.swift \
  native/macos_background/permission_cache.swift native/macos_background/runtime_settings.swift \
  native/macos_session/*.swift platforms/macos/Funput/Model/InputMethod.swift \
  platforms/macos/Funput/ExtraOnsets/ExtraOnsetLetters.swift platforms/macos/Funput/IME/FunputComposer.swift \
  platforms/macos/Funput/IME/ComposerConfiguration.swift platforms/macos/Funput/IME/FunputComposer+Text.swift \
  native/tests/session_adapter/main.swift target/release/libfunput_ffi.a -framework AppKit -framework Carbon \
  -o build/session_adapter_fixture/session_adapter_fixture
build/session_adapter_fixture/session_adapter_fixture
