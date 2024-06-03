use anyhow::Context as _;

fn main() -> anyhow::Result<()> {
    let out_dir = std::env::var_os("OUT_DIR").context("OUT_DIR not set")?;

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
        .write_to_file(std::path::PathBuf::from(out_dir).join("bindings.rs"))?;

    cc::Build::new()
        .cpp(true)
        .files(glob::glob("vendor/src/*.cpp")?.flatten())
        .std("c++14")
        .define("DDS_THREADS_STL", None)
        .cargo_warnings(false)
        .try_compile("dds")?;

    Ok(())
}
