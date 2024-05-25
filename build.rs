use glob::glob;
use std::env;
use std::path::PathBuf;

fn main() -> anyhow::Result<()> {
    // This line must precede the cc::Build::try_compile call to ensure correct
    // linking order.
    println!("cargo:rustc-link-lib=dds");

    bindgen::Builder::default()
        .header("vendor/src/dds.h")
        .use_core()
        .allowlist_file("vendor/.*")
        .clang_arg("-xc++")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()?
        .write_to_file(PathBuf::from(env::var("OUT_DIR")?).join("bindings.rs"))?;

    cc::Build::new()
        .cpp(true)
        .files(glob("vendor/src/*.cpp")?.flatten())
        .define("DDS_THREADS_STL", None)
        .flag_if_supported("-flto")
        .flag_if_supported("/GL")
        .try_compile("dds")?;

    Ok(())
}
