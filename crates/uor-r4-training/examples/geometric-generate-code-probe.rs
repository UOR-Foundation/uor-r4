//! Offline one-coordinate native code intervention on sealed teacher-forced states.
//! No optimizer resume, state fitting, own-prefix generation or transfer claim.
use serde_json::{json, Value};
use std::{
    collections::BTreeMap,
    fs, io,
    path::{Component, Path, PathBuf},
    time::Instant,
};
use uor_r4_core::{
    native_geometric::learner::geometric_generate::{GenerateReadCounts, NativeGeometricGenerate},
    report_output,
};
use uor_r4_integer::{
    geometric_source_actions::SourceActionBinding,
    geometric_vocabulary_actions::{GenerateSubstitutionCache, NativeVocabularyActions},
    h4_tables::H4Code,
};
use uor_r4_training::sha256_bytes;
#[path = "../../uor-r4-integer/examples/support/source_probe.rs"]
mod output_support;
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
fn bad(s: &str) -> Box<dyn std::error::Error> {
    io::Error::new(io::ErrorKind::InvalidData, s).into()
}
fn read(p: &Path) -> Result<Value> {
    Ok(serde_json::from_slice(&fs::read(p)?)?)
}
fn arr<'a>(v: &'a Value, k: &str) -> Result<&'a Vec<Value>> {
    v[k].as_array().ok_or_else(|| bad("required array absent"))
}
fn uint(v: &Value, k: &str) -> Result<u64> {
    v[k].as_u64().ok_or_else(|| bad("required integer absent"))
}
fn ints(v: &Value) -> Result<Vec<i64>> {
    v.as_array()
        .ok_or_else(|| bad("integer array absent"))?
        .iter()
        .map(|x| x.as_i64().ok_or_else(|| bad("integer absent")))
        .collect()
}
fn relative(root: &Path, s: &str) -> Result<PathBuf> {
    let p = Path::new(s);
    if p.components().any(|c| !matches!(c, Component::Normal(_))) {
        return Err(bad("row path must be relative without traversal"));
    }
    Ok(root.join(p))
}
fn deadline(start: Instant) -> Result<()> {
    if start.elapsed().as_secs() >= 240 {
        return Err(bad("native probe 240-second execution ceiling"));
    }
    Ok(())
}
fn load_row(root: &Path, reference: &Value) -> Result<Value> {
    let p = relative(
        root,
        reference["row_file"]
            .as_str()
            .ok_or_else(|| bad("row_file absent"))?,
    )?;
    let bytes = fs::read(p)?;
    if reference["row_sha256"].as_str() != Some(sha256_bytes(&bytes).as_str()) {
        return Err(bad("sealed row SHA mismatch"));
    }
    let row: Value = serde_json::from_slice(&bytes)?;
    if row["id"] != reference["id"] {
        return Err(bad("row ID mismatch"));
    }
    Ok(row)
}
fn state(v: &Value) -> Result<Vec<H4Code>> {
    ints(&v["retained_state_codes"])?
        .into_iter()
        .map(|x| Ok(H4Code::try_from(u8::try_from(x)?)?))
        .collect()
}
fn replay(
    v: &Value,
    model: &NativeGeometricGenerate,
    pool: &mut NativeVocabularyActions,
) -> Result<(Vec<H4Code>, Vec<i64>, Vec<u32>, Vec<i64>)> {
    let s = state(v)?;
    let ids = ints(&v["copy_token_ids"])?
        .into_iter()
        .map(u32::try_from)
        .collect::<std::result::Result<Vec<_>, _>>()?;
    let copy = ints(&v["copy_raw_scores_q24"])?;
    let mut gen = vec![0; model.vocab_size()];
    model.score_into(&s, &mut gen, &mut GenerateReadCounts::default())?;
    if v["generate_raw_scores_sha256"].as_str()
        != Some(sha256_bytes(&serde_json::to_vec(&gen)?).as_str())
    {
        return Err(bad("full native score replay SHA mismatch"));
    }
    let trace = pool.reduce_trace(&gen, &ids, &copy)?;
    // The fitter removes per-token mass arrays from saved packets. Replay
    // the complete summary here; supervised target masses are checked below.
    if serde_json::to_value(&trace.summary)? != v["pool"]["summary"] {
        return Err(bad("authoritative native pool replay mismatch"));
    }
    Ok((s, gen, ids, copy))
}
fn loss(mass: u64, total: u64) -> Result<f64> {
    if mass == 0 || mass > total {
        return Err(bad("invalid positive alias mass"));
    }
    Ok(-(mass as f64 / total as f64).ln())
}
fn best_code(values: &[f64; 120], incumbent: usize) -> usize {
    let mut best = incumbent;
    for (code, &v) in values.iter().enumerate() {
        if v < values[best] {
            best = code;
        }
    }
    best
}
struct Position {
    state: Vec<H4Code>,
    copy_ids: Vec<u32>,
    copy: Vec<i64>,
    gold: u32,
    alternatives: [i64; 120],
    cache: GenerateSubstitutionCache,
}
fn probe(fit: &Path, config_path: &Path, out: &Path, start: Instant) -> Result<()> {
    report_output::verify(fit)?;
    deadline(start)?;
    let report = read(&fit.join("report.json"))?;
    if report["balanced_token_geometry"] != true
        || report["schema"] != "uor-r4.geometric-bank-generate-fit/1"
        || report["status"] != "COMPLETED"
        || uint(&report, "updates")? != 128
    {
        return Err(bad("requires completed 128-update fit"));
    }
    let config_bytes = fs::read(config_path)?;
    let config: Value = serde_json::from_slice(&config_bytes)?;
    for k in ["seed", "arm", "balanced_token_geometry"] {
        if config[k] != report[k] {
            return Err(bad("supplied fit config identity differs"));
        }
    }
    let admission = read(&fit.join("input-admission.json"))?;
    let mut data_hashes = BTreeMap::new();
    for (a, b) in [
        ("training_inputs", "development_inputs"),
        ("training_labels", "development_labels"),
    ] {
        let path_a = config[a]
            .as_str()
            .ok_or_else(|| bad("fit data path absent"))?;
        let path_b = config[b]
            .as_str()
            .ok_or_else(|| bad("fit data path absent"))?;
        let ha = sha256_bytes(&fs::read(path_a)?);
        let hb = sha256_bytes(&fs::read(path_b)?);
        if ha != hb
            || admission["input_sha256"][path_a].as_str() != Some(ha.as_str())
            || admission["input_sha256"][path_b].as_str() != Some(hb.as_str())
        {
            return Err(bad(
                "training/development data or labels are not the same bound construction panel",
            ));
        }
        data_hashes.insert(a, ha);
    }
    let stages = arr(&report, "stages")?;
    let stage = stages.last().ok_or_else(|| bad("final stage absent"))?;
    if uint(stage, "step")? != 128 {
        return Err(bad("final stage must be128"));
    }
    let cp = fit.join("checkpoint-0128");
    let binding = SourceActionBinding::new(&fs::read(cp.join("native/tokenizer.json"))?)?;
    let bytes = fs::read(cp.join("generate.bin"))?;
    let model = NativeGeometricGenerate::from_bytes(&bytes, &binding)?;
    if stage["checkpoint"]["generate_sha256"].as_str() != Some(sha256_bytes(&bytes).as_str())
        || model.to_bytes()? != bytes
    {
        return Err(bad("Generate checkpoint identity differs"));
    }
    let initial_cp = fit.join("checkpoint-0000");
    let initial_binding =
        SourceActionBinding::new(&fs::read(initial_cp.join("native/tokenizer.json"))?)?;
    let initial = NativeGeometricGenerate::from_bytes(
        &fs::read(initial_cp.join("generate.bin"))?,
        &initial_binding,
    )?;
    if initial_binding != binding
        || initial.prototypes() != model.prototypes()
        || uint(&stage["evaluation"], "complete")? != 0
    {
        return Err(bad("requires balanced static-code completed failure; otherwise this discriminator is inapplicable"));
    }
    let exp = fs::read(cp.join("native/consumer/exp-q31.bin"))?;
    let mut pool = NativeVocabularyActions::new(binding.clone(), &exp)?;
    let refs = arr(&stage["evaluation"], "rows")?;
    let oracles = read(&fit.join("frozen-development-answer-oracles.json"))?;
    let oracle_rows = arr(&oracles, "cases")?;
    if refs.len() != 512 || oracle_rows.len() != refs.len() {
        return Err(bad("requires bound full512 construction panel"));
    }
    let mut frequency = BTreeMap::<u32, usize>::new();
    // Predeclare lane0; token selection uses only incumbent failed first targets.
    for (reference, oracle) in refs.iter().zip(oracle_rows) {
        deadline(start)?;
        let row = load_row(fit, reference)?;
        let first = arr(&row, "canonical")?
            .first()
            .ok_or_else(|| bad("canonical first absent"))?;
        let native = &first["native"];
        if !arr(native, "actual_prefix_ids")?.is_empty() {
            return Err(bad("first prefix nonempty"));
        }
        let (_, scores, ids, _) = replay(native, &model, &mut pool)?;
        if row["id"] != oracle["id"]
            || row["canonical_target_ids_labels_only"] != oracle["canonical_ids_labels_only"]
        {
            return Err(bad("frozen oracle identity/order/labels differ"));
        }
        let gold = u32::try_from(uint(first, "target_label_only")?)?;
        if !binding.admits_token(gold) {
            return Err(bad("gold token invalid"));
        }
        let best = pool
            .legal_token_ids()
            .iter()
            .copied()
            .fold(None, |best: Option<u32>, id| {
                Some(match best {
                    Some(b)
                        if scores[b as usize].clamp(-8i64 << 24, 8i64 << 24)
                            >= scores[id as usize].clamp(-8i64 << 24, 8i64 << 24) =>
                    {
                        b
                    }
                    _ => id,
                })
            })
            .ok_or_else(|| bad("legal domain empty"))?;
        if best != gold
            && uint(&native["pool"]["summary"], "chosen_token_id")? != u64::from(gold)
            && !ids.contains(&gold)
        {
            *frequency.entry(gold).or_default() += 1;
        }
    }
    let token = frequency
        .iter()
        .max_by(|(a, fa), (b, fb)| fa.cmp(fb).then_with(|| b.cmp(a)))
        .map(|(&t, _)| t)
        .ok_or_else(|| {
            bad("no failed first Generate-only target; static-code discriminator inapplicable")
        })?;
    let lane = 0usize;
    let incumbent = usize::from(model.prototypes()[token as usize * model.lanes() + lane]);
    fs::write(
        out.join("coordinate.json"),
        serde_json::to_vec_pretty(
            &json!({"token":token,"lane":lane,"incumbent":incumbent,"rule":"most frequent failed first pooled gold with a clipped Generate ranking deficit and absent Copy; lowest token ID frequency ties; lane0 fixed before alternatives","frequency":frequency,"input_generate_sha256":sha256_bytes(&bytes)}),
        )?,
    )?;
    let mut episodes = Vec::<Vec<Position>>::new();
    let mut baseline = 0.;
    let mut total_positions = 0;
    for reference in refs {
        deadline(start)?;
        let row = load_row(fit, reference)?;
        let labels = ints(&row["canonical_target_ids_labels_only"])?;
        let canonical = arr(&row, "canonical")?;
        if labels.is_empty() || labels.len() != canonical.len() || labels.len() > 128 {
            return Err(bad("canonical target shape outside contract"));
        }
        let mut ep = Vec::new();
        let mut ep_loss = 0.;
        for (t, p) in canonical.iter().enumerate() {
            deadline(start)?;
            if ints(&p["native"]["actual_prefix_ids"])? != labels[..t]
                || p["target_label_only"].as_i64() != Some(labels[t])
            {
                return Err(bad("teacher prefix or target order mismatch"));
            }
            let (s, gen, ids, copy) = replay(&p["native"], &model, &mut pool)?;
            let gold = u32::try_from(labels[t])?;
            let cache =
                pool.prepare_generate_substitution(gen, ids.clone(), copy.clone(), token, gold)?;
            let masses = cache.incumbent_mass();
            if masses.target_weight_q31 != uint(p, "native_target_mass")?
                || masses.total_weight_q31 != uint(p, "native_denominator")?
            {
                return Err(bad("canonical target mass differs"));
            }
            ep_loss += loss(masses.target_weight_q31, masses.total_weight_q31)?;
            let mut alternatives = [0; 120];
            model.code_conditional_scores_into(
                &s,
                lane,
                token as usize,
                &mut alternatives,
                &mut GenerateReadCounts::default(),
            )?;
            ep.push(Position {
                state: s,
                copy_ids: ids,
                copy,
                gold,
                alternatives,
                cache,
            });
        }
        let row_loss = ep_loss / labels.len() as f64;
        if row["native_equal_episode_ce"].as_f64() != Some(row_loss) {
            return Err(bad("baseline row CE replay differs"));
        }
        baseline += row_loss;
        total_positions += ep.len();
        episodes.push(ep);
    }
    baseline /= episodes.len() as f64;
    if stage["evaluation"]["native_equal_episode_ce"].as_f64() != Some(baseline) {
        return Err(bad("baseline whole-objective replay differs"));
    }
    let mut losses = [0.; 120];
    let mut fallbacks = [0usize; 120];
    for ep in &mut episodes {
        let mut row = [0.; 120];
        for p in ep.iter_mut() {
            deadline(start)?;
            for code in 0..120 {
                let m = pool.evaluate_generate_substitution(&mut p.cache, p.alternatives[code])?;
                row[code] += loss(m.target_weight_q31, m.total_weight_q31)?;
                fallbacks[code] += usize::from(m.used_full_reduction);
            }
        }
        for code in 0..120 {
            losses[code] += row[code] / ep.len() as f64;
        }
    }
    for v in &mut losses {
        *v /= episodes.len() as f64;
    }
    if losses[incumbent] != baseline {
        return Err(bad(
            "incumbent substitution fails exact whole-objective replay",
        ));
    }
    let chosen = best_code(&losses, incumbent);
    let improved = chosen != incumbent;
    let mut candidate = model.prototypes().to_vec();
    candidate[token as usize * model.lanes() + lane] = chosen as u8;
    let compiled = NativeGeometricGenerate::compile(
        &binding,
        model.lanes(),
        &candidate,
        model.packed_biases(),
        model.energy().clone(),
    )?;
    let candidate_bytes = compiled.to_bytes()?;
    fs::write(out.join("candidate-generate.bin"), &candidate_bytes)?;
    let reloaded = NativeGeometricGenerate::from_bytes(
        &fs::read(out.join("candidate-generate.bin"))?,
        &binding,
    )?;
    if reloaded.to_bytes()? != candidate_bytes
        || reloaded.energy() != model.energy()
        || reloaded.packed_biases() != model.packed_biases()
        || reloaded.prototypes() != candidate
    {
        return Err(bad("isolated candidate reload differs"));
    }
    let mut accepted_loss = 0.;
    let mut first_correct = 0;
    let mut first_correct_before = 0;
    for ep in &episodes {
        let mut row_loss = 0.;
        for (t, p) in ep.iter().enumerate() {
            deadline(start)?;
            let mut scores = vec![0; model.vocab_size()];
            reloaded.score_into(&p.state, &mut scores, &mut GenerateReadCounts::default())?;
            if scores[token as usize] != p.alternatives[chosen] {
                return Err(bad(
                    "accepted conditional score differs from independently reloaded scorer",
                ));
            }
            let trace = pool.reduce_trace(&scores, &p.copy_ids, &p.copy)?;
            let gold = trace
                .token_masses
                .iter()
                .find(|m| m.token_id == p.gold)
                .ok_or_else(|| bad("accepted gold mass absent"))?
                .weight_q31;
            row_loss += loss(gold, trace.summary.total_weight_q31)?;
            if t == 0 {
                first_correct += usize::from(trace.summary.chosen_token_id == p.gold);
                model.score_into(&p.state, &mut scores, &mut GenerateReadCounts::default())?;
                first_correct_before += usize::from(
                    pool.reduce_trace(&scores, &p.copy_ids, &p.copy)?
                        .summary
                        .chosen_token_id
                        == p.gold,
                );
            }
        }
        accepted_loss += row_loss / ep.len() as f64;
    }
    accepted_loss /= episodes.len() as f64;
    if accepted_loss != losses[chosen] {
        return Err(bad(
            "accepted full-reducer whole-objective differs from cached selection",
        ));
    }
    report_output::verify(fit)?;
    let alternatives:Vec<_>=(0..120).map(|code|json!({"code":code,"native_equal_episode_ce":losses[code],"changed_reference_full_reductions":fallbacks[code]})).collect();
    fs::write(
        out.join("report.json"),
        serde_json::to_vec_pretty(
            &json!({"schema":"uor-r4.geometric-generate-code-probe/1","status":"COMPLETED","scope":"one frozen-state supervised native prototype intervention on exposed construction panel; no optimizer resume/own-prefix generation/transfer or chat qualification","source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT"),"executable_sha256":sha256_bytes(&fs::read(std::env::current_exe()?)?),"input_report_sha256":sha256_bytes(&fs::read(fit.join("report.json"))?),"input_manifest_sha256":sha256_bytes(&fs::read(fit.join("manifest.json"))?),"config_sha256":sha256_bytes(&config_bytes),"panel_sha256":data_hashes,"cases":episodes.len(),"positions":total_positions,"coordinate":{"token":token,"lane":lane,"incumbent":incumbent,"chosen":chosen},"baseline_native_equal_episode_ce":baseline,"candidate_native_equal_episode_ce":accepted_loss,"strict_improvement":improved,"alternatives":alternatives,"candidate_generate_sha256":sha256_bytes(&candidate_bytes),"independent_candidate_full_objective_replay":true,"first_teacher_forced_correct_before":first_correct_before,"first_teacher_forced_correct_after":first_correct,"elapsed_seconds":start.elapsed().as_secs_f64(),"execution_ceiling_seconds":240}),
        )?,
    )?;
    Ok(())
}
fn main() -> Result<()> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() != 3 {
        return Err(bad(
            "usage: geometric-generate-code-probe FIT_ROOT FIT_CONFIG REPORT_OUTPUT",
        ));
    }
    let fit = PathBuf::from(&args[0]);
    let config = PathBuf::from(&args[1]);
    let out = PathBuf::from(&args[2]);
    if !fit.is_dir() || !config.is_file() {
        return Err(bad("fit/config absent"));
    }
    let prospective = output_support::prospective_output(&out)?;
    for input in [&fit, &config] {
        let input = fs::canonicalize(input)?;
        if prospective.starts_with(&input) || input.starts_with(&prospective) {
            return Err(bad("output/input overlap"));
        }
        if input
            .ancestors()
            .any(|a| a.join("manifest.json").is_file() && prospective.starts_with(a))
        {
            return Err(bad("output beneath sealed input"));
        }
    }
    report_output::claim(&out)?;
    let result = probe(&fit, &config, &out, Instant::now());
    if let Err(e) = &result {
        fs::write(
            out.join("failure.json"),
            serde_json::to_vec_pretty(
                &json!({"status":"FAILED","error":e.to_string(),"scope":"instrument/execution failure; not a model-quality verdict"}),
            )?,
        )?;
    }
    report_output::seal(&out)?;
    report_output::verify(&out)?;
    result
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn incumbent_retained_on_ties_and_only_strict_improvement_selected() {
        let mut v = [3.; 120];
        assert_eq!(best_code(&v, 17), 17);
        v[2] = 2.;
        v[4] = 2.;
        assert_eq!(best_code(&v, 17), 2);
        assert_eq!(best_code(&v, 4), 4);
    }
    #[test]
    fn teacher_objective_positive_mass_contract_and_relative_paths() -> Result<()> {
        assert_eq!(loss(2, 4)?, -(0.5f64).ln());
        assert!(loss(0, 4).is_err());
        assert!(loss(5, 4).is_err());
        assert!(relative(Path::new("/input"), "../elsewhere").is_err());
        assert!(relative(Path::new("/input"), "/absolute").is_err());
        Ok(())
    }
}
