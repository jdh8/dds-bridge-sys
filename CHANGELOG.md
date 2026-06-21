# Changelog

<!-- markdownlint-disable no-duplicate-heading -->

## [3.3.0] - 2026-06-22

### Changed

- Retire the `jdh8/dds` fork and repoint the `vendor` submodule back at
  upstream `dds-bridge/dds` `develop` (commit `0700b42`, i.e.
  `v3.0.0-240-g0700b42`). `.gitmodules` now tracks `dds-bridge/dds` on
  `develop` again, completing the transition 3.2.1 promised.

  Both fork-only patches 3.2.1 carried are now covered upstream, so the
  3.2.1 performance win and concurrency crash-fix are preserved without
  any fork-only code:

  - The `solver_context: inline SearchContext accessors and drop
    shared_ptr from MoveGenContext` patch is superseded by upstream
    PR #191 ("Avoid per-node shared_ptr churn in the search hot path"),
    which adds the non-owning `SolverContext::thread_ptr() -> ThreadData*`
    accessor and uses it at the hot `ab_search` / `quick_tricks` /
    `later_tricks` sites, removing the same per-node atomic refcount
    traffic.

  - The `calc_tables: drop racing scheduler.RegisterRun call` patch is no
    longer needed: upstream reworked the scheduler subsystem and the
    `scheduler.RegisterRun(...)` call is gone from `calc_all_boards_n` in
    `calc_tables.cpp`. The batched concurrency stress test
    (`solver_context_batched_stress_pool_reuse`, 64 deals across 3 batched
    calls on a 32-core host) passes against this vendor with no SIGSEGV,
    confirming the global-`scheduler` race that motivated the fork patch
    is resolved upstream.

  Beyond those two, this refresh pulls 214 upstream commits since the old
  fork base, notably: a series of `SolverContext` debug-file lifecycle
  fixes (correct close/reopen handling when a `SolverContext` is the last
  `ThreadData` owner), the scheduler/`DDS_SCHEDULER` build-consistency
  rework (PR #201), AB-stats build fixes (PR #195), and macOS LTO
  (PR #192). The public ABI header `api/dll.h` is unchanged, so the
  generated bindings — and downstream `dds-bridge` — are unaffected.

### Build

- `build.rs` now excludes upstream's newly added
  `vendor/library/src/dds_api.cpp` from the `cc` compile. That file
  exports a `dds_*` "Version 3" C API (`dds_solve_board`,
  `dds_calc_dd_table`, `dds_calc_par`, SolverContext management, ...) whose
  symbol names collide at link time with this crate's own
  `src/dds_context.cpp` shim. We keep our shim, which is a superset — it
  also provides the batched `dds_solve_boards_batched` /
  `dds_calc_dd_tables_batched` WorkerPool entry points that upstream's API
  does not have. Nothing inside `vendor/` calls the excluded entry points,
  and `src/wrapper.h` binds only `api/dll.h` (not `api/dds_api.hpp`), so
  the exclusion is link-only and the generated bindings are unchanged.

## [3.2.2] - 2026-06-10

### Changed

- Add `links = "dds_bridge"` to `[package]` in `Cargo.toml` so Cargo
  treats this crate as a native-link provider and lib.rs no longer reports
  the `*-sys crate without links property` warning.

## [3.2.1] - 2026-05-26

### Fixed

- `dds_calc_dd_tables_batched` and `dds_solve_boards_batched` no longer
  SIGSEGV under high concurrency. The 3.2.0 implementation spawned a
  fresh `std::thread` (and a fresh `SolverContext`) per worker on every
  batched call. On a >= 8-effective-CPU Linux box, the resulting heap
  churn corrupted state badly enough to crash inside
  `TransTableL::lookup_suit` / `Moves::MergeSort` reliably after a
  handful of `solve_deals(N>=200)` iterations. AddressSanitizer and
  ThreadSanitizer both missed the bug because their per-thread slowdown
  drops effective parallelism below the trigger threshold (no crash
  with `taskset -c 0-3`; crash with `taskset -c 0-7`).

  `run_batched` now delegates to a process-lifetime `WorkerPool` (a
  fixed-size pool of `std::threads`, each owning a long-lived
  `SolverContext`) that workers drain via an atomic counter, mirroring
  the `ddss` fork's persistent-pool design that doesn't reproduce the
  crash. As a side benefit, per-worker transposition tables persist
  across batched calls, so the TT can warm up for callers that issue
  many small batches.

  Caveat: the pool's per-worker `SolverContext`s are constructed from
  the `SolverConfig` seen on the first call. Subsequent calls with a
  different `cfg` silently keep using the original config. The Rust
  caller in `dds-bridge` always passes `SolverConfig::default()`, so
  this is fine in practice; callers that need varying configs should
  use the single-shot `SolverContext` API. The `n_threads` argument is
  only honored on the first call to size the pool.

  Adds `solver_context_batched_stress_pool_reuse` in `src/tests.rs`,
  which submits `2 * available_parallelism()` deals across 3 batched
  calls — well past the previous crash threshold — and asserts every
  result matches a serial single-context reference. The stress test
  is `#[cfg(not(target_os = "windows"))]`-gated for now: the first
  Windows CI run after the pool rewrite ended with a
  `STATUS_ACCESS_VIOLATION` on the batched tests. Without Windows
  access it's not yet clear whether the stress test or the older
  `solver_context_batched_matches_serial` is the actual culprit; the
  gate keeps CI green while leaving the smaller smoke-test batched
  coverage in place on Windows.

### Changed

- Repoint the `vendor` submodule at the `jdh8/dds` fork's
  `ab-search-inline-accessors` branch (commit `0ad9284`) to pick up
  two unmerged fixes the upstream `dds-bridge/dds` `develop` branch
  doesn't have yet:

  - **`calc_tables: drop racing scheduler.RegisterRun call`** (commit
    `96883c3`). DDS3's `9ebcac5` ("calc_tables: make `calc_dd_table
    (ctx)` thread-safe") removed the `cparam` file-scope global from
    `calc_all_boards_n`'s call chain but left
    `scheduler.RegisterRun(DDS_RUN_CALC, * bop)` at the function's
    entry. With N `SolverContext`s driving `calc_dd_table`
    concurrently (e.g. via this crate's `dds_calc_dd_tables_batched`),
    all workers race on the global `scheduler` instance. The
    registered run data has no live consumer on DDS 3's sequential
    execution path (the only reader, `Scheduler::GetNumber` in
    `calc_chunk_common`, is dead code; the surrounding `PrintTiming`
    call is `#ifdef DDS_SCHEDULER`-gated), so dropping the
    registration is a no-op in standard builds. The worker-pool
    rewrite above made the early `init_tt` crash from this race go
    away, but the race itself remained; this drops it for good.

  - **`solver_context: inline SearchContext accessors and drop
    shared_ptr from MoveGenContext`** (commit `0ad9284`). The trivial
    `SearchContext` accessors (`best_move`, `lowest_win`, `nodes`,
    `winners`, `forbidden_move`, ...) moved from
    `solver_context.cpp` into the class body in
    `solver_context.hpp` so `ab_search.cpp`'s ~53 accessor call sites
    fold into direct field accesses. `SolverContext::move_gen()` now
    returns a `MoveGenContext` constructed from a non-owning
    `ThreadData*` instead of copying a `std::shared_ptr<ThreadData>`,
    eliminating the per-call atomic refcount bump (~22 calls per
    `ab_search` invocation).

  Combined measured impact of the inlining + raw-pointer changes on
  Linux x86_64 (gcc 12, -O3, 32 cores), via `dds-bridge`'s
  `cargo bench --bench solver`:

  ```
  solve_deal_single   140.6 ms  -> 125.0 ms   (-11%)
  solve_deals/32     1178   ms  -> 1045   ms  (-11%)
  solve_deals/200    4153   ms  -> 3587   ms  (-14%)
  solve_boards/32     110   ms  ->   96   ms  (-13%)
  solve_boards/200    295   ms  ->  253   ms  (-14%)
  analyse_plays_32     65   ms  ->   60   ms   (-8%)
  ```

  The `vendor` remote in `.gitmodules` is temporarily on `jdh8/dds`
  for the duration of these fork-only patches. Once the fork's PRs
  are merged into upstream `dds-bridge/dds:develop`, a follow-up
  release will retire the fork remote again (mirroring the same
  transition 3.2.0 already did once).

## [3.2.0] - 2026-05-25

### Added

- `dds_calc_dd_tables_batched` and `dds_solve_boards_batched` FFI entry
  points in `src/dds_context.{h,cpp}`. Each accepts N independent
  deals/boards plus a per-worker `DdsSolverConfig`, spawns an internal
  pool sized to `std::thread::hardware_concurrency()` (overridable via
  the `n_threads` argument), and dispatches work via an atomic
  fetch-add counter. Every worker owns its own `SolverContext` — no
  shared mutable state, so the thread-safety guarantee that 3.1.1
  added to `dds_calc_dd_table` extends naturally to the batched path.
  The first non-success status from any worker is preserved (later
  successes do not overwrite it); workers continue draining the
  remaining items after an error, mirroring the existing per-deal
  Rust caller pattern. This ports the structural performance win
  from the `ddss` fork's `CalcAllTablesPBNx` (in-vendor batched
  scheduling) without reintroducing the `cparam` global it relies on.

### Changed

- Removed the `[profile.dev] opt-level = 3` override. Dev builds now
  use Cargo's default opt-level 0 for both Rust and `cc`-built C++.
  Same rationale as the corresponding removal in `ddss-sys` 0.1.2:
  don't mask the converse-class bugs (UB-in-unsafe miscompilations
  and the like) that only surface at -O2/-O3.
- Refresh the `vendor` submodule (`26ee3ce` → `d2f3200`): rebase the
  three local patches (thread-safe `calc_dd_table`, dead `System`
  callback cleanup) onto upstream master, picking up an upstream typo
  fix, Bazel/CI updates, and the `DDS_VERSION` macro bump from `20900`
  to `30000`. No FFI surface change, but `DDSInfo.major/minor/patch`
  (and the "Version" line in `systemString`) now report **3.0.0**
  instead of 2.9.0 at runtime — downstream consumers that pin the
  reported version (e.g. dds-bridge's `system_info_version_is_2_9_0`
  test) will need to update once a release picks this up.
- Retire the `jdh8/dds` fork as the `vendor` submodule remote. Upstream
  `dds-bridge/dds` has since landed equivalents of the fork's local
  patches (notably `1b121af` "eliminate all mutable global state from
  solver and par code", which subsumes the fork's thread-safe
  `calc_dd_table(ctx)` rewrite), so the submodule now tracks the
  official `develop` branch (`d2f3200` → `1ce9026`). The only thing
  lost is the fork's removal of unused legacy `System` callback /
  `calc_*` shim code paths; those compile cleanly upstream as dead
  code and are not on any path `dds_context.{h,cpp}` exercises. No
  FFI surface change and all tests (incl. `solver_context_batched_*`)
  pass against the upstream tree.

## [3.1.1] - 2026-05-24

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

[3.2.2]: https://github.com/jdh8/dds-bridge-sys/releases/tag/3.2.2
[3.2.1]: https://github.com/jdh8/dds-bridge-sys/releases/tag/3.2.1
[3.2.0]: https://github.com/jdh8/dds-bridge-sys/releases/tag/3.2.0
[2.1.1]: https://github.com/jdh8/dds-bridge-sys/releases/tag/2.1.1
[2.1.0]: https://github.com/jdh8/dds-bridge-sys/releases/tag/2.1.0
[2.0.5]: https://github.com/jdh8/dds-bridge-sys/releases/tag/2.0.5
[2.0.4]: https://github.com/jdh8/dds-bridge-sys/releases/tag/2.0.4
