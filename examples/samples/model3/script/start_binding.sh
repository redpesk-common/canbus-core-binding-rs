#!/bin/bash
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"

afb-binder \
        --traceevt=all \
        --tracereq=all \
        --traceapi=all \
        --config="${SCRIPT_DIR}/../binding-config/binding-config.json" \
        -vvv
