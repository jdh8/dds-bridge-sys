use glob::glob;
use std::env;
use std::path::PathBuf;

#[cfg(windows)]
#[cfg(feature = "openmp")]
compile_error!("DDS does not use OpenMP when there is Windows API support");

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
        .std("c++14")
        .define("DDS_THREADS_STL", None)
        .cargo_warnings(false)
        .flag_if_supported("-flto")
        .flag_if_supported("/GL");

    #[cfg(windows)]
    build.define("DDS_THREADS_WINAPI", None);

    #[cfg(any(target_os = "macos", target_os = "ios"))]
    build.define("DDS_THREADS_GCD", None);

    #[cfg(any(feature = "openmp", all(target_os = "linux", feature = "linux-openmp")))]
    build.define("DDS_THREADS_OPENMP", None).flag("-fopenmp");

    build.try_compile("dds")?;

    // This line must follow the cc::Build::try_compile call to ensure correct
    // linking order.
    #[cfg(any(feature = "openmp", all(target_os = "linux", feature = "linux-openmp")))]
    println!("cargo:rustc-link-arg=-fopenmp");

    Ok(())
}
