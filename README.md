# canbus-core-binding-rs

Rust bindings and helper libraries to bridge Linux SocketCAN / BCM with the AFB v4 application framework.

This repository provides a reusable CAN bus core binding (`afb_sockcan`), shared data/serialization types, and higher-level DBC-based APIs. It also contains example bindings that demonstrate how to build CAN services on top of AFB and SocketCAN.

---

## goals

- Provide a **robust, modern Rust implementation** of the legacy C/C++ CAN bindings used with AFB.
- Encapsulate **SocketCAN BCM** logic into a reusable core binding.
- Offer **typed data models** for CAN frames, signals, and subscription parameters.
- Provide a **DBC-based API layer** to expose CAN signals/messages in a structured way.
- Include **examples** (BMS, Model3, etc.) to show how to build concrete CAN services.

---

## repository layout

Top-level workspace:

- `Cargo.toml`
  Rust workspace definition, including core crates and examples.

Core crates:

- `sockcan-data/` (`sockcan_data` crate)
  Common types and data converters shared across bindings.
  - `src/lib.rs`
    Crate root, re-exports modules.
  - `src/types.rs`
    Pure data types:
    - CAN BCM frame data (`CanBcmData`, `CanBcmError`, etc.).
    - DBC-related data structures (signals, message wrappers).
    - Subscription parameter structures (`SubscribeParam`, `UnsubscribeParam`, flags).
    - Shared binding configuration (`SockcanBindingConfig`) parsed from JSON.
  - `src/utils.rs`
    Helpers for mapping core CAN errors to AFB errors, plus utility functions.

- `sockcan-binding/` (`sockcan_binding` crate, library name `afb_sockcan`)
  Core SocketCAN BCM binding for AFB.
  - `src/lib.rs`
    Crate root:
    - wires modules (`binding`, `context`, `callbacks`, `verbs`).
    - exposes selected items (e.g. binding config) to other crates.
  - `src/binding.rs`
    Binding entry point:
    - `binding_init(rootv4: AfbApiV4, jconf: JsoncObj)`
      Called by AFB at load time; parses config, registers data converters, creates API, and registers verbs.
  - `src/context.rs`
    Runtime context:
    - `SockcanBindingConfig` (binding configuration structure).
    - SocketCAN client state (BCM socket handle, AFB event).
    - Session context (`AfbSessionRegister!` wrapper) for per-client state.
    - Verb-specific context structures (e.g. subscribe, check).
  - `src/callbacks.rs`
    Runtime callbacks:
    - File descriptor callback for BCM events.
    - Verb handlers: subscribe, unsubscribe, close, check.
    - Error handling, logging, and cleanup logic.
  - `src/verbs.rs`
    API surface:
    - AFB verbs and their registration (`subscribe`, `unsubscribe`, `check`, `close`).
    - Context wiring (`set_context`) and usage strings/samples.

- `dbcapi/` (`dbcapi` crate)
  DBC-based API layer:
  - Provides message pools and signal/message verbs on top of the core CAN binding.
  - Uses `sockcan_data` types and `afbv4` to expose structured APIs.
  - Handles mapping of CAN IDs and signals to DBC-defined messages.
  - Offers helpers to create verbs for reading, subscribing, resetting messages, etc.

Examples:

- `examples/bms/` (`can-bms` crate, library name `afb_bms`)
  BMS-oriented DBC example binding.
  - `Cargo.toml`
    Declares dependencies on `sockcan_data`, `dbcapi`, `afbv4`, `lib_sockcan`, etc.
  - `src/bms-binding.rs`
    Example AFB binding:
    - Reads a JSON configuration for the BMS DBC.
    - Builds a `SockcanBindingConfig` (shared config struct).
    - Uses `create_pool_verbs` from `dbcapi` with a generated message pool (`CanMsgPool`).
  - `build.rs`
    Uses `dbcparser` to generate Rust code from DBC files (`__bms-dbcgen.rs`).
  - `etc/`
    Example configs, DBC files, and CAN dumps.

- `examples/model3/` (`can-model3` crate, library name `afb_model3`)
  Example binding for a Model3-like DBC.
  - Similar structure to `bms`, but using a different DBC/DB and message pool.
  - Shows how to reuse the same core crates with a different CAN application.

Tests and tooling:

- `tests/run.sh`
  Convenience runner for functional tests. Can optionally setup/cleanup a `vcan` interface.
- `tests/setup_vcan.sh`, `tests/cleanup_vcan.sh`
  Helper scripts used by `tests/run.sh` when options are enabled.
- `tests/functional/test_canbus-binding.py`
  Generic functional test runner that loads a DBC and validates AFB events by injecting frames on SocketCAN.
- `tests/functional/test_can_multiplexing.py`
  Multiplexing-focused functional tests.

There may be additional test/example crates (e.g. `examples/test`) used for experimentation and integration testing.

---

## dependencies and requirements

To build and run this project, you typically need:

- **Rust toolchain**
  - Rust stable (edition 2021 is recommended).
  - `cargo` for building workspace crates.

- **AFB v4 Rust bindings**
  - `afbv4` from `afb-librust/afb-librs` (path dependency).

- **SocketCAN core library**
  - `lib_sockcan` from `canbus-core-rs/sockcan` (path dependency).
  - Requires Linux with SocketCAN support (e.g. `vcan0` or real CAN interface).

- **DBC parser**
  - `dbcparser` from `canforge-rs/dbcparser` (path dependency) for example code generation.

- **AFB binder runtime**
  - The generated bindings (`afb_sockcan`, `afb_bms`, `afb_model3`) are meant to be loaded into the AFB binder at runtime via a binding configuration JSON.

The `Cargo.toml` files use relative `path =` dependencies to connect these components. The directory layout must match the paths used in the workspace configuration.

---

## building the workspace

From the root of `canbus-core-binding-rs`:

```bash
cargo build
```

This will:

- build `sockcan_data`, `sockcan_binding` (`afb_sockcan`), and `dbcapi`,
- build the example bindings (`can-bms`, `can-model3`).

If you only want to build a specific crate, use:

```bash
# Build only the core sockcan binding
cargo build -p sockcan_binding

# Build only the BMS example binding
cargo build -p can-bms

# Build only the Model3 example binding
cargo build -p can-model3
```

Make sure the path dependencies in `Cargo.toml` are valid (e.g. `../../sockcan-data`, `../../dbcapi`, `../../../canbus-core-rs/sockcan`, `../../../afb-librust/afb-librs`, etc.).

---

## running with a virtual CAN interface

For development and testing, you can use a virtual CAN (`vcan`) interface instead of real hardware.

### manual setup

As root or with sudo:

```bash
# Load the vcan module (best-effort)
sudo modprobe vcan || true
sudo modprobe can-bcm || true

# Create + bring up vcan0 (idempotent-ish)
sudo ip link add dev vcan0 type vcan || true
sudo ip link set up vcan0
```

Cleanup:

```bash
sudo ip link del dev vcan0
```

### using the test helper scripts

`tests/run.sh` can manage `vcan` for you (see "running the functional tests").

---

## using the sockcan binding in AFB

The `sockcan-binding` crate builds a library called `afb_sockcan` that the AFB binder can load dynamically.

Example binder config snippet (schematic, AFB-side):

```jsonc
{
  "bindings": [
    {
      "uid": "sockcan",
      "path": "/usr/lib/afb/afb_sockcan.so",
      "info": "SocketCAN BCM core binding",
      "args": {
        "dev": "vcan0",
        "uid": "sockcan",
        "sock_api": "sockcan",
        "event_uid": "sockbcm",
        "acls": "acl:sockcan"
      }
    }
  ]
}
```

The binding will:

- parse `args` into a `SockcanBindingConfig`,
- register data converters via `sockdata_register`,
- create an AFB API with the given `uid` and `info`,
- register verbs (`subscribe`, `unsubscribe`, `check`, `close`) in `verbs::register`.

---

## using the DBC API (dbcapi)

The `dbcapi` crate provides tooling to expose DBC-based message pools as AFB verbs:

- You define a **message pool** (generated from a DBC file), typically via a `build.rs` using `dbcparser`.
- The pool type (e.g. `CanMsgPool`) is generated into `__xxx-dbcgen.rs`.
- In your binding, you include the generated file and call:

```rust
include!("./__bms-dbcgen.rs");
use crate::DbcBms::CanMsgPool;

// ...

let pool = Box::new(CanMsgPool::new(api_uid));
create_pool_verbs(rootv4, api, jconf, pool)?;
```

`dbcapi::create_pool_verbs` will:

- Register verbs for:
  - subscribing/unsubscribing to messages/signals,
  - reading/resetting messages/signals,
  - any DBC-specific actions defined in the pool.
- Wire these verbs to the underlying `sockcan` core binding and `sockcan_data` types.

---

## examples

### BMS example (`examples/bms`)

Demonstrates how to build a BMS CAN service on top of:

- `sockcan_binding` (core BCM binding),
- `sockcan_data` (data types and converters),
- `dbcapi` (DBC-based verbs),
- a generated `CanMsgPool` from the BMS DBC.

Key file:

- `src/bms-binding.rs`:
  - Reads JSON config (device, API uid, ACLs, etc.).
  - Constructs a `SockcanBindingConfig` shared with the core binding.
  - Calls `create_pool_verbs` with the generated BMS message pool.

Typical JSON configuration:

```jsonc
{
  "uid": "bms",
  "info": "BMS DBC demo API",
  "dev": "vcan0",
  "can_api": "sockcan",
  "sock_api": "sockcan",
  "acls": "acl:bms"
}
```

### Model3 example (`examples/model3`)

Similar to BMS but using a different DBC/DB:

- Demonstrates how to reuse the same crates with a different message set.
- Serves as a template for building further CAN services.

---

## prerequisites (system)

You need a Linux environment with SocketCAN support.

- `iproute2` (for `ip link ...`)
- `can-utils` (for `canplayer`)
- kernel modules:
  - `vcan`
  - `can-bcm` (recommended)

Example on a typical Fedora/Debian-like system:

```bash
# Fedora
sudo dnf install -y iproute can-utils

# Debian/Ubuntu
sudo apt-get update
sudo apt-get install -y iproute2 can-utils
```

---

## python test dependencies

From the repository root:

```bash
python3 -m venv .venv
. .venv/bin/activate
pip install -U pip

# install Python deps
pip install -r tests/requirements.txt
```

Recommended `tests/requirements.txt` content:

```txt
cantools
python-can
git+https://github.com/redpesk-common/afb-test-py.git@master
```

Notes:

- The Python standard library modules (`argparse`, `json`, `logging`, etc.) do not belong in `requirements.txt`.
- The `afb-test-py` dependency provides the `afb_test` harness and the `libafb` Python bindings used by the functional tests.
- `canplayer` comes from `can-utils` (system package), not via pip.

---

## build the bindings (Rust)

```bash
cargo build --all-targets --all-features
```

Make sure the produced `.so` are reachable at runtime (example for debug builds):

```bash
export LD_LIBRARY_PATH="$PWD/target/debug:${LD_LIBRARY_PATH:-}"
```

---

## functional test runner config format

The generic functional test script (`tests/functional/test_canbus-binding.py`) consumes a JSON file passed with `--config`.

In the sample files under `examples/samples/*/binding-config/binding-config.json`, the binding paths and per-binding settings are split:

- `binding`: list of shared objects to load (only `path` is required)
- `set`: per-binding configuration keyed by the `.so` basename
  - `uid` is stored here (not inside `binding[]`)

Minimal example:

```jsonc
{
  "binding": [
    { "path": "${CARGO_TARGET_DIR}/debug/libafb_sockcan.so" },
    { "path": "${CARGO_TARGET_DIR}/debug/libafb_model3.so" }
  ],
  "set": {
    "libafb_sockcan.so": {
      "uid": "sockbcm",
      "info": "RUST sockbcm/can APIs",
      "sock_api": "sockcan",
      "sock_evt": "sockbcm"
    },
    "libafb_model3.so": {
      "uid": "model3",
      "info": "RUST model3/can APIs",
      "sock_api": "sockbcm",
      "sock_evt": "sockcan"
    }
  }
}
```

Optional top-level fields understood by the test runner:

- `can_api` (string)
- `dbc_file` (path)
- `vcan_iface` (string)
- `canids` (list or CSV string)

---

## running the functional tests

### run all tests

```bash
./tests/run.sh
```

`tests/run.sh` will set:

- `CARGO_TARGET_DIR` (defaults to `<repo>/target` if not already set)
- `LD_LIBRARY_PATH` to include `${CARGO_TARGET_DIR}/debug` (for the generated `.so`)

### optionally setup/cleanup vcan from the runner

```bash
./tests/run.sh --setup_vcan --cleanup_vcan
```

Options:

- `--setup_vcan` creates (or brings up) the `vcan` interface before running tests (requires sudo/root).
- `--cleanup_vcan` attempts to remove the `vcan` interface when the script exits (best-effort, requires sudo/root).
- `--cleanup_vscan` is accepted as an alias for `--cleanup_vcan`.

Select the interface with `VCAN_IFACE`:

```bash
VCAN_IFACE=vcan1 ./tests/run.sh --setup_vcan --cleanup_vcan
```

### filtering which CAN IDs are tested

You can filter tested CAN IDs:

1) CLI option to the test script (used only if `TEST_CANIDS` is unset):

```bash
./tests/functional/test_canbus-binding.py ... --canids 0x221,641
```

### replay a CAN scenario during tests (canplayer)

Some tests may replay a recorded CAN dump to validate subscriptions and events.

Example using the provided Model3 candump sample:

```bash
canplayer vcan0=elmcan -l i -g 1 -I examples/samples/model3/candump/model3.log
```

In the test suite, `canplayer` is typically started via `subprocess.Popen()` while the test waits for the expected AFB event(s).
