#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage: ./tests/run.sh [--setup_vcan] [--cleanup_vcan|--cleanup_vscan] [--help]

  --setup_vcan        Create/bring up VCAN interface before running tests (requires root).
  --cleanup_vcan      Cleanup VCAN interface after tests (requires root).
  --log_dir           Log directory for test output.
EOF
}

PACKAGE_NAME="canbus-core-binding-rs"

SETUP_VCAN=0
CLEANUP_VCAN=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    --setup_vcan|-s)
      SETUP_VCAN=1
      shift
      ;;
    --cleanup_vcan|-c)
      CLEANUP_VCAN=1
      shift
      ;;
    --log_dir|-l)
      LOG_DIR="$2"
      shift 2
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

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$(dirname "${SCRIPT_DIR}")" && pwd)"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-${ROOT_DIR}/target}"
export LD_LIBRARY_PATH="${CARGO_TARGET_DIR}/debug:${CARGO_TARGET_DIR}/release:${LD_LIBRARY_PATH:-}"

LOG_DIR="${ROOT_DIR}/logs"
mkdir -p "$LOG_DIR"

run_tap () {
  local name="$1"; shift
    "$@" --tap 2>&1 | tee "${LOG_DIR}/${name}.tap"
}

# Use VCAN_IFACE env var if provided, else default.
VCAN_IFACE="${VCAN_IFACE:-vcan0}"

if [[ "$CLEANUP_VCAN" -eq 1 ]]; then
  # Always attempt cleanup on exit (even on failure).
  trap "sudo ${ROOT_DIR}/tests/cleanup_vcan.sh \"${VCAN_IFACE}\" >/dev/null 2>&1 || true" EXIT
fi

if [[ "$SETUP_VCAN" -eq 1 ]]; then
  sudo ${ROOT_DIR}/tests/setup_vcan.sh "${VCAN_IFACE}"
fi

run_tap test_can_multiplexing ${ROOT_DIR}/tests/functional/test_can_multiplexing.py

run_tap model3_vcan ${ROOT_DIR}/tests/functional/test_canbus-binding.py \
    --config ${ROOT_DIR}/examples/samples/model3/binding-config/binding-config.json \
    --can-api model3 \
    --dbc-file ${ROOT_DIR}/examples/samples/model3/dbc/model3can.dbc \
    --vcan-iface vcan0 \
    --debug

run_tap bms_vcan ${ROOT_DIR}/tests/functional/test_canbus-binding.py \
    --config ${ROOT_DIR}/examples/samples/bms/binding-config/binding-config.json \
    --can-api bms \
    --dbc-file ${ROOT_DIR}/examples/samples/bms/dbc/BMS.dbc \
    --vcan-iface vcan0 \
    --canids 0x221 \
    --debug

run_tap bms_canids ${ROOT_DIR}/tests/functional/test_canbus-binding.py \
    --config ${ROOT_DIR}/examples/samples/bms/binding-config/binding-config.json \
    --can-api bms \
    --dbc-file ${ROOT_DIR}/examples/samples/bms/dbc/BMS.dbc \
    --canids 257,641 \
    --verbose

