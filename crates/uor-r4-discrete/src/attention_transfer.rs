//! Deferred treatment-only fresh evaluation. PM records bind reviewed bytes;
//! they do not cryptographically authenticate scientific analysis or issue a
//! training capability. An uncertain transfer intent is never replayed.
use super::*;

#[cfg(any(feature = "cuda", test))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ArmPins {
    admission: FilePin,
    final_checkpoint: FilePin,
    final_dev: FilePin,
}

#[cfg(any(feature = "cuda", test))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct TransferDecision {
    schema: String,
    verdict: String,
    pm_receipt_url: String,
    treatment: ArmPins,
    control: ArmPins,
    treatment_ablation_checkpoint: FilePin,
    treatment_ablation_dev: FilePin,
    /// Exact paired DEV analysis reviewed by PM. This module validates its byte
    /// pin, not the scientific discriminator or the analysis's authorship.
    paired_discriminator: FilePin,
}

#[cfg(any(feature = "cuda", test))]
fn validate_decision_header(decision: &TransferDecision) -> Result<()> {
    let receipt_id = decision
        .pm_receipt_url
        .strip_prefix("https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-");
    if decision.schema != "uor-r4.attention-transfer-decision/1"
        || decision.verdict != "RELEASE_TREATMENT_ONLY_FRESH_AFTER_PAIRED_DEV_REVIEW"
        || !receipt_id.is_some_and(|id| !id.is_empty() && id.bytes().all(|b| b.is_ascii_digit()))
        || !valid_receipt_url(&decision.pm_receipt_url)
    {
        return Err(invalid(
            "transfer needs a distinct explicit paired-DEV PM decision",
        ));
    }
    for pin in [
        &decision.treatment.admission,
        &decision.treatment.final_checkpoint,
        &decision.treatment.final_dev,
        &decision.control.admission,
        &decision.control.final_checkpoint,
        &decision.control.final_dev,
        &decision.treatment_ablation_checkpoint,
        &decision.treatment_ablation_dev,
        &decision.paired_discriminator,
    ] {
        corpus::validate_sha256(&pin.sha256)?;
        if !pin.path.is_absolute() {
            return Err(invalid("transfer decision pins require absolute paths"));
        }
    }
    Ok(())
}

#[cfg(any(feature = "cuda", test))]
fn reserve_intent(state: &Path, binding: &Value) -> Result<()> {
    write_new(&state.join("fresh-transfer-intent.json"), binding)?;
    sync_directory(state)
}

#[cfg(any(feature = "cuda", test))]
fn require_fresh_inventory(rows: &[(u64, u32, bool)]) -> Result<()> {
    if rows.len() < 2000
        || rows.iter().filter(|row| row.2).count() < 500
        || rows.windows(2).any(|pair| pair[0].0 >= pair[1].0)
    {
        return Err(invalid(
            "fresh transfer needs >=2000 ordered unique rows and >=500 actual copy-successor rows",
        ));
    }
    Ok(())
}

#[cfg(feature = "cuda")]
fn fresh_rows(
    source: &corpus::PreparedCorpus,
    config: &DriverConfig,
) -> Result<Vec<(u64, u32, bool)>> {
    let mut rows = Vec::with_capacity(source.eligible_len());
    let mut first = 0;
    while first < source.eligible_len() as u64 {
        let batch = source.batch(first, config.spec.context, config.batch_rows)?;
        for ((window, &position), &target) in batch
            .windows
            .iter()
            .zip(&batch.target_positions)
            .zip(&batch.targets)
        {
            rows.push((
                position,
                target,
                evidence::is_copy_successor(window, target),
            ));
        }
        first = batch.next_target;
    }
    require_fresh_inventory(&rows)?;
    Ok(rows)
}

#[cfg(feature = "cuda")]
fn pinned_json<T: serde::de::DeserializeOwned>(pin: &FilePin) -> Result<T> {
    serde_json::from_slice(&pin.read()?).map_err(|e| invalid(e.to_string()))
}

#[cfg(feature = "cuda")]
fn same_path(actual: &Path, expected: &Path) -> Result<()> {
    if fs::canonicalize(actual).map_err(|e| invalid(e.to_string()))?
        != fs::canonicalize(expected).map_err(|e| invalid(e.to_string()))?
    {
        return Err(invalid(
            "transfer pin is not the canonical registered state file",
        ));
    }
    Ok(())
}

#[cfg(feature = "cuda")]
struct Arm {
    admission: AttentionAdmission,
    driver: PreparedDriver,
    release: ValidatedRelease,
    progress: AttentionCheckpoint,
}

#[cfg(feature = "cuda")]
fn admit_arm(pins: &ArmPins, compiled: &Value, executable: &str) -> Result<Arm> {
    let admission: AttentionAdmission = pinned_json(&pins.admission)?;
    let driver = PreparedDriver::load(admission.plan.config.clone())?;
    let release = release(&driver, &admission, compiled, executable)?;
    let progress = read_progress(&pins.final_checkpoint)?;
    validate_progress(&driver, &release, &progress)?;
    if progress.checkpoint.cursor.completed_batches != ATTENTION_BATCHES
        || pins.final_checkpoint.path.file_name()
            != Some(std::ffi::OsStr::new("checkpoint-16404.json"))
    {
        return Err(invalid("transfer refuses nonfinal or selected checkpoints"));
    }
    let state = &release.manifest.run_state_directory;
    let latest: FilePin = read_json(&state.join("latest.json"))?;
    if latest.sha256 != pins.final_checkpoint.sha256 {
        return Err(invalid(
            "transfer checkpoint differs from the final state pointer",
        ));
    }
    same_path(&latest.path, &pins.final_checkpoint.path)?;
    // Preserve the exact immutable initialization baseline, not only an epoch0
    // label supplied alongside a selected final checkpoint.
    let initial_pin: FilePin = read_json(&state.join("initial.json"))?;
    let initial = read_progress(&initial_pin)?;
    validate_progress(&driver, &release, &initial)?;
    if initial.checkpoint.cursor != training::Cursor::initial()
        || initial.counters != Counters::default()
        || initial.baseline_snapshot_metadata
            != training::snapshot_metadata(&initial.checkpoint.model)?
        || progress.baseline_evaluation != initial.baseline_evaluation
        || progress.baseline_snapshot_metadata != initial.baseline_snapshot_metadata
    {
        return Err(invalid("transfer initial baseline identity differs"));
    }
    verify_evaluation(&initial.baseline_evaluation, &initial.checkpoint)?;
    Ok(Arm {
        admission,
        driver,
        release,
        progress,
    })
}

#[cfg(feature = "cuda")]
fn dev_record(
    pin: &FilePin,
    arm: &Arm,
    checkpoint: &training::DriverCheckpoint,
    filename: &str,
) -> Result<Value> {
    same_path(
        &pin.path,
        &arm.release.manifest.run_state_directory.join(filename),
    )?;
    let value: Value = pinned_json(pin)?;
    if value["schema"] != json!("uor-r4.attention-learning-evaluation/1")
        || value["admission_sha256"] != json!(arm.release.manifest_sha256)
        || value["driver_checkpoint_sha256"] != json!(corpus::sha256(&encode(checkpoint)?))
        || value["transfer"] != Value::Null
        || value["evaluation"].get("transfer_corpus").is_some()
    {
        return Err(invalid(
            "transfer DEV evidence has wrong arm/checkpoint/panel",
        ));
    }
    verify_evaluation(&value["evaluation"], checkpoint)?;
    Ok(value["evaluation"].clone())
}

#[cfg(feature = "cuda")]
fn match_arms(treatment: &Arm, control: &Arm) -> Result<()> {
    let t = &treatment.admission.plan;
    let c = &control.admission.plan;
    if !matches!(t.config.read_spec, ReadSpec::Relation { .. })
        || t.config.relation_learning_policy != training::RelationLearningPolicy::LearnV1
        || c.config.relation_learning_policy != training::RelationLearningPolicy::FrozenZeroV1
    {
        return Err(invalid(
            "transfer needs Relation treatment and matched45-coordinate off control",
        ));
    }
    let mut matched = c.config.clone();
    matched.relation_learning_policy = training::RelationLearningPolicy::LearnV1;
    let mut execution = c.execution.clone();
    execution.config_sha256 = t.execution.config_sha256.clone();
    execution.lineage_sha256 = t.execution.lineage_sha256.clone();
    if matched != t.config
        || execution != t.execution
        || c.sheet != t.sheet
        || c.transfer != t.transfer
        || treatment.progress.baseline_snapshot_metadata
            != control.progress.baseline_snapshot_metadata
        || treatment.progress.baseline_evaluation["model_snapshot_sha256"]
            != control.progress.baseline_evaluation["model_snapshot_sha256"]
    {
        return Err(invalid(
            "transfer arms differ beyond registered relation policy/identities",
        ));
    }
    require_disjoint_directories(&t.run_state_directory, &c.run_state_directory)
}

#[cfg(not(feature = "cuda"))]
pub fn run(_decision_path: &Path, _out: &Path) -> Result<()> {
    Err(invalid(
        "attention transfer requires CUDA; no fresh content read or capability issued",
    ))
}

/// One treatment-only transfer attempt after sealed paired DEV review. A FilePin
/// authenticates retained bytes, not PM authorship or discriminator correctness.
#[cfg(feature = "cuda")]
pub fn run(decision_path: &Path, out: &Path) -> Result<()> {
    let decision_bytes = fs::read(decision_path).map_err(|e| invalid(e.to_string()))?;
    let decision: TransferDecision =
        serde_json::from_slice(&decision_bytes).map_err(|e| invalid(e.to_string()))?;
    validate_decision_header(&decision)?;
    let decision_pin = FilePin {
        path: fs::canonicalize(decision_path).map_err(|e| invalid(e.to_string()))?,
        sha256: corpus::sha256(&decision_bytes),
    };
    decision.paired_discriminator.read()?;
    let compiled = crate::device::compiled_identity();
    let executable = executable_sha256()?;
    let treatment = admit_arm(&decision.treatment, &compiled, &executable)?;
    let control = admit_arm(&decision.control, &compiled, &executable)?;
    match_arms(&treatment, &control)?;
    // Both existing state locks prevent a concurrent training/resume invocation.
    // Failure to acquire either refuses; this command never waits on a peer.
    let _treatment_lock = lock_run(&treatment.release, true)?;
    let _control_lock = lock_run(&control.release, true)?;
    let treatment_dev = dev_record(
        &decision.treatment.final_dev,
        &treatment,
        &treatment.progress.checkpoint,
        "final-evaluation.json",
    )?;
    let control_dev = dev_record(
        &decision.control.final_dev,
        &control,
        &control.progress.checkpoint,
        "final-evaluation.json",
    )?;
    let state = &treatment.release.manifest.run_state_directory;
    same_path(
        &decision.treatment_ablation_checkpoint.path,
        &state.join("relation-off-checkpoint.json"),
    )?;
    let ablation: Value = pinned_json(&decision.treatment_ablation_checkpoint)?;
    let mut expected = treatment.progress.checkpoint.clone();
    let (combine, total) = gate_inventory(treatment.driver.config())?;
    let gates = expected.model["gates"]
        .as_array_mut()
        .ok_or_else(|| invalid("ablation parent gates missing"))?;
    if gates.len() != total as usize {
        return Err(invalid("ablation gate shape differs"));
    }
    for word in &mut gates[combine as usize..] {
        *word = json!(0);
    }
    expected.model_sha256 = corpus::sha256(&encode(&expected.model)?);
    if ablation
        != json!({"schema":"uor-r4.attention-relation-off/1",
        "admission_sha256":treatment.release.manifest_sha256,
        "parent_driver_checkpoint_sha256":corpus::sha256(&encode(&treatment.progress.checkpoint)?),
        "parent_model_sha256":treatment.progress.checkpoint.model_sha256,
        "modified_storage":"only relation gate trits set to zero", "checkpoint":expected})
    {
        return Err(invalid(
            "transfer ablation differs beyond relation-zero storage",
        ));
    }
    treatment.driver.validate_checkpoint(&expected)?;
    let ablation_dev = dev_record(
        &decision.treatment_ablation_dev,
        &treatment,
        &expected,
        "relation-off-evaluation.json",
    )?;
    // Reuse the exact pinned DEV row/partition and EvaluationRow count contract;
    // no Forward slot IDs or host model score are substituted.
    let rows = dev_row_identities(treatment.driver.config())?;
    for evaluation in [&treatment_dev, &control_dev, &ablation_dev] {
        evidence::summarize(evaluation, &rows)?;
    }
    if evidence::summarize(&treatment.progress.baseline_evaluation, &rows)?
        != evidence::summarize(&control.progress.baseline_evaluation, &rows)?
    {
        return Err(invalid("matched initial admission rows differ"));
    }
    require_disjoint_directories(out, state)?;
    require_disjoint_directories(out, &control.release.manifest.run_state_directory)?;
    let binding = json!({"decision":decision_pin,
        "treatment_admission_sha256":treatment.release.manifest_sha256,
        "control_admission_sha256":control.release.manifest_sha256,
        "treatment_final_checkpoint":decision.treatment.final_checkpoint,
        "transfer_source":treatment.admission.plan.transfer});
    let intent_path = state.join("fresh-transfer-intent.json");
    let result_path = state.join("fresh-transfer-evaluation.json");
    if result_path.exists() {
        let intent: Value = read_json(&intent_path)?;
        let receipt: Value = read_json(&result_path)?;
        if intent != binding
            || receipt["binding"] != binding
            || receipt["schema"] != json!("uor-r4.attention-transfer-result/1")
            || receipt["status"] != json!("TREATMENT_ONLY_FRESH_OBSERVED_PENDING_PM_DISCRIMINATOR")
            || receipt["snapshot_unchanged"] != json!(true)
            || receipt["updates_applied"] != json!(0)
            || receipt["control_fresh_evaluations"] != json!(0)
            || receipt["ablation_fresh_evaluations"] != json!(0)
        {
            return Err(invalid("completed transfer binding differs"));
        }
        verify_evaluation(&receipt["evaluation"], &treatment.progress.checkpoint)?;
        if receipt["evaluation"]["transfer_corpus"]["source"]
            != json!(treatment.admission.plan.transfer)
            || receipt["evaluation"]["transfer_is_separate_evaluation_content"] != json!(true)
        {
            return Err(invalid("completed transfer source differs"));
        }
        // This is metadata revalidation of an already-completed observation,
        // never another model evaluation. The original durable intent exists.
        let fresh = corpus::PreparedCorpus::load(
            &treatment.admission.plan.transfer,
            treatment.driver.config().spec.vocabulary,
            &treatment.driver.config().bos,
        )?;
        let rows = fresh_rows(&fresh, treatment.driver.config())?;
        if receipt["evaluation"]["transfer_corpus"] != json!(fresh.binding())
            || receipt["b3"] != json!(evidence::summarize(&receipt["evaluation"], &rows)?)
        {
            return Err(invalid("completed transfer inventory/B3 differs"));
        }
        fs::create_dir(out).map_err(|e| invalid(e.to_string()))?;
        write_new(&out.join("report.json"), &receipt)?;
        return sync_directory(out);
    }
    // Exclusive durable intent precedes fresh loading and every CUDA call. If
    // interrupted, even before evaluation, another invocation refuses replay.
    fs::create_dir(out).map_err(|e| invalid(e.to_string()))?;
    reserve_intent(state, &binding)?;
    let result = (|| -> Result<()> {
        let fresh = corpus::PreparedCorpus::load(
            &treatment.admission.plan.transfer,
            treatment.driver.config().spec.vocabulary,
            &treatment.driver.config().bos,
        )?;
        let rows = fresh_rows(&fresh, treatment.driver.config())?;
        let (model, _) = treatment
            .driver
            .restore(treatment.progress.checkpoint.clone(), 0)?;
        let evaluation = treatment.driver.evaluate_transfer(&model, &fresh)?;
        verify_evaluation(&evaluation, &treatment.progress.checkpoint)?;
        let b3 = evidence::summarize(&evaluation, &rows)?;
        if serde_json::to_value(model.snapshot()?).map_err(|e| invalid(e.to_string()))?
            != treatment.progress.checkpoint.model
        {
            return Err(invalid("fresh evaluation changed snapshot storage"));
        }
        let receipt = json!({"schema":"uor-r4.attention-transfer-result/1",
            "status":"TREATMENT_ONLY_FRESH_OBSERVED_PENDING_PM_DISCRIMINATOR",
            "binding":binding,"evaluation":evaluation,"b3":b3,"snapshot_unchanged":true,
            "updates_applied":0,"control_fresh_evaluations":0,"ablation_fresh_evaluations":0,
            "scientific_discriminator":"PENDING_PM; no automated KEEP or paired fresh-control claim",
            "training_other_scopes":"HELD","q4":"NOT_EXECUTABLE"});
        write_new(&result_path, &receipt)?;
        sync_directory(state)?;
        write_new(&out.join("report.json"), &receipt)?;
        sync_directory(out)
    })();
    if let Err(error) = &result {
        let _ = write_new(
            &out.join("failure.json"),
            &json!({
            "status":"TRANSFER_ATTEMPT_UNCERTAIN_OR_FAILED_NO_AUTOMATIC_REPLAY",
            "error":error.to_string(),"binding":binding,"mechanism_negative":false}),
        );
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    fn decision() -> TransferDecision {
        let pin = FilePin {
            path: PathBuf::from("/sealed/metadata.json"),
            sha256: "a".repeat(64),
        };
        let arm = ArmPins {
            admission: pin.clone(),
            final_checkpoint: pin.clone(),
            final_dev: pin.clone(),
        };
        TransferDecision {
            schema: "uor-r4.attention-transfer-decision/1".into(),
            verdict: "RELEASE_TREATMENT_ONLY_FRESH_AFTER_PAIRED_DEV_REVIEW".into(),
            pm_receipt_url: "https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-1"
                .into(),
            treatment: arm.clone(),
            control: arm,
            treatment_ablation_checkpoint: pin.clone(),
            treatment_ablation_dev: pin.clone(),
            paired_discriminator: pin,
        }
    }
    #[test]
    fn transfer_metadata_requires_explicit_prospective_decision_and_all_pins() -> Result<()> {
        let good = decision();
        validate_decision_header(&good)?;
        for verdict in ["H_GEOM", "APPROVE", "KEEP"] {
            let mut bad = good.clone();
            bad.verdict = verdict.into();
            assert!(validate_decision_header(&bad).is_err());
        }
        let mut bad = good.clone();
        bad.pm_receipt_url = "owner-terminal:inherited".into();
        assert!(validate_decision_header(&bad).is_err());
        let mut bad = good.clone();
        bad.pm_receipt_url =
            "https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-".into();
        assert!(validate_decision_header(&bad).is_err());
        let mut bad = good.clone();
        bad.control.final_checkpoint.sha256 = "bad".into();
        assert!(validate_decision_header(&bad).is_err());
        let mut bad = good;
        bad.paired_discriminator.path = PathBuf::from("relative.json");
        assert!(validate_decision_header(&bad).is_err());
        Ok(())
    }

    #[test]
    fn intent_is_canonical_exclusive_and_preserves_uncertain_attempt() -> Result<()> {
        let state = std::env::temp_dir().join(format!(
            "uor-attention-transfer-intent-{}-{}",
            std::process::id(),
            unix_ns()?
        ));
        fs::create_dir(&state).map_err(|e| invalid(e.to_string()))?;
        let binding = json!({"decision_sha256":"a".repeat(64),"checkpoint":16404});
        reserve_intent(&state, &binding)?;
        assert!(reserve_intent(&state, &binding).is_err());
        assert!(reserve_intent(&state, &json!({"decision_sha256":"b".repeat(64)})).is_err());
        let retained: Value = read_json(&state.join("fresh-transfer-intent.json"))?;
        assert_eq!(retained, binding);
        assert!(!state.join("fresh-transfer-evaluation.json").exists());
        fs::remove_dir_all(&state).map_err(|e| invalid(e.to_string()))?;
        Ok(())
    }

    #[test]
    fn actual_fresh_inventory_requires_both_floors_and_unique_order() -> Result<()> {
        let rows = (0..2000).map(|i| (i, 0, i < 500)).collect::<Vec<_>>();
        require_fresh_inventory(&rows)?;
        assert!(require_fresh_inventory(&rows[..1999]).is_err());
        let mut fewer_successors = rows.clone();
        fewer_successors[499].2 = false;
        assert!(require_fresh_inventory(&fewer_successors).is_err());
        let mut duplicate = rows;
        duplicate[1].0 = duplicate[0].0;
        assert!(require_fresh_inventory(&duplicate).is_err());
        Ok(())
    }
}
