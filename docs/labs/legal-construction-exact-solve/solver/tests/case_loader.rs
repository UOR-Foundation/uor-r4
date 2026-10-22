//! Case-loader tests. Binary32 values are written by the test harness only, so
//! the crate's `no_floats` scan (src/*.rs) still sees binary floating point only in
//! `exact.rs`: these tests live outside `src/`.

use std::path::{Path, PathBuf};

use legal_exact_solver::bb::Assignment;
use legal_exact_solver::case::{self, FAMILIES, GUARDS};
use legal_exact_solver::check;

fn tiny_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("uor-legal-case-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn write_bits(dir: &Path, name: &str, values: &[f32]) {
    let bytes: Vec<u8> = values
        .iter()
        .flat_map(|v| v.to_bits().to_le_bytes())
        .collect();
    std::fs::write(dir.join(name), bytes).unwrap();
}

fn tiny_case(dir: &Path, guards: usize, row_len: usize) {
    for (i, (stem, count)) in FAMILIES.iter().enumerate() {
        let base = if i == 0 { 0.25 } else { -0.5 };
        let m: Vec<f32> = (0..*count).map(|j| base + 0.25 * (j % 3) as f32).collect();
        let g: Vec<f32> = (0..*count)
            .map(|j| 0.125 * ((j % 5) as f32 - 2.0))
            .collect();
        write_bits(dir, &format!("coupled-{stem}-initial-master.f32le"), &m);
        write_bits(dir, &format!("coupled-{stem}-gradient.f32le"), &g);
    }
    for i in 0..guards {
        let row: Vec<f32> = (0..row_len).map(|j| ((i + j) % 7) as f32 * 0.25).collect();
        write_bits(dir, &format!("protected-margin-{i:03}.f32le"), &row);
    }
}

#[test]
fn loads_the_expected_shape_and_counts_fractional() {
    let dir = tiny_dir("shape");
    tiny_case(&dir, GUARDS, 1920);
    let case = case::load(&dir).unwrap();
    assert_eq!(case.problem.n(), 1920);
    assert_eq!(case.guard_rows, GUARDS);
    assert_eq!(
        case.fractional, 0,
        "the tiny fixture masters are all on the quarter grid"
    );
    let report = check::check(&case.problem, &Assignment::zero(1920));
    assert!(report.legal, "zero assignment is legal");
    assert!(report.guards_hold, "all-zero guard values hold");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn refuses_a_nan_value() {
    let dir = tiny_dir("nan");
    tiny_case(&dir, GUARDS, 1920);
    write_bits(&dir, "protected-margin-007.f32le", &vec![f32::NAN; 1920]);
    let err = case::load(&dir).unwrap_err();
    assert!(err.contains("NaN or infinite"), "{err}");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn refuses_a_wrong_guard_row_length() {
    let dir = tiny_dir("rowlen");
    tiny_case(&dir, GUARDS, 1920);
    write_bits(&dir, "protected-margin-123.f32le", &vec![1.0; 1919]);
    let err = case::load(&dir).unwrap_err();
    assert!(err.contains("1919 coefficients, expected 1920"), "{err}");
    let _ = std::fs::remove_dir_all(&dir);
}
