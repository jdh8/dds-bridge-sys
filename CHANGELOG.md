# Changelog

## [Unreleased]

### Tooling

- Restore a committed `.clangd` at the crate root and drop the per-build
  `compile_commands.json` emission from `build.rs`. The build-script write
  broke docs.rs (its workdir is mounted read-only — only `OUT_DIR` is
  writable), and would have shipped that breakage in v3.1.1; v3.1.0 was
  unaffected because it predates the emission. The new `.clangd` supplies
  the flags needed for files in `src/` (where the wrapper and
  `dds_context.{h,cpp}` live) and suppresses diagnostics under `vendor/`,
  which we don't lint anyway. No `cargo build` step is needed to get
  IntelliSense.
- `.gitignore` now also covers the remaining DDS diagnostic-dump prefixes
  (`toplevel*.txt`, `retrieved*.txt`, `stored*.txt`, `sched*.txt`) in
  addition to the previously-listed `ABstats*`/`TTstats*`/`timer*`/
  `movestats*` patterns, so any future debug build never leaves untracked
  files behind.

### Known issues

- DDS's per-thread diagnostic-dump compile flags (`DDS_TOP_LEVEL`,
  `DDS_AB_STATS`, `DDS_TT_STATS`, `DDS_TIMING`, `DDS_MOVES`, `DDS_AB_HITS`,
  and the umbrella `DDS_DEBUG_ALL` in
  `vendor/library/src/utility/debug.h`) are non-buildable in v3.x and
  therefore not exposed as Cargo features. Two upstream issues need fixing
  before any of them can be turned on: the `File` class referenced by
  `ThreadData::file*` members was deleted as "unused" in upstream commit
  `7a3ace6` ("Deletes unused File.h") even though `thread_data.{hpp,cpp}`
  and `solver_if.cpp` still reference it under `#ifdef` guards; and
  `thread_data.hpp` reads `DDS_AB_STATS`/`DDS_TIMING` before
  `utility/debug.h` is included, so the `DDS_DEBUG_ALL` umbrella never
  fans out for that header. Resolving both — most likely by restoring
  `library/src/File.{h,cpp}` (~60 LOC `ofstream` wrapper, recoverable
  from `7a3ace6^` on the `jdh8/dds` fork) and adding `#include "File.h"`
  plus `#include "utility/debug.h"` to `thread_data.hpp` — is a
  prerequisite for any future `debug-*` Cargo feature.

### Fixed

- Repoint the `vendor` submodule at the `jdh8/dds` fork's
  `pons-parallel-calc` branch, which removes the file-scope `ParamType
  cparam` global from `library/src/calc_tables.cpp`. Before the patch, two
  threads each holding their own `DdsSolverContext` and calling
  `dds_calc_dd_table` simultaneously could race on `cparam.bop` /
  `cparam.solvedp` / `cparam.error` and write results into the wrong
  output buffer — contradicting the 3.1.0 changelog's "safe for
  concurrent use across threads" claim, which was true for
  `dds_solve_board` but not for `dds_calc_dd_table`. The patch also drops
  the now-unreachable `scheduler.RegisterRun(...)` call and reduces the
  legacy `calc_single_common` / `copy_calc_single` / `calc_chunk_common`
  shims to empty stubs so `init.cpp`'s `System` constructor still links.

## [3.1.0] - 2026-05-21

### Added

- `DdsSolverContext` opaque handle and `dds_solver_context_*` /
  `dds_solve_board` / `dds_calc_dd_table` / `dds_calc_dd_table_pbn` /
  `dds_calc_par` / `dds_calc_par_from_table` C entry points for DDS 3's
  modern `SolverContext` API. A small `extern "C"` shim
  (`src/dds_context.{h,cpp}`) compiles alongside the vendor sources and is
  bindgen-wrapped like the rest of the FFI surface. One context per OS
  thread; safe for concurrent use across threads when each thread owns
  its own context.

### Documented (existing v3.0.0 behavior)

- Internal batch threading in DDS is removed: `SolveAllBoardsBin`,
  `CalcAllTables`, and `AnalyseAllPlays*` execute sequentially inside
  DDS regardless of `SetMaxThreads(N)`. Concurrent callers must drive
  parallelism from the application side, preferably via per-thread
  `DdsSolverContext` instances.
- Legacy `SolveBoard(threadIndex>0)` returns `RETURN_THREAD_INDEX`; only
  `threadIndex=0` remains valid on the legacy entry point, and each call
  builds a fresh internal context (no TT reuse across calls).

### Internal

- Bindgen now reads `src/wrapper.h` (which includes both `api/dll.h`
  and the new `dds_context.h`) instead of `api/dll.h` directly.
- Bindgen runs with `prepend_enum_name(false)` so enum constants keep
  their natural C names (e.g., `DDS_TT_KIND_LARGE`) without the
  `<TypeName>_` prefix that bindgen would otherwise emit.

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
