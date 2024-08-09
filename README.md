dds-bridge-sys
==============
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
