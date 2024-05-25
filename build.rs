use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rustc-link-lib=dds");
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("Cargo should have provided OUT_DIR"));

    bindgen::Builder::default()
        .header("vendor/src/dds.h")
        .use_core()
        .allowlist_file("vendor/.*")
        .clang_arg("-xc++")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Failed to generate bindings")
        .write_to_file(out_dir.join("bindings.rs"))
        .expect("Failed to write bindings");
}