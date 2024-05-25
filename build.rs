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

    let mut build = cc::Build::new();
    build
        .cpp(true)
        .files(glob("vendor/src/*.cpp")?.flatten())
        .define("DDS_THREADS_STL", None)
        .flag_if_supported("-flto")
        .flag_if_supported("/GL");

    #[cfg(windows)]
    build.define("DDS_THREADS_WINAPI", None);

    #[cfg(any(target_os = "macos", target_os = "ios"))]
    build.define("DDS_THREADS_GCD", None);

    #[cfg(target_os = "linux")]
    {
        build.define("DDS_THREADS_OPENMP", None);
        env::var("DEP_OPENMP_FLAG")?.split(' ').for_each(|flag| {
            build.flag(flag);
        });
    }

    build.try_compile("dds")?;

    #[cfg(target_os = "linux")]
    if let Some(link) = env::var_os("DEP_OPENMP_CARGO_LINK_INSTRUCTIONS") {
        env::split_paths(&link).for_each(|i| println!("cargo:{}", i.display()));
    }

    Ok(())
}
