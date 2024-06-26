dds-bridge-sys
==============
![Build Status](https://github.com/jdh8/dds-bridge-sys/actions/workflows/rust.yml/badge.svg)
[![Crates.io](https://img.shields.io/crates/v/dds-bridge-sys.svg)](https://crates.io/crates/dds-bridge-sys)
[![Docs.rs](https://docs.rs/dds-bridge-sys/badge.svg)](https://docs.rs/dds-bridge-sys)

Generated bindings to [dds-bridge/dds](https://github.com/dds-bridge/dds),
the C++ double dummy solver for contract bridge.

## Usage
Initialize the thread pool before calling other library functions:
```rust
use dds_bridge_sys as dds;
// 0 stands for automatic configuration
unsafe { dds::SetMaxThreads(0) };
```

## Supported parallel backends
This crate supports the following parallel backends (in order of precedence by DDS):
1. [Windows API](https://en.wikipedia.org/wiki/Windows_API)
2. [OpenMP](https://en.wikipedia.org/wiki/OpenMP)
3. [Grand Central Dispatch](https://en.wikipedia.org/wiki/Grand_Central_Dispatch) (on macOS and iOS)
4. C++ [`std::thread`](https://en.cppreference.com/w/cpp/thread/thread)

## Features
The default `openmp` feature automatically uses OpenMP if the C++ compiler
supports it.  If this causes issues, you can disable this feature with
`--no-default-features`.
