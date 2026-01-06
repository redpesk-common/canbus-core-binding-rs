#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$ROOT_DIR/target}"

# Important: rendre visibles les .so Rust
export LD_LIBRARY_PATH="$CARGO_TARGET_DIR/debug:${LD_LIBRARY_PATH:-}"

#pytest -q --junitxml="$ROOT_DIR/reports/afb-functional.xml" "$ROOT_DIR/tests/functional/"

python ./tests/functional/test_model3_replay.py
