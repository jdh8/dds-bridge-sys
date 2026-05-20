#![doc = include_str!("../README.md")]
#![cfg_attr(not(test), no_std)]
#![allow(non_snake_case)]
#![allow(clippy::all, clippy::pedantic)]
#![allow(rustdoc::broken_intra_doc_links)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[cfg(test)]
mod tests;
