fn main() {
    // Déclare "afbv4" comme cfg connu → plus de warning
    println!("cargo::rustc-check-cfg=cfg(afbv4)");

    // (facultatif) active réellement le cfg si AFBV4=1
    if std::env::var("AFBV4").is_ok() {
        println!("cargo:rustc-cfg=afbv4");
    }
}
