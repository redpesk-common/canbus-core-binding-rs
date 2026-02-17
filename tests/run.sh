#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage: ./tests/run.sh [--setup_vcan] [--cleanup_vcan|--cleanup_vscan] [--help]

  --setup_vcan        Create/bring up VCAN interface before running tests (requires root).
  --cleanup_vcan      Cleanup VCAN interface after tests (requires root).
EOF
}

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

SETUP_VCAN=0
CLEANUP_VCAN=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    --setup_vcan)
      SETUP_VCAN=1
      shift
      ;;
    --cleanup_vcan|--cleanup_vscan)
      CLEANUP_VCAN=1
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "ERROR: unknown option: $1" >&2
      usage >&2
      exit 2
      ;;
  esac
done


export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$ROOT_DIR/target}"
export LD_LIBRARY_PATH="$CARGO_TARGET_DIR/debug:${LD_LIBRARY_PATH:-}"

# Use VCAN_IFACE env var if provided, else default.
VCAN_IFACE="${VCAN_IFACE:-vcan0}"

if [[ "$CLEANUP_VCAN" -eq 1 ]]; then
  # Always attempt cleanup on exit (even on failure).
  trap "sudo ${ROOT_DIR}/tests/cleanup_vcan.sh \"${VCAN_IFACE}\" >/dev/null 2>&1 || true" EXIT
fi

if [[ "$SETUP_VCAN" -eq 1 ]]; then
  sudo ${ROOT_DIR}/tests/setup_vcan.sh "${VCAN_IFACE}"
fi

./tests/functional/test_can_multiplexing.py

./tests/functional/test_canbus-binding.py \
    --config ./examples/samples/model3/binding-config/binding-config.json \
    --can-api model3 \
    --dbc-file ./examples/samples/model3/dbc/model3can.dbc \
    --vcan-iface vcan0 \
    --debug \
    -v

./tests/functional/test_canbus-binding.py \
    --config ./examples/samples/bms/binding-config/binding-config.json \
    --can-api bms \
    --dbc-file ./examples/samples/bms/dbc/BMS.dbc \
    --vcan-iface vcan0 \
    --canids 0x221 \
    --debug \
    -v

./tests/functional/test_canbus-binding.py \
    --config ./examples/samples/bms/binding-config/binding-config.json \
    --can-api bms \
    --dbc-file ./examples/samples/bms/dbc/BMS.dbc \
    --canids 257,641 \
    --verbose \
    -v

