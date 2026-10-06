//! Current composed native predictive loss into existing geometric state families.
//! Zero updates: connection/identity admission, not a learning-quality verdict.
use super::*;

fn gradient_receipt(loss: &Tensor, parameters: &BTreeMap<String, Var>) -> Result<Value> {
    let gradients = loss.backward()?;
    let mut rows = BTreeMap::new();
    for (name, var) in parameters {
        let Some(g) = gradients.get(var.as_tensor()) else {
            rows.insert(name.clone(), json!({"connected":false,"nonzero":0}));
            continue;
        };
        let values = g.flatten_all()?.to_vec1::<f32>()?;
        if values.len() != var.elem_count() || values.iter().any(|x| !x.is_finite()) {
            return Err(invalid("composed state nonfinite or mismatched gradient").into());
        }
        let nonzero = values.iter().filter(|x| **x != 0.).count();
        let l2 = values
            .iter()
            .map(|x| f64::from(*x).powi(2))
            .sum::<f64>()
            .sqrt();
        let max_abs = values.iter().map(|x| x.abs()).fold(0f32, f32::max);
        rows.insert(name.clone(), json!({"connected":true,"elements":values.len(),"nonzero":nonzero,"l2":l2,"max_abs":max_abs}));
    }
    Ok(json!(rows))
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
) -> Result<Value> {
    if development.len() != 512 || donor.joint_config().is_none() {
        return Err(invalid("composed state requires full512 and actual joint donor").into());
    }
    let parameters = source.context_state_parameters();
    if parameters.len() != 9 {
        return Err(invalid("composed state requires existing nine context families").into());
    }
    let original_parameters = parameter_receipts(&source.parameters())?;
    let cue = parent.compile_cue_carrier(donor.native()?)?;
    let (prefix, end) = f.training(parent, &cue)?;
    let ic = integer.compile_cue_carrier(donor.native()?)?;
    let (ip, ie) = f.integer(integer, &ic)?;
    let prepared = source.prepare(parent)?;
    let mut rows = Vec::new();
    // Fixed first four published development rows. No quality/support filtering.
    for (index, episode) in development.iter().take(4).enumerate() {
        let segments = episode.segments()?;
        let mut steps = vec![
            0,
            episode.target.len().saturating_sub(2),
            episode.target.len() - 1,
        ];
        steps.sort_unstable();
        steps.dedup();
        for step in steps {
            deadline(a, start)?;
            let target = episode.target[step];
            let actual_prefix = &episode.target[..step];
            let independent = integer.read_bank_with_source_end_transport(
                &segments,
                &episode.packet.query_ids,
                actual_prefix,
                &ic,
                &ip,
                &ie,
            )?;
            let observed = prepared.loss_bank_composed_state(
                &segments,
                &episode.packet.query_ids,
                actual_prefix,
                target,
                &cue,
                &prefix,
                &end,
            )?;
            if serde_json::to_value(&observed.trace)? != serde_json::to_value(&independent)? {
                return Err(invalid(
                    "composed predictive hard trace differs from independent runtime",
                )
                .into());
            }
            let loss = observed.loss.to_scalar::<f32>()?;
            let expected = -observed.target_probability.ln();
            if !loss.is_finite()
                || (f64::from(loss) - expected).abs() > 2e-6 * (1. + expected.abs())
            {
                return Err(invalid(
                    "composed predictive loss differs from native target probability",
                )
                .into());
            }
            let gradients = gradient_receipt(&observed.loss, &parameters)?;
            let mut components = BTreeMap::new();
            // Isolate every consumed path on two real boundary states. Their
            // anchored scalar objectives must not be summed as a training loss.
            if index == 0 && (step == 0 || step == episode.target.len() - 1) {
                for name in observed.state_credit_components.keys() {
                    deadline(a, start)?;
                    components.insert(
                        *name,
                        gradient_receipt(&observed.component_loss(name, target)?, &parameters)?,
                    );
                }
            }
            rows.push(json!({"index":index,"id":episode.packet.id,"step":step,
                "canonical_supervised_prefix_ids":actual_prefix,"target_label_only_after_native_read":target,
                "native_probability":observed.target_probability,"loss":loss,
                "independent_integer_trace_parity":true,"credit_scope":observed.credit_scope,
                "context_state_gradients":gradients,"isolated_components":components,
                "native_chosen_token":independent.actions.chosen_token_id}));
        }
    }
    drop(prepared);
    if original_parameters != parameter_receipts(&source.parameters())? {
        return Err(invalid("zero-update composed state probe changed source parameters").into());
    }
    write_json(
        &a.out,
        "state-credit.json",
        &json!({"updates":0,"rows":rows,
        "component_losses_sum_allowed":false,"families":parameters.keys().collect::<Vec<_>>() }),
    )?;
    let reload = a.out.join("independent-reload");
    report_output::claim(&reload)?;
    let reload_result = (|| -> Result<Value> {
        let receipt = source.save_context_state_rebound(
            &reload.join("artifact"),
            parent,
            &a.native_artifact,
            &cue,
            &prefix,
            &end,
        )?;
        let binding: NativeArtifactBinding = serde_json::from_value(receipt["new_parent"].clone())?;
        let loaded = IntegerRealizer::load_native(&reload.join("artifact/native"), &binding)?;
        let lc = loaded.compile_cue_carrier(donor.native()?)?;
        let (lp, le) = f.integer(&loaded, &lc)?;
        let original_generation =
            source_end_fit::generation(integer, &ic, &ip, &ie, &development[..4], tok, a, start)?;
        let loaded_generation =
            source_end_fit::generation(&loaded, &lc, &lp, &le, &development[..4], tok, a, start)?;
        if original_generation != loaded_generation {
            return Err(
                invalid("zero-export actual independent own-prefix generation differs").into(),
            );
        }
        write_json(&reload, "generation.json", &loaded_generation)?;
        Ok(json!({"rebind":receipt,"actual_loaded_ownprefix_parity":true,"rows":4}))
    })();
    match &reload_result {
        Ok(receipt) => write_json(&reload, "receipt.json", receipt)?,
        Err(error) => write_json(&reload, "failure.json", &json!({"error":error.to_string()}))?,
    }
    report_output::seal(&reload)?;
    report_output::verify(&reload)?;
    reload_result?;
    let baseline = source_end_fit::canonical(integer, &ic, &ip, &ie, development, a, start)?;
    let generation =
        source_end_fit::generation(integer, &ic, &ip, &ie, development, tok, a, start)?;
    write_json(&a.out, "baseline-canonical.json", &baseline)?;
    write_json(&a.out, "baseline-generation.json", &generation)?;
    immutable(inputs, seals)?;
    Ok(
        json!({"schema":"uor-r4.composed-geometric-state-credit-probe/1","status":"COMPLETED",
        "mode":a.mode,"updates":0,"selection":"NONE","fresh_predictions_count":0,
        "development_rows":development.len(),"gradient_rows":4,"gradient_positions":rows.len(),
        "active_families":parameters.keys().collect::<Vec<_>>(),
        "baseline_complete":generation["accepted_complete"],"baseline_native_ce":baseline["native_equal_episode_ce"],
        "context_state_gradient_report_sha256":sha256_file(&a.out.join("state-credit.json"))?,
        "independent_reload_manifest_sha256":sha256_file(&reload.join("manifest.json"))?,
        "input_files_sha256":inputs,"original_source_parameters":original_parameters,
        "source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT"),"executable_sha256":sha256_file(&executable()?.0)?,
        "elapsed_seconds":start.elapsed().as_secs_f64(),"host_os":std::env::consts::OS,"host_arch":std::env::consts::ARCH,
        "scope":"connected composed attention prediction into existing geometric state; fixed scorer coefficients; stopped factual source route; Copy/Period/Stop support; no fit, general Generate, gradient correctness for argmax, or language improvement claim"}),
    )
}
