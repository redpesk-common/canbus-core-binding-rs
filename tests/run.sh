#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$ROOT_DIR/target}"
export LD_LIBRARY_PATH="$CARGO_TARGET_DIR/debug:${LD_LIBRARY_PATH:-}"

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
    --vcan-iface vcan0 \
    --canids 257,641 \
    --debug \
    -v
