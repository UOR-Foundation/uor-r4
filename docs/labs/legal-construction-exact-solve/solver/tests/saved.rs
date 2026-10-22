//! The saved-case loader: exact load of a synthetic case, refusal of a
//! tampered file and of an out-of-range master, and a smoke run on the real
//! case (ignored by default).

use std::fs;
use std::path::{Path, PathBuf};

use legal_exact_solver::exact::f32_to_rational;
use legal_exact_solver::saved::{self, family_file, margin_file, LoadError, FAMILIES};
use num_rational::BigRational;
use serde_json::json;
use sha2::{Digest, Sha256};

fn hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes).iter().map(|b| format!("{b:02x}")).collect()
}

fn le(values: &[f32]) -> Vec<u8> {
    values.iter().flat_map(|v| v.to_bits().to_le_bytes()).collect()
}

fn exact(values: &[f32]) -> Vec<BigRational> {
    values.iter().map(|v| f32_to_rational(v.to_bits()).unwrap()).collect()
}

const PREFIX_M: [f32; 2] = [0.25, 0.3];
const GEN_M: [f32; 1] = [-1.75];
const PREFIX_G: [f32; 2] = [1.0, -0.5];
const GEN_G: [f32; 1] = [3.0e-5];
const GUARDS: [[f32; 3]; 2] = [[1.0, 0.0, -0.375], [0.0, 0.0, 0.0]];

/// Write a 3-coefficient, 2-guard case with its two receipts.
fn write_case(name: &str, gen_m: &[f32]) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("saved-{name}"));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    let mut files = Vec::new();
    for (family, m, g) in [(FAMILIES[0], &PREFIX_M[..], &PREFIX_G[..]), (FAMILIES[1], gen_m, &GEN_G[..])] {
        for (kind, vals) in [("initial-master", m), ("gradient", g)] {
            let file = family_file(family, kind);
            let bytes = le(vals);
            files.push(json!({"file": file, "kind": kind, "sha256": hex(&bytes)}));
            fs::write(dir.join(&file), bytes).unwrap();
        }
    }
    fs::write(dir.join(saved::GRADIENT_RECEIPT), json!({"files": files}).to_string()).unwrap();
    let mut terms = Vec::new();
    for (i, row) in GUARDS.iter().enumerate() {
        let file = margin_file(i);
        let bytes = le(row);
        terms.push(json!({"file": file, "guard_index": i, "sha256": hex(&bytes)}));
        fs::write(dir.join(&file), bytes).unwrap();
    }
    fs::write(dir.join(saved::MARGIN_RECEIPT), json!({"terms": terms}).to_string()).unwrap();
    dir
}

#[test]
fn synthetic_case_loads_exactly() {
    let dir = write_case("ok", &GEN_M);
    let (p, verified) = saved::load(&dir).unwrap();
    assert_eq!(p.masters, exact(&[0.25, 0.3, -1.75]));
    assert_eq!(p.g, exact(&[1.0, -0.5, 3.0e-5]));
    assert_eq!(p.guards, vec![exact(&GUARDS[0]), exact(&GUARDS[1])]);
    assert_eq!(verified.len(), 6);
    let q0: Vec<i64> = p.coeffs().iter().map(|c| c.q0).collect();
    assert_eq!(q0, vec![1, 1, -7]);
    let frac: Vec<bool> = p.coeffs().iter().map(|c| c.fractional).collect();
    assert_eq!(frac, vec![false, true, false]);
}

#[test]
fn one_changed_byte_is_refused_naming_the_file() {
    let dir = write_case("tampered", &GEN_M);
    let file = margin_file(1);
    let path = dir.join(&file);
    let mut bytes = fs::read(&path).unwrap();
    let before = hex(&bytes);
    bytes[5] ^= 0x01;
    let after = hex(&bytes);
    fs::write(&path, bytes).unwrap();
    let err = saved::load(&dir).unwrap_err();
    match &err {
        LoadError::HashMismatch { file: f, expected, actual } => {
            assert_eq!(f, &file);
            assert_eq!(expected, &before);
            assert_eq!(actual, &after);
        }
        other => panic!("expected a hash mismatch, got {other}"),
    }
    let msg = err.to_string();
    assert!(msg.contains(&file) && msg.contains(&before) && msg.contains(&after), "{msg}");
}

#[test]
fn master_outside_range_is_refused() {
    let dir = write_case("range", &[1.8]);
    match saved::load(&dir).unwrap_err() {
        LoadError::MasterOutOfRange { index, file, .. } => {
            assert_eq!(index, 2);
            assert_eq!(file, family_file(FAMILIES[1], "initial-master"));
        }
        other => panic!("expected an out-of-range refusal, got {other}"),
    }
}

#[test]
#[ignore = "reads the real saved case from the owner's local disk"]
fn real_case_smoke() {
    let dir = Path::new(
        "/Users/casey.allard/uor-r4/local/claude/legal-case/codex-direct-legal-centered-20261009/attempt2",
    );
    let (p, verified) = saved::load(dir).unwrap();
    assert_eq!(p.n(), 1920);
    assert_eq!(p.coeffs().iter().filter(|c| c.fractional).count(), 20);
    assert_eq!(p.guards.len(), 380);
    assert_eq!(verified.len(), 384);
}
