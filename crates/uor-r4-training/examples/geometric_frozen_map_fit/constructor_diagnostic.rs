//! Phase two: freeze cross-arm, per-seed selections from four sealed training
//! reports before making any opened-panel prediction. No optimization here.
use super::constructor_experiment as experiment;
use super::*;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Config {
    reports: [PathBuf; 4],
    out: PathBuf,
    maximum_seconds: u64,
    maximum_report_bytes: u64,
}
struct Run {
    root: PathBuf,
    args: Args,
    report: Value,
    seed: u64,
    arm: u8,
    pins: Value,
}
#[derive(Clone, Debug, serde::Serialize)]
struct Choice {
    run: usize,
    candidate: usize,
    seed: u64,
    arm: u8,
    epoch: usize,
    radius: u8,
    complete: u64,
    objective: f64,
    checkpoint_index: usize,
}
fn number(v: &Value, label: &str) -> Result<u64> {
    v.as_u64().ok_or_else(|| bad(label))
}
fn index(v: &Value, label: &str) -> Result<usize> {
    Ok(usize::try_from(number(v, label)?)?)
}
fn array<'a>(v: &'a Value, label: &str) -> Result<&'a Vec<Value>> {
    v.as_array().ok_or_else(|| bad(label))
}
fn arm(v: &Value) -> Result<u8> {
    match v.as_str() {
        Some("legal") => Ok(0),
        Some("projected") => Ok(1),
        _ => Err(bad("diagnostic unknown arm")),
    }
}
fn normalized_config(mut v: Value) -> Result<Value> {
    let object = v
        .as_object_mut()
        .ok_or_else(|| bad("diagnostic config not object"))?;
    object.remove("out");
    object.remove("seed");
    object
        .get_mut("constructor")
        .and_then(Value::as_object_mut)
        .ok_or_else(|| bad("diagnostic constructor config absent"))?
        .remove("arm");
    Ok(v)
}
fn validate_rows(value: &Value, expected: usize) -> Result<Vec<Value>> {
    let rows = array(&value["rows"], "diagnostic saved rows missing")?;
    let mut ids = BTreeSet::new();
    let mut count = 0;
    replay_require(rows.len() == expected, "diagnostic saved row extent")?;
    for row in rows {
        let id = row["id"]
            .as_str()
            .ok_or_else(|| bad("diagnostic saved row ID"))?;
        replay_require(
            ids.insert(id) && row["complete"].is_boolean(),
            "diagnostic duplicate ID/nonboolean completion",
        )?;
        let generated: Vec<u32> = serde_json::from_value(row["generated_ids"].clone())?;
        replay_require(generated.len() <= 32, "diagnostic saved output cap")?;
        count += usize::from(row["complete"] == true);
    }
    replay_require(
        number(&value["complete"], "diagnostic aggregate count absent")? == count as u64,
        "diagnostic aggregate count differs from rows",
    )?;
    Ok(rows.iter().map(|row|json!({"id":row["id"],"generated_ids":row["generated_ids"],"complete":row["complete"]})).collect())
}
fn checkpoint_path(root: &Path, index: usize) -> Result<PathBuf> {
    replay_require(
        (1..=8).contains(&index),
        "diagnostic checkpoint index outside registered bounds",
    )?;
    Ok(root.join(format!("checkpoint-{index:04}")))
}
fn candidate_choices(run: usize, r: &Run) -> Result<Vec<Choice>> {
    let candidates = array(&r.report["candidates"], "diagnostic candidates missing")?;
    replay_require(
        candidates.len() <= if r.arm == 0 { 2 } else { 8 },
        "diagnostic too many candidates",
    )?;
    let mut checkpoints = BTreeSet::new();
    candidates
        .iter()
        .enumerate()
        .map(|(i, c)| {
            replay_require(
                index(&c["candidate_index"], "candidate index missing")? == i,
                "candidate order mismatch",
            )?;
            let epoch = index(&c["epoch"], "candidate epoch missing")?;
            let name = c["name"]
                .as_str()
                .ok_or_else(|| bad("candidate name missing"))?;
            let radius = if r.arm == 0 {
                0
            } else {
                [1, 2, 4, 7]
                    .into_iter()
                    .find(|n| name == format!("epoch-{epoch}-radius-{n}"))
                    .ok_or_else(|| bad("diagnostic radius/name mismatch"))?
            };
            replay_require(
                epoch < 2 && (r.arm != 0 || name == format!("epoch-{epoch}-legal")),
                "candidate epoch/name mismatch",
            )?;
            let checkpoint_index = index(&c["checkpoint_index"], "checkpoint index missing")?;
            replay_require(
                checkpoint_index > 0 && checkpoints.insert(checkpoint_index),
                "duplicate/zero candidate checkpoint",
            )?;
            let objective = c["objective"]
                .as_f64()
                .filter(|x| x.is_finite())
                .ok_or_else(|| bad("candidate objective nonfinite"))?;
            let complete = number(&c["development"]["complete"], "candidate complete missing")?;
            replay_require(
                complete <= 512
                    && array(&c["development"]["rows"], "candidate dev rows missing")?.len() == 512,
                "candidate development extent",
            )?;
            let identities = validate_rows(&c["development"], 512)?;
            let base = validate_rows(&r.report["initial"], 512)?;
            replay_require(
                identities
                    .iter()
                    .zip(&base)
                    .all(|(a, b)| a["id"] == b["id"]),
                "candidate development row IDs differ from baseline",
            )?;
            // Never interpret a caller-authored path as artifact authority.
            let path = checkpoint_path(&r.root, checkpoint_index)?;
            replay_require(
                path.is_dir(),
                "selected checkpoint absent from sealed report",
            )?;
            Ok(Choice {
                run,
                candidate: i,
                seed: r.seed,
                arm: r.arm,
                epoch,
                radius,
                complete,
                objective,
                checkpoint_index,
            })
        })
        .collect()
}
fn first_key(c: &Choice) -> (usize, u8, u8, usize) {
    (c.epoch, c.arm, c.radius, c.candidate)
}
fn better(a: &Choice, b: &Choice) -> bool {
    a.complete > b.complete
        || (a.complete == b.complete
            && (a.objective < b.objective
                || (a.objective == b.objective && first_key(a) < first_key(b))))
}
fn pick(candidates: &[Choice]) -> (Option<Choice>, Option<Choice>) {
    let first = candidates.iter().min_by_key(|c| first_key(c)).cloned();
    let nominee = candidates
        .iter()
        .fold(None, |best: Option<&Choice>, c| match best {
            Some(b) if !better(c, b) => Some(b),
            _ => Some(c),
        })
        .cloned();
    (first, nominee)
}
fn selection(runs: &[Run]) -> Result<Value> {
    let mut seeds = Vec::new();
    for seed in [1001, 2001] {
        let mut candidates = Vec::new();
        let mut endpoints = Vec::new();
        for (i, r) in runs.iter().enumerate().filter(|(_, r)| r.seed == seed) {
            let choices = candidate_choices(i, r)?;
            if !r.report["endpoint"].is_null() {
                let endpoint = index(&r.report["endpoint"], "endpoint missing")?;
                let c = choices
                    .get(endpoint)
                    .ok_or_else(|| bad("endpoint index out of range"))?;
                replay_require(
                    r.report["candidates"][endpoint]["committed"] == true,
                    "endpoint not committed",
                )?;
                endpoints.push(serde_json::to_value(c)?);
            } else {
                endpoints.push(json!({"run":i,"seed":seed,"arm":r.arm,"baseline":true}));
            }
            candidates.extend(choices);
        }
        let (first, nominee) = pick(&candidates);
        seeds
            .push(json!({"seed":seed,"endpoints":endpoints,"first_legal":first,"nominee":nominee}));
    }
    Ok(
        json!({"schema":"uor-r4.d22-global-selection/1","seeds":seeds,
        "nomination":"complete descending; native objective ascending; epoch then arm legal/projected then radius; includes native/linear-vetoed valid Q4 candidates",
        "first":"epoch then arm legal/projected then fixed radius order",
        "diagnostic_predictions":"NOT_RUN","all_four_training_reports_sealed_before_selection":true}),
    )
}
fn initial_credit_identity(v: &Value, width: usize, guards: usize) -> Result<Value> {
    let gradient: Vec<f32> = serde_json::from_value(v["effective_gradient_f32"].clone())?;
    let masters: Vec<f32> = serde_json::from_value(v["original_masters"].clone())?;
    let rows: Vec<Vec<f64>> = serde_json::from_value(v["unit_rows"].clone())?;
    replay_require(
        gradient.len() == width
            && masters.len() == width
            && rows.len() == guards
            && gradient.iter().chain(&masters).all(|x| x.is_finite())
            && rows
                .iter()
                .all(|r| r.len() == width && r.iter().all(|x| x.is_finite())),
        "paired initial credit shape/nonfinite; comparison setup failure",
    )?;
    Ok(json!({"width":width,"guards":guards,
        "effective_gradient_f32_bits_sha256":sha256_bytes(&gradient.into_iter().flat_map(f32::to_le_bytes).collect::<Vec<_>>()),
        "original_masters_f32_bits_sha256":sha256_bytes(&masters.into_iter().flat_map(f32::to_le_bytes).collect::<Vec<_>>()),
        "unit_rows_f64_bits_sha256":sha256_bytes(&rows.into_iter().flatten().flat_map(f64::to_le_bytes).collect::<Vec<_>>())}))
}
fn paired_initial_credit(a: &Value, b: &Value, width: usize, guards: usize) -> Result<Value> {
    let left = initial_credit_identity(a, width, guards)?;
    let right = initial_credit_identity(b, width, guards)?;
    replay_require(left==right,"paired epoch0 effective gradient/guards/masters differ; comparison setup failure, not a model negative")?;
    Ok(left)
}
fn validate_training_roots(c: &Config) -> Result<Vec<Run>> {
    let mut runs = Vec::new();
    let mut combinations = BTreeSet::new();
    let mut roots = BTreeSet::new();
    let mut common = None;
    let mut source = None;
    let mut baseline = None;
    for root in &c.reports {
        let root = fs::canonicalize(root)?;
        replay_require(
            roots.insert(root.clone()),
            "diagnostic duplicate training root",
        )?;
        report_output::verify(&root)?;
        let report = read(&root.join("report.json"))?;
        let raw = fs::read(root.join("config.json"))?;
        let config_value: Value = serde_json::from_slice(&raw)?;
        let args: Args = serde_json::from_slice(&raw)?;
        experiment::settings(&args)?;
        let seed = number(&report["seed"], "training seed absent")?;
        let arm = arm(&report["arm"])?;
        replay_require(
            [1001, 2001].contains(&seed)
                && combinations.insert((seed, arm))
                && seed == args.seed
                && arm == arm_value(&config_value["constructor"]["arm"])?
                && report["schema"] == "uor-r4.d22-constructor-result/1"
                && report["status"] == "COMPLETED_TRAINING_DIAGNOSTIC_PENDING"
                && report["base_report"] == experiment::REPORT
                && report["base_field"] == experiment::FIELD
                && report["initial"]["complete"] == 437
                && array(&report["initial"]["rows"], "baseline rows missing")?.len() == 512,
            "diagnostic training authority/status/base mismatch",
        )?;
        let normalized = normalized_config(config_value)?;
        if let Some(ref old) = common {
            replay_require(
                *old == normalized,
                "paired run configurations differ beyond seed/arm/out",
            )?;
        } else {
            common = Some(normalized);
        }
        let commit = report["source_commit"]
            .as_str()
            .filter(|x| x.len() == 40 && x.bytes().all(|b| b.is_ascii_hexdigit()))
            .ok_or_else(|| bad("training source commit absent"))?
            .to_owned();
        if let Some(ref old) = source {
            replay_require(*old == commit, "training producer source differs")?;
        } else {
            source = Some(commit.clone());
        }
        replay_require(
            option_env!("UOR_BUILD_SOURCE_COMMIT") == Some(commit.as_str()),
            "diagnostic executable source differs from training",
        )?;
        let identities = validate_rows(&report["initial"], 512)?;
        if let Some(ref old) = baseline {
            replay_require(*old == identities, "paired saved437 baseline rows differ")?;
        } else {
            baseline = Some(identities);
        }
        let pins = json!({"root":root,"report_sha256":sha256_file(&root.join("report.json"))?,
            "manifest_sha256":sha256_file(&root.join("manifest.json"))?,"config_sha256":sha256_bytes(&raw),
            "guard_identities_sha256":sha256_file(&root.join("guard-identities.json"))?,"source_commit":commit});
        runs.push(Run {
            root,
            args,
            report,
            seed,
            arm,
            pins,
        });
    }
    replay_require(
        combinations == BTreeSet::from([(1001, 0), (1001, 1), (2001, 0), (2001, 1)]),
        "diagnostic paired seed/arm matrix incomplete",
    )?;
    for seed in [1001, 2001] {
        let pair = runs
            .iter()
            .enumerate()
            .filter(|(_, r)| r.seed == seed)
            .map(|(i, _)| i)
            .collect::<Vec<_>>();
        replay_require(
            pair.len() == 2
                && fs::read(runs[pair[0]].root.join("guard-identities.json"))?
                    == fs::read(runs[pair[1]].root.join("guard-identities.json"))?,
            "paired sampled guard identities differ",
        )?;
        let left = runs[pair[0]].root.join("epoch-0-credit.json");
        let right = runs[pair[1]].root.join("epoch-0-credit.json");
        let identity = paired_initial_credit(&read(&left)?, &read(&right)?, 1920, 380)?;
        let receipt = json!({"seed":seed,"epoch":0,"exact_decoded_bits_equal":true,"fields":identity,
            "reports":[{"arm":runs[pair[0]].arm,"credit_file_sha256":sha256_file(&left)?},
                       {"arm":runs[pair[1]].arm,"credit_file_sha256":sha256_file(&right)?}],
            "scope":"matched initial credit; later accepted parents may differ; failure is comparison setup, never model quality"});
        for i in pair {
            runs[i].pins["paired_initial_credit"] = receipt.clone();
        }
    }
    // Stable order makes report-list input ordering irrelevant to nomination.
    runs.sort_by_key(|r| (r.seed, r.arm));
    Ok(runs)
}
fn arm_value(v: &Value) -> Result<u8> {
    arm(v)
}
fn breakdown(v: &Value, eps: &[Episode]) -> Result<Value> {
    let rows = array(&v["rows"], "diagnostic result rows absent")?;
    replay_require(rows.len() == eps.len(), "diagnostic row extent")?;
    let mut strata = BTreeMap::<String, [usize; 2]>::new();
    let mut banks = BTreeMap::<String, [usize; 2]>::new();
    for (row, e) in rows.iter().zip(eps) {
        replay_require(
            row["id"] == e.packet.id && row["complete"].is_boolean(),
            "diagnostic row identity/boolean",
        )?;
        let stratum = e
            .packet
            .id
            .split('-')
            .nth(2)
            .ok_or_else(|| bad("diagnostic stratum ID"))?;
        replay_require(
            ["length2", "length4", "length8", "update", "reassert"].contains(&stratum),
            "diagnostic unknown stratum",
        )?;
        let mut values = e
            .packet
            .segments
            .iter()
            .filter_map(|s| match s {
                Segment::Source {
                    original_source_ids,
                    ..
                } => Some(original_source_ids.clone()),
                _ => None,
            })
            .collect::<Vec<_>>();
        replay_require(
            values.len() == 2 && values[0] != values[1],
            "diagnostic expected two distinct current literals",
        )?;
        values.sort();
        let key = sha256_bytes(&serde_json::to_vec(&values)?);
        for count in [
            strata.entry(stratum.into()).or_default(),
            banks.entry(key).or_default(),
        ] {
            count[0] += 1;
            count[1] += usize::from(row["complete"] == true);
        }
    }
    replay_require(
        banks.len() == 8
            && strata.iter().all(|(k, v)| {
                v[0] == if ["update", "reassert"].contains(&k.as_str()) {
                    16
                } else {
                    32
                }
            }),
        "diagnostic frozen block/stratum quota differs",
    )?;
    Ok(
        json!({"strata_total_complete":strata,"bank_total_complete":banks,
        "successful_bank_blocks":banks.values().filter(|v|v[1]>0).count(),
        "bank_key":"sha256 sorted current full-literal token vectors; shared across role swaps/query/order"}),
    )
}
fn execute(c: &Config, start: Instant) -> Result<Value> {
    let runs = validate_training_roots(c)?;
    let mut a = runs[0].args.clone();
    a.out = c.out.clone();
    a.maximum_seconds = c.maximum_seconds;
    a.maximum_report_bytes = c.maximum_report_bytes;
    write(
        &a,
        "paired-initial-credit.json",
        &json!({"schema":"uor-r4.d22-paired-initial-credit/1",
        "pairs":runs.iter().filter(|r|r.arm==0).map(|r|&r.pins["paired_initial_credit"]).collect::<Vec<_>>()}),
    )?;
    let selected = selection(&runs)?;
    write(&a, "selection-before-diagnostic.json", &selected)?;
    write(
        &a,
        "training-identities.json",
        &json!({"runs":runs.iter().map(|r|&r.pins).collect::<Vec<_>>()}),
    )?;
    // No model is loaded until all four training reports and global selection are fixed.
    let original = ContinuationParent::load(&a)?;
    let (field, _) = experiment::load_field(&a, &original)?;
    let config = a
        .constructor
        .as_ref()
        .ok_or_else(|| bad("diagnostic constructor config"))?;
    for (path, pin) in [
        (&config.diagnostic_inputs, experiment::DIAGNOSTIC_INPUT),
        (&config.diagnostic_labels, experiment::DIAGNOSTIC_LABEL),
    ] {
        report_output::verify(&seal_for(path)?)?;
        replay_require(sha256_file(path)? == pin, "diagnostic panel pin mismatch")?;
    }
    let reducer = NativeVocabularyActions::new(original.integer.binding().clone(), &original.exp)?;
    let legal = reducer
        .legal_token_ids()
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    let eps = load_panel(
        &config.diagnostic_inputs,
        &config.diagnostic_labels,
        &original.integer,
        &original.tokenizer,
        &legal,
        128,
    )?;
    replay_require(eps.len() == 128, "diagnostic panel must have128 rows")?;
    let baseline =
        experiment::own_prefix(&a, "baseline-opened128", &original, &field, &eps, start)?;
    replay_require(
        baseline["complete"] == 0,
        "saved437 diagnostic baseline differs",
    )?;
    let baseline_breakdown = breakdown(&baseline, &eps)?;
    let mut chosen = BTreeSet::new();
    for seed in array(&selected["seeds"], "selection seeds missing")? {
        for endpoint in array(&seed["endpoints"], "selection endpoints missing")? {
            if endpoint["baseline"] != true {
                chosen.insert((
                    index(&endpoint["run"], "selection run")?,
                    index(&endpoint["candidate"], "selection candidate")?,
                ));
            }
        }
        for key in ["first_legal", "nominee"] {
            if !seed[key].is_null() {
                chosen.insert((
                    index(&seed[key]["run"], "selection run")?,
                    index(&seed[key]["candidate"], "selection candidate")?,
                ));
            }
        }
    }
    let mut results = Vec::new();
    for (ri, ci) in chosen {
        let r = runs
            .get(ri)
            .ok_or_else(|| bad("selected run out of range"))?;
        let choices = candidate_choices(ri, r)?;
        let choice = choices
            .get(ci)
            .ok_or_else(|| bad("selected candidate out of range"))?;
        let cp = checkpoint_path(&r.root, choice.checkpoint_index)?;
        let (p, f) = experiment::read_checkpoint(&cp)?;
        replay_require(
            p.binding == original.binding
                && p.bridge == original.bridge
                && p.cue == original.cue
                && p.joint == original.joint
                && f.packed_cross_state() == field.packed_cross_state(),
            "diagnostic candidate frozen payload mismatch",
        )?;
        let name = format!("seed-{}-arm-{}-candidate-{}-opened128", r.seed, r.arm, ci);
        let result = experiment::own_prefix(&a, &name, &p, &f, &eps, start)?;
        let strata = breakdown(&result, &eps)?;
        results.push(json!({"selection":choice,"training_identity":r.pins,"checkpoint":cp,
            "checkpoint_receipt_sha256":sha256_file(&cp.join("receipt.json"))?,
            "linear_screen":r.report["candidates"][ci]["linear_screen"],"native_guards":r.report["candidates"][ci]["native_guards"],
            "committed":r.report["candidates"][ci]["committed"],"diagnostic":result,"breakdown":strata}));
    }
    Ok(
        json!({"schema":"uor-r4.d22-constructor-diagnostic/1","status":"COMPLETED","source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT"),
        "selection":selected,"training_runs":runs.iter().map(|r|&r.pins).collect::<Vec<_>>(),
        "baseline":baseline,"baseline_breakdown":baseline_breakdown,"selected_results":results,
        "elapsed_seconds":start.elapsed().as_secs_f64(),"scope":"opened128 diagnostic only; all four training results sealed and selection fixed before predictions; fresh qualification NOT_RUN"}),
    )
}
pub(super) fn run(path: &Path) -> Result<()> {
    let raw = fs::read(path)?;
    let c: Config = serde_json::from_slice(&raw)?;
    replay_require(
        c.maximum_seconds > 0
            && c.maximum_report_bytes >= 64 << 20
            && c.maximum_report_bytes <= 2 << 30,
        "diagnostic resource bounds",
    )?;
    replay_require(
        raw.len() as u64 + 4096 < c.maximum_report_bytes,
        "diagnostic config report bound",
    )?;
    let output = output_support::prospective_output(&c.out)?;
    let config_path = fs::canonicalize(path)?;
    replay_require(
        !config_path.starts_with(&output),
        "diagnostic config/output overlap",
    )?;
    for root in &c.reports {
        let input = fs::canonicalize(root)?;
        replay_require(
            !input.starts_with(&output) && !output.starts_with(&input),
            "diagnostic training/output overlap",
        )?;
        // Protect every bound model/data input as well as the four sealed reports.
        let mut a: Args = serde_json::from_slice(&fs::read(root.join("config.json"))?)?;
        a.out = c.out.clone();
        validate_input_output_paths(&a)?;
    }
    report_output::claim(&c.out)?;
    let start = Instant::now();
    let result = (|| -> Result<Value> {
        fs::write(c.out.join("config.json"), &raw)?;
        execute(&c, start)
    })();
    let report = match &result {
        Ok(v) => v.clone(),
        Err(e) => json!({"schema":"uor-r4.d22-constructor-diagnostic/1","status":"FAILED",
        "error":e.to_string(),"elapsed_seconds":start.elapsed().as_secs_f64(),"model_verdict":"UNQUALIFIED; execution failure is not model failure"}),
    };
    fs::write(
        c.out.join("report.json"),
        serde_json::to_vec_pretty(&report)?,
    )?;
    report_output::seal(&c.out)?;
    report_output::verify(&c.out)?;
    result.map(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn choice(arm: u8, epoch: usize, radius: u8, complete: u64, objective: f64) -> Choice {
        Choice {
            run: arm as usize,
            candidate: epoch,
            seed: 1001,
            arm,
            epoch,
            radius,
            complete,
            objective,
            checkpoint_index: epoch + 1,
        }
    }
    #[test]
    fn constructor_diagnostic_global_nomination_is_cross_arm_and_first_is_separate() -> Result<()> {
        let legal = choice(0, 0, 0, 440, 2.);
        let projected = choice(1, 0, 1, 470, 3.);
        let later = choice(0, 1, 0, 469, 1.);
        let candidates = vec![later, projected, legal];
        let (first, best) = pick(&candidates);
        assert_eq!(first.ok_or_else(|| bad("first missing"))?.arm, 0);
        assert_eq!(best.ok_or_else(|| bad("nominee missing"))?.arm, 1);
        let mut reversed = candidates.clone();
        reversed.reverse();
        assert_eq!(
            serde_json::to_value(pick(&candidates))?,
            serde_json::to_value(pick(&reversed))?
        );
        // Both production inputs and the selection type intentionally have no
        // transaction/native-guard eligibility filter.
        Ok(())
    }
    #[test]
    fn constructor_diagnostic_tie_order_uses_objective_then_epoch_arm_radius() -> Result<()> {
        let candidates = vec![
            choice(1, 0, 7, 470, 2.),
            choice(1, 0, 1, 470, 2.),
            choice(0, 1, 0, 470, 2.),
        ];
        let (_, best) = pick(&candidates);
        let best = best.ok_or_else(|| bad("best missing"))?;
        assert_eq!((best.epoch, best.arm, best.radius), (0, 1, 1));
        let mut with_legal = candidates;
        with_legal.push(choice(0, 0, 0, 470, 2.));
        assert_eq!(
            pick(&with_legal)
                .1
                .ok_or_else(|| bad("legal tie missing"))?
                .arm,
            0
        );
        with_legal.push(choice(1, 1, 4, 470, 1.));
        assert_eq!(
            pick(&with_legal)
                .1
                .ok_or_else(|| bad("objective tie missing"))?
                .objective,
            1.
        );
        assert!(pick(&[]).0.is_none() && pick(&[]).1.is_none());
        Ok(())
    }
    #[test]
    fn constructor_diagnostic_config_comparison_only_ignores_registered_axes() -> Result<()> {
        let a = json!({"seed":1001,"out":"a","constructor":{"arm":"legal","saved437":"fixed"},"credit":"raw_identity","updates":2});
        let mut b = a.clone();
        b["seed"] = json!(2001);
        b["out"] = json!("b");
        b["constructor"]["arm"] = json!("projected");
        assert_eq!(normalized_config(a.clone())?, normalized_config(b.clone())?);
        b["constructor"]["saved437"] = json!("different");
        assert_ne!(normalized_config(a.clone())?, normalized_config(b.clone())?);
        b = a.clone();
        b["updates"] = json!(3);
        assert_ne!(normalized_config(a)?, normalized_config(b)?);
        assert!(normalized_config(json!({"seed":1001})).is_err());
        Ok(())
    }
    #[test]
    fn constructor_diagnostic_rejects_count_identity_tampering_and_unbound_checkpoint_index(
    ) -> Result<()> {
        let v = json!({"complete":1,"rows":[{"id":"a","generated_ids":[4,1],"complete":true},{"id":"b","generated_ids":[1],"complete":false}]});
        validate_rows(&v, 2)?;
        let mut bad_count = v.clone();
        bad_count["complete"] = json!(2);
        assert!(validate_rows(&bad_count, 2).is_err());
        let mut duplicate = v.clone();
        duplicate["rows"][1]["id"] = json!("a");
        assert!(validate_rows(&duplicate, 2).is_err());
        let mut malformed = v;
        malformed["rows"][0]["complete"] = json!("true");
        assert!(validate_rows(&malformed, 2).is_err());
        assert_eq!(
            checkpoint_path(Path::new("owned/report"), 2)?,
            PathBuf::from("owned/report/checkpoint-0002")
        );
        assert!(checkpoint_path(Path::new("owned/report"), 0).is_err());
        assert!(checkpoint_path(Path::new("owned/report"), usize::MAX).is_err());
        Ok(())
    }
    #[test]
    fn constructor_diagnostic_requires_exact_initial_credit_bits_in_both_arms() -> Result<()> {
        let initial = json!({"effective_gradient_f32":[0.,1.],"unit_rows":[[0.,1.]],"original_masters":[0.1,0.25]});
        let identity = paired_initial_credit(&initial, &initial, 2, 1)?;
        assert_eq!(identity["width"], 2);
        for field in ["effective_gradient_f32", "original_masters"] {
            let mut changed = initial.clone();
            changed[field][0] = json!(0.2);
            assert!(paired_initial_credit(&initial, &changed, 2, 1).is_err());
        }
        let mut changed = initial.clone();
        changed["unit_rows"][0][0] = json!(1e-300);
        assert!(paired_initial_credit(&initial, &changed, 2, 1).is_err());
        // Ordinary numeric equality would hide this change; exact master/credit
        // identity is deliberately bitwise, including the sign of zero.
        changed = initial.clone();
        changed["effective_gradient_f32"][0] = json!(-0.0);
        assert!(paired_initial_credit(&initial, &changed, 2, 1).is_err());
        assert!(paired_initial_credit(&initial, &initial, 3, 1).is_err());
        changed = initial.clone();
        changed
            .as_object_mut()
            .ok_or_else(|| bad("fixture object"))?
            .remove("unit_rows");
        assert!(paired_initial_credit(&initial, &changed, 2, 1).is_err());
        Ok(())
    }
}
