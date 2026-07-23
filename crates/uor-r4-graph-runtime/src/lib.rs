#![no_std]
#![forbid(unsafe_code)]

extern crate alloc;

pub mod cayley_dickson;
pub mod engine;
pub mod patch_chain;
pub mod routing;
pub mod runtime_state;
pub mod status;

pub use engine::{R4G1Runtime, RuntimeError};
pub use status::ResolutionStatus;
