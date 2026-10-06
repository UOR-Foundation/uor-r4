//! Offline sealed-checkpoint margin diagnosis. Labels never enter native scoring.
//! This neither fits a model nor changes runtime admission or selection.
use serde_json::{json, Value};
use std::{
    collections::BTreeSet,
    fs, io,
    path::{Component, Path, PathBuf},
};
use uor_r4_core::{
    native_geometric::learner::geometric_generate::{
        GenerateReadCounts, NativeGeometricGenerate, SCORE_SHIFT,
    },
    report_output,
};
use uor_r4_integer::{geometric_source_actions::SourceActionBinding, h4_tables::H4Code};
use uor_r4_training::sha256_bytes;
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
fn bad(message: &str) -> Box<dyn std::error::Error> {
    io::Error::new(io::ErrorKind::InvalidData, message).into()
}
fn read(path: &Path) -> Result<Value> {
    Ok(serde_json::from_slice(&fs::read(path)?)?)
}
fn array<'a>(value: &'a Value, key: &str) -> Result<&'a Vec<Value>> {
    value[key]
        .as_array()
        .ok_or_else(|| bad(&format!("missing array {key}")))
}
fn uint(value: &Value, key: &str) -> Result<u64> {
    value[key]
        .as_u64()
        .ok_or_else(|| bad(&format!("missing integer {key}")))
}
fn integers(value: &Value) -> Result<Vec<i64>> {
    value
        .as_array()
        .ok_or_else(|| bad("integer array absent"))?
        .iter()
        .map(|v| v.as_i64().ok_or_else(|| bad("noninteger score")))
        .collect()
}
fn relative_file(root: &Path, name: &str) -> Result<PathBuf> {
    let path = Path::new(name);
    if path
        .components()
        .any(|c| !matches!(c, Component::Normal(_)))
    {
        return Err(bad("row path must be relative with no traversal"));
    }
    Ok(root.join(path))
}
fn decomposition(
    model: &NativeGeometricGenerate,
    state: &[H4Code],
    token: usize,
    expected: i64,
) -> Result<Value> {
    if state.len() != model.lanes() || token >= model.vocab_size() {
        return Err(bad("decomposition shape/domain mismatch"));
    }
    let mut relative = Vec::with_capacity(model.lanes());
    let mut unary = Vec::new();
    let mut sum = 0i64;
    for (lane, s) in state.iter().enumerate() {
        let prototype = model.prototypes()[token * model.lanes() + lane];
        let r = model
            .algebra()
            .compose(model.algebra().inverse(s.index())?, prototype)?;
        relative.push(r);
        let score = i64::from(model.energy().get_unary(lane as u8, r)?) << SCORE_SHIFT;
        sum = sum
            .checked_add(score)
            .ok_or_else(|| bad("unary sum overflow"))?;
        unary.push(json!({"lane":lane,"state":s.index(),"prototype":prototype,"relative":r,"score_q24":score}));
    }
    let mut pair = Vec::new();
    for (edge, e) in model.energy().edges().iter().enumerate() {
        let score = i64::from(model.energy().get_pair(
            edge,
            relative[usize::from(e.left)],
            relative[usize::from(e.right)],
        )?) << SCORE_SHIFT;
        sum = sum
            .checked_add(score)
            .ok_or_else(|| bad("pair sum overflow"))?;
        pair.push(json!({"edge":edge,"left":e.left,"right":e.right,"score_q24":score}));
    }
    let bias = i64::from(model.token_bias(token)?) << SCORE_SHIFT;
    sum = sum
        .checked_add(bias)
        .ok_or_else(|| bad("bias sum overflow"))?;
    if sum != expected {
        return Err(bad(
            "factor decomposition differs from authoritative native score",
        ));
    }
    Ok(json!({"token_id":token,"unary":unary,"pair":pair,"bias_q24":bias,"sum_q24":sum}))
}
fn margin(a: &Value, b: &Value) -> Result<Value> {
    let differences = |key: &str| -> Result<Vec<i64>> {
        let left = array(a, key)?;
        let right = array(b, key)?;
        if left.len() != right.len() {
            return Err(bad("factor margin shape differs"));
        }
        left.iter()
            .zip(right)
            .map(|(x, y)| {
                x["score_q24"]
                    .as_i64()
                    .and_then(|x| y["score_q24"].as_i64().and_then(|y| x.checked_sub(y)))
                    .ok_or_else(|| bad("factor margin overflow"))
            })
            .collect()
    };
    let unary = differences("unary")?;
    let pair = differences("pair")?;
    let bias = a["bias_q24"]
        .as_i64()
        .and_then(|x| b["bias_q24"].as_i64().and_then(|y| x.checked_sub(y)))
        .ok_or_else(|| bad("bias margin overflow"))?;
    let total = unary.iter().chain(&pair).try_fold(bias, |sum, x| {
        sum.checked_add(*x)
            .ok_or_else(|| bad("margin sum overflow"))
    })?;
    let expected = a["sum_q24"]
        .as_i64()
        .and_then(|x| b["sum_q24"].as_i64().and_then(|y| x.checked_sub(y)))
        .ok_or_else(|| bad("score margin overflow"))?;
    if total != expected {
        return Err(bad("margin decomposition mismatch"));
    }
    Ok(json!({"unary_q24":unary,"pair_q24":pair,"bias_q24":bias,"sum_q24":total}))
}
fn audit(fit: &Path, out: &Path) -> Result<Value> {
    report_output::verify(fit)?;
    let fit_report = read(&fit.join("report.json"))?;
    if fit_report["schema"] != "uor-r4.geometric-bank-generate-fit/1"
        || fit_report["status"] != "COMPLETED"
    {
        return Err(bad("requires completed sealed Generate fit report"));
    }
    let stages = array(&fit_report, "stages")?;
    if stages.is_empty() || stages.len() > 32 {
        return Err(bad("stage count outside1..32"));
    }
    let mut results = Vec::new();
    let mut initial: Option<Vec<u8>> = None;
    let mut previous: Option<Vec<u8>> = None;
    let mut previous_step = None;
    let mut tokenizer_sha: Option<String> = None;
    let mut audited_positions = 0usize;
    for stage in stages {
        let step = uint(stage, "step")?;
        if previous_step.is_some_and(|s| step <= s) {
            return Err(bad("stages not strictly increasing"));
        }
        previous_step = Some(step);
        let checkpoint = fit.join(format!("checkpoint-{step:04}"));
        let tokenizer = fs::read(checkpoint.join("native/tokenizer.json"))?;
        let binding = SourceActionBinding::new(&tokenizer)?;
        if tokenizer_sha
            .as_ref()
            .is_some_and(|sha| sha != binding.tokenizer_sha256())
        {
            return Err(bad("tokenizer identity changed across stages"));
        }
        tokenizer_sha = Some(binding.tokenizer_sha256().to_owned());
        let bytes = fs::read(checkpoint.join("generate.bin"))?;
        let hash = sha256_bytes(&bytes);
        if stage["checkpoint"]["generate_sha256"].as_str() != Some(hash.as_str()) {
            return Err(bad("checkpoint Generate hash differs from sealed receipt"));
        }
        let model = NativeGeometricGenerate::from_bytes(&bytes, &binding)?;
        if model.to_bytes()? != bytes {
            return Err(bad("Generate independent byte reload differs"));
        }
        let refs = array(&stage["evaluation"], "rows")?;
        if refs.is_empty()
            || refs.len() > 512
            || uint(&stage["evaluation"], "cases")? != refs.len() as u64
        {
            return Err(bad("evaluation row count outside1..512 or inconsistent"));
        }
        audited_positions = audited_positions
            .checked_add(refs.len())
            .ok_or_else(|| bad("audit count overflow"))?;
        if audited_positions > 1024 {
            return Err(bad(
                "audit exceeds1024 first positions per fit; two-arm budget2048",
            ));
        }
        let legal: Vec<usize> = (0..model.vocab_size())
            .filter(|&id| binding.admits_token(id as u32))
            .collect();
        if legal.is_empty() {
            return Err(bad("no legal Generate tokens"));
        }
        let prototypes = model.prototypes().to_vec();
        if initial
            .as_ref()
            .is_some_and(|v| v.len() != prototypes.len())
        {
            return Err(bad("prototype dimensions changed across stages"));
        }
        let changed = |prior: &Option<Vec<u8>>| {
            prior
                .as_ref()
                .map(|p| p.iter().zip(&prototypes).filter(|(a, b)| a != b).count())
                .unwrap_or(0)
        };
        let lane_diversity: Vec<usize> = (0..model.lanes())
            .map(|l| {
                legal
                    .iter()
                    .map(|&t| prototypes[t * model.lanes() + l])
                    .collect::<BTreeSet<_>>()
                    .len()
            })
            .collect();
        let tuple_diversity = legal
            .iter()
            .map(|&t| prototypes[t * model.lanes()..(t + 1) * model.lanes()].to_vec())
            .collect::<BTreeSet<_>>()
            .len();
        let mut rows = Vec::new();
        let mut gold_absent_copy = 0;
        for reference in refs {
            let name = reference["row_file"]
                .as_str()
                .ok_or_else(|| bad("row_file absent"))?;
            let path = relative_file(fit, name)?;
            let row_bytes = fs::read(&path)?;
            let row_hash = sha256_bytes(&row_bytes);
            if reference["row_sha256"].as_str() != Some(row_hash.as_str()) {
                return Err(bad("row hash differs from sealed evaluation reference"));
            }
            let row: Value = serde_json::from_slice(&row_bytes)?;
            if row["id"] != reference["id"] {
                return Err(bad("row identity differs"));
            }
            let generation = array(&row, "generation")?
                .first()
                .ok_or_else(|| bad("first generation position absent"))?;
            if !array(generation, "actual_prefix_ids")?.is_empty() {
                return Err(bad("first generation is not empty-prefix"));
            }
            let state = integers(&generation["retained_state_codes"])?;
            let state = state
                .into_iter()
                .map(|v| {
                    u8::try_from(v)
                        .map_err(|_| bad("retained code out of range"))
                        .and_then(|v| H4Code::try_from(v).map_err(|e| e.into()))
                })
                .collect::<Result<Vec<_>>>()?;
            let mut scores = vec![0i64; model.vocab_size()];
            let mut counts = GenerateReadCounts::default();
            model.score_into(&state, &mut scores, &mut counts)?;
            // Exact driver encoding: SHA256(serde_json::to_vec(&Vec<i64>)).
            // Gold labels are deliberately opened only after this target-free replay.
            let score_hash = sha256_bytes(&serde_json::to_vec(&scores)?);
            if generation["generate_raw_scores_sha256"].as_str() != Some(score_hash.as_str()) {
                return Err(bad(
                    "authoritative full-vocabulary Generate replay hash mismatch",
                ));
            }
            let canonical = array(&row, "canonical")?
                .first()
                .ok_or_else(|| bad("first canonical position absent"))?;
            let canonical_native = &canonical["native"];
            if !array(canonical_native, "actual_prefix_ids")?.is_empty() {
                return Err(bad("first canonical position is not empty-prefix"));
            }
            for field in [
                "retained_state_codes",
                "generate_raw_scores_sha256",
                "copy_token_ids",
                "copy_raw_scores_q24",
                "pool",
            ] {
                if canonical_native[field] != generation[field] {
                    return Err(bad(&format!(
                        "first canonical/generation native parity differs: {field}"
                    )));
                }
            }
            let gold = array(&row, "canonical_target_ids_labels_only")?
                .first()
                .and_then(Value::as_u64)
                .ok_or_else(|| bad("first gold token label absent"))?;
            let gold = usize::try_from(gold)?;
            if gold >= scores.len() || !binding.admits_token(gold as u32) {
                return Err(bad("gold label outside legal Generate domain"));
            }
            if canonical["target_label_only"].as_u64() != Some(gold as u64) {
                return Err(bad("first canonical label differs from row target"));
            }
            let target_mass = uint(canonical, "native_target_mass")?;
            let denominator = uint(canonical, "native_denominator")?;
            let summary = &generation["pool"]["summary"];
            let chosen_mass = uint(summary, "chosen_weight_q31")?;
            if denominator == 0
                || target_mass == 0
                || chosen_mass == 0
                || uint(summary, "total_weight_q31")? != denominator
                || target_mass > chosen_mass
                || chosen_mass > denominator
            {
                return Err(bad("saved pooled target/winner weight contract differs"));
            }
            let weight_margin = i64::try_from(target_mass)?
                .checked_sub(i64::try_from(chosen_mass)?)
                .ok_or_else(|| bad("pooled weight margin overflow"))?;
            let best = *legal
                .iter()
                .max_by(|&&a, &&b| scores[a].cmp(&scores[b]).then_with(|| b.cmp(&a)))
                .ok_or_else(|| bad("best Generate absent"))?;
            let chosen = usize::try_from(uint(&generation["pool"]["summary"], "chosen_token_id")?)?;
            if chosen >= scores.len() || !binding.admits_token(chosen as u32) {
                return Err(bad("saved pool winner outside legal domain"));
            }
            let copy_ids = integers(&generation["copy_token_ids"])?;
            let copy_scores = integers(&generation["copy_raw_scores_q24"])?;
            if copy_ids.len() != copy_scores.len() || copy_ids.len() > 128 {
                return Err(bad("Copy score/id shape or capacity mismatch"));
            }
            let absent = !copy_ids.contains(&(gold as i64));
            gold_absent_copy += usize::from(absent);
            let copy_max = copy_scores
                .iter()
                .enumerate()
                .max_by(|(a, x), (b, y)| x.cmp(y).then_with(|| b.cmp(a)))
                .map(|(i, &s)| json!({"occurrence_index":i,"token_id":copy_ids[i],"score_q24":s}));
            let target = decomposition(&model, &state, gold, scores[gold])?;
            let best_parts = decomposition(&model, &state, best, scores[best])?;
            let chosen_parts = decomposition(&model, &state, chosen, scores[chosen])?;
            rows.push(json!({"id":row["id"],"row_file":name,"generate_scores_replay_sha256":score_hash,"retained_states":state.iter().map(|s|s.index()).collect::<Vec<_>>(),"target_label_only":gold,"target_score_q24":scores[gold],"best_generate_token_id":best,"best_generate_score_q24":scores[best],"pool_winner_token_id":chosen,"winner_generate_component_score_q24":scores[chosen],"copy_raw_winner":copy_max,"pooled_saved_weights":{"native_target_mass":target_mass,"native_denominator":denominator,"chosen_weight_q31":chosen_mass,"chosen_generate_weight_q31":uint(summary,"chosen_generate_weight_q31")?,"chosen_copy_weight_q31":uint(summary,"chosen_copy_weight_q31")?,"target_minus_chosen_weight_q31":weight_margin,"source":"canonical saved exact reducer mass; canonical/generation first native packets verified identical; no reducer reimplementation"},"first_gold_absent_copy":absent,"target_decomposition":target,"best_generate_decomposition":best_parts,"pool_winner_generate_decomposition":chosen_parts,"target_minus_best_generate":margin(&target,&best_parts)?,"target_minus_pool_winner_generate":margin(&target,&chosen_parts)?}));
        }
        results.push(json!({"step":step,"generate_sha256":hash,"tokenizer_sha256":binding.tokenizer_sha256(),"cases":rows.len(),"first_gold_absent_copy":gold_absent_copy,"native_prototypes":{"changed_entries_from_initial":changed(&initial),"changed_entries_from_previous":changed(&previous),"entries":prototypes.len(),"legal_token_count":legal.len(),"distinct_codes_by_lane":lane_diversity,"distinct_legal_token_tuples":tuple_diversity},"rows":rows}));
        if initial.is_none() {
            initial = Some(prototypes.clone());
        }
        previous = Some(prototypes);
    }
    // Verify the input remained sealed throughout the read-only analysis.
    report_output::verify(fit)?;
    let report = json!({"schema":"uor-r4.geometric-generate-margin-audit/1","status":"COMPLETED","scope":"offline label-only first-position integer margin analysis; no fit, runtime admission, design selection or chat qualification","input_root":fit,"audited_first_positions":audited_positions,"position_cap_per_fit":1024,"input_fit_source_commit":fit_report["source_commit"],"input_fit_executable_sha256":fit_report["executable_sha256"],"audit_source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT"),"audit_executable_sha256":sha256_bytes(&fs::read(std::env::current_exe()?)?),"input_manifest_sha256":sha256_bytes(&fs::read(fit.join("manifest.json"))?),"input_report_sha256":sha256_bytes(&fs::read(fit.join("report.json"))?),"score_hash_encoding":"SHA256(serde_json::to_vec(Vec<i64>)); exact fit driver encoding","score_unit":"Q24; signed coefficient sum shifted20","stages":results});
    fs::write(out.join("report.json"), serde_json::to_vec_pretty(&report)?)?;
    Ok(report)
}
fn main() -> Result<()> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() != 2 {
        return Err(bad(
            "usage: geometric-generate-margin-audit FIT_ROOT REPORT_OUTPUT",
        ));
    }
    let fit = PathBuf::from(&args[0]);
    let out = PathBuf::from(&args[1]);
    if !fit.is_dir() {
        return Err(bad("fit_root must exist"));
    }
    report_output::claim(&out)?;
    let result = audit(&fit, &out);
    if let Err(error) = &result {
        fs::write(
            out.join("failure.json"),
            serde_json::to_vec_pretty(
                &json!({"status":"FAILED","error":error.to_string(),"scope":"offline audit failure; not a model verdict"}),
            )?,
        )?;
    }
    report_output::seal(&out)?;
    report_output::verify(&out)?;
    result.map(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn decomposition_margin_exact_sum_and_shape_admission() -> Result<()> {
        let a = json!({"unary":[{"score_q24":3},{"score_q24":-2}],"pair":[{"score_q24":4}],"bias_q24":1,"sum_q24":6});
        let b = json!({"unary":[{"score_q24":1},{"score_q24":2}],"pair":[{"score_q24":-3}],"bias_q24":-2,"sum_q24":-2});
        let result = margin(&a, &b)?;
        assert_eq!(result["unary_q24"], json!([2, -4]));
        assert_eq!(result["pair_q24"], json!([7]));
        assert_eq!(result["bias_q24"], 3);
        assert_eq!(result["sum_q24"], 8);
        let mut corrupted = a.clone();
        corrupted["sum_q24"] = json!(7);
        assert!(margin(&corrupted, &b).is_err());
        let mut wrong_shape = a;
        wrong_shape["pair"] = json!([]);
        assert!(margin(&wrong_shape, &b).is_err());
        Ok(())
    }
    #[test]
    fn row_paths_cannot_escape_claimed_input() -> Result<()> {
        let root = Path::new("/audit-input");
        assert_eq!(
            relative_file(root, "development-row-0000.json")?,
            root.join("development-row-0000.json")
        );
        for name in [
            "../elsewhere.json",
            "/absolute.json",
            "sub/../../other.json",
        ] {
            assert!(relative_file(root, name).is_err());
        }
        Ok(())
    }
}
