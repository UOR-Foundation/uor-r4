//! Saved437 joint Context/prototype connection. No fixed-state training cache.
use super::*;
use uor_r4_integer::{
    geometric_cue_carrier::NativeCueCarrier, geometric_prefix_transport::NativePrefixTransport,
};
use uor_r4_training::geometric_bank_generate::BankGenerateOutput;

const REPORT: &str = "787f1a694dd894f8dcf636e158b2470fd88a824f3dd73535749bc4f36f05a165";
const MANIFEST: &str = "2f630ad56a6abd09271681fcb514d22caefac8af918321224bc173446c20a84c";
const FIELD: &str = "de4a3234d6e92f4b12a657cc195425687bc67c243d9eded9a1df0797b9942a61";
const MASTER: &str = "af5e263c2e3dca56232ba7d43f52514c0a4fbfd5c79956df0cfa0ebe1567fc88";
const OBJECTIVE: &str = "equal-episode-logmeanexp-native-pooled-strongest-wrong-softplus-negative-log-mass-margin/1;margin0;temperature1;smallest-token-ID-ties;all-targets-including-EOS;existing-raw-identity-STE;no-phase-weighting";

#[derive(Clone, Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Config {
    pub saved437: PathBuf,
    pub field_carrier_credit: bool,
    pub context_learning_rate: f64,
    pub prototype_learning_rate: f64,
}

pub(super) fn settings(a: &Args) -> Result<()> {
    let Some(c) = &a.unified_geometry else {
        return replay_require(a.mode != Mode::UnifiedGeometry, "unified config absent");
    };
    replay_require(
        a.mode == Mode::UnifiedGeometry
            && a.credit == Credit::RawIdentity
            && a.read_state_pullback == ReadStatePullback::Categorical
            && a.loss_scope == LossScope::All
            && a.updates == 128
            && [1001, 2001].contains(&a.seed)
            && c.context_learning_rate == 0.002
            && c.prototype_learning_rate == 0.01
            && a.joint_continuation.is_none()
            && a.continuation.is_none()
            && a.cross_state_resume.is_none()
            && !a.cross_state_bottleneck
            && !a.cross_state_pooled_rank
            && !a.native_code_proposals
            && !a.reached_frontier_objective
            && !a.categorical_action_learning
            && !a.categorical_action_only
            && !a.constrained_context_learning
            && !a.constrained_emission_learning
            && a.retained_context_root.is_none()
            && a.reference_replay.is_none()
            && a.baseline.is_none()
            && !a.query_conditioned_read
            && !a.ceiling_scorer
            && !a.ceiling_float
            && a.panel_inputs.is_none()
            && a.panel_labels.is_none()
            && a.prototype_compensation.is_none()
            && a.reached_u.is_none()
            && a.readout_coadaptation.is_none()
            && a.prefix_context_credit.is_none()
            && a.prefix_fragment_learning.is_none()
            && a.prefix_artifact_check.is_none()
            && a.context_path_credit.is_none()
            && a.context_cue_coadapt.is_none()
            && a.generate_episode_learning.is_none()
            && a.generate_episode_completion.is_none()
            && a.coupled_episode_learning.is_none(),
        "unified requires exclusive saved437 two-epoch Context/prototype recipe",
    )
}

fn load_field(
    a: &Args,
    p: &ContinuationParent,
    d: &Device,
) -> Result<(ContinuationLearningWeights, Value)> {
    let root = &a
        .unified_geometry
        .as_ref()
        .ok_or_else(|| bad("unified config absent"))?
        .saved437;
    report_output::verify(root)?;
    replay_require(
        sha256_file(&root.join("report.json"))? == REPORT
            && sha256_file(&root.join("manifest.json"))? == MANIFEST,
        "unified saved437 report/seal differs",
    )?;
    let report = read(&root.join("report.json"))?;
    let cp = root.join("checkpoint-0256");
    let receipt = read(&cp.join("receipt.json"))?;
    let bytes = fs::read(cp.join("continuation-field.bin"))?;
    replay_require(
        report["status"] == "COMPLETED"
            && report["mode"] == "cross_state_continuation"
            && report["updates"] == 256
            && report["prior_updates"] == 832
            && report["lineage_step"] == 1088
            && report["cross_state_bottleneck"] == true
            && report["cross_state_pooled_rank"] == true
            && report["training_objective"] == OBJECTIVE
            && report["initial"]["complete"] == 177
            && report["final"]["complete"] == 437
            && report["final_receipt"] == receipt
            && receipt["parent"] == serde_json::to_value(&p.binding)?
            && receipt["frozen_parent_receipt"] == p.receipt
            && receipt["generate_sha256"] == p.generate_sha256
            && receipt["continuation_sha256"] == FIELD
            && sha256_bytes(&bytes) == FIELD
            && report["final_active_masters"]["continuation.cross_state"] == MASTER
            && receipt["parameters"]["continuation.cross_state"]["sha256"] == MASTER
            && receipt["parameters"]["continuation.cross_state"]["shape"] == json!([8, 14400])
            && receipt["parameters"]["continuation.cross_state"]["bytes"] == 460800
            && read(&cp.join("continuation-source/metadata.json"))? == receipt,
        "unified saved437 ancestry/checkpoint/master identity differs",
    )?;
    let generate = p.generator()?;
    let field = NativeContinuationField::from_bytes(&bytes, &p.binding, generate.generate_model())?;
    replay_require(field.is_cross_state(), "unified requires v3 ordered field")?;
    let weights = ContinuationLearningWeights::from_native(
        &field,
        generate.generate_model(),
        p.integer.binding(),
        d,
    )?;
    restore(
        &cp.join("continuation-source"),
        &receipt["parameters"],
        &weights.parameters(),
        d,
    )?;
    replay_require(
        weights
            .export_native(&p.binding, generate.generate_model())?
            .to_bytes()?
            == bytes,
        "unified fractional masters do not re-export exact saved437",
    )?;
    Ok((weights, report["final"].clone()))
}

fn active_parameters(l: &Loaded) -> Result<BTreeMap<String, Var>> {
    let mut parameters = l.source.context_state_parameters();
    replay_require(!parameters.is_empty(), "unified Context inventory empty")?;
    parameters.insert(
        "generate.prototype_choices".into(),
        l.generate.prototype_choices.clone(),
    );
    Ok(parameters)
}

fn frozen_parameters(
    l: &Loaded,
    u: &ContinuationLearningWeights,
) -> Result<BTreeMap<String, String>> {
    let active = active_parameters(l)?;
    let mut parameters = l.source.parameters();
    parameters.extend(l.generate.parameters());
    parameters.extend(l.original_bridge.parameters());
    parameters.extend(
        l.marker
            .parameters()
            .into_iter()
            .map(|(n, v)| (format!("marker.{n}"), v)),
    );
    parameters.extend(u.parameters());
    parameters.retain(|name, _| !active.contains_key(name));
    identities(&parameters)
}

fn with_bank<T>(
    l: &Loaded,
    u: &ContinuationLearningWeights,
    credit: bool,
    d: &Device,
    f: impl FnOnce(
        &mut PreparedBankGenerate<'_, '_>,
        &NativeCueCarrier<'_>,
        &NativePrefixTransport<'_>,
    ) -> Result<T>,
) -> Result<T> {
    let current = l.source.compile_context_potential_rebound(&l.frozen)?;
    let prepared = l.source.prepare_context_potential_on_device(&current, d)?;
    let cue = current.compile_cue_carrier(l.cue.clone())?;
    let prefix = current.compile_prefix_transport(&cue, prefix_clone(&l.prefix)?)?;
    let gs = l.generate.prepare_native()?;
    let bs = l.marker.prepare_native()?;
    replay_require(
        sha256_bytes(&bs.native.to_bytes()?) == CAT_SHA,
        "unified frozen bridge changed",
    )?;
    let field = u.prepare_native(&current.execution_binding()?, &gs.native)?;
    let mut learner = PreparedBankGenerate::new(&prepared, &l.generate, &gs, &l.exp)?
        .with_prefix_temporal_utility(false)
        .with_read_state_bridge(&l.marker, &bs)?
        .with_read_selector_credit(true)
        .with_continuation_field(u, &field)?
        .with_continuation_context_credit(credit)?
        .with_continuation_prototype_credit(credit)?;
    f(&mut learner, &cue, &prefix)
}

fn checkpoint_current(
    a: &Args,
    step: usize,
    l: &Loaded,
    u: &ContinuationLearningWeights,
) -> Result<(ContinuationParent, NativeContinuationField)> {
    let (_, _, _, mut receipt) = checkpoint(a, step, l)?;
    let root = a.out.join(format!("checkpoint-{step:04}"));
    let p = ContinuationParent::from_checkpoint(&root)?;
    let generator = p.generator()?;
    let field = u.export_native(&p.binding, generator.generate_model())?;
    let bytes = field.to_bytes()?;
    let masters = save_masters(&root.join("continuation-source"), &u.parameters())?;
    fs::write(root.join("continuation-field.bin"), &bytes)?;
    let reloaded = NativeContinuationField::from_bytes(
        &fs::read(root.join("continuation-field.bin"))?,
        &p.binding,
        generator.generate_model(),
    )?;
    let cpu = ContinuationLearningWeights::from_native(
        &reloaded,
        generator.generate_model(),
        p.integer.binding(),
        &Device::Cpu,
    )?;
    restore(
        &root.join("continuation-source"),
        &masters,
        &cpu.parameters(),
        &Device::Cpu,
    )?;
    replay_require(
        cpu.export_native(&p.binding, generator.generate_model())?
            .to_bytes()?
            == bytes,
        "unified saved U masters/native reload differs",
    )?;
    receipt["mode"] = json!("unified_geometry");
    receipt["continuation_sha256"] = json!(sha256_bytes(&bytes));
    receipt["continuation_parameters"] = masters;
    receipt["active_parameter_names"] = json!(active_parameters(l)?.keys().collect::<Vec<_>>());
    receipt["training_objective"] = json!(OBJECTIVE);
    receipt["saved437_report_sha256"] = json!(REPORT);
    receipt["unified_geometry"] = serde_json::to_value(&a.unified_geometry)?;
    write(a, &format!("checkpoint-{step:04}/receipt.json"), &receipt)?;
    Ok((ContinuationParent::from_checkpoint(&root)?, reloaded))
}

fn forward(
    learner: &mut PreparedBankGenerate<'_, '_>,
    e: &Episode,
    actual_prefix: &[u32],
    cue: &NativeCueCarrier<'_>,
    prefix: &NativePrefixTransport<'_>,
) -> Result<BankGenerateOutput> {
    // The original complete bank, occurrence versions and query remain input.
    Ok(learner.forward_bank(
        &e.segments()?,
        &e.packet.query_ids,
        actual_prefix,
        cue,
        prefix,
    )?)
}

fn evaluate_cuda(
    a: &Args,
    name: &str,
    l: &Loaded,
    u: &ContinuationLearningWeights,
    eps: &[Episode],
    d: &Device,
    start: Instant,
) -> Result<Value> {
    with_bank(l, u, false, d, |learner, cue, prefix| {
        let mut rows = Vec::new();
        let mut complete = 0;
        let mut ce = 0.;
        for e in eps {
            let mut canonical = Vec::new();
            let mut rowce = 0.;
            for (t, &target) in e.target.iter().enumerate() {
                deadline(a, start)?;
                let out = forward(learner, e, supervised_prefix(&e.target, t)?, cue, prefix)?;
                let mass = out
                    .actions
                    .token_masses
                    .iter()
                    .find(|m| m.token_id == target)
                    .ok_or_else(|| bad("unified evaluation target support absent"))?
                    .weight_q31;
                replay_require(
                    mass > 0 && out.actions.summary.total_weight_q31 > 0,
                    "unified native support invalid",
                )?;
                let loss = -(mass as f64 / out.actions.summary.total_weight_q31 as f64).ln();
                rowce += loss;
                let continuation = out
                    .continuation
                    .as_ref()
                    .ok_or_else(|| bad("unified local field witness absent"))?;
                canonical.push(json!({"target_label_only":target,"native_ce":loss,"native":{"pool":{"summary":out.actions.summary},"copy_token_ids":out.copy_token_ids,"generate_raw_scores_sha256":sha256_bytes(&serde_json::to_vec(&out.generate.scores_q24)?),"copy_raw_scores_sha256":sha256_bytes(&serde_json::to_vec(&out.copy_scores_q24)?),"post_state_codes":out.final_state_codes.iter().map(|c|c.index()).collect::<Vec<_>>(),"continuation":{"state_codes":continuation.final_state_codes.iter().map(|c|c.index()).collect::<Vec<_>>(),"delta_scores_q24_sha256":sha256_bytes(&serde_json::to_vec(&continuation.field.delta_scores_q24)?)}}}));
            }
            let mut generated = Vec::new();
            let mut generation = Vec::new();
            let mut eos = false;
            for _ in 0..32 {
                deadline(a, start)?;
                let out = forward(learner, e, &generated, cue, prefix)?;
                let chosen = out.actions.summary.chosen_token_id;
                generation.push(json!({"actual_prefix_ids":generated,"pool":{"summary":out.actions.summary},"generate_raw_scores_sha256":sha256_bytes(&serde_json::to_vec(&out.generate.scores_q24)?),"copy_raw_scores_sha256":sha256_bytes(&serde_json::to_vec(&out.copy_scores_q24)?)}));
                generated.push(chosen);
                if chosen == l.generate.binding().eos_token_id() {
                    eos = true;
                    break;
                }
            }
            let text = l
                .tokenizer
                .decode(&generated[..generated.len() - usize::from(eos)]);
            let accepted = eos && e.answers.accepts(&text);
            complete += usize::from(accepted);
            ce += rowce / e.target.len() as f64;
            let filename = format!("{name}-row-{:04}.json", rows.len());
            write(
                a,
                &filename,
                &json!({"id":e.packet.id,"canonical":canonical,"generation":generation,"generated_ids":generated,"decoded":text,"eos":eos,"complete":accepted,"native_equal_episode_ce":rowce/e.target.len()as f64}),
            )?;
            rows.push(json!({"id":e.packet.id,"complete":accepted,"eos":eos,"generated_ids":generated,"row_file":filename,"row_sha256":sha256_file(&a.out.join(&filename))?}));
        }
        let result = json!({"cases":eps.len(),"complete":complete,"rows":rows,"native_equal_episode_ce":ce/eps.len()as f64,"runtime":"CUDA exact Generate/U and learning state path, host bank/alias reduction; not fully device resident","scope":"exposed development; own emitted prefix/EOS membership, not fresh qualification"});
        write(a, &format!("{name}.json"), &result)?;
        Ok(result)
    })
}

fn baseline_score_parity(a: &Args, saved: &Value, current: &Value) -> Result<()> {
    let root = &a
        .unified_geometry
        .as_ref()
        .ok_or_else(|| bad("unified config absent"))?
        .saved437;
    let old = saved["rows"]
        .as_array()
        .ok_or_else(|| bad("unified saved rows absent"))?;
    let new = current["rows"]
        .as_array()
        .ok_or_else(|| bad("unified current rows absent"))?;
    replay_require(
        old.len() == 512 && new.len() == 512,
        "unified baseline full512 coverage differs",
    )?;
    for (old, new) in old.iter().zip(new) {
        let old_path = root.join(
            old["row_file"]
                .as_str()
                .ok_or_else(|| bad("unified saved row path absent"))?,
        );
        let new_path = a.out.join(
            new["row_file"]
                .as_str()
                .ok_or_else(|| bad("unified current row path absent"))?,
        );
        replay_require(
            sha256_file(&old_path)? == old["row_sha256"]
                && sha256_file(&new_path)? == new["row_sha256"],
            "unified baseline row identity differs",
        )?;
        let old = read(&old_path)?;
        let new = read(&new_path)?;
        let a = old["canonical"]
            .as_array()
            .ok_or_else(|| bad("unified saved canonical absent"))?;
        let b = new["canonical"]
            .as_array()
            .ok_or_else(|| bad("unified current canonical absent"))?;
        replay_require(
            old["id"] == new["id"] && a.len() == b.len(),
            "unified baseline row/position coverage differs",
        )?;
        for (a, b) in a.iter().zip(b) {
            for pointer in [
                "/target_label_only",
                "/native/pool/summary",
                "/native/copy_token_ids",
                "/native/post_state_codes",
                "/native/generate_raw_scores_sha256",
                "/native/continuation/state_codes",
                "/native/continuation/delta_scores_q24_sha256",
            ] {
                replay_require(
                    a.pointer(pointer).is_some() && a.pointer(pointer) == b.pointer(pointer),
                    "unified baseline scores/states/pool differ",
                )?;
            }
        }
    }
    Ok(())
}

fn batch(
    a: &Args,
    l: &Loaded,
    u: &ContinuationLearningWeights,
    eps: &[Episode],
    indices: &[usize],
    d: &Device,
    start: Instant,
) -> Result<(BTreeMap<String, Tensor>, Value)> {
    let credit = a
        .unified_geometry
        .as_ref()
        .ok_or_else(|| bad("unified config absent"))?
        .field_carrier_credit;
    let parameters = active_parameters(l)?;
    with_bank(l, u, credit, d, |learner, cue, prefix| {
        let mut sums = BTreeMap::<String, Tensor>::new();
        let mut total = 0.;
        let mut positions = 0;
        for &index in indices {
            let e = eps
                .get(index)
                .ok_or_else(|| bad("unified batch index absent"))?;
            let union = e
                .views
                .iter()
                .flatten()
                .flat_map(|v| v.emitted_token_ids().iter().copied())
                .collect::<BTreeSet<_>>();
            let mut aggregate = cross_state_completion::EpisodeBottleneck::default();
            for (t, &target) in e.target.iter().enumerate() {
                deadline(a, start)?;
                let out = forward(learner, e, supervised_prefix(&e.target, t)?, cue, prefix)?;
                replay_require(
                    out.copy_token_ids.iter().copied().collect::<BTreeSet<_>>() == union,
                    "unified all-source Copy union differs",
                )?;
                let loss = cross_state_completion::rank_loss_from_pool(
                    &out.actions,
                    &out.generate.raw_scores,
                    out.copy.as_ref().map(|c| &c.copy_raw),
                    target,
                )?;
                let value = loss.to_scalar::<f32>()? as f64;
                let gradients = loss.backward()?;
                let mut token = BTreeMap::new();
                for (name, var) in &parameters {
                    let g = match gradients.get(var.as_tensor()) {
                        Some(g) => g.clone(),
                        None => Tensor::zeros(var.dims(), candle_core::DType::F32, d)?,
                    };
                    replay_require(
                        g.device().same_device(d)
                            && g.abs()?
                                .flatten_all()?
                                .max(0)?
                                .to_scalar::<f32>()?
                                .is_finite(),
                        "unified gradient device/finite admission",
                    )?;
                    token.insert(name.clone(), g.detach());
                }
                aggregate.push(value, token)?;
                positions += 1;
            }
            let (value, gradients) = aggregate.finish(e.target.len(), indices.len())?;
            total += value;
            for (name, gradient) in gradients {
                let sum = if let Some(old) = sums.remove(&name) {
                    (old + gradient)?
                } else {
                    gradient
                };
                sums.insert(name, sum.detach());
            }
        }
        Ok((
            sums,
            json!({"indices":indices,"positions":positions,"objective":total,"field_carrier_credit":credit,"objective_policy":OBJECTIVE,"token_backward_chunk":1,"snapshot":"fresh current Source/Generate/U once per batch; no state survives update"}),
        ))
    })
}

pub(super) fn run(a: &Args, start: Instant, d: &Device) -> Result<Value> {
    settings(a)?;
    replay_require(
        d.is_cuda(),
        "unified long runs require CUDA; no CPU fallback",
    )?;
    let config = a
        .unified_geometry
        .as_ref()
        .ok_or_else(|| bad("unified config absent"))?;
    let parent = ContinuationParent::load(a)?;
    let l = load_joint_continuation(a, &parent, d)?;
    let (u, saved) = load_field(a, &parent, d)?;
    let legal = NativeVocabularyActions::new(l.integer.binding().clone(), &l.exp)?
        .legal_token_ids()
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    let eps = load_panel(
        &a.training_inputs,
        &a.training_labels,
        &l.integer,
        &l.tokenizer,
        &legal,
        512,
    )?;
    replay_require(
        eps.len() == 512 && eps.iter().all(Episode::has_source),
        "unified complete512 bank panel differs",
    )?;
    pairs(&eps)?;
    let active = active_parameters(&l)?;
    let frozen = frozen_parameters(&l, &u)?;
    let context = l.source.context_state_parameters();
    let prototypes = BTreeMap::from([(
        "generate.prototype_choices".into(),
        l.generate.prototype_choices.clone(),
    )]);
    let schedule = order(a.seed, 512);
    write(
        a,
        "admission.json",
        &json!({"mode":"unified_geometry","source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT"),"device":"cuda:0","config":config,"base_report_sha256":REPORT,"base_manifest_sha256":MANIFEST,"base_field_sha256":FIELD,"base_master_sha256":MASTER,"initial_active_masters":identities(&active)?,"frozen_masters":frozen,"active_names":active.keys().collect::<Vec<_>>(),"epochs":2,"updates":128,"batch":8,"objective":OBJECTIVE,"native_forward":"unchanged typed H4 products, full Generate/all physical Copy aliases, common clipping and pool","credit_scope":uor_r4_training::geometric_continuation_learning::JOINT_CREDIT_SCOPE,"device_scope":"CUDA tensors/Generate integer kernel; host artifact preparation and exact alias reduction remain explicit","cost_scope":"no whole-path speed/energy or SpiralCore basis bridge inferred"}),
    )?;
    write(
        a,
        "order.json",
        &json!({"seed":a.seed,"order":schedule,"epochs":2}),
    )?;
    let (initial_parent, initial_field) = checkpoint_current(a, 0, &l, &u)?;
    replay_require(
        initial_parent.binding == parent.binding
            && initial_parent.generate == parent.generate
            && initial_field.to_bytes()?
                == fs::read(
                    config
                        .saved437
                        .join("checkpoint-0256/continuation-field.bin"),
                )?,
        "unified zero-update native artifact changed",
    )?;
    let parity = joint_native_parity(
        &l,
        &u,
        &initial_parent,
        &initial_field,
        &eps,
        &cross_state_completion::ORIGINAL8,
        d,
        None,
    )?;
    write(a, "initial-original8-native-parity.json", &parity)?;
    let initial = evaluate_cuda(a, "development-0000", &l, &u, &eps, d, start)?;
    let comparison = joint_row_comparison(&saved, &initial)?;
    baseline_score_parity(a, &saved, &initial)?;
    replay_require(
        initial["complete"] == 437
            && comparison["changed_output_ids"] == json!([])
            && comparison["lost_complete_ids"] == json!([]),
        "unified saved437 own-prefix baseline differs",
    )?;
    write(a, "baseline-comparison.json", &comparison)?;
    let mut co = optimizer(&context, config.context_learning_rate)?;
    let mut po = optimizer(&prototypes, config.prototype_learning_rate)?;
    let mut updates = Vec::new();
    for update in 0..128 {
        disk_floor(a)?;
        let indices = cross_state_completion::schedule_indices(&schedule, update);
        let clock = Instant::now();
        let (gradients, receipt) = batch(a, &l, &u, &eps, &indices, d, start)?;
        let (denominator, norm) = clip_denominator(&gradients, d)?;
        apply(&mut co, &context, &gradients, &denominator)?;
        apply(&mut po, &prototypes, &gradients, &denominator)?;
        for var in context.values() {
            var.set(&var.as_tensor().clamp(-1.75, 1.75)?)?;
        }
        d.synchronize()?;
        updates.push(json!({"step":update+1,"before_update":receipt,"global_norm":norm,"elapsed_seconds":clock.elapsed().as_secs_f64()}));
        write(a, "updates.json", &json!(updates))?;
        if (update + 1) % 32 == 0 {
            replay_require(
                frozen_parameters(&l, &u)? == frozen,
                "unified frozen masters mutated",
            )?;
            checkpoint_current(a, update + 1, &l, &u)?;
        }
    }
    replay_require(
        frozen_parameters(&l, &u)? == frozen,
        "unified final frozen masters mutated",
    )?;
    let root = a.out.join("checkpoint-0128");
    let final_parent = ContinuationParent::from_checkpoint(&root)?;
    let mut reload_args = a.clone();
    reload_args.checkpoint = root.clone();
    let reloaded = load_joint_continuation(&reload_args, &final_parent, d)?;
    let bytes = fs::read(root.join("continuation-field.bin"))?;
    let generator = final_parent.generator()?;
    let field = NativeContinuationField::from_bytes(
        &bytes,
        &final_parent.binding,
        generator.generate_model(),
    )?;
    let reloaded_u = ContinuationLearningWeights::from_native(
        &field,
        generator.generate_model(),
        final_parent.integer.binding(),
        d,
    )?;
    restore(
        &root.join("continuation-source"),
        &final_parent.receipt["continuation_parameters"],
        &reloaded_u.parameters(),
        d,
    )?;
    replay_require(
        identities(&active_parameters(&reloaded)?)? == identities(&active)?,
        "unified reloaded active masters differ",
    )?;
    let parity = joint_native_parity(
        &reloaded,
        &reloaded_u,
        &final_parent,
        &field,
        &eps,
        &cross_state_completion::ORIGINAL8,
        d,
        None,
    )?;
    write(a, "final-original8-native-parity.json", &parity)?;
    let final_eval = evaluate_cuda(
        a,
        "development-0128",
        &reloaded,
        &reloaded_u,
        &eps,
        d,
        start,
    )?;
    let comparison = joint_row_comparison(&initial, &final_eval)?;
    write(a, "complete-row-comparison.json", &comparison)?;
    Ok(
        json!({"schema":"uor-r4.unified-geometric-learning/1","status":"COMPLETED","mode":"unified_geometry","source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT"),"config":config,"order_seed":a.seed,"updates":128,"epochs":2,"objective":OBJECTIVE,"initial":initial,"final":final_eval,"rowwise":comparison,"final_receipt":final_parent.receipt,"final_active_masters":identities(&active)?,"frozen_masters":frozen,"elapsed_seconds":start.elapsed().as_secs_f64(),"scope":"exposed512 live Context/prototype geometric learning; opened diagnostics and genuinely fresh qualification NOT_RUN; no whole-path energy or general chat claim"}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config_value() -> Value {
        json!({"mode":"unified_geometry","credit":"raw_identity","read_state_pullback":"categorical",
            "seed":1001,"checkpoint":"parent/checkpoint-0000","saved_fit":"parent","categorical":"categorical",
            "parent_config":"parent-config.json","training_inputs":"inputs.json","training_labels":"labels.json",
            "development_inputs":"inputs.json","development_labels":"labels.json","maximum_seconds":7200,
            "maximum_report_bytes":1073741824,"out":"fresh-attempt","updates":128,"loss_scope":"all",
            "unified_geometry":{"saved437":"saved437","field_carrier_credit":true,"context_learning_rate":0.002,"prototype_learning_rate":0.01}})
    }

    #[test]
    fn unified_recipe_rejects_scientific_drift_and_keeps_both_controls() -> Result<()> {
        for seed in [1001, 2001] {
            for credit in [false, true] {
                let mut value = config_value();
                value["seed"] = json!(seed);
                value["unified_geometry"]["field_carrier_credit"] = json!(credit);
                settings(&serde_json::from_value(value)?)?;
            }
        }
        for (path, value) in [
            ("/seed", json!(1002)),
            ("/updates", json!(64)),
            ("/credit", json!("clipped")),
            ("/read_state_pullback", json!("legacy")),
            ("/loss_scope", json!("entry_only")),
            ("/query_conditioned_read", json!(true)),
            ("/native_code_proposals", json!(true)),
            ("/cross_state_pooled_rank", json!(true)),
            ("/unified_geometry/context_learning_rate", json!(0.02)),
            ("/unified_geometry/prototype_learning_rate", json!(0.1)),
        ] {
            let mut changed = config_value();
            let object = changed
                .as_object_mut()
                .ok_or_else(|| bad("fixture object absent"))?;
            // Optional flags are absent from the minimal config initially.
            if path.matches('/').count() == 1 {
                object.insert(path[1..].into(), value);
            } else {
                *changed
                    .pointer_mut(path)
                    .ok_or_else(|| bad("fixture field absent"))? = value;
            }
            assert!(
                settings(&serde_json::from_value(changed)?).is_err(),
                "admitted {path}"
            );
        }
        let mut wrong_mode = config_value();
        wrong_mode["mode"] = json!("fit");
        assert!(settings(&serde_json::from_value(wrong_mode)?).is_err());
        let mut unknown = config_value();
        unknown["unified_geometry"]["change_panel"] = json!(true);
        assert!(serde_json::from_value::<Args>(unknown).is_err());
        Ok(())
    }

    #[test]
    #[ignore = "requires explicit authenticated saved437 artifact config; never pass when absent"]
    fn unified_actual_saved437_restores_fractional_weights_and_factor_digest() -> Result<()> {
        let path = std::env::var_os("UOR_UNIFIED_FIXTURE_CONFIG")
            .ok_or_else(|| bad("UOR_UNIFIED_FIXTURE_CONFIG is required"))?;
        let a: Args = serde_json::from_slice(&fs::read(PathBuf::from(path))?)?;
        settings(&a)?;
        let p = ContinuationParent::load(&a)?;
        let (u, saved) = load_field(&a, &p, &Device::Cpu)?;
        replay_require(
            identities(&u.parameters())?["continuation.cross_state"] == MASTER,
            "actual fractional bits differ",
        )?;
        let root = &a
            .unified_geometry
            .as_ref()
            .ok_or_else(|| bad("fixture config absent"))?
            .saved437;
        let row_ref = &saved["rows"][0];
        let row_path = root.join(
            row_ref["row_file"]
                .as_str()
                .ok_or_else(|| bad("saved row filename absent"))?,
        );
        replay_require(
            sha256_file(&row_path)? == row_ref["row_sha256"],
            "actual saved row hash differs",
        )?;
        let row = read(&row_path)?;
        let native = &row["canonical"][0]["native"];
        let codes = |value: &Value| -> Result<Vec<uor_r4_integer::h4_tables::H4Code>> {
            value
                .as_array()
                .ok_or_else(|| bad("saved codes absent"))?
                .iter()
                .map(|v| {
                    let index = v
                        .as_u64()
                        .filter(|v| *v < 120)
                        .ok_or_else(|| bad("saved code outside H4"))?;
                    uor_r4_integer::h4_tables::H4Code::try_from(index as u8)
                        .map_err(|e| bad(&e.to_string()))
                })
                .collect()
        };
        let factual = codes(&native["post_state_codes"])?;
        let local = codes(&native["continuation"]["state_codes"])?;
        let g = p.generator()?;
        let field = u.export_native(&p.binding, g.generate_model())?;
        let mut delta = vec![0; 4096];
        let mut counts = uor_r4_core::native_geometric::learner::geometric_continuation_field::ContinuationReadCounts::default();
        field.score_delta_with_factual_into(
            &factual,
            &local,
            g.generate_model(),
            &mut delta,
            &mut counts,
        )?;
        assert_eq!(
            sha256_bytes(&serde_json::to_vec(&delta)?),
            native["continuation"]["delta_scores_q24_sha256"]
        );
        assert!(counts.coefficient_reads <= 960);
        assert_eq!(counts.algebra.product_reads, 2 * counts.coefficient_reads);
        assert_eq!(counts.prototype_reads, 32768);
        assert_eq!(counts.factor_cache_reads, 32768);
        let mut scores = vec![0; 4096];
        g.generate_model()
            .score_into(&factual, &mut scores, &mut GenerateReadCounts::default())?;
        for (score, delta) in scores.iter_mut().zip(delta) {
            *score = score
                .checked_add(delta)
                .ok_or_else(|| bad("actual field overflow"))?;
        }
        assert_eq!(
            sha256_bytes(&serde_json::to_vec(&scores)?),
            native["generate_raw_scores_sha256"]
        );
        assert_eq!(sha256_bytes(&field.to_bytes()?), FIELD);
        eprintln!("ACTUAL_SAVED437_SINGLE_POSITION: exact field/Generate digests; coefficient_reads={}; H4_products={}; token/prototype_reads=32768; full ownprefix replay NOT_RUN", counts.coefficient_reads, counts.algebra.product_reads);
        Ok(())
    }
}
