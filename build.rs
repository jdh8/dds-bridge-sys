use anyhow::Context as _;
use std::path::PathBuf;

fn main() -> anyhow::Result<()> {
    let out_dir = std::env::var_os("OUT_DIR").context("OUT_DIR not set")?;

    bindgen::Builder::default()
        .header("src/wrapper.h")
        .use_core()
        .allowlist_file("(vendor/.*|.*[/\\\\]dds_context\\.h)")
        .clang_arg("-xc++")
        .clang_arg("-Ivendor/library/src")
        .clang_arg("-Isrc")
        .prepend_enum_name(false)
        .derive_default(true)
        .derive_eq(true)
        .derive_hash(true)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()?
        .write_to_file(PathBuf::from(out_dir).join("bindings.rs"))?;

    let mut sources: Vec<PathBuf> = glob::glob("vendor/library/src/**/*.cpp")?
        .flatten()
        .collect();
    sources.push(PathBuf::from("src/dds_context.cpp"));

    let dump = std::env::var_os("CARGO_FEATURE_DEBUG_DUMP").is_some();

    let mut build = cc::Build::new();
    build
        .cpp(true)
        .files(&sources)
        .include("vendor/library/src")
        .include("src")
        .std("c++20")
        .flag("-include")
        .flag("sstream")
        .flag("-include")
        .flag("iomanip")
        .define("DDS_THREADS_STL", None)
        .cargo_warnings(false);
    if !dump {
        build.define("DDS_NO_DUMP_ON_ERROR", None);
    }
    build.try_compile("dds")?;
    Ok(())
}
