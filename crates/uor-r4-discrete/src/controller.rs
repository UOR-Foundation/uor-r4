//! Operational admission and recoverable, diagnostic-only A2 pilot controller.
//! Release records are operator-supplied evidence, not cryptographic owner authentication.
//! No receipt is manufactured by this module; missing owner evidence leaves training held.
#![forbid(unsafe_code)]
use crate::{
    corpus::{self, FilePin},
    training::{self, DriverConfig, PreparedDriver},
    Error, ReadSpec, Result, Spec,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};

pub const PILOT_BATCHES: u64 = 1_640;
pub const BASE_SOURCE: &str = "e7d926fbe38537f4faa90b817705c61a780778d6";
pub const PILOT_SCOPE: &str = "m1-a2-seed1-vector0..1639-diagnostic-pilot";
fn invalid(s: impl Into<String>) -> Error {
    Error::Invalid(s.into())
}
fn encode<T: Serialize>(v: &T) -> Result<Vec<u8>> {
    serde_json::to_vec(v).map_err(|e| invalid(e.to_string()))
}
fn read_json<T: serde::de::DeserializeOwned>(p: &Path) -> Result<T> {
    serde_json::from_slice(&fs::read(p).map_err(|e| invalid(format!("{}: {e}", p.display())))?)
        .map_err(|e| invalid(e.to_string()))
}
fn write_new<T: Serialize>(p: &Path, v: &T) -> Result<()> {
    let mut f = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(p)
        .map_err(|e| invalid(format!("exclusive {}: {e}", p.display())))?;
    f.write_all(&encode(v)?)
        .and_then(|_| f.sync_all())
        .map_err(|e| invalid(e.to_string()))
}
/// Resolve existing paths and parents so symlinks cannot hide overlapping operational/output roots.
fn resolved_destination(path: &Path) -> Result<PathBuf> {
    if path.exists() {
        return fs::canonicalize(path).map_err(|e| invalid(e.to_string()));
    }
    let name = path
        .file_name()
        .ok_or_else(|| invalid("destination requires an explicit directory basename"))?;
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let parent = fs::canonicalize(parent)
        .map_err(|e| invalid(format!("destination parent {}: {e}", parent.display())))?;
    Ok(parent.join(name))
}
fn require_disjoint_directories(output: &Path, state: &Path) -> Result<()> {
    let output = resolved_destination(output)?;
    let state = resolved_destination(state)?;
    if output.starts_with(&state) || state.starts_with(&output) {
        return Err(invalid("pilot report and operational run-state directories must be disjoint, including ancestry and symlinks"));
    }
    Ok(())
}
fn sync_directory(p: &Path) -> Result<()> {
    fs::File::open(p)
        .and_then(|f| f.sync_all())
        .map_err(|e| invalid(e.to_string()))
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutionIdentity {
    pub base_source: String,
    /// Exact source commit reviewed by Claude. Embedded byte hashes below bind this executable.
    pub reviewed_source_commit: String,
    pub source_sha256: String,
    pub kernel_sha256: String,
    pub geometry_sha256: String,
    pub executable_sha256: String,
    pub driver_source_sha256: String,
    pub config_sha256: String,
    pub lineage_sha256: String,
    pub sheet_sha256: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReleaseManifest {
    pub schema: String,
    pub scope: String,
    pub execution: ExecutionIdentity,
    pub sheet: FilePin,
    pub claude_review: FilePin,
    pub owner_release: FilePin,
    /// Dedicated operational directory, outside sealed report roots; never shared between runs.
    pub run_state_directory: PathBuf,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ReviewRecord {
    reviewer: String,
    verdict: String,
    scope: String,
    execution: ExecutionIdentity,
    receipt_url: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct OwnerRecord {
    owner: String,
    decision: String,
    scope: String,
    execution: ExecutionIdentity,
    claude_review_sha256: String,
    receipt_url: String,
}
/// Its private fields prevent unvalidated issuance through safe library calls.
#[derive(Debug)]
pub struct ValidatedRelease {
    manifest: ReleaseManifest,
    manifest_sha256: String,
}
impl ValidatedRelease {
    pub(crate) fn lineage_sha256(&self) -> &str {
        &self.manifest.execution.lineage_sha256
    }
    pub(crate) fn max_batches(&self) -> u64 {
        PILOT_BATCHES
    }
    pub fn manifest_sha256(&self) -> &str {
        &self.manifest_sha256
    }
}

pub fn validate_pilot_config(config: &DriverConfig) -> Result<()> {
    config.validate()?;
    if config.spec
        != (Spec {
            vocabulary: 4096,
            blocks: 4,
            context: 192,
            kept_cells: 8,
        })
        || config.read_spec != ReadSpec::Route
        || config.seed != 1
        || config.batch_rows != 64
        || config.train.expected_payload_tokens != 7_512_797
        || config.train.expected_eligible_targets != 2_425_925
        || config.dev.expected_payload_tokens != 42_856
        || config.dev.expected_eligible_targets != 14_287
        || config.train.tokens.sha256
            != "3dee858f84b750f507ceb3dc3f393711741950954735efaaf61bf704c017a4c8"
        || config.train.mask.sha256
            != "b5e7352d67d6883a833155a5fd397a03f6d4f1b2649e3d4ec3e2ea7d95b949e3"
        || config.tokenizer.sha256
            != "d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89"
        || config.dev.tokens.sha256
            != "0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b"
        || config.dev.mask.sha256
            != "a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078"
        || config.bos.token != "<|bos|>"
        || config.bos.id != 0
    {
        return Err(invalid("only the registered A2 seed-1 ascending VectorBlock pilot is admitted; configuration/data change requires a reviewed amendment"));
    }
    Ok(())
}
fn valid_receipt_url(s: &str) -> bool {
    s.starts_with("https://github.com/UOR-Foundation/uor-r4/") && s.contains("#issuecomment-")
}
/// Pure admission metadata. `compiled` must come from the compiled CUDA identity, not a file.
fn validate_release(
    driver: &PreparedDriver,
    manifest: ReleaseManifest,
    compiled: &Value,
    executable_sha256: &str,
) -> Result<ValidatedRelease> {
    validate_release_metadata(
        driver.config(),
        &driver.lineage_sha256()?,
        manifest,
        compiled,
        executable_sha256,
    )
}
fn validate_release_metadata(
    config: &DriverConfig,
    lineage_sha256: &str,
    manifest: ReleaseManifest,
    compiled: &Value,
    executable_sha256: &str,
) -> Result<ValidatedRelease> {
    validate_pilot_config(config)?;
    let e = &manifest.execution;
    for hash in [
        &e.source_sha256,
        &e.kernel_sha256,
        &e.geometry_sha256,
        &e.executable_sha256,
        &e.driver_source_sha256,
        &e.config_sha256,
        &e.lineage_sha256,
        &e.sheet_sha256,
    ] {
        corpus::validate_sha256(hash)?;
    }
    if manifest.schema != "uor-r4.pilot-owner-admission/1"
        || manifest.scope != PILOT_SCOPE
        || e.base_source != BASE_SOURCE
        || e.reviewed_source_commit.len() != 40
        || !e
            .reviewed_source_commit
            .bytes()
            .all(|c| c.is_ascii_hexdigit())
        || !manifest.run_state_directory.is_absolute()
        || e.driver_source_sha256 != training::driver_source_sha256()
        || e.config_sha256 != corpus::sha256(&encode(config)?)
        || e.lineage_sha256 != lineage_sha256
        || e.executable_sha256 != executable_sha256
        || e.sheet_sha256 != manifest.sheet.sha256
    {
        return Err(invalid(
            "pilot admission source/executable/config/lineage/scope mismatch",
        ));
    }
    for (name, want) in [
        ("source_sha256", &e.source_sha256),
        ("kernel_sha256", &e.kernel_sha256),
        ("geometry_sha256", &e.geometry_sha256),
    ] {
        if compiled.get(name).and_then(Value::as_str) != Some(want.as_str()) {
            return Err(invalid(format!("admission does not match compiled {name}")));
        }
    }
    let sheet = manifest.sheet.read()?;
    let sheet = std::str::from_utf8(&sheet).map_err(|_| invalid("sheet is not UTF8"))?;
    if !sheet.contains("TEST FITNESS: FIT") || !sheet.contains("Pilot-reporting pins P1/P2") {
        return Err(invalid("pinned sheet lacks recorded A2 FIT/P1/P2"));
    }
    let review: ReviewRecord = serde_json::from_slice(&manifest.claude_review.read()?)
        .map_err(|e| invalid(e.to_string()))?;
    let owner: OwnerRecord = serde_json::from_slice(&manifest.owner_release.read()?)
        .map_err(|e| invalid(e.to_string()))?;
    if review.reviewer != "Claude"
        || review.verdict != "APPROVE_A2_ADMISSION_DIFF"
        || review.scope != PILOT_SCOPE
        || review.execution != *e
        || !valid_receipt_url(&review.receipt_url)
        || owner.owner != "Casey"
        || owner.decision != "RELEASE_D4_FOR_A2_PILOT_ONLY"
        || owner.scope != PILOT_SCOPE
        || owner.execution != *e
        || owner.claude_review_sha256 != manifest.claude_review.sha256
        || !valid_receipt_url(&owner.receipt_url)
    {
        return Err(Error::TrainingHeld);
    }
    Ok(ValidatedRelease {
        manifest_sha256: corpus::sha256(&encode(&manifest)?),
        manifest,
    })
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KindCounters {
    pub attempted: u64,
    pub any_candidate_cost_differs_from_identity: u64,
    pub accepted_updates: u64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PilotCheckpoint {
    schema: String,
    manifest_sha256: String,
    checkpoint: training::DriverCheckpoint,
    baseline_evaluation: Value,
    baseline_snapshot_metadata: Value,
    vector: KindCounters,
}
fn validate_resume(
    driver: &PreparedDriver,
    release: &ValidatedRelease,
    p: &PilotCheckpoint,
) -> Result<()> {
    driver.validate_checkpoint(&p.checkpoint)?;
    validate_resume_metadata(release, p)
}
fn validate_resume_metadata(release: &ValidatedRelease, p: &PilotCheckpoint) -> Result<()> {
    let c = &p.checkpoint.cursor;
    if p.schema != "uor-r4.pilot-checkpoint/1"
        || p.manifest_sha256 != release.manifest_sha256
        || c.completed_batches > PILOT_BATCHES
        || p.vector.attempted != c.completed_batches
        || p.vector.accepted_updates != c.accepted_updates
        || p.vector.any_candidate_cost_differs_from_identity > p.vector.attempted
        || p.vector.accepted_updates > p.vector.any_candidate_cost_differs_from_identity
        || p.baseline_evaluation
            .get("model_epoch")
            .and_then(Value::as_u64)
            != Some(0)
        || p.baseline_evaluation
            .get("evaluation_lineage_sha256")
            .and_then(Value::as_str)
            != Some(release.lineage_sha256())
    {
        return Err(invalid(
            "pilot resume provenance, baseline or counter mismatch",
        ));
    }
    Ok(())
}
fn validate_baseline_pair(initial: &PilotCheckpoint, resumed: &PilotCheckpoint) -> Result<()> {
    if initial.checkpoint.cursor != training::Cursor::initial()
        || initial.manifest_sha256 != resumed.manifest_sha256
        || initial.baseline_evaluation != resumed.baseline_evaluation
        || initial.baseline_snapshot_metadata != resumed.baseline_snapshot_metadata
    {
        return Err(invalid(
            "resume baseline/P2 differs from immutable checkpoint zero",
        ));
    }
    Ok(())
}
fn validate_initial_baseline(release: &ValidatedRelease, resumed: &PilotCheckpoint) -> Result<()> {
    let pin: FilePin = read_json(&release.manifest.run_state_directory.join("initial.json"))?;
    let initial: PilotCheckpoint =
        serde_json::from_slice(&pin.read()?).map_err(|e| invalid(e.to_string()))?;
    validate_baseline_pair(&initial, resumed)
}
struct RunLock {
    path: PathBuf,
}
impl Drop for RunLock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}
fn lock_run(release: &ValidatedRelease, resume: bool) -> Result<RunLock> {
    let state = &release.manifest.run_state_directory;
    if !resume {
        fs::create_dir(state).map_err(|e| {
            invalid(format!(
                "exclusive pilot run state {}: {e}; use pilot-resume for an existing run",
                state.display()
            ))
        })?;
    }
    let lock = state.join("active.lock");
    write_new(
        &lock,
        &json!({"pid":std::process::id(),"manifest_sha256":release.manifest_sha256}),
    )?;
    let guard = RunLock { path: lock };
    let registration = state.join("registration.json");
    if resume {
        let previous: Value = read_json(&registration)?;
        if previous.get("manifest_sha256").and_then(Value::as_str)
            != Some(release.manifest_sha256.as_str())
        {
            return Err(invalid("resume manifest differs from original pilot"));
        }
    } else {
        write_new(
            &registration,
            &json!({"manifest_sha256":release.manifest_sha256,"scope":PILOT_SCOPE}),
        )?;
    }
    Ok(guard)
}
#[cfg(feature = "cuda")]
fn checkpoint_file(
    driver: &PreparedDriver,
    release: &ValidatedRelease,
    out: &Path,
    p: &PilotCheckpoint,
) -> Result<()> {
    validate_resume(driver, release, p)?;
    let name = format!(
        "checkpoint-{:04}.json",
        p.checkpoint.cursor.completed_batches
    );
    let path = out.join(name);
    write_new(&path, p)?;
    sync_directory(out)?;
    let pin = FilePin {
        path: fs::canonicalize(&path).map_err(|e| invalid(e.to_string()))?,
        sha256: corpus::sha256(&fs::read(&path).map_err(|e| invalid(e.to_string()))?),
    };
    let state = &release.manifest.run_state_directory;
    if p.checkpoint.cursor.completed_batches == 0 && !state.join("initial.json").exists() {
        write_new(&state.join("initial.json"), &pin)?;
        sync_directory(state)?;
    }
    // Only this operational pointer is replaced. All checkpoint/report files are immutable.
    let temporary = state.join(format!("latest-{}.pending", std::process::id()));
    write_new(&temporary, &pin)?;
    fs::rename(&temporary, state.join("latest.json")).map_err(|e| invalid(e.to_string()))?;
    sync_directory(state)?;
    sync_directory(out)
}

#[cfg(feature = "cuda")]
pub fn run_pilot(
    config: DriverConfig,
    manifest: ReleaseManifest,
    out: &Path,
    resume: bool,
) -> Result<()> {
    let attempt_started = std::time::Instant::now();
    validate_pilot_config(&config)?;
    let driver = PreparedDriver::load(config)?;
    let executable = std::env::current_exe().map_err(|e| invalid(e.to_string()))?;
    let executable_sha256 =
        corpus::sha256(&fs::read(executable).map_err(|e| invalid(e.to_string()))?);
    let release = validate_release(
        &driver,
        manifest,
        &crate::device::compiled_identity(),
        &executable_sha256,
    )?;
    require_disjoint_directories(out, &release.manifest.run_state_directory)?;
    // Claim output and operational lease before CUDA initialization. No reuse/append to an old report.
    fs::create_dir(out).map_err(|e| invalid(format!("exclusive pilot output: {e}")))?;
    let _lock = lock_run(&release, resume)?;
    write_new(
        &out.join("admission.json"),
        &json!({"manifest":release.manifest,"manifest_sha256":release.manifest_sha256,"global_training_hold":"HELD_EXCEPT_EXPLICIT_RECORDED_PILOT_RELEASE","m2":"HELD","authorization":"operator-supplied pinned receipts; not cryptographic authentication"}),
    )?;
    let result = (|| -> Result<()> {
        let (mut model, mut progress) = if resume {
            let pin: FilePin =
                read_json(&release.manifest.run_state_directory.join("latest.json"))?;
            let progress: PilotCheckpoint =
                serde_json::from_slice(&pin.read()?).map_err(|e| invalid(e.to_string()))?;
            validate_resume(&driver, &release, &progress)?;
            validate_initial_baseline(&release, &progress)?;
            write_new(&out.join("resume-parent.json"), &pin)?;
            let (model, _) = driver.restore(progress.checkpoint.clone(), 0)?;
            (model, progress)
        } else {
            let model = driver.new_model(0)?;
            let checkpoint = driver.checkpoint(&model, training::Cursor::initial())?;
            let metadata = training::snapshot_metadata(&checkpoint.model)?;
            let evaluation = driver.evaluate(&model)?;
            let progress = PilotCheckpoint {
                schema: "uor-r4.pilot-checkpoint/1".into(),
                manifest_sha256: release.manifest_sha256.clone(),
                checkpoint,
                baseline_evaluation: evaluation,
                baseline_snapshot_metadata: metadata,
                vector: KindCounters::default(),
            };
            (model, progress)
        };
        // This durable baseline checkpoint exists before admission issuance and before first apply.
        checkpoint_file(&driver, &release, out, &progress)?;
        let admission = training::issue_corpus_update_admission(&driver, &release)?;
        let mut receipts = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(out.join("steps.jsonl"))
            .map_err(|e| invalid(e.to_string()))?;
        while progress.checkpoint.cursor.completed_batches < PILOT_BATCHES {
            let receipt = driver.transition(&mut model, &progress.checkpoint.cursor, &admission)?;
            receipts
                .write_all(&encode(&receipt)?)
                .and_then(|_| receipts.write_all(b"\n"))
                .and_then(|_| receipts.sync_all())
                .map_err(|e| invalid(e.to_string()))?;
            progress.vector.attempted += 1;
            progress.vector.any_candidate_cost_differs_from_identity +=
                u64::from(receipt.any_candidate_cost_differs_from_identity);
            progress.vector.accepted_updates += u64::from(receipt.applied);
            progress.checkpoint.cursor = receipt.cursor_after;
            // Immutable recovery snapshots every 64 transitions; uninterrupted end is always saved.
            // A failed segment resumes its previous exact snapshot, never the mutable device state.
            if receipt.cursor_after.completed_batches % 64 == 0
                || receipt.cursor_after.completed_batches == PILOT_BATCHES
            {
                progress.checkpoint = driver.checkpoint(&model, receipt.cursor_after)?;
                checkpoint_file(&driver, &release, out, &progress)?;
            }
        }
        let final_evaluation = driver.evaluate(&model)?;
        let final_metadata = training::snapshot_metadata(&progress.checkpoint.model)?;
        write_new(
            &out.join("report.json"),
            &json!({"schema":"uor-r4.a2-pilot-report/1","status":"COMPLETE_DIAGNOSTIC_ONLY","attempt_elapsed_ns":attempt_started.elapsed().as_nanos().to_string(),"attempt_resume":resume,"timing_scope":"current controller invocation only, includes admission and I/O; earlier failed/resumed attempts remain separately charged","scope":PILOT_SCOPE,"completed_batches":PILOT_BATCHES,"seed":1,"eligible_train_target_exposures":104960,"nominal_candidate_slots":26240,"bars":"NOT_DECIDED_PILOT","guard":"NOT_DECIDED_PILOT","mechanism_verdict":"NOT_ASSIGNED","fresh_transfer":"UNAVAILABLE_NOT_RUN","full_run":"NOT_AUTHORIZED_BY_THIS_PILOT","m2":"HELD","manifest_sha256":release.manifest_sha256,"baseline_evaluation":progress.baseline_evaluation,"final_evaluation":final_evaluation,"sensitivity":{"vector_block":progress.vector,"route":{"attempted":0,"any_candidate_cost_differs_from_identity":"NOT_RUN","accepted_updates":0}},"P1":"pilot_slice_bias in baseline_evaluation and final_evaluation; ascending token IDs 0..409 only; no frequency reordering","P2":{"checkpoint_0":progress.baseline_snapshot_metadata,"pilot_end":final_metadata},"interpretation":"insensitivity is a dose/schedule finding, never a configuration negative; collision-free integer init is a named successor requiring amendment"}),
        )?;
        sync_directory(out)
    })();
    if let Err(error) = &result {
        write_new(
            &out.join("failure.json"),
            &json!({"status":"INSTRUMENT_OR_EXECUTION_FAILURE","attempt_elapsed_ns":attempt_started.elapsed().as_nanos().to_string(),"attempt_resume":resume,"error":error.to_string(),"mechanism_negative":false,"resume":"latest durable checkpoint only; unapersisted segment is replayed","m2":"HELD"}),
        )?;
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    fn config() -> Result<DriverConfig> {
        let mut config: DriverConfig = serde_json::from_str(include_str!(
            "../../../docs/labs/discrete-driver-cost-2026-10-13/driver-config.json"
        ))
        .map_err(|e| invalid(e.to_string()))?;
        config.seed = 1;
        Ok(config)
    }
    fn directory(name: &str) -> Result<PathBuf> {
        let root =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../../local/controller-metadata-tests");
        fs::create_dir_all(&root).map_err(|e| invalid(e.to_string()))?;
        let path = root.join(format!(
            "{name}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|e| invalid(e.to_string()))?
                .as_nanos()
        ));
        fs::create_dir(&path).map_err(|e| invalid(e.to_string()))?;
        Ok(path)
    }
    fn pin(path: PathBuf, bytes: &[u8]) -> Result<FilePin> {
        fs::write(&path, bytes).map_err(|e| invalid(e.to_string()))?;
        Ok(FilePin {
            path,
            sha256: corpus::sha256(bytes),
        })
    }
    fn manifest(config: &DriverConfig, root: &Path) -> Result<ReleaseManifest> {
        let sheet = pin(
            root.join("sheet.md"),
            b"TEST FITNESS: FIT; Pilot-reporting pins P1/P2",
        )?;
        let execution = ExecutionIdentity {
            base_source: BASE_SOURCE.into(),
            reviewed_source_commit: "a".repeat(40),
            source_sha256: "1".repeat(64),
            kernel_sha256: "2".repeat(64),
            geometry_sha256: "3".repeat(64),
            executable_sha256: "4".repeat(64),
            driver_source_sha256: training::driver_source_sha256(),
            config_sha256: corpus::sha256(&encode(config)?),
            lineage_sha256: "5".repeat(64),
            sheet_sha256: sheet.sha256.clone(),
        };
        let review = ReviewRecord {
            reviewer: "Claude".into(),
            verdict: "APPROVE_A2_ADMISSION_DIFF".into(),
            scope: PILOT_SCOPE.into(),
            execution: execution.clone(),
            receipt_url: "https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-1"
                .into(),
        };
        let claude_review = pin(root.join("review.json"), &encode(&review)?)?;
        let owner = OwnerRecord {
            owner: "Casey".into(),
            decision: "RELEASE_D4_FOR_A2_PILOT_ONLY".into(),
            scope: PILOT_SCOPE.into(),
            execution: execution.clone(),
            claude_review_sha256: claude_review.sha256.clone(),
            receipt_url: "https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-2"
                .into(),
        };
        let owner_release = pin(root.join("owner.json"), &encode(&owner)?)?;
        Ok(ReleaseManifest {
            schema: "uor-r4.pilot-owner-admission/1".into(),
            scope: PILOT_SCOPE.into(),
            execution,
            sheet,
            claude_review,
            owner_release,
            run_state_directory: root.join("run"),
        })
    }
    fn admit(config: &DriverConfig, m: ReleaseManifest) -> Result<ValidatedRelease> {
        validate_release_metadata(
            config,
            &"5".repeat(64),
            m,
            &json!({"source_sha256":"1".repeat(64),"kernel_sha256":"2".repeat(64),"geometry_sha256":"3".repeat(64)}),
            &"4".repeat(64),
        )
    }
    #[test]
    fn controller_is_pilot_only_and_unknown_release_fields_refuse() {
        assert_eq!(PILOT_BATCHES, 1640);
        assert!(serde_json::from_str::<ReleaseManifest>(r#"{"released":true}"#).is_err());
        assert!(!valid_receipt_url("https://example.org/#issuecomment-1"));
    }
    #[test]
    fn pilot_configuration_rejects_unregistered_seed_dose_or_data() -> Result<()> {
        let c = config()?;
        validate_pilot_config(&c)?;
        for field in ["seed", "batch", "blocks", "context", "train", "dev", "bos"] {
            let mut wrong = c.clone();
            match field {
                "seed" => wrong.seed = 2,
                "batch" => wrong.batch_rows = 32,
                "blocks" => wrong.spec.blocks = 2,
                "context" => wrong.spec.context = 27,
                "train" => wrong.train.mask.sha256 = "a".repeat(64),
                "dev" => wrong.dev.tokens.sha256 = "a".repeat(64),
                _ => wrong.bos.token = "other".into(),
            };
            assert!(validate_pilot_config(&wrong).is_err(), "{field}");
        }
        Ok(())
    }
    #[test]
    fn release_records_bind_owner_review_scope_and_every_identity() -> Result<()> {
        let root = directory("release")?;
        let c = config()?;
        let m = manifest(&c, &root)?;
        assert!(admit(&c, m.clone()).is_ok());
        for field in [
            "scope",
            "base_source",
            "reviewed_source_commit",
            "source_sha256",
            "kernel_sha256",
            "geometry_sha256",
            "executable_sha256",
            "driver_source_sha256",
            "config_sha256",
            "lineage_sha256",
            "sheet_sha256",
        ] {
            let mut wire = serde_json::to_value(&m).map_err(|e| invalid(e.to_string()))?;
            if field == "scope" {
                wire[field] = json!("full-run");
            } else {
                wire["execution"][field] = json!("b".repeat(64));
            }
            let wrong: ReleaseManifest =
                serde_json::from_value(wire).map_err(|e| invalid(e.to_string()))?;
            assert!(admit(&c, wrong).is_err(), "{field}");
        }
        let mut owner: OwnerRecord =
            serde_json::from_slice(&m.owner_release.read()?).map_err(|e| invalid(e.to_string()))?;
        owner.decision = "HELD".into();
        let mut held = m.clone();
        held.owner_release = pin(root.join("held.json"), &encode(&owner)?)?;
        assert!(matches!(admit(&c, held), Err(Error::TrainingHeld)));
        let mut missing = m;
        missing.owner_release.path = root.join("missing.json");
        assert!(admit(&c, missing).is_err());
        fs::remove_dir_all(root).map_err(|e| invalid(e.to_string()))?;
        Ok(())
    }
    #[test]
    fn resume_counters_require_exact_pilot_cursor_and_baseline() -> Result<()> {
        let root = directory("resume")?;
        let c = config()?;
        let release = admit(&c, manifest(&c, &root)?)?;
        let binding = |source: crate::corpus::CorpusSource| crate::corpus::CorpusBinding {
            source,
            vocabulary: 4096,
            bos: c.bos.clone(),
            payload_tokens: 0,
            mask_bytes: 0,
            mask_selected_positions: 0,
            eligible_targets: 0,
            documents: 0,
            window_policy: crate::corpus::WINDOW_POLICY.into(),
            identity_policy: "metadata fixture only".into(),
        };
        let lineage = training::Lineage {
            schema: training::DRIVER_SCHEMA.into(),
            driver_source_sha256: training::driver_source_sha256(),
            config: c.clone(),
            train: binding(c.train.clone()),
            dev: binding(c.dev.clone()),
        };
        let p = PilotCheckpoint {
            schema: "uor-r4.pilot-checkpoint/1".into(),
            manifest_sha256: release.manifest_sha256.clone(),
            checkpoint: training::DriverCheckpoint {
                schema: training::CHECKPOINT_SCHEMA.into(),
                lineage,
                cursor: training::Cursor::initial(),
                model_sha256: "0".repeat(64),
                model: json!({}),
            },
            baseline_evaluation: json!({"model_epoch":0,"evaluation_lineage_sha256":"5".repeat(64)}),
            baseline_snapshot_metadata: json!({}),
            vector: KindCounters::default(),
        };
        validate_resume_metadata(&release, &p)?;
        validate_baseline_pair(&p, &p)?;
        let mut replaced = p.clone();
        replaced.baseline_evaluation["aggregate"] = json!({"different":true});
        assert!(validate_baseline_pair(&p, &replaced).is_err());
        replaced = p.clone();
        replaced.baseline_snapshot_metadata = json!({"different":true});
        assert!(validate_baseline_pair(&p, &replaced).is_err());
        for case in 0..5 {
            let mut wrong = p.clone();
            match case {
                0 => wrong.checkpoint.cursor.completed_batches = 1641,
                1 => wrong.vector.attempted = 1,
                2 => wrong.baseline_evaluation["model_epoch"] = json!(1),
                3 => wrong.baseline_evaluation["evaluation_lineage_sha256"] = json!("other"),
                _ => wrong.vector.accepted_updates = 1,
            };
            assert!(validate_resume_metadata(&release, &wrong).is_err());
        }
        fs::remove_dir_all(root).map_err(|e| invalid(e.to_string()))?;
        Ok(())
    }
    #[test]
    fn immutable_checkpoint_output_refuses_reuse() -> Result<()> {
        let root = directory("exclusive")?;
        let p = root.join("checkpoint.json");
        write_new(&p, &json!({"generation":1}))?;
        assert!(write_new(&p, &json!({"generation":2})).is_err());
        assert_eq!(read_json::<Value>(&p)?["generation"], json!(1));
        let output = root.join("report");
        let state = root.join("state");
        require_disjoint_directories(&output, &state)?;
        fs::create_dir(&output).map_err(|e| invalid(e.to_string()))?;
        assert!(require_disjoint_directories(&output, &output).is_err());
        assert!(require_disjoint_directories(&output, &output.join("state")).is_err());
        assert!(require_disjoint_directories(&output, &root).is_err());
        #[cfg(unix)]
        {
            let alias = root.join("alias");
            std::os::unix::fs::symlink(&output, &alias).map_err(|e| invalid(e.to_string()))?;
            assert!(require_disjoint_directories(&output, &alias.join("state")).is_err());
        }
        fs::remove_dir_all(root).map_err(|e| invalid(e.to_string()))?;
        Ok(())
    }
}

/// Evaluate only an admitted, registered pilot checkpoint. Never mutates model parameters.
#[cfg(feature = "cuda")]
pub fn evaluate_pilot(config: DriverConfig, manifest: ReleaseManifest, out: &Path) -> Result<()> {
    validate_pilot_config(&config)?;
    let driver = PreparedDriver::load(config)?;
    let executable = std::env::current_exe().map_err(|e| invalid(e.to_string()))?;
    let executable_sha256 =
        corpus::sha256(&fs::read(executable).map_err(|e| invalid(e.to_string()))?);
    let release = validate_release(
        &driver,
        manifest,
        &crate::device::compiled_identity(),
        &executable_sha256,
    )?;
    require_disjoint_directories(out, &release.manifest.run_state_directory)?;
    fs::create_dir(out).map_err(|e| invalid(format!("exclusive evaluation output: {e}")))?;
    let _lock = lock_run(&release, true)?;
    let pin: FilePin = read_json(&release.manifest.run_state_directory.join("latest.json"))?;
    let progress: PilotCheckpoint =
        serde_json::from_slice(&pin.read()?).map_err(|e| invalid(e.to_string()))?;
    validate_resume(&driver, &release, &progress)?;
    validate_initial_baseline(&release, &progress)?;
    write_new(
        &out.join("admission.json"),
        &json!({"manifest":release.manifest,"manifest_sha256":release.manifest_sha256,"checkpoint":pin,"purpose":"evaluation only"}),
    )?;
    let (model, _cursor) = driver.restore(progress.checkpoint, 0)?;
    write_new(&out.join("report.json"), &driver.evaluate(&model)?)?;
    sync_directory(out)
}
