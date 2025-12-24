# Benchmarks for `canbus-core-binding-rs`

This document explains how to add, run, and interpret **Criterion** micro-benchmarks in the `canbus-core-binding-rs` workspace.

This repository is a workspace composed of multiple crates (notably `sockcan_data`, `sockcan_binding`, and `dbcapi`). Benchmarks should focus on **in-memory** hot paths such as:

- construction of commonly used data structures (e.g. `sockdata::types::CanBcmData`),
- serialization/deserialization (e.g. `serde_json` roundtrips for API payloads),
- pure helper/utility logic (ID mapping, conversions, validations),
- DBC-related processing in `dbcapi` (when it is isolated from I/O).

Benchmarks must avoid real SocketCAN I/O and AFB runtime side-effects to keep results stable and representative.

---

## 1. prerequisites

Before running benchmarks:

- Use a recent **Rust stable** toolchain (edition 2021).
- Run commands from the **workspace root** (the directory containing the root `Cargo.toml`).
- Ensure the workspace **builds successfully**, including any `path =` dependencies declared in the crates (e.g. `lib_sockcan`, `afbv4`).

No CAN hardware is required for micro-benchmarks that only exercise data construction and pure functions.

---

## 2. where benchmarks live

Criterion benchmarks live per crate under:

- `<crate>/benches/*.rs`

Examples:

- `sockcan-data/benches/*.rs` (recommended starting point)
- `dbcapi/benches/*.rs` (DBC APIs and message pool logic, where possible)

Note: the `sockcan_data` **package** is named `sockcan_data`, but its **library crate name** is `sockdata` (see `[lib] name = "sockdata"` in `sockcan-data/Cargo.toml`). Therefore benches should import `sockdata::...`.

---

## 3. enabling Criterion in a crate

Add Criterion under `dev-dependencies` and declare the bench target in the crate where you want to benchmark.

Example for `sockcan-data/Cargo.toml`:

```toml
[dev-dependencies]
criterion = { version = "0.5", features = ["html_reports"] }

[[bench]]
name = "sockdata_types"
harness = false
```

- `harness = false` is required because Criterion provides its own harness.

---

## 4. example benchmark

Create `sockcan-data/benches/sockdata_types.rs`:

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_can_bcmdata_new(c: &mut Criterion) {
    c.bench_function("sockdata::CanBcmData::new (8 bytes)", |b| {
        b.iter(|| {
            let canid = black_box(0x123u32);
            let stamp = black_box(42u64);
            let len = black_box(8u8);

            let opcode = black_box(sockcan::prelude::CanBcmOpCode::RxSetup);

            // NOTE: This benchmark intentionally includes the Vec allocation.
            // Once we move to slice-based APIs, we can benchmark the no-allocation path separately.
            let data = vec![1u8, 2, 3, 4, 5, 6, 7, 8];

            let frame = sockdata::types::CanBcmData::new(canid, opcode, stamp, data, len);

            black_box(frame.get_id());
            black_box(frame.get_stamp());
            black_box(frame.get_len());
            black_box(frame.get_opcode());
        })
    });
}

criterion_group!(benches, bench_can_bcmdata_new);
criterion_main!(benches);
```

If the `CanBcmOpCode` variant differs in your environment, replace `RxSetup` with a valid variant from your `sockcan` dependency.

---

## 5. running benchmarks

From the workspace root:

```bash
# Run all benchmarks for the sockcan_data package
cargo bench -p sockcan_data
```

To run a specific benchmark binary:

```bash
cargo bench -p sockcan_data --bench sockdata_types
```

Criterion outputs:

- a text summary in the terminal,
- HTML reports under `target/criterion/`.

---

## 6. recommended bench profile (optional)

To make results more representative, add the following to the workspace root `Cargo.toml`:

```toml
[profile.bench]
lto = true
codegen-units = 1
```

This improves optimization consistency at the cost of longer compile times.

---

## 7. what to benchmark in this repository

Suggested benchmark categories (progressively):

### 7.1 `sockcan_data`

- `CanBcmData` construction (with/without allocation, once APIs allow it)
- `serde_json` encode/decode for API payload types used frequently
- conversion helpers in `sockcan-data/src/utils.rs` (if they are pure and stable)

### 7.2 `dbcapi`

- message pool update paths (in-memory)
- signal decode/encode logic where it does not require socket activity
- lookup and mapping logic (CAN ID → message/signal routing)

---

## 8. best practices for stable and meaningful results

- Keep the benchmark **hot loop** minimal; prepare inputs outside `b.iter`.
- Use `black_box` for inputs and outputs to prevent the optimizer from removing work.
- Avoid syscalls, sockets, filesystem, and thread synchronization in micro-benchmarks.
- Separate benchmarks by scenario (e.g. “small payload”, “max payload”, “error path”).
- Prefer deterministic datasets checked into the repo for reproducibility.

---

## 9. CI considerations

Benchmarks are usually not run in CI by default due to noise and runtime cost. Recommended approach:

- run `cargo bench` locally before/after performance-sensitive refactors,
- optionally add a manual CI workflow to run benchmarks on a controlled runner.

---
