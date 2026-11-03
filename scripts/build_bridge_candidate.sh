#!/bin/sh
# Development pair only. Rebuilding either component invalidates its pin; rebuild
# and prepare the pair again. Never installs/registers/selects an input source.
set -eu
root="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
cd "$root"
sh scripts/build_native_candidate.sh
sh scripts/build_desktop_candidate.sh
python3 scripts/prepare_bridge_candidate.py
