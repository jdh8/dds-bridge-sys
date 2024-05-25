use glob::glob;
use std::env;
use std::path::PathBuf;

fn main() -> anyhow::Result<()> {
    cc::Build::new()
        .cpp(true)
        .files(glob("vendor/src/*.cpp")?.flatten())
        .define("DDS_THREADS_STL", None)
        .static_flag(true)
        .shared_flag(false)
        .compile("dds");
    bindgen::Builder::default()
        .header("vendor/src/dds.h")
        .use_core()
        .allowlist_file("vendor/.*")
        .clang_arg("-xc++")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()?
        .write_to_file(PathBuf::from(env::var("OUT_DIR")?).join("bindings.rs"))?;
    println!("cargo:rustc-link-lib=dds");
    Ok(())
}