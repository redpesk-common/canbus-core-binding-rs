#!/bin/bash
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$(dirname "${SCRIPT_DIR}")/../../../" && pwd)"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-${ROOT_DIR}/target}"
export LD_LIBRARY_PATH="${CARGO_TARGET_DIR}/debug:${CARGO_TARGET_DIR}/release:${LD_LIBRARY_PATH:-}"

afb-binder \
        --traceevt=all \
        --tracereq=all \
        --traceapi=all \
        --config="${SCRIPT_DIR}/../binding-config/binding-config.json" \
        -vvv
