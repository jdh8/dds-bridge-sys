use anyhow::Context as _;

fn main() -> anyhow::Result<()> {
    let out_dir = std::env::var_os("OUT_DIR").context("OUT_DIR not set")?;

    bindgen::Builder::default()
        .header("vendor/library/src/api/dll.h")
        .use_core()
        .allowlist_file("vendor/.*")
        .clang_arg("-xc++")
        .clang_arg("-Ivendor/library/src")
        .derive_default(true)
        .derive_eq(true)
        .derive_hash(true)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()?
        .write_to_file(std::path::PathBuf::from(out_dir).join("bindings.rs"))?;

    let sources: Vec<_> = glob::glob("vendor/library/src/**/*.cpp")?
        .flatten()
        .collect();

    let mut build = cc::Build::new();
    build
        .cpp(true)
        .files(&sources)
        .include("vendor/library/src")
        .std("c++20")
        .flag("-include")
        .flag("sstream")
        .flag("-include")
        .flag("iomanip")
        .define("DDS_THREADS_STL", None)
        .cargo_warnings(false);

    if std::env::var_os("CARGO_FEATURE_DEBUG_DUMP").is_none() {
        build.define("DDS_NO_DUMP_ON_ERROR", None);
    }

    build.try_compile("dds")?;
    Ok(())
}
