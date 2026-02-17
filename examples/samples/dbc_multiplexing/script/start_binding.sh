#!/bin/bash

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"

#afb-binder \
#        --traceevt=all \
#        --tracereq=all \
#        --traceapi=all \
#        --config="${SCRIPT_DIR}/../binding-config/binding-config.json" \
#        -vvv

afb-binder --traceevt=all --tracereq=all --traceapi=all \
                --config="${SCRIPT_DIR}/../binding-config/binding-config-mini.json" \
                --set libafb_sockcan.so/sock_api:sockcan \
                --set libafb_sockcan.so/sock_evt:sockbcm  \
                --set libafb_sockcan.so/uid:sockbcm  \
                --set libafb_can_multiplexing.so/sock_api:sockbcm\
                --set libafb_can_multiplexing.so/sock_evt:sockcan \
                --set libafb_can_multiplexing.so/uid:can-multiplexing \
                -vvv
