//! Fixed gain-four re-export of retained bridge masters; geometric operator probe only.
//! No training, label reads, language evaluation, or change to the parent artifact.
#![recursion_limit = "256"]
use candle_core::{Device, Var};
use serde::Deserialize;
use serde_json::{json, Value};
use std::{
    collections::BTreeSet,
    fs, io,
    path::{Path, PathBuf},
};
use uor_r4_core::{
    native_geometric::learner::geometric_read_state_bridge::{
        BridgeReadCounts, NativeGeometricReadStateBridge, ROOTS,
    },
    report_output,
};
use uor_r4_integer::{
    geometric_source_realizer::{NativeArtifactBinding, NativeSourceRealizer as IntegerRealizer},
    h4_tables::H4Code,
};
use uor_r4_training::{geometric_read_state_bridge::BridgeLearningWeights, sha256_bytes};
#[path = "../../uor-r4-integer/examples/support/source_probe.rs"]
mod output_support;
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
const GAIN: f32 = 4.;
const GAP_BOUND: f64 = 0.125;
fn bad(s: impl Into<String>) -> Box<dyn std::error::Error> {
    Box::new(io::Error::new(io::ErrorKind::InvalidData, s.into()))
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct MasterIdentity {
    shape: Vec<usize>,
    bytes: usize,
    sha256: String,
}
fn read_json(p: &Path) -> Result<Value> {
    Ok(serde_json::from_slice(&fs::read(p)?)?)
}
fn seal_for(p: &Path) -> Result<PathBuf> {
    p.ancestors()
        .find(|a| a.join("manifest.json").is_file())
        .map(Path::to_path_buf)
        .ok_or_else(|| bad("checkpoint has no enclosing seal"))
}
fn masters(cp: &Path, name: &str, shape: &[usize], metadata: &Value) -> Result<Vec<f32>> {
    let id: MasterIdentity = serde_json::from_value(metadata["source_parameters"][name].clone())?;
    let bytes = fs::read(
        cp.join("read-state-bridge-source")
            .join(format!("{name}.f32le")),
    )?;
    let count = shape
        .iter()
        .try_fold(1usize, |n, &d| n.checked_mul(d))
        .ok_or_else(|| bad("shape overflow"))?;
    if id.shape != shape
        || id.bytes != bytes.len()
        || bytes.len()
            != count
                .checked_mul(4)
                .ok_or_else(|| bad("byte count overflow"))?
        || id.sha256 != sha256_bytes(&bytes)
    {
        return Err(bad(format!("master shape/bytes/SHA mismatch: {name}")));
    }
    let values = bytes
        .chunks_exact(4)
        .map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]]))
        .collect::<Vec<_>>();
    validate_gain(&values)?;
    Ok(values)
}
fn validate_gain(values: &[f32]) -> Result<()> {
    if values.iter().any(|&v| {
        !v.is_finite() || v.abs() > 1.75 || !(v * GAIN).is_finite() || (v * GAIN).abs() > 1.75
    }) {
        return Err(bad("original or uniformly gain-four master outside finite strict quarter range; no clipping permitted"));
    }
    Ok(())
}
// Native tie policy starts at identity code 1, then visits ascending codes strictly.
fn winner(scores: &[f64]) -> Result<(usize, f64, usize)> {
    if scores.len() != ROOTS || scores.iter().any(|x| !x.is_finite()) {
        return Err(bad("invalid reference scores"));
    }
    let mut best = 1;
    for a in 0..ROOTS {
        if scores[a] > scores[best] {
            best = a;
        }
    }
    let runner = scores
        .iter()
        .enumerate()
        .filter(|(a, _)| *a != best)
        .map(|(_, s)| *s)
        .fold(f64::NEG_INFINITY, f64::max);
    let ties = scores.iter().filter(|&&s| s == scores[best]).count();
    Ok((best, scores[best] - runner, ties))
}
fn export(
    binding: &uor_r4_integer::geometric_source_actions::SourceActionBinding,
    original: &NativeGeometricReadStateBridge,
    bias: &[f32],
    relative: &[f32],
    gain: f32,
) -> Result<NativeGeometricReadStateBridge> {
    let lanes = original.lanes();
    let weights = BridgeLearningWeights::from_native(original, binding, &Device::Cpu)?;
    weights.bias.set(
        Var::from_vec(
            bias.iter().map(|x| x * gain).collect::<Vec<_>>(),
            (lanes, ROOTS),
            &Device::Cpu,
        )?
        .as_tensor(),
    )?;
    weights.relative.set(
        Var::from_vec(
            relative.iter().map(|x| x * gain).collect::<Vec<_>>(),
            (lanes, ROOTS, ROOTS),
            &Device::Cpu,
        )?
        .as_tensor(),
    )?;
    Ok(weights.export_native()?)
}
fn apply(native: &NativeGeometricReadStateBridge, d: u8) -> Result<(Vec<u8>, BridgeReadCounts)> {
    let lanes = native.lanes();
    let q = vec![H4Code::IDENTITY; lanes];
    let k = vec![H4Code::try_from(d)?; lanes];
    let mut post = q.clone();
    let mut actions = q.clone();
    let mut scores = vec![0; lanes * ROOTS];
    let mut counts = BridgeReadCounts::default();
    native.apply_into(&q, &k, &mut post, &mut actions, &mut scores, &mut counts)?;
    if post != actions {
        return Err(bad("identity-query native post differs from action"));
    }
    Ok((actions.iter().map(|a| a.index()).collect(), counts))
}
fn nonzero(native: &NativeGeometricReadStateBridge) -> Result<Value> {
    let mut bias = 0usize;
    let mut relative = 0usize;
    for l in 0..native.lanes() {
        for a in 0..ROOTS {
            bias += usize::from(native.coefficient_bias(l, a)? != 0);
            for d in 0..ROOTS {
                relative += usize::from(native.coefficient_relative(l, a, d)? != 0);
            }
        }
    }
    Ok(
        json!({"bias_coefficient_nonzero":bias,"relative_coefficient_nonzero":relative,"packed_bias_bytes_nonzero":native.packed_bias().iter().filter(|&&b|b!=0).count(),"packed_relative_bytes_nonzero":native.packed_relative().iter().filter(|&&b|b!=0).count()}),
    )
}
fn run(cp: &Path, out: &Path, seal: &Path) -> Result<Value> {
    report_output::verify(seal)?;
    let receipt = read_json(&cp.join("receipt.json"))?;
    let metadata = read_json(&cp.join("read-state-bridge-source/metadata.json"))?;
    if receipt["read_state_bridge"] != metadata || metadata["enabled"] != true {
        return Err(bad(
            "checkpoint receipt and master metadata differ/disabled",
        ));
    }
    let binding: NativeArtifactBinding = serde_json::from_value(receipt["parent"].clone())?;
    let integer = IntegerRealizer::load_native(&cp.join("native"), &binding)?;
    let original_bytes = fs::read(cp.join("read-state-bridge.bin"))?;
    if metadata["native_sha256"] != sha256_bytes(&original_bytes) {
        return Err(bad("parent bridge SHA mismatch"));
    }
    let original = NativeGeometricReadStateBridge::from_bytes(&original_bytes, integer.binding())?;
    if metadata["metadata"] != serde_json::to_value(original.metadata())? {
        return Err(bad("parent bridge metadata mismatch"));
    }
    let lanes = original.lanes();
    if lanes != 8 {
        return Err(bad(
            "predeclared probe requires the retained eight-lane bridge",
        ));
    }
    let bias = masters(cp, "read_state_bridge.bias", &[lanes, ROOTS], &metadata)?;
    let relative = masters(
        cp,
        "read_state_bridge.relative",
        &[lanes, ROOTS, ROOTS],
        &metadata,
    )?;
    let coarse = export(integer.binding(), &original, &bias, &relative, 1.)?;
    if coarse.to_bytes()? != original_bytes {
        return Err(bad(
            "retained masters do not reproduce original coarse artifact byte-for-byte",
        ));
    }
    let fine = export(integer.binding(), &original, &bias, &relative, GAIN)?;
    let fine_bytes = fine.to_bytes()?;
    fs::write(out.join("read-state-bridge-gain4.bin"), &fine_bytes)?;
    let fine = NativeGeometricReadStateBridge::from_bytes(
        &fs::read(out.join("read-state-bridge-gain4.bin"))?,
        integer.binding(),
    )?;
    if fine.to_bytes()? != fine_bytes {
        return Err(bad("derived native independent disk reload differs"));
    }
    let mut rows = Vec::new();
    let mut coarse_sets = vec![BTreeSet::new(); lanes];
    let mut fine_sets = coarse_sets.clone();
    let mut reference_sets = vec![BTreeSet::<usize>::new(); lanes];
    let mut coarse_agree = vec![0usize; lanes];
    let mut fine_agree = coarse_agree.clone();
    let mut guaranteed = coarse_agree.clone();
    let mut violations = coarse_agree.clone();
    let mut ties = coarse_agree.clone();
    let mut min_gap = vec![f64::INFINITY; lanes];
    let mut max_gap = vec![0f64; lanes];
    for d in 0..ROOTS {
        let (ca, cc) = apply(&original, d as u8)?;
        let (fa, fc) = apply(&fine, d as u8)?;
        for l in 0..lanes {
            let scores = (0..ROOTS)
                .map(|a| {
                    f64::from(bias[l * ROOTS + a])
                        + f64::from(relative[(l * ROOTS + a) * ROOTS + d])
                })
                .collect::<Vec<_>>();
            let (ra, gap, tie_count) = winner(&scores)?;
            coarse_sets[l].insert(ca[l]);
            fine_sets[l].insert(fa[l]);
            reference_sets[l].insert(ra);
            coarse_agree[l] += usize::from(usize::from(ca[l]) == ra);
            fine_agree[l] += usize::from(usize::from(fa[l]) == ra);
            ties[l] += usize::from(tie_count > 1);
            min_gap[l] = min_gap[l].min(gap);
            max_gap[l] = max_gap[l].max(gap);
            let protected = tie_count == 1 && gap > GAP_BOUND;
            guaranteed[l] += usize::from(protected);
            violations[l] += usize::from(protected && usize::from(fa[l]) != ra);
            rows.push(json!({"lane":l,"relative_code":d,"coarse_action":ca[l],"gain4_action":fa[l],"reference_action":ra,"runner_up_gap":gap,"maximum_tie_count":tie_count,"gap_bound_protects_winner":protected,"bound_violation":protected && usize::from(fa[l])!=ra,"coarse_counts_all_lanes":cc,"gain4_counts_all_lanes":fc}));
        }
    }
    fs::write(out.join("rows.json"), serde_json::to_vec_pretty(&rows)?)?;
    let per_lane = (0..lanes).map(|l| json!({"lane":l,"keys":ROOTS,"coarse_reference_agreement":coarse_agree[l],"gain4_reference_agreement":fine_agree[l],"coarse_distinct_actions":coarse_sets[l],"gain4_distinct_actions":fine_sets[l],"reference_distinct_actions":reference_sets[l],"coarse_key_dependent":coarse_sets[l].len()>1,"gain4_key_dependent":fine_sets[l].len()>1,"reference_key_dependent":reference_sets[l].len()>1,"reference_tied_keys":ties[l],"minimum_runner_up_gap":min_gap[l],"maximum_runner_up_gap":max_gap[l],"gap_protected_keys":guaranteed[l],"bound_violations":violations[l]})).collect::<Vec<_>>();
    let report = json!({"status":if violations.iter().any(|&n|n>0){"FAILED_BOUND"}else{"COMPLETED"},"schema":"uor-r4.read-state-export-probe/1","checkpoint":cp,"parent_seal_root":seal,"parent_manifest_sha256":sha256_bytes(&fs::read(seal.join("manifest.json"))?),"checkpoint_receipt_sha256":sha256_bytes(&fs::read(cp.join("receipt.json"))?),"master_metadata_sha256":sha256_bytes(&fs::read(cp.join("read-state-bridge-source/metadata.json"))?),"master_identities":metadata["source_parameters"],"parent_binding":binding,"original_native_sha256":sha256_bytes(&original_bytes),"derived_native_sha256":sha256_bytes(&fine_bytes),"derived_payload_sha256":fine.metadata().payload_sha256,"gain":GAIN,"rule":"uniform exact power-of-two gain4 on BOTH retained F32 families, existing quarter exporter, no clipping/fit/labels/sweep; original master-space quantum1/16; native energy scale changes uniformly","reference":"original unquantized B+T converted individually F32 to F64, identity-first ascending strict ties, q=identity and all120 relative codes","winner_gap_bound":GAP_BOUND,"bound_reason":"two coefficient errors <=1/32 each => score error <=1/16 => winner difference error <=1/8; unique gap strictly greater1/8 must agree","original_coarse_reproduction":"BYTE_IDENTICAL","original_nonzero":nonzero(&original)?,"gain4_nonzero":nonzero(&fine)?,"lanes":per_lane,"scope":"geometric operator export discriminator only; no context-selection, language, gradient, CUDA-parity or semantic-distance claim","source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT"),"executable_sha256":sha256_bytes(&fs::read(std::env::current_exe()?)?)});
    fs::write(out.join("probe.json"), serde_json::to_vec_pretty(&report)?)?;
    if violations.iter().any(|&n| n > 0) {
        return Err(bad(
            "gain-four quantization winner bound violated; probe evidence retained",
        ));
    }
    Ok(report)
}
fn main() -> Result<()> {
    let mut args = std::env::args_os().skip(1);
    let cp = PathBuf::from(
        args.next()
            .ok_or_else(|| bad("usage: geometric-read-state-export-probe CHECKPOINT OUT"))?,
    )
    .canonicalize()?;
    let out = PathBuf::from(args.next().ok_or_else(|| bad("output path missing"))?);
    if args.next().is_some() {
        return Err(bad("unexpected argument; gain is fixed4"));
    }
    let seal = seal_for(&cp)?;
    let out = output_support::prospective_output(&out)?;
    if out.starts_with(&seal)
        || out
            .ancestors()
            .any(|ancestor| ancestor.join("manifest.json").is_file())
    {
        return Err(bad("output must be outside every sealed report"));
    }
    report_output::claim(&out)?;
    let result = run(&cp, &out, &seal);
    let report = match &result {
        Ok(v) => v.clone(),
        Err(e) => {
            json!({"schema":"uor-r4.read-state-export-probe/1","status":"FAILED","error":e.to_string(),"checkpoint":cp,"gain":GAIN,"scope":"execution/export probe failure; no language verdict"})
        }
    };
    fs::write(out.join("report.json"), serde_json::to_vec_pretty(&report)?)?;
    report_output::seal(&out)?;
    report_output::verify(&out)?;
    result.map(|_| ())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn identity_and_ascending_strict_ties() -> Result<()> {
        let mut scores = vec![0.; ROOTS];
        assert_eq!(winner(&scores)?.0, 1);
        scores[0] = 1.;
        scores[2] = 1.;
        let (a, g, t) = winner(&scores)?;
        assert_eq!((a, g, t), (0, 0., 2));
        Ok(())
    }
    #[test]
    fn gain_rejects_nonfinite_and_range_without_clipping() {
        assert!(validate_gain(&[f32::NAN]).is_err());
        assert!(validate_gain(&[0.43750003]).is_err());
        assert!(validate_gain(&[0.4375, -0.4375]).is_ok());
    }
}
