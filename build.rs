use glob::glob;
use std::env;
use std::path::PathBuf;

fn main() -> anyhow::Result<()> {
    cc::Build::new()
        .cpp(true)
        .files(glob("vendor/src/*.cpp")?.flatten())
        .define("DDS_THREADS_STL", None)
        .compile("dds");
    bindgen::Builder::default()
        .header("vendor/src/dds.h")
        .use_core()
        .allowlist_file("vendor/.*")
        .clang_arg("-xc++")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()?
        .write_to_file(PathBuf::from(env::var("OUT_DIR")?).join("bindings.rs"))?;
    println!("cargo:rustc-link-lib=dds");

    #[cfg(any(target_os = "macos", target_os = "ios", target_os = "freebsd", target_os = "dragonfly", target_os = "netbsd", target_os = "openbsd"))]
    println!("cargo:rustc-link-lib=c++");
    #[cfg(any(target_os = "linux", target_os = "none"))]
    println!("cargo:rustc-link-lib=stdc++");
    #[cfg(target_os = "android")]
    println!("cargo:rustc-link-lib=c++_shared");
    Ok(())
}