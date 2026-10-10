//! D22 current-parent construction. Historical importers retain their contracts.
use super::constructor_current_model as model;
use super::native_proposals as np;
use super::*;
use uor_r4_training::geometric_occurrence_consumer::source_realizer::PrefixAngularWeights;
#[path = "protected_legal_set.rs"]
mod legal;

pub(super) const REPORT: &str = "787f1a694dd894f8dcf636e158b2470fd88a824f3dd73535749bc4f36f05a165";
const MANIFEST: &str = "2f630ad56a6abd09271681fcb514d22caefac8af918321224bc173446c20a84c";
pub(super) const FIELD: &str = "de4a3234d6e92f4b12a657cc195425687bc67c243d9eded9a1df0797b9942a61";
pub(super) const DIAGNOSTIC_INPUT: &str =
    "c5a643ff9ef86f5f16f15b420f42fe8b1921c5c8c7df5a7bbb88f7759cda5bda";
pub(super) const DIAGNOSTIC_LABEL: &str =
    "b70f57761238ad4044ebdc158e465e978cfd098c9dfbe5b2b4d4a79e582aec77";
const PREFIX: &str = "prefix.coefficients";
const GENERATE: &str = "generate.unary";
const N: usize = 960;

#[derive(Clone, Copy, Debug, Deserialize, serde::Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(super) enum Arm {
    Legal,
    Projected,
}
#[derive(Clone, Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Config {
    pub saved437: PathBuf,
    pub diagnostic_inputs: PathBuf,
    pub diagnostic_labels: PathBuf,
    pub arm: Arm,
}
impl Config {
    pub(super) fn input_roots(&self) -> [&PathBuf; 3] {
        [
            &self.saved437,
            &self.diagnostic_inputs,
            &self.diagnostic_labels,
        ]
    }
}
pub(super) fn settings(a: &Args) -> Result<()> {
    if a.mode != Mode::ConstructorCurrent {
        return replay_require(
            a.constructor.is_none(),
            "constructor settings require constructor_current mode",
        );
    }
    replay_require(
        a.constructor.is_some()
            && [1001, 2001].contains(&a.seed)
            && a.updates == 2
            && a.credit == Credit::RawIdentity
            && a.loss_scope == LossScope::All
            && a.maximum_seconds > 0
            && a.maximum_report_bytes >= 64 << 20
            && a.maximum_report_bytes <= 2 << 30
            && !a.query_conditioned_read
            && !a.ceiling_scorer
            && !a.ceiling_float
            && a.baseline.is_none()
            && a.panel_inputs.is_none()
            && a.panel_labels.is_none()
            && a.cross_state_resume.is_none()
            && a.continuation.is_none()
            && a.joint_continuation.is_none()
            && a.coupled_episode_learning.is_none()
            && a.reference_replay.is_none()
            && !a.native_code_proposals
            && a.read_state_pullback == ReadStatePullback::Legacy
            && !a.cross_state_bottleneck
            && !a.cross_state_pooled_rank
            && !a.reached_frontier_objective
            && !a.categorical_action_learning
            && !a.categorical_action_only
            && !a.constrained_context_learning
            && !a.constrained_emission_learning
            && a.retained_context_root.is_none()
            && a.prototype_compensation.is_none()
            && a.reached_u.is_none()
            && a.readout_coadaptation.is_none()
            && a.prefix_context_credit.is_none()
            && a.context_path_credit.is_none()
            && a.context_cue_coadapt.is_none()
            && a.prefix_fragment_learning.is_none()
            && a.generate_episode_learning.is_none()
            && a.generate_episode_completion.is_none()
            && a.prefix_artifact_check.is_none(),
        "constructor requires the registered two-epoch paired-seed current-parent mode",
    )
}
fn config(a: &Args) -> Result<&Config> {
    a.constructor
        .as_ref()
        .ok_or_else(|| bad("constructor config missing"))
}
pub(super) fn load_field(
    a: &Args,
    p: &ContinuationParent,
) -> Result<(NativeContinuationField, Value)> {
    let c = config(a)?;
    report_output::verify(&c.saved437)?;
    replay_require(
        sha256_file(&c.saved437.join("report.json"))? == REPORT
            && sha256_file(&c.saved437.join("manifest.json"))? == MANIFEST,
        "constructor saved437 exact report/seal mismatch",
    )?;
    let report = read(&c.saved437.join("report.json"))?;
    let bytes = fs::read(c.saved437.join("checkpoint-0256/continuation-field.bin"))?;
    replay_require(
        sha256_bytes(&bytes) == FIELD && report["final"]["complete"] == 437,
        "constructor saved437 field/endpoint mismatch",
    )?;
    let g = NativeGeometricGenerate::from_bytes(&p.generate, p.integer.binding())?;
    let field = NativeContinuationField::from_bytes(&bytes, &p.binding, &g)?;
    replay_require(
        field.is_cross_state(),
        "constructor requires frozen cross-state payload",
    )?;
    Ok((field, report["final"].clone()))
}
fn prefix_weights(
    a: &Args,
    l: &Loaded,
    p: &ContinuationParent,
    d: &Device,
) -> Result<PrefixAngularWeights> {
    let cue = l.frozen.compile_cue_carrier(l.cue.clone())?;
    // The selected native checkpoint retains Prefix Q4 bytes, not a Prefix
    // fractional source. Match the existing from_native initializer exactly.
    // Generate fractional masters are restored independently by the parent loader.
    let native = l
        .frozen
        .compile_prefix_transport(&cue, prefix_clone(&l.prefix)?)?;
    let live = PrefixAngularWeights::from_native(
        &l.frozen,
        &a.checkpoint.join("native"),
        &a.checkpoint.join("cue"),
        &cue,
        &native,
        d,
    )?;
    replay_require(
        live.packed_coefficients()? == p.prefix,
        "constructor canonical Prefix/native authority mismatch",
    )?;
    Ok(live)
}
fn parameters(pw: &PrefixAngularWeights, l: &Loaded) -> Result<BTreeMap<String, Var>> {
    let mut p = pw.parameters();
    p.extend(l.generate.parameters());
    let keep = [PREFIX, GENERATE].into_iter().collect::<BTreeSet<_>>();
    p.retain(|k, _| keep.contains(k.as_str()));
    replay_require(
        p.len() == 2 && p.values().all(|v| v.elem_count() == N),
        "constructor parameter shape",
    )?;
    Ok(p)
}
fn masters(params: &BTreeMap<String, Var>) -> Result<Vec<f32>> {
    let s = np::snapshot(params)?;
    Ok([
        s.get(PREFIX)
            .ok_or_else(|| bad("Prefix master absent"))?
            .as_slice(),
        s.get(GENERATE)
            .ok_or_else(|| bad("Generate master absent"))?
            .as_slice(),
    ]
    .concat())
}
fn set_masters(params: &BTreeMap<String, Var>, m: &[f32]) -> Result<()> {
    replay_require(
        m.len() == 2 * N
            && m.iter()
                .all(|x| x.is_finite() && (-1.75..=1.75).contains(x)),
        "constructor destination masters malformed",
    )?;
    np::restore(
        params,
        &BTreeMap::from([
            (PREFIX.into(), m[..N].to_vec()),
            (GENERATE.into(), m[N..].to_vec()),
        ]),
    )
}
fn rows(eval: &Value) -> Result<&Vec<Value>> {
    eval["rows"]
        .as_array()
        .ok_or_else(|| bad("constructor evaluation rows absent"))
}
fn baseline(current: &Value, saved: &Value) -> Result<()> {
    replay_require(
        current["complete"] == 437
            && current["continuation_sha256"] == FIELD
            && rows(current)?.len() == 512
            && rows(saved)?.len() == 512,
        "constructor baseline extent/count",
    )?;
    let mut seen = BTreeSet::new();
    for (a, b) in rows(current)?.iter().zip(rows(saved)?) {
        replay_require(
            seen.insert(a["id"].as_str().ok_or_else(|| bad("baseline ID"))?),
            "duplicate baseline ID",
        )?;
        for key in ["id", "generated_ids", "complete"] {
            replay_require(a[key] == b[key], "constructor exact baseline row mismatch")?;
        }
    }
    Ok(())
}
/// Hash-ranked sampling without replacement, fixed before backward. Actual
/// generated prefixes are retained, including alternate accepted answer forms.
fn sample_guards(eval: &Value, seed: u64, epoch: usize) -> Result<Vec<model::Position>> {
    let mut pools: [Vec<(String, model::Position)>; 3] = Default::default();
    for (input, row) in rows(eval)?.iter().enumerate() {
        if row["complete"] != true {
            continue;
        }
        let id = row["id"].as_str().ok_or_else(|| bad("guard ID"))?;
        let ids: Vec<u32> = serde_json::from_value(row["generated_ids"].clone())?;
        replay_require(
            ids.len() >= 2 && row["eos"] == true,
            "guard phase overlap/missing EOS",
        )?;
        for (position, &target) in ids.iter().enumerate() {
            let phase = if position == 0 {
                0
            } else if position + 1 == ids.len() {
                1
            } else {
                2
            };
            let key = sha256_bytes(
                format!("uor-d22-guards-v1\0{seed}\0{epoch}\0{phase}\0{id}\0{position}").as_bytes(),
            );
            pools[phase].push((
                key,
                model::Position {
                    input,
                    position,
                    prefix: ids[..position].to_vec(),
                    target,
                },
            ));
        }
    }
    let mut selected = Vec::new();
    for (pool, quota) in pools.iter_mut().zip([96, 96, 188]) {
        pool.sort_by(|a, b| {
            a.0.cmp(&b.0)
                .then_with(|| a.1.input.cmp(&b.1.input))
                .then_with(|| a.1.position.cmp(&b.1.position))
        });
        replay_require(
            pool.len() >= quota,
            "original437 guard phase quota unavailable",
        )?;
        selected.extend(pool.iter().take(quota).map(|x| x.1.clone()));
    }
    selected.sort_by(|a, b| {
        eval["rows"][a.input]["id"]
            .as_str()
            .cmp(&eval["rows"][b.input]["id"].as_str())
            .then_with(|| a.position.cmp(&b.position))
    });
    replay_require(
        selected.len() == 380
            && selected
                .windows(2)
                .all(|w| (w[0].input, w[0].position) != (w[1].input, w[1].position)),
        "duplicate guard identity",
    )?;
    Ok(selected)
}
fn unit_rows(j: &[Vec<f64>]) -> Result<Vec<Vec<f64>>> {
    replay_require(j.len() == 380, "constructor guard Jacobian count")?;
    j.iter()
        .map(|r| {
            replay_require(
                r.len() == 2 * N && r.iter().all(|x| x.is_finite()),
                "constructor Jacobian shape/nonfinite",
            )?;
            let n = r.iter().map(|x| x * x).sum::<f64>().sqrt();
            replay_require(n.is_finite(), "constructor Jacobian norm overflow")?;
            Ok(r.iter().map(|x| if n == 0. { 0. } else { x / n }).collect())
        })
        .collect()
}
fn screen(m: &[f32], dest: &[f32], g: &[f64], rows: &[Vec<f64>]) -> Result<Value> {
    replay_require(
        [dest.len(), g.len()].into_iter().all(|n| n == m.len()),
        "constructor screen shape",
    )?;
    let d = m
        .iter()
        .zip(dest)
        .map(|(a, b)| f64::from(*b) - f64::from(*a))
        .collect::<Vec<_>>();
    let norm = d.iter().map(|x| x * x).sum::<f64>().sqrt();
    let residuals = rows
        .iter()
        .map(|r| r.iter().zip(&d).map(|(a, b)| a * b).sum::<f64>())
        .collect::<Vec<_>>();
    let gd = g.iter().zip(&d).map(|(a, b)| a * b).sum::<f64>();
    replay_require(
        norm.is_finite() && gd.is_finite() && residuals.iter().all(|x| x.is_finite()),
        "constructor screen overflow",
    )?;
    Ok(
        json!({"nonzero":norm>0.,"gradient_dot":gd,"norm":norm,"tolerance":1e-10*norm,
        "residuals":residuals,"eligible":norm>0.&&gd<0.&&residuals.iter().all(|r|*r>=-1e-10*norm)}),
    )
}
fn project(m: &[f32], g: &[f64], rows: &[Vec<f64>]) -> Result<Vec<(String, Vec<f32>, Value)>> {
    let mut d = g.iter().map(|x| -x).collect::<Vec<_>>();
    for _ in 0..256 {
        for row in rows {
            let v = row.iter().zip(&d).map(|(x, y)| x * y).sum::<f64>();
            if v < 0. {
                for (x, j) in d.iter_mut().zip(row) {
                    *x -= v * j;
                }
            }
        }
    }
    let max = d.iter().fold(0f64, |a, x| a.max(x.abs()));
    replay_require(max.is_finite(), "projection nonfinite")?;
    let norm = d.iter().map(|v| v * v).sum::<f64>().sqrt();
    let residuals = rows
        .iter()
        .map(|r| r.iter().zip(&d).map(|(a, b)| a * b).sum::<f64>())
        .collect::<Vec<_>>();
    let continuous = norm.is_finite()
        && residuals
            .iter()
            .all(|v| v.is_finite() && *v >= -1e-10 * norm);
    [1,2,4,7].into_iter().map(|radius|{
        let dest=m.iter().zip(&d).map(|(&m,&v)|{
            let original=(m*4.).round() as i8;
            let x=(f64::from(m)+if max==0.{0.}else{f64::from(radius)*0.25*v/max}).clamp(-1.75,1.75) as f32;
            let q=(x*4.).round() as i8;
            if q==original {m}else{f32::from(q)*0.25}
        }).collect::<Vec<_>>();
        let mut receipt=screen(m,&dest,g,rows)?;
        receipt["eligible"]=json!(receipt["eligible"]==true && continuous && max>0.);
        Ok((format!("radius-{radius}"),dest,json!({"projection_passes":256,"direction":d,
            "continuous_residuals":residuals,"continuous_constraints_passed":continuous,"screen":receipt})))
    }).collect()
}

fn export(
    a: &Args,
    index: usize,
    l: &mut Loaded,
    pw: &PrefixAngularWeights,
    original: &ContinuationParent,
    frozen_field: &NativeContinuationField,
) -> Result<(ContinuationParent, NativeContinuationField, PathBuf)> {
    l.prefix = pw.native()?;
    let (_, _, _, mut receipt) = checkpoint(a, index, l)?;
    let root = a.out.join(format!("checkpoint-{index:04}"));
    // Preserve the unchanged Cue source authority. The generic checkpoint only
    // writes served sidecars; this mode also retains Prefix fractional masters.
    for entry in fs::read_dir(a.checkpoint.join("cue"))? {
        let entry = entry?;
        replay_require(
            entry.file_type()?.is_file(),
            "constructor Cue source has non-file entry",
        )?;
        fs::copy(entry.path(), root.join("cue").join(entry.file_name()))?;
    }
    let temporary = root.join("prefix-masters");
    pw.save(&temporary)?;
    for entry in fs::read_dir(&temporary)? {
        let entry = entry?;
        fs::copy(entry.path(), root.join("prefix").join(entry.file_name()))?;
    }
    let current = ContinuationParent::from_checkpoint(&root)?;
    replay_require(
        current.binding == original.binding
            && current.cue == original.cue
            && current.joint == original.joint
            && current.bridge == original.bridge
            && current.exp == original.exp,
        "constructor frozen native family changed",
    )?;
    let native = NativeGeometricGenerate::from_bytes(&current.generate, current.integer.binding())?;
    let old = NativeGeometricGenerate::from_bytes(&original.generate, original.integer.binding())?;
    replay_require(
        native.prototypes() == old.prototypes(),
        "constructor prototype change",
    )?;
    let field = NativeContinuationField::compile_cross_state(
        &current.binding,
        &native,
        frozen_field
            .packed_cross_state()
            .ok_or_else(|| bad("constructor cross payload missing"))?,
    )?;
    let bytes = field.to_bytes()?;
    fs::write(root.join("continuation-field.bin"), &bytes)?;
    let reloaded = NativeContinuationField::from_bytes(
        &fs::read(root.join("continuation-field.bin"))?,
        &current.binding,
        &native,
    )?;
    replay_require(
        reloaded.to_bytes()? == bytes
            && reloaded.packed_cross_state() == frozen_field.packed_cross_state(),
        "constructor field independent reload/payload mismatch",
    )?;
    let cue = l.frozen.compile_cue_carrier(l.cue.clone())?;
    let restored = PrefixAngularWeights::load(
        &root.join("prefix"),
        &l.frozen,
        &root.join("native"),
        &root.join("cue"),
        &cue,
    )?;
    replay_require(
        np::same_bits(
            &np::snapshot(&restored.parameters())?,
            &np::snapshot(&pw.parameters())?,
        ) && restored.packed_coefficients()? == current.prefix,
        "constructor Prefix independent master reload mismatch",
    )?;
    receipt["schema"] = json!("uor-r4.d22-constructor-checkpoint/1");
    receipt["mode"] = json!("constructor_current");
    receipt["fresh_adam"] = json!(false);
    receipt["optimizer_updates"] = json!(0);
    receipt["continuation_sha256"] = json!(sha256_bytes(&bytes));
    receipt["saved437_report_sha256"] = json!(REPORT);
    receipt["frozen_cross_coefficients_sha256"] = json!(sha256_bytes(
        reloaded
            .packed_cross_state()
            .ok_or_else(|| bad("cross bytes absent"))?
    ));
    receipt["active_master_identities"] = serde_json::to_value(identities(&parameters(pw, l)?)?)?;
    receipt["frozen_source_master_identities"] =
        serde_json::to_value(identities(&l.source.parameters())?)?;
    receipt["scope"]=json!("saved native legal candidate; transaction/quality decisions are separate in report; original Source/Context/Cue/prototypes/cross coefficients frozen");
    fs::write(
        root.join("receipt.json"),
        serde_json::to_vec_pretty(&receipt)?,
    )?;
    Ok((ContinuationParent::from_checkpoint(&root)?, reloaded, root))
}

struct Candidate {
    name: String,
    index: usize,
    epoch: usize,
    masters: Vec<f32>,
    objective: f64,
    linear: bool,
    native_guards: bool,
    committed: bool,
    development: Value,
    checkpoint: PathBuf,
}
fn complete(v: &Value) -> Result<u64> {
    v["complete"]
        .as_u64()
        .ok_or_else(|| bad("complete count missing"))
}
fn compare_rows(initial: &Value, current: &Value) -> Result<Value> {
    replay_require(
        rows(initial)?.len() == rows(current)?.len(),
        "rowwise comparison extent",
    )?;
    let mut kept = Vec::new();
    let mut lost = Vec::new();
    let mut gained = Vec::new();
    for (a, b) in rows(initial)?.iter().zip(rows(current)?) {
        replay_require(a["id"] == b["id"], "rowwise ID order mismatch")?;
        match (a["complete"] == true, b["complete"] == true) {
            (true, true) => kept.push(a["id"].clone()),
            (true, false) => lost.push(a["id"].clone()),
            (false, true) => gained.push(a["id"].clone()),
            _ => {}
        }
    }
    Ok(json!({"kept":kept,"lost":lost,"gained":gained}))
}
pub(super) fn own_prefix(
    a: &Args,
    name: &str,
    p: &ContinuationParent,
    field: &NativeContinuationField,
    eps: &[Episode],
    start: Instant,
) -> Result<Value> {
    continuation_evaluate_rows_impl(
        a,
        name,
        p,
        field,
        &eps.iter().collect::<Vec<_>>(),
        start,
        false,
    )
}
pub(super) fn read_checkpoint(
    root: &Path,
) -> Result<(ContinuationParent, NativeContinuationField)> {
    let p = ContinuationParent::from_checkpoint(root)?;
    let g = NativeGeometricGenerate::from_bytes(&p.generate, p.integer.binding())?;
    let f = NativeContinuationField::from_bytes(
        &fs::read(root.join("continuation-field.bin"))?,
        &p.binding,
        &g,
    )?;
    Ok((p, f))
}

pub(super) fn run(a: &Args, start: Instant, d: &Device) -> Result<Value> {
    settings(a)?;
    let c = config(a)?;
    let original = ContinuationParent::load(a)?;
    let (field, saved) = load_field(a, &original)?;
    let mut l = load_joint_continuation(a, &original, d)?;
    let pw = prefix_weights(a, &l, &original, d)?;
    let params = parameters(&pw, &l)?;
    let initial_masters = masters(&params)?;
    let frozen_source = identities(&l.source.parameters())?;
    let public = NativeVocabularyActions::new(original.integer.binding().clone(), &original.exp)?;
    let legal_tokens = public
        .legal_token_ids()
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    let eps = load_panel(
        &a.training_inputs,
        &a.training_labels,
        &original.integer,
        &original.tokenizer,
        &legal_tokens,
        512,
    )?;
    replay_require(eps.len() == 512, "constructor objective panel count")?;
    for (path, hash) in [
        (&c.diagnostic_inputs, DIAGNOSTIC_INPUT),
        (&c.diagnostic_labels, DIAGNOSTIC_LABEL),
    ] {
        report_output::verify(&seal_for(path)?)?;
        replay_require(
            sha256_file(path)? == hash,
            "constructor opened diagnostic identity",
        )?;
    }
    // Diagnostic identities are pinned here; scoring waits for all four sealed
    // training reports and a global per-seed nomination in a separate phase.
    let initial = own_prefix(a, "baseline-development", &original, &field, &eps, start)?;
    baseline(&initial, &saved)?;
    let objective_positions = model::objective_positions(&eps)?;
    let guard_positions = (0..2)
        .map(|epoch| sample_guards(&initial, a.seed, epoch))
        .collect::<Result<Vec<_>>>()?;
    write(
        a,
        "guard-identities.json",
        &json!({"schema":"uor-r4.d22-paired-guard-identities/1","seed":a.seed,
        "sampler":"sha256 uor-d22-guards-v1 NUL seed NUL epoch NUL phase NUL input-id NUL target-position; rank then take without replacement; sort input-id/position",
        "phase_quotas":[96,96,188],"epochs":guard_positions}),
    )?;
    write(
        a,
        "admission.json",
        &json!({"schema":"uor-r4.d22-constructor-admission/1","config":c,
        "source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT"),"seed":a.seed,"base_field":FIELD,
        "base_report":REPORT,"prefix_master_origin":"canonical quarter centres from saved Prefix Q4; fractional Prefix source absent from selected saved artifact","generate_master_origin":"exact saved fractional Generate masters","initial_masters":initial_masters,"active_master_identities":identities(&params)?,
        "frozen_source_master_identities":frozen_source,"objective_positions":objective_positions.len(),
        "guard_identities_sha256":sha256_file(&a.out.join("guard-identities.json"))?,
        "diagnostic_scope":"opened128; no final fresh qualification; never enters training/selection"}),
    )?;
    let mut candidates = Vec::<Candidate>::new();
    let mut incumbent = initial_masters.clone();
    let mut endpoint: Option<usize> = None;
    let mut epochs = Vec::new();
    let mut next_checkpoint = 1;
    for epoch in 0..2 {
        set_masters(&params, &incumbent)?;
        let (parent, current_field) =
            model::candidate_parent(&original, &field, &l, &pw, &l.generate)?;
        let objective = model::capture(
            a,
            start,
            &parent,
            &current_field,
            &eps,
            &objective_positions,
        )?;
        let guards = model::capture(
            a,
            start,
            &parent,
            &current_field,
            &eps,
            &guard_positions[epoch],
        )?;
        let credit = model::credit(
            a,
            start,
            &parent,
            &current_field,
            &pw,
            &l.generate,
            &objective,
            &guards,
        )?;
        let g32 = credit
            .gradient
            .iter()
            .map(|x| *x as f32)
            .collect::<Vec<_>>();
        replay_require(
            g32.len() == 2 * N && g32.iter().all(|x| x.is_finite()),
            "constructor gradient decode",
        )?;
        let g = g32.iter().map(|x| f64::from(*x)).collect::<Vec<_>>();
        let normalized = unit_rows(&credit.jacobian)?;
        write(
            a,
            &format!("epoch-{epoch}-credit.json"),
            &json!({"objective":credit.objective,"gradient_f64":credit.gradient,
            "effective_gradient_f32":g32,"jacobian":credit.jacobian,"unit_rows":normalized,
            "original_masters":incumbent,"guard_margins":credit.guard_margins,"receipt":credit.receipt}),
        )?;
        // Both arms consume the same f32-projected gradient boundary.
        let offers = match c.arm {
            Arm::Projected => project(&incumbent, &g, &normalized)?,
            Arm::Legal => {
                let receipt = legal::run(&incumbent, &g32, &normalized)?;
                legal::authenticate(&incumbent, &g32, &normalized, &receipt)?;
                write(
                    a,
                    &format!("epoch-{epoch}-solver.json"),
                    &serde_json::to_value(&receipt)?,
                )?;
                match &receipt.destination_master_bits {
                    Some(bits) => {
                        let dest = bits.iter().map(|b| f32::from_bits(*b)).collect::<Vec<_>>();
                        let checked = screen(&incumbent, &dest, &g, &normalized)?;
                        vec![(
                            "legal".into(),
                            dest,
                            json!({"solver":receipt,"screen":checked}),
                        )]
                    }
                    None => {
                        // Numeric/internal failure is an execution blocker, never
                        // a no-offer model result or an infeasibility certificate.
                        if receipt.status != "NODE_LIMIT_NO_INCUMBENT" {
                            return Err(bad(&format!(
                                "constructor execution blocker: {} / {}",
                                receipt.status, receipt.termination
                            )));
                        }
                        Vec::new()
                    }
                }
            }
        };
        let mut accepted: Option<usize> = None;
        let mut epoch_offers = Vec::new();
        for (name, dest, receipt) in offers {
            let nonzero = dest
                .iter()
                .zip(&incumbent)
                .any(|(a, b)| a.to_bits() != b.to_bits());
            if !nonzero {
                epoch_offers.push(json!({"name":name,"status":"NOOP","receipt":receipt}));
                continue;
            }
            set_masters(&params, &dest)?;
            let (p, f, cp) = export(a, next_checkpoint, &mut l, &pw, &original, &field)?;
            let obj = model::capture(a, start, &p, &f, &eps, &objective_positions)?;
            let grd = model::capture(a, start, &p, &f, &eps, &guard_positions[epoch])?;
            let native = model::evaluate_frames(a, start, &p, &f, &obj, &grd)?;
            let linear = receipt["screen"]["eligible"] == true;
            let can_commit =
                linear && native.guard_preserved && native.objective < credit.objective;
            let label = format!("epoch-{epoch}-{name}");
            let development = own_prefix(a, &format!("{label}-development"), &p, &f, &eps, start)?;
            let index = candidates.len();
            write(
                a,
                &format!("{label}-candidate.json"),
                &json!({"name":label,"receipt":receipt,"native":native,
                "transaction_eligible":can_commit,"checkpoint":cp,"development_complete":complete(&development)?,
                "rowwise_vs_original":compare_rows(&initial,&development)?}),
            )?;
            candidates.push(Candidate {
                name: label,
                index: next_checkpoint,
                epoch,
                masters: dest,
                objective: native.objective,
                linear,
                native_guards: native.guard_preserved,
                committed: false,
                development,
                checkpoint: cp,
            });
            if can_commit && accepted.is_none_or(|i| native.objective < candidates[i].objective) {
                accepted = Some(index);
            }
            epoch_offers.push(
                json!({"name":name,"candidate_index":index,"checkpoint":next_checkpoint,
                "native_objective":native.objective,"transaction_eligible":can_commit}),
            );
            next_checkpoint += 1;
        }
        if let Some(i) = accepted {
            candidates[i].committed = true;
            incumbent = candidates[i].masters.clone();
            endpoint = Some(i);
        }
        set_masters(&params, &incumbent)?;
        replay_require(
            identities(&l.source.parameters())? == frozen_source,
            "constructor frozen source masters changed",
        )?;
        epochs.push(json!({"epoch":epoch,"parent_objective":credit.objective,"offers":epoch_offers,"accepted_candidate":accepted}));
        write(
            a,
            &format!("epoch-{epoch}-result.json"),
            epochs.last().ok_or_else(|| bad("epoch receipt missing"))?,
        )?;
    }
    // Nomination is completed and written BEFORE any diagnostic predictions.
    let mut nominee: Option<usize> = None;
    let mut first: Option<usize> = None;
    for (i, candidate) in candidates.iter().enumerate() {
        if first.is_none() {
            first = Some(i);
        }
        let count = complete(&candidate.development)?;
        let better = match nominee {
            None => true,
            Some(j) => {
                let prior = complete(&candidates[j].development)?;
                count > prior || (count == prior && candidate.objective < candidates[j].objective)
            }
        };
        if better {
            nominee = Some(i);
        }
    }
    write(
        a,
        "selection-before-diagnostic.json",
        &json!({"endpoint":endpoint,"first_legal":first,"nominee":nominee,
        "selection":"all exported valid Q4 candidates: development count descending, objective ascending, earliest epoch/radius; linear eligibility and native guard veto reported separately",
        "diagnostic_predictions":"NOT_RUN","epochs":epochs}),
    )?;
    Ok(
        json!({"schema":"uor-r4.d22-constructor-result/1","status":"COMPLETED_TRAINING_DIAGNOSTIC_PENDING","seed":a.seed,"arm":c.arm,
        "source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT"),"base_report":REPORT,"base_field":FIELD,
        "initial":initial,"diagnostic_scope":"opened128 pending globally frozen per-seed selection; final fresh qualification NOT_RUN","epochs":epochs,
        "candidates":candidates.iter().enumerate().map(|(i,c)|json!({"candidate_index":i,"name":c.name,
            "checkpoint_index":c.index,"epoch":c.epoch,"checkpoint":c.checkpoint,"objective":c.objective,
            "linear_screen":c.linear,"native_guards":c.native_guards,"committed":c.committed,"development":c.development})).collect::<Vec<_>>(),
        "endpoint":endpoint,"first_legal":first,"nominee":nominee,
        "elapsed_seconds":start.elapsed().as_secs_f64(),"scope":"paired-seed result requires both reports and control comparison; opened128 is diagnostic, final fresh qualification NOT_RUN; mechanism remains open"}),
    )
}

#[cfg(test)]
mod current_constructor_tests {
    use super::*;

    fn original_success_fixture(successes: usize) -> Value {
        json!({"complete":successes,"rows":(0..512).map(|i|json!({
            // Reverse IDs ensure lexical order is observably different from input order.
            "id":format!("bound-input-{:04}",511-i),"complete":i<successes,"eos":true,
            "generated_ids":[100+i as u32,700+i as u32,1300+i as u32,1],
            // The sampler must use actual generated IDs, never a canonical alternative.
            "canonical_target_ids_labels_only":[9,8,7,1]
        })).collect::<Vec<_>>()})
    }
    #[test]
    fn constructor_guards_preserve_actual_prefixes_phase_quotas_and_paired_seed_identity(
    ) -> Result<()> {
        let original = original_success_fixture(437);
        let a = sample_guards(&original, 1001, 0)?;
        let paired = sample_guards(&original, 1001, 0)?;
        assert_eq!(serde_json::to_value(&a)?, serde_json::to_value(&paired)?);
        assert_eq!(a.len(), 380);
        let mut quotas = [0usize; 3];
        let mut unique = BTreeSet::new();
        let mut ordered = Vec::new();
        for p in &a {
            assert!(p.input < 437);
            assert!(unique.insert((p.input, p.position)));
            let actual: Vec<u32> =
                serde_json::from_value(original["rows"][p.input]["generated_ids"].clone())?;
            assert_eq!(p.prefix, actual[..p.position]);
            assert_eq!(p.target, actual[p.position]);
            assert_ne!(actual, vec![9, 8, 7, 1]);
            quotas[if p.position == 0 {
                0
            } else if p.position + 1 == actual.len() {
                1
            } else {
                2
            }] += 1;
            ordered.push((
                original["rows"][p.input]["id"]
                    .as_str()
                    .ok_or_else(|| bad("fixture ID"))?
                    .to_owned(),
                p.position,
            ));
        }
        assert_eq!(quotas, [96, 96, 188]);
        assert!(ordered.windows(2).all(|w| w[0] < w[1]));
        let other_seed = sample_guards(&original, 2001, 0)?;
        let other_epoch = sample_guards(&original, 1001, 1)?;
        assert_ne!(serde_json::to_value(&a)?, serde_json::to_value(other_seed)?);
        assert_ne!(
            serde_json::to_value(&a)?,
            serde_json::to_value(other_epoch)?
        );
        Ok(())
    }
    #[test]
    fn constructor_guard_phase_shortage_fails_without_refilling_from_another_phase() -> Result<()> {
        // 95 successful rows have abundant interior positions but too few entry/EOS guards.
        assert!(sample_guards(&original_success_fixture(95), 1001, 0).is_err());
        let mut missing_eos = original_success_fixture(437);
        missing_eos["rows"][0]["eos"] = json!(false);
        assert!(sample_guards(&missing_eos, 1001, 0).is_err());
        Ok(())
    }
    #[test]
    fn constructor_projection_keeps_fractional_noop_bits_and_fixed_radius_order() -> Result<()> {
        let original = [0.10f32, 0.11f32];
        let offers = project(&original, &[-1., 0.], &[])?;
        assert_eq!(
            offers.iter().map(|x| x.0.as_str()).collect::<Vec<_>>(),
            ["radius-1", "radius-2", "radius-4", "radius-7"]
        );
        for (_, dest, receipt) in &offers {
            assert_eq!(dest[1].to_bits(), original[1].to_bits());
            assert_ne!(dest[1].to_bits(), 0f32.to_bits());
            assert_eq!(dest[0] * 4., (dest[0] * 4.).round());
            assert_eq!(receipt["projection_passes"], 256);
            assert_eq!(receipt["continuous_constraints_passed"], true);
            assert_eq!(receipt["screen"]["eligible"], true);
        }
        let noops = project(&original, &[0., 0.], &[])?;
        for (_, dest, receipt) in noops {
            assert_eq!(
                dest.iter().map(|x| x.to_bits()).collect::<Vec<_>>(),
                original.iter().map(|x| x.to_bits()).collect::<Vec<_>>()
            );
            assert_eq!(receipt["screen"]["eligible"], false);
        }
        Ok(())
    }
    #[test]
    fn constructor_projection_rejects_rounding_that_breaks_a_valid_continuous_guard() -> Result<()>
    {
        // Continuous [1,1] lies exactly in this guard's tangent plane. Rounding
        // fractional [0.10,0] to [.25,.25] gives delta[.15,.25], violating it.
        let original = [0.10f32, 0.];
        let offers = project(&original, &[-1., -1.], &[vec![1., -1.]])?;
        let (_, dest, receipt) = &offers[0];
        assert_eq!(*dest, vec![0.25, 0.25]);
        assert_eq!(receipt["continuous_constraints_passed"], true);
        assert_eq!(receipt["continuous_residuals"][0], 0.);
        assert!(
            receipt["screen"]["gradient_dot"]
                .as_f64()
                .ok_or_else(|| bad("fixture gradient dot"))?
                < 0.
        );
        assert!(
            receipt["screen"]["residuals"][0]
                .as_f64()
                .ok_or_else(|| bad("fixture residual"))?
                < -0.09
        );
        assert_eq!(receipt["screen"]["eligible"], false);
        Ok(())
    }
}
