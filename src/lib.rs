#![feature(trace_macros)]

extern crate bytes;
extern crate chrono;
#[macro_use]
extern crate nom;
extern crate uuid;
extern crate ordered_float;
#[macro_use]
extern crate error_chain;

pub mod codec;
pub mod framing;
pub mod types;
mod errors;
pub use errors::*; // todo: revisit API guidelines for this