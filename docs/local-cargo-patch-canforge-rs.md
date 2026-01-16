# developer guide: local overrides for git dependencies (canforge-rs and afb-librust)

This project consumes Rust crates hosted in external git repositories. For day-to-day development, contributors should be able to work with **local checkouts** of those repositories **without modifying this project’s sources**.

The recommended approach is a **per-developer Cargo patch** in `~/.cargo/config.toml`.

This document covers two upstream repositories used by this project:

- `https://github.com/redpesk-common/canforge-rs.git` (e.g., `dbcparser`)
- `https://github.com/redpesk-common/afb-librust` (crate `afbv4`)

## goals and constraints

- Do not commit local `path = ...` dependencies into this repository.
- Developers can switch between upstream git sources and local checkouts by editing only:
  - `~/.cargo/config.toml`
- Keep the consumer repository buildable for CI using upstream git dependencies.

## prerequisites

- Rust toolchain installed (stable recommended)
- `git`

Optional:

- `cargo-tree` comes with Cargo (`cargo tree` is available on recent toolchains)
- `cargo-edit`

## how Cargo patching works (overview)

Cargo supports overriding dependency sources via `[patch]` tables.

When a crate is pulled from a specific git URL, you can replace that crate with a local checkout using:

- a patch key matching the git URL exactly, and
- a `path` entry pointing to the crate directory containing the crate’s `Cargo.toml`.

This happens **locally** and does not require changing any project manifests.

## 1) clone the upstream repositories locally

Pick a place for local development checkouts:

```bash
mkdir -p ~/dev/redpesk-common
cd ~/dev/redpesk-common
```

Clone `canforge-rs`:

```bash
git clone https://github.com/redpesk-common/canforge-rs.git
```

Clone `afb-librust`:

```bash
git clone https://github.com/redpesk-common/afb-librust
```

You may optionally checkout a specific branch:

```bash
cd ~/dev/redpesk-common/canforge-rs
git checkout master

cd ~/dev/redpesk-common/afb-librust
git checkout master
```

## 2) identify the crate names and paths

Patching requires the **crate package name** and the **path to the crate directory**.

### canforge-rs: dbcparser

Typical layout:

- crate directory: `canforge-rs/dbcparser`
- crate package name: usually `dbcparser`

Confirm the package name:

```bash
cd ~/dev/redpesk-common/canforge-rs
cat dbcparser/Cargo.toml | grep -E '^name\s*='
```

### afb-librust: afbv4

Important: the crate package name is **`afbv4`**, even though the directory is `afb-librs`.

Typical layout:

- crate directory: `afb-librust/afb-librs`
- crate package name: `afbv4` (from `[package] name = "afbv4"`)

Confirm the package name:

```bash
cd ~/dev/redpesk-common/afb-librust
cat afb-librs/Cargo.toml | grep -E '^name\s*='
```

## 3) configure per-developer patches in ~/.cargo/config.toml

Create or edit:

- `~/.cargo/config.toml`

Add patches for the crates you want to override.

### recommended example (both overrides)

```toml
# ~/.cargo/config.toml

[patch."https://github.com/redpesk-common/canforge-rs.git"]
dbcparser = { path = "/home/<you>/dev/redpesk-common/canforge-rs/dbcparser" }

[patch."https://github.com/redpesk-common/afb-librust"]
afbv4 = { path = "/home/<you>/dev/redpesk-common/afb-librust/afb-librs" }
```

Notes:

- Use absolute paths to reduce ambiguity.
- Replace `/home/<you>/...` with your real local paths.
- The patch table key must match the dependency git URL exactly (including `.git` when present).

### patching multiple crates from the same repository (optional)

If the consumer project uses additional crates from `canforge-rs`, add them under the same patch table:

```toml
[patch."https://github.com/redpesk-common/canforge-rs.git"]
dbcparser = { path = "/home/<you>/dev/redpesk-common/canforge-rs/dbcparser" }
some-other-crate = { path = "/home/<you>/dev/redpesk-common/canforge-rs/some-other-crate" }
```

## 4) build the consumer project normally

From this repository root:

```bash
cargo build --all-targets --all-features
cargo test --all-features
```

Cargo will still resolve dependencies as coming from their git sources, but will replace the selected crates with your local versions.

## 5) verify patches are applied

From this repository root:

```bash
cargo tree | grep -E "dbcparser|canforge-rs"
cargo tree | grep -E "afbv4|afb-librust"
```

For reverse dependency inspection:

```bash
cargo tree -i dbcparser
cargo tree -i afbv4
```

## 6) common issues and fixes

### patch does not apply

Checklist:

- The patch key matches the git URL exactly (including `.git`).
- The patched crate key matches the crate `package.name`.
- The `path` points to the directory containing the crate’s `Cargo.toml`.

### “no matching package named … found” for afb-librust

If you see an error like:

- `no matching package named 'afb-librs' found`

This means you attempted to reference the package name as `afb-librs`. In this repository, the correct crate **package name** is:

- `afbv4`

The directory name `afb-librs` is not the package name.

### lockfile or stale build behavior

When switching between patched and non-patched builds:

```bash
cargo clean
cargo build --all-targets --all-features
```

If needed:

```bash
rm -rf target
cargo build --all-targets --all-features
```

## 7) disabling local overrides

To return to upstream git sources:

- remove or comment out the corresponding `[patch."..."]` sections in `~/.cargo/config.toml`

Then rebuild:

```bash
cargo clean
cargo build --all-targets --all-features
```

## 8) team convention (recommended)

- Keep upstream git dependencies in project manifests for reproducible CI builds.
- Do not commit local path overrides into this repository.
- Use this document as the standard workflow for local iteration on upstream crates.
