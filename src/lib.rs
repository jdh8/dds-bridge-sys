#![doc = include_str!("../README.md")]
#![cfg_attr(not(test), no_std)]
#![allow(non_upper_case_globals, non_camel_case_types, non_snake_case)]
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[cfg(test)]
mod test;
