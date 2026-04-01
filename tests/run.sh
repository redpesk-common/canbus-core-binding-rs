#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage: ./tests/run.sh [--setup_vcan] [--cleanup_vcan] [--log_dir DIR] [--help]

  --setup_vcan        Create/bring up VCAN interface before running tests (requires root).
  --cleanup_vcan      Cleanup VCAN interface after tests (requires root).
  --log_dir DIR       Log directory for test output.
EOF
}

PACKAGE_NAME="canbus-core-binding-rs"

SETUP_VCAN=0
CLEANUP_VCAN=0
LOG_DIR=""

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

LOG_DIR="${LOG_DIR:-${ROOT_DIR}/logs}"
mkdir -p "${LOG_DIR}"

run_tap() {
  local name="$1"
  shift

  if "$@" --tap 2>&1 | tee "${LOG_DIR}/${name}.tap"; then
    return 0
  else
    TEST_FAILURE=1
    return 0
  fi
}

VCAN_IFACE="${VCAN_IFACE:-vcan0}"
TEST_FAILURE=0

if [[ "${CLEANUP_VCAN}" -eq 1 ]]; then
  trap 'sudo '"${SCRIPT_DIR}"'/cleanup_vcan.sh "'"${VCAN_IFACE}"'" >/dev/null 2>&1 || true' EXIT
fi

if [[ "${SETUP_VCAN}" -eq 1 ]]; then
  sudo "${SCRIPT_DIR}/setup_vcan.sh" "${VCAN_IFACE}"
fi

python3 -m pip install -r "${SCRIPT_DIR}/requirements.txt"

run_tap test_can_multiplexing \
  "${SCRIPT_DIR}/functional/test_can_multiplexing.py"

run_tap model3_vcan \
  "${SCRIPT_DIR}/functional/test_canbus-binding.py" \
  --config "${SCRIPT_DIR}/../examples/samples/model3/binding-config/binding-config.json" \
  --can-api model3 \
  --dbc-file "${SCRIPT_DIR}/../examples/samples/model3/dbc/model3can.dbc" \
  --vcan-iface "${VCAN_IFACE}" \
  --debug

run_tap bms_vcan \
  "${SCRIPT_DIR}/functional/test_canbus-binding.py" \
  --config "${SCRIPT_DIR}/../examples/samples/bms/binding-config/binding-config.json" \
  --can-api bms \
  --dbc-file "${SCRIPT_DIR}/../examples/samples/bms/dbc/BMS.dbc" \
  --vcan-iface "${VCAN_IFACE}" \
  --canids 0x221 \
  --debug

run_tap bms_canids \
  "${SCRIPT_DIR}/functional/test_canbus-binding.py" \
  --config "${SCRIPT_DIR}/../examples/samples/bms/binding-config/binding-config.json" \
  --can-api bms \
  --dbc-file "${SCRIPT_DIR}/../examples/samples/bms/dbc/BMS.dbc" \
  --vcan-iface "${VCAN_IFACE}" \
  --canids 257,641 \
  --verbose

exit "${TEST_FAILURE}"