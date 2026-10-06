//! Frozen four-arm joint-channel evaluation. No gradients, fitting or selection.
use super::*;
use uor_r4_integer::geometric_cue_carrier::{CueAngularQ4, CueJointConfig, CueJointQ4};

fn configs() -> (CueJointConfig, CueJointConfig) {
    (
        CueJointConfig {
            head: 1,
            left_lane: 1,
            right_lane: 3,
        },
        CueJointConfig {
            head: 1,
            left_lane: 3,
            right_lane: 1,
        },
    )
}
fn arm_weights(
    name: &str,
    donor: &CueAngularWeights,
    parent: &NativeSourceRealizer,
    native_root: &Path,
    unary: &[u8],
    learned: &[u8],
) -> Result<CueAngularWeights> {
    let (forward, reverse) = configs();
    let angular = CueAngularQ4::new(donor.config(), unary).map_err(|e| invalid(e.to_string()))?;
    let angular = match name {
        "original-unary" | "zero-joint" => angular,
        "learned-joint" => angular
            .with_joint(
                CueJointQ4::new(forward, learned.to_vec()).map_err(|e| invalid(e.to_string()))?,
            )
            .map_err(|e| invalid(e.to_string()))?,
        "reversed-order" => angular
            .with_joint(
                CueJointQ4::new(reverse, learned.to_vec()).map_err(|e| invalid(e.to_string()))?,
            )
            .map_err(|e| invalid(e.to_string()))?,
        _ => return Err(invalid("unknown frozen joint evaluation arm").into()),
    };
    let carrier = parent.compile_cue_carrier(angular)?;
    let weights = CueAngularWeights::from_native(parent, native_root, &carrier)?;
    if name == "zero-joint" {
        Ok(weights.with_zero_joint(parent, forward)?)
    } else {
        Ok(weights)
    }
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
    _development: &[Episode],
    inputs: &BTreeMap<String, String>,
    seals: &BTreeSet<PathBuf>,
    receipts: &Value,
) -> Result<Value> {
    let (forward, reverse) = configs();
    if donor.joint_config() != Some(forward) {
        return Err(invalid(
            "frozen joint evaluation requires learned ordered pair head1/lane1/lane3",
        )
        .into());
    }
    let unary = donor.packed_coefficients()?;
    let learned = donor
        .joint_packed_coefficients()?
        .ok_or_else(|| invalid("learned joint packed payload absent"))?;
    let donor_receipts = parameter_receipts(&donor.parameters())?;
    let episodes = panel(&a.fresh_panel, 128, integer, tok, a)?;
    let panel_report = read_json(&a.fresh_panel.join("report.json"))?;
    if episodes.len() != 128 {
        return Err(invalid("joint frozen panel requires all128 rows").into());
    }
    let mut prior: Vec<(String, Value)> = Vec::new();
    let mut arms = Vec::new();
    let mut original: Option<(Value, Value)> = None;
    let mut zero_equal = false;
    let mut observed_full_bytes = 0usize;
    for (ordinal, name) in [
        "original-unary",
        "zero-joint",
        "learned-joint",
        "reversed-order",
    ]
    .into_iter()
    .enumerate()
    {
        deadline(a, start)?;
        // Observed-shape storage guard before subsequent arms. The configured
        // cap remains authoritative if actual generated trajectories grow.
        if ordinal > 0 {
            let required = directory_bytes(&a.out)?
                .saturating_add(observed_full_bytes.saturating_mul(4 - ordinal))
                .saturating_add(128 * 1024 * 1024);
            if required > a.maximum_report_bytes {
                return Err(
                    invalid("frozen joint remaining-arm storage projection exceeds cap").into(),
                );
            }
        }
        let weights = arm_weights(name, donor, parent, &a.native_artifact, &unary, &learned)?;
        let expected_joint = match name {
            "original-unary" => None,
            "reversed-order" => Some(reverse),
            _ => Some(forward),
        };
        if weights.joint_config() != expected_joint
            || weights.packed_coefficients()? != unary
            || ((name == "learned-joint" || name == "reversed-order")
                && weights.joint_packed_coefficients()?.as_deref() != Some(learned.as_slice()))
        {
            return Err(invalid("frozen joint arm mutated configuration or coefficients").into());
        }
        let root = a.out.join(name);
        report_output::claim(&root)?;
        save_chain(&root, &weights, parent, f)?;
        let restored = CueAngularWeights::load(&root.join("cue"), parent, &a.native_artifact)?;
        if parameter_receipts(&weights.parameters())? != parameter_receipts(&restored.parameters())?
        {
            return Err(invalid("frozen joint arm independent source reload differs").into());
        }
        let (c, p, e) = load_chain(&root, integer)?;
        if c.packed_coefficients() != unary
            || p.packed_coefficients() != f.prefix
            || e.period_packed_coefficients() != f.period
            || e.stop_packed_coefficients() != f.stop
        {
            return Err(invalid("frozen joint arm native payload changed").into());
        }
        let canonical = source_end_fit::canonical(integer, &c, &p, &e, &episodes, a, start)?;
        let generation = source_end_fit::generation(integer, &c, &p, &e, &episodes, tok, a, start)?;
        let metrics = causal_metrics(&a.fresh_panel, &episodes, &canonical, &generation)?;
        if name == "original-unary" {
            observed_full_bytes = serde_json::to_vec(&canonical)?.len()
                + serde_json::to_vec(&generation)?.len()
                + serde_json::to_vec(&metrics)?.len();
            original = Some((
                joint_probe::numerical(canonical.clone())?,
                joint_probe::numerical(generation.clone())?,
            ));
        } else if name == "zero-joint" {
            let (oc, og) = original
                .as_ref()
                .ok_or_else(|| invalid("frozen joint original replay absent"))?;
            zero_equal = *oc == joint_probe::numerical(canonical.clone())?
                && *og == joint_probe::numerical(generation.clone())?;
            if !zero_equal {
                return Err(invalid(
                    "frozen zero overlay numerical/state/route/mass/output replay differs",
                )
                .into());
            }
            original = None;
        }
        let comparisons = prior
            .iter()
            .map(|(old, m)| -> Result<Value> {
                Ok(json!({"prior_arm":old,"causal_comparison":causal_comparison(m,&metrics)?}))
            })
            .collect::<Result<Vec<_>>>()?;
        let rec = json!({"arm":name,"cases":128,"joint_config":restored.joint_config(),"joint_packed_sha256":restored.joint_packed_coefficients()?.as_ref().map(|b|sha256_bytes(b)),"cue_native_metadata":c.metadata(),"source_parameter_receipts":receipts,"arm_source_parameter_receipts":parameter_receipts(&restored.parameters())?,"frozen_unary_sha256":sha256_bytes(&unary),"frozen_payloads":f.hashes(),"native_equal_episode_ce":canonical["native_equal_episode_ce"],"zero_support_positions":canonical["zero_support_positions"],"causal_counts":metrics["counts"],"comparisons_to_all_prior_arms":comparisons,"independent_source_native_reload":true,"optimizer_updates":0,"selection":"NONE"});
        write_json(&root, "canonical.json", &canonical)?;
        write_json(&root, "generation.json", &generation)?;
        write_json(&root, "causal-metrics.json", &metrics)?;
        write_json(&root, "receipt.json", &rec)?;
        frozen(source, receipts)?;
        immutable(inputs, seals)?;
        if parameter_receipts(&donor.parameters())? != donor_receipts {
            return Err(invalid("frozen learned donor changed during evaluation").into());
        }
        report_output::seal(&root)?;
        report_output::verify(&root)?;
        prior.push((name.to_owned(), metrics));
        arms.push(rec);
    }
    Ok(
        json!({"schema":"uor-r4.geometric-cue-joint-frozen-evaluation/1","status":"completed","cases_per_arm":128,"arms":arms,"evaluation_panel":a.fresh_panel,"evaluation_manifest_sha256":a.fresh_manifest_sha256,"panel_report":panel_report,"panel_exposure_status":"not inferred by evaluator; caller must identify exposed diagnostic versus prospectively frozen panel","development_predictions":0,"optimizer_updates":0,"gradient_passes":0,"selected_arm":null,"adoption":"NONE","zero_overlay_numerical_replay_equal":zero_equal,"ordered_config":forward,"reversed_config":reverse,"reversal_policy":"same16packedcoefficients; swaporderedlanes only; no transpose/refit","learned_joint_packed_sha256":sha256_bytes(&learned),"frozen_unary_sha256":sha256_bytes(&unary),"frozen_payloads":f.hashes(),"frozen_donor_source_parameter_receipts":donor_receipts,"input_manifests_sha256":inputs,"elapsed_seconds":start.elapsed().as_secs_f64(),"limitation":"frozen panel evaluation only; no learning, selection or general chat qualification; all native operation/state/cost traces retained per arm"}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn reversal_changes_order_without_transposing_packed_coefficients() -> Result<()> {
        let (a, b) = configs();
        let bytes = vec![0x12, 0x34, 0x56, 0x70, 0x12, 0x34, 0x56, 0x70];
        let f = CueJointQ4::new(a, bytes.clone()).map_err(|e| invalid(e.to_string()))?;
        let r = CueJointQ4::new(b, bytes.clone()).map_err(|e| invalid(e.to_string()))?;
        assert_eq!(f.packed_coefficients(), r.packed_coefficients());
        assert_ne!(f.config(), r.config());
        assert_eq!(
            f.metadata().config.left_lane,
            r.metadata().config.right_lane
        );
        assert_eq!(
            f.metadata().config.right_lane,
            r.metadata().config.left_lane
        );
        Ok(())
    }
}
