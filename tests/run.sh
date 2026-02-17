#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$ROOT_DIR/target}"
export LD_LIBRARY_PATH="$CARGO_TARGET_DIR/debug:${LD_LIBRARY_PATH:-}"

python ./tests/functional/test_model3_replay.py
python ./tests/functional/test_can_multiplexing.py
