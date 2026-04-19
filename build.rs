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

    let mut build = cc::Build::new();
    build
        .cpp(true)
        .files(glob::glob("vendor/src/*.cpp")?.flatten())
        .std("c++14")
        .define("DDS_THREADS_STL", None)
        .cargo_warnings(false);

    if std::env::var_os("CARGO_FEATURE_DEBUG_DUMP").is_none() {
        build.define("DDS_NO_DUMP_ON_ERROR", None);
    }

    for (feat, macro_name) in [
        ("CARGO_FEATURE_DEBUG_TIMING", "DDS_TIMING"),
        ("CARGO_FEATURE_DEBUG_AB_STATS", "DDS_AB_STATS"),
        ("CARGO_FEATURE_DEBUG_TT_STATS", "DDS_TT_STATS"),
        ("CARGO_FEATURE_DEBUG_MOVES", "DDS_MOVES"),
    ] {
        if std::env::var_os(feat).is_some() {
            build.define(macro_name, None);
        }
    }

    build.try_compile("dds")?;
    Ok(())
}
