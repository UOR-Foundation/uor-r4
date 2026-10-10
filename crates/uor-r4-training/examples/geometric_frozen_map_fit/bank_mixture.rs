//! Registered original512 + independently sealed expanded-bank512 learning.
//! No serving change and no fresh qualification in this training run.
use super::*;

const DIAGNOSTIC_INPUT_SHA: &str =
    "c5a643ff9ef86f5f16f15b420f42fe8b1921c5c8c7df5a7bbb88f7759cda5bda";
const DIAGNOSTIC_LABEL_SHA: &str =
    "b70f57761238ad4044ebdc158e465e978cfd098c9dfbe5b2b4d4a79e582aec77";
const DIAGNOSTIC_MANIFEST_SHA: &str =
    "f8cbb66a683df75fad6c13bdc734fd6dad70b04dfa7932e9119992f61f0a3e1d";
const BASELINE_REPORT_SHA: &str =
    "0bbd3f09b6e05dbd25697eb0b38ff727ac84c98d629a05016d682f965bc4108f";
const BASELINE_MANIFEST_SHA: &str =
    "4c20a585073c85dadf2d5152bf6d8ffaa8c74c9b089ee9a689333c0253635bcd";
const FIELD437_SHA: &str = "de4a3234d6e92f4b12a657cc195425687bc67c243d9eded9a1df0797b9942a61";
pub(super) const POLICY: &str = "original512+expanded-concrete-donor512;4+4-per-batch;independent-seed1001-permutations;two-passes-each;256updates;opened128-diagnostic-not-fresh/1";

#[derive(Clone, Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Config {
    pub recomposition_root: PathBuf,
    pub expected_recomposition_manifest_sha256: String,
    pub expected_recomposition_inputs_sha256: String,
    pub expected_recomposition_labels_sha256: String,
    pub expected_recomposition_target_positions: usize,
    pub diagnostic_root: PathBuf,
    pub diagnostic_baseline_root: PathBuf,
    pub preparation_audit: PathBuf,
    pub expected_preparation_audit_sha256: String,
}
pub(super) struct Data {
    pub additional: Vec<Episode>,
    pub diagnostic: Vec<Episode>,
    pub receipt: Value,
    baseline: Vec<Value>,
}

pub(super) fn settings(a: &Args) -> Result<()> {
    let Some(c) = &a.cross_state_bank_mixture else {
        return Ok(());
    };
    if a.mode != Mode::CrossStateContinuation
        || !a.cross_state_pooled_rank
        || !a.cross_state_bottleneck
        || a.cross_state_resume.is_none()
        || !(512..=512 * 32).contains(&c.expected_recomposition_target_positions)
        || [
            &c.expected_recomposition_manifest_sha256,
            &c.expected_recomposition_inputs_sha256,
            &c.expected_recomposition_labels_sha256,
            &c.expected_preparation_audit_sha256,
        ]
        .iter()
        .any(|s| s.len() != 64 || !s.bytes().all(|b| b.is_ascii_hexdigit()))
        || c.recomposition_root == c.diagnostic_root
        || c.recomposition_root == c.diagnostic_baseline_root
        || c.diagnostic_root == c.diagnostic_baseline_root
    {
        return Err(bad(
            "bank mixture requires strict ranked saved437 mode and separate pinned datasets",
        ));
    }
    Ok(())
}
fn bound(root: &Path, manifest: &str) -> Result<()> {
    report_output::verify(root)?;
    if sha256_file(&root.join("manifest.json"))? != manifest {
        return Err(bad("bank mixture sealed dataset identity differs"));
    }
    Ok(())
}
fn input_identity(root: &Path, input: &str, labels: &str) -> Result<()> {
    if sha256_file(&root.join("inputs.json"))? != input
        || sha256_file(&root.join("labels.json"))? != labels
    {
        return Err(bad("bank mixture input/label identity differs"));
    }
    Ok(())
}
// This is an independently reconstructed prelaunch audit, not an assertion
// inferred from packet hashes. Its exact bytes are published and config-bound
// before fitting; the audit authenticates the sealed preparation chain and
// semantic whole-bank exclusions, including alternate query/chronology forms.
fn preparation_audit(c: &Config) -> Result<Value> {
    if sha256_file(&c.preparation_audit)? != c.expected_preparation_audit_sha256 {
        return Err(bad("bank mixture preparation audit identity differs"));
    }
    let audit = read(&c.preparation_audit)?;
    if audit["schema"] != "uor-r4.bank-transfer-preparation-audit/1"
        || audit["status"] != "PASS"
        || audit["recomposition"]["manifest_sha256"] != c.expected_recomposition_manifest_sha256
        || audit["recomposition"]["inputs_sha256"] != c.expected_recomposition_inputs_sha256
        || audit["recomposition"]["labels_sha256"] != c.expected_recomposition_labels_sha256
        || audit["recomposition"]["target_positions"] != c.expected_recomposition_target_positions
        || audit["recomposition"]["rows"] != 512
        || audit["original_inputs_sha256"] != INPUT_SHA
        || audit["original_labels_sha256"] != LABEL_SHA
        || audit["diagnostic_inputs_sha256"] != DIAGNOSTIC_INPUT_SHA
        || audit["diagnostic_labels_sha256"] != DIAGNOSTIC_LABEL_SHA
        || audit["concrete_donor_roots"] != 12
        || audit["exposed_whole_bank_overlap"] != 0
        || audit["fresh_rows_authored"] != 0
        || [
            "sealed_preparation_chain",
            "exact_concrete_wire_origins",
            "whole_bank_exclusion",
            "balanced512_quotas",
            "typed_answer_membership",
            "development_only_no_fresh",
        ]
        .iter()
        .any(|key| audit["checks"][*key] != true)
    {
        return Err(bad(
            "bank mixture independently audited preparation differs",
        ));
    }
    Ok(audit)
}
fn packet_set(episodes: &[Episode]) -> Result<BTreeSet<String>> {
    episodes
        .iter()
        .map(|e| {
            let mut value = serde_json::to_value(&e.packet)?;
            value
                .as_object_mut()
                .ok_or_else(|| bad("packet object absent"))?
                .remove("id");
            Ok(sha256_bytes(&serde_json::to_vec(&value)?))
        })
        .collect()
}
pub(super) fn load(
    a: &Args,
    p: &ContinuationParent,
    legal: &BTreeSet<u32>,
    original: &[Episode],
) -> Result<Option<Data>> {
    settings(a)?;
    let Some(c) = &a.cross_state_bank_mixture else {
        return Ok(None);
    };
    bound(
        &c.recomposition_root,
        &c.expected_recomposition_manifest_sha256,
    )?;
    input_identity(
        &c.recomposition_root,
        &c.expected_recomposition_inputs_sha256,
        &c.expected_recomposition_labels_sha256,
    )?;
    let preparation_audit = preparation_audit(c)?;
    let report = read(&c.recomposition_root.join("report.json"))?;
    if report["schema"] != "uor-r4.bank-generate-panel/1"
        || report["status"] != "COMPLETED"
        || report["cases"] != 512
        || report["learned_prose_labels_only"] != true
        || report["input_bytes_sha256"] != c.expected_recomposition_inputs_sha256
        || report["output_inputs_sha256"] != c.expected_recomposition_inputs_sha256
    {
        return Err(bad(
            "bank mixture requires complete sealed Rust prose preparation",
        ));
    }
    let additional = load_panel(
        &c.recomposition_root.join("inputs.json"),
        &c.recomposition_root.join("labels.json"),
        &p.integer,
        &p.tokenizer,
        legal,
        512,
    )?;
    if original.len() != 512
        || additional.len() != 512
        || original.iter().map(|e| e.target.len()).sum::<usize>() != 6664
        || additional.iter().map(|e| e.target.len()).sum::<usize>()
            != c.expected_recomposition_target_positions
    {
        return Err(bad("bank mixture exact partition/target coverage differs"));
    }
    pairs(&additional)?;
    bound(&c.diagnostic_root, DIAGNOSTIC_MANIFEST_SHA)?;
    input_identity(
        &c.diagnostic_root,
        DIAGNOSTIC_INPUT_SHA,
        DIAGNOSTIC_LABEL_SHA,
    )?;
    let diagnostic = load_panel(
        &c.diagnostic_root.join("inputs.json"),
        &c.diagnostic_root.join("labels.json"),
        &p.integer,
        &p.tokenizer,
        legal,
        128,
    )?;
    if diagnostic.len() != 128 || scoped_pairs(&diagnostic)?.len() != 64 {
        return Err(bad("bank mixture opened diagnostic coverage differs"));
    }
    let mut ids = BTreeSet::new();
    if original
        .iter()
        .chain(&additional)
        .chain(&diagnostic)
        .any(|e| !ids.insert(e.packet.id.as_str()))
    {
        return Err(bad(
            "bank mixture IDs must be unique across all three partitions",
        ));
    }
    let new_packets = packet_set(&additional)?;
    if new_packets.len() != 512
        || !new_packets.is_disjoint(&packet_set(original)?)
        || !new_packets.is_disjoint(&packet_set(&diagnostic)?)
    {
        return Err(bad(
            "bank mixture new packets overlap exposed original or diagnostic",
        ));
    }
    bound(&c.diagnostic_baseline_root, BASELINE_MANIFEST_SHA)?;
    if sha256_file(&c.diagnostic_baseline_root.join("report.json"))? != BASELINE_REPORT_SHA {
        return Err(bad("bank mixture saved diagnostic report differs"));
    }
    let old = read(&c.diagnostic_baseline_root.join("report.json"))?;
    let refs = old["rows"]
        .as_array()
        .filter(|r| r.len() == 128)
        .ok_or_else(|| bad("saved diagnostic128 absent"))?;
    if old["complete"] != 0 || old["continuation_sha256"] != FIELD437_SHA {
        return Err(bad("saved diagnostic must bind zero-of128 and saved437"));
    }
    let mut baseline = Vec::new();
    for (index, r) in refs.iter().enumerate() {
        let filename = format!("row-{index:04}.json");
        if r["row_file"] != filename {
            return Err(bad("saved diagnostic row path differs"));
        }
        let path = c.diagnostic_baseline_root.join(filename);
        if r["row_sha256"] != sha256_file(&path)? {
            return Err(bad("saved diagnostic row hash differs"));
        }
        baseline.push(read(&path)?);
    }
    let receipt = json!({"policy":POLICY,"config":c,"preparation_audit":preparation_audit,"original_inputs_sha256":INPUT_SHA,"original_labels_sha256":LABEL_SHA,
        "original_rows":512,"recomposition_rows":512,"training_rows":1024,"original_target_positions":6664,
        "recomposition_target_positions":c.expected_recomposition_target_positions,
        "diagnostic_inputs_sha256":DIAGNOSTIC_INPUT_SHA,"diagnostic_labels_sha256":DIAGNOSTIC_LABEL_SHA,
        "diagnostic_report_sha256":BASELINE_REPORT_SHA,"diagnostic_manifest_sha256":BASELINE_MANIFEST_SHA,
        "diagnostic_scope":"opened #2164 panel; no training bank overlap; not fresh qualification"});
    Ok(Some(Data {
        additional,
        diagnostic,
        receipt,
        baseline,
    }))
}

pub(super) fn schedule(seed: u64) -> Vec<usize> {
    order(seed, 512)
}
pub(super) fn indices(schedule: &[usize], update: usize) -> Result<Vec<usize>> {
    if schedule.len() != 512
        || update >= 256
        || schedule.iter().copied().collect::<BTreeSet<_>>() != (0..512).collect()
    {
        return Err(bad(
            "bank mixture schedule must be a512 permutation and local update<256",
        ));
    }
    let mut result = Vec::with_capacity(8);
    for offset in [0, 512] {
        result.extend((0..4).map(|j| offset + schedule[(update * 4 + j) % 512]));
    }
    Ok(result)
}
pub(super) fn baseline(data: &Data, current: &Value) -> Result<()> {
    let rows = current["rows"]
        .as_array()
        .filter(|r| r.len() == 128)
        .ok_or_else(|| bad("diagnostic baseline coverage"))?;
    if current["complete"] != 0 || current["continuation_sha256"] != FIELD437_SHA {
        return Err(bad("diagnostic baseline count/field differs"));
    }
    for (old, row) in data.baseline.iter().zip(rows) {
        for key in ["id", "generated_ids", "eos", "complete"] {
            if old[key] != row[key] {
                return Err(bad("diagnostic baseline exact saved128 output differs"));
            }
        }
    }
    Ok(())
}
pub(super) fn evaluate(
    a: &Args,
    name: &str,
    p: &ContinuationParent,
    field: &NativeContinuationField,
    data: &Data,
    start: Instant,
) -> Result<Value> {
    let mut result = continuation_evaluate_rows_impl(
        a,
        name,
        p,
        field,
        &data.diagnostic.iter().collect::<Vec<_>>(),
        start,
        false,
    )?;
    result["scope"] = json!("all128 opened #2164 diagnostic rows; own-prefix only; canonical metrics NOT_RUN; not fresh qualification");
    result["membership_grader"] = json!("unchanged native EOS and frozen UTF8 answer membership");
    write(a, &format!("{name}.json"), &result)?;
    Ok(result)
}
fn count(value: &Value, cases: usize) -> Result<usize> {
    let rows = value["rows"]
        .as_array()
        .filter(|r| r.len() == cases)
        .ok_or_else(|| bad("mixture outcome row coverage"))?;
    let mut seen = BTreeSet::new();
    let mut complete = 0;
    for row in rows {
        let id = row["id"]
            .as_str()
            .ok_or_else(|| bad("mixture outcome ID absent"))?;
        if !seen.insert(id) {
            return Err(bad("mixture outcome duplicate ID"));
        }
        complete += usize::from(
            row["complete"]
                .as_bool()
                .ok_or_else(|| bad("mixture outcome complete flag absent"))?,
        );
    }
    if value["complete"] != complete {
        return Err(bad("mixture outcome count mismatch"));
    }
    Ok(complete)
}
pub(super) fn outcome(dev: &Value, diagnostic: &Value) -> Result<Value> {
    let d = count(dev, 512)?;
    let t = count(diagnostic, 128)?;
    let candidate = d >= 256 && t >= 52;
    Ok(
        json!({"policy":POLICY,"development_complete":d,"opened_diagnostic_complete":t,
        "qualification_candidate":candidate,"decision":if candidate {"KEEP_QUALIFICATION_CANDIDATE_NOT_M2_ACCEPTANCE"} else {"REJECT_MIXTURE_RECIPE"},
        "best_development_complete":437usize.max(d),"development_headline_improved":d>437,
        "count_after":if d>437 {0} else {2},"bar":"development>=256/512 AND opened diagnostic>=52/128; actual fresh NOT_RUN",
        "fresh_qualification":"NOT_RUN; freeze candidate then separate prospective unchanged128 draw",
        "parent":"saved437 retained even if this candidate has lower development score",
        "next":if candidate {"after protected delivery/cleanup freeze candidate and preregister truly fresh128"} else {"stop this mixture recipe; choose distinct model-changing successor; no unchanged dose/rate/seed repeat"}}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    struct OwnedRoot(PathBuf);
    impl Drop for OwnedRoot {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    fn owned_root() -> Result<OwnedRoot> {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos();
        fs::create_dir_all("local")?;
        let path = PathBuf::from("local").join(format!(
            "bank-mixture-boundary-test-{}-{nonce}",
            std::process::id()
        ));
        fs::create_dir(&path)?;
        Ok(OwnedRoot(path))
    }

    #[test]
    fn preparation_audit_binds_actual_bytes_and_semantic_exclusion_receipt() -> Result<()> {
        let owned = owned_root()?;
        let path = owned.0.join("preparation-audit.json");
        let mut config = Config {
            recomposition_root: owned.0.join("new512"),
            expected_recomposition_manifest_sha256: "a".repeat(64),
            expected_recomposition_inputs_sha256: "b".repeat(64),
            expected_recomposition_labels_sha256: "c".repeat(64),
            expected_recomposition_target_positions: 7000,
            diagnostic_root: owned.0.join("opened128"),
            diagnostic_baseline_root: owned.0.join("saved-diagnostic"),
            preparation_audit: path.clone(),
            expected_preparation_audit_sha256: String::new(),
        };
        let audit = json!({"schema":"uor-r4.bank-transfer-preparation-audit/1","status":"PASS",
            "recomposition":{"manifest_sha256":config.expected_recomposition_manifest_sha256,
                "inputs_sha256":config.expected_recomposition_inputs_sha256,
                "labels_sha256":config.expected_recomposition_labels_sha256,"rows":512,"target_positions":7000},
            "original_inputs_sha256":INPUT_SHA,"original_labels_sha256":LABEL_SHA,
            "diagnostic_inputs_sha256":DIAGNOSTIC_INPUT_SHA,"diagnostic_labels_sha256":DIAGNOSTIC_LABEL_SHA,
            "concrete_donor_roots":12,"exposed_whole_bank_overlap":0,"fresh_rows_authored":0,
            "checks":{"sealed_preparation_chain":true,"exact_concrete_wire_origins":true,
                "whole_bank_exclusion":true,"balanced512_quotas":true,
                "typed_answer_membership":true,"development_only_no_fresh":true}});
        let bytes = serde_json::to_vec(&audit)?;
        fs::write(&path, &bytes)?;
        config.expected_preparation_audit_sha256 = sha256_bytes(&bytes);
        assert_eq!(preparation_audit(&config)?, audit);
        let mut wrong_sha = config.clone();
        wrong_sha.expected_preparation_audit_sha256 = "0".repeat(64);
        assert!(preparation_audit(&wrong_sha).is_err());
        let mut changed_data = config.clone();
        changed_data.expected_recomposition_inputs_sha256 = "e".repeat(64);
        assert!(preparation_audit(&changed_data).is_err());
        // Matching a rewritten JSON file's digest is insufficient when its
        // contents no longer certify the pinned training/diagnostic boundary.
        for pointer in [
            "/recomposition/manifest_sha256",
            "/recomposition/inputs_sha256",
            "/recomposition/labels_sha256",
            "/recomposition/rows",
            "/recomposition/target_positions",
            "/original_inputs_sha256",
            "/original_labels_sha256",
            "/diagnostic_inputs_sha256",
            "/diagnostic_labels_sha256",
            "/concrete_donor_roots",
            "/exposed_whole_bank_overlap",
            "/fresh_rows_authored",
            "/checks/sealed_preparation_chain",
            "/checks/exact_concrete_wire_origins",
            "/checks/whole_bank_exclusion",
            "/checks/balanced512_quotas",
            "/checks/typed_answer_membership",
            "/checks/development_only_no_fresh",
        ] {
            let mut changed = audit.clone();
            *changed
                .pointer_mut(pointer)
                .ok_or_else(|| bad("audit fixture field absent"))? = json!(false);
            let changed_bytes = serde_json::to_vec(&changed)?;
            fs::write(&path, &changed_bytes)?;
            assert!(preparation_audit(&config).is_err());
            let mut repinned = config.clone();
            repinned.expected_preparation_audit_sha256 = sha256_bytes(&changed_bytes);
            assert!(preparation_audit(&repinned).is_err(), "{pointer}");
        }
        fs::write(&path, bytes)?;
        assert_eq!(preparation_audit(&config)?, audit);
        Ok(())
    }

    #[test]
    fn sealed_dataset_boundary_rejects_hash_and_complete_file_set_tampering() -> Result<()> {
        let owned = owned_root()?;
        let root = owned.0.join("sealed-input");
        report_output::claim(&root)?;
        let input = b"{\"cases\":[{\"id\":\"fixed\"}]}";
        let labels = b"{\"cases\":[{\"id\":\"fixed\",\"answer\":\"original\"}]}";
        fs::write(root.join("inputs.json"), input)?;
        fs::write(root.join("labels.json"), labels)?;
        fs::write(root.join("report.json"), b"{\"status\":\"COMPLETED\"}")?;
        report_output::seal(&root)?;
        let manifest_sha = sha256_file(&root.join("manifest.json"))?;
        let input_sha = sha256_bytes(input);
        let label_sha = sha256_bytes(labels);
        bound(&root, &manifest_sha)?;
        input_identity(&root, &input_sha, &label_sha)?;
        assert!(bound(&root, &"0".repeat(64)).is_err());
        assert!(input_identity(&root, &label_sha, &label_sha).is_err());
        assert!(input_identity(&root, &input_sha, &input_sha).is_err());
        // These mutate only this exclusively created test fixture, never an
        // actual experiment root. Exercise the production complete-file seal.
        fs::write(root.join("inputs.json"), b"{\"cases\":[]}")?;
        assert!(bound(&root, &manifest_sha).is_err());
        assert!(input_identity(&root, &input_sha, &label_sha).is_err());
        fs::write(root.join("inputs.json"), input)?;
        fs::write(root.join("unreceipted.json"), b"{}")?;
        assert!(bound(&root, &manifest_sha).is_err());
        fs::remove_file(root.join("unreceipted.json"))?;
        fs::remove_file(root.join("labels.json"))?;
        assert!(bound(&root, &manifest_sha).is_err());
        assert!(input_identity(&root, &input_sha, &label_sha).is_err());
        fs::write(root.join("labels.json"), labels)?;
        bound(&root, &manifest_sha)?;
        let manifest = fs::read(root.join("manifest.json"))?;
        fs::write(root.join("manifest.json"), b"{}")?;
        assert!(bound(&root, &manifest_sha).is_err());
        fs::write(root.join("manifest.json"), manifest)?;
        bound(&root, &manifest_sha)?;
        Ok(())
    }

    #[test]
    fn diagnostic_baseline_rejects_same_count_output_eos_identity_and_order_changes() -> Result<()>
    {
        // Raw saved evaluator rows and fitter summary rows have these same
        // comparison fields, even though their step traces differ in layout.
        let saved = (0..128)
            .map(|i| {
                json!({"id":format!("fresh-case-{i}"),
                "generated_ids":[100+i,200+i],"eos":i<4,"complete":false})
            })
            .collect::<Vec<_>>();
        let data = Data {
            additional: Vec::new(),
            diagnostic: Vec::new(),
            receipt: Value::Null,
            baseline: saved.clone(),
        };
        let current = json!({"complete":0,"continuation_sha256":FIELD437_SHA,"rows":saved});
        baseline(&data, &current)?;
        for (key, wrong) in [
            ("generated_ids", json!([999, 200])),
            ("eos", json!(false)),
            ("id", json!("changed-fresh-case")),
            ("complete", json!(true)),
        ] {
            let mut changed = current.clone();
            changed["rows"][0][key] = wrong;
            assert_eq!(changed["complete"], 0);
            assert!(baseline(&data, &changed).is_err(), "{key}");
        }
        let mut reordered = current.clone();
        reordered["rows"]
            .as_array_mut()
            .ok_or_else(|| bad("diagnostic fixture rows absent"))?
            .swap(50, 51);
        assert!(baseline(&data, &reordered).is_err());
        let mut missing = current.clone();
        missing["rows"]
            .as_array_mut()
            .ok_or_else(|| bad("diagnostic fixture rows absent"))?
            .pop();
        assert!(baseline(&data, &missing).is_err());
        for (key, wrong) in [
            ("complete", json!(1)),
            ("continuation_sha256", json!("0".repeat(64))),
        ] {
            let mut changed = current.clone();
            changed[key] = wrong;
            assert!(baseline(&data, &changed).is_err(), "{key}");
        }
        Ok(())
    }

    #[test]
    fn two_stream_schedule_is_exact_two_passes_each_without_global_shuffle() -> Result<()> {
        let schedule = schedule(1001);
        let mut counts = vec![0usize; 1024];
        for update in 0..256 {
            let batch = indices(&schedule, update)?;
            assert!(batch[..4].iter().all(|&i| i < 512));
            assert!(batch[4..].iter().all(|&i| i >= 512 && i < 1024));
            for i in batch {
                counts[i] += 1;
            }
        }
        assert!(counts.iter().all(|&n| n == 2));
        assert!(indices(&schedule, 256).is_err());
        let mut broken = schedule.clone();
        broken[0] = broken[1];
        assert!(indices(&broken, 0).is_err());
        Ok(())
    }
    fn endpoint(cases: usize, complete: usize) -> Value {
        json!({"complete":complete,"rows":(0..cases).map(|i|json!({"id":format!("case-{i}"),"complete":i<complete})).collect::<Vec<_>>()})
    }
    #[test]
    fn transfer_bar_is_not_development_net_gain_or_fresh_acceptance() -> Result<()> {
        assert_eq!(
            outcome(&endpoint(512, 256), &endpoint(128, 52))?["qualification_candidate"],
            true
        );
        assert_eq!(
            outcome(&endpoint(512, 255), &endpoint(128, 128))?["qualification_candidate"],
            false
        );
        assert_eq!(
            outcome(&endpoint(512, 500), &endpoint(128, 51))?["qualification_candidate"],
            false
        );
        let retained = outcome(&endpoint(512, 300), &endpoint(128, 80))?;
        assert_eq!(retained["best_development_complete"], 437);
        assert_eq!(retained["count_after"], 2);
        assert_eq!(
            outcome(&endpoint(512, 438), &endpoint(128, 52))?["count_after"],
            0
        );
        let mut bad = endpoint(128, 52);
        bad["rows"][1]["id"] = bad["rows"][0]["id"].clone();
        assert!(outcome(&endpoint(512, 437), &bad).is_err());
        let mut wrong_count = endpoint(128, 52);
        wrong_count["complete"] = json!(53);
        assert!(outcome(&endpoint(512, 437), &wrong_count).is_err());
        Ok(())
    }
}
