//! Frozen-head sibling support proposals, selected by actual native CE.
use super::*;
use uor_r4_integer::geometric_cue_carrier::CueAngularQ4;
use uor_r4_integer::geometric_potential_q4::{pack_coefficients, unpack_coefficients};

fn ranking(values: &[i8], means: &[f64]) -> Result<Vec<(usize, i8, f64)>> {
    if values.len() != means.len()
        || means.iter().any(|x| !x.is_finite())
        || values.iter().any(|x| !(-7..=7).contains(x))
    {
        return Err(invalid("support gradient/value shape or finite bounds differ").into());
    }
    let mut out = Vec::new();
    for (index, (&value, &mean)) in values.iter().zip(means).enumerate() {
        if mean == 0. {
            continue;
        }
        let step = if mean > 0. { -1 } else { 1 };
        if value
            .checked_add(step)
            .is_some_and(|v| (-7..=7).contains(&v))
        {
            out.push((index, step, mean));
        }
    }
    out.sort_by(|a, b| b.2.abs().total_cmp(&a.2.abs()).then(a.0.cmp(&b.0)));
    Ok(out)
}

pub(super) fn validate_binding(
    a: &Args,
    weights: &CueAngularWeights,
    f: &Frozen,
    receipts: &Value,
    inputs: &BTreeMap<String, String>,
) -> Result<Value> {
    let c = a
        .cue_support_probe
        .as_ref()
        .ok_or_else(|| invalid("support config absent"))?;
    report_output::verify(&c.audit_root)?;
    if sha256_file(&c.audit_root.join("manifest.json"))? != c.audit_manifest_sha256 {
        return Err(invalid("support audit manifest changed").into());
    }
    let r = read_json(&c.audit_root.join("report.json"))?;
    if r["schema"] != "uor-r4.geometric-cue-credit-audit/1"
        || r["status"] != "completed"
        || r["cases"] != 512
        || r["coefficient_width"] != weights.config().coefficient_count()?
        || r["optimizer_updates"] != 0
        || r["proposal_count"] != 0
        || r["evaluation_predictions"] != 0
        || r["gradient_passes"] != 513
        || r["positive_control"]["row_average_matches_fullbatch"] != true
        || r["initial_cue_packed_sha256"] != sha256_bytes(&weights.packed_coefficients()?)
        || r["frozen_cue_parameter_receipts"] != parameter_receipts(&weights.parameters())?
        || r["frozen_source_receipts"] != *receipts
        || r["frozen_payloads"] != f.hashes()
    {
        return Err(invalid("support audit frozen checkpoint/positive control differs").into());
    }
    for (path, hash) in inputs {
        if r["input_manifests_sha256"][path] != *hash {
            return Err(invalid("support audit source/panel/input identity differs").into());
        }
    }
    Ok(r)
}

pub(super) fn run(
    a: &Args,
    start: Instant,
    source: &SourceRealizerWeights,
    parent: &NativeSourceRealizer,
    integer: &IntegerRealizer,
    tok: &ByteBpeTokenizer,
    weights: &CueAngularWeights,
    f: &Frozen,
    development: &[Episode],
    baseline: &Value,
    parent_gen: &Value,
    inputs: &BTreeMap<String, String>,
    seals: &BTreeSet<PathBuf>,
    receipts: &Value,
) -> Result<Value> {
    let cfg = a
        .cue_support_probe
        .as_ref()
        .ok_or_else(|| invalid("support config absent"))?;
    let audit = validate_binding(a, weights, f, receipts, inputs)?;
    let original_receipts = parameter_receipts(&weights.parameters())?;
    let original = weights.packed_coefficients()?;
    let count = weights.config().coefficient_count()?;
    let values = unpack_coefficients(count, &original).map_err(|e| invalid(e.to_string()))?;
    let credit_path = cfg.audit_root.join("coefficient-credit.json");
    let credit_sha = sha256_file(&credit_path)?;
    let credit = read_json(&credit_path)?;
    let entries = credit["coefficients"]
        .as_array()
        .ok_or_else(|| invalid("support credit absent"))?;
    if entries.len() != count {
        return Err(invalid("support credit width differs").into());
    }
    let mut means = Vec::new();
    for (i, entry) in entries.iter().enumerate() {
        if entry["coefficient_index"] != i
            || entry["lane"] != i / 120
            || entry["angular_bin"] != i % 120
        {
            return Err(invalid("support credit ordering differs").into());
        }
        means.push(
            entry["mean"]
                .as_f64()
                .filter(|v| v.is_finite())
                .ok_or_else(|| invalid("support signed mean absent/nonfinite"))?,
        );
    }
    let ranked = ranking(&values, &means)?;
    if ranked.len() < 8 {
        return Err(invalid("support predeclared eight legal coordinates unavailable").into());
    }
    let baseline_ce = baseline["native_equal_episode_ce"]
        .as_f64()
        .filter(|v| v.is_finite())
        .ok_or_else(|| invalid("support baseline CE absent/nonfinite"))?;
    if (baseline_ce
        - audit["reference_native_ce"]
            .as_f64()
            .ok_or_else(|| invalid("support reference CE absent"))?)
    .abs()
        > 1e-10
    {
        return Err(invalid("support baseline native CE differs from frozen audit").into());
    }
    // Baseline + two full canonical/generation arms + two 128-row diagnostic arms.
    let base_bytes = serde_json::to_vec(baseline)?.len() + serde_json::to_vec(parent_gen)?.len();
    let projected = base_bytes
        .saturating_mul(4)
        .saturating_add(32 * 1024 * 1024);
    if projected.saturating_add(1024 * 1024) > a.maximum_report_bytes {
        return Err(invalid("support observed complete report projection exceeds cap").into());
    }
    write_json(
        &a.out,
        "proposal-plan.json",
        &json!({"audit_manifest_sha256":cfg.audit_manifest_sha256,
        "coefficient_credit_sha256":credit_sha,"ranking":"legal abs signedmean descending, indexascending ties",
        "arms":[1,8],"parent_cue_packed_sha256":sha256_bytes(&original),"ranked_coordinates":ranked,
        "baseline_eligible":true,"fresh_predictions_before_selection":0,"gradient_passes":0,
        "dose_caveat":"eight coordinates has eightfold coefficient L1 quarter dose; cannot isolate coverage from dose"}),
    )?;
    let baseline_metrics = causal_metrics(&a.development_panel, development, baseline, parent_gen)?;
    let baseline_summary =
        discrete::source_summary(&a.development_panel, development, baseline, integer)?;
    let mut selected = "initial-chain".to_string();
    let mut best = baseline_ce;
    let mut prior = vec![(
        "baseline".to_string(),
        baseline_metrics.clone(),
        baseline_summary.clone(),
    )];
    let mut arms =
        vec![json!({"name":"baseline","native_ce":baseline_ce,"causal_outcomes":baseline_metrics})];
    for support in [1usize, 8] {
        let name = format!("support-{support:02}");
        let mut proposed = values.clone();
        for &(index, step, _) in &ranked[..support] {
            proposed[index] += step;
        }
        let packed = pack_coefficients(&proposed).map_err(|e| invalid(e.to_string()))?;
        let compiled = parent.compile_cue_carrier(
            CueAngularQ4::new(weights.config(), &packed).map_err(|e| invalid(e.to_string()))?,
        )?;
        let proposal = CueAngularWeights::from_native(parent, &a.native_artifact, &compiled)?;
        let root = a.out.join(&name);
        report_output::claim(&root)?;
        save_chain(&root, &proposal, parent, f)?;
        let restored = CueAngularWeights::load(&root.join("cue"), parent, &a.native_artifact)?;
        if parameter_receipts(&proposal.parameters())?
            != parameter_receipts(&restored.parameters())?
        {
            return Err(invalid("support proposal shadow reload differs").into());
        }
        let (c, p, e) = load_chain(&root, integer)?;
        let loaded = unpack_coefficients(count, c.packed_coefficients())
            .map_err(|e| invalid(e.to_string()))?;
        if loaded != proposed || c.packed_coefficients() != packed {
            return Err(invalid("support independent exact coefficient delta differs").into());
        }
        let canonical = source_end_fit::canonical(integer, &c, &p, &e, development, a, start)?;
        let generation =
            source_end_fit::generation(integer, &c, &p, &e, development, tok, a, start)?;
        let ce = canonical["native_equal_episode_ce"]
            .as_f64()
            .filter(|v| v.is_finite())
            .ok_or_else(|| invalid("support proposal native CE absent/nonfinite"))?;
        let metrics = causal_metrics(&a.development_panel, development, &canonical, &generation)?;
        let summary =
            discrete::source_summary(&a.development_panel, development, &canonical, integer)?;
        let metrics_new = metrics.clone();
        let summary_new = summary.clone();
        let comparisons = prior.iter().map(|(name,metrics,summary)| {
            Ok(json!({"prior_arm":name,"causal_comparison":causal_comparison(metrics,&metrics_new)?,
                "source_changes":discrete::source_changes(summary,&summary_new)?}))
        }).collect::<Result<Vec<Value>>>()?;
        let outcome = json!({"name":name,"support":support,"native_ce":ce,"native_ce_delta":ce-baseline_ce,
            "coordinates":&ranked[..support],"predicted_linear_ce_delta":ranked[..support].iter().map(|(_,s,g)| f64::from(*s)*g/4.).sum::<f64>(),
            "coefficient_L1_quarter_dose":support,"cue_packed_sha256":sha256_bytes(&packed),
            "exact_independent_coefficient_delta_verified":true,"independent_native_reload":true,
            "causal_outcomes":metrics,"comparison_vs_baseline":causal_comparison(&baseline_metrics,&metrics)?,
            "source_changes_vs_baseline":discrete::source_changes(&baseline_summary,&summary)?,"comparisons_vs_all_prior_arms":comparisons,"frozen_payloads":f.hashes()});
        write_json(&root, "canonical.json", &canonical)?;
        write_json(&root, "generation.json", &generation)?;
        write_json(&root, "receipt.json", &outcome)?;
        report_output::seal(&root)?;
        report_output::verify(&root)?;
        if ce < best - 1e-9 {
            selected = name.clone();
            best = ce;
        }
        prior.push((name.clone(), metrics, summary));
        arms.push(outcome);
        frozen(source, receipts)?;
    }
    write_json(
        &a.out,
        "selection-before-fresh.json",
        &json!({"selected_arm":selected,"selected_native_ce":best,
        "criterion":"finite full512 native CE, strict improvement1e-9, baseline then support1 then support8",
        "fresh_predictions_before_selection":0,"gradient_passes":0,"adam_updates":0}),
    )?;
    let evaluation = panel(&a.fresh_panel, 128, integer, tok, a)?;
    let mut diagnostic = Vec::new();
    for (name, chain) in [("parent", "initial-chain"), ("selected", selected.as_str())] {
        let (c, p, e) = load_chain(&a.out.join(chain), integer)?;
        let canonical = source_end_fit::canonical(integer, &c, &p, &e, &evaluation, a, start)?;
        let generation =
            source_end_fit::generation(integer, &c, &p, &e, &evaluation, tok, a, start)?;
        write_json(&a.out, &format!("fresh-{name}-canonical.json"), &canonical)?;
        write_json(
            &a.out,
            &format!("fresh-{name}-generation.json"),
            &generation,
        )?;
        diagnostic.push(causal_metrics(
            &a.fresh_panel,
            &evaluation,
            &canonical,
            &generation,
        )?);
    }
    write_json(
        &a.out,
        "fresh-causal-outcomes.json",
        &json!({"parent":diagnostic[0],"selected":diagnostic[1],
        "comparison":causal_comparison(&diagnostic[0],&diagnostic[1])?,"scope":"already exposed128, postselection only"}),
    )?;
    if parameter_receipts(&weights.parameters())? != original_receipts {
        return Err(invalid("support frozen cue changed").into());
    }
    frozen(source, receipts)?;
    immutable(inputs, seals)?;
    report_output::verify(&cfg.audit_root)?;
    if sha256_file(&cfg.audit_root.join("manifest.json"))? != cfg.audit_manifest_sha256
        || sha256_file(&credit_path)? != credit_sha
    {
        return Err(invalid("support audit evidence changed").into());
    }
    Ok(
        json!({"schema":"uor-r4.geometric-cue-support-probe/1","mode":a.mode,"status":"completed",
        "cases":512,"evaluation_cases":128,"proposal_count":2,"gradient_passes":0,"adam_updates":0,
        "optimizer_updates":0,"selected_arm":selected,"selected_native_ce":best,"arms":arms,
        "cue_support_probe":cfg,"audit_coefficient_credit_sha256":credit_sha,"input_manifests_sha256":inputs,
        "frozen_source_receipts":receipts,"frozen_payloads":f.hashes(),"storage_projection_bytes":projected,
        "fresh_predictions_before_selection":0,"elapsed_seconds":start.elapsed().as_secs_f64(),
        "peak_rss_kib_linux":peak_rss_kib(),"claim":"frozenhead broader finitequarter discriminator; dose and support confounded; no transfer/chat/representation impossibility conclusion"}),
    )
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn support_ranking_filters_boundaries_and_uses_signed_mean_not_absolute_credit() -> Result<()> {
        assert_eq!(
            ranking(&[-7, 7, 0, 0, 0], &[100., -100., 2., -2., 0.])?,
            vec![(2, -1, 2.), (3, 1, -2.)]
        );
        assert!(ranking(&[0], &[f64::NAN]).is_err());
        assert!(ranking(&[8], &[1.]).is_err());
        assert!(ranking(&[0], &[]).is_err());
        Ok(())
    }
}
