//! Serving-only build of the question reader the discrete crate trains.
//!
//! `conjunction.rs` and the files under `conjunction/` are compiled twice
//! from one source: as `uor_r4_discrete::conjunction`, where TRAIN writes the
//! rows, and here, with no GPU build pipeline, for the integer client and
//! WebAssembly. The operators a reply uses (`Asked::place`, `Asked::answer`,
//! the history address, the saved-file reader) are therefore the same
//! definitions in both. This file is not a module of the discrete crate.
#![forbid(unsafe_code)]

/// The one error the reader raises: a refused input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    Invalid(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Invalid(message) => write!(f, "invalid reader: {message}"),
        }
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;

pub mod conjunction;
