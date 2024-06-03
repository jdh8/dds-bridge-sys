use glob::glob;
use std::env;
use std::path::PathBuf;

fn main() -> anyhow::Result<()> {
    bindgen::Builder::default()
        .header("vendor/src/dds.h")
        .use_core()
        .allowlist_file("vendor/.*")
        .clang_arg("-xc++")
        .derive_default(true)
        .derive_eq(true)
        .derive_hash(true)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()?
        .write_to_file(PathBuf::from(env::var("OUT_DIR")?).join("bindings.rs"))?;

    cc::Build::new()
        .cpp(true)
        .files(glob("vendor/src/*.cpp")?.flatten())
        .std("c++14")
        .define("DDS_THREADS_STL", None)
        .cargo_warnings(false)
        .try_compile("dds")?;

    Ok(())
}
