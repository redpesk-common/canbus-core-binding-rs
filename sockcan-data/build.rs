// build.rs
// Build script configuring an optional `afbv4` cfg flag for this crate.
//
// This script:
// - declares `cfg(afbv4)` as an accepted configuration key for rustc,
// - enables the `afbv4` configuration when the `AFBV4` environment variable is present.
//
fn main() {
    // Inform rustc that `cfg(afbv4)` is a valid configuration key to avoid
    // "unknown cfg" warnings when using `#[cfg(afbv4)]` in the code.
    println!("cargo::rustc-check-cfg=cfg(afbv4)");

    // When the `AFBV4` environment variable is present, propagate a cfg flag to the crate
    // so that conditional compilation blocks guarded by `#[cfg(afbv4)]` are enabled.
    if std::env::var("AFBV4").is_ok() {
        println!("cargo:rustc-cfg=afbv4");
    }
}
