#!/bin/bash

# use libafb development version if any
export LD_LIBRARY_PATH="/usr/local/lib64:$LD_LIBRARY_PATH"
export PATH="/usr/local/lib64:$PATH"
clear

if ! test -f ${CARGO_TARGET_DIR}/debug/examples/libafb_sockbmc.so; then
    cargo build --target-dir=${CARGO_TARGET_DIR}
    if test $? != 0; then
        echo "FATAL: fail to compile libafb sample"
        exit 1
    fi
fi

# rebuilt test binding
cargo build --target-dir=${CARGO_TARGET_DIR} --example tap_sockbmc
if test $? != 0; then
    echo "FATAL: fail to compile test suite"
    exit 1
fi

# start binder with test config
afb-binder --config=examples/test/etc/binding-test-tap.json
