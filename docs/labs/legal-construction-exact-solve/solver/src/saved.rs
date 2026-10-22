//! Exact loader for a saved legal-construction case (#2828).
//!
//! The case directory holds little-endian binary32 arrays. Each file is read
//! as raw bytes, its sha256 is checked against the case's receipts, and only
//! then are the bytes read as `u32` words and converted bit-exactly through
//! `exact`. The mapping:
//!
//! - masters `m`: the prefix-coefficients initial master, then the
//!   generate-unary initial master (the prefix family comes first);
//! - objective `g`: the two matching gradient files, in the same order;
//! - guards: every row `protected-margin-NNN` listed in
//!   `protected-margin-receipt.json`, raw and unnormalised (`J·δ >= 0`).
//!
//! A master outside `[-7/4, 7/4]` is refused, so `round_ties_away(4m)` lies
//! in `[-7, 7]` and needs no clamp.

use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use num_bigint::BigInt;
use num_rational::BigRational;
use num_traits::Signed;
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::exact::{self, ExactError, BINARY32_LE_SUFFIX};
use crate::problem::Problem;

/// Receipt naming the master and gradient files.
pub const GRADIENT_RECEIPT: &str = "coupled-gradient-receipt.json";
/// Receipt naming the guard rows.
pub const MARGIN_RECEIPT: &str = "protected-margin-receipt.json";
/// Family stems, in coefficient order.
pub const FAMILIES: [&str; 2] = ["coupled-prefix.coefficients", "coupled-generate.unary"];

/// One file whose bytes matched its receipt's sha256.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiedHash {
    pub file: String,
    pub sha256: String,
}

#[derive(Debug)]
pub enum LoadError {
    Io { file: String, source: std::io::Error },
    Receipt { file: String, msg: String },
    HashMismatch { file: String, expected: String, actual: String },
    Length { file: String, msg: String },
    NotFinite { file: String, source: ExactError },
    MasterOutOfRange { index: usize, file: String, value: BigRational },
}

impl fmt::Display for LoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LoadError::Io { file, source } => write!(f, "cannot read {file}: {source}"),
            LoadError::Receipt { file, msg } => write!(f, "receipt {file}: {msg}"),
            LoadError::HashMismatch { file, expected, actual } => write!(
                f,
                "sha256 mismatch for {file}: receipt says {expected}, file hashes to {actual}"
            ),
            LoadError::Length { file, msg } => write!(f, "{file}: {msg}"),
            LoadError::NotFinite { file, source } => write!(f, "{file}: {source}"),
            LoadError::MasterOutOfRange { index, file, value } => write!(
                f,
                "master {index} (from {file}) is {value}, outside [-7/4, 7/4]; refused"
            ),
        }
    }
}

impl std::error::Error for LoadError {}

pub type Result<T> = std::result::Result<T, LoadError>;

/// File name of guard row `i`.
pub fn margin_file(i: usize) -> String {
    format!("protected-margin-{i:03}{BINARY32_LE_SUFFIX}")
}

/// File name of a family's initial master or gradient.
pub fn family_file(family: &str, kind: &str) -> String {
    format!("{family}-{kind}{BINARY32_LE_SUFFIX}")
}

fn read(dir: &Path, file: &str) -> Result<Vec<u8>> {
    let path: PathBuf = dir.join(file);
    fs::read(&path).map_err(|source| LoadError::Io { file: file.to_string(), source })
}

fn read_json(dir: &Path, file: &str) -> Result<Value> {
    let bytes = read(dir, file)?;
    serde_json::from_slice(&bytes)
        .map_err(|e| LoadError::Receipt { file: file.to_string(), msg: e.to_string() })
}

fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes).iter().map(|b| format!("{b:02x}")).collect()
}

/// Read a file and check its sha256 against the receipt's value.
fn read_verified(dir: &Path, file: &str, expected: &str, out: &mut Vec<VerifiedHash>) -> Result<Vec<u8>> {
    let bytes = read(dir, file)?;
    let actual = sha256_hex(&bytes);
    if actual != expected {
        return Err(LoadError::HashMismatch {
            file: file.to_string(),
            expected: expected.to_string(),
            actual,
        });
    }
    out.push(VerifiedHash { file: file.to_string(), sha256: actual });
    Ok(bytes)
}

fn words(file: &str, bytes: &[u8]) -> Result<Vec<u32>> {
    if !bytes.len().is_multiple_of(4) {
        return Err(LoadError::Length {
            file: file.to_string(),
            msg: format!("{} bytes is not a whole number of 4-byte words", bytes.len()),
        });
    }
    Ok(bytes.chunks_exact(4).map(|c| u32::from_le_bytes([c[0], c[1], c[2], c[3]])).collect())
}

fn rationals(file: &str, bytes: &[u8]) -> Result<Vec<BigRational>> {
    let w = words(file, bytes)?;
    exact::rationals_from_bits(&w).map_err(|source| LoadError::NotFinite { file: file.to_string(), source })
}

fn str_field<'a>(v: &'a Value, key: &str, receipt: &str, ctx: &str) -> Result<&'a str> {
    v.get(key).and_then(Value::as_str).ok_or_else(|| LoadError::Receipt {
        file: receipt.to_string(),
        msg: format!("{ctx}: missing string field `{key}`"),
    })
}

/// The receipt sha256 for `file` in `coupled-gradient-receipt.json` `files[]`.
fn gradient_hash(receipt: &Value, file: &str) -> Result<String> {
    let entries = receipt.get("files").and_then(Value::as_array).ok_or_else(|| LoadError::Receipt {
        file: GRADIENT_RECEIPT.to_string(),
        msg: "missing array `files`".to_string(),
    })?;
    let mut found = None;
    for e in entries {
        if e.get("file").and_then(Value::as_str) == Some(file) {
            if found.is_some() {
                return Err(LoadError::Receipt {
                    file: GRADIENT_RECEIPT.to_string(),
                    msg: format!("{file} is listed more than once"),
                });
            }
            found = Some(str_field(e, "sha256", GRADIENT_RECEIPT, file)?.to_string());
        }
    }
    found.ok_or_else(|| LoadError::Receipt {
        file: GRADIENT_RECEIPT.to_string(),
        msg: format!("{file} is not listed in `files`"),
    })
}

/// Load the saved case in `dir`: verify every sha256, then convert exactly.
pub fn load(dir: &Path) -> Result<(Problem, Vec<VerifiedHash>)> {
    let mut verified = Vec::new();

    // 1. Read every file and check its hash; nothing is converted yet.
    let grad_receipt = read_json(dir, GRADIENT_RECEIPT)?;
    let mut family_bytes = Vec::new();
    for family in FAMILIES {
        let mut pair = Vec::new();
        for kind in ["initial-master", "gradient"] {
            let file = family_file(family, kind);
            let expected = gradient_hash(&grad_receipt, &file)?;
            let bytes = read_verified(dir, &file, &expected, &mut verified)?;
            pair.push((file, bytes));
        }
        family_bytes.push(pair);
    }

    let margin_receipt = read_json(dir, MARGIN_RECEIPT)?;
    let terms = margin_receipt.get("terms").and_then(Value::as_array).ok_or_else(|| LoadError::Receipt {
        file: MARGIN_RECEIPT.to_string(),
        msg: "missing array `terms`".to_string(),
    })?;
    let mut guard_bytes = Vec::with_capacity(terms.len());
    for (i, term) in terms.iter().enumerate() {
        let file = margin_file(i);
        let ctx = format!("terms[{i}]");
        let listed = str_field(term, "file", MARGIN_RECEIPT, &ctx)?;
        if listed != file {
            return Err(LoadError::Receipt {
                file: MARGIN_RECEIPT.to_string(),
                msg: format!("{ctx} names {listed}, expected {file}"),
            });
        }
        let expected = str_field(term, "sha256", MARGIN_RECEIPT, &ctx)?;
        let bytes = read_verified(dir, &file, expected, &mut verified)?;
        guard_bytes.push((file, bytes));
    }

    // 2. Convert, bit-exactly, through `exact`.
    let mut masters = Vec::new();
    let mut master_files = Vec::new();
    let mut g = Vec::new();
    for pair in &family_bytes {
        let (mfile, mbytes) = &pair[0];
        let (gfile, gbytes) = &pair[1];
        let m = rationals(mfile, mbytes)?;
        let gv = rationals(gfile, gbytes)?;
        if m.len() != gv.len() {
            return Err(LoadError::Length {
                file: gfile.clone(),
                msg: format!("{} words, but {mfile} has {}", gv.len(), m.len()),
            });
        }
        master_files.extend(std::iter::repeat_n(mfile.clone(), m.len()));
        masters.extend(m);
        g.extend(gv);
    }
    let n = masters.len();
    let mut guards = Vec::with_capacity(guard_bytes.len());
    for (file, bytes) in &guard_bytes {
        let row = rationals(file, bytes)?;
        if row.len() != n {
            return Err(LoadError::Length {
                file: file.clone(),
                msg: format!("{} words, expected N = {n}", row.len()),
            });
        }
        guards.push(row);
    }

    // 3. Refuse a master outside [-7/4, 7/4], as the original does.
    let bound = BigRational::new(BigInt::from(7), BigInt::from(4));
    for (index, m) in masters.iter().enumerate() {
        if m.abs() > bound {
            return Err(LoadError::MasterOutOfRange {
                index,
                file: master_files[index].clone(),
                value: m.clone(),
            });
        }
    }

    Ok((Problem::new(masters, g, guards), verified))
}
