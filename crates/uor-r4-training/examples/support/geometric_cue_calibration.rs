//! Warm cue-only learning through the complete native prefix/endpoint path.
use super::*;
#[path = "geometric_cue_credit_audit.rs"]
mod credit_audit;
#[path = "geometric_cue_discrete.rs"]
mod discrete;
use uor_r4_integer::geometric_source_end_transport::{
    NativeSourceEndTransport, SourceEndAngularConfig, SourceEndAngularQ4, SourceEndScoreMode,
};

const FAMILIES: &str = "cue-angular-HxLx120-only/full-native-SourceEnd/1";
const DATA_SCOPE: &str =
    "explicit-current-role-assertions/raw-current-role-queries/all-source-candidates/2";
const COMPOSITION_PROFILE: &str = "supported-prospective-role-diversity/1";

fn composition_panel(a: &Args) -> Option<&CueCompositionPanel> {
    a.cue_credit_audit
        .as_ref()
        .map(|c| &c.composition_panel)
        .or_else(|| {
            a.cue_discrete_fit
                .as_ref()
                .and_then(|c| c.composition_panel.as_ref())
        })
}
fn panel_counts(a: &Args) -> (usize, usize) {
    composition_panel(a).map_or((128, 32), |p| (p.development_rows, p.evaluation_rows))
}

fn composition_report_matches(report: &Value, split: &str, rows: usize) -> bool {
    report["schema"] == "uor-r4.native-bank-observation-panel/1"
        && report["status"] == "COMPLETED"
        && report["transfer_profile"] == COMPOSITION_PROFILE
        && report["source_policy"] == DATA_SCOPE
        && report["split"] == split
        && report["cases"] == rows
        && report["samebank_query_pairs"] == rows / 2
}

pub(super) fn validate(a: &Args) -> Result<()> {
    if cue_calibration_mode(a) != a.cue_calibration_warmstart.is_some() {
        return Err(invalid("cue calibration warmstart is explicit-mode-only").into());
    }
    if (a.mode == "cue-calibration-quantum-probe") != a.cue_quantum_probe.is_some() {
        return Err(invalid("cue quantum proposal is explicit-probe-only").into());
    }
    if matches!(
        a.mode.as_str(),
        "cue-calibration-discrete-fit" | "cue-calibration-discrete-complete"
    ) != a.cue_discrete_fit.is_some()
    {
        return Err(invalid("cue discrete configuration is explicit-discrete-fit-only").into());
    }
    if (a.mode == "cue-calibration-discrete-complete") != a.cue_discrete_completion_root.is_some()
        || a.cue_discrete_completion_root.is_some()
            != a.cue_discrete_completion_manifest_sha256.is_some()
        || a.cue_discrete_completion_root.is_some()
            != a.cue_discrete_completion_config_sha256.is_some()
    {
        return Err(invalid(
            "completion requires explicit sealed failed root and manifest binding",
        )
        .into());
    }
    if (a.mode == "cue-calibration-credit-audit") != a.cue_credit_audit.is_some() {
        return Err(invalid("credit audit configuration is explicit-audit-only").into());
    }
    if let Some(c) = &a.cue_credit_audit {
        let w = warm(a)?;
        if c.composition_panel.profile != COMPOSITION_PROFILE
            || c.composition_panel.development_rows != 512
            || c.composition_panel.evaluation_rows != 128
            || c.checkpoint_root.join("cue") != w.initial_cue_bundle
            || c.checkpoint_root.join("prefix")
                != *a
                    .frozen_prefix_bundle
                    .as_ref()
                    .ok_or_else(|| invalid("audit prefix absent"))?
            || c.checkpoint_root.join("source-end") != w.frozen_end_bundle
            || [&c.checkpoint_manifest_sha256, &c.reference_canonical_sha256]
                .iter()
                .any(|s| s.len() != 64 || !s.bytes().all(|b| b.is_ascii_hexdigit()))
            || a.admission.is_some()
            || a.fit_authorization.is_some()
            || a.admission_manifest_sha256.is_some()
        {
            return Err(invalid(
                "credit audit fixed checkpoint/profile/no-optimizer contract differs",
            )
            .into());
        }
    }
    if let Some(c) = &a.cue_discrete_fit {
        if let Some(p) = &c.composition_panel {
            if p.profile != COMPOSITION_PROFILE
                || p.development_rows != 512
                || p.evaluation_rows != 128
            {
                return Err(invalid("prospective composition profile/counts differ").into());
            }
        }
        if !(1..=16).contains(&c.maximum_trials)
            || !(1..=8).contains(&c.maximum_accepted_updates)
            || c.maximum_accepted_updates > c.maximum_trials
            || a.admission.is_some()
            || a.admission_manifest_sha256.is_some()
            || a.fit_authorization.is_some()
        {
            return Err(invalid(
                "cue discrete trial/update/no-legacy-Adam-admission contract differs",
            )
            .into());
        }
    }
    if let Some(q) = &a.cue_quantum_probe {
        if q.coefficient_index != 123
            || q.initial_quarters != -1
            || q.preferred_step != -1
            || q.evidence_receipt_sha256
                != "7f2d33f5fc9562922110b2e5d9f032553665d7a4b2263c51ace113a6793a41d1"
            || a.admission.is_some()
            || a.admission_manifest_sha256.is_some()
            || a.fit_authorization.is_some()
        {
            return Err(invalid(
                "quantum probe predeclared coordinate/evidence/no-optimizer scope differs",
            )
            .into());
        }
    }
    if let Some(w) = &a.cue_calibration_warmstart {
        if w.data_scope != DATA_SCOPE
            || a.cue_score_mode != Some(CueScoreMode::DirectedRelative)
            || a.prefix_score_mode != Some(PrefixScoreMode::DirectedRelative)
            || a.source_end_warmstart.is_some()
            || a.source_end_score_mode.is_some()
            || a.source_end_incumbent_fit.is_some()
            || a.source_end_incumbent_manifest_sha256.is_some()
            || a.learned_source_weights.is_some()
            || a.learned_native_artifact.is_some()
            || a.learned_trusted_native_binding.is_some()
            || a.frozen_cue_bundle.is_some()
            || a.frozen_cue_native_metadata_sha256.is_some()
            || a.frozen_cue_packed_sha256.is_some()
            || a.maximum_generation_tokens != 32
            || a.frozen_prefix_bundle.is_none()
            || a.frozen_prefix_native_metadata_sha256.is_none()
            || a.frozen_prefix_packed_sha256.is_none()
            || a.exposed_controls.is_some()
            || [
                &w.initial_cue_metadata_sha256,
                &w.initial_cue_packed_sha256,
                &w.frozen_end_metadata_sha256,
                &w.frozen_end_period_packed_sha256,
                &w.frozen_end_stop_packed_sha256,
            ]
            .iter()
            .any(|s| s.len() != 64 || !s.bytes().all(|b| b.is_ascii_hexdigit()))
        {
            return Err(invalid("cue calibration scope/mode/initializer contract differs").into());
        }
    }
    Ok(())
}

struct Frozen {
    prefix_config: PrefixAngularConfig,
    prefix: Vec<u8>,
    end_config: SourceEndAngularConfig,
    period: Vec<u8>,
    stop: Vec<u8>,
}
impl Frozen {
    fn training<'a>(
        &self,
        n: &'a NativeSourceRealizer,
        c: &NativeCueCarrier<'_>,
    ) -> Result<(NativePrefixTransport<'a>, NativeSourceEndTransport<'a>)> {
        let p = n.compile_prefix_transport(
            c,
            PrefixAngularQ4::new(self.prefix_config, &self.prefix)
                .map_err(|e| invalid(e.to_string()))?,
        )?;
        let e = n.compile_source_end_transport(
            c,
            &p,
            SourceEndAngularQ4::new(self.end_config, &self.period, &self.stop)
                .map_err(|e| invalid(e.to_string()))?,
        )?;
        Ok((p, e))
    }
    fn integer<'a>(
        &self,
        n: &'a IntegerRealizer,
        c: &NativeCueCarrier<'_>,
    ) -> Result<(NativePrefixTransport<'a>, NativeSourceEndTransport<'a>)> {
        let p = n.compile_prefix_transport(
            c,
            PrefixAngularQ4::new(self.prefix_config, &self.prefix)
                .map_err(|e| invalid(e.to_string()))?,
        )?;
        let e = n.compile_source_end_transport(
            c,
            &p,
            SourceEndAngularQ4::new(self.end_config, &self.period, &self.stop)
                .map_err(|e| invalid(e.to_string()))?,
        )?;
        Ok((p, e))
    }
    fn hashes(&self) -> Value {
        json!({"prefix_packed_sha256":sha256_bytes(&self.prefix),"period_packed_sha256":sha256_bytes(&self.period),"stop_packed_sha256":sha256_bytes(&self.stop),"prefix_config":self.prefix_config,"end_config":self.end_config})
    }
}

fn warm(a: &Args) -> Result<&CueCalibrationWarmstart> {
    a.cue_calibration_warmstart
        .as_ref()
        .ok_or_else(|| invalid("cue initializer absent").into())
}
fn panel(
    root: &Path,
    count: usize,
    integer: &IntegerRealizer,
    tok: &ByteBpeTokenizer,
    a: &Args,
) -> Result<Vec<Episode>> {
    let r = read_json(&root.join("report.json"))?;
    if r["source_policy"] != DATA_SCOPE
        || r["cue_origin_policy"] != "prospectively-authored-current-role-bytes/bound-byteBPE/2"
    {
        return Err(invalid("cue calibration natural source/cue policy differs").into());
    }
    let rows = if composition_panel(a).is_some() {
        let split = if root == a.development_panel {
            "development"
        } else if root == a.fresh_panel {
            "fresh"
        } else {
            return Err(invalid("unbound prospective panel root").into());
        };
        if !composition_report_matches(&read_json(&root.join("report.json"))?, split, count) {
            return Err(invalid("prospective layout report mismatch").into());
        }
        load_panel_with_layout(
            root,
            count,
            integer,
            tok,
            true,
            PanelLayout::ProspectiveAllBank,
        )?
    } else {
        load_natural_panel(root, count, integer, tok)?
    };
    validate_raw_cues(
        root,
        &rows,
        tok,
        &sha256_file(&a.native_artifact.join("tokenizer.json"))?,
    )?;
    Ok(rows)
}
fn immutable(inputs: &BTreeMap<String, String>, seals: &BTreeSet<PathBuf>) -> Result<()> {
    for (p, h) in inputs {
        if sha256_file(Path::new(p))? != *h {
            return Err(invalid("cue calibration input changed").into());
        }
    }
    for p in seals {
        report_output::verify(p)?;
    }
    Ok(())
}
fn frozen(source: &SourceRealizerWeights, receipt: &Value) -> Result<()> {
    if parameter_receipts(&source.parameters())? != *receipt {
        return Err(invalid("cue calibration frozen source changed").into());
    }
    Ok(())
}

// Rebind using actual compilation, never rewrite the original sealed donors.
fn save_chain(
    root: &Path,
    weights: &CueAngularWeights,
    parent: &NativeSourceRealizer,
    f: &Frozen,
) -> Result<()> {
    weights.save(&root.join("cue"))?;
    let c = parent.compile_cue_carrier(weights.native()?)?;
    let (p, e) = f.training(parent, &c)?;
    for name in ["prefix", "source-end"] {
        fs::create_dir(root.join(name))?;
    }
    fs::write(root.join("prefix/prefix-q4.bin"), &f.prefix)?;
    fs::write(
        root.join("prefix/native-metadata.json"),
        serde_json::to_vec_pretty(p.metadata())?,
    )?;
    fs::write(root.join("source-end/source-end-period-q4.bin"), &f.period)?;
    fs::write(root.join("source-end/source-end-stop-q4.bin"), &f.stop)?;
    fs::write(
        root.join("source-end/native-metadata.json"),
        serde_json::to_vec_pretty(e.metadata())?,
    )?;
    if p.packed_coefficients() != f.prefix
        || e.period_packed_coefficients() != f.period
        || e.stop_packed_coefficients() != f.stop
    {
        return Err(invalid("cue calibration rebound frozen payload changed").into());
    }
    Ok(())
}
fn load_chain<'a>(
    root: &Path,
    n: &'a IntegerRealizer,
) -> Result<(
    NativeCueCarrier<'a>,
    NativePrefixTransport<'a>,
    NativeSourceEndTransport<'a>,
)> {
    let c = cue_native_load(&root.join("cue"), n)?;
    let p = prefix_native_load(&root.join("prefix"), n, &c)?;
    let meta = read_json(&root.join("source-end/native-metadata.json"))?;
    let e = n.compile_source_end_transport(
        &c,
        &p,
        SourceEndAngularQ4::new(
            serde_json::from_value(meta["potential"].clone())?,
            &fs::read(root.join("source-end/source-end-period-q4.bin"))?,
            &fs::read(root.join("source-end/source-end-stop-q4.bin"))?,
        )
        .map_err(|e| invalid(e.to_string()))?,
    )?;
    if serde_json::to_value(e.metadata())? != meta {
        return Err(invalid("cue calibration native endpoint metadata differs").into());
    }
    Ok((c, p, e))
}

fn batch(
    indices: &[usize],
    es: &[Episode],
    source: &SourceRealizerWeights,
    parent: &NativeSourceRealizer,
    integer: Option<&IntegerRealizer>,
    weights: &CueAngularWeights,
    f: &Frozen,
    require_nonzero: bool,
    a: &Args,
    start: Instant,
) -> Result<Batch> {
    let began = Instant::now();
    let prepared = source.prepare(parent)?;
    let c = parent.compile_cue_carrier(weights.native()?)?;
    let (p, e) = f.training(parent, &c)?;
    let independent = integer
        .map(|n| -> Result<_> {
            let c = n.compile_cue_carrier(weights.native()?)?;
            let (p, e) = f.integer(n, &c)?;
            Ok((c, p, e))
        })
        .transpose()?;
    let params = weights.parameters();
    let parentvars = source.parameters();
    let mut gradients = BTreeMap::<String, Tensor>::new();
    let mut rows = Vec::new();
    let mut mean = 0.;
    let mut positions = 0;
    for &i in indices {
        let row = es.get(i).ok_or_else(|| invalid("cue batch index absent"))?;
        let segments = row.segments()?;
        let mut ce = 0.;
        for (step, &target) in row.target.iter().enumerate() {
            deadline(a, start)?;
            let out = prepared.loss_bank_cue_source_end(
                &segments,
                &row.packet.query_ids,
                &row.target[..step],
                target,
                weights,
                &c,
                &p,
                &e,
            )?;
            if let (Some(n), Some((ic, ip, ie))) = (integer, independent.as_ref()) {
                if out.trace
                    != n.read_bank_with_source_end_transport(
                        &segments,
                        &row.packet.query_ids,
                        &row.target[..step],
                        ic,
                        ip,
                        ie,
                    )?
                {
                    return Err(
                        invalid("cue calibration independent full native trace differs").into(),
                    );
                }
            }
            let native_ce = -out.target_probability.ln();
            let scalar = f64::from(out.loss.to_scalar::<f32>()?);
            if !native_ce.is_finite()
                || !scalar.is_finite()
                || (scalar - native_ce).abs() > 1e-4 + 1e-5 * native_ce.abs()
            {
                return Err(invalid(
                    "cue calibration authoritative final native likelihood differs",
                )
                .into());
            }
            ce += native_ce;
            positions += 1;
            let store = (&out.loss * scale(indices.len(), row.target.len())?)?.backward()?;
            if parentvars
                .values()
                .any(|v| store.get(v.as_tensor()).is_some())
            {
                return Err(invalid("cue calibration connects frozen parent Var").into());
            }
            for (name, var) in &params {
                if let Some(g) = store.get(var.as_tensor()) {
                    let g = g.detach();
                    let next = if let Some(old) = gradients.remove(name) {
                        (&old + &g)?.detach()
                    } else {
                        g
                    };
                    gradients.insert(name.clone(), next);
                }
            }
        }
        mean += ce / row.target.len() as f64 / indices.len() as f64;
        rows.push(json!({"id":row.packet.id,"target_positions":row.target.len(),"native_mean_token_ce":ce/row.target.len() as f64}));
    }
    let mut sq = 0.;
    let mut stats = BTreeMap::new();
    for (name, g) in &gradients {
        let v = g.flatten_all()?.to_vec1::<f32>()?;
        if v.iter().any(|x| !x.is_finite()) {
            return Err(invalid("cue calibration nonfinite coefficient gradient").into());
        }
        sq += v.iter().map(|x| f64::from(*x).powi(2)).sum::<f64>();
        stats.insert(
            name.clone(),
            json!({"elements":v.len(),"finite":true,"nonzero":v.iter().filter(|x|**x!=0.).count()}),
        );
    }
    if gradients.len() != params.len() || !sq.is_finite() || (require_nonzero && sq <= 0.) {
        return Err(invalid(
            "cue calibration coefficient family absent/nonfinite/nonzero requirement failed",
        )
        .into());
    }
    Ok(Batch {
        gradients,
        report: json!({"episodes":indices.len(),"episode_indices":indices,"target_positions":positions,"native_equal_episode_ce":mean,"gradient_global_norm":sq.sqrt(),"gradient_nonzero_observed":sq>0.,"nonzero_required":require_nonzero,"gradient_families":stats,"parent_gradient_graph_absent":true,"independent_native_trace_parity":integer.is_some(),"frozen_payloads":f.hashes(),"rows":rows,"elapsed_seconds":began.elapsed().as_secs_f64(),"active_families":FAMILIES,"objective":"equalepisode fullanswer+EOS ordinary final native alias CE; no floor; labels after full native read","gradient_scope":"cue coefficients at factual present angular bins only; encoder/roots/prefix/endpoint/factualSource argmax stopped"}),
    })
}

fn checkpoint(
    step: usize,
    prior: &[Value],
    weights: &CueAngularWeights,
    source: &SourceRealizerWeights,
    parent: &NativeSourceRealizer,
    integer: &IntegerRealizer,
    f: &Frozen,
    es: &[Episode],
    a: &Args,
    start: Instant,
) -> Result<Value> {
    deadline(a, start)?;
    if directory_bytes(&a.out)?.saturating_add(80 * 1024 * 1024) > a.maximum_report_bytes {
        return Err(invalid("cue checkpoint storage reserve reached").into());
    }
    let root = a.out.join(format!("checkpoint-{step:04}"));
    report_output::claim(&root)?;
    let result = (|| -> Result<Value> {
        save_chain(&root, weights, parent, f)?;
        let restored = CueAngularWeights::load(&root.join("cue"), parent, &a.native_artifact)?;
        if parameter_receipts(&restored.parameters())? != parameter_receipts(&weights.parameters())?
        {
            return Err(invalid("cue calibration shadow reload differs").into());
        }
        let (c, p, e) = load_chain(&root, integer)?;
        let canonical = source_end_fit::canonical(integer, &c, &p, &e, es, a, start)?;
        write_json(&root, "canonical.json", &canonical)?;
        let mut comparisons = Vec::new();
        for old in prior {
            let previous = old["updates"]
                .as_u64()
                .ok_or_else(|| invalid("prior cue checkpoint step absent"))?;
            let report = read_json(
                &a.out
                    .join(format!("checkpoint-{previous:04}/canonical.json")),
            )?;
            comparisons
                .push(json!({"prior_updates":previous,"comparison":compare(&report,&canonical)?}));
        }
        write_json(&root, "comparisons.json", &json!(comparisons))?;
        let receipt = json!({"updates":step,"native_equal_episode_ce":canonical["native_equal_episode_ce"],"zero_support_positions":canonical["zero_support_positions"],"cue_parameter_receipts":parameter_receipts(&weights.parameters())?,"cue_native_metadata_sha256":sha256_file(&root.join("cue/native-metadata.json"))?,"cue_packed_sha256":sha256_file(&root.join("cue/cue-q4.bin"))?,"rebound_prefix_metadata_sha256":sha256_file(&root.join("prefix/native-metadata.json"))?,"rebound_end_metadata_sha256":sha256_file(&root.join("source-end/native-metadata.json"))?,"frozen_payloads":f.hashes(),"canonical_sha256":sha256_file(&root.join("canonical.json"))?,"source_parameter_receipts":parameter_receipts(&source.parameters())?,"native_only_reload":true});
        write_json(&root, "receipt.json", &receipt)?;
        Ok(receipt)
    })();
    if let Err(e) = &result {
        write_json(
            &root,
            "failure.json",
            &json!({"error":e.to_string(),"updates":step}),
        )?;
    }
    report_output::seal(&root)?;
    report_output::verify(&root)?;
    result
}

// Offline scorer only: relation labels identify truth after native predictions.
// They are never supplied to admission, cue scoring, loss or runtime selection.
fn causal_metrics(
    panel: &Path,
    episodes: &[Episode],
    canonical: &Value,
    generation: &Value,
) -> Result<Value> {
    let data = read_json(&panel.join("context-data.json"))?;
    let labels = data["cases"]
        .as_array()
        .ok_or_else(|| invalid("metric labels absent"))?;
    let cr = canonical["rows"]
        .as_array()
        .ok_or_else(|| invalid("metric canonical absent"))?;
    let gr = generation["rows"]
        .as_array()
        .ok_or_else(|| invalid("metric generation absent"))?;
    if labels.len() != episodes.len() || cr.len() != episodes.len() || gr.len() != episodes.len() {
        return Err(invalid("metric row counts differ").into());
    }
    let mut rows = Vec::new();
    let mut pairs = BTreeMap::<String, Vec<Value>>::new();
    for (((label, e), c), g) in labels.iter().zip(episodes).zip(cr).zip(gr) {
        if label["id"] != e.packet.id || c["id"] != label["id"] || g["id"] != label["id"] {
            return Err(invalid("metric row identities differ").into());
        }
        let relation = match label["query_role"].as_str() {
            Some("job") => 1,
            Some("where") => 2,
            _ => return Err(invalid("metric query role unsupported").into()),
        };
        let expected = e
            .packet
            .segments
            .iter()
            .filter_map(|s| match s {
                Segment::Source {
                    record,
                    commit,
                    relation: r,
                    ..
                } if *r == relation => Some(json!({"record":record,"commit":commit})),
                _ => None,
            })
            .collect::<Vec<_>>();
        if expected.len() != 1 {
            return Err(invalid("metric truth requires unique role source").into());
        }
        let first = &c["tokens"][0];
        let actual = match first["source_end"]["selected_bank_index"].as_u64() {
            Some(i) => {
                let map = first["native"]["candidate_mapping"]
                    .as_array()
                    .ok_or_else(|| invalid("metric native candidate mapping absent"))?;
                let entry = map
                    .get(i as usize)
                    .ok_or_else(|| invalid("metric selected candidate outside bank"))?;
                if entry["bank_index"] != i {
                    return Err(invalid("metric bank index differs").into());
                }
                let o = &entry["occurrence"];
                if !o["record"].is_u64() || !o["commit"].is_u64() {
                    return Err(invalid("metric typed occurrence absent").into());
                }
                json!({"record":o["record"],"commit":o["commit"]})
            }
            None => Value::Null,
        };
        let first_target = *e
            .target
            .first()
            .ok_or_else(|| invalid("metric target absent"))?;
        let first_emitted = g["generated_ids_including_eos"][0]
            .as_u64()
            .ok_or_else(|| invalid("metric first emitted token absent"))?;
        let complete = g["accepted_complete_answer"]
            .as_bool()
            .ok_or_else(|| invalid("metric acceptance absent"))?;
        let row = json!({"id":e.packet.id,"pair_id":label["pair_id"],"query_role":label["query_role"],"expected_source_labels_only":expected[0],"actual_first_canonical_factual_source":actual,"first_canonical_source_correct":actual==expected[0],"first_emitted_token_id":first_emitted,"first_target_token_label_only":first_target,"first_emitted_token_exact_correct":first_emitted==u64::from(first_target),"complete_own_prefix":complete});
        let pair = label["pair_id"]
            .as_str()
            .ok_or_else(|| invalid("metric pair absent"))?;
        pairs.entry(pair.into()).or_default().push(row.clone());
        rows.push(row);
    }
    if pairs.values().any(|p| p.len() != 2) {
        return Err(invalid("metric pair width differs").into());
    }
    let mut counts = serde_json::Map::new();
    for key in [
        "first_canonical_source_correct",
        "first_emitted_token_exact_correct",
        "complete_own_prefix",
    ] {
        counts.insert(
            key.into(),
            json!(rows.iter().filter(|r| r[key] == true).count()),
        );
        counts.insert(
            format!("both_paired_{key}"),
            json!(pairs
                .values()
                .filter(|p| p.iter().all(|r| r[key] == true))
                .count()),
        );
    }
    Ok(
        json!({"rows":rows,"counts":counts,"scope":"scorer-only typed role-to-record truth; first canonical factual source, actual first emitted token and complete own-prefix are distinct; frozen endpoint bytes do not freeze corrections when factual source changes"}),
    )
}
fn causal_comparison(parent: &Value, selected: &Value) -> Result<Value> {
    let old = parent["rows"]
        .as_array()
        .ok_or_else(|| invalid("parent metric rows absent"))?;
    let new = selected["rows"]
        .as_array()
        .ok_or_else(|| invalid("selected metric rows absent"))?;
    if old.len() != new.len() {
        return Err(invalid("metric comparison count differs").into());
    }
    let mut changes = serde_json::Map::new();
    for key in [
        "first_canonical_source_correct",
        "first_emitted_token_exact_correct",
        "complete_own_prefix",
    ] {
        let mut gains = Vec::new();
        let mut losses = Vec::new();
        for (a, b) in old.iter().zip(new) {
            if a["id"] != b["id"] {
                return Err(invalid("metric comparison identities differ").into());
            }
            match (a[key].as_bool(), b[key].as_bool()) {
                (Some(false), Some(true)) => gains.push(b["id"].clone()),
                (Some(true), Some(false)) => losses.push(b["id"].clone()),
                (Some(_), Some(_)) => {}
                _ => return Err(invalid("metric boolean absent").into()),
            }
        }
        changes.insert(key.into(),json!({"gains":gains,"losses":losses,"parent_count":parent["counts"][key],"selected_count":selected["counts"][key],"parent_both_paired":parent["counts"][format!("both_paired_{key}")],"selected_both_paired":selected["counts"][format!("both_paired_{key}")]}));
    }
    Ok(Value::Object(changes))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Authorization {
    schema: String,
    fit_admitted: bool,
    parent_frozen: bool,
    active_families: String,
    admission_report_sha256: String,
    development_manifest_sha256: String,
    fresh_manifest_sha256: String,
    trusted_binding_sha256: String,
    initial_cue_metadata_sha256: String,
    cue_calibration_warmstart: CueCalibrationWarmstart,
    frozen_payloads: Value,
    updates: usize,
    batch_episodes: usize,
    maximum_fit_seconds: u64,
}
fn admission_matches(
    r: &Value,
    a: &Args,
    trusted: &str,
    initial: &str,
    receipts: &Value,
    f: &Frozen,
    inputs: &BTreeMap<String, String>,
) -> bool {
    r["schema"] == "uor-r4.geometric-cue-calibration/1"
        && r["mode"] == "cue-calibration-broadbatch"
        && r["status"] == "completed"
        && r["optimizer_updates"] == 0
        && r["cases"] == 128
        && r["active_families"] == FAMILIES
        && r["parent_frozen"] == true
        && r["gradient_report"]["parent_gradient_graph_absent"] == true
        && r["gradient_report"]["independent_native_trace_parity"] == true
        && r["gradient_report"]["gradient_nonzero_observed"] == true
        && r["development_manifest_sha256"] == a.development_manifest_sha256
        && r["fresh_manifest_sha256"] == a.fresh_manifest_sha256
        && r["trusted_binding_sha256"] == trusted
        && r["initial_cue_metadata_sha256"] == initial
        && r["frozen_source_receipts"] == *receipts
        && r["frozen_payloads"] == f.hashes()
        && serde_json::to_value(inputs).ok() == Some(r["input_manifests_sha256"].clone())
        && r["B8_calibration"]["initial_cue_metadata_sha256"] == initial
        && r["B8_calibration"]["trusted_binding_sha256"] == trusted
        && r["B8_calibration"]["frozen_payloads"] == f.hashes()
        && r["B8_calibration"]["development_manifest_sha256"] == a.development_manifest_sha256
        && r["B8_calibration"]["cue_calibration_warmstart"] == r["cue_calibration_warmstart"]
        && serde_json::to_value(&a.cue_calibration_warmstart).ok()
            == Some(r["cue_calibration_warmstart"].clone())
        && r["B8_calibration"]["batch_episodes"] == BATCH
        && r["B8_calibration"]["independent_native_trace_parity"] == true
        && r["B8_calibration"]["measured_b8_seconds"]
            .as_f64()
            .is_some_and(|v| v.is_finite() && v > 0.)
}

pub(super) fn run(a: &Args, start: Instant) -> Result<Value> {
    validate(a)?;
    if let Some(root) = &a.cue_discrete_completion_root {
        return discrete::complete(a, start, root);
    }
    let w = warm(a)?;
    let pr = a
        .frozen_prefix_bundle
        .as_ref()
        .ok_or_else(|| invalid("frozen prefix donor absent"))?;
    let mut inputs = BTreeMap::new();
    let mut seals = BTreeSet::new();
    for (root, expected) in [
        (&a.development_panel, &a.development_manifest_sha256),
        (&a.fresh_panel, &a.fresh_manifest_sha256),
    ] {
        report_output::verify(root)?;
        if sha256_file(&root.join("manifest.json"))? != *expected {
            return Err(invalid("cue calibration panel manifest differs").into());
        }
    }
    for root in [
        &a.source_weights,
        &a.native_artifact,
        &a.development_panel,
        &a.fresh_panel,
        &w.initial_cue_bundle,
        pr,
        &w.frozen_end_bundle,
    ] {
        let seal = nearest_seal(root)?;
        report_output::verify(&seal)?;
        inputs.insert(
            seal.join("manifest.json").to_string_lossy().into_owned(),
            sha256_file(&seal.join("manifest.json"))?,
        );
        seals.insert(seal);
    }
    for (root, name, expected) in [
        (
            &w.initial_cue_bundle,
            "native-metadata.json",
            &w.initial_cue_metadata_sha256,
        ),
        (
            &w.initial_cue_bundle,
            "cue-q4.bin",
            &w.initial_cue_packed_sha256,
        ),
        (
            pr,
            "native-metadata.json",
            a.frozen_prefix_native_metadata_sha256
                .as_ref()
                .ok_or_else(|| invalid("prefix metadata SHA absent"))?,
        ),
        (
            pr,
            "prefix-q4.bin",
            a.frozen_prefix_packed_sha256
                .as_ref()
                .ok_or_else(|| invalid("prefix packed SHA absent"))?,
        ),
        (
            &w.frozen_end_bundle,
            "native-metadata.json",
            &w.frozen_end_metadata_sha256,
        ),
        (
            &w.frozen_end_bundle,
            "source-end-period-q4.bin",
            &w.frozen_end_period_packed_sha256,
        ),
        (
            &w.frozen_end_bundle,
            "source-end-stop-q4.bin",
            &w.frozen_end_stop_packed_sha256,
        ),
    ] {
        let path = root.join(name);
        if sha256_file(&path)? != *expected {
            return Err(invalid("cue calibration initializer/frozen hash differs").into());
        }
        inputs.insert(path.to_string_lossy().into_owned(), expected.clone());
    }
    let trusted = sha256_file(&a.trusted_native_binding)?;
    inputs.insert(
        a.trusted_native_binding.to_string_lossy().into_owned(),
        trusted.clone(),
    );
    let binding: NativeArtifactBinding =
        serde_json::from_slice(&read_capped(&a.trusted_native_binding)?)?;
    let integer = IntegerRealizer::load_native(&a.native_artifact, &binding)?;
    let bytes = fs::read(a.native_artifact.join("tokenizer.json"))?;
    let tok = ByteBpeTokenizer::from_tokenizer_json_bytes(&bytes)
        .ok_or_else(|| invalid("cue calibration ByteBPE absent"))?;
    let source = SourceRealizerWeights::load_source(&a.source_weights, &bytes)?;
    let identity: ConsumerIdentity = serde_json::from_value(
        read_json(&a.native_artifact.join("metadata.json"))?["identity"].clone(),
    )?;
    let parent = NativeSourceRealizer::load(&a.native_artifact, &source, &identity)?;
    if parent.artifact_binding()? != binding {
        return Err(invalid("cue calibration source/native binding differs").into());
    }
    let receipts = parameter_receipts(&source.parameters())?;
    let donor = prefix_training_cue_load(&w.initial_cue_bundle, &parent)?;
    let weights = CueAngularWeights::from_native(&parent, &a.native_artifact, &donor)?;
    let cue = cue_native_load(&w.initial_cue_bundle, &integer)?;
    let prefix = prefix_native_load(pr, &integer, &cue)?;
    if Some(donor.metadata().potential.mode) != a.cue_score_mode
        || Some(cue.metadata().potential.mode) != a.cue_score_mode
        || Some(prefix.metadata().potential.mode) != a.prefix_score_mode
    {
        return Err(
            invalid("actual cue/prefix donor modes differ from declared configuration").into(),
        );
    }
    let end_meta = read_json(&w.frozen_end_bundle.join("native-metadata.json"))?;
    let f = Frozen {
        prefix_config: prefix.metadata().potential,
        prefix: fs::read(pr.join("prefix-q4.bin"))?,
        end_config: serde_json::from_value(end_meta["potential"].clone())?,
        period: fs::read(w.frozen_end_bundle.join("source-end-period-q4.bin"))?,
        stop: fs::read(w.frozen_end_bundle.join("source-end-stop-q4.bin"))?,
    };
    if f.end_config.mode != SourceEndScoreMode::DirectedRelative {
        return Err(invalid("cue calibration endpoint mode differs").into());
    }
    let (_, end) = f.integer(&integer, &cue)?;
    if serde_json::to_value(end.metadata())? != end_meta {
        return Err(invalid("cue calibration original endpoint chain differs").into());
    }
    let (tp, te) = f.training(&parent, &donor)?;
    if serde_json::to_value(tp.metadata())? != serde_json::to_value(prefix.metadata())?
        || serde_json::to_value(te.metadata())? != end_meta
    {
        return Err(invalid("cue calibration training/independent donor metadata differs").into());
    }
    let evidence = if let Some(q) = &a.cue_quantum_probe {
        if sha256_file(&q.evidence_receipt)? != q.evidence_receipt_sha256 {
            return Err(invalid("quantum audit receipt SHA differs").into());
        }
        let v = read_json(&q.evidence_receipt)?;
        if v["schema"] != "uor-r4.cue-quantization-audit/1"
            || v["fit_report"]["selected_updates"] != 0
            || v["fit_report"]["updates"] != 64
            || v["shadow"]["packed_equal_all64"] != true
            || v["shadow"]["coefficients"] != weights.config().coefficient_count()?
            || v["recommended_next_discriminator"]["coefficient"]["index"] != q.coefficient_index
            || v["recommended_next_discriminator"]["coefficient"]["parent"]
                != f64::from(q.initial_quarters) / 4.
            || v["inputs_sha256"]["initial-chain/cue/cue-source-f32.bin"]
                != parameter_receipts(&weights.parameters())?["cue.coefficients"]
                    ["f32_le_bits_sha256"]
        {
            return Err(
                invalid("quantum audit source/coordinate/warm-shadow evidence differs").into(),
            );
        }
        let donor_seal = nearest_seal(&q.evidence_receipt).ok();
        if let Some(seal) = donor_seal {
            report_output::verify(&seal)?;
            seals.insert(seal);
        }
        inputs.insert(
            q.evidence_receipt.to_string_lossy().into_owned(),
            q.evidence_receipt_sha256.clone(),
        );
        Some(v)
    } else {
        None
    };
    let (development_rows, evaluation_rows) = panel_counts(a);
    if composition_panel(a).is_some()
        && !composition_report_matches(
            &read_json(&a.development_panel.join("report.json"))?,
            "development",
            development_rows,
        )
    {
        return Err(invalid("prospective development panel report differs").into());
    }
    if composition_panel(a).is_some()
        && !composition_report_matches(
            &read_json(&a.fresh_panel.join("report.json"))?,
            "fresh",
            evaluation_rows,
        )
    {
        return Err(invalid("prospective evaluation panel report differs before learning").into());
    }
    let development = panel(&a.development_panel, development_rows, &integer, &tok, a)?;
    if a.cue_credit_audit.is_some() {
        return credit_audit::run(
            a,
            start,
            &source,
            &parent,
            &integer,
            &weights,
            &f,
            &development,
            &inputs,
            &seals,
            &receipts,
        );
    }
    let initial = a.out.join("initial-chain");
    report_output::claim(&initial)?;
    save_chain(&initial, &weights, &parent, &f)?;
    let restored = CueAngularWeights::load(&initial.join("cue"), &parent, &a.native_artifact)?;
    if parameter_receipts(&weights.parameters())? != parameter_receipts(&restored.parameters())? {
        return Err(invalid("cue calibration baseline shadow reload differs").into());
    }
    let (ic, ip, ie) = load_chain(&initial, &integer)?;
    let baseline = source_end_fit::canonical(&integer, &ic, &ip, &ie, &development, a, start)?;
    let donor_baseline =
        source_end_fit::canonical(&integer, &cue, &prefix, &end, &development, a, start)?;
    let parent_gen =
        source_end_fit::generation(&integer, &ic, &ip, &ie, &development, &tok, a, start)?;
    let donor_gen =
        source_end_fit::generation(&integer, &cue, &prefix, &end, &development, &tok, a, start)?;
    if baseline != donor_baseline
        || parent_gen != donor_gen
        || baseline["native_equal_episode_ce"].is_null()
    {
        return Err(
            invalid("cue calibration exact complete baseline replay differs/nonfinite").into(),
        );
    }
    write_json(
        &initial,
        "fidelity.json",
        &json!({"canonical_equal":true,"own_prefix_generation_equal":true,"signed_quarter_roundtrip":true,"frozen_payloads":f.hashes()}),
    )?;
    report_output::seal(&initial)?;
    report_output::verify(&initial)?;
    let initial_sha = sha256_file(&initial.join("cue/native-metadata.json"))?;
    write_json(&a.out, "initial-canonical.json", &baseline)?;
    write_json(&a.out, "development-parent-generation.json", &parent_gen)?;
    write_json(
        &a.out,
        "parent-query-pair-diagnostics.json",
        &source_end_fit::natural_pair_diagnostics(&a.development_panel, &baseline, &parent_gen)?,
    )?;
    if a.mode == "cue-calibration-quantum-probe" {
        return quantum_probe(
            a,
            start,
            &source,
            &parent,
            &integer,
            &tok,
            &weights,
            &f,
            &development,
            &baseline,
            &parent_gen,
            &inputs,
            &seals,
            &receipts,
            evidence
                .as_ref()
                .ok_or_else(|| invalid("probe evidence absent"))?,
        );
    }
    if a.mode == "cue-calibration-discrete-fit" {
        return discrete::run(
            a,
            start,
            &source,
            &parent,
            &integer,
            &tok,
            &weights,
            &f,
            &development,
            &baseline,
            &parent_gen,
            &inputs,
            &seals,
            &receipts,
        );
    }
    let mut projection = source_end_fit::project_storage(&baseline, &parent_gen, a)?;
    // This driver additionally retains both 32-row diagnostic canonical traces,
    // and all 64 recoverable cue shadows; do not reuse endpoint-only accounting.
    let extra = serde_json::to_vec(&baseline)?
        .len()
        .div_ceil(2)
        .saturating_add(64 * (weights.native()?.packed_coefficients().len() + 64 * 1024));
    let projected = projection["observed_shape_full_cap_bytes"]
        .as_u64()
        .ok_or_else(|| invalid("cue storage projection absent"))?;
    projection["cue_additional_canonical_and_recovery_reserve_bytes"] = json!(extra);
    projection["observed_shape_full_cap_bytes"] = json!(projected.saturating_add(extra as u64));
    write_json(&a.out, "storage-projection.json", &projection)?;
    let common = json!({"schema":"uor-r4.geometric-cue-calibration/1","mode":a.mode,"status":"completed","cases":128,"parent_frozen":true,"active_families":FAMILIES,"cue_calibration_warmstart":w,"initial_cue_metadata_sha256":initial_sha,"development_manifest_sha256":a.development_manifest_sha256,"fresh_manifest_sha256":a.fresh_manifest_sha256,"trusted_binding_sha256":trusted,"frozen_source_receipts":receipts,"frozen_payloads":f.hashes(),"input_manifests_sha256":inputs,"data_scope":DATA_SCOPE,"fresh_scope":"exposed diagnostic only; predictions after ordinary development CE selector freezes","storage_projection":projection,"gradient_scope":"cue coefficient STE only; frozen encoder/roots/prefix/endpoint; hard Source branch stopped","initialization":"same exact signed-quarter native cue; no historical fractional optimizer resume"});
    if a.mode == "cue-calibration-broadbatch" {
        let prep = start.elapsed().as_secs_f64();
        let broad = batch(
            &(0..128).collect::<Vec<_>>(),
            &development,
            &source,
            &parent,
            Some(&integer),
            &weights,
            &f,
            true,
            a,
            start,
        )?
        .report;
        write_json(&a.out, "broadbatch.json", &broad)?;
        let before = parameter_receipts(&weights.parameters())?;
        let b8 = batch(
            &balanced_indices(0),
            &development,
            &source,
            &parent,
            Some(&integer),
            &weights,
            &f,
            false,
            a,
            start,
        )?
        .report;
        if before != parameter_receipts(&weights.parameters())? {
            return Err(invalid("cue zero-update probe changed shadows").into());
        }
        let measured = b8["elapsed_seconds"]
            .as_f64()
            .filter(|x| x.is_finite() && *x > 0.)
            .ok_or_else(|| invalid("cue B8 elapsed absent"))?;
        let b8_report = json!({"schema":"uor-r4.cue-calibration-B8/1","optimizer_updates":0,"batch_episodes":BATCH,"episode_indices":balanced_indices(0),"independent_native_trace_parity":true,"gradient_nonzero_observed":b8["gradient_nonzero_observed"],"measured_b8_seconds":measured,"projected64_batches_plus_reserve_seconds":measured*UPDATES as f64+240.,"preparation_seconds":prep,"full128_batch_seconds":broad["elapsed_seconds"],"gradient_report":b8,"initial_cue_metadata_sha256":initial_sha,"trusted_binding_sha256":trusted,"frozen_payloads":f.hashes(),"cue_calibration_warmstart":w,"development_manifest_sha256":a.development_manifest_sha256,"scope":"actual B8 backward including conservative independent parity; finite zero allowed only cost probe; no optimizer/admission"});
        write_json(&a.out, "B8-calibration.json", &b8_report)?;
        frozen(&source, &receipts)?;
        immutable(&inputs, &seals)?;
        let mut r = common;
        r["optimizer_updates"] = json!(0);
        r["gradient_report"] = broad;
        r["B8_calibration"] = b8_report;
        r["fit_admitted"] = json!(false);
        r["fresh_predictions"] = json!("NOT_RUN");
        r["elapsed_seconds"] = json!(start.elapsed().as_secs_f64());
        r["peak_rss_kib_linux"] = json!(peak_rss_kib());
        return Ok(r);
    }
    let admission = a
        .admission
        .as_ref()
        .ok_or_else(|| invalid("cue fit admission absent"))?;
    report_output::verify(admission)?;
    if Some(sha256_file(&admission.join("manifest.json"))?) != a.admission_manifest_sha256 {
        return Err(invalid("cue admission manifest differs").into());
    }
    let report = read_json(&admission.join("report.json"))?;
    let authpath = a
        .fit_authorization
        .as_ref()
        .ok_or_else(|| invalid("cue fit authorization absent"))?;
    let auth: Authorization = serde_json::from_slice(&read_capped(authpath)?)?;
    if report["source_commit"] != json!(option_env!("UOR_BUILD_SOURCE_COMMIT"))
        || report["executable_sha256"] != sha256_file(&executable()?.0)?
        || !admission_matches(&report, a, &trusted, &initial_sha, &receipts, &f, &inputs)
        || read_json(&admission.join("B8-calibration.json"))? != report["B8_calibration"]
        || auth.schema != "uor-r4.cue-calibration-fit-authorization/1"
        || !auth.fit_admitted
        || !auth.parent_frozen
        || auth.active_families != FAMILIES
        || auth.admission_report_sha256 != sha256_file(&admission.join("report.json"))?
        || auth.development_manifest_sha256 != a.development_manifest_sha256
        || auth.fresh_manifest_sha256 != a.fresh_manifest_sha256
        || auth.trusted_binding_sha256 != trusted
        || auth.initial_cue_metadata_sha256 != initial_sha
        || auth.cue_calibration_warmstart != *w
        || auth.frozen_payloads != f.hashes()
        || auth.updates != UPDATES
        || auth.batch_episodes != BATCH
        || auth.maximum_fit_seconds != a.maximum_seconds
    {
        return Err(invalid("cue calibration explicit admission/authorization differs").into());
    }
    inputs.insert(
        admission
            .join("manifest.json")
            .to_string_lossy()
            .into_owned(),
        sha256_file(&admission.join("manifest.json"))?,
    );
    seals.insert(admission.clone());
    inputs.insert(
        authpath.to_string_lossy().into_owned(),
        sha256_file(authpath)?,
    );
    let measured = report["B8_calibration"]["measured_b8_seconds"]
        .as_f64()
        .ok_or_else(|| invalid("admitted cue timing absent"))?;
    let remaining = measured * UPDATES as f64 + 240.;
    let projected = projection["observed_shape_full_cap_bytes"]
        .as_u64()
        .ok_or_else(|| invalid("cue storage projection absent"))?;
    if start.elapsed().as_secs_f64() + remaining > a.maximum_seconds as f64
        || projected.saturating_add(1024 * 1024) > a.maximum_report_bytes as u64
    {
        return Err(invalid("cue fit preoptimizer time/storage projection refused").into());
    }
    write_json(
        &a.out,
        "preoptimizer-projection.json",
        &json!({"optimizer_updates":0,"actual_admitted_b8_seconds":measured,"remaining_seconds":remaining,"storage_bytes":projected}),
    )?;
    let mut stages = vec![checkpoint(
        0,
        &[],
        &weights,
        &source,
        &parent,
        &integer,
        &f,
        &development,
        a,
        start,
    )?];
    if read_json(&a.out.join("checkpoint-0000/canonical.json"))? != baseline {
        return Err(invalid("cue fit independently reloaded zero checkpoint differs").into());
    }
    let mut opt = AdamW::new(
        weights.parameters().into_values().collect(),
        ParamsAdamW {
            lr: 0.003,
            beta1: 0.9,
            beta2: 0.999,
            eps: 1e-8,
            weight_decay: 0.,
        },
    )?;
    let initial_packed = weights.packed_coefficients()?;
    let mut first_cross = None;
    let mut batches = Vec::new();
    fs::create_dir(a.out.join("recovery"))?;
    for update in 0..UPDATES {
        deadline(a, start)?;
        frozen(&source, &receipts)?;
        let b = batch(
            &balanced_indices(update),
            &development,
            &source,
            &parent,
            None,
            &weights,
            &f,
            false,
            a,
            start,
        )?;
        if update == 0 {
            let actual = b.report["elapsed_seconds"]
                .as_f64()
                .ok_or_else(|| invalid("cue actual first batch timing absent"))?;
            if start.elapsed().as_secs_f64() + actual * UPDATES as f64 + 240.
                > a.maximum_seconds as f64
            {
                return Err(invalid("cue actual first batch projection refused optimizer1").into());
            }
            write_json(
                &a.out,
                "first-batch-projection.json",
                &json!({"optimizer_updates":0,"measured_batch_seconds":actual,"projected_remaining_seconds":actual*UPDATES as f64+240.}),
            )?;
        }
        let clip = cue_apply(&weights, &mut opt, b.gradients)?;
        weights.save(&a.out.join(format!("recovery/update-{:04}", update + 1)))?;
        if first_cross.is_none() && weights.packed_coefficients()? != initial_packed {
            first_cross = Some(update + 1);
        }
        batches.push(json!({"update":update+1,"clip_factor":clip,"cue_packed_sha256":sha256_bytes(&weights.packed_coefficients()?),"batch":b.report}));
        write_json(
            &a.out,
            "progress.json",
            &json!({"optimizer_updates":update+1,"batches":batches,"first_native_crossing":first_cross,"elapsed_seconds":start.elapsed().as_secs_f64()}),
        )?;
        if (update + 1) % 16 == 0 {
            stages.push(checkpoint(
                update + 1,
                &stages,
                &weights,
                &source,
                &parent,
                &integer,
                &f,
                &development,
                a,
                start,
            )?);
        }
    }
    let selected = select(&stages)?;
    let step = stages[selected]["updates"]
        .as_u64()
        .ok_or_else(|| invalid("cue selected step absent"))?;
    write_json(
        &a.out,
        "selection-before-fresh.json",
        &json!({"selected_updates":step,"criterion":"ordinary full128 native equalepisode answer+EOS CE including parent0;earliest strict minimum","fresh_predictions_before_selection":0}),
    )?;
    let (sc, sp, se) = load_chain(&a.out.join(format!("checkpoint-{step:04}")), &integer)?;
    let selected_gen =
        source_end_fit::generation(&integer, &sc, &sp, &se, &development, &tok, a, start)?;
    write_json(
        &a.out,
        "development-selected-generation.json",
        &selected_gen,
    )?;
    let selected_can = read_json(&a.out.join(format!("checkpoint-{step:04}/canonical.json")))?;
    write_json(
        &a.out,
        "selected-query-pair-diagnostics.json",
        &source_end_fit::natural_pair_diagnostics(
            &a.development_panel,
            &selected_can,
            &selected_gen,
        )?,
    )?;
    let parent_metrics =
        causal_metrics(&a.development_panel, &development, &baseline, &parent_gen)?;
    let selected_metrics = causal_metrics(
        &a.development_panel,
        &development,
        &selected_can,
        &selected_gen,
    )?;
    write_json(
        &a.out,
        "development-causal-outcomes.json",
        &json!({"parent":parent_metrics,"selected":selected_metrics,"comparison":causal_comparison(&parent_metrics,&selected_metrics)?}),
    )?;
    let diagnostic = panel(&a.fresh_panel, 32, &integer, &tok, a)?;
    let oldgen = source_end_fit::generation(&integer, &ic, &ip, &ie, &diagnostic, &tok, a, start)?;
    let newgen = source_end_fit::generation(&integer, &sc, &sp, &se, &diagnostic, &tok, a, start)?;
    let oldcan = source_end_fit::canonical(&integer, &ic, &ip, &ie, &diagnostic, a, start)?;
    let newcan = source_end_fit::canonical(&integer, &sc, &sp, &se, &diagnostic, a, start)?;
    write_json(&a.out, "fresh-parent-generation.json", &oldgen)?;
    write_json(&a.out, "fresh-selected-generation.json", &newgen)?;
    write_json(&a.out, "fresh-parent-canonical.json", &oldcan)?;
    write_json(&a.out, "fresh-selected-canonical.json", &newcan)?;
    write_json(
        &a.out,
        "fresh-parent-query-pair-diagnostics.json",
        &source_end_fit::natural_pair_diagnostics(&a.fresh_panel, &oldcan, &oldgen)?,
    )?;
    write_json(
        &a.out,
        "fresh-selected-query-pair-diagnostics.json",
        &source_end_fit::natural_pair_diagnostics(&a.fresh_panel, &newcan, &newgen)?,
    )?;
    let old_metrics = causal_metrics(&a.fresh_panel, &diagnostic, &oldcan, &oldgen)?;
    let new_metrics = causal_metrics(&a.fresh_panel, &diagnostic, &newcan, &newgen)?;
    write_json(
        &a.out,
        "fresh-causal-outcomes.json",
        &json!({"parent":old_metrics,"selected":new_metrics,"comparison":causal_comparison(&old_metrics,&new_metrics)?}),
    )?;
    frozen(&source, &receipts)?;
    immutable(&inputs, &seals)?;
    let mut r = common;
    r["optimizer_updates"] = json!(UPDATES);
    r["batch_episodes"] = json!(BATCH);
    r["episode_visits"] = json!(4);
    r["batch_schedule"]=json!("fixed cyclic balanced4rows from each64-row half;two intact querypairs perhalf;4visits perrow");
    r["checkpoints"] = json!(stages);
    r["selected_updates"] = json!(step);
    r["first_native_packed_crossing_update"] = json!(first_cross);
    r["fresh_predictions_before_selection"] = json!(0);
    r["input_manifests_sha256"] = json!(inputs);
    r["elapsed_seconds"] = json!(start.elapsed().as_secs_f64());
    r["peak_rss_kib_linux"] = json!(peak_rss_kib());
    r["claim"]=json!("bounded cue-table calibration; exposed composition diagnostic; no seed-consistency/transfer/chat/energy qualification");
    Ok(r)
}

fn quantum_packed(original: &[u8], count: usize, q: &CueQuantumProbe, step: i8) -> Result<Vec<u8>> {
    use uor_r4_integer::geometric_potential_q4::{pack_coefficients, unpack_coefficients};
    let original_values =
        unpack_coefficients(count, original).map_err(|e| invalid(e.to_string()))?;
    let mut values = original_values.clone();
    let slot = values
        .get_mut(q.coefficient_index)
        .ok_or_else(|| invalid("quantum coordinate outside actual donor"))?;
    if *slot != q.initial_quarters || !matches!(step, -1 | 1) {
        return Err(invalid("quantum initial donor quarter/step differs").into());
    }
    *slot = slot
        .checked_add(step)
        .filter(|x| (-7..=7).contains(x))
        .ok_or_else(|| invalid("quantum legal quarter range exceeded"))?;
    if values
        .iter()
        .zip(&original_values)
        .filter(|(a, b)| a != b)
        .count()
        != 1
    {
        return Err(invalid("quantum altered more than one coefficient").into());
    }
    let packed = pack_coefficients(&values).map_err(|e| invalid(e.to_string()))?;
    if unpack_coefficients(count, &packed).map_err(|e| invalid(e.to_string()))? != values
        || packed.iter().zip(original).filter(|(a, b)| a != b).count() != 1
    {
        return Err(invalid("quantum packed one-quarter witness differs").into());
    }
    Ok(packed)
}

// A score audit of saved native predictions, never a substitute for native runs.
fn quantum_margin_changes(
    baseline: &Value,
    candidate: &Value,
    truth: &Value,
    q: &CueQuantumProbe,
    step: i8,
) -> Result<Value> {
    let old = baseline["rows"]
        .as_array()
        .ok_or_else(|| invalid("quantum baseline rows absent"))?;
    let new = candidate["rows"]
        .as_array()
        .ok_or_else(|| invalid("quantum candidate rows absent"))?;
    let labels = truth["rows"]
        .as_array()
        .ok_or_else(|| invalid("quantum truth rows absent"))?;
    if old.len() != new.len() || old.len() != labels.len() {
        return Err(invalid("quantum margin row count differs").into());
    }
    let lane = q.coefficient_index / 120;
    let bin = q.coefficient_index % 120;
    let mut rows = Vec::new();
    let mut touched_rows = 0;
    let mut touched_positions = 0;
    let mut checked_copy_positions = 0;
    for ((a, b), label) in old.iter().zip(new).zip(labels) {
        if a["id"] != b["id"] || a["id"] != label["id"] {
            return Err(invalid("quantum margin IDs differ").into());
        }
        let aa = a["tokens"]
            .as_array()
            .ok_or_else(|| invalid("quantum tokens absent"))?;
        let bb = b["tokens"]
            .as_array()
            .ok_or_else(|| invalid("quantum tokens absent"))?;
        if aa.len() != bb.len() {
            return Err(invalid("quantum canonical target count differs").into());
        }
        let mut row_touch = false;
        let mut first_record_scores = Vec::new();
        for (position, (ta, tb)) in aa.iter().zip(bb).enumerate() {
            let mapping = ta["native"]["candidate_mapping"]
                .as_array()
                .ok_or_else(|| invalid("quantum map absent"))?;
            if ta["native"]["candidate_mapping"] != tb["native"]["candidate_mapping"]
                || ta["cue_carrier"]["angular_indices"] != tb["cue_carrier"]["angular_indices"]
            {
                return Err(
                    invalid("quantum changed frozen occurrence/address representation").into(),
                );
            }
            let bins = ta["cue_carrier"]["angular_indices"][lane]
                .as_array()
                .ok_or_else(|| invalid("quantum lane absent"))?;
            let acts_a = ta["native"]["actions"]["actions"]
                .as_array()
                .ok_or_else(|| invalid("quantum native actions absent"))?;
            let acts_b = tb["native"]["actions"]["actions"]
                .as_array()
                .ok_or_else(|| invalid("quantum native actions absent"))?;
            if bins.len() != mapping.len()
                || acts_a.len() != mapping.len() + 2
                || acts_b.len() != acts_a.len()
            {
                return Err(invalid("quantum native Copy shape differs").into());
            }
            let mut records = BTreeMap::<(u64, u64), (i64, i64, bool)>::new();
            for (j, m) in mapping.iter().enumerate() {
                let x = acts_a[j]["score_q24"]
                    .as_i64()
                    .ok_or_else(|| invalid("quantum raw Copy absent"))?;
                let y = acts_b[j]["score_q24"]
                    .as_i64()
                    .ok_or_else(|| invalid("quantum raw Copy absent"))?;
                let touched = bins[j].as_u64() == Some(bin as u64);
                let expected = if touched {
                    i64::from(step) * (1i64 << 22)
                } else {
                    0
                };
                if y.checked_sub(x) != Some(expected) {
                    return Err(invalid(
                        "quantum native Copy delta does not match actual single address",
                    )
                    .into());
                }
                checked_copy_positions += 1;
                touched_positions += usize::from(touched);
                row_touch |= touched;
                let id = (
                    m["occurrence"]["record"]
                        .as_u64()
                        .ok_or_else(|| invalid("quantum record absent"))?,
                    m["occurrence"]["commit"]
                        .as_u64()
                        .ok_or_else(|| invalid("quantum commit absent"))?,
                );
                let v = records.entry(id).or_insert((i64::MIN, i64::MIN, false));
                v.0 = v.0.max(x);
                v.1 = v.1.max(y);
                v.2 |= touched;
            }
            if position == 0 {
                for ((record, commit), (x, y, touched)) in records {
                    first_record_scores.push(json!({"record":record,"commit":commit,"baseline_max_raw_copy_q24":x,"candidate_max_raw_copy_q24":y,"delta_q24":y-x,"touched_address":touched}));
                }
            }
        }
        touched_rows += usize::from(row_touch);
        let expected = &label["expected_source_labels_only"];
        let mut good = None;
        let mut other_old = i64::MIN;
        let mut other_new = i64::MIN;
        for r in &first_record_scores {
            let old = r["baseline_max_raw_copy_q24"]
                .as_i64()
                .ok_or_else(|| invalid("quantum record score absent"))?;
            let new = r["candidate_max_raw_copy_q24"]
                .as_i64()
                .ok_or_else(|| invalid("quantum record score absent"))?;
            if r["record"] == expected["record"] && r["commit"] == expected["commit"] {
                good = Some((old, new));
            } else {
                other_old = other_old.max(old);
                other_new = other_new.max(new);
            }
        }
        let (good_old, good_new) =
            good.ok_or_else(|| invalid("quantum true record missing from native map"))?;
        if other_old == i64::MIN || other_new == i64::MIN {
            return Err(invalid("quantum distractor record missing").into());
        }
        rows.push(json!({"id":a["id"],"touched":row_touch,"first_record_scores":first_record_scores,"baseline_correct_vs_distractor_margin_q24":good_old-other_old,"candidate_correct_vs_distractor_margin_q24":good_new-other_new,"margin_delta_q24":(good_new-other_new)-(good_old-other_old)}));
    }
    if touched_positions == 0 {
        return Err(invalid("quantum declared address not consumed").into());
    }
    Ok(
        json!({"rows":rows,"touched_rows":touched_rows,"touched_copy_positions":touched_positions,"checked_copy_positions":checked_copy_positions,"native_one_quarter_address_delta_verified":true,"scope":"actual canonical native outputs; raw Copy max per exact record/commit; role truth only after native read; terminals may change conditionally"}),
    )
}

#[allow(clippy::too_many_arguments)]
fn quantum_probe(
    a: &Args,
    start: Instant,
    source: &SourceRealizerWeights,
    parent: &NativeSourceRealizer,
    integer: &IntegerRealizer,
    tok: &ByteBpeTokenizer,
    weights: &CueAngularWeights,
    f: &Frozen,
    development: &[Episode],
    baseline: &Value,
    parent_gen: &Value,
    inputs: &BTreeMap<String, String>,
    seals: &BTreeSet<PathBuf>,
    receipts: &Value,
    evidence: &Value,
) -> Result<Value> {
    use uor_r4_integer::geometric_cue_carrier::CueAngularQ4;
    let q = a
        .cue_quantum_probe
        .as_ref()
        .ok_or_else(|| invalid("quantum configuration absent"))?;
    if evidence["inputs_sha256"]["initial-canonical.json"]
        != sha256_file(&a.out.join("initial-canonical.json"))?
    {
        return Err(invalid("quantum warm native baseline differs from audit evidence").into());
    }
    let original = weights.packed_coefficients()?;
    let count = weights.config().coefficient_count()?;
    let cue_receipts = parameter_receipts(&weights.parameters())?;
    let baseline_metrics = causal_metrics(&a.development_panel, development, baseline, parent_gen)?;
    let baseline_ce = baseline["native_equal_episode_ce"]
        .as_f64()
        .filter(|x| x.is_finite())
        .ok_or_else(|| invalid("quantum finite baseline CE absent"))?;
    let canonical_bytes = serde_json::to_vec(baseline)?.len();
    let generation_bytes = serde_json::to_vec(parent_gen)?.len();
    let observed_total = canonical_bytes
        .saturating_add(generation_bytes)
        .saturating_mul(7)
        .div_ceil(2)
        .saturating_add(16 * 1024 * 1024);
    if observed_total.saturating_add(1024 * 1024) > a.maximum_report_bytes {
        return Err(
            invalid("quantum observed-shape report projection exceeds admitted cap").into(),
        );
    }
    let projection = json!({"configured_report_cap_bytes":a.maximum_report_bytes,"observed_shape_total_bytes":observed_total,"initial_and_baseline_canonical_bytes":canonical_bytes,"observed_parent_generation_bytes":generation_bytes,"arms":3,"proposals":2,"maximum_generation_tokens":a.maximum_generation_tokens,"administrative_reserve_bytes":16*1024*1024,"scope":"baseline observed sizes; each proposal retains one canonical and one actual ownprefix trace; selected diagnostic32 after freeze; write_json enforces total256MiB actual cap with1MiB stop margin; no optimizer/recovery artifacts"});
    write_json(&a.out, "storage-projection.json", &projection)?;
    let mut arms = vec![
        json!({"name":"baseline","proposal":false,"optimizer_updates":0,"coefficient_quarters":q.initial_quarters,"native_equal_episode_ce":baseline_ce,"cue_packed_sha256":sha256_bytes(&original),"chain":"initial-chain","canonical":"initial-canonical.json","generation":"development-parent-generation.json","causal_outcomes":baseline_metrics}),
    ];
    let mut selected_name = "baseline";
    let mut best = baseline_ce;
    for (name, step) in [
        ("preferred", q.preferred_step),
        ("opposite", -q.preferred_step),
    ] {
        deadline(a, start)?;
        let packed = quantum_packed(&original, count, q, step)?;
        let cue = parent.compile_cue_carrier(
            CueAngularQ4::new(weights.config(), &packed).map_err(|e| invalid(e.to_string()))?,
        )?;
        let candidate = CueAngularWeights::from_native(parent, &a.native_artifact, &cue)?;
        if candidate.packed_coefficients()? != packed {
            return Err(invalid("quantum signed-quarter source reload differs").into());
        }
        let root = a.out.join(format!("arm-{name}"));
        report_output::claim(&root)?;
        save_chain(&root, &candidate, parent, f)?;
        let restored = CueAngularWeights::load(&root.join("cue"), parent, &a.native_artifact)?;
        if parameter_receipts(&candidate.parameters())?
            != parameter_receipts(&restored.parameters())?
        {
            return Err(invalid("quantum source shadow reload differs").into());
        }
        let (c, p, e) = load_chain(&root, integer)?;
        if c.packed_coefficients() != packed {
            return Err(invalid("quantum independent native cue differs").into());
        }
        let canonical = source_end_fit::canonical(integer, &c, &p, &e, development, a, start)?;
        let generation =
            source_end_fit::generation(integer, &c, &p, &e, development, tok, a, start)?;
        let metrics = causal_metrics(&a.development_panel, development, &canonical, &generation)?;
        let margins = quantum_margin_changes(baseline, &canonical, &baseline_metrics, q, step)?;
        write_json(&root, "canonical.json", &canonical)?;
        write_json(&root, "generation.json", &generation)?;
        write_json(
            &root,
            "causal-outcomes.json",
            &json!({"parent":baseline_metrics,"candidate":metrics,"comparison":causal_comparison(&baseline_metrics,&metrics)?}),
        )?;
        write_json(&root, "record-margin-changes.json", &margins)?;
        let ce = canonical["native_equal_episode_ce"]
            .as_f64()
            .filter(|x| x.is_finite());
        let receipt = json!({"name":name,"proposal":true,"optimizer_updates":0,"coefficient_index":q.coefficient_index,"coefficient_quarters":q.initial_quarters+step,"step_quarters":step,"native_equal_episode_ce":ce,"zero_support_positions":canonical["zero_support_positions"],"cue_packed_sha256":sha256_bytes(&packed),"all_other_coefficients_unchanged":true,"native_one_quarter_address_delta_verified":true,"independent_native_reload":true,"frozen_payloads":f.hashes(),"cue_native_metadata_sha256":sha256_file(&root.join("cue/native-metadata.json"))?,"rebound_prefix_metadata_sha256":sha256_file(&root.join("prefix/native-metadata.json"))?,"rebound_end_metadata_sha256":sha256_file(&root.join("source-end/native-metadata.json"))?,"causal_counts":metrics["counts"],"touched_rows":margins["touched_rows"]});
        write_json(&root, "receipt.json", &receipt)?;
        report_output::seal(&root)?;
        report_output::verify(&root)?;
        if ce.is_some_and(|x| x < best) {
            best = ce.ok_or_else(|| invalid("quantum selector CE absent"))?;
            selected_name = name;
        }
        arms.push(receipt);
    }
    write_json(
        &a.out,
        "selection-before-fresh.json",
        &json!({"selected_arm":selected_name,"criterion":"ordinary full128 native equalepisode answer+EOS CE; baseline-inclusive; earliest strict minimum in baseline/preferred/opposite order","fresh_predictions_before_selection":0,"optimizer_updates":0,"proposal_count":2}),
    )?;
    let selected_root = if selected_name == "baseline" {
        a.out.join("initial-chain")
    } else {
        a.out.join(format!("arm-{selected_name}"))
    };
    let (c, p, e) = load_chain(&selected_root, integer)?;
    let diagnostic = panel(&a.fresh_panel, 32, integer, tok, a)?;
    let (bc, bp, be) = load_chain(&a.out.join("initial-chain"), integer)?;
    let oldcan = source_end_fit::canonical(integer, &bc, &bp, &be, &diagnostic, a, start)?;
    let oldgen = source_end_fit::generation(integer, &bc, &bp, &be, &diagnostic, tok, a, start)?;
    let newcan = source_end_fit::canonical(integer, &c, &p, &e, &diagnostic, a, start)?;
    let newgen = source_end_fit::generation(integer, &c, &p, &e, &diagnostic, tok, a, start)?;
    let oldmetrics = causal_metrics(&a.fresh_panel, &diagnostic, &oldcan, &oldgen)?;
    let newmetrics = causal_metrics(&a.fresh_panel, &diagnostic, &newcan, &newgen)?;
    write_json(&a.out, "fresh-parent-canonical.json", &oldcan)?;
    write_json(&a.out, "fresh-selected-canonical.json", &newcan)?;
    write_json(&a.out, "fresh-parent-generation.json", &oldgen)?;
    write_json(&a.out, "fresh-selected-generation.json", &newgen)?;
    write_json(
        &a.out,
        "fresh-causal-outcomes.json",
        &json!({"parent":oldmetrics,"selected":newmetrics,"comparison":causal_comparison(&oldmetrics,&newmetrics)?}),
    )?;
    if parameter_receipts(&weights.parameters())? != cue_receipts {
        return Err(invalid("quantum original cue source mutated").into());
    }
    frozen(source, receipts)?;
    immutable(inputs, seals)?;
    Ok(
        json!({"schema":"uor-r4.geometric-cue-quantum-probe/1","mode":a.mode,"status":"completed","cases":128,"optimizer_updates":0,"proposal_count":2,"selected_arm":selected_name,"selected_native_equal_episode_ce":best,"arms":arms,"cue_quantum_probe":q,"cue_calibration_warmstart":a.cue_calibration_warmstart,"input_manifests_sha256":inputs,"evidence_source_commit":evidence["source_commit"],"evidence_executable_sha256":evidence["executable_sha256"],"evidence_receipt_sha256":q.evidence_receipt_sha256,"frozen_source_receipts":receipts,"frozen_payloads":f.hashes(),"storage_projection":projection,"fresh_predictions_before_selection":0,"elapsed_seconds":start.elapsed().as_secs_f64(),"peak_rss_kib_linux":peak_rss_kib(),"claim":"bounded single-address actual native quarter intervention; development selection plus exposed diagnostic; no learned-update/seed/transfer/chat/energy claim"}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    fn valid_args() -> Result<Args> {
        Ok(serde_json::from_value(
            json!({"mode":"cue-calibration-broadbatch","cue_score_mode":"DirectedRelative","prefix_score_mode":"DirectedRelative","source_weights":"s","native_artifact":"n","trusted_native_binding":"b","frozen_prefix_bundle":"p","frozen_prefix_native_metadata_sha256":"f".repeat(64),"frozen_prefix_packed_sha256":"0".repeat(64),"development_panel":"d","development_manifest_sha256":"d","fresh_panel":"f","fresh_manifest_sha256":"f","out":"o","maximum_seconds":300,"maximum_context_tokens":128,"maximum_generation_tokens":32,"maximum_report_bytes":134217728,"cue_calibration_warmstart":{"initial_cue_bundle":"c","initial_cue_metadata_sha256":"a".repeat(64),"initial_cue_packed_sha256":"b".repeat(64),"frozen_end_bundle":"e","frozen_end_metadata_sha256":"c".repeat(64),"frozen_end_period_packed_sha256":"d".repeat(64),"frozen_end_stop_packed_sha256":"e".repeat(64),"data_scope":DATA_SCOPE}}),
        )?)
    }
    #[test]
    fn credit_audit_requires_fixed_checkpoint_and_prospective_loader_profile() -> Result<()> {
        let mut a = valid_args()?;
        a.mode = "cue-calibration-credit-audit".into();
        let checkpoint = PathBuf::from("checkpoint");
        a.cue_credit_audit = Some(CueCreditAudit {
            checkpoint_root: checkpoint.clone(),
            checkpoint_manifest_sha256: "a".repeat(64),
            reference_canonical: "reference.json".into(),
            reference_canonical_sha256: "b".repeat(64),
            composition_panel: CueCompositionPanel {
                profile: COMPOSITION_PROFILE.into(),
                development_rows: 512,
                evaluation_rows: 128,
            },
        });
        let w = a
            .cue_calibration_warmstart
            .as_mut()
            .ok_or_else(|| invalid("test warm absent"))?;
        w.initial_cue_bundle = checkpoint.join("cue");
        w.frozen_end_bundle = checkpoint.join("source-end");
        a.frozen_prefix_bundle = Some(checkpoint.join("prefix"));
        validate(&a)?;
        assert_eq!(panel_counts(&a), (512, 128));
        assert!(composition_panel(&a).is_some());
        a.mode = "cue-calibration-broadbatch".into();
        assert!(validate(&a).is_err());
        a.mode = "cue-calibration-credit-audit".into();
        a.frozen_prefix_bundle = Some("other-prefix".into());
        assert!(validate(&a).is_err());
        a.frozen_prefix_bundle = Some(checkpoint.join("prefix"));
        a.cue_credit_audit
            .as_mut()
            .ok_or_else(|| invalid("test config absent"))?
            .composition_panel
            .development_rows = 128;
        assert!(validate(&a).is_err());
        Ok(())
    }
    #[test]
    fn explicit_cue_calibration_rejects_frozen_initializer_and_foreign_scope() -> Result<()> {
        let mut a = valid_args()?;
        validate(&a)?;
        a.frozen_cue_bundle = Some("ambiguous".into());
        assert!(validate(&a).is_err());
        a.frozen_cue_bundle = None;
        a.cue_calibration_warmstart
            .as_mut()
            .ok_or_else(|| invalid("test warmstart absent"))?
            .data_scope = "oracle-selected".into();
        assert!(validate(&a).is_err());
        a.cue_calibration_warmstart
            .as_mut()
            .ok_or_else(|| invalid("test warmstart absent"))?
            .data_scope = DATA_SCOPE.into();
        a.learned_source_weights = Some("ignored-donor".into());
        assert!(validate(&a).is_err());
        a.learned_source_weights = None;
        a.source_end_score_mode = Some(SourceEndScoreMode::DirectedRelative);
        assert!(validate(&a).is_err());
        a.source_end_score_mode = None;
        a.mode = "cue-broadbatch".into();
        assert!(validate(&a).is_err());
        Ok(())
    }
    #[test]
    fn admission_rejects_stale_input_or_donor_and_unbound_b8() -> Result<()> {
        let a = valid_args()?;
        let f = Frozen {
            prefix_config: PrefixAngularConfig {
                heads: 1,
                lanes_per_head: 1,
                mode: PrefixScoreMode::DirectedRelative,
            },
            prefix: vec![0; 60],
            end_config: SourceEndAngularConfig {
                heads: 1,
                lanes_per_head: 1,
                mode: SourceEndScoreMode::DirectedRelative,
            },
            period: vec![0; 60],
            stop: vec![0; 60],
        };
        let receipts = json!({"frozen":"bits"});
        let inputs = BTreeMap::from([("prefix/native-metadata.json".to_string(), "f".repeat(64))]);
        let mut r = json!({"schema":"uor-r4.geometric-cue-calibration/1","mode":"cue-calibration-broadbatch","status":"completed","optimizer_updates":0,"cases":128,"active_families":FAMILIES,"parent_frozen":true,"gradient_report":{"parent_gradient_graph_absent":true,"independent_native_trace_parity":true,"gradient_nonzero_observed":true},"development_manifest_sha256":a.development_manifest_sha256,"fresh_manifest_sha256":a.fresh_manifest_sha256,"trusted_binding_sha256":"trusted","initial_cue_metadata_sha256":"initial","frozen_source_receipts":receipts,"frozen_payloads":f.hashes(),"input_manifests_sha256":inputs,"cue_calibration_warmstart":a.cue_calibration_warmstart,"B8_calibration":{"batch_episodes":BATCH,"independent_native_trace_parity":true,"measured_b8_seconds":1.,"initial_cue_metadata_sha256":"initial","trusted_binding_sha256":"trusted","frozen_payloads":f.hashes(),"development_manifest_sha256":a.development_manifest_sha256,"cue_calibration_warmstart":a.cue_calibration_warmstart}});
        assert!(admission_matches(
            &r, &a, "trusted", "initial", &receipts, &f, &inputs
        ));
        let original = r.clone();
        r["input_manifests_sha256"]["prefix/native-metadata.json"] = json!("stale");
        assert!(!admission_matches(
            &r, &a, "trusted", "initial", &receipts, &f, &inputs
        ));
        r = original.clone();
        r["cue_calibration_warmstart"]["frozen_end_metadata_sha256"] = json!("9".repeat(64));
        assert!(!admission_matches(
            &r, &a, "trusted", "initial", &receipts, &f, &inputs
        ));
        r = original.clone();
        r["B8_calibration"]["trusted_binding_sha256"] = json!("other");
        assert!(!admission_matches(
            &r, &a, "trusted", "initial", &receipts, &f, &inputs
        ));
        r = original.clone();
        r["mode"] = json!("cue-broadbatch");
        assert!(!admission_matches(
            &r, &a, "trusted", "initial", &receipts, &f, &inputs
        ));
        r = original;
        r["B8_calibration"]["measured_b8_seconds"] = json!(0.);
        assert!(!admission_matches(
            &r, &a, "trusted", "initial", &receipts, &f, &inputs
        ));
        Ok(())
    }
    fn probe() -> CueQuantumProbe {
        CueQuantumProbe {
            coefficient_index: 123,
            initial_quarters: -1,
            preferred_step: -1,
            evidence_receipt: "receipt.json".into(),
            evidence_receipt_sha256:
                "7f2d33f5fc9562922110b2e5d9f032553665d7a4b2263c51ace113a6793a41d1".into(),
        }
    }
    #[test]
    fn quantum_mode_coordinate_and_donor_quarter_are_explicit() -> Result<()> {
        use uor_r4_integer::geometric_potential_q4::{pack_coefficients, unpack_coefficients};
        let mut a = valid_args()?;
        a.mode = "cue-calibration-quantum-probe".into();
        a.cue_quantum_probe = Some(probe());
        validate(&a)?;
        a.mode = "cue-calibration-fit".into();
        assert!(validate(&a).is_err());
        a.mode = "cue-calibration-quantum-probe".into();
        a.cue_quantum_probe
            .as_mut()
            .ok_or_else(|| invalid("test probe absent"))?
            .coefficient_index = 122;
        assert!(validate(&a).is_err());
        a.cue_quantum_probe = Some(probe());
        a.fit_authorization = Some("optimizer.json".into());
        assert!(validate(&a).is_err());
        a.fit_authorization = None;
        a.cue_quantum_probe
            .as_mut()
            .ok_or_else(|| invalid("test probe absent"))?
            .evidence_receipt_sha256 = "stale".into();
        assert!(validate(&a).is_err());
        let q = probe();
        let mut values = vec![0i8; 960];
        values[122] = 3;
        values[123] = -1;
        let packed = pack_coefficients(&values).map_err(|e| invalid(e.to_string()))?;
        for step in [-1, 1] {
            let changed = quantum_packed(&packed, 960, &q, step)?;
            let decoded = unpack_coefficients(960, &changed).map_err(|e| invalid(e.to_string()))?;
            assert_eq!(decoded[123], -1 + step);
            assert_eq!(decoded[122], 3);
            assert_eq!(
                decoded.iter().zip(&values).filter(|(a, b)| a != b).count(),
                1
            );
        }
        values[123] = 0;
        let wrong = pack_coefficients(&values).map_err(|e| invalid(e.to_string()))?;
        assert!(quantum_packed(&wrong, 960, &q, -1).is_err());
        assert!(quantum_packed(&packed, 120, &q, -1).is_err());
        assert!(quantum_packed(&packed, 960, &q, 0).is_err());
        Ok(())
    }
    #[test]
    fn quantum_saved_score_audit_checks_consumed_address_and_record_binding() -> Result<()> {
        let q = probe();
        let mapping =
            json!([{"occurrence":{"record":1,"commit":1}},{"occurrence":{"record":2,"commit":2}}]);
        let old = json!({"rows":[{"id":"x","tokens":[{"native":{"candidate_mapping":mapping,"actions":{"actions":[{"score_q24":10},{"score_q24":20},{"score_q24":0},{"score_q24":0}]}},"cue_carrier":{"angular_indices":[[null,null],[3,4]]}}]}]});
        let mut new = old.clone();
        new["rows"][0]["tokens"][0]["native"]["actions"]["actions"][0]["score_q24"] =
            json!(10 - (1i64 << 22));
        let truth =
            json!({"rows":[{"id":"x","expected_source_labels_only":{"record":1,"commit":1}}]});
        let result = quantum_margin_changes(&old, &new, &truth, &q, -1)?;
        assert_eq!(result["touched_rows"], 1);
        assert_eq!(result["touched_copy_positions"], 1);
        assert_eq!(
            result["rows"][0]["baseline_correct_vs_distractor_margin_q24"],
            -10
        );
        assert_eq!(result["rows"][0]["margin_delta_q24"], -(1i64 << 22));
        let mut bad = new.clone();
        bad["rows"][0]["tokens"][0]["native"]["candidate_mapping"][0]["occurrence"]["record"] =
            json!(9);
        assert!(quantum_margin_changes(&old, &bad, &truth, &q, -1).is_err());
        assert!(quantum_margin_changes(&old, &old, &truth, &q, -1).is_err());
        let mut absent = old.clone();
        absent["rows"][0]["tokens"][0]["cue_carrier"]["angular_indices"][1][0] = json!(4);
        assert!(quantum_margin_changes(&absent, &absent, &truth, &q, -1).is_err());
        Ok(())
    }
    #[test]
    fn discrete_learning_config_rejects_wrong_mode_manual_overlay_and_unbounded_run() -> Result<()>
    {
        let mut a = valid_args()?;
        a.mode = "cue-calibration-discrete-fit".into();
        a.maximum_seconds = 600;
        a.maximum_report_bytes = 256 * 1024 * 1024;
        a.cue_discrete_fit = Some(CueDiscreteFit {
            maximum_trials: 16,
            maximum_accepted_updates: 8,
            composition_panel: None,
        });
        validate(&a)?;
        a.mode = "cue-calibration-fit".into();
        assert!(validate(&a).is_err());
        a.mode = "cue-calibration-discrete-fit".into();
        a.cue_quantum_probe = Some(probe());
        assert!(validate(&a).is_err());
        a.cue_quantum_probe = None;
        a.fit_authorization = Some("legacy-Adam-admission.json".into());
        assert!(validate(&a).is_err());
        a.fit_authorization = None;
        a.cue_discrete_fit
            .as_mut()
            .ok_or_else(|| invalid("test discrete config absent"))?
            .maximum_trials = 17;
        assert!(validate(&a).is_err());
        a.cue_discrete_fit = Some(CueDiscreteFit {
            maximum_trials: 1,
            maximum_accepted_updates: 2,
            composition_panel: None,
        });
        assert!(validate(&a).is_err());
        a.cue_discrete_fit = Some(CueDiscreteFit {
            maximum_trials: 0,
            maximum_accepted_updates: 0,
            composition_panel: None,
        });
        assert!(validate(&a).is_err());
        Ok(())
    }

    #[test]
    fn prospective_composition_is_explicit_and_cannot_truncate_to_legacy_counts() -> Result<()> {
        let mut a = valid_args()?;
        a.mode = "cue-calibration-discrete-fit".into();
        a.cue_discrete_fit = Some(CueDiscreteFit {
            maximum_trials: 16,
            maximum_accepted_updates: 8,
            composition_panel: Some(CueCompositionPanel {
                profile: COMPOSITION_PROFILE.into(),
                development_rows: 512,
                evaluation_rows: 128,
            }),
        });
        validate(&a)?;
        assert_eq!(panel_counts(&a), (512, 128));
        let report = json!({"schema":"uor-r4.native-bank-observation-panel/1","status":"COMPLETED","transfer_profile":COMPOSITION_PROFILE,"source_policy":DATA_SCOPE,"split":"development","cases":512,"samebank_query_pairs":256});
        assert!(composition_report_matches(&report, "development", 512));
        assert!(!composition_report_matches(&report, "fresh", 512));
        assert!(!composition_report_matches(&report, "development", 128));
        let mut legacy = report.clone();
        legacy["transfer_profile"] = json!("supported-untouched-composition/1");
        assert!(!composition_report_matches(&legacy, "development", 512));
        a.cue_discrete_fit
            .as_mut()
            .ok_or_else(|| invalid("test config absent"))?
            .composition_panel
            .as_mut()
            .ok_or_else(|| invalid("test composition absent"))?
            .development_rows = 128;
        assert!(validate(&a).is_err());
        a.cue_discrete_fit
            .as_mut()
            .ok_or_else(|| invalid("test config absent"))?
            .composition_panel = None;
        validate(&a)?;
        assert_eq!(panel_counts(&a), (128, 32));
        Ok(())
    }
}
