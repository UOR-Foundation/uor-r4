//! Four frozen development arms for unary/joint local cue sensitivity.
//! Separate anchored losses supply disjoint credit; no fit or arm selection.
use super::*;
use uor_r4_integer::geometric_cue_carrier::CueJointConfig;
use uor_r4_integer::geometric_potential_q4::unpack_coefficients;

fn coordinate(q: &[i8], g: &[f32]) -> Result<Option<(usize, i8)>> {
    if q.is_empty() || q.len() != g.len() || g.iter().any(|x| !x.is_finite()) {
        return Err(invalid("coadapt quarter/gradient shape or finite contract differs").into());
    }
    let mut best = None;
    for i in 0..q.len() {
        if !(-7..=7).contains(&q[i]) {
            return Err(invalid("coadapt incumbent quarter outside signed Q4").into());
        }
        let step = if g[i] > 0. { -1 } else { 1 };
        if g[i] == 0. || !(-7..=7).contains(&(q[i] + step)) {
            continue;
        }
        if best.is_none_or(|(j, _): (usize, i8)| g[i].abs() > g[j].abs()) {
            best = Some((i, step));
        }
    }
    Ok(best)
}

fn set_family(weights: &CueAngularWeights, name: &str, quarters: &[i8]) -> Result<()> {
    let params = weights.parameters();
    let var = params
        .get(name)
        .ok_or_else(|| invalid("coadapt proposal family absent"))?;
    if var.elem_count() != quarters.len() {
        return Err(invalid("coadapt proposal family shape differs").into());
    }
    var.set(&Tensor::from_vec(
        quarters
            .iter()
            .map(|&x| f32::from(x) * 0.25)
            .collect::<Vec<_>>(),
        quarters.len(),
        var.device(),
    )?)?;
    Ok(())
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
    let config = CueJointConfig {
        head: 1,
        left_lane: 1,
        right_lane: 3,
    };
    if donor.joint_config() != Some(config) || development.len() != 512 {
        return Err(invalid(
            "coadapt requires selected ordered joint donor and full512 development",
        )
        .into());
    }
    let donor_receipts = parameter_receipts(&donor.parameters())?;
    let unary = donor.packed_coefficients()?;
    let joint = donor
        .joint_packed_coefficients()?
        .ok_or_else(|| invalid("coadapt donor joint absent"))?;
    let unary_count = donor
        .parameters()
        .get("cue.coefficients")
        .ok_or_else(|| invalid("coadapt unary Var absent"))?
        .elem_count();
    let uq = unpack_coefficients(unary_count, &unary).map_err(|e| invalid(e.to_string()))?;
    let jq = unpack_coefficients(16, &joint).map_err(|e| invalid(e.to_string()))?;
    let prepared = source.prepare(parent)?;
    let c = parent.compile_cue_carrier(donor.native()?)?;
    let (p, e) = f.training(parent, &c)?;
    let ic = integer.compile_cue_carrier(donor.native()?)?;
    let (ip, ie) = f.integer(integer, &ic)?;
    let params = donor.parameters();
    if params.len() != 2 {
        return Err(invalid("coadapt donor coefficient families differ").into());
    }
    let parentvars = source.parameters();
    let mut gradients = BTreeMap::<String, Tensor>::new();
    let mut mean = 0f64;
    let mut positions = 0usize;
    for row in development {
        if row.target.is_empty() {
            return Err(invalid("coadapt empty target episode").into());
        }
        let segments = row.segments()?;
        let mut ce = 0f64;
        for (step, &target) in row.target.iter().enumerate() {
            deadline(a, start)?;
            let u = prepared.loss_bank_cue_source_end(
                &segments,
                &row.packet.query_ids,
                &row.target[..step],
                target,
                donor,
                &c,
                &p,
                &e,
            )?;
            let j = prepared.loss_bank_cue_joint_source_end(
                &segments,
                &row.packet.query_ids,
                &row.target[..step],
                target,
                donor,
                &c,
                &p,
                &e,
            )?;
            if u.trace != j.trace
                || u.target_probability != j.target_probability
                || u.trace
                    != integer.read_bank_with_source_end_transport(
                        &segments,
                        &row.packet.query_ids,
                        &row.target[..step],
                        &ic,
                        &ip,
                        &ie,
                    )?
            {
                return Err(
                    invalid("coadapt separate credit native traces/probability differ").into(),
                );
            }
            let native_ce = -u.target_probability.ln();
            for (name, out) in [("cue.coefficients", u), ("cue.joint.coefficients", j)] {
                let scalar = f64::from(out.loss.to_scalar::<f32>()?);
                if !native_ce.is_finite()
                    || !scalar.is_finite()
                    || (scalar - native_ce).abs() > 1e-4 + 1e-5 * native_ce.abs()
                {
                    return Err(invalid("coadapt anchored native likelihood differs").into());
                }
                let store =
                    (&out.loss * scale(development.len(), row.target.len())?)?.backward()?;
                if parentvars
                    .values()
                    .any(|v| store.get(v.as_tensor()).is_some())
                    || params
                        .iter()
                        .any(|(n, v)| n != name && store.get(v.as_tensor()).is_some())
                {
                    return Err(invalid(
                        "coadapt gradient connects frozen parent/other cue family",
                    )
                    .into());
                }
                let var = params
                    .get(name)
                    .ok_or_else(|| invalid("coadapt gradient Var absent"))?;
                let g = store
                    .get(var.as_tensor())
                    .ok_or_else(|| invalid("coadapt coefficient gradient absent"))?
                    .detach();
                let next = if let Some(old) = gradients.remove(name) {
                    (&old + &g)?.detach()
                } else {
                    g
                };
                gradients.insert(name.into(), next);
            }
            ce += native_ce;
            positions += 1;
        }
        mean += ce / row.target.len() as f64 / development.len() as f64;
    }
    let mut vectors = BTreeMap::new();
    for (name, g) in gradients {
        vectors.insert(name, g.flatten_all()?.to_vec1::<f32>()?);
    }
    let ug = vectors
        .get("cue.coefficients")
        .ok_or_else(|| invalid("coadapt unary gradient missing"))?;
    let jg = vectors
        .get("cue.joint.coefficients")
        .ok_or_else(|| invalid("coadapt joint gradient missing"))?;
    let (ui, us) =
        coordinate(&uq, ug)?.ok_or_else(|| invalid("coadapt no legal nonzero unary proposal"))?;
    let (ji, js) =
        coordinate(&jq, jg)?.ok_or_else(|| invalid("coadapt no legal nonzero joint proposal"))?;
    write_json(
        &a.out,
        "gradient.json",
        &json!({"episodes":512,"target_positions":positions,
        "native_equal_episode_ce":mean,"gradient_values":vectors,
        "unary_proposal":{"coordinate":ui,"signed_quarter_step":us},
        "joint_proposal":{"coordinate":ji,"signed_quarter_step":js},
        "law":"independent largest absolute full512 mean gradient among legal opposite-sign quarter steps; lower index ties",
        "scope":"separate identical anchored ordinary native CE losses; disjoint unary/joint credit; no sum of loss scalars; parent/encoder/roots/prefix/end/factualSource argmax stopped",
        "independent_integer_trace_parity":true}),
    )?;
    let mut prior = Vec::<(PathBuf, Value, Value)>::new();
    let mut arms = Vec::new();
    let mut scores = Vec::new();
    for (name, update_u, update_j) in [
        ("baseline", false, false),
        ("unary", true, false),
        ("joint", false, true),
        ("unary-plus-joint", true, true),
    ] {
        deadline(a, start)?;
        let proposal = CueAngularWeights::from_native(
            parent,
            &a.native_artifact,
            &parent.compile_cue_carrier(donor.native()?)?,
        )?;
        let mut next_u = uq.clone();
        let mut next_j = jq.clone();
        if update_u {
            next_u[ui] += us;
            set_family(&proposal, "cue.coefficients", &next_u)?;
        }
        if update_j {
            next_j[ji] += js;
            set_family(&proposal, "cue.joint.coefficients", &next_j)?;
        }
        let root = a.out.join(name);
        report_output::claim(&root)?;
        save_chain(&root, &proposal, parent, f)?;
        let restored = CueAngularWeights::load(&root.join("cue"), parent, &a.native_artifact)?;
        let restored_joint = restored
            .joint_packed_coefficients()?
            .ok_or_else(|| invalid("coadapt restored joint absent"))?;
        if restored.joint_config() != Some(config)
            || parameter_receipts(&restored.parameters())?
                != parameter_receipts(&proposal.parameters())?
            || unpack_coefficients(unary_count, &restored.packed_coefficients()?)
                .map_err(|e| invalid(e.to_string()))?
                != next_u
            || unpack_coefficients(16, &restored_joint).map_err(|e| invalid(e.to_string()))?
                != next_j
        {
            return Err(invalid("coadapt independent reload/exact family deltas differ").into());
        }
        if !update_u && !update_j && parameter_receipts(&restored.parameters())? != donor_receipts {
            return Err(
                invalid("coadapt baseline source receipts differ from selected donor").into(),
            );
        }
        if !update_u && restored.packed_coefficients()? != unary
            || !update_j && restored_joint != joint
        {
            return Err(invalid("coadapt unchanged coefficient payload differs").into());
        }
        let (nc, np, ne) = load_chain(&root, integer)?;
        let canonical = source_end_fit::canonical(integer, &nc, &np, &ne, development, a, start)?;
        let generation =
            source_end_fit::generation(integer, &nc, &np, &ne, development, tok, a, start)?;
        let score = joint_fit::objective(&canonical, &generation)?;
        if name == "baseline" {
            if !score.1.is_finite() || (score.1 - mean).abs() > 1e-8 + 1e-8 * mean.abs() {
                return Err(invalid("coadapt baseline loss/parity differs").into());
            }
            let full =
                serde_json::to_vec(&canonical)?.len() + serde_json::to_vec(&generation)?.len();
            let projection = (full as u64)
                .saturating_mul(4)
                .saturating_add(136 * 1024 * 1024);
            write_json(
                &a.out,
                "storage-projection.json",
                &json!({"arms":4,"full_baseline_trace_bytes":full,
                "projection_with_chain_and_comparison_reserve_and_stop_margin_bytes":projection,
                "maximum_report_bytes":a.maximum_report_bytes}),
            )?;
            if projection > a.maximum_report_bytes as u64 {
                return Err(
                    invalid("coadapt projected report cap before candidate predictions").into(),
                );
            }
        }
        let summary = joint_fit::compact(&canonical, &generation, development)?;
        let metrics = causal_metrics(&a.development_panel, development, &canonical, &generation)?;
        let comparisons = prior.iter().map(|(path,rows,m)| -> Result<Value> {
            Ok(json!({"prior_artifact":path,"complete_comparison":joint_fit::gains(rows,&summary)?,
                "causal_comparison":causal_comparison(m,&metrics)?}))
        }).collect::<Result<Vec<_>>>()?;
        let rec = json!({"arm":name,"artifact":root,"unary_step":if update_u {json!([ui,us])} else {Value::Null},
            "joint_step":if update_j {json!([ji,js])} else {Value::Null},"complete":score.0,
            "native_equal_episode_ce":if score.1.is_finite(){json!(score.1)}else{Value::Null},
            "objective_finite":score.1.is_finite(),"postprediction_causal_counts":metrics["counts"],
            "comparisons_to_all_prior_arms":comparisons,"unary_packed_sha256":sha256_bytes(&restored.packed_coefficients()?),
            "joint_packed_sha256":sha256_bytes(&restored_joint),"source_parameter_receipts":parameter_receipts(&restored.parameters())?,
            "frozen_payloads":f.hashes(),"independent_native_reload":true,"selection":"NONE"});
        write_json(&root, "canonical.json", &canonical)?;
        write_json(&root, "generation.json", &generation)?;
        write_json(&root, "rows.json", &summary)?;
        write_json(&root, "causal-metrics.json", &metrics)?;
        write_json(&root, "receipt.json", &rec)?;
        frozen(source, receipts)?;
        immutable(inputs, seals)?;
        report_output::seal(&root)?;
        report_output::verify(&root)?;
        prior.push((root, summary, metrics));
        arms.push(rec);
        scores.push(score);
    }
    if parameter_receipts(&donor.parameters())? != donor_receipts {
        return Err(invalid("coadapt selected donor was mutated").into());
    }
    frozen(source, receipts)?;
    immutable(inputs, seals)?;
    let mut row_interactions = Vec::new();
    for i in 0..development.len() {
        let mut flags = Vec::new();
        for (_, rows, _) in &prior {
            let row = rows["rows"]
                .as_array()
                .and_then(|r| r.get(i))
                .ok_or_else(|| invalid("coadapt interaction row absent"))?;
            if row["id"] != development[i].packet.id {
                return Err(invalid("coadapt interaction row identity differs").into());
            }
            flags.push(i64::from(
                row["accepted_complete_answer"]
                    .as_bool()
                    .ok_or_else(|| invalid("coadapt interaction completion absent"))?,
            ));
        }
        let delta = flags[3] - flags[1] - flags[2] + flags[0];
        if delta != 0 {
            row_interactions.push(json!({"id":development[i].packet.id,
                "baseline_unary_joint_combined_complete":flags,"second_difference":delta}));
        }
    }
    let interaction_ce = if scores.iter().all(|s| s.1.is_finite()) {
        Some(scores[3].1 - scores[1].1 - scores[2].1 + scores[0].1)
    } else {
        None
    };
    Ok(
        json!({"schema":"uor-r4.geometric-cue-coadapt-sensitivity/1","status":"completed","cases":512,
        "joint_config":config,"arms":arms,"baseline_eligible":true,"selection":"NONE","adoption":"NONE",
        "optimizer_updates":0,"gradient_passes":2,"heldout_predictions":0,
        "interaction":{"ce_second_difference":interaction_ce,"nonzero_complete_row_interactions":row_interactions,"complete_second_difference":scores[3].0 as i64-scores[1].0 as i64-scores[2].0 as i64+scores[0].0 as i64,
            "definition":"U+J minus U minus J plus baseline; descriptive finite local response, not significance or synergy qualification"},
        "donor_unary_sha256":sha256_bytes(&unary),"donor_joint_sha256":sha256_bytes(&joint),
        "input_manifests_sha256":inputs,"frozen_source_receipts":receipts,"frozen_payloads":f.hashes(),
        "elapsed_seconds":start.elapsed().as_secs_f64(),
        "limitation":"four fixed development-only local sensitivity arms; independent baseline gradients, no refresh/fit/selection/128 predictions; no transfer, chat or geometric advantage claim"}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn coadapt_coordinate_lower_index_ties_and_legal_first() -> Result<()> {
        assert_eq!(coordinate(&[0, 0, 0], &[0., 2., -2.])?, Some((1, -1)));
        assert_eq!(coordinate(&[-7, 0, 7], &[9., 2., -8.])?, Some((1, -1)));
        assert_eq!(coordinate(&[0, 0], &[0., 0.])?, None);
        Ok(())
    }
    #[test]
    fn coadapt_coordinate_rejects_malformed_or_nonfinite() {
        assert!(coordinate(&[], &[]).is_err());
        assert!(coordinate(&[0], &[1., 2.]).is_err());
        assert!(coordinate(&[8], &[1.]).is_err());
        assert!(coordinate(&[0], &[f32::NAN]).is_err());
    }
}
