static CARGO_MANIFEST_DIR: &str = env!("CARGO_MANIFEST_DIR");

fn main() {
    println!("cargo:rerun-if-changed=src/linker.ld");
    println!(
        "cargo:rustc-link-arg=-T{}/src/linker.ld",
        CARGO_MANIFEST_DIR
    );
}
