//! Actual composed context learning on CUDA; native serving remains integer CPU.
//! Zero updates, no fresh predictions. CPU is an explicit reference, never fallback.
use super::*;
const ABSOLUTE_TOLERANCE: f64 = 1e-4;
const RELATIVE_TOLERANCE: f64 = 5e-3;

fn cuda_device() -> Result<Device> {
    #[cfg(feature = "cuda")]
    {
        Ok(Device::new_cuda(0)?)
    }
    #[cfg(not(feature = "cuda"))]
    {
        Err(
            invalid("native CUDA probe requires the compiled cuda feature; CPU fallback forbidden")
                .into(),
        )
    }
}
fn compare_gradients(
    cpu: &Tensor,
    gpu: &Tensor,
    cpu_vars: &BTreeMap<String, Var>,
    gpu_vars: &BTreeMap<String, Var>,
    device: &Device,
    require_positive: bool,
) -> Result<Value> {
    let cpu_tick = Instant::now();
    let cpu_grad = cpu.backward()?;
    let cpu_backward_seconds = cpu_tick.elapsed().as_secs_f64();
    device.synchronize()?;
    let gpu_tick = Instant::now();
    let gpu_grad = gpu.backward()?;
    device.synchronize()?;
    let cuda_backward_synced_seconds = gpu_tick.elapsed().as_secs_f64();
    let evidence_tick = Instant::now();
    let mut families = BTreeMap::new();
    for (name, cv) in cpu_vars {
        let gv = gpu_vars
            .get(name)
            .ok_or_else(|| invalid("CUDA context family absent"))?;
        let cg = cpu_grad.get(cv.as_tensor());
        let gg = gpu_grad.get(gv.as_tensor());
        if cg.is_some() != gg.is_some() {
            return Err(invalid("CPU/CUDA gradient connectivity differs").into());
        }
        let (Some(cg), Some(gg)) = (cg, gg) else {
            if require_positive {
                return Err(invalid("combined CUDA/CPU positive family is disconnected").into());
            }
            families.insert(name.clone(), json!({"connected":false}));
            continue;
        };
        if !gg.device().same_device(device) {
            return Err(invalid("CUDA gradient left learning device").into());
        }
        let x = cg.flatten_all()?.to_vec1::<f32>()?;
        let y = gg.flatten_all()?.to_vec1::<f32>()?;
        if x.len() != y.len() || x.len() != cv.elem_count() {
            return Err(invalid("CPU/CUDA gradient shape differs").into());
        }
        let mut max_abs = 0f64;
        let mut max_scaled = 0f64;
        let mut nonzero = 0usize;
        let mut cpu_nonzero = 0usize;
        let (mut cpu_sq, mut gpu_sq, mut error_sq) = (0f64, 0f64, 0f64);
        for (&a, &b) in x.iter().zip(&y) {
            if !a.is_finite() || !b.is_finite() {
                return Err(invalid("CPU/CUDA nonfinite gradient").into());
            }
            let error = (f64::from(a) - f64::from(b)).abs();
            let limit = ABSOLUTE_TOLERANCE + RELATIVE_TOLERANCE * f64::from(a).abs();
            max_abs = max_abs.max(error);
            max_scaled = max_scaled.max(error / limit);
            nonzero += usize::from(b != 0.);
            cpu_nonzero += usize::from(a != 0.);
            cpu_sq += f64::from(a).powi(2);
            gpu_sq += f64::from(b).powi(2);
            error_sq += error.powi(2);
        }
        let cpu_l2 = cpu_sq.sqrt();
        let cuda_l2 = gpu_sq.sqrt();
        let error_l2 = error_sq.sqrt();
        let l2_limit = 1e-6 + RELATIVE_TOLERANCE * cpu_l2;
        if max_scaled > 1.
            || error_l2 > l2_limit
            || (cpu_nonzero > 0 && nonzero == 0)
            || (require_positive && (cpu_nonzero == 0 || nonzero == 0))
        {
            return Err(invalid(format!(
                "CPU/CUDA gradient tolerance exceeded: {name} {max_scaled}"
            ))
            .into());
        }
        families.insert(name.clone(),json!({"connected":true,"elements":x.len(),"cpu_nonzero":cpu_nonzero,"cuda_nonzero":nonzero,"cpu_l2":cpu_l2,"cuda_l2":cuda_l2,"error_l2":error_l2,"l2_error_limit":l2_limit,"max_absolute_error":max_abs,"max_tolerance_ratio":max_scaled,"gradient_device":"CUDA"}));
    }
    Ok(
        json!({"families":families,"timing":{"cpu_backward_seconds":cpu_backward_seconds,"cuda_backward_synced_seconds":cuda_backward_synced_seconds,"gradient_download_and_comparison_seconds":evidence_tick.elapsed().as_secs_f64()}}),
    )
}
fn compare_context(
    cpu: &uor_r4_training::geometric_context::ContextQ4Output,
    gpu: &uor_r4_training::geometric_context::ContextQ4Output,
    device: &Device,
) -> Result<Value> {
    if cpu.trace != gpu.trace {
        return Err(invalid("direct CPU/CUDA context native trace differs").into());
    }
    let mut fields = BTreeMap::new();
    for (name, c, g, exact) in [
        ("root_logits", &cpu.root_logits, &gpu.root_logits, false),
        (
            "category_logits",
            &cpu.category_logits,
            &gpu.category_logits,
            false,
        ),
        ("latent_roots", &cpu.latent_roots, &gpu.latent_roots, true),
        ("state_logits", &cpu.state_logits, &gpu.state_logits, false),
    ] {
        if !g.device().same_device(device) || c.dims() != g.dims() {
            return Err(invalid("direct CUDA context output device/shape differs").into());
        }
        let x = c.flatten_all()?.to_vec1::<f32>()?;
        let y = g.flatten_all()?.to_vec1::<f32>()?;
        let mut max_error = 0f64;
        let mut max_scaled = 0f64;
        for (&a, &b) in x.iter().zip(&y) {
            if !a.is_finite() || !b.is_finite() {
                return Err(invalid("direct CUDA context output nonfinite").into());
            }
            if exact && a.to_bits() != b.to_bits() {
                return Err(invalid("direct CUDA retained latent basis bits differ").into());
            }
            let error = (f64::from(a) - f64::from(b)).abs();
            let limit = 1e-5 + 1e-5 * f64::from(a).abs();
            max_error = max_error.max(error);
            max_scaled = max_scaled.max(error / limit);
        }
        if !exact && max_scaled > 1. {
            return Err(invalid(format!(
                "direct CUDA context logits tolerance differs {name}"
            ))
            .into());
        }
        fields.insert(name,json!({"elements":x.len(),"bit_exact_required":exact,"max_absolute_error":max_error,"max_tolerance_ratio":max_scaled,"absolute_tolerance":1e-5,"relative_tolerance":1e-5}));
    }
    Ok(json!(fields))
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
        return Err(invalid("CUDA probe requires full512 and actual joint donor").into());
    }
    let device = cuda_device()?;
    let staging_start = Instant::now();
    let gpu_source = SourceRealizerWeights::load_source_on_device(
        &a.source_weights,
        &fs::read(a.source_weights.join("tokenizer.json"))?,
        &device,
    )?;
    let identity: ConsumerIdentity = serde_json::from_value(
        read_json(&a.native_artifact.join("metadata.json"))?["identity"].clone(),
    )?;
    let gpu_parent = NativeSourceRealizer::load(&a.native_artifact, &gpu_source, &identity)?;
    if gpu_parent.artifact_binding()? != parent.artifact_binding()? {
        return Err(invalid("CPU/CUDA parent artifact differs").into());
    }
    let cpu_original = parameter_receipts(&source.parameters())?;
    let gpu_original = parameter_receipts(&gpu_source.parameters())?;
    if cpu_original != gpu_original {
        return Err(invalid("CPU/CUDA staged source bits differ").into());
    }
    let cpu_vars = source.context_state_parameters();
    let gpu_vars = gpu_source.context_state_parameters();
    if cpu_vars.len() != 9 || gpu_vars.len() != 9 {
        return Err(invalid("CUDA probe requires nine context families").into());
    }
    let cc = parent.compile_cue_carrier(donor.native()?)?;
    let (cp, ce) = f.training(parent, &cc)?;
    let gc = gpu_parent.compile_cue_carrier(donor.native()?)?;
    let (gp, ge) = f.training(&gpu_parent, &gc)?;
    let ic = integer.compile_cue_carrier(donor.native()?)?;
    let (ip, ie) = f.integer(integer, &ic)?;
    let cpu_prepared = source.prepare(parent)?;
    let gpu_prepared = gpu_source.prepare_on_device(&gpu_parent, &device)?;
    device.synchronize()?;
    let staging_seconds = staging_start.elapsed().as_secs_f64();
    let mut rows = Vec::new();
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
            let actual_prefix = &episode.target[..step];
            let target = episode.target[step];
            let oracle_start = Instant::now();
            let oracle = integer.read_bank_with_source_end_transport(
                &segments,
                &episode.packet.query_ids,
                actual_prefix,
                &ic,
                &ip,
                &ie,
            )?;
            let oracle_seconds = oracle_start.elapsed().as_secs_f64();
            let mut context_parity = Vec::new();
            if index == 0 && step == episode.target.len() - 1 {
                let mut packets = vec![
                    (
                        "causal-bank".to_string(),
                        oracle.prefix_bank.cue_bank.bank.context.tokens.clone(),
                        false,
                    ),
                    ("query".to_string(), episode.packet.query_ids.clone(), false),
                    (
                        "query-reset-each-token".to_string(),
                        episode.packet.query_ids.clone(),
                        true,
                    ),
                ];
                packets.extend(
                    oracle
                        .prefix_bank
                        .cue_bank
                        .carrier
                        .cues
                        .iter()
                        .enumerate()
                        .map(|(i, p)| (format!("cue-{i}"), p.state.token_ids.clone(), false)),
                );
                packets.extend(
                    oracle
                        .prefix_bank
                        .prefix
                        .sources
                        .iter()
                        .enumerate()
                        .map(|(i, p)| (format!("source-{i}"), p.token_ids.clone(), false)),
                );
                if !actual_prefix.is_empty() {
                    packets.push((
                        "actual-response-prefix".into(),
                        actual_prefix.to_vec(),
                        false,
                    ));
                }
                for (name, ids, reset) in packets {
                    let c = cpu_prepared.context_output(&ids, reset)?;
                    let g = gpu_prepared.context_output(&ids, reset)?;
                    device.synchronize()?;
                    context_parity.push(json!({"packet":name,"actual_ids":ids,"reset_each_token":reset,"fields":compare_context(&c,&g,&device)?}));
                }
            }

            let cpu_start = Instant::now();
            let cpu = cpu_prepared.loss_bank_composed_state(
                &segments,
                &episode.packet.query_ids,
                actual_prefix,
                target,
                &cc,
                &cp,
                &ce,
            )?;
            let cpu_forward_seconds = cpu_start.elapsed().as_secs_f64();
            device.synchronize()?;
            let gpu_start = Instant::now();
            let gpu = gpu_prepared.loss_bank_composed_state(
                &segments,
                &episode.packet.query_ids,
                actual_prefix,
                target,
                &gc,
                &gp,
                &ge,
            )?;
            device.synchronize()?;
            let gpu_forward_seconds = gpu_start.elapsed().as_secs_f64();
            if !gpu.loss.device().same_device(&device) {
                return Err(invalid("composed CUDA loss is not on CUDA").into());
            }
            if serde_json::to_value(&cpu.trace)? != serde_json::to_value(&oracle)?
                || serde_json::to_value(&gpu.trace)? != serde_json::to_value(&oracle)?
                || cpu.target_probability != gpu.target_probability
            {
                return Err(
                    invalid("CPU/CUDA composed hard native trace/probability differs").into(),
                );
            }
            let cpu_loss = cpu.loss.to_scalar::<f32>()?;
            let gpu_loss = gpu.loss.to_scalar::<f32>()?;
            let expected = -cpu.target_probability.ln();
            for v in [cpu_loss, gpu_loss] {
                if !v.is_finite() || (f64::from(v) - expected).abs() > 2e-6 * (1. + expected.abs())
                {
                    return Err(invalid("CPU/CUDA loss differs native probability").into());
                }
            }
            let backward_start = Instant::now();
            let gradients =
                compare_gradients(&cpu.loss, &gpu.loss, &cpu_vars, &gpu_vars, &device, true)?;
            let backward_and_evidence_seconds = backward_start.elapsed().as_secs_f64();
            let mut components = BTreeMap::new();
            if index == 0 && (step == 0 || step == episode.target.len() - 1) {
                for name in cpu.state_credit_components.keys() {
                    components.insert(
                        *name,
                        compare_gradients(
                            &cpu.component_loss(name, target)?,
                            &gpu.component_loss(name, target)?,
                            &cpu_vars,
                            &gpu_vars,
                            &device,
                            false,
                        )?,
                    );
                }
            }
            rows.push(json!({"id":episode.packet.id,"index":index,"step":step,"canonical_supervised_prefix_ids":actual_prefix,"target_label_only_after_native_read":target,"direct_context_output_parity":context_parity,"native_trace_parity":true,"native_target_probability":cpu.target_probability,"cpu_loss":cpu_loss,"cuda_loss":gpu_loss,"gradients":gradients,"isolated_components":components,"timing":{"independent_host_oracle_seconds":oracle_seconds,"cpu_composed_forward_seconds":cpu_forward_seconds,"cuda_composed_forward_synced_seconds":gpu_forward_seconds,"backward_comparison_and_host_evidence_seconds":backward_and_evidence_seconds,"cuda_forward_scope":"includes current host native oracle/admission, constant staging, and one scalar device finite-status synchronization; not isolated GPU kernel timing"}}));
        }
    }
    let mut repeated = Vec::new();
    let episode = &development[0];
    let segments = episode.segments()?;
    for repetition in 0..3 {
        deadline(a, start)?;
        device.synchronize()?;
        let tick = Instant::now();
        let loss = gpu_prepared.loss_bank_composed_state(
            &segments,
            &episode.packet.query_ids,
            &[],
            episode.target[0],
            &gc,
            &gp,
            &ge,
        )?;
        device.synchronize()?;
        let backward_tick = Instant::now();
        let grad = loss.loss.backward()?;
        device.synchronize()?;
        let backward_only_seconds = backward_tick.elapsed().as_secs_f64();
        for var in gpu_vars.values() {
            if let Some(g) = grad.get(var.as_tensor()) {
                if !g.device().same_device(&device) {
                    return Err(invalid("repeated CUDA adjoint left device").into());
                }
            }
        }
        repeated.push(json!({"repetition":repetition,"synced_composed_forward_backward_seconds":tick.elapsed().as_secs_f64(),"cuda_backward_only_synced_seconds":backward_only_seconds,"scope":"includes host native oracle/admission; excludes gradient downloads and source export"}));
    }
    drop(cpu_prepared);
    drop(gpu_prepared);
    let evidence_start = Instant::now();
    if cpu_original != parameter_receipts(&source.parameters())?
        || gpu_original != parameter_receipts(&gpu_source.parameters())?
    {
        return Err(invalid("zero-update CPU/CUDA source bits changed").into());
    }
    let final_hash_download_seconds = evidence_start.elapsed().as_secs_f64();
    write_json(
        &a.out,
        "cpu-cuda-state-credit.json",
        &json!({"rows":rows,"repeated_cuda_timings":repeated,"absolute_tolerance":ABSOLUTE_TOLERANCE,"relative_tolerance":RELATIVE_TOLERANCE,"l2_absolute_tolerance":1e-6,"l2_relative_tolerance":RELATIVE_TOLERANCE,"require_cuda_nonzero_if_cpu_nonzero":true,"component_losses_sum_allowed":false}),
    )?;
    let export_start = Instant::now();
    let reload = a.out.join("independent-cuda-source-reload");
    report_output::claim(&reload)?;
    let reload_result = (|| -> Result<Value> {
        let receipt = gpu_source.save_context_state_rebound(
            &reload.join("artifact"),
            &gpu_parent,
            &a.native_artifact,
            &gc,
            &gp,
            &ge,
        )?;
        let binding: NativeArtifactBinding = serde_json::from_value(receipt["new_parent"].clone())?;
        let loaded = IntegerRealizer::load_native(&reload.join("artifact/native"), &binding)?;
        let lc = loaded.compile_cue_carrier(donor.native()?)?;
        let (lp, le) = f.integer(&loaded, &lc)?;
        let reference =
            source_end_fit::generation(integer, &ic, &ip, &ie, &development[..4], tok, a, start)?;
        let actual =
            source_end_fit::generation(&loaded, &lc, &lp, &le, &development[..4], tok, a, start)?;
        if reference != actual {
            return Err(invalid("CUDA-staged zero export/reload own-prefix differs").into());
        }
        write_json(&reload, "generation.json", &actual)?;
        Ok(json!({"rebind":receipt,"actual_loaded_ownprefix_parity":true,"rows":4}))
    })();
    match &reload_result {
        Ok(v) => write_json(&reload, "receipt.json", v)?,
        Err(e) => write_json(&reload, "failure.json", &json!({"error":e.to_string()}))?,
    }
    report_output::seal(&reload)?;
    report_output::verify(&reload)?;
    reload_result?;
    let host_export_and_reload_seconds = export_start.elapsed().as_secs_f64();
    // Original full512 byte-verified replay is retained in the preceding
    // composed probe. This device admission adds only the declared four reload
    // replies, not another full CPU-generation campaign.
    immutable(inputs, seals)?;
    Ok(
        json!({"schema":"uor-r4.native-geometric-cuda-learning-probe/1","status":"COMPLETED","mode":a.mode,"updates":0,"selection":"NONE","fresh_predictions_count":0,"development_rows":512,"gradient_rows":4,"gradient_positions":rows.len(),"active_families":gpu_vars.keys().collect::<Vec<_>>(),"learning_device":"CUDA:0","cpu_reference_explicit":true,"cpu_fallback":false,"source_staging_and_native_admission_seconds":staging_seconds,"final_source_hash_download_seconds":final_hash_download_seconds,"host_export_and_actual_reload_seconds":host_export_and_reload_seconds,"actual_reload_ownprefix_rows":4,"full512_baseline_reexecution":"NOT_RUN; reuse retained byte-verified composed-state probe","state_credit_report_sha256":sha256_file(&a.out.join("cpu-cuda-state-credit.json"))?,"input_files_sha256":inputs,"source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT"),"executable_sha256":sha256_file(&executable()?.0)?,"elapsed_seconds":start.elapsed().as_secs_f64(),"scope":"CUDA context-only composed learning graph with frozen host scorer parameters and native oracle; local120 conditional adjoints, earlier4D recurrence, stopped source route; zero updates, no Generate or acceleration claim"}),
    )
}

#[cfg(all(test, not(feature = "cuda")))]
mod tests {
    use super::*;
    #[test]
    fn cuda_unavailable_is_explicit_error_without_cpu_fallback() {
        assert!(cuda_device().is_err());
    }
}
