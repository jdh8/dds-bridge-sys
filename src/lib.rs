//! Generated bindings to [dds-bridge/dds](https://github.com/dds-bridge/dds),
//! the C++ double dummy solver for contract bridge.
//!
//! This library needs manual initialization!  Initialize the thread pool with
//! [`SetMaxThreads`] before calling other functions.
//!
//! ```
//! use dds_bridge_sys as dds;
//! // 0 stands for automatic configuration
//! unsafe { dds::SetMaxThreads(0) };
//! ```
//!
//! Also note that functions using the thread pool are not reentrant.  You may
//! want to use a mutex to ensure that only one thread is using the thread
//! pool.
#![cfg_attr(not(test), no_std)]
#![allow(non_upper_case_globals, non_camel_case_types, non_snake_case)]
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[cfg(test)]
mod test;
