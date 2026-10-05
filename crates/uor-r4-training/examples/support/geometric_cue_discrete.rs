//! Discrete cue learning: ordinary-gradient proposals, actual-native decisions.
use super::*;
use uor_r4_integer::geometric_cue_carrier::CueAngularQ4;
use uor_r4_integer::geometric_potential_q4::unpack_coefficients;

const ACCEPT_EPSILON: f64 = 1e-9;

#[derive(Default)]
struct HeadRejections {
    packed_sha256: String,
    indices: BTreeSet<usize>,
}
impl HeadRejections {
    fn at_head(&mut self, packed: &[u8]) {
        let head = sha256_bytes(packed);
        if self.packed_sha256 != head {
            self.packed_sha256 = head;
            self.indices.clear();
        }
    }
}
fn improves(incumbent: f64, proposal: Option<f64>) -> bool {
    incumbent.is_finite()
        && proposal.is_some_and(|v| v.is_finite() && v < incumbent - ACCEPT_EPSILON)
}
fn next_coordinate(
    values: &[i8],
    gradients: &[f32],
    rejected: &BTreeSet<usize>,
) -> Result<Option<(usize, i8, f32)>> {
    if values.len() != gradients.len() || gradients.iter().any(|x| !x.is_finite()) {
        return Err(invalid("discrete gradient shape/nonfinite contract differs").into());
    }
    let mut chosen: Option<(usize, i8, f32)> = None;
    for (index, (&quarter, &gradient)) in values.iter().zip(gradients).enumerate() {
        if !(-7..=7).contains(&quarter) {
            return Err(invalid("discrete incumbent quarter is illegal").into());
        }
        if rejected.contains(&index) || gradient == 0. {
            continue;
        }
        let step = if gradient > 0. { -1 } else { 1 };
        if !quarter
            .checked_add(step)
            .is_some_and(|x| (-7..=7).contains(&x))
        {
            continue;
        }
        // Traversal order supplies the lower-index tie breaker.
        if chosen.is_none_or(|(_, _, old)| gradient.abs() > old.abs()) {
            chosen = Some((index, step, gradient));
        }
    }
    Ok(chosen)
}

// Geometry-only analysis of the first read: no target, role or correctness field
// is consulted. Evaluation is classified only after model selection freezes.
fn first_read_tuples(canonical: &Value) -> Result<Vec<(String, Vec<Vec<Option<usize>>>)>> {
    let mut out = Vec::new();
    for row in canonical["rows"]
        .as_array()
        .ok_or_else(|| invalid("coverage rows absent"))?
    {
        let id = row["id"]
            .as_str()
            .ok_or_else(|| invalid("coverage ID absent"))?;
        let lanes = row["tokens"]
            .as_array()
            .and_then(|xs| xs.first())
            .and_then(|t| t["cue_carrier"]["angular_indices"].as_array())
            .ok_or_else(|| invalid("coverage initial angular indices absent"))?;
        let width = lanes
            .first()
            .and_then(Value::as_array)
            .map(Vec::len)
            .ok_or_else(|| invalid("coverage candidate width absent"))?;
        let mut tuples = vec![Vec::new(); width];
        for lane in lanes {
            let bins = lane
                .as_array()
                .ok_or_else(|| invalid("coverage lane absent"))?;
            if bins.len() != width {
                return Err(invalid("coverage lane width differs").into());
            }
            for (tuple, bin) in tuples.iter_mut().zip(bins) {
                tuple.push(if bin.is_null() {
                    None
                } else {
                    Some(usize::try_from(
                        bin.as_u64()
                            .filter(|b| *b < 120)
                            .ok_or_else(|| invalid("coverage angular bin invalid"))?,
                    )?)
                });
            }
        }
        tuples.retain(|t| t.iter().any(Option::is_some));
        if tuples.is_empty() {
            return Err(invalid("coverage has no observed source geometry").into());
        }
        out.push((id.to_owned(), tuples));
    }
    Ok(out)
}

fn coverage_strata(training: &Value, evaluation: &Value) -> Result<Value> {
    let train = first_read_tuples(training)?;
    let eval = first_read_tuples(evaluation)?;
    let mut bins = BTreeSet::new();
    let mut joint = BTreeSet::new();
    for (_, tuples) in &train {
        for tuple in tuples {
            for (lane, bin) in tuple.iter().enumerate() {
                if let Some(bin) = bin {
                    bins.insert((lane, *bin));
                }
            }
            joint.insert(tuple.clone());
        }
    }
    let mut rows = Vec::new();
    for (id, tuples) in eval {
        let all_bins_supported = tuples.iter().all(|tuple| {
            tuple
                .iter()
                .enumerate()
                .all(|(lane, bin)| bin.is_none_or(|b| bins.contains(&(lane, b))))
        });
        let all_joint_seen = tuples.iter().all(|tuple| joint.contains(tuple));
        rows.push(
            json!({"id":id,"all_source_constituent_bins_supported":all_bins_supported,
            "all_source_joint_tuples_seen":all_joint_seen,"stratum":if !all_bins_supported {
                "unsupported-constituent-bin"
            } else if all_joint_seen { "seen-joint-geometric-tuples" }
            else { "covered-bins/new-joint-tuple" }}),
        );
    }
    Ok(
        json!({"training_rows":train.len(),"training_lane_bin_union":bins,
        "training_distinct_joint_tuples":joint.len(),"evaluation_rows":rows,
        "scope":"all observed initial-read source tuples; canonical token0 only; no labels/outcomes or filtering; analysis after selection, not a model-selection criterion"}),
    )
}

// These labels only score already-produced canonical predictions. No generation
// is fabricated for trials, and this summary never ranks or accepts a proposal.
fn source_summary(
    panel: &Path,
    episodes: &[Episode],
    canonical: &Value,
    integer: &IntegerRealizer,
) -> Result<Value> {
    let context = read_json(&panel.join("context-data.json"))?;
    let labels = context["cases"]
        .as_array()
        .ok_or_else(|| invalid("discrete truth metadata absent"))?;
    let rows = canonical["rows"]
        .as_array()
        .ok_or_else(|| invalid("discrete canonical rows absent"))?;
    if labels.len() != episodes.len() || rows.len() != episodes.len() {
        return Err(invalid("discrete summary count differs").into());
    }
    let mut summaries = Vec::new();
    let mut pairs = BTreeMap::<String, Vec<bool>>::new();
    let mut source_correct = 0;
    for ((label, row), episode) in labels.iter().zip(rows).zip(episodes) {
        if label["id"] != row["id"] || row["id"] != episode.packet.id {
            return Err(invalid("discrete summary IDs differ").into());
        }
        let relation = match label["query_role"].as_str() {
            Some("job") => 1,
            Some("where") => 2,
            _ => return Err(invalid("discrete summary query role invalid").into()),
        };
        let expected = episode
            .packet
            .segments
            .iter()
            .filter_map(|s| match s {
                Segment::Source {
                    record,
                    commit,
                    relation: r,
                    ..
                } if *r == relation => Some(json!({"record":record,"commit":commit})),
                _ => None,
            })
            .collect::<Vec<_>>();
        if expected.len() != 1 {
            return Err(invalid("discrete scorer requires unique truthful role source").into());
        }
        let tokens = row["tokens"]
            .as_array()
            .ok_or_else(|| invalid("discrete summary tokens absent"))?;
        let first = tokens
            .first()
            .ok_or_else(|| invalid("discrete first prediction absent"))?;
        let actual = match first["source_end"]["selected_bank_index"].as_u64() {
            Some(i) => {
                let mapping = first["native"]["candidate_mapping"]
                    .as_array()
                    .ok_or_else(|| invalid("discrete native map absent"))?;
                let entry = mapping
                    .get(i as usize)
                    .ok_or_else(|| invalid("discrete selected bank index invalid"))?;
                if entry["bank_index"] != i {
                    return Err(invalid("discrete bank ordinal differs").into());
                }
                let o = &entry["occurrence"];
                if !o["record"].is_u64() || !o["commit"].is_u64() {
                    return Err(invalid("discrete typed source identity absent").into());
                }
                json!({"record":o["record"],"commit":o["commit"]})
            }
            None => Value::Null,
        };
        let correct = actual == expected[0];
        source_correct += usize::from(correct);
        let pair = label["pair_id"]
            .as_str()
            .ok_or_else(|| invalid("discrete pair ID absent"))?;
        pairs.entry(pair.into()).or_default().push(correct);
        let mut payload_sum = 0.;
        let mut terminal_sum = 0.;
        let mut payload_positions = 0;
        let mut terminal_positions = 0;
        let mut payload_zero = 0;
        let mut terminal_zero = 0;
        for token in tokens {
            let target = token["target_label_only_after_read"]
                .as_u64()
                .ok_or_else(|| invalid("discrete target audit absent"))?;
            let terminal = target == u64::from(integer.binding().period_token_id())
                || target == u64::from(integer.binding().eos_token_id());
            if terminal {
                terminal_positions += 1;
            } else {
                payload_positions += 1;
            }
            if let Some(v) = token["native_ce"].as_f64().filter(|v| v.is_finite()) {
                if terminal {
                    terminal_sum += v;
                } else {
                    payload_sum += v;
                }
            } else if terminal {
                terminal_zero += 1;
            } else {
                payload_zero += 1;
            }
        }
        summaries.push(json!({"id":row["id"],"pair_id":pair,"query_role":label["query_role"],"expected_source_labels_only":expected[0],"actual_first_canonical_factual_source":actual,"first_canonical_source_correct":correct,"canonical_first_chosen_token_id":first["native"]["actions"]["chosen_token_id"],"native_mean_token_ce":row["native_mean_token_ce"],"payload_target_ce_sum":if payload_zero==0{Some(payload_sum)}else{None},"payload_target_positions":payload_positions,"terminal_target_ce_sum":if terminal_zero==0{Some(terminal_sum)}else{None},"terminal_target_positions":terminal_positions,"payload_zero_support_positions":payload_zero,"terminal_zero_support_positions":terminal_zero}));
    }
    if pairs.values().any(|p| p.len() != 2) {
        return Err(invalid("discrete summary intact-pair width differs").into());
    }
    Ok(
        json!({"rows":summaries,"native_equal_episode_ce":canonical["native_equal_episode_ce"],"source_correct":source_correct,"both_paired_source_correct":pairs.values().filter(|p|p.iter().all(|x|*x)).count(),"own_prefix_generation":"NOT_RUN_FOR_TRIAL; canonical source/CE summary only","cases":episodes.len(),"scope":"all-panel actual native canonical predictions; typed role truth only after read; payload/Period-EOS target CE partition is descriptive, not alternate objective"}),
    )
}
fn source_changes(old: &Value, new: &Value) -> Result<Value> {
    let a = old["rows"]
        .as_array()
        .ok_or_else(|| invalid("discrete old summary absent"))?;
    let b = new["rows"]
        .as_array()
        .ok_or_else(|| invalid("discrete new summary absent"))?;
    if a.len() != b.len() {
        return Err(invalid("discrete comparison count differs").into());
    }
    let mut gains = Vec::new();
    let mut harms = Vec::new();
    let mut pairs = BTreeMap::<String, Vec<(bool, bool)>>::new();
    for (old, new) in a.iter().zip(b) {
        if old["id"] != new["id"] {
            return Err(invalid("discrete source comparison IDs differ").into());
        }
        let pair = old["pair_id"]
            .as_str()
            .ok_or_else(|| invalid("discrete old pair absent"))?;
        if new["pair_id"].as_str() != Some(pair) {
            return Err(invalid("discrete compared pair IDs differ").into());
        }
        let parent_correct = old["first_canonical_source_correct"]
            .as_bool()
            .ok_or_else(|| invalid("discrete parent source boolean absent"))?;
        let proposal_correct = new["first_canonical_source_correct"]
            .as_bool()
            .ok_or_else(|| invalid("discrete proposal source boolean absent"))?;
        pairs
            .entry(pair.into())
            .or_default()
            .push((parent_correct, proposal_correct));
        match (
            old["first_canonical_source_correct"].as_bool(),
            new["first_canonical_source_correct"].as_bool(),
        ) {
            (Some(false), Some(true)) => gains.push(new["id"].clone()),
            (Some(true), Some(false)) => harms.push(new["id"].clone()),
            (Some(_), Some(_)) => {}
            _ => return Err(invalid("discrete source outcome absent").into()),
        }
    }
    if pairs.values().any(|p| p.len() != 2) {
        return Err(invalid("discrete source comparison pair width differs").into());
    }
    let paired_gains = pairs
        .iter()
        .filter(|(_, p)| !p.iter().all(|(a, _)| *a) && p.iter().all(|(_, b)| *b))
        .map(|(id, _)| id.clone())
        .collect::<Vec<_>>();
    let paired_harms = pairs
        .iter()
        .filter(|(_, p)| p.iter().all(|(a, _)| *a) && !p.iter().all(|(_, b)| *b))
        .map(|(id, _)| id.clone())
        .collect::<Vec<_>>();
    Ok(
        json!({"gains":gains,"harms":harms,"both_paired_gains":paired_gains,"both_paired_harms":paired_harms,"parent_source_correct":old["source_correct"],"proposal_source_correct":new["source_correct"],"parent_both_paired_source_correct":old["both_paired_source_correct"],"proposal_both_paired_source_correct":new["both_paired_source_correct"]}),
    )
}

// A failed sealed fit is immutable evidence; completion never enters the optimizer.
pub(super) fn complete(a: &Args, start: Instant, failed_root: &Path) -> Result<Value> {
    report_output::verify(failed_root)?;
    let failed_manifest = sha256_file(&failed_root.join("manifest.json"))?;
    if a.cue_discrete_completion_manifest_sha256.as_deref() != Some(failed_manifest.as_str()) {
        return Err(invalid("completion failed attempt manifest differs").into());
    }
    let attempt = read_json(&failed_root.join("attempt.json"))?;
    let config_path = attempt["argv"][1]
        .as_str()
        .ok_or_else(|| invalid("completion original configuration path absent"))?;
    let config_sha = sha256_file(Path::new(config_path))?;
    if a.cue_discrete_completion_config_sha256.as_deref() != Some(config_sha.as_str()) {
        return Err(invalid("completion original configuration hash differs").into());
    }
    let original: Args = serde_json::from_slice(&read_capped(Path::new(config_path))?)?;
    macro_rules! same { ($($field:ident),+ $(,)?) => { $(
        if a.$field != original.$field { return Err(invalid(concat!("completion configuration differs: ", stringify!($field))).into()); }
    )+ }; }
    same!(
        source_weights,
        native_artifact,
        trusted_native_binding,
        development_panel,
        development_manifest_sha256,
        fresh_panel,
        fresh_manifest_sha256,
        cue_calibration_warmstart,
        frozen_prefix_bundle,
        frozen_prefix_native_metadata_sha256,
        frozen_prefix_packed_sha256,
        cue_score_mode,
        prefix_score_mode,
        cue_discrete_fit,
        maximum_context_tokens,
        maximum_generation_tokens
    );
    if original.mode != "cue-calibration-discrete-fit"
        || original.out != failed_root
        || a.out == failed_root
        || a.out.starts_with(failed_root)
    {
        return Err(invalid("completion original mode/output differs").into());
    }
    let failure = read_json(&failed_root.join("failure.json"))?;
    let selection = read_json(&failed_root.join("selection-before-fresh.json"))?;
    let progress = read_json(&failed_root.join("progress.json"))?;
    let selected_trial = selection["selected_trial"]
        .as_u64()
        .ok_or_else(|| invalid("completion selected trial absent"))?;
    let accepted = selection["accepted_discrete_updates"]
        .as_u64()
        .ok_or_else(|| invalid("completion accepted count absent"))?;
    if failure["error"] != "report cap reached"
        || failure["mode"] != original.mode
        || selected_trial == 0
        || selected_trial != accepted
        || selection["fresh_predictions_before_selection"] != 0
        || selection["stop_reason"] != "maximum_accepted_updates"
        || selection["selected_trial"] != progress["selected_trial"]
        || selection["accepted_discrete_updates"] != progress["accepted_discrete_updates"]
        || failure["completed_updates_in_progress"] != progress
        || failed_root.join("fresh-selected-generation.json").exists()
        || accepted
            != a.cue_discrete_fit
                .as_ref()
                .ok_or_else(|| invalid("completion frozen discrete configuration absent"))?
                .maximum_accepted_updates as u64
    {
        return Err(invalid("completion selected failed fit state differs").into());
    }
    let w = warm(a)?;
    let pr = a
        .frozen_prefix_bundle
        .as_ref()
        .ok_or_else(|| invalid("frozen prefix donor absent"))?;
    let mut inputs = BTreeMap::new();
    let mut seals = BTreeSet::new();
    for (root, expected) in [
        (&a.development_panel, &a.development_manifest_sha256),
        (&a.fresh_panel, &a.fresh_manifest_sha256),
    ] {
        report_output::verify(root)?;
        if sha256_file(&root.join("manifest.json"))? != *expected {
            return Err(invalid("cue calibration panel manifest differs").into());
        }
    }
    for root in [
        &a.source_weights,
        &a.native_artifact,
        &a.development_panel,
        &a.fresh_panel,
        &w.initial_cue_bundle,
        pr,
        &w.frozen_end_bundle,
    ] {
        let seal = nearest_seal(root)?;
        report_output::verify(&seal)?;
        inputs.insert(
            seal.join("manifest.json").to_string_lossy().into_owned(),
            sha256_file(&seal.join("manifest.json"))?,
        );
        seals.insert(seal);
    }
    for (root, name, expected) in [
        (
            &w.initial_cue_bundle,
            "native-metadata.json",
            &w.initial_cue_metadata_sha256,
        ),
        (
            &w.initial_cue_bundle,
            "cue-q4.bin",
            &w.initial_cue_packed_sha256,
        ),
        (
            pr,
            "native-metadata.json",
            a.frozen_prefix_native_metadata_sha256
                .as_ref()
                .ok_or_else(|| invalid("prefix metadata SHA absent"))?,
        ),
        (
            pr,
            "prefix-q4.bin",
            a.frozen_prefix_packed_sha256
                .as_ref()
                .ok_or_else(|| invalid("prefix packed SHA absent"))?,
        ),
        (
            &w.frozen_end_bundle,
            "native-metadata.json",
            &w.frozen_end_metadata_sha256,
        ),
        (
            &w.frozen_end_bundle,
            "source-end-period-q4.bin",
            &w.frozen_end_period_packed_sha256,
        ),
        (
            &w.frozen_end_bundle,
            "source-end-stop-q4.bin",
            &w.frozen_end_stop_packed_sha256,
        ),
    ] {
        let path = root.join(name);
        if sha256_file(&path)? != *expected {
            return Err(invalid("cue calibration initializer/frozen hash differs").into());
        }
        inputs.insert(path.to_string_lossy().into_owned(), expected.clone());
    }
    let trusted = sha256_file(&a.trusted_native_binding)?;
    inputs.insert(
        a.trusted_native_binding.to_string_lossy().into_owned(),
        trusted.clone(),
    );
    let binding: NativeArtifactBinding =
        serde_json::from_slice(&read_capped(&a.trusted_native_binding)?)?;
    let integer = IntegerRealizer::load_native(&a.native_artifact, &binding)?;
    let bytes = fs::read(a.native_artifact.join("tokenizer.json"))?;
    let tok = ByteBpeTokenizer::from_tokenizer_json_bytes(&bytes)
        .ok_or_else(|| invalid("cue calibration ByteBPE absent"))?;
    let source = SourceRealizerWeights::load_source(&a.source_weights, &bytes)?;
    let identity: ConsumerIdentity = serde_json::from_value(
        read_json(&a.native_artifact.join("metadata.json"))?["identity"].clone(),
    )?;
    let parent = NativeSourceRealizer::load(&a.native_artifact, &source, &identity)?;
    if parent.artifact_binding()? != binding {
        return Err(invalid("cue calibration source/native binding differs").into());
    }
    let receipts = parameter_receipts(&source.parameters())?;
    let donor = prefix_training_cue_load(&w.initial_cue_bundle, &parent)?;
    let weights = CueAngularWeights::from_native(&parent, &a.native_artifact, &donor)?;
    let cue = cue_native_load(&w.initial_cue_bundle, &integer)?;
    let prefix = prefix_native_load(pr, &integer, &cue)?;
    if Some(donor.metadata().potential.mode) != a.cue_score_mode
        || Some(cue.metadata().potential.mode) != a.cue_score_mode
        || Some(prefix.metadata().potential.mode) != a.prefix_score_mode
    {
        return Err(
            invalid("actual cue/prefix donor modes differ from declared configuration").into(),
        );
    }
    let end_meta = read_json(&w.frozen_end_bundle.join("native-metadata.json"))?;
    let f = Frozen {
        prefix_config: prefix.metadata().potential,
        prefix: fs::read(pr.join("prefix-q4.bin"))?,
        end_config: serde_json::from_value(end_meta["potential"].clone())?,
        period: fs::read(w.frozen_end_bundle.join("source-end-period-q4.bin"))?,
        stop: fs::read(w.frozen_end_bundle.join("source-end-stop-q4.bin"))?,
    };
    if f.end_config.mode != SourceEndScoreMode::DirectedRelative {
        return Err(invalid("cue calibration endpoint mode differs").into());
    }
    let (_, end) = f.integer(&integer, &cue)?;
    if serde_json::to_value(end.metadata())? != end_meta {
        return Err(invalid("cue calibration original endpoint chain differs").into());
    }
    let (tp, te) = f.training(&parent, &donor)?;
    if serde_json::to_value(tp.metadata())? != serde_json::to_value(prefix.metadata())?
        || serde_json::to_value(te.metadata())? != end_meta
    {
        return Err(invalid("cue calibration training/independent donor metadata differs").into());
    }
    inputs.insert(config_path.into(), config_sha.clone());
    inputs.insert(
        failed_root
            .join("manifest.json")
            .to_string_lossy()
            .into_owned(),
        failed_manifest.clone(),
    );
    seals.insert(failed_root.to_path_buf());
    let selected_root = failed_root.join(format!("trial-{selected_trial:04}"));
    report_output::verify(&selected_root)?;
    let selected_receipt = read_json(&selected_root.join("receipt.json"))?;
    let selected_hash = sha256_file(&selected_root.join("cue/cue-q4.bin"))?;
    if selected_receipt["accepted"] != true
        || selected_receipt["trial"] != selected_trial
        || selected_receipt["accepted_updates_after"] != accepted
        || selected_receipt["proposal_cue_packed_sha256"] != selected_hash
        || progress["current_cue_packed_sha256"] != selected_hash
        || selected_receipt["actual_proposal_ce"] != progress["incumbent_native_ce"]
        || selected_receipt["frozen_payloads"] != f.hashes()
    {
        return Err(invalid("completion selected native identity differs").into());
    }
    let initial_root = failed_root.join("initial-chain");
    report_output::verify(&initial_root)?;
    let restored = CueAngularWeights::load(&initial_root.join("cue"), &parent, &a.native_artifact)?;
    if parameter_receipts(&restored.parameters())? != parameter_receipts(&weights.parameters())? {
        return Err(invalid("completion original cue shadow differs").into());
    }
    for root in [&initial_root, &selected_root] {
        for (relative, expected) in [
            ("prefix/prefix-q4.bin", &f.prefix),
            ("source-end/source-end-period-q4.bin", &f.period),
            ("source-end/source-end-stop-q4.bin", &f.stop),
        ] {
            if fs::read(root.join(relative))? != *expected {
                return Err(invalid("completion frozen payload changed").into());
            }
        }
    }
    let (baseline_cue, _baseline_prefix, _baseline_end) = load_chain(&initial_root, &integer)?;
    if serde_json::to_value(baseline_cue.metadata())? != serde_json::to_value(cue.metadata())? {
        return Err(invalid("completion original native cue differs").into());
    }
    let (c, p, e) = load_chain(&selected_root, &integer)?;
    let (development_rows, evaluation_rows) = panel_counts(a);
    let development = panel(&a.development_panel, development_rows, &integer, &tok, a)?;
    let diagnostic = panel(&a.fresh_panel, evaluation_rows, &integer, &tok, a)?;
    let baseline = read_json(&failed_root.join("initial-canonical.json"))?;
    let final_canonical = read_json(&failed_root.join("development-selected-canonical.json"))?;
    let parent_gen = read_json(&failed_root.join("development-parent-generation.json"))?;
    let final_gen = read_json(&failed_root.join("development-selected-generation.json"))?;
    let oldcan = read_json(&failed_root.join("fresh-parent-canonical.json"))?;
    let newcan = read_json(&failed_root.join("fresh-selected-canonical.json"))?;
    let oldgen = read_json(&failed_root.join("fresh-parent-generation.json"))?;
    if final_canonical["native_equal_episode_ce"] != progress["incumbent_native_ce"] {
        return Err(invalid("completion saved selected objective differs").into());
    }
    let olddev = causal_metrics(&a.development_panel, &development, &baseline, &parent_gen)?;
    let newdev = causal_metrics(
        &a.development_panel,
        &development,
        &final_canonical,
        &final_gen,
    )?;
    let devout =
        json!({"parent":olddev,"selected":newdev,"comparison":causal_comparison(&olddev,&newdev)?});
    if devout != read_json(&failed_root.join("development-causal-outcomes.json"))? {
        return Err(invalid("completion saved development metrics differ").into());
    }
    drop(final_canonical);
    drop(parent_gen);
    drop(final_gen);
    immutable(&inputs, &seals)?;
    deadline(a, start)?;
    // The only model execution in this path. Selection is already sealed.
    let newgen = source_end_fit::generation(&integer, &c, &p, &e, &diagnostic, &tok, a, start)?;
    write_json(&a.out, "fresh-selected-generation.json", &newgen)?;
    if a.cue_discrete_fit
        .as_ref()
        .is_some_and(|cfg| cfg.composition_panel.is_some())
    {
        write_json(
            &a.out,
            "prospective-first-read-coverage.json",
            &coverage_strata(&baseline, &oldcan)?,
        )?;
    }
    let oldmetrics = causal_metrics(&a.fresh_panel, &diagnostic, &oldcan, &oldgen)?;
    let newmetrics = causal_metrics(&a.fresh_panel, &diagnostic, &newcan, &newgen)?;
    write_json(&a.out, "development-causal-outcomes.json", &devout)?;
    write_json(
        &a.out,
        "fresh-causal-outcomes.json",
        &json!({"parent":oldmetrics,"selected":newmetrics,"comparison":causal_comparison(&oldmetrics,&newmetrics)?}),
    )?;
    frozen(&source, &receipts)?;
    immutable(&inputs, &seals)?;
    Ok(
        json!({"schema":"uor-r4.geometric-cue-discrete-completion/1","mode":a.mode,"status":"completed",
        "failed_attempt":failed_root,"failed_attempt_manifest_sha256":failed_manifest,
        "original_configuration_sha256":config_sha,"original_source_commit":failure["source_commit"],
        "selected_trial":selected_trial,"selected_cue_packed_sha256":selected_hash,
        "original_accepted_discrete_updates":accepted,"original_selection":selection,
        "optimizer_updates":0,"adam_updates":0,"accepted_discrete_updates":0,"proposal_count":0,"gradient_passes":0,
        "model_operations":"only selected prospective own-prefix generation; no optimization/canonical rerun",
        "reused_saved_files":["initial-canonical.json","development-selected-canonical.json","development-parent-generation.json","development-selected-generation.json","fresh-parent-canonical.json","fresh-selected-canonical.json","fresh-parent-generation.json"],
        "input_manifests_sha256":inputs,"frozen_source_receipts":receipts,"frozen_payloads":f.hashes(),
        "cases":development_rows,"evaluation_cases":evaluation_rows,"elapsed_seconds":start.elapsed().as_secs_f64(),"peak_rss_kib_linux":peak_rss_kib(),
        "claim":"evaluation-only completion of sealed accepted native cue checkpoint; no new learning or general chat qualification"}),
    )
}

#[allow(clippy::too_many_arguments)]
pub(super) fn run(
    a: &Args,
    start: Instant,
    source: &SourceRealizerWeights,
    parent: &NativeSourceRealizer,
    integer: &IntegerRealizer,
    tok: &ByteBpeTokenizer,
    initial_weights: &CueAngularWeights,
    f: &Frozen,
    development: &[Episode],
    baseline: &Value,
    parent_gen: &Value,
    inputs: &BTreeMap<String, String>,
    seals: &BTreeSet<PathBuf>,
    receipts: &Value,
) -> Result<Value> {
    let cfg = a
        .cue_discrete_fit
        .as_ref()
        .ok_or_else(|| invalid("discrete configuration absent"))?;
    let original_receipts = parameter_receipts(&initial_weights.parameters())?;
    let canonical_bytes = serde_json::to_vec(baseline)?.len();
    let generation_bytes = serde_json::to_vec(parent_gen)?.len();
    let projection_bytes = canonical_bytes
        .saturating_add(generation_bytes)
        .saturating_mul(5)
        .div_ceil(2)
        .saturating_add(16 * 1024 * 1024);
    if projection_bytes.saturating_add(1024 * 1024) > a.maximum_report_bytes {
        return Err(invalid("discrete observed-shape storage exceeds admitted cap").into());
    }
    let projection = json!({"observed_shape_total_bytes":projection_bytes,"baseline_canonical_bytes":canonical_bytes,"baseline_generation_bytes":generation_bytes,"full_trace_sets":2.5,"proposal_artifacts":"at most16 tiny cue/prefix/end chains plus compact per-row CE/source/margin/gradient receipts; no pertrial full canonical/generation","administration_reserve_bytes":16*1024*1024,"configured_report_cap_bytes":a.maximum_report_bytes,"scope":"observed baseline shape projection, not formal size bound; actual writer cap/stop margin authoritative"});
    write_json(&a.out, "storage-projection.json", &projection)?;
    let mut current =
        CueAngularWeights::load(&a.out.join("initial-chain/cue"), parent, &a.native_artifact)?;
    if parameter_receipts(&current.parameters())? != original_receipts {
        return Err(invalid("discrete initial shadow reload differs").into());
    }
    let mut current_canonical = baseline.clone();
    let baseline_summary = source_summary(&a.development_panel, development, baseline, integer)?;
    let mut current_summary = baseline_summary.clone();
    let mut incumbent_ce = baseline["native_equal_episode_ce"]
        .as_f64()
        .filter(|x| x.is_finite())
        .ok_or_else(|| invalid("discrete finite baseline CE absent"))?;
    let mut rejected = HeadRejections::default();
    let mut cached_gradient: Option<(Vec<f32>, Value)> = None;
    let mut accepted = 0usize;
    let mut selected_trial = 0usize;
    let mut trials = Vec::new();
    let mut gradient_passes = 0usize;
    let mut stop_reason = "maximum_trials";
    for trial in 1..=cfg.maximum_trials {
        deadline(a, start)?;
        if accepted >= cfg.maximum_accepted_updates {
            stop_reason = "maximum_accepted_updates";
            break;
        }
        let packed = current.packed_coefficients()?;
        let count = current.config().coefficient_count()?;
        rejected.at_head(&packed);
        if cached_gradient.is_none() {
            let measured = batch(
                &(0..development.len()).collect::<Vec<_>>(),
                development,
                source,
                parent,
                Some(integer),
                &current,
                f,
                false,
                a,
                start,
            )?;
            let measured_ce = measured.report["native_equal_episode_ce"]
                .as_f64()
                .filter(|x| x.is_finite())
                .ok_or_else(|| invalid("discrete current gradient native CE absent"))?;
            if (measured_ce - incumbent_ce).abs() > 1e-12 {
                return Err(
                    invalid("discrete gradient and actual incumbent objective differ").into(),
                );
            }
            let tensor = measured
                .gradients
                .get("cue.coefficients")
                .ok_or_else(|| invalid("discrete cue gradient missing"))?;
            let gradient = tensor.flatten_all()?.to_vec1::<f32>()?;
            let metadata = json!({"pass":gradient_passes+1,"current_cue_packed_sha256":sha256_bytes(&packed),"current_native_equal_episode_ce":incumbent_ce,"gradient_report":measured.report,"gradient_coefficients_f32":gradient,"gradient_scope":"ordinary full-panel equalepisode answer+EOS CE; allsource admission; encoder/roots/prefix/end/argmax stopped; no role/source-correctness proposal steering"});
            gradient_passes += 1;
            write_json(
                &a.out,
                &format!("gradient-{gradient_passes:04}.json"),
                &metadata,
            )?;
            cached_gradient = Some((gradient, metadata));
        }
        let (gradient, gradient_receipt) = cached_gradient
            .as_ref()
            .ok_or_else(|| invalid("discrete cached gradient absent"))?;
        if gradient_receipt["current_cue_packed_sha256"] != sha256_bytes(&packed) {
            return Err(invalid("discrete stale gradient head").into());
        }
        let values = unpack_coefficients(count, &packed).map_err(|e| invalid(e.to_string()))?;
        let Some((index, step, credit)) = next_coordinate(&values, gradient, &rejected.indices)?
        else {
            stop_reason = "no_eligible_nonzero_legal_untried_coordinate";
            break;
        };
        // This generic coordinate witness is not a quantum-probe configuration:
        // its coordinate and sign are solely outputs of the ordinary gradient.
        let coordinate = CueQuantumProbe {
            coefficient_index: index,
            initial_quarters: values[index],
            preferred_step: step,
            evidence_receipt: PathBuf::new(),
            evidence_receipt_sha256: String::new(),
        };
        let proposed_packed = quantum_packed(&packed, count, &coordinate, step)?;
        let proposed_cue = parent.compile_cue_carrier(
            CueAngularQ4::new(current.config(), &proposed_packed)
                .map_err(|e| invalid(e.to_string()))?,
        )?;
        let proposal = CueAngularWeights::from_native(parent, &a.native_artifact, &proposed_cue)?;
        let root = a.out.join(format!("trial-{trial:04}"));
        report_output::claim(&root)?;
        save_chain(&root, &proposal, parent, f)?;
        let restored = CueAngularWeights::load(&root.join("cue"), parent, &a.native_artifact)?;
        if parameter_receipts(&proposal.parameters())?
            != parameter_receipts(&restored.parameters())?
        {
            return Err(invalid("discrete proposal source reload differs").into());
        }
        let (c, p, e) = load_chain(&root, integer)?;
        if c.packed_coefficients() != proposed_packed {
            return Err(invalid("discrete proposal independent packed differs").into());
        }
        let proposed_canonical =
            source_end_fit::canonical(integer, &c, &p, &e, development, a, start)?;
        let proposal_ce = proposed_canonical["native_equal_episode_ce"]
            .as_f64()
            .filter(|x| x.is_finite());
        let summary = source_summary(
            &a.development_panel,
            development,
            &proposed_canonical,
            integer,
        )?;
        let margins = quantum_margin_changes(
            &current_canonical,
            &proposed_canonical,
            &current_summary,
            &coordinate,
            step,
        )?;
        let decision = improves(incumbent_ce, proposal_ce);
        let receipt = json!({"trial":trial,"optimizer_updates":accepted+usize::from(decision),"adam_updates":0,"accepted":decision,"accepted_updates_before":accepted,"accepted_updates_after":accepted+usize::from(decision),"parent_cue_packed_sha256":sha256_bytes(&packed),"proposal_cue_packed_sha256":sha256_bytes(&proposed_packed),"coefficient_index":index,"lane":index/120,"angular_bin":index%120,"initial_quarters":values[index],"step_quarters":step,"gradient_credit":credit,"predicted_ce_delta":f64::from(credit)*f64::from(step)/4.,"actual_parent_ce":incumbent_ce,"actual_proposal_ce":proposal_ce,"realized_ce_delta":proposal_ce.map(|v|v-incumbent_ce),"acceptance":"finite actual full-panel native CE < incumbent -1e-9; no source/role correctness gate","source_changes_vs_incumbent":source_changes(&current_summary,&summary)?,"source_changes_vs_baseline":source_changes(&baseline_summary,&summary)?,"gradient_pass":gradient_passes,"all_other_coefficients_unchanged":true,"frozen_payloads":f.hashes(),"independent_native_reload":true,"native_single_address_delta_verified":true,"touched_rows":margins["touched_rows"],"own_prefix_generation":"NOT_RUN_FOR_TRIAL"});
        write_json(&root, "canonical-summary.json", &summary)?;
        write_json(&root, "record-margin-changes.json", &margins)?;
        write_json(&root, "receipt.json", &receipt)?;
        report_output::seal(&root)?;
        report_output::verify(&root)?;
        trials.push(receipt);
        if decision {
            incumbent_ce = proposal_ce.ok_or_else(|| invalid("discrete accepted CE absent"))?;
            current = restored;
            current_canonical = proposed_canonical;
            current_summary = summary;
            accepted += 1;
            selected_trial = trial;
            cached_gradient = None;
            rejected.at_head(&proposed_packed);
        } else {
            rejected.indices.insert(index);
        }
        frozen(source, receipts)?;
        write_json(
            &a.out,
            "progress.json",
            &json!({"optimizer_updates":accepted,"adam_updates":0,"trials_completed":trials.len(),"accepted_discrete_updates":accepted,"selected_trial":selected_trial,"current_cue_packed_sha256":sha256_bytes(&current.packed_coefficients()?),"incumbent_native_ce":incumbent_ce,"gradient_passes":gradient_passes,"elapsed_seconds":start.elapsed().as_secs_f64()}),
        )?;
    }
    if accepted >= cfg.maximum_accepted_updates {
        stop_reason = "maximum_accepted_updates";
    }
    write_json(
        &a.out,
        "selection-before-fresh.json",
        &json!({"selected_trial":selected_trial,"accepted_discrete_updates":accepted,"optimizer_updates":accepted,"adam_updates":0,"criterion":"native ordinary full-panel CE descent; baseline eligible; fresh never selects","fresh_predictions_before_selection":0,"stop_reason":stop_reason}),
    )?;
    let selected_root = if selected_trial == 0 {
        a.out.join("initial-chain")
    } else {
        a.out.join(format!("trial-{selected_trial:04}"))
    };
    let (c, p, e) = load_chain(&selected_root, integer)?;
    let final_canonical = source_end_fit::canonical(integer, &c, &p, &e, development, a, start)?;
    if final_canonical != current_canonical {
        return Err(invalid("discrete selected independent canonical differs").into());
    }
    let final_generation =
        source_end_fit::generation(integer, &c, &p, &e, development, tok, a, start)?;
    write_json(
        &a.out,
        "development-selected-canonical.json",
        &final_canonical,
    )?;
    write_json(
        &a.out,
        "development-selected-generation.json",
        &final_generation,
    )?;
    let parent_metrics = causal_metrics(&a.development_panel, development, baseline, parent_gen)?;
    let final_metrics = causal_metrics(
        &a.development_panel,
        development,
        &final_canonical,
        &final_generation,
    )?;
    write_json(
        &a.out,
        "development-causal-outcomes.json",
        &json!({"parent":parent_metrics,"selected":final_metrics,"comparison":causal_comparison(&parent_metrics,&final_metrics)?}),
    )?;
    let (_, evaluation_rows) = panel_counts(a);
    if cfg.composition_panel.is_some()
        && !composition_report_matches(
            &read_json(&a.fresh_panel.join("report.json"))?,
            "fresh",
            evaluation_rows,
        )
    {
        return Err(invalid("prospective evaluation panel report differs").into());
    }
    let diagnostic = panel(&a.fresh_panel, evaluation_rows, integer, tok, a)?;
    let (bc, bp, be) = load_chain(&a.out.join("initial-chain"), integer)?;
    let oldcan = source_end_fit::canonical(integer, &bc, &bp, &be, &diagnostic, a, start)?;
    let oldgen = source_end_fit::generation(integer, &bc, &bp, &be, &diagnostic, tok, a, start)?;
    let newcan = source_end_fit::canonical(integer, &c, &p, &e, &diagnostic, a, start)?;
    let newgen = source_end_fit::generation(integer, &c, &p, &e, &diagnostic, tok, a, start)?;
    write_json(&a.out, "fresh-parent-canonical.json", &oldcan)?;
    write_json(&a.out, "fresh-selected-canonical.json", &newcan)?;
    write_json(&a.out, "fresh-parent-generation.json", &oldgen)?;
    write_json(&a.out, "fresh-selected-generation.json", &newgen)?;
    if cfg.composition_panel.is_some() {
        write_json(
            &a.out,
            "prospective-first-read-coverage.json",
            &coverage_strata(baseline, &oldcan)?,
        )?;
    }
    let oldmetrics = causal_metrics(&a.fresh_panel, &diagnostic, &oldcan, &oldgen)?;
    let newmetrics = causal_metrics(&a.fresh_panel, &diagnostic, &newcan, &newgen)?;
    write_json(
        &a.out,
        "fresh-causal-outcomes.json",
        &json!({"parent":oldmetrics,"selected":newmetrics,"comparison":causal_comparison(&oldmetrics,&newmetrics)?}),
    )?;
    if parameter_receipts(&initial_weights.parameters())? != original_receipts {
        return Err(invalid("discrete original parent cue mutated").into());
    }
    frozen(source, receipts)?;
    immutable(inputs, seals)?;
    Ok(
        json!({"schema":"uor-r4.geometric-cue-discrete-fit/1","mode":a.mode,"status":"completed","cases":development.len(),"evaluation_cases":evaluation_rows,"cue_discrete_fit":cfg,"cue_calibration_warmstart":a.cue_calibration_warmstart,"optimizer_updates":accepted,"adam_updates":0,"proposal_count":trials.len(),"accepted_discrete_updates":accepted,"selected_trial":selected_trial,"selected_native_equal_episode_ce":incumbent_ce,"selected_cue_packed_sha256":sha256_bytes(&current.packed_coefficients()?),"gradient_passes":gradient_passes,"trials":trials,"stop_reason":stop_reason,"input_manifests_sha256":inputs,"frozen_source_receipts":receipts,"frozen_payloads":f.hashes(),"storage_projection":projection,"fresh_predictions_before_selection":0,"elapsed_seconds":start.elapsed().as_secs_f64(),"peak_rss_kib_linux":peak_rss_kib(),"learning_law":"recomputed ordinary full-panel gradient; largestabs legal observed nonzero coordinate; deterministic index ties; one negativegradient quarter; actualnative finiteCE descent; rejectedindices scopedto currentpackedhead","evaluation_scope":if cfg.composition_panel.is_some(){"prospectively partitioned retained-literal bank compositions; predictions after selection; no unseen-literal/word claim"}else{"exposed diagnostic"},"claim":"bounded native discrete cue learning; fullbank admission and baseline/final actual ownprefix; no seedconsistency/generalchat/energy qualification"}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn coverage_separates_new_joint_tuples_from_unsupported_bins_without_labels() -> Result<()> {
        fn trace(id: &str, bins: Value) -> Value {
            json!({"id":id,"tokens":[{"cue_carrier":{"angular_indices":bins}}]})
        }
        let training = json!({"rows":[trace("a",json!([[1,2],[3,4]]))]});
        let evaluation = json!({"rows":[
            trace("seen",json!([[1,2],[3,4]])),
            trace("newjoint",json!([[1,2],[4,3]])),
            trace("unsupported",json!([[1,9],[3,4]]))]});
        let result = coverage_strata(&training, &evaluation)?;
        assert_eq!(
            result["evaluation_rows"][0]["stratum"],
            "seen-joint-geometric-tuples"
        );
        assert_eq!(
            result["evaluation_rows"][1]["stratum"],
            "covered-bins/new-joint-tuple"
        );
        assert_eq!(
            result["evaluation_rows"][2]["stratum"],
            "unsupported-constituent-bin"
        );
        let bad = json!({"rows":[trace("bad",json!([[120],[4]]))]});
        assert!(coverage_strata(&training, &bad).is_err());
        let mut targets = evaluation;
        targets["rows"][0]["target_label_only_after_read"] = json!(999);
        assert_eq!(coverage_strata(&training, &targets)?, result);
        Ok(())
    }
    #[test]
    fn coordinate_ordering_respects_legal_boundaries_ties_and_current_rejections() -> Result<()> {
        let values = [-7, 7, 0, 0, 0];
        let gradient = [100., -100., 2., -2., 0.];
        let mut rejected = BTreeSet::new();
        assert_eq!(
            next_coordinate(&values, &gradient, &rejected)?,
            Some((2, -1, 2.))
        );
        rejected.insert(2);
        assert_eq!(
            next_coordinate(&values, &gradient, &rejected)?,
            Some((3, 1, -2.))
        );
        rejected.insert(3);
        assert_eq!(next_coordinate(&values, &gradient, &rejected)?, None);
        assert!(next_coordinate(&values, &[f32::NAN; 5], &rejected).is_err());
        assert!(next_coordinate(&values, &[1.], &rejected).is_err());
        assert!(next_coordinate(&[8], &[1.], &BTreeSet::new()).is_err());
        Ok(())
    }
    #[test]
    fn rejected_coordinate_is_reconsidered_after_a_native_head_change() {
        let mut head = HeadRejections::default();
        head.at_head(&[1, 2]);
        head.indices.insert(123);
        head.at_head(&[1, 2]);
        assert!(head.indices.contains(&123));
        head.at_head(&[1, 3]);
        assert!(head.indices.is_empty());
        assert_eq!(head.packed_sha256, sha256_bytes(&[1, 3]));
    }
    #[test]
    fn acceptance_requires_finite_actual_native_loss_descent() {
        assert!(improves(1., Some(0.99)));
        assert!(!improves(1., Some(1.)));
        assert!(!improves(1., Some(1. - 0.5e-9)));
        assert!(!improves(1., Some(f64::NEG_INFINITY)));
        assert!(!improves(1., Some(f64::NAN)));
        assert!(!improves(1., None));
        assert!(!improves(f64::INFINITY, Some(1.)));
    }
    #[test]
    fn source_only_comparison_does_not_invent_generation_or_gate_loss() -> Result<()> {
        let old = json!({"source_correct":1,"both_paired_source_correct":0,"rows":[{"id":"job","pair_id":"pair","first_canonical_source_correct":true},{"id":"home","pair_id":"pair","first_canonical_source_correct":false}]});
        let new = json!({"source_correct":1,"both_paired_source_correct":0,"rows":[{"id":"job","pair_id":"pair","first_canonical_source_correct":false},{"id":"home","pair_id":"pair","first_canonical_source_correct":true}]});
        let changes = source_changes(&old, &new)?;
        assert_eq!(changes["gains"], json!(["home"]));
        assert_eq!(changes["harms"], json!(["job"]));
        assert_eq!(changes["both_paired_gains"], json!([]));
        let mut both = old.clone();
        both["rows"][1]["first_canonical_source_correct"] = json!(true);
        both["source_correct"] = json!(2);
        both["both_paired_source_correct"] = json!(1);
        assert_eq!(
            source_changes(&both, &new)?["both_paired_harms"],
            json!(["pair"])
        );
        assert_eq!(
            source_changes(&new, &both)?["both_paired_gains"],
            json!(["pair"])
        );
        // Native CE descent remains the learning law, even with a source loss.
        assert!(improves(1., Some(0.99)));
        let mut foreign = new;
        foreign["rows"][0]["id"] = json!("wrong");
        assert!(source_changes(&old, &foreign).is_err());
        Ok(())
    }
}
