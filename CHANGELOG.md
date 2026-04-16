# Changelog

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

[2.0.5]: https://github.com/jdh8/dds-bridge-sys/releases/tag/2.0.5
[2.0.4]: https://github.com/jdh8/dds-bridge-sys/releases/tag/2.0.4
