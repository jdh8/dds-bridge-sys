use anyhow::Context as _;

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
        .write_to_file(std::path::PathBuf::from(out_dir).join("bindings.rs"))?;

    let sources: Vec<_> = glob::glob("vendor/library/src/**/*.cpp")?
        .flatten()
        .collect();

    let mut build = cc::Build::new();
    build
        .cpp(true)
        .files(&sources)
        .file("src/dds_context.cpp")
        .include("vendor/library/src")
        .include("src")
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
