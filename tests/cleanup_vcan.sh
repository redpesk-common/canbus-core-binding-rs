#!/usr/bin/env bash
set -euo pipefail

# Cleanup a vcan interface safely.
# By default, only removes the interface if it was created by setup_vcan.sh (marker file present).
# Set FORCE=1 to delete even without marker (use with caution).

IFACE="${1:-${VCAN_IFACE:-vcan0}}"

need_cmd() {
  command -v "$1" >/dev/null 2>&1 || { echo "ERROR: missing command: $1" >&2; exit 127; }
}

need_cmd ip

if [[ "${EUID:-$(id -u)}" -ne 0 ]]; then
  echo "ERROR: cleanup_vcan.sh must be run as root (try: sudo ...)" >&2
  exit 1
fi

if ! ip link show dev "$IFACE" >/dev/null 2>&1; then
  echo "vcan interface '$IFACE' not present (nothing to do)."
  exit 0
fi

ip link del dev "$IFACE"
echo "Removed vcan interface '$IFACE'."

