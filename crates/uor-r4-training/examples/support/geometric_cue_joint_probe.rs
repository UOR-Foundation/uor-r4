//! Zero-overlay replay and one native quarter-grid sensitivity intervention.
//! Development only: no fitting, held-out predictions, or mechanism adoption.
use super::*;
use uor_r4_integer::geometric_cue_carrier::CueJointConfig;

// Only declared receipt differences are ignored. All numerical/state fields,
// routes, occurrence identities, native masses and generated tokens remain.
fn numerical(mut report: Value) -> Result<Value> {
    let rows = report["rows"]
        .as_array_mut()
        .ok_or_else(|| invalid("joint rows absent"))?;
    for row in rows {
        for token in row["tokens"]
            .as_array_mut()
            .ok_or_else(|| invalid("joint tokens absent"))?
        {
            for component in ["cue_carrier", "prefix_transport", "source_end"] {
                let map = token[component]
                    .as_object_mut()
                    .ok_or_else(|| invalid("joint component absent"))?;
                map.remove("metadata")
                    .ok_or_else(|| invalid("joint component metadata absent"))?;
                if component == "cue_carrier" {
                    map.remove("costs")
                        .ok_or_else(|| invalid("joint cue costs absent"))?;
                }
            }
        }
    }
    Ok(report)
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
        return Err(
            invalid("joint foundation requires unary-only parent and512 development rows").into(),
        );
    }
    let donor_unary = donor.packed_coefficients()?;
    let config = CueJointConfig {
        head: 1,
        left_lane: 1,
        right_lane: 3,
    };
    let zero = CueAngularWeights::from_native(
        parent,
        &a.native_artifact,
        &parent.compile_cue_carrier(donor.native()?)?,
    )?
    .with_zero_joint(parent, config)?;
    let root = a.out.join("zero-joint");
    report_output::claim(&root)?;
    save_chain(&root, &zero, parent, f)?;
    let restored = CueAngularWeights::load(&root.join("cue"), parent, &a.native_artifact)?;
    if parameter_receipts(&zero.parameters())? != parameter_receipts(&restored.parameters())?
        || restored.packed_coefficients()? != donor_unary
    {
        return Err(invalid("joint independent source roundtrip differs").into());
    }
    let (zc, zp, ze) = load_chain(&root, integer)?;
    let dc = integer.compile_cue_carrier(donor.native()?)?;
    let (dp, de) = f.integer(integer, &dc)?;
    let baseline = source_end_fit::canonical(integer, &dc, &dp, &de, development, a, start)?;
    let zero_canonical = source_end_fit::canonical(integer, &zc, &zp, &ze, development, a, start)?;
    let baseline_generation =
        source_end_fit::generation(integer, &dc, &dp, &de, development, tok, a, start)?;
    let zero_generation =
        source_end_fit::generation(integer, &zc, &zp, &ze, development, tok, a, start)?;
    if numerical(baseline.clone())? != numerical(zero_canonical.clone())?
        || numerical(baseline_generation.clone())? != numerical(zero_generation.clone())?
    {
        return Err(invalid("zero joint numerical/state/route/mass/output replay differs").into());
    }
    let indices: Vec<_> = (0..development.len()).collect();
    let gradient = batch(
        &indices,
        development,
        source,
        parent,
        Some(integer),
        &restored,
        f,
        true,
        a,
        start,
    )?;
    let params = restored.joint_parameters();
    if params.len() != 1 || gradient.gradients.len() != 1 {
        return Err(invalid("joint gradient family differs").into());
    }
    let (name, var) = params
        .iter()
        .next()
        .ok_or_else(|| invalid("joint Var absent"))?;
    let values = gradient.gradients[name].flatten_all()?.to_vec1::<f32>()?;
    if values.len() != 16 {
        return Err(invalid("joint gradient width differs").into());
    }
    let index = (0..16)
        .max_by(|&i, &j| values[i].abs().total_cmp(&values[j].abs()).then(j.cmp(&i)))
        .ok_or_else(|| invalid("joint maximum absent"))?;
    let mut q = vec![0f32; 16];
    q[index] = if values[index] > 0. { -0.25 } else { 0.25 };
    write_json(
        &root,
        "gradient.json",
        &json!({"batch":gradient.report,"signed_values":values,"selection":"maximum absolute signed mean gradient; smallest index breaks ties; negative sign quarter"}),
    )?;
    write_json(&root, "canonical.json", &zero_canonical)?;
    write_json(&root, "generation.json", &zero_generation)?;
    report_output::seal(&root)?;
    report_output::verify(&root)?;
    var.set(&Tensor::from_vec(q, (16,), var.device())?)?;
    if restored.packed_coefficients()? != donor_unary {
        return Err(invalid("joint sensitivity changed frozen unary bytes").into());
    }
    let intervention = a.out.join("quarter-intervention");
    report_output::claim(&intervention)?;
    save_chain(&intervention, &restored, parent, f)?;
    let reread = CueAngularWeights::load(&intervention.join("cue"), parent, &a.native_artifact)?;
    if parameter_receipts(&restored.parameters())? != parameter_receipts(&reread.parameters())? {
        return Err(invalid("joint intervention source roundtrip differs").into());
    }
    let (nc, np, ne) = load_chain(&intervention, integer)?;
    let canonical = source_end_fit::canonical(integer, &nc, &np, &ne, development, a, start)?;
    let generation =
        source_end_fit::generation(integer, &nc, &np, &ne, development, tok, a, start)?;
    let effect = numerical(canonical.clone())? != numerical(zero_canonical.clone())?;
    write_json(&intervention, "canonical.json", &canonical)?;
    write_json(&intervention, "generation.json", &generation)?;
    report_output::seal(&intervention)?;
    report_output::verify(&intervention)?;
    frozen(source, receipts)?;
    immutable(inputs, seals)?;
    if donor.packed_coefficients()? != donor_unary {
        return Err(invalid("joint parent changed").into());
    }
    Ok(
        json!({"schema":"uor-r4.geometric-cue-joint-foundation/1", "status":"completed",
        "cases":512, "evaluation_predictions":0, "optimizer_updates":0,
        "sensitivity_interventions":1, "selected_coordinate":index, "signed_quarter_step":if values[index]>0. {-1} else {1},
        "gradient":gradient.report, "zero_numerical_replay_equal":true, "zero_generated_outputs_equal":true,
        "independent_source_reload":true, "native_nonzero_intervention_effect":effect,
        "joint_config":config, "joint_metadata":nc.metadata().joint,
        "frozen_unary_sha256":sha256_bytes(&donor_unary), "frozen_payloads":f.hashes(),
        "input_manifests_sha256":inputs, "baseline_native_ce":baseline["native_equal_episode_ce"],
        "intervention_native_ce":canonical["native_equal_episode_ce"],
        "limitation":"development foundation and one forward sensitivity intervention only; no fit, transfer, attention/chat qualification or adoption"}),
    )
}
