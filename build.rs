use anyhow::Context as _;
use std::path::{Path, PathBuf};

fn main() -> anyhow::Result<()> {
    let out_dir = std::env::var_os("OUT_DIR").context("OUT_DIR not set")?;
    let manifest_dir = PathBuf::from(
        std::env::var_os("CARGO_MANIFEST_DIR").context("CARGO_MANIFEST_DIR not set")?,
    );

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

    // GCC/clang-spelled mirror of the cc::Build flags above, for clangd's
    // compile_commands.json. Keep in sync with the cc::Build call.
    let mut flags: Vec<String> = vec![
        "-xc++".into(),
        "-std=c++20".into(),
        "-Ivendor/library/src".into(),
        "-Isrc".into(),
        "-include".into(),
        "sstream".into(),
        "-include".into(),
        "iomanip".into(),
        "-DDDS_THREADS_STL".into(),
    ];
    if !dump {
        flags.push("-DDDS_NO_DUMP_ON_ERROR".into());
    }
    write_compile_commands(&manifest_dir, &sources, &flags)?;
    Ok(())
}

fn write_compile_commands(
    manifest_dir: &Path,
    sources: &[PathBuf],
    flags: &[String],
) -> anyhow::Result<()> {
    let dir = manifest_dir.display().to_string();
    let entries: Vec<String> = sources
        .iter()
        .map(|p| {
            let file = p.display().to_string();
            let args = std::iter::once("clang++".to_string())
                .chain(flags.iter().cloned())
                .chain(["-c".into(), file.clone()])
                .map(|s| json_string(&s))
                .collect::<Vec<_>>()
                .join(", ");
            format!(
                "  {{ \"directory\": {}, \"file\": {}, \"arguments\": [{}] }}",
                json_string(&dir),
                json_string(&file),
                args,
            )
        })
        .collect();
    std::fs::write(
        manifest_dir.join("compile_commands.json"),
        format!("[\n{}\n]\n", entries.join(",\n")),
    )
    .context("writing compile_commands.json")?;
    Ok(())
}

fn json_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            '\r' => out.push_str("\\r"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}
