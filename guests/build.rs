//! Hands the linker our memory layout (`link.ld`), wherever cargo is invoked from.

fn main() {
    let dir = std::env::var("CARGO_MANIFEST_DIR").expect("cargo sets CARGO_MANIFEST_DIR");
    println!("cargo:rustc-link-search={dir}");
    println!("cargo:rustc-link-arg-bins=-Tlink.ld");
    println!("cargo:rerun-if-changed=link.ld");
}
