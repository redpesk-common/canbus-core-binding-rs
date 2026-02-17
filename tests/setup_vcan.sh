#!/usr/bin/env bash
set -euo pipefail

# Setup a vcan interface in an idempotent way.
# Creates a marker file so cleanup only removes what we created.

IFACE="${1:-${VCAN_IFACE:-vcan0}}"

need_cmd() {
  command -v "$1" >/dev/null 2>&1 || { echo "ERROR: missing command: $1" >&2; exit 127; }
}

need_cmd ip

# vcan requires root. Fail fast with a clear message.
if [[ "${EUID:-$(id -u)}" -ne 0 ]]; then
  echo "ERROR: setup_vcan.sh must be run as root (try: sudo ./tests/run.sh --setup_vcan ...)" >&2
  exit 1
fi

# Load vcan kernel module if possible (best-effort).
if command -v modprobe >/dev/null 2>&1; then
  modprobe vcan >/dev/null 2>&1 || true
fi

if ip link show dev "${IFACE}" >/dev/null 2>&1; then
  # Interface already exists: just ensure it's up.
  ip link set dev "${IFACE}" up
  echo "vcan interface ${IFACE} already exists (kept)."
  exit 0
fi

# Create + bring up.
ip link add dev "${IFACE}" type vcan
ip link set dev "${IFACE}" up

echo "Created vcan interface ${IFACE}."
