#!/bin/sh
set -eu
root="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
cd "$root"
mkdir -p build/native_tests build/swift_module_cache
swiftc -module-cache-path build/swift_module_cache native/tests/composition_safety.swift platforms/macos/Funput/IME/Composition/CompositionPresenter.swift platforms/macos/Funput/IME/Composition/DirectPresenter.swift -o build/native_tests/composition_safety
build/native_tests/composition_safety
