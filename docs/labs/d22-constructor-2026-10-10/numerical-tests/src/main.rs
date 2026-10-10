//! Saved constructor with an explicitly patched, residual-qualified local LU backend.
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    error::Error,
    fs,
    io::BufReader,
    path::{Path, PathBuf},
    time::Instant,
};
#[path = "constructor.rs"]
mod constructor;
#[path = "report_output.rs"]
mod report_output;
const SOURCE: &[u8] = include_bytes!("constructor.rs");
const SOURCE_SHA: &str = "f0242836d08dbadbb5659cd8f158d79c8de9420c9de2fb213de9ef149510c24b";
const BASIS_PINS: [&str; 4] = [
    "c5c73e1b9f7baf598cdda33e4a458f9208f0c943b4d471ba646eb129c495a794",
    "55d8c2eadbcfc072d86a6310602184603030a318986ebeb2a2d885958f10ae05",
    "337e0917af3a3c397ce70db1d096f24adc07c054de32a5cf6e1ed6c23491e221",
    "4b5abc63cc98cde36c8ff1ba47f1e9388548f81dad3cb26bf15e73bfb2fbd2b6",
];
const REPAIR: &[u8] =
    include_bytes!("../../../../../crates/uor-r4-training/vendor/microlp/src/repair.rs");
const LU: &[u8] = include_bytes!("../../../../../crates/uor-r4-training/vendor/microlp/src/lu.rs");
const SOLVER: &[u8] =
    include_bytes!("../../../../../crates/uor-r4-training/vendor/microlp/src/solver.rs");
fn stats_json() -> serde_json::Value {
    let s = microlp::repair::stats();
    serde_json::json!({"fresh_factors":s.fresh_factors,"exact_fallbacks":s.exact_fallbacks,"exact_singular":s.exact_singular,"certified_factors":s.certified_factors,"certified_solves":s.certified_solves,"rejected_solves":s.rejected_solves,"old_basis_refreshes":s.old_basis_refreshes})
}
fn source_json() -> serde_json::Value {
    serde_json::json!({"repair_sha256":hash(REPAIR),"lu_sha256":hash(LU),"solver_sha256":hash(SOLVER),"constructor_sha256":hash(SOURCE)})
}
fn basis_outcome_admits_replay(index: usize, r: &serde_json::Value) -> bool {
    let qualified = r["status"] == "QUALIFIED"
        && r["error"].is_null()
        && r["numerics"]["certified_factors"] == 1
        && r["numerics"]["certified_solves"] == 1760
        && r["numerics"]["exact_singular"] == 0;
    // A genuinely singular later capture cannot be soundly admitted as a
    // factorization. Its exact rejection qualifies the numerical boundary;
    // replay must still test whether transactional refresh/reselection works.
    let exact_singular = (2..4).contains(&index)
        && r["status"] == "NUMERICALLY_REJECTED"
        && r["error"] == "Singular matrix"
        && r["numerics"]["exact_fallbacks"] == 1
        && r["numerics"]["exact_singular"] == 1
        && r["numerics"]["certified_factors"] == 0
        && r["numerics"]["certified_solves"] == 0;
    index < 4 && (qualified || exact_singular)
}

#[cfg(test)]
mod replay_gate_tests {
    use super::basis_outcome_admits_replay;

    #[test]
    fn replay_gate_accepts_exact_later_rejection_only() {
        let mut r = serde_json::json!({"status":"NUMERICALLY_REJECTED",
            "error":"Singular matrix","numerics":{"exact_fallbacks":1,
            "exact_singular":1,"certified_factors":0,"certified_solves":0}});
        assert!(!basis_outcome_admits_replay(0, &r));
        assert!(!basis_outcome_admits_replay(1, &r));
        assert!(basis_outcome_admits_replay(2, &r));
        assert!(basis_outcome_admits_replay(3, &r));
        assert!(!basis_outcome_admits_replay(4, &r));
        r["numerics"]["exact_singular"] = 0.into();
        assert!(!basis_outcome_admits_replay(2, &r));
        r["numerics"]["exact_singular"] = 1.into();
        r["error"] = "Exact resource limit".into();
        assert!(!basis_outcome_admits_replay(2, &r));
    }

    #[test]
    fn replay_gate_requires_all_residual_checks() {
        let mut r = serde_json::json!({"status":"QUALIFIED","error":null,
            "numerics":{"certified_factors":1,"certified_solves":1760,
            "exact_singular":0}});
        for i in 0..4 {
            assert!(basis_outcome_admits_replay(i, &r));
        }
        r["numerics"]["certified_solves"] = 1759.into();
        assert!(!basis_outcome_admits_replay(0, &r));
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Input {
    schema: String,
    master_bits: Vec<u32>,
    gradient_bits: Vec<u32>,
    jacobian_bits: Vec<Vec<u32>>,
    provenance: serde_json::Value,
}
#[derive(Serialize)]
struct VariableMapping {
    backend_variable: usize,
    coordinate: usize,
    family: &'static str,
    family_index: usize,
    component: &'static str,
    lower: i32,
    upper: i32,
}
fn hash(b: &[u8]) -> String {
    format!("{:x}", Sha256::digest(b))
}
fn write(p: &Path, v: &impl Serialize) -> Result<(), Box<dyn Error>> {
    use std::io::Write;
    let mut f = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(p)?;
    serde_json::to_writer_pretty(&mut f, v)?;
    f.write_all(b"\n")?;
    Ok(())
}
fn unit_rows(j: &[Vec<f32>]) -> Result<Vec<Vec<f64>>, Box<dyn Error>> {
    j.iter()
        .map(|v| {
            if v.len() != 1920 || v.iter().any(|x| !x.is_finite()) {
                return Err("malformed J row".into());
            }
            let mut r = v.iter().map(|x| f64::from(*x)).collect::<Vec<_>>();
            let n = r.iter().map(|x| x * x).sum::<f64>().sqrt();
            if !n.is_finite() {
                return Err("J norm overflow".into());
            }
            if n > 0. {
                for x in &mut r {
                    *x /= n;
                }
            }
            Ok(r)
        })
        .collect()
}
fn mapping(m: &[f32]) -> (Vec<VariableMapping>, Vec<serde_json::Value>) {
    let mut vars = Vec::new();
    let mut rows = Vec::new();
    for (i, &m) in m.iter().enumerate() {
        let q0 = (m * 4.).round() as i32;
        let family = if i < 960 { "prefix" } else { "generate.unary" };
        let fi = i % 960;
        vars.push(VariableMapping {
            backend_variable: vars.len(),
            coordinate: i,
            family,
            family_index: fi,
            component: "k=q-q0",
            lower: -7 - q0,
            upper: 7 - q0,
        });
        if m != (q0 as f32) * 0.25 {
            for component in ["lower_branch_l", "upper_branch_r"] {
                vars.push(VariableMapping {
                    backend_variable: vars.len(),
                    coordinate: i,
                    family,
                    family_index: fi,
                    component,
                    lower: 0,
                    upper: 1,
                });
            }
            for kind in ["side_sum_le1", "centered_lower_ge0", "centered_upper_le0"] {
                rows.push(
                    serde_json::json!({"model_constraint":rows.len(),"kind":kind,"coordinate":i}),
                );
            }
        }
    }
    for i in 0..380 {
        rows.push(serde_json::json!({"model_constraint":rows.len(),"kind":"unit_guard_ge0","guard_index":i}));
    }
    (vars, rows)
}
fn execute(input: &Path, expected: &str, out: &Path) -> Result<serde_json::Value, Box<dyn Error>> {
    let started = Instant::now();
    if hash(SOURCE) != SOURCE_SHA {
        return Err("constructor source identity differs".into());
    }
    let meta = fs::metadata(input)?;
    if !meta.is_file() || meta.len() > 16 * 1024 * 1024 {
        return Err("input must be regular file <=16MiB".into());
    }
    let bytes = fs::read(input)?;
    if hash(&bytes) != expected {
        return Err("saved input SHA mismatch".into());
    }
    let x: Input = serde_json::from_reader(BufReader::new(bytes.as_slice()))?;
    if x.schema != "uor-r4.legal-basis-input/1"
        || x.master_bits.len() != 1920
        || x.gradient_bits.len() != 1920
        || x.jacobian_bits.len() != 380
    {
        return Err("input shape/schema differs".into());
    }
    let m = x
        .master_bits
        .iter()
        .map(|x| f32::from_bits(*x))
        .collect::<Vec<_>>();
    let g = x
        .gradient_bits
        .iter()
        .map(|x| f32::from_bits(*x))
        .collect::<Vec<_>>();
    let j = x
        .jacobian_bits
        .iter()
        .map(|r| r.iter().map(|x| f32::from_bits(*x)).collect::<Vec<_>>())
        .collect::<Vec<_>>();
    let rows = unit_rows(&j)?;
    let (variables, constraints) = mapping(&m);
    write(
        &out.join("model-mapping.json"),
        &serde_json::json!({"variables":variables,"constraints":constraints,"source_sha256":SOURCE_SHA,"internal_row_mapping":"model order is authoritative; backend context contains row scales and full internal basic/nonbasic/slack mapping; no assumption of identity without inspection"}),
    )?;
    microlp::repair::reset_stats();
    let receipt = constructor::run(&m, &g, &rows)?;
    constructor::authenticate(&m, &g, &rows, &receipt)?;

    write(&out.join("constructor-receipt.json"), &receipt)?;
    Ok(
        serde_json::json!({"status":"COMPLETED_SAVED_CONSTRUCTOR_REPLAY","schema":"uor-r4.d22-saved-constructor/1","input_sha256":expected,"producer_provenance":x.provenance,"source":source_json(),"numerics":stats_json(),"backend_status":receipt.status,"backend_termination":receipt.termination,"elapsed_seconds":started.elapsed().as_secs_f64(),"scope":"Historical saved input regression only; zero new gradients/native scoring/model acceptance; wrapper completion is not a returned assignment"}),
    )
}

fn execute_basis(
    input: &Path,
    expected: &str,
    out: &Path,
) -> Result<serde_json::Value, Box<dyn Error>> {
    let started = Instant::now();
    let bytes = fs::read(input)?;
    if hash(&bytes) != expected || bytes.len() > 16 * 1024 * 1024 {
        return Err("basis identity/size differs".into());
    }
    let text = std::str::from_utf8(&bytes)?;
    let mut columns: Vec<Vec<(usize, f64)>> = Vec::new();
    let mut sizes = Vec::new();
    for line in text.lines() {
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        let fields = line.split('\t').collect::<Vec<_>>();
        match fields.as_slice() {
            ["COLUMN", c, count] => {
                if c.parse::<usize>()? != columns.len() {
                    return Err("column order differs".into());
                }
                columns.push(Vec::new());
                sizes.push(count.parse::<usize>()?);
            }
            ["ENTRY", c, r, bits] => {
                let c = c.parse::<usize>()?;
                let r = r.parse::<usize>()?;
                if columns.is_empty() || c != columns.len() - 1 || r >= 440 {
                    return Err("entry coordinate differs".into());
                }
                let x = f64::from_bits(u64::from_str_radix(bits, 16)?);
                if !x.is_finite() || columns[c].iter().any(|(row, _)| *row == r) {
                    return Err("invalid or duplicate entry".into());
                }
                columns[c].push((r, x));
            }
            _ => return Err("unknown basis line".into()),
        }
    }
    if columns.len() != 440 || columns.iter().zip(sizes).any(|(c, n)| c.len() != n) {
        return Err("basis dimensions/counts differ".into());
    }
    microlp::repair::reset_stats();
    let result = microlp::repair::qualify(
        columns
            .iter()
            .map(|col| col.iter().copied().unzip())
            .collect(),
    );
    Ok(serde_json::json!({
        "schema":"uor-r4.d22-saved-basis-qualification/1",
        "status":if result.is_ok() {"QUALIFIED"} else {"NUMERICALLY_REJECTED"},
        "basis_sha256":expected,"dimension":440,"stored_entries":columns.iter().map(Vec::len).sum::<usize>(),
        "error":result.as_ref().err(),"numerics":stats_json(),"source":source_json(),
        "elapsed_seconds":started.elapsed().as_secs_f64(),
        "scope":"Original stored basis; all unit RHS normal/transpose dense/sparse. Exact fallback concerns stored dyadics; no LP feasibility/model claim."
    }))
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    if args.len() == 2 && args[0] == "--verify-report" {
        report_output::verify(Path::new(&args[1]))?;
        return Ok(());
    }
    if args.len() != 6
        || !["--input", "--basis"].contains(&args[0].as_str())
        || args[2] != "--expected-sha256"
        || args[4] != "--output"
        || args[3].len() != 64
        || !args[3].bytes().all(|x| x.is_ascii_hexdigit())
    {
        return Err(
            "usage: --input/--basis FILE --expected-sha256 HEX --output EXCLUSIVE_DIR".into(),
        );
    }
    if args[0] == "--basis" && !BASIS_PINS.contains(&args[3].as_str()) {
        return Err("basis is not one of four registered captures".into());
    }
    if args[0] == "--input" {
        if args[3] != "0fe91060f10c0d6107be3565f9ae0746c30e742a00d02935398f6764b84b974d" {
            return Err("wrong historical input".into());
        }
        let gate = PathBuf::from(std::env::var("D22_BASIS_ADMISSION_ROOT")?);
        for (i, pin) in BASIS_PINS.iter().enumerate() {
            let root = gate.join(format!("basis{i}"));
            report_output::verify(&root)?;
            let r: serde_json::Value =
                serde_json::from_slice(&fs::read(root.join("report.json"))?)?;
            let current_source = source_json();
            // Basis qualification calls repair::qualify/LU directly. A later
            // solver-state correction does not invalidate those unchanged
            // factorization checks; its new identity is recorded in replay.
            let basis_source_matches = ["repair_sha256", "lu_sha256", "constructor_sha256"]
                .iter()
                .all(|&key| r["source"][key] == current_source[key]);
            if !basis_outcome_admits_replay(i, &r)
                || r["basis_sha256"] != *pin
                || !basis_source_matches
            {
                return Err("four exact same-source basis outcomes do not admit replay".into());
            }
        }
    }
    let input = PathBuf::from(&args[1]);
    let out = PathBuf::from(&args[5]);
    if out.exists() {
        return Err("output already exists".into());
    }
    report_output::claim(&out)?;
    let result = if args[0] == "--basis" {
        execute_basis(&input, &args[3], &out)
    } else {
        execute(&input, &args[3], &out)
    };
    let report = match &result {
        Ok(v) => v.clone(),
        Err(e) => serde_json::json!({"status":"FAILED","error":e.to_string()}),
    };
    write(&out.join("report.json"), &report)?;
    report_output::seal(&out)?;
    report_output::verify(&out)?;
    result.map(|_| ())
}
