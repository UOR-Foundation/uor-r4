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
fn probe(
    fit: &Path,
    config_path: &Path,
    out: &Path,
    start: Instant,
    state_ceiling: bool,
    field_credit: bool,
) -> Result<()> {
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
        || (!(state_ceiling || field_credit)
            && (initial.prototypes() != model.prototypes()
                || uint(&stage["evaluation"], "complete")? != 0))
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
    if field_credit {
        #[cfg(feature = "cuda")]
        return field_credit_audit(
            fit,
            out,
            start,
            stage,
            refs,
            oracle_rows,
            &model,
            &mut pool,
            &config_bytes,
        );
        #[cfg(not(feature = "cuda"))]
        return Err(bad(
            "field-credit requires a CUDA build and device; no CPU fallback",
        ));
    }
    if state_ceiling {
        return state_ceiling_audit(
            fit,
            out,
            start,
            &report,
            stage,
            refs,
            oracle_rows,
            &binding,
            &model,
            &mut pool,
            &config_bytes,
        );
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

/// Disjoint pairs and isolated unary lanes separate exactly. Overlapping edges
/// are rejected; this is not a heuristic optimizer for a coupled graph.
fn relative_ceiling(
    energy: &uor_r4_core::native_geometric::learner::integrated_attention::geometry::EnergyTables,
    lanes: usize,
) -> Result<(Vec<u8>, i64)> {
    let mut used = vec![false; lanes];
    let mut roots = vec![0u8; lanes];
    let mut sum = 0i64;
    for (edge, pair) in energy.edges().iter().enumerate() {
        let a = usize::from(pair.left);
        let b = usize::from(pair.right);
        if a >= lanes || b >= lanes || a == b || used[a] || used[b] {
            return Err(bad("state ceiling requires disjoint valid pair edges"));
        }
        used[a] = true;
        used[b] = true;
        let mut best = i64::MIN;
        // Lexicographic relative-code ties choose the smallest (left,right).
        for left in 0..120u8 {
            for right in 0..120u8 {
                let v = i64::from(energy.get_unary(pair.left, left)?)
                    + i64::from(energy.get_unary(pair.right, right)?)
                    + i64::from(energy.get_pair(edge, left, right)?);
                if v > best {
                    best = v;
                    roots[a] = left;
                    roots[b] = right;
                }
            }
        }
        sum = sum
            .checked_add(best)
            .ok_or_else(|| bad("ceiling overflow"))?;
    }
    for lane in 0..lanes {
        if !used[lane] {
            let mut best = i64::MIN;
            for root in 0..120u8 {
                let v = i64::from(energy.get_unary(lane as u8, root)?);
                if v > best {
                    best = v;
                    roots[lane] = root;
                }
            }
            sum = sum
                .checked_add(best)
                .ok_or_else(|| bad("ceiling overflow"))?;
        }
    }
    Ok((roots, sum))
}
fn state_ceiling_audit(
    fit: &Path,
    out: &Path,
    start: Instant,
    report: &Value,
    stage: &Value,
    refs: &[Value],
    oracles: &[Value],
    binding: &SourceActionBinding,
    model: &NativeGeometricGenerate,
    pool: &mut NativeVocabularyActions,
    config_bytes: &[u8],
) -> Result<()> {
    let (relative, factor_max) = relative_ceiling(model.energy(), model.lanes())?;
    let mut rows = Vec::with_capacity(refs.len());
    let mut before = 0usize;
    let mut after = 0usize;
    for (reference, oracle) in refs.iter().zip(oracles) {
        deadline(start)?;
        let row = load_row(fit, reference)?;
        let first = arr(&row, "canonical")?
            .first()
            .ok_or_else(|| bad("first canonical absent"))?;
        let native = &first["native"];
        if !arr(native, "actual_prefix_ids")?.is_empty()
            || row["id"] != oracle["id"]
            || row["canonical_target_ids_labels_only"] != oracle["canonical_ids_labels_only"]
        {
            return Err(bad("first prefix/oracle identity differs"));
        }
        let (parent_state, _, ids, copy) = replay(native, model, pool)?;
        // Labels enter only after the complete incumbent native replay.
        let gold = u32::try_from(uint(first, "target_label_only")?)?;
        if !binding.admits_token(gold)
            || oracle["canonical_ids_labels_only"][0].as_u64() != Some(u64::from(gold))
        {
            return Err(bad("gold label identity differs"));
        }
        let mut witness = Vec::with_capacity(model.lanes());
        for (lane, &r) in relative.iter().enumerate() {
            let prototype = model.prototypes()[gold as usize * model.lanes() + lane];
            // inv(state)*prototype=r implies state=prototype*inv(r).
            let state = model
                .algebra()
                .compose(prototype, model.algebra().inverse(r)?)?;
            witness.push(H4Code::try_from(state)?);
        }
        let max_q24 = (factor_max + i64::from(model.token_bias(gold as usize)?))
            .checked_mul(
                1i64 << uor_r4_core::native_geometric::learner::geometric_generate::SCORE_SHIFT,
            )
            .ok_or_else(|| bad("ceiling Q24 overflow"))?;
        let mut scores = vec![0i64; model.vocab_size()];
        model.score_into(&witness, &mut scores, &mut GenerateReadCounts::default())?;
        if scores[gold as usize] != max_q24 {
            return Err(bad("native witness does not attain computed gold maximum"));
        }
        // Copy is intentionally frozen: this is a readout feasibility witness,
        // not a recomputed attention/transition or a deployable state intervention.
        let trace = pool.reduce_trace(&scores, &ids, &copy)?;
        let old = u32::try_from(uint(&native["pool"]["summary"], "chosen_token_id")?)?;
        before += usize::from(old == gold);
        after += usize::from(trace.summary.chosen_token_id == gold);
        rows.push(json!({"id":row["id"],"gold_label_only":gold,"parent_state":parent_state.iter().map(|x|x.index()).collect::<Vec<_>>(),"witness_state":witness.iter().map(|x|x.index()).collect::<Vec<_>>(),"max_gold_raw_q24":max_q24,"parent_chosen_token_id":old,"witness_chosen_token_id":trace.summary.chosen_token_id,"witness_pool":trace.summary,"gold_wins":trace.summary.chosen_token_id==gold}));
    }
    report_output::verify(fit)?;
    fs::write(
        out.join("report.json"),
        serde_json::to_vec_pretty(
            &json!({"schema":"uor-r4.geometric-generate-state-ceiling/1","status":"COMPLETED","scope":"gold-conditioned oracle readout feasibility diagnostic; arbitrary state per first position; frozen factual Copy scores; no learned transition/attention, candidate artifact, update, own-prefix or transfer claim","source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT"),"executable_sha256":sha256_bytes(&fs::read(std::env::current_exe()?)?),"input_report_sha256":sha256_bytes(&fs::read(fit.join("report.json"))?),"input_manifest_sha256":sha256_bytes(&fs::read(fit.join("manifest.json"))?),"config_sha256":sha256_bytes(config_bytes),"input_generate_sha256":stage["checkpoint"]["generate_sha256"],"fit_seed":report["seed"],"cases":rows.len(),"updates":0,"relative_code_maximizer":relative,"factor_max_unshifted":factor_max,"first_teacher_forced_correct_parent":before,"first_teacher_forced_correct_max_gold_witness":after,"limitation":"A losing max-gold witness does not prove that no state can win; ceiling is only an absolute target-score bound, not maximum target margin or mass","rows":rows,"elapsed_seconds":start.elapsed().as_secs_f64()}),
        )?,
    )?;
    Ok(())
}

#[cfg(feature = "cuda")]
fn field_credit_audit(
    fit: &Path,
    out: &Path,
    start: Instant,
    stage: &Value,
    refs: &[Value],
    oracles: &[Value],
    model: &NativeGeometricGenerate,
    pool: &mut NativeVocabularyActions,
    config_bytes: &[u8],
) -> Result<()> {
    use candle_core::{Device, Tensor};
    use uor_r4_training::geometric_generate_learning::{
        vocabulary_marginal_loss, GenerateLearningWeights,
    };
    let device = Device::new_cuda(0)?;
    let cp = fit.join("checkpoint-0128");
    let g = GenerateLearningWeights::from_native(pool.binding().clone(), model, &device)?;
    let metadata_bytes = fs::read(cp.join("generate-source/metadata.json"))?;
    let metadata: Value = serde_json::from_slice(&metadata_bytes)?;
    if metadata["tokenizer_sha256"].as_str() != Some(g.binding().tokenizer_sha256()) {
        return Err(bad("field master tokenizer differs"));
    }
    let params = g.parameters();
    let mut coefficient_masters = BTreeMap::<String, Vec<f32>>::new();
    if metadata["parameters"].as_object().map(|v| v.len()) != Some(params.len()) {
        return Err(bad("field master inventory differs"));
    }
    for (name, var) in &params {
        let raw = fs::read(cp.join("generate-source").join(format!("{name}.f32le")))?;
        let m = &metadata["parameters"][name];
        if raw.len() != var.elem_count() * 4
            || m["bytes"].as_u64() != Some(raw.len() as u64)
            || m["sha256"].as_str() != Some(sha256_bytes(&raw).as_str())
            || m["shape"] != json!(var.dims())
        {
            return Err(bad("source master hash/shape differs"));
        }
        let values = raw
            .chunks_exact(4)
            .map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]]))
            .collect::<Vec<_>>();
        if values.iter().any(|v| !v.is_finite()) {
            return Err(bad("nonfinite source master"));
        }
        if name != "generate.prototype_choices" {
            coefficient_masters.insert(name.clone(), values.clone());
        }
        var.set(&Tensor::from_vec(values, var.shape(), &device)?)?;
    }
    if g.export_native()?.to_bytes()? != model.to_bytes()? {
        return Err(bad("restored master native export differs from parent"));
    }
    let prepared = g.prepare_native()?;
    let names = ["generate.unary", "generate.pair", "generate.bias"];
    // Keep all live adjoints and phase accumulation on CUDA. Only completed
    // aggregate field vectors are downloaded for the offline diagnostic.
    let mut sums: Vec<BTreeMap<String, Tensor>> = (0..4)
        .map(|_| {
            names
                .iter()
                .map(|name| {
                    Ok((
                        (*name).to_owned(),
                        Tensor::zeros(params[*name].shape(), candle_core::DType::F32, &device)?,
                    ))
                })
                .collect::<candle_core::Result<BTreeMap<String, Tensor>>>()
        })
        .collect::<candle_core::Result<Vec<_>>>()?;
    let mut counts = [0usize; 3];
    let mut phase_ce = [0f64; 3];
    let mut episode_weight = [0f64; 3];
    let mut actual_packet_parity = [false; 3];
    for (reference, oracle) in refs.iter().zip(oracles) {
        if start.elapsed().as_secs() >= 900 {
            return Err(bad("field credit 900-second ceiling"));
        }
        let row = load_row(fit, reference)?;
        let canonical = arr(&row, "canonical")?;
        let labels = ints(&row["canonical_target_ids_labels_only"])?;
        if row["id"] != oracle["id"]
            || row["canonical_target_ids_labels_only"] != oracle["canonical_ids_labels_only"]
            || canonical.is_empty()
            || canonical.len() != labels.len()
            || labels.len() > 128
        {
            return Err(bad("field credit oracle/target identity differs"));
        }
        let weight = 1. / refs.len() as f64 / labels.len() as f64;
        for (t, position) in canonical.iter().enumerate() {
            if start.elapsed().as_secs() >= 900 {
                return Err(bad("field credit 900-second ceiling"));
            }
            if position["target_label_only"].as_i64() != Some(labels[t])
                || ints(&position["native"]["actual_prefix_ids"])? != labels[..t]
            {
                return Err(bad("field credit canonical prefix/label mismatch"));
            }
            let (s, gen, ids, copy) = replay(&position["native"], model, pool)?;
            let trace = pool.reduce_trace(&gen, &ids, &copy)?;
            let target = u32::try_from(labels[t])?;
            let phase = if t == 0 {
                0
            } else if ids.contains(&target) {
                1
            } else {
                2
            };
            let generated = g.forward_prepared_coefficients_only(&prepared, &s)?;
            if generated.scores_q24 != gen {
                return Err(bad("restored graph native scores differ"));
            }
            let copy_graph = Tensor::from_vec(
                copy.iter()
                    .map(|&v| (v as f64 / 16_777_216.) as f32)
                    .collect::<Vec<_>>(),
                copy.len(),
                &device,
            )?;
            let native_mass = trace
                .token_masses
                .iter()
                .find(|m| m.token_id == target)
                .ok_or_else(|| bad("gold mass absent"))?
                .weight_q31;
            if uint(position, "native_target_mass")? != native_mass
                || uint(position, "native_denominator")? != trace.summary.total_weight_q31
            {
                return Err(bad("saved target/denominator differs"));
            }
            phase_ce[phase] += loss(native_mass, trace.summary.total_weight_q31)? * weight;
            episode_weight[phase] += weight;
            let l =
                vocabulary_marginal_loss(&trace, &generated.raw_scores, Some(&copy_graph), target)?
                    .affine(weight, 0.)?;
            let grads = l.backward()?;
            if !actual_packet_parity[phase] {
                let mut onehot = vec![0f32; model.lanes() * 120];
                for (lane, code) in s.iter().enumerate() {
                    onehot[lane * 120 + usize::from(code.index())] = 1.;
                }
                let full = g.forward_prepared_state_choices(
                    &prepared,
                    &s,
                    &Tensor::from_vec(onehot, (model.lanes(), 120), &device)?,
                )?;
                if full.scores_q24 != generated.scores_q24 {
                    return Err(bad("actual packet full/coeff-only hard scores differ"));
                }
                let full_loss =
                    vocabulary_marginal_loss(&trace, &full.raw_scores, Some(&copy_graph), target)?
                        .affine(weight, 0.)?;
                if full_loss.to_scalar::<f32>()? != l.to_scalar::<f32>()? {
                    return Err(bad("actual packet full/coeff-only loss differs"));
                }
                let full_grads = full_loss.backward()?;
                for name in names {
                    let old = grads
                        .get(params[name].as_tensor())
                        .ok_or_else(|| bad("coefficient parity gradient absent"))?;
                    let other = full_grads
                        .get(params[name].as_tensor())
                        .ok_or_else(|| bad("full parity gradient absent"))?;
                    let error = (old - other)?.abs()?.max_all()?.to_scalar::<f32>()?;
                    if !error.is_finite() || error > 1e-6 {
                        return Err(bad(
                            "actual packet coefficient adjoints exceed1e-6 parity envelope",
                        ));
                    }
                }
                actual_packet_parity[phase] = true;
            }
            if grads
                .get(params["generate.prototype_choices"].as_tensor())
                .is_some()
            {
                return Err(bad("coefficient replay unexpectedly credits prototypes"));
            }
            for name in names {
                let grad = grads
                    .get(params[name].as_tensor())
                    .ok_or_else(|| bad("field adjoint disconnected"))?;
                if !grad.device().is_cuda() {
                    return Err(bad("field adjoint CPU fallback"));
                }
                for group in [phase, 3] {
                    let next = if let Some(old) = sums[group].get(name) {
                        (old + grad)?.detach()
                    } else {
                        grad.detach()
                    };
                    sums[group].insert(name.into(), next);
                }
            }
            counts[phase] += 1;
        }
    }
    device.synchronize()?;
    let mut families = BTreeMap::new();
    let mut aggregate_download_bytes = 0usize;
    for name in names {
        let vectors = sums
            .iter()
            .map(|m| m[name].flatten_all()?.to_vec1::<f32>())
            .collect::<candle_core::Result<Vec<_>>>()?;
        if vectors.iter().flatten().any(|v| !v.is_finite()) {
            return Err(bad("nonfinite phase adjoints"));
        }
        aggregate_download_bytes += vectors.iter().map(|v| v.len() * 4).sum::<usize>();
        let body = vectors[1]
            .iter()
            .zip(&vectors[2])
            .map(|(&a, &b)| f64::from(a) + f64::from(b))
            .collect::<Vec<_>>();
        let first = vectors[0].iter().map(|&v| f64::from(v)).collect::<Vec<_>>();
        let dot = first.iter().zip(&body).map(|(a, b)| a * b).sum::<f64>();
        let nf = first.iter().map(|v| v * v).sum::<f64>().sqrt();
        let nb = body.iter().map(|v| v * v).sum::<f64>().sqrt();
        let opposite = first
            .iter()
            .zip(&body)
            .filter(|(a, b)| **a * **b < 0.)
            .count();
        let body_overrides_opposed_first = first
            .iter()
            .zip(&body)
            .filter(|(a, b)| **a * **b < 0. && b.abs() > a.abs())
            .count();
        let first_dot_total = first
            .iter()
            .zip(&vectors[3])
            .map(|(a, &b)| a * f64::from(b))
            .sum::<f64>();
        let first_wants_raise_body_wants_lower = first
            .iter()
            .zip(&body)
            .filter(|(a, b)| **a < 0. && **b > 0.)
            .count();
        let masters = &coefficient_masters[name];
        let quantum_margins = masters
            .iter()
            .map(|&v| 0.125f64 - (f64::from(v) - f64::from((v * 4.).round() * 0.25)).abs())
            .collect::<Vec<_>>();
        let min_quantum_margin = quantum_margins
            .iter()
            .copied()
            .fold(f64::INFINITY, f64::min);
        let saturated = masters.iter().filter(|v| v.abs() == 1.75).count();
        let descent_outward =
            |v: f32, grad: f64| (v == 1.75 && grad < 0.) || (v == -1.75 && grad > 0.);
        let outward_first = masters
            .iter()
            .zip(&first)
            .filter(|(v, g)| descent_outward(**v, **g))
            .count();
        let outward_body = masters
            .iter()
            .zip(&body)
            .filter(|(v, g)| descent_outward(**v, **g))
            .count();
        let outward_total = masters
            .iter()
            .zip(&vectors[3])
            .filter(|(v, g)| descent_outward(**v, f64::from(**g)))
            .count();
        let projected_first_dot_total = masters
            .iter()
            .zip(&first)
            .zip(&vectors[3])
            .map(|((&v, &a), &b)| {
                if descent_outward(v, f64::from(b)) {
                    0.
                } else {
                    a * f64::from(b)
                }
            })
            .sum::<f64>();
        let directed_boundary = |grad: &[f64]| {
            let mut distances = Vec::new();
            let mut projection_terminal = 0usize;
            let mut zero = 0usize;
            for (&v, &g) in masters.iter().zip(grad) {
                if g == 0. {
                    zero += 1;
                    continue;
                }
                let code = f64::from((v * 4.).round() * 0.25);
                if (g < 0. && code == 1.75) || (g > 0. && code == -1.75) {
                    projection_terminal += 1;
                    continue;
                }
                let boundary = if g < 0. { code + 0.125 } else { code - 0.125 };
                distances.push((boundary - f64::from(v)).abs());
            }
            distances.sort_by(f64::total_cmp);
            json!({"supported_nonzero_entries":distances.len(),"zero_gradient_entries":zero,"no_next_code_in_descent_direction":projection_terminal,"minimum_distance":distances.first(),"median_distance":distances.get(distances.len()/2),"scope":"distance to quarter-round boundary, not measured native crossing or Adam step; tie crossing convention remains exporter-owned"})
        };
        let first_boundary = directed_boundary(&first);
        let body_boundary = directed_boundary(&body);

        let residual = first
            .iter()
            .zip(&body)
            .zip(&vectors[3])
            .map(|((a, b), &c)| (a + b - f64::from(c)).abs())
            .fold(0., f64::max);
        let scale = vectors[3].iter().map(|v| v.abs() as f64).fold(0., f64::max);
        if residual > 1e-5 + 1e-4 * scale {
            return Err(bad(
                "phase adjoint reconstruction exceeds F32 accumulation envelope",
            ));
        }
        for (i, v) in vectors.iter().enumerate() {
            let bytes = v.iter().flat_map(|x| x.to_le_bytes()).collect::<Vec<_>>();
            fs::write(out.join(format!("{name}.phase-{i}.f32le")), &bytes)?;
        }
        families.insert(name, json!({"elements":first.len(),"first_l2":nf,"body_l2":nb,"first_dot_body":dot,"cosine":if nf>0.&&nb>0.{Some(dot/nf/nb)}else{None},"opposite_sign_entries":opposite,"body_overrides_opposed_first_entries":body_overrides_opposed_first,"first_wants_raise_body_wants_lower_entries":first_wants_raise_body_wants_lower,"first_dot_total_gradient":first_dot_total,"infinitesimal_first_loss_change_under_unclipped_total_gradient":-first_dot_total,"direction_scope":"unpreconditioned infinitesimal negative-gradient direction only; not Adam/no measured update","minimum_master_distance_to_quarter_round_boundary":min_quantum_margin,"master_projection_saturated_entries":saturated,"outward_descent_at_projection_endpoint":{"first":outward_first,"body":outward_body,"total":outward_total},"infinitesimal_first_loss_change_under_projected_total_gradient":-projected_first_dot_total,"first_descent_quarter_boundary":first_boundary,"body_descent_quarter_boundary":body_boundary,"max_phase_reconstruction_absolute_error":residual,"total_gradient_max_absolute":scale,"binary_phases":["first","body-Copy-present","body-Copy-absent","all"]}));
    }
    let total_ce = phase_ce.iter().sum::<f64>();
    let expected = stage["evaluation"]["native_equal_episode_ce"]
        .as_f64()
        .ok_or_else(|| bad("parent CE absent"))?;
    if (total_ce - expected).abs() > 1e-10 {
        return Err(bad("phase native objective does not reconstruct parent"));
    }
    report_output::verify(fit)?;
    fs::write(
        out.join("report.json"),
        serde_json::to_vec_pretty(
            &json!({"schema":"uor-r4.geometric-generate-field-credit/1","status":"COMPLETED","source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT"),"executable_sha256":sha256_bytes(&fs::read(std::env::current_exe()?)?),"input_report_sha256":sha256_bytes(&fs::read(fit.join("report.json"))?),"input_manifest_sha256":sha256_bytes(&fs::read(fit.join("manifest.json"))?),"source_master_metadata_sha256":sha256_bytes(&metadata_bytes),"config_sha256":sha256_bytes(config_bytes),"input_generate_sha256":stage["checkpoint"]["generate_sha256"],"device":"cuda:0","updates":0,"episodes":refs.len(),"positions_by_phase":counts,"actual_checkpoint_full_vs_coefficients_only_adjoint_parity_by_phase":actual_packet_parity,"native_ce_contribution_by_phase":phase_ce,"objective_weight_by_phase":episode_weight,"families":families,"aggregate_adjoint_download_bytes":aggregate_download_bytes,"elapsed_seconds":start.elapsed().as_secs_f64(),"scope":"selected Generate coefficient adjoints of existing anchored native marginal loss at restored source masters; frozen factual states/prototypes/Copy; no recurrence or prototype credit, optimizer resume, training, curriculum adoption or generated-reply claim","decision":"opposing adjoints are local source-gradient evidence, not proof that a reweighted fit improves native complete replies"}),
        )?,
    )?;
    Ok(())
}

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() != 3
        && !(args.len() == 4 && (args[3] == "--state-ceiling" || args[3] == "--field-credit"))
    {
        return Err(bad(
            "usage: geometric-generate-code-probe FIT_ROOT FIT_CONFIG REPORT_OUTPUT [--state-ceiling|--field-credit]",
        ));
    }
    let fit = PathBuf::from(&args[0]);
    let config = PathBuf::from(&args[1]);
    let out = PathBuf::from(&args[2]);
    if !fit.is_dir() || !config.is_file() {
        return Err(bad("fit/config absent"));
    }
    #[cfg(not(feature = "cuda"))]
    if args.len() == 4 && args[3] == "--field-credit" {
        return Err(bad("field-credit requires a CUDA build; no CPU fallback"));
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
    let result = probe(
        &fit,
        &config,
        &out,
        Instant::now(),
        args.len() == 4 && args[3] == "--state-ceiling",
        args.len() == 4 && args[3] == "--field-credit",
    );
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
    fn disjoint_ceiling_exact_pair_and_isolated_lane_with_smallest_ties() -> Result<()> {
        use uor_r4_core::native_geometric::learner::integrated_attention::geometry::{
            EnergyTables, LanePair,
        };
        let mut e = EnergyTables::zeroed(3, vec![LanePair { left: 0, right: 1 }])?;
        e.set_unary(0, 7, 3)?;
        e.set_unary(1, 9, 2)?;
        e.set_pair(0, 7, 9, 4)?;
        e.set_unary(2, 11, 5)?;
        e.set_unary(2, 12, 5)?;
        let (roots, maximum) = relative_ceiling(&e, 3)?;
        assert_eq!(roots, vec![7, 9, 11]);
        assert_eq!(maximum, 14);
        let overlapping = EnergyTables::zeroed(
            3,
            vec![
                LanePair { left: 0, right: 1 },
                LanePair { left: 1, right: 2 },
            ],
        )?;
        assert!(relative_ceiling(&overlapping, 3).is_err());
        assert!(relative_ceiling(&e, 1).is_err());
        Ok(())
    }
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
