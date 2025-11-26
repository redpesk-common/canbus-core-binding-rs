// build.rs
fn main() {
    println!("cargo::rustc-check-cfg=cfg(afbv4)");

    if std::env::var("AFBV4").is_ok() {
        println!("cargo:rustc-cfg=afbv4");
    }
}
