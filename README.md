# dds-bridge-sys

[![Build Status](https://github.com/jdh8/dds-bridge-sys/actions/workflows/rust.yml/badge.svg)](https://github.com/jdh8/dds-bridge-sys)
[![Crates.io](https://img.shields.io/crates/v/dds-bridge-sys.svg)](https://crates.io/crates/dds-bridge-sys)
[![Docs.rs](https://docs.rs/dds-bridge-sys/badge.svg)](https://docs.rs/dds-bridge-sys)

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

Functions fall into four categories:

### Non-reentrant — exclusive thread-pool access required

These functions drive the internal thread pool and must not be called
concurrently.  Protect all calls with a single mutex:

- [`CalcDDtable`] / [`CalcDDtablePBN`] — compute a full DD table (threads per strain)
- [`CalcAllTables`] / [`CalcAllTablesPBN`] — batch DD tables
- [`SolveAllBoards`] / [`SolveAllBoardsBin`] — batch board solving
- [`SolveAllChunksBin`] / [`SolveAllChunksPBN`] (deprecated aliases)
- [`AnalyseAllPlaysBin`] / [`AnalyseAllPlaysPBN`] — batch play analysis

### Reentrant — explicit `threadIndex` parameter

These are safe to call from multiple threads simultaneously.  Each concurrent
call must use a distinct `threadIndex` in `0..SetMaxThreads(0)`.

- [`SolveBoard`] / [`SolveBoardPBN`]
- [`AnalysePlayBin`] / [`AnalysePlayPBN`]

### Always safe — no thread pool involved

- Par: [`Par`], [`CalcPar`], [`CalcParPBN`], [`SidesPar`], [`DealerPar`],
  [`DealerParBin`], [`SidesParBin`]
- Text: [`ConvertToDealerTextFormat`], [`ConvertToSidesTextFormat`]
- Utilities: [`GetDDSInfo`], [`ErrorMessage`]

### Thread-pool management

Call one of the following once at startup before any other function:

- [`SetMaxThreads`] — initialize (`0` = auto-configure)
- [`SetResources`] — set memory and thread limits

## Cargo features

All features are off by default. The `debug-*` diagnostic features cause the
C++ solver to write per-thread `.txt` files into the current working directory
at runtime; they are intended for solver development, not production use.

- `debug-dump` — let DDS write `dump.txt` on solver errors
- `debug-timing` — function timings → `timer.*.txt`
- `debug-ab-stats` — alpha-beta search stats → `ABstats.*.txt`
- `debug-tt-stats` — transposition-table memory usage → `TTstats.*.txt`
- `debug-moves` — move-generation quality → `movestats.*.txt`
