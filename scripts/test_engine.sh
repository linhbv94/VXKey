#!/bin/sh
set -eu
root="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
cd "$root"
mkdir -p docs/qa
cargo +stable test -p funput-core -p funput-engine -p funput-config -p funput-convert -p funput-ffi -p vxkey-runtime --features funput-ffi/convert --offline > docs/qa/engine_cargo_tests.log 2>&1
cargo +stable build -p vxkey-runtime --bin vxkey_backend --offline
python3 scripts/smoke_backend.py
sh scripts/test_native_composition.sh
node desktop/scripts/check_settings_ui.mjs
