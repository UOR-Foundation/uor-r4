//! One original-parent Prefix-only fragment construction at authenticated actual prefixes.
//! Existing retained geometric features and donor-specific native hard pools; no serving changes.
use super::context_cue_coadapt as shared;
use super::native_proposals as np;
use super::*;
use uor_r4_training::geometric_occurrence_consumer::source_realizer::{
    CueAngularWeights, PrefixAngularWeights,
};
const NAME: &str = "prefix.coefficients";
#[derive(Clone, Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Config {
    pub retained_intermediate_root: PathBuf,
    pub retained_probe_root: PathBuf,
}
pub(super) fn validate_settings(a: &Args) -> Result<()> {
    if let Some(c) = &a.prefix_fragment_learning {
        replay_require(
            a.mode == Mode::JointContinuation
                && a.updates == 1
                && a.loss_scope == LossScope::All
                && a.context_cue_coadapt.is_none()
                && a.prefix_context_credit.is_none()
                && a.context_path_credit.is_none()
                && a.readout_coadaptation.is_none()
                && a.reached_u.is_none()
                && a.prototype_compensation.is_none()
                && a.reference_replay.is_none()
                && a.retained_context_root.is_none()
                && !a.native_code_proposals
                && !a.reached_frontier_objective
                && !a.constrained_context_learning
                && !a.constrained_emission_learning
                && !a.categorical_action_learning
                && !a.categorical_action_only,
            "Prefix fragment requires exclusive original-parent one-pass joint mode",
        )?;
        replay_require(
            fs::canonicalize(&a.checkpoint)?
                == fs::canonicalize(c.retained_intermediate_root.join("checkpoint-0001"))?
                && a.maximum_report_bytes <= 256 * 1024 * 1024,
            "Prefix original parent/report admission differs",
        )?;
    }
    Ok(())
}
pub(super) fn policy() -> Value {
    json!({"schema":"uor-r4.prefix-fragment-learning/1","active_family":NAME,"coordinates":960,
 "initialization":"ORIGINAL Source9f/Cue/Prefix fractional masters/G4248/U82ae; no negative Context/Cue adoption",
 "gradient":"one aggregate from18 backwards; only Prefix master gradients extracted/proposed; frozen Generate graph transports direct gather and detached conditional-donor contrast, not hard argmax derivative",
 "construction":"one frozen actual-master-delta preferred adjacent960 pass; strict current18 CE descent AND all17 original winners per accepted transaction; no refill/revisit/rerank",
 "final_gate":"strict ORIGINAL18 CE descent, exact task probability increase, all17 original winners",
 "fragment_correction":"actual pooled target267 winner is a distinct required qualification",
 "strict_ce_tolerance":"1e-10*(1+abs(current_or_original_CE))","optimizer_updates":0,"context_gradient":"NOT_RUN","cue_gradient":"NOT_RUN","serving_changes":false})
}
fn transaction_accept(current: &Value, staged: &Value) -> Result<bool> {
    let before = current["combined"]
        .as_f64()
        .ok_or_else(|| bad("Prefix current CE missing"))?;
    let after = staged["combined"]
        .as_f64()
        .ok_or_else(|| bad("Prefix staged CE missing"))?;
    replay_require(
        before.is_finite() && after.is_finite(),
        "Prefix CE nonfinite",
    )?;
    Ok(after < before - 1e-10 * (1. + before.abs()) && staged["correct_reference_frames"] == 17)
}
pub(super) fn run(a: &Args, start: Instant, d: &Device) -> Result<Value> {
    validate_settings(a)?;
    replay_require(
        !d.is_cpu(),
        "Prefix coefficient graph requires admitted CUDA device",
    )?;
    let c = a
        .prefix_fragment_learning
        .as_ref()
        .ok_or_else(|| bad("Prefix config absent"))?;
    shared::sealed(
        &c.retained_intermediate_root,
        shared::P_REPORT,
        shared::P_SEAL,
    )?;
    shared::sealed(&c.retained_probe_root, shared::B_REPORT, shared::B_SEAL)?;
    let original = ContinuationParent::from_checkpoint(&a.checkpoint)?;
    replay_require(
        original.binding.metadata_sha256 == shared::SOURCE
            && sha256_bytes(&original.generate) == shared::G_SHA
            && sha256_file(&a.checkpoint.join("continuation-field.bin"))? == shared::U_SHA,
        "Prefix original Source/G/U differs",
    )?;
    let sw = SourceRealizerWeights::load_context_potential_on_device(
        &a.checkpoint.join("source"),
        &fs::read(a.checkpoint.join("native/tokenizer.json"))?,
        &Device::Cpu,
    )?;
    let native = NativeSourceRealizer::load(
        &a.checkpoint.join("native"),
        &sw,
        &shared::dec::<ConsumerIdentity>(
            &read(&a.checkpoint.join("native/metadata.json"))?["identity"],
        )?,
    )?;
    let source_bits = identities(&sw.parameters())?;
    let cue = CueAngularWeights::load(
        &a.checkpoint.join("cue"),
        &native,
        &a.checkpoint.join("native"),
    )?;
    let carrier = native.compile_cue_carrier(cue.native()?)?;
    let actual = PrefixAngularWeights::load(
        &a.checkpoint.join("prefix"),
        &native,
        &a.checkpoint.join("native"),
        &a.checkpoint.join("cue"),
        &carrier,
    )?;
    let prefix_payload = native.compile_prefix_transport(&carrier, actual.native()?)?;
    // Device construction is not master authority: restore all original f32 bits before any credit.
    let active = PrefixAngularWeights::from_native(
        &native,
        &a.checkpoint.join("native"),
        &a.checkpoint.join("cue"),
        &carrier,
        &prefix_payload,
        d,
    )?;
    let actual_bits = np::snapshot(&actual.parameters())?;
    np::restore(&active.parameters(), &actual_bits)?;
    replay_require(
        np::same_bits(&actual_bits, &np::snapshot(&active.parameters())?)
            && active.packed_coefficients()?
                == fs::read(a.checkpoint.join("prefix/prefix-q4.bin"))?,
        "Prefix original fractional masters/native payload lost",
    )?;
    let parent = shared::floats(&a.checkpoint.join("prefix/prefix-source-f32.bin"))?;
    replay_require(
        actual_bits.get(NAME).is_some_and(|v| {
            v.iter()
                .zip(&parent)
                .all(|(a, b)| a.to_bits() == b.to_bits())
        }),
        "Prefix original master receipt differs",
    )?;
    let seed = shared::Config {
        retained_intermediate_root: c.retained_intermediate_root.clone(),
        retained_probe_root: c.retained_probe_root.clone(),
        retained_finite_root: c.retained_probe_root.clone(),
    };
    let (frames, baseline) = shared::prepare_frames(a, &seed, &original, &original, true)?;
    let ng = NativeGeometricGenerate::from_bytes(&original.generate, original.integer.binding())?;
    let field = NativeContinuationField::from_bytes(
        &fs::read(a.checkpoint.join("continuation-field.bin"))?,
        &original.binding,
        &ng,
    )?;
    for f in &frames {
        let state = shared::codes(&f.native["continuation"]["state_codes"])?;
        let mut u = vec![0; 4096];
        field.score_delta_into(&state, &ng, &mut u, &mut Default::default())?;
        replay_require(u == f.u, "Prefix frozen U saved arithmetic differs")?;
    }
    write(
        a,
        "prefix-original-master-binding.json",
        &json!({"original_master_file":a.checkpoint.join("prefix/prefix-source-f32.bin"),
  "original_master_sha256":sha256_file(&a.checkpoint.join("prefix/prefix-source-f32.bin"))?,
  "shape":[960],"bytes":3840,"all960_original_bits_restored_before_graph":true,
  "source_binding":original.binding,"frozen_cue":carrier.metadata(),"prefix_metadata":prefix_payload.metadata(),
  "temporary_quarter_constructor":"all960 values overwritten with authenticated original f32; not initialization authority",
  "source_parameters":source_bits,"cue_parameters":identities(&cue.parameters())?}),
    )?;
    fs::write(
        a.out.join("prefix-initial-masters.f32le"),
        parent
            .iter()
            .flat_map(|x| x.to_le_bytes())
            .collect::<Vec<_>>(),
    )?;
    let saved_bytes = frames.iter().try_fold(0u64, |s, f| {
        Ok::<_, Box<dyn std::error::Error>>(s + serde_json::to_vec(&f.native)?.len() as u64)
    })?;
    let numerical = saved_bytes * 3 + shared::CACHE_LIMIT as u64 + 64 * 1024 * 1024;
    let projected = size(&a.checkpoint)? + saved_bytes + 48 * 1024 * 1024;
    replay_require(
        numerical <= 512 * 1024 * 1024 && projected + 1048576 < a.maximum_report_bytes,
        "Prefix resource projection exceeded",
    )?;
    write(
        a,
        "resource-projection.json",
        &json!({"numerical_upper_bound_bytes":numerical,"report_projection_bytes":projected,
  "numeric_cap":536870912,"report_cap":a.maximum_report_bytes,"process_ram_cap":4294967296u64,"temporary_cap":536870912,
  "threads":2,"donor_cache_cap":shared::CACHE_LIMIT,"scope":"saved nativeframes/rawvectors/stagedpools/cache; model/autodiff tensors charged separately to process RAM"}),
    )?;
    let mut cache = shared::DonorCache::new(&original);
    let mut reducer =
        NativeVocabularyActions::new(original.integer.binding().clone(), &original.exp)?;
    let mut pools = frames
        .iter()
        .enumerate()
        .map(|(i, f)| shared::score(i, f, &parent, &parent, &original, &mut cache, &mut reducer))
        .collect::<Result<Vec<_>>>()?;
    let initial = shared::objective(&frames, &pools)?;
    replay_require(
        initial == baseline && initial["correct_reference_frames"] == 17,
        "Prefix ORIGINAL native initial objective differs",
    )?;
    write(a, "initial-original-objective.json", &initial)?;
    let g = GenerateLearningWeights::from_native(original.integer.binding().clone(), &ng, d)?;
    restore(
        &a.checkpoint.join("generate-source"),
        &read(&a.checkpoint.join("generate-source/metadata.json"))?["parameters"],
        &g.parameters(),
        d,
    )?;
    let gradients = shared::gradient(
        a,
        start,
        &frames,
        &original,
        &shared::ActiveCredit::Prefix(&active),
        &g,
        &parent,
        &pools,
        &mut cache,
    )?;
    drop(g);
    let ranking = shared::ranking(&parent, &gradients)?;
    write(a, "frozen-prefix-ranking.json", &json!(ranking))?;
    let mut current = parent.clone();
    let mut value = initial.clone();
    let mut trials = Vec::new();
    let mut accepted = 0;
    for (order, m) in ranking.iter().enumerate() {
        shared::progress(a, start)?;
        if m.status != "eligible" {
            trials.push(json!({"order":order,"move":m,"status":m.status,"current":value,"staged":"NOT_RUN",
    "native_effect":"no code displacement; existing18 full pools reused","original_gate":shared::gate(&baseline,&value)?}));
            continue;
        }
        let before = value.clone();
        let mut proposed = current.clone();
        proposed[m.index] = m.master_after;
        let staged = frames
            .iter()
            .enumerate()
            .map(|(i, f)| {
                shared::score(
                    i,
                    f,
                    &parent,
                    &proposed,
                    &original,
                    &mut cache,
                    &mut reducer,
                )
            })
            .collect::<Result<Vec<_>>>()?;
        let next = shared::objective(&frames, &staged)?;
        let accept = transaction_accept(&value, &next)?;
        trials.push(json!({"order":order,"move":m,"status":if accept{"accepted"}else{"rejected"},"before":before,"staged":next,
   "native_all18_checked":true,"strict_current_ce_and_all17_original_winners":accept,
   "original_task_probability_improved":shared::improved(&baseline,&next)?,"original_gate":shared::gate(&baseline,&next)?}));
        if accept {
            current = proposed;
            pools = staged;
            value = next;
            accepted += 1;
        }
    }
    write(
        a,
        "prefix-construction.json",
        &json!({"coordinates":960,"accepted":accepted,"trials":trials,"initial":initial,
  "final_objective":value,"frozen_order":true,"revisited_coordinates":0,"cache_peak_bytes":cache.peak,
  "donor_recomputations":cache.calls,"cache_identity":{"source_binding":cache.source_binding,"generate_sha256":cache.generate_sha256,"bridge_sha256":cache.bridge_sha256},
  "cache_key":"physical ordinal/frame/occurrence state within immutable Source/G/bridge epoch","accepted_guard":"all17 original winners on every accepted transaction"}),
    )?;
    let params = active.parameters();
    let saved = np::snapshot(&params)?;
    drop(sw);
    let loaded = load_joint_continuation(a, &original, d)?;
    let frozen_source = identities(&loaded.source.parameters())?;
    replay_require(
        frozen_source == source_bits,
        "Prefix loaded frozen Source fractional bits differ",
    )?;
    let frozen_generate = identities(&loaded.generate.parameters())?;
    let frozen_bridges = json!({"original":identities(&loaded.original_bridge.parameters())?,"marker":identities(&loaded.marker.parameters())?,
  "categorical":loaded.categorical.as_ref().map(|x|identities(&x.parameters())).transpose()?});
    let result = np::attempt_restored(&params, &saved, || {
        np::restore(
            &params,
            &BTreeMap::from([(NAME.to_string(), current.clone())]),
        )?;
        replay_require(
            frozen_source == identities(&loaded.source.parameters())?
                && frozen_generate == identities(&loaded.generate.parameters())?
                && frozen_bridges
                    == json!({"original":identities(&loaded.original_bridge.parameters())?,"marker":identities(&loaded.marker.parameters())?,
   "categorical":loaded.categorical.as_ref().map(|x|identities(&x.parameters())).transpose()?}),
            "Prefix frozen model master bits changed",
        )?;
        let (cp, u, receipt) = shared::export_joint(a, &loaded, &cue, Some(&active), &original, d)?;
        replay_require(
            cp.binding == original.binding
                && cp.generate == original.generate
                && cp.bridge == original.bridge
                && cp.joint == original.joint
                && cp.cue == original.cue
                && u.to_bytes()? == fs::read(a.checkpoint.join("continuation-field.bin"))?,
            "Prefix export changed frozen Source/Cue/G/bridge/U artifacts",
        )?;
        replay_require(
            fs::read(a.out.join("checkpoint-0001/cue/cue-source-f32.bin"))?
                == fs::read(a.checkpoint.join("cue/cue-source-f32.bin"))?,
            "Prefix frozen Cue f32 bits changed",
        )?;
        shared::reload_candidate(a, &cp, &u, &frames, &pools)?;
        Ok(receipt)
    });
    replay_require(
        np::same_bits(&saved, &np::snapshot(&params)?),
        "Prefix original master bits not restored",
    )?;
    let receipt = result?;
    let final_gate = shared::gate(&baseline, &value)?;
    let task = value["terms"]
        .as_array()
        .ok_or_else(|| bad("Prefix final terms absent"))?
        .iter()
        .find(|x| x["input_index"] == 245 && x["position"] == 4)
        .ok_or_else(|| bad("Prefix task term absent"))?;
    let corrected = task["pool"]["chosen_token_id"] == 267;
    write(a, "final-objective.json", &value)?;
    Ok(
        json!({"schema":"uor-r4.prefix-fragment-learning/1","status":"COMPLETED","mode":"prefix_fragment_learning","policy":policy(),
 "source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT"),"baseline_objective":baseline,"initial_original":initial,"candidate_objective":value,
 "final_gate":final_gate,"finite_prefix_positive":final_gate["finite_joint_positive"],"actual_fragment_corrected":corrected,
 "qualified_fragment":final_gate["finite_joint_positive"]==true && corrected,"selected_model":false,"useful_candidate":false,
 "candidate_receipt":receipt,"parent_master_bits_restored":true,"new_prefix_gradients":1,"prefix_backward_calls":18,
 "new_context_gradients":0,"new_cue_gradients":0,"optimizer_updates":0,"candidate_native_steps":18,"baseline_encoder_calls":0,
 "autoregressive_rollout":"NOT_RUN; parent admits cheap actual-artifact ownprefix/multiturn/original8 only after construction gate"}),
    )
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn prefix_transaction_rejects_descent_with_lost_original_winner() -> Result<()> {
        let current = json!({"combined":4.0,"correct_reference_frames":17});
        assert!(!transaction_accept(
            &current,
            &json!({"combined":3.0,"correct_reference_frames":16})
        )?);
        assert!(!transaction_accept(
            &current,
            &json!({"combined":4.0,"correct_reference_frames":17})
        )?);
        assert!(transaction_accept(
            &current,
            &json!({"combined":3.9,"correct_reference_frames":17})
        )?);
        Ok(())
    }
    #[test]
    fn prefix_unmasked_lane_updates_every_physical_alias() -> Result<()> {
        let mut parent = vec![0f32; 960];
        parent[941] = -0.139;
        let mut changed = parent.clone();
        changed[941] = -0.5;
        let row = vec![
            Some(81),
            Some(156),
            Some(303),
            Some(424),
            Some(533),
            Some(712),
            Some(825),
            Some(941),
        ];
        let keys = vec![
            row.clone(),
            row,
            vec![
                Some(59),
                Some(176),
                Some(316),
                Some(383),
                Some(533),
                Some(691),
                Some(798),
                Some(935),
            ],
        ];
        let original = vec![7, 9, 11];
        let next = shared::staged_base(&original, &keys, &parent, &changed)?;
        assert_eq!(next, vec![7 - (1 << 22), 9 - (1 << 22), 11]);
        assert!(shared::staged_base(&original, &[vec![Some(960); 8]], &parent, &changed).is_err());
        Ok(())
    }
    #[test]
    fn prefix_late_export_error_restores_fractional_bits_and_frozen_cue() -> Result<()> {
        let prefix = Var::from_vec(vec![0.139f32, -0.0], 2, &Device::Cpu)?;
        let cue = Var::from_vec(vec![-0.14452013f32, -0.0], 2, &Device::Cpu)?;
        let vars = BTreeMap::from([
            (NAME.to_string(), prefix),
            ("cue.coefficients".to_string(), cue),
        ]);
        let original = np::snapshot(&vars)?;
        let result: Result<()> = np::attempt_restored(&vars, &original, || {
            let mut changed = original.clone();
            changed
                .get_mut(NAME)
                .ok_or_else(|| bad("test Prefix absent"))?[0] = 0.5;
            np::restore(&vars, &changed)?;
            Err(bad("late export/parity failure"))
        });
        assert!(result.is_err());
        assert!(np::same_bits(&original, &np::snapshot(&vars)?));
        Ok(())
    }
}
