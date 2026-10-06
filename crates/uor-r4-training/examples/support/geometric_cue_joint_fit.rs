//! Joint-only discrete learning selected by actual development trajectories.
//! Ordinary marginal CE proposes directions; it is not a rollout derivative.
use super::*;
use uor_r4_integer::geometric_cue_carrier::CueJointConfig;
use uor_r4_integer::geometric_potential_q4::unpack_coefficients;

const MAX_TRIALS: usize = 16;
const MAX_ACCEPTED: usize = 8;
const EPSILON: f64 = 1e-9;

fn improves(old_complete: usize, old_ce: f64, complete: usize, ce: f64) -> bool {
    old_ce.is_finite()
        && ce.is_finite()
        && (complete > old_complete || (complete == old_complete && ce < old_ce - EPSILON))
}
fn coordinate(q: &[i8], g: &[f32], rejected: &BTreeSet<usize>) -> Result<Option<(usize, i8)>> {
    if q.len() != 16 || g.len() != 16 || g.iter().any(|x| !x.is_finite()) {
        return Err(invalid("joint fit gradient/quarter shape or finite contract differs").into());
    }
    let mut best = None;
    for i in 0..16 {
        if !(-7..=7).contains(&q[i]) {
            return Err(invalid("joint fit illegal incumbent quarter").into());
        }
        let step = if g[i] > 0. { -1 } else { 1 };
        if g[i] == 0. || rejected.contains(&i) || !(-7..=7).contains(&(q[i] + step)) {
            continue;
        }
        if best.is_none_or(|(j, _): (usize, i8)| g[i].abs() > g[j].abs()) {
            best = Some((i, step));
        }
    }
    Ok(best)
}
fn objective(canonical: &Value, generation: &Value) -> Result<(usize, f64)> {
    let ce = if canonical["native_equal_episode_ce"].is_null()
        && canonical["zero_support_positions"]
            .as_array()
            .is_some_and(|xs| !xs.is_empty())
    {
        f64::INFINITY
    } else {
        canonical["native_equal_episode_ce"]
            .as_f64()
            .filter(|x| x.is_finite())
            .ok_or_else(|| invalid("joint fit malformed native objective"))?
    };
    let rows = generation["rows"]
        .as_array()
        .ok_or_else(|| invalid("joint fit generation rows absent"))?;
    let complete = rows.iter().try_fold(0usize, |sum, row| -> Result<usize> {
        Ok(sum
            + usize::from(
                row["accepted_complete_answer"]
                    .as_bool()
                    .ok_or_else(|| invalid("joint fit row completeness absent"))?,
            ))
    })?;
    if generation["accepted_complete"].as_u64() != Some(complete as u64) {
        return Err(invalid("joint fit aggregate completion differs from rows").into());
    }
    Ok((complete, ce))
}
fn compact(c: &Value, g: &Value, episodes: &[Episode]) -> Result<Value> {
    let cs = c["rows"]
        .as_array()
        .ok_or_else(|| invalid("joint fit canonical rows absent"))?;
    let gs = g["rows"]
        .as_array()
        .ok_or_else(|| invalid("joint fit generation rows absent"))?;
    if cs.len() != episodes.len() || gs.len() != episodes.len() {
        return Err(invalid("joint fit compact row count differs").into());
    }
    let mut rows = Vec::new();
    for ((c, g), e) in cs.iter().zip(gs).zip(episodes) {
        if c["id"] != e.packet.id || g["id"] != e.packet.id {
            return Err(invalid("joint fit compact row identities differ").into());
        }
        let ids = g["generated_ids_including_eos"]
            .as_array()
            .ok_or_else(|| invalid("joint fit emitted IDs absent"))?;
        let emitted = ids
            .iter()
            .map(|v| {
                v.as_u64()
                    .and_then(|x| u32::try_from(x).ok())
                    .ok_or_else(|| invalid("joint fit emitted ID invalid"))
            })
            .collect::<std::result::Result<Vec<_>, _>>()?;
        let first_error =
            (0..emitted.len().max(e.target.len())).find(|&i| emitted.get(i) != e.target.get(i));
        rows.push(json!({"id":e.packet.id,"native_mean_token_ce":c["native_mean_token_ce"],"generated_ids_including_eos":emitted,"eos":g["eos"],"accepted_complete_answer":g["accepted_complete_answer"],"first_error_position_labels_only":first_error}));
    }
    Ok(
        json!({"rows":rows,"scope":"postprediction labels only; no source-answer admission or gradient filtering"}),
    )
}
fn gains(old: &Value, new: &Value) -> Result<Value> {
    let old = old["rows"]
        .as_array()
        .ok_or_else(|| invalid("joint fit old compact rows absent"))?;
    let new = new["rows"]
        .as_array()
        .ok_or_else(|| invalid("joint fit new compact rows absent"))?;
    if old.len() != new.len() {
        return Err(invalid("joint fit comparison count differs").into());
    }
    let mut gained = Vec::new();
    let mut lost = Vec::new();
    for (a, b) in old.iter().zip(new) {
        if a["id"] != b["id"] {
            return Err(invalid("joint fit comparison identity differs").into());
        }
        let ac = a["accepted_complete_answer"]
            .as_bool()
            .ok_or_else(|| invalid("joint fit old completion invalid"))?;
        let bc = b["accepted_complete_answer"]
            .as_bool()
            .ok_or_else(|| invalid("joint fit new completion invalid"))?;
        if !ac && bc {
            gained.push(b["id"].clone());
        }
        if ac && !bc {
            lost.push(b["id"].clone());
        }
    }
    Ok(json!({"complete_gained_ids":gained,"complete_lost_ids":lost}))
}

pub(super) fn run(
    a: &Args,
    start: Instant,
    source: &SourceRealizerWeights,
    parent: &NativeSourceRealizer,
    integer: &IntegerRealizer,
    tok: &ByteBpeTokenizer,
    donor: &CueAngularWeights,
    f: &Frozen,
    development: &[Episode],
    inputs: &BTreeMap<String, String>,
    seals: &BTreeSet<PathBuf>,
    receipts: &Value,
) -> Result<Value> {
    if donor.joint_config().is_some() || development.len() != 512 {
        return Err(invalid("joint fit requires unary donor and all512 development rows").into());
    }
    let unary = donor.packed_coefficients()?;
    let donor_receipts = parameter_receipts(&donor.parameters())?;
    let config = CueJointConfig {
        head: 1,
        left_lane: 1,
        right_lane: 3,
    };
    let mut current = CueAngularWeights::from_native(
        parent,
        &a.native_artifact,
        &parent.compile_cue_carrier(donor.native()?)?,
    )?
    .with_zero_joint(parent, config)?;
    let initial = a.out.join("initial-chain");
    report_output::claim(&initial)?;
    save_chain(&initial, &current, parent, f)?;
    let zero_receipts = parameter_receipts(&current.parameters())?;
    current = CueAngularWeights::load(&initial.join("cue"), parent, &a.native_artifact)?;
    if parameter_receipts(&current.parameters())? != zero_receipts
        || current.packed_coefficients()? != unary
    {
        return Err(invalid("joint fit baseline independent source reload differs").into());
    }
    let (c, p, e) = load_chain(&initial, integer)?;
    let (dc, dp, de) = {
        let c = integer.compile_cue_carrier(donor.native()?)?;
        let (p, e) = f.integer(integer, &c)?;
        (c, p, e)
    };
    let donor_canonical = source_end_fit::canonical(integer, &dc, &dp, &de, development, a, start)?;
    let donor_generation =
        source_end_fit::generation(integer, &dc, &dp, &de, development, tok, a, start)?;
    let mut selected_canonical =
        source_end_fit::canonical(integer, &c, &p, &e, development, a, start)?;
    let mut selected_generation =
        source_end_fit::generation(integer, &c, &p, &e, development, tok, a, start)?;
    if joint_probe::numerical(donor_canonical)?
        != joint_probe::numerical(selected_canonical.clone())?
        || joint_probe::numerical(donor_generation)?
            != joint_probe::numerical(selected_generation.clone())?
    {
        return Err(invalid("joint fit zero overlay numerical replay differs").into());
    }
    let mut best = objective(&selected_canonical, &selected_generation)?;
    if !best.1.is_finite() {
        return Err(invalid("joint fit baseline has infinite native objective; no floor").into());
    }
    let baseline_objective = best;
    let baseline = compact(&selected_canonical, &selected_generation, development)?;
    let baseline_metrics = causal_metrics(
        &a.development_panel,
        development,
        &selected_canonical,
        &selected_generation,
    )?;
    let mut prior_artifacts = vec![(initial.clone(), baseline.clone(), baseline_metrics.clone())];
    let mut selected_compact = baseline.clone();
    // Only baseline/final retain full traces. Candidate chains and compact rows
    // preserve every legal proposal, including rejects, without16 full traces.
    let full_bytes = serde_json::to_vec(&selected_canonical)?.len()
        + serde_json::to_vec(&selected_generation)?.len();
    let compact_bytes =
        serde_json::to_vec(&baseline)?.len() + serde_json::to_vec(&baseline_metrics)?.len();
    let projection = (full_bytes as u64)
        .saturating_mul(2)
        .saturating_add((compact_bytes as u64 + 2 * 1024 * 1024).saturating_mul(MAX_TRIALS as u64))
        .saturating_add(128 * 1024 * 1024);
    write_json(
        &a.out,
        "storage-projection.json",
        &json!({"maximum_trials":MAX_TRIALS,"full_baseline_and_final_bytes":full_bytes*2,"compact_per_trial_observed_bytes":compact_bytes,"projection_with_chain_reserve_and_stop_margin_bytes":projection,"maximum_report_bytes":a.maximum_report_bytes,"maximum_prior_compact_artifacts":MAX_TRIALS+1,"prior_compact_memory_observed_bytes":compact_bytes*(MAX_TRIALS+1),"comparison_reserve_per_trial_bytes":2*1024*1024}),
    )?;
    if projection > a.maximum_report_bytes as u64 {
        return Err(invalid("joint fit projected report cap before learning").into());
    }
    write_json(&initial, "canonical.json", &selected_canonical)?;
    write_json(&initial, "generation.json", &selected_generation)?;
    write_json(&initial, "causal-metrics.json", &baseline_metrics)?;
    report_output::seal(&initial)?;
    report_output::verify(&initial)?;
    let indices: Vec<_> = (0..512).collect();
    let mut gradient: Option<Vec<f32>> = None;
    let mut gradient_report = Value::Null;
    let mut rejected = BTreeSet::new();
    let mut trials = Vec::new();
    let mut accepted = 0usize;
    let mut selected_root = initial.clone();
    for trial in 1..=MAX_TRIALS {
        deadline(a, start)?;
        if accepted == MAX_ACCEPTED {
            break;
        }
        let packed = current
            .joint_packed_coefficients()?
            .ok_or_else(|| invalid("joint fit packed absent"))?;
        let q = unpack_coefficients(16, &packed).map_err(|e| invalid(e.to_string()))?;
        if gradient.is_none() {
            let b = batch(
                &indices,
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
            if b.gradients.len() != 1 {
                return Err(invalid("joint fit gradient family count differs").into());
            }
            let params = current.joint_parameters();
            let (name, _) = params
                .iter()
                .next()
                .ok_or_else(|| invalid("joint fit Var absent"))?;
            gradient = Some(
                b.gradients
                    .get(name)
                    .ok_or_else(|| invalid("joint fit gradient family absent"))?
                    .flatten_all()?
                    .to_vec1::<f32>()?,
            );
            gradient_report = b.report;
        }
        let gs = gradient
            .as_ref()
            .ok_or_else(|| invalid("joint fit cached gradient absent"))?;
        let Some((index, step)) = coordinate(&q, gs, &rejected)? else {
            break;
        };
        let proposal = CueAngularWeights::from_native(
            parent,
            &a.native_artifact,
            &parent.compile_cue_carrier(current.native()?)?,
        )?;
        let params = proposal.joint_parameters();
        let (_, var) = params
            .iter()
            .next()
            .ok_or_else(|| invalid("joint fit proposal Var absent"))?;
        let mut next = q.clone();
        next[index] += step;
        var.set(&Tensor::from_vec(
            next.iter()
                .map(|&x| f32::from(x) * 0.25)
                .collect::<Vec<_>>(),
            16,
            var.device(),
        )?)?;
        if proposal.packed_coefficients()? != unary {
            return Err(invalid("joint fit proposal changed unary").into());
        }
        let root = a.out.join(format!("trial-{trial:04}"));
        report_output::claim(&root)?;
        save_chain(&root, &proposal, parent, f)?;
        let restored = CueAngularWeights::load(&root.join("cue"), parent, &a.native_artifact)?;
        if parameter_receipts(&restored.parameters())?
            != parameter_receipts(&proposal.parameters())?
        {
            return Err(invalid("joint fit independent shadow reload differs").into());
        }
        let restored_packed = restored
            .joint_packed_coefficients()?
            .ok_or_else(|| invalid("joint fit restored packed absent"))?;
        if unpack_coefficients(16, &restored_packed).map_err(|e| invalid(e.to_string()))? != next {
            return Err(invalid("joint fit proposal changed other quarter or exact delta").into());
        }
        let (nc, np, ne) = load_chain(&root, integer)?;
        let canonical = source_end_fit::canonical(integer, &nc, &np, &ne, development, a, start)?;
        let generation =
            source_end_fit::generation(integer, &nc, &np, &ne, development, tok, a, start)?;
        let candidate = objective(&canonical, &generation)?;
        let summary = compact(&canonical, &generation, development)?;
        let metrics = causal_metrics(&a.development_panel, development, &canonical, &generation)?;
        let comparisons = prior_artifacts
            .iter()
            .map(|(path, rows, prior_metrics)| -> Result<Value> {
                Ok(
                    json!({"prior_artifact":path,"complete_comparison":gains(rows,&summary)?,
                "causal_comparison":causal_comparison(prior_metrics,&metrics)?}),
                )
            })
            .collect::<Result<Vec<_>>>()?;
        let accept = improves(best.0, best.1, candidate.0, candidate.1);
        let rec = json!({"trial":trial,"incumbent_root":selected_root,"incumbent_joint_packed_sha256":sha256_bytes(&packed),"coordinate":index,"signed_quarter_step":step,"gradient_values":gs,"gradient_report":gradient_report,"predicted_local_ce_delta":f64::from(gs[index])*f64::from(step)*0.25,"realized_ce_delta":candidate.1-best.1,"incumbent_complete":best.0,"proposal_complete":candidate.0,"proposal_native_ce":candidate.1,"proposal_objective_finite":candidate.1.is_finite(),"accepted":accept,"comparison_to_incumbent":gains(&selected_compact,&summary)?,"comparison_to_baseline":gains(&baseline,&summary)?,"comparisons_to_all_prior_artifacts":comparisons,"postprediction_causal_counts":metrics["counts"],"joint_quarters":next,"joint_packed_sha256":sha256_bytes(&restored.joint_packed_coefficients()?.ok_or_else(|| invalid("joint fit proposal packed absent"))?),"frozen_unary_sha256":sha256_bytes(&unary),"frozen_payloads":f.hashes(),"independent_native_reload":true});
        write_json(&root, "rows.json", &summary)?;
        write_json(&root, "causal-metrics.json", &metrics)?;
        write_json(&root, "receipt.json", &rec)?;
        frozen(source, receipts)?;
        immutable(inputs, seals)?;
        report_output::seal(&root)?;
        report_output::verify(&root)?;
        trials.push(rec);
        // Rejected artifacts remain comparators at every later trial too.
        prior_artifacts.push((root.clone(), summary.clone(), metrics));
        if accept {
            accepted += 1;
            current = restored;
            best = candidate;
            selected_root = root;
            selected_compact = summary;
            selected_canonical = canonical;
            selected_generation = generation;
            rejected.clear();
            gradient = None;
        } else {
            rejected.insert(index);
        }
    }
    write_json(
        &a.out,
        "selected-development-canonical.json",
        &selected_canonical,
    )?;
    write_json(
        &a.out,
        "selected-development-generation.json",
        &selected_generation,
    )?;
    frozen(source, receipts)?;
    immutable(inputs, seals)?;
    if current.packed_coefficients()? != unary
        || parameter_receipts(&donor.parameters())? != donor_receipts
    {
        return Err(invalid("joint fit frozen unary donor changed").into());
    }
    Ok(
        json!({"schema":"uor-r4.geometric-cue-joint-discrete-fit/1","status":"completed","cases":512,"joint_config":config,"maximum_trials":MAX_TRIALS,"maximum_accepted_updates":MAX_ACCEPTED,"trials":trials,"selected_chain":selected_root,"selected_complete":best.0,"selected_native_ce":best.1,"baseline_complete":baseline_objective.0,"baseline_native_ce":baseline_objective.1,"optimizer_updates":accepted,"accepted_discrete_updates":accepted,"adam_updates":0,"heldout_predictions":0,"baseline_eligible":true,"selection":"lexicographic actual complete ownprefix count then ordinary full native CE epsilon1e-9; no individual-row preservation veto","frozen_unary_sha256":sha256_bytes(&unary),"frozen_payloads":f.hashes(),"source_parameter_receipts":receipts,"input_manifests_sha256":inputs,"comparison_to_baseline":gains(&baseline,&selected_compact)?,"elapsed_seconds":start.elapsed().as_secs_f64(),"limitation":"development-only discrete trajectory-selected learning; teacherforced gradient proposals are not differentiated rollout credit; no transfer, chat qualification or adoption"}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn trajectory_selection_allows_tradeoffs_but_requires_finite_ce() {
        assert!(improves(59, 1., 60, 2.));
        assert!(improves(59, 1., 59, 0.9));
        assert!(!improves(59, 1., 58, 0.5));
        assert!(!improves(59, 1., 60, f64::NAN));
        assert!(!improves(59, 1., 59, 1. - 0.5e-9));
    }
    #[test]
    fn legal_gradient_ties_and_rejected_head_scope() -> Result<()> {
        let mut q = vec![0; 16];
        let mut g = vec![0f32; 16];
        g[2] = 2.;
        g[3] = -2.;
        assert_eq!(coordinate(&q, &g, &BTreeSet::new())?, Some((2, -1)));
        let rejected = BTreeSet::from([2]);
        assert_eq!(coordinate(&q, &g, &rejected)?, Some((3, 1)));
        q[3] = 7;
        assert_eq!(coordinate(&q, &g, &rejected)?, None);
        assert_eq!(
            coordinate(&vec![0; 16], &g, &BTreeSet::new())?,
            Some((2, -1))
        );
        Ok(())
    }
}
