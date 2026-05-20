# Changelog

## [3.0.0] - 2026-05-20

### Breaking

- Bump vendored `dds` library to **v3.0.0** (was v2.8.2-220). Bindgen now reads
  the public C API header `vendor/library/src/api/dll.h` only; the broader set
  of C++ internals previously surfaced through `vendor/src/dds.h` is no longer
  exposed in the FFI bindings.
- Upstream renamed every public struct from camelCase to PascalCase and renamed
  several struct fields from camelCase to snake_case. The bindgen output now
  mirrors these names verbatim. Notable renames:
  - `futureTricks` → `FutureTricks`
  - `deal` → `Deal`, `dealPBN` → `DealPBN`
  - `boards` → `Boards`, `boardsPBN` → `BoardsPBN`
  - `solvedBoards` → `SolvedBoards` (field `solvedBoard` → `solved_board`)
  - `ddTableDeal[s]` / `ddTableDeal[s]PBN` → `DdTableDeal[s]` /
    `DdTableDeal[s]PBN` (field `noOfTables` → `no_of_tables`)
  - `ddTableResults` → `DdTableResults` (field `resTable` → `res_table`)
  - `ddTablesRes` → `DdTablesRes`
  - `parResults` → `ParResults`, `allParResults` → `AllParResults`
    (field `presults` → `par_results`)
  - `parResultsDealer` → `ParResultsDealer`
  - `parResultsMaster` → `ParResultsMaster`
  - `parTextResults` → `ParTextResults`
  - `contractType` → `ContractType` (fields `underTricks` → `under_tricks`,
    `overTricks` → `over_tricks`)
  - `playTraceBin` / `playTracePBN` / `playTracesBin` / `playTracesPBN` →
    `PlayTraceBin` / `PlayTracePBN` / `PlayTracesBin` / `PlayTracesPBN`
  - `solvedPlay[s]` → `SolvedPlay[s]`
  - `Boards`, `SolvedBoards`, `DdTablesRes`, `PlayTracesBin`, `SolvedPlays`
    each rename `noOfBoards` → `no_of_boards`
  - `DDSInfo` is unchanged.
- Removed Cargo features `debug-timing`, `debug-ab-stats`, `debug-tt-stats`,
  `debug-moves`. The corresponding upstream code paths no longer compile in
  v3.0.0. `debug-dump` is retained.

### Changed

- Build C++ sources with `-std=c++20` (was c++14), matching upstream's Bazel
  config.
- Compile the full new vendor tree (`vendor/library/src/**/*.cpp` across
  `system/`, `moves/`, `heuristic_sorting/`, `utility/`, `lookup_tables/`,
  `solver_context/`, `trans_table/`) instead of only `vendor/src/*.cpp`.
- Force-include `<sstream>` and `<iomanip>` via `cc::Build` flags to work
  around upstream sources that depend on these standard headers transitively
  but never include them.

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
- Ignore diagnostic dumps emitted by the `debug-*` features (`dump.txt`,
  `timer*.txt`, `ABstats*.txt`, `TTstats*.txt`, `movestats*.txt`) in
  `.gitignore`

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
