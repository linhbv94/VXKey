#!/bin/sh
# Real Funput + spike state policy/event construction; NO tap or event posting.
set -eu
root="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
cd "$root"
python3 scripts/check_macos_sdk.py >/dev/null
cargo +stable build -p funput-ffi --features convert,vxkey-native-dev --release --offline
mkdir -p build/session_spike_fixture build/swift_module_cache
swiftc -O -sdk "$(xcrun --show-sdk-path)" -target arm64-apple-macos26.0 -module-cache-path build/swift_module_cache \
  -import-objc-header native/macos/bridging_vxkey.h -I crates/funput-ffi/include \
  native/macos_background/edit_plan.swift \
  native/macos_session_spike/fixed_transport_trace.swift \
  native/macos_session_spike/session_composition.swift native/macos_session_spike/session_driver.swift \
  platforms/macos/Funput/ExtraOnsets/ExtraOnsetLetters.swift \
  platforms/macos/Funput/IME/FunputComposer.swift platforms/macos/Funput/IME/FunputComposer+Text.swift \
  native/tests/session_spike/main.swift target/release/libfunput_ffi.a \
  -framework AppKit -framework Carbon -o build/session_spike_fixture/session_spike_fixture
build/session_spike_fixture/session_spike_fixture
