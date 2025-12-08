// Build script configuring conditional compilation flags for afbv4 integration.
fn main() {
    // Instruct Cargo to accept the custom `afbv4` cfg so that unknown cfg warnings are avoided.
    println!("cargo::rustc-check-cfg=cfg(afbv4)");

    // Propagate the `afbv4` cfg flag to the crate when the AFBV4 environment variable is present.
    if std::env::var("AFBV4").is_ok() {
        println!("cargo:rustc-cfg=afbv4");
    }
}
