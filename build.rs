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

    let mut build = cc::Build::new();
    build
        .cpp(true)
        .files(glob("vendor/src/*.cpp")?.flatten())
        .std("c++14")
        .define("DDS_THREADS_STL", None)
        .cargo_warnings(false);

    #[cfg(windows)]
    build.define("DDS_THREADS_WINAPI", None);

    #[cfg(any(target_os = "macos", target_os = "ios"))]
    build.define("DDS_THREADS_GCD", None);

    #[cfg(feature = "openmp")]
    env::var("DEP_OPENMP_FLAG")?.split(' ').for_each(|flag| {
        build.flag_if_supported(flag);
    });

    build.try_compile("dds")?;

    #[cfg(feature = "openmp")]
    if let Some(link) = env::var_os("DEP_OPENMP_CARGO_LINK_INSTRUCTIONS") {
        if !link.is_empty() {
            for i in env::split_paths(&link) {
                println!("cargo:{}", i.display());
            }
        }
    }

    Ok(())
}
