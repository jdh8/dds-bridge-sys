# Changelog

## [2.1.1] - 2026-04-24

### Changed

- Simplify `examples/info.rs` to print DDS's `systemString` (comprehensive
  version/compiler/system info) in a single line, instead of per-field output
  for version, thread count, and thread sizes

### Internal

- Rename `mod test` to `mod tests` in `src/lib.rs` to mirror the `tests/`
  directory convention and avoid visual collision with `#[cfg(test)]`
- Allow `clippy::all` and `clippy::pedantic` in the generated bindings module so
  lints aren't enforced on auto-generated code
- Add `CLAUDE.md` with contributor notes for AI-assisted development
- Require Windows CI to pass: drop `continue-on-error` on the Windows test
  matrix leg so failures block the workflow instead of being silently ignored

## [2.1.0] - 2026-04-20

### Added

- Cargo features `debug-dump`, `debug-timing`, `debug-ab-stats`,
  `debug-tt-stats`, `debug-moves` — enable upstream DDS debug/profiling
  output files (off by default; each emits per-thread `.txt` files into
  the cwd)

## [2.0.5] - 2026-04-17

### Documentation

- Document thread safety categories for all functions in README, covering non-reentrant, reentrant (explicit `threadIndex`), always-safe, and thread-pool management groups
- Add reference to the higher-level `dds-bridge` crate and its `Solver` type

### Dependencies

- Relax build-dependency version pins to minor ranges (`anyhow = "1"`, `bindgen = "0.72"`, `cc = "1"`, `glob = "0.3"`)
- Replace `once_cell` dev-dependency with `parking_lot = "0.12.5"` for simpler mutex handling in tests

### Internal

- Simplify test mutex using `parking_lot::Mutex` + `std::sync::LazyLock` (Rust 1.85 stable), removing manual poison handling
- Use `&raw mut` instead of `&mut` for raw pointer coercions in test FFI calls
- Set `rust-version = "1.85"` in `Cargo.toml`
- Add `homepage`, `documentation`, and `readme` fields to `Cargo.toml`
- Fix GitHub workflow to recurse submodules when fetching the C++ dependency

## [2.0.4] - 2026-03-18

- Update Rust to 2024
- Update bindgen

[2.1.1]: https://github.com/jdh8/dds-bridge-sys/releases/tag/2.1.1
[2.1.0]: https://github.com/jdh8/dds-bridge-sys/releases/tag/2.1.0
[2.0.5]: https://github.com/jdh8/dds-bridge-sys/releases/tag/2.0.5
[2.0.4]: https://github.com/jdh8/dds-bridge-sys/releases/tag/2.0.4
