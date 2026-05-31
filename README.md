# dds-bridge-sys

[![Crates.io](https://img.shields.io/crates/v/dds-bridge-sys.svg)](https://crates.io/crates/dds-bridge-sys)
[![Docs.rs](https://docs.rs/dds-bridge-sys/badge.svg)](https://docs.rs/dds-bridge-sys)
[![Build Status](https://github.com/jdh8/dds-bridge-sys/actions/workflows/rust.yml/badge.svg)](https://github.com/jdh8/dds-bridge-sys)

Generated bindings to [dds-bridge/dds](https://github.com/dds-bridge/dds),
the C++ double dummy solver for contract bridge.

## Usage

This library needs manual initialization! Initialize the thread pool with
[`SetMaxThreads`](https://docs.rs/dds-bridge-sys/latest/dds_bridge_sys/fn.SetMaxThreads.html)
before calling other library functions:

```rust
use dds_bridge_sys as dds;
// 0 stands for automatic configuration
unsafe { dds::SetMaxThreads(0) };
```

Also note that functions using the thread pool are not reentrant.  You may want
to use a mutex to ensure that only one thread is using the thread pool.

For a higher-level safe wrapper, see the [`dds-bridge`](https://crates.io/crates/dds-bridge)
crate, whose [`Solver`](https://docs.rs/dds-bridge/latest/dds_bridge/solver/struct.Solver.html)
type manages the mutex for you.

## Thread Safety

DDS 3 removed the internal thread pool: `SetMaxThreads(N>1)` is clamped to 1,
legacy batch entry points run sequentially, and the only path to parallelism
is the new per-context API exposed here as `DdsSolverContext`. Functions fall
into four categories.

### Modern `SolverContext` API — one context per OS thread

Each `DdsSolverContext` owns private solver state (thread-local memory,
transposition table, search state). Multiple contexts on multiple threads is
the upstream-recommended parallelism model — give each worker its own context,
never share one.

- [`dds_solver_context_new`] / [`dds_solver_context_free`] — lifecycle
- [`dds_solve_board`] — single-board solve
- [`dds_calc_dd_table`] / [`dds_calc_dd_table_pbn`] — full DD table
- [`dds_calc_par`] / [`dds_calc_par_from_table`] — par calculation
- [`dds_solver_context_reset_for_solve`] / [`dds_solver_context_clear_tt`]
  / [`dds_solver_context_resize_tt`] / [`dds_solver_context_configure_tt`]
  / [`dds_solver_context_dispose_trans_table`] — TT lifecycle

### Sequential inside DDS — legacy batch APIs

These functions used to drive the internal thread pool. As of DDS 3 they
execute sequentially in a single internal thread regardless of
`SetMaxThreads(N)`. Concurrent callers must drive parallelism from the
application side (one `DdsSolverContext` per worker, in preference to these).

- [`CalcDDtable`] / [`CalcDDtablePBN`]
- [`CalcAllTables`] / [`CalcAllTablesPBN`]
- [`SolveAllBoards`] / [`SolveAllBoardsBin`]
- [`SolveAllChunksBin`] / [`SolveAllChunksPBN`] (deprecated aliases)
- [`AnalyseAllPlaysBin`] / [`AnalyseAllPlaysPBN`]

### Legacy single-board APIs — `threadIndex=0` only

These accept a `threadIndex` parameter, but DDS 3 only allocates one internal
slot; any `threadIndex>0` returns `RETURN_THREAD_INDEX`. Each call internally
constructs a fresh `SolverContext`, so TT state is not retained across calls.
Concurrent calls from multiple threads are safe (each gets its own internal
context) but pay full setup cost per call; prefer the modern API above for
hot paths.

- [`SolveBoard`] / [`SolveBoardPBN`]
- [`AnalysePlayBin`] / [`AnalysePlayPBN`]

### Always safe — no solver state involved

- Par: [`Par`], [`CalcPar`], [`CalcParPBN`], [`SidesPar`], [`DealerPar`],
  [`DealerParBin`], [`SidesParBin`]
- Text: [`ConvertToDealerTextFormat`], [`ConvertToSidesTextFormat`]
- Utilities: [`GetDDSInfo`], [`ErrorMessage`]

### Thread-pool management (deprecated upstream, retained for compatibility)

- [`SetMaxThreads`] — initialize (`0` = auto-configure)
- [`SetResources`] — set memory and thread limits

Both functions still need to be called once at startup before any legacy entry
point. They are no-ops for the modern `DdsSolverContext` path, which manages
its own resources per instance.

## Cargo features

- `debug-dump` (off by default) — let DDS write `dump.txt` to the current
  working directory on solver errors. Intended for solver development.
