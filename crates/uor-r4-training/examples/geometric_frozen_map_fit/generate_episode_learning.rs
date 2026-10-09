//! Offline shared Generate-unary complete-episode learning. No serving changes.
use super::context_cue_coadapt as shared;
use super::native_proposals as np;
use super::prefix_fragment_learning as prefix;
use super::*;
use uor_r4_integer::geometric_vocabulary_actions::{GeneratePatchCache, PendingGeneratePatch};
use uor_r4_training::geometric_generate_learning::vocabulary_marginal_loss_with_credit;
const NAME: &str = "generate.unary";
const COUNT: usize = 960;
const VOCAB: usize = 4096;
const GUARDS: usize = 380;
const UNION: usize = 391;
const CACHE_CAP: usize = 256 * 1024 * 1024;

#[derive(Clone, Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Config {
    pub retained_episode_root: PathBuf,
    pub expected_report_sha256: String,
    pub expected_manifest_sha256: String,
    pub original_inputs: prefix::Config,
}
impl Config {
    pub(super) fn input_roots(&self) -> Vec<&PathBuf> {
        let mut out = vec![
            &self.retained_episode_root,
            &self.original_inputs.retained_intermediate_root,
            &self.original_inputs.retained_probe_root,
        ];
        if let Some(e) = &self.original_inputs.episode {
            out.extend([
                &e.typed_authority,
                &e.retained_projection.root,
                &e.retained_supplement_root,
            ]);
            out.extend(e.phases.iter().map(|p| &p.capture.root));
        }
        out
    }
}
pub(super) fn validate_settings(a: &Args) -> Result<()> {
    if let Some(c) = &a.generate_episode_learning {
        replay_require(
            a.mode == Mode::JointContinuation
                && a.updates == 1
                && a.loss_scope == LossScope::All
                && a.prefix_fragment_learning.is_none()
                && a.prefix_artifact_check.is_none()
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
            "Generate episode requires exclusive fresh unary mode",
        )?;
        replay_require(
            [&c.expected_report_sha256, &c.expected_manifest_sha256]
                .iter()
                .all(|h| h.len() == 64 && h.bytes().all(|b| b.is_ascii_hexdigit())),
            "Generate episode witness hashes invalid",
        )?;
        replay_require(
            c.original_inputs.episode.is_some()
                && c.original_inputs.joint.is_none()
                && c.original_inputs.trajectory.is_none(),
            "Generate original episode spec absent",
        )?;
        let mut inherited = a.clone();
        inherited.generate_episode_learning = None;
        inherited.prefix_fragment_learning = Some(c.original_inputs.clone());
        prefix::validate_settings(&inherited)?;
    }
    Ok(())
}
pub(super) fn policy() -> Value {
    json!({"schema":"uor-r4.generate-episode-learning/1","active_parameter_names":[NAME],
        "coordinate_count":COUNT,"alternatives_per_coordinate":14,
        "initialization":"ORIGINAL selected Source9f/G4248/U82ae/fractional Cue+Prefix+Generate; no negative candidate",
        "credit":"factual native coefficient-only quarter-shadow STE; fixed poststate/prototypes; no relaxed transport/donor selector",
        "construction":"frozen fresh-gradient best actual displacement order; all14 other legal codes per960, including initialzero/countergradient; one best feasible commit percoordinate, no rerank/revisit",
        "trial_gate":"strict current32-role native CE margin1e-10*(1+abs(current)) AND17refs/all380 original winners",
        "code_selection":"lowest feasible native combinedCE; exact numeric CE tie chooses smallest signed code",
        "final_gate":"strict ORIGINAL episode and combinedCE, all15 conditional winners/EOS,17refs/all380; actual9 typed answer/EOS separate",
        "native_score_unit_q24":1i64<<20,"frozen":"all other Generate masters/Source/Cue/Prefix/bridge/U coefficient bits",
        "gradient_context_encoder_calls":0,"captured_objective_encoder_calls":0,"native_reload_steps":391,"new_context_gradients":0,"new_prefix_gradients":0,"optimizer_updates":0})
}
fn code(m: f32) -> Result<i8> {
    replay_require(
        m.is_finite() && (-1.75..=1.75).contains(&m),
        "Generate unary master outside existing strict Q4 admission",
    )?;
    Ok((4. * m).round() as i8)
}
fn improves(before: f64, after: f64) -> bool {
    after < before - 1e-10 * (1. + before.abs())
}
fn lower(a: (f64, i8), b: (f64, i8)) -> bool {
    a.0 < b.0 || (a.0 == b.0 && a.1 < b.1)
}
fn coordinate_order(m: &[f32], g: &[f32]) -> Result<Vec<usize>> {
    replay_require(
        m.len() == COUNT && g.len() == COUNT,
        "Generate rank shape differs",
    )?;
    let mut rows = Vec::with_capacity(COUNT);
    for (i, (&m, &g)) in m.iter().zip(g).enumerate() {
        let q = code(m)?;
        replay_require(g.is_finite(), "Generate gradient nonfinite")?;
        let potential = (-7i8..=7)
            .filter(|c| *c != q)
            .map(|c| f64::from(g) * (f64::from(c) * 0.25 - f64::from(m)))
            .min_by(f64::total_cmp)
            .ok_or_else(|| bad("Generate legal code grid empty"))?;
        rows.push((i, potential));
    }
    rows.sort_by(|a, b| a.1.total_cmp(&b.1).then_with(|| a.0.cmp(&b.0)));
    Ok(rows.into_iter().map(|r| r.0).collect())
}
fn saved_pool(f: &shared::Frame, reducer: &mut NativeVocabularyActions) -> Result<shared::Pool> {
    let generate: Vec<i64> = shared::dec(&f.native["generate_q24"])?;
    let copy: Vec<i64> = shared::dec(&f.native["copy_q24"])?;
    let trace = reducer.reduce_trace(&generate, &f.ids, &copy)?;
    replay_require(
        serde_json::to_value(&trace)? == f.native["pool"],
        "Generate original full pool parity differs",
    )?;
    Ok(shared::Pool {
        donor: shared::idx(&f.native["bridge"]["selected_ordinal"])?,
        post: shared::codes(&f.native["post_state"])?,
        generate,
        copy,
        base_copy: f.base_copy.clone(),
        trace,
    })
}
fn slim(frames: &mut [shared::Frame]) -> Result<()> {
    for f in frames {
        f.native["pool"]
            .as_object_mut()
            .ok_or_else(|| bad("Generate native pool missing"))?
            .remove("actions");
        f.native["pool_action_trace"]=json!("OMITTED_RECONSTRUCTIBLE_FROM_COMPLETE_SCORES after complete hard parity; all rawG/Copy/U/tokenmasses/summary retained");
    }
    Ok(())
}
fn gradient(
    a: &Args,
    start: Instant,
    p: &ContinuationParent,
    frames: &[shared::Frame],
    pools: &[shared::Pool],
    d: &Device,
) -> Result<Vec<f32>> {
    let ng = NativeGeometricGenerate::from_bytes(&p.generate, p.integer.binding())?;
    let g = GenerateLearningWeights::from_native(p.integer.binding().clone(), &ng, d)?;
    restore(
        &a.checkpoint.join("generate-source"),
        &read(&a.checkpoint.join("generate-source/metadata.json"))?["parameters"],
        &g.parameters(),
        d,
    )?;
    replay_require(
        g.export_native()?.to_bytes()? == p.generate,
        "Generate actual master restore/native differs",
    )?;
    let prepared = g.prepare_native()?;
    // Native factual parity for every physical objective precedes ANY backward.
    for (f, pool) in frames.iter().zip(pools) {
        let out = g.forward_prepared_coefficients_only(&prepared, &pool.post)?;
        replay_require(
            out.scores_q24
                .iter()
                .zip(&f.u)
                .map(|(g, u)| g.checked_add(*u))
                .collect::<Option<Vec<_>>>()
                == Some(pool.generate.clone()),
            "Generate coefficient graph raw native anchor differs",
        )?;
        replay_require(
            pool.copy == shared::dec::<Vec<i64>>(&f.native["copy_q24"])?
                && serde_json::to_value(&pool.trace)? == f.native["pool"],
            "Generate Copy/fullpool native parity differs",
        )?;
    }
    write(
        a,
        "generate-forward-parity.json",
        &json!({"physical_frames":frames.len(),"weighted_roles":32,
        "all_before_any_backward":true,"native_raw_generate_copy_full_alias_pool":true,
        "frozen_poststate":true,"relaxed_transport":"NOT_RUN","device":"cuda"}),
    )?;
    let mut aggregate = vec![0f32; COUNT];
    let mut weighted_forward_ce = 0f64;
    let mut terms = Vec::new();
    for (i, (f, pool)) in frames.iter().zip(pools).enumerate() {
        shared::progress(a, start)?;
        let out = g.forward_prepared_coefficients_only(&prepared, &pool.post)?;
        let u = Tensor::from_vec(
            f.u.iter()
                .map(|x| (*x as f64 / 16777216.) as f32)
                .collect::<Vec<_>>(),
            VOCAB,
            d,
        )?;
        let hard = Tensor::from_vec(
            pool.generate
                .iter()
                .map(|x| (*x as f64 / 16777216.) as f32)
                .collect::<Vec<_>>(),
            VOCAB,
            d,
        )?;
        let soft = (&out.raw_scores + &u)?;
        let joined = (&hard + (&soft - soft.detach())?)?;
        let copy = Tensor::from_vec(
            pool.copy
                .iter()
                .map(|x| (*x as f64 / 16777216.) as f32)
                .collect::<Vec<_>>(),
            pool.copy.len(),
            d,
        )?;
        let loss = (vocabulary_marginal_loss_with_credit(
            &pool.trace,
            &joined,
            Some(&copy),
            f.target,
            a.credit.policy(),
        )? * f.weight)?;
        weighted_forward_ce += loss.to_scalar::<f32>()? as f64;
        let grads = loss.backward()?;
        let v = grads
            .get(g.unary.as_tensor())
            .ok_or_else(|| bad("Generate unary gradient MISSING; no fill"))?
            .flatten_all()?
            .to_device(&Device::Cpu)?
            .to_vec1::<f32>()?;
        replay_require(
            v.len() == COUNT && v.iter().all(|x| x.is_finite()),
            "Generate unary raw gradient shape/nonfinite",
        )?;
        for (sum, x) in aggregate.iter_mut().zip(&v) {
            *sum += *x;
        }
        let bytes = v.iter().flat_map(|x| x.to_le_bytes()).collect::<Vec<_>>();
        let file = format!("generate-gradient-term-{i:02}.f32le");
        fs::write(a.out.join(&file), &bytes)?;
        terms.push(json!({"physical_index":i,"input_index":f.input,"position":f.position,"id":f.id,
            "target":f.target,"weight":f.weight,"file":file,"bytes":bytes.len(),"sha256":sha256_bytes(&bytes),
            "status":"PRESENT","missing_gradient_filled_zero":false,"all_zero":v.iter().all(|x|*x==0.),
            "l2_norm":v.iter().map(|x|f64::from(*x).powi(2)).sum::<f64>().sqrt(),
            "termination_target":f.target==1,"period_target":f.target==16}));
    }
    let native_ce = read(&a.out.join("initial-original-objective.json"))?["combined"]
        .as_f64()
        .ok_or_else(|| bad("Generate native baseline CE missing"))?;
    replay_require(
        (weighted_forward_ce - native_ce).abs() < 1e-5,
        "Generate floating anchored CE differs from native complete objective",
    )?;
    replay_require(
        aggregate.iter().all(|x| x.is_finite()),
        "Generate aggregate nonfinite",
    )?;
    let bytes = aggregate
        .iter()
        .flat_map(|x| x.to_le_bytes())
        .collect::<Vec<_>>();
    fs::write(a.out.join("generate-gradient.f32le"), &bytes)?;
    write(
        a,
        "generate-gradient-receipt.json",
        &json!({"active_parameter_names":[NAME],"shape":[8,120],
        "coefficient_backward_calls":frames.len(),"weighted_roles":32,"native_combined_ce":native_ce,"float_graph_combined_ce":weighted_forward_ce,"loss_anchor_tolerance":1e-5,"perterm":terms,
        "aggregate":{"file":"generate-gradient.f32le","bytes":bytes.len(),"sha256":sha256_bytes(&bytes)},
        "sum":"ordered physical-frame f32 sum; duplicate task/reference coalesced weight, not32-backward bitwise equivalence",
        "frozen_other_generate_masters":identities(&BTreeMap::from([
            ("generate.pair".into(),g.pair.clone()),("generate.bias".into(),g.bias.clone()),
            ("generate.prototype_choices".into(),g.prototype_choices.clone())]))?}),
    )?;
    // g/prepared/graphs released before allocating the complete guard population.
    Ok(aggregate)
}
struct Incidence {
    offsets: Vec<usize>,
    atoms: Vec<u32>,
}
fn incidence(
    ng: &NativeGeometricGenerate,
    frames: &[shared::Frame],
    pools: &[shared::Pool],
    legal: &[u32],
) -> Result<Incidence> {
    replay_require(
        frames.len() <= UNION && legal.iter().all(|t| (*t as usize) < VOCAB),
        "Generate incidence domain differs",
    )?;
    let mut counts = vec![0usize; COUNT];
    for pool in pools {
        let state = pool.post.clone();
        for &token in legal {
            let mut keys = [0u32; 12];
            ng.factor_incidence_into(&state, token as usize, &mut keys, &mut Default::default())?;
            for &key in &keys[..8] {
                counts[key as usize] += 1;
            }
        }
    }
    let mut offsets = vec![0usize; COUNT + 1];
    for i in 0..COUNT {
        offsets[i + 1] = offsets[i] + counts[i];
    }
    let mut atoms = vec![0u32; offsets[COUNT]];
    let mut cursor = offsets[..COUNT].to_vec();
    for (row, pool) in pools.iter().enumerate() {
        let state = pool.post.clone();
        for &token in legal {
            let mut keys = [0u32; 12];
            ng.factor_incidence_into(&state, token as usize, &mut keys, &mut Default::default())?;
            for &key in &keys[..8] {
                let k = key as usize;
                atoms[cursor[k]] = ((row as u32) << 12) | token;
                cursor[k] += 1;
            }
        }
    }
    Ok(Incidence { offsets, atoms })
}
type Patches = BTreeMap<usize, PendingGeneratePatch>;
fn view(
    row: usize,
    target: u32,
    caches: &[GeneratePatchCache],
    staged: &Patches,
) -> Result<(u64, u64, u32)> {
    if let Some(p) = staged.get(&row) {
        let s = p.summary();
        Ok((
            p.token_mass(&caches[row], target)?,
            s.total_weight_q31,
            s.chosen_token_id,
        ))
    } else {
        let c = &caches[row];
        Ok((
            c.token_masses()[target as usize],
            c.summary().total_weight_q31,
            c.summary().chosen_token_id,
        ))
    }
}
fn objective(
    frames: &[shared::Frame],
    map: &[usize],
    spec: &shared::ObjectiveSpec,
    caches: &[GeneratePatchCache],
    staged: &Patches,
) -> Result<Value> {
    let roles = spec
        .roles
        .as_ref()
        .ok_or_else(|| bad("Generate explicit episode roles absent"))?;
    let mut task = 0.;
    let mut reference = 0.;
    let mut goodrefs = 0;
    let mut phases = Vec::new();
    let mut masses = Vec::new();
    for &row in map {
        let f = &frames[row];
        let (m, t, c) = view(row, f.target, caches, staged)?;
        replay_require(m > 0 && t > 0, "Generate native objective zero target mass")?;
        masses.push(json!([m, t, c]));
    }
    for role in roles {
        let row = map
            .iter()
            .copied()
            .find(|&row| frames[row].input == role.input && frames[row].position == role.position)
            .ok_or_else(|| bad("Generate objective role frame missing"))?;
        let f = &frames[row];
        let (mass, total, chosen) = view(row, f.target, caches, staged)?;
        let ce = -(mass as f64 / total as f64).ln() * role.weight;
        if role.task {
            task += ce;
            phases.push(json!({"position":f.position,"target":f.target,"chosen":chosen,"target_mass":mass,"total_mass":total}));
        } else {
            reference += ce;
            goodrefs += usize::from(chosen == f.target);
        }
    }
    Ok(
        json!({"combined":task+reference,"task":task,"reference":reference,
        "correct_reference_frames":goodrefs,"phases":phases,"objective_masses":masses,
        "all_phase_winners":phases.iter().all(|x|x["target"]==x["chosen"])}),
    )
}
fn valid_objective(current: &Value, next: &Value) -> Result<bool> {
    Ok(improves(
        current["combined"]
            .as_f64()
            .ok_or_else(|| bad("Generate currentCE absent"))?,
        next["combined"]
            .as_f64()
            .ok_or_else(|| bad("Generate nextCE absent"))?,
    ) && next["correct_reference_frames"] == 17)
}
fn final_gate(original: &Value, next: &Value, guards: bool) -> Result<Value> {
    let combined = improves(
        original["combined"]
            .as_f64()
            .ok_or_else(|| bad("Generate baselineCE absent"))?,
        next["combined"]
            .as_f64()
            .ok_or_else(|| bad("Generate finalCE absent"))?,
    );
    let task = improves(
        original["task"]
            .as_f64()
            .ok_or_else(|| bad("Generate baseline taskCE absent"))?,
        next["task"]
            .as_f64()
            .ok_or_else(|| bad("Generate final taskCE absent"))?,
    );
    Ok(
        json!({"strict_combined_descent":combined,"strict_episode_descent":task,"all15_conditional_winners":next["all_phase_winners"],
        "all17_references":next["correct_reference_frames"]==17,"all380_original_winners":guards,
        "passed":combined && task && next["all_phase_winners"]==true && next["correct_reference_frames"]==17 && guards,
        "actual_wholeanswer_EOS":"NOT_RUN_SEPARATE_QUALIFICATION"}),
    )
}
fn stage_row(
    row: usize,
    changes: &BTreeMap<usize, Vec<(u32, i64)>>,
    caches: &[GeneratePatchCache],
    staged: &mut Patches,
    reducer: &mut NativeVocabularyActions,
) -> Result<()> {
    if let Some(delta) = changes.get(&row) {
        if !staged.contains_key(&row) {
            staged.insert(row, reducer.evaluate_generate_patch(&caches[row], delta)?);
        }
    }
    Ok(())
}
fn digest_staged(
    staged: &Patches,
    frames: &[shared::Frame],
    caches: &[GeneratePatchCache],
) -> Result<String> {
    let rows = staged
        .iter()
        .map(|(&i, p)| {
            let s = p.summary();
            Ok(json!({"row":i,"reference_q24":s.reference_q24,
            "total_weight_q31":s.total_weight_q31,"chosen_token_id":s.chosen_token_id,
            "chosen_weight_q31":s.chosen_weight_q31,
            "target_mass":p.token_mass(&caches[i],frames[i].target)?}))
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(sha256_bytes(&serde_json::to_vec(&rows)?))
}
fn construct(
    a: &Args,
    start: Instant,
    frames: &[shared::Frame],
    map: &[usize],
    spec: &shared::ObjectiveSpec,
    master: &[f32],
    grad: &[f32],
    csr: &Incidence,
    caches: &mut [GeneratePatchCache],
    reducer: &mut NativeVocabularyActions,
) -> Result<(Vec<f32>, Value)> {
    let order = coordinate_order(master, grad)?;
    write(a, "generate-coordinate-order.json", &json!(order))?;
    let baseline = objective(frames, map, spec, caches, &Patches::new())?;
    let mut current = master.to_vec();
    let mut value = baseline.clone();
    let mut epoch = 0u64;
    use std::io::Write as _;
    let file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(a.out.join("generate-construction.json"))?;
    let mut records = std::io::BufWriter::new(file);
    records.write_all(b"{\"policy\":")?;
    serde_json::to_writer(&mut records, &policy())?;
    records.write_all(b",\"coordinate_records\":[")?;
    let mut row_patches = 0u64;
    let mut accepted = 0;
    for (ordinal, &coordinate) in order.iter().enumerate() {
        shared::progress(a, start)?;
        let incumbent = code(current[coordinate])?;
        let old_epoch = epoch;
        let mut alternatives = Vec::with_capacity(14);
        let mut best: Option<(f64, i8, Patches, Value)> = None;
        for proposed in -7i8..=7 {
            if proposed == incumbent {
                continue;
            }
            let d = i64::from(proposed - incumbent) * (1 << 20);
            let mut changes = BTreeMap::<usize, Vec<(u32, i64)>>::new();
            for &atom in &csr.atoms[csr.offsets[coordinate]..csr.offsets[coordinate + 1]] {
                let row = (atom >> 12) as usize;
                let token = atom & 4095;
                let raw = caches[row].generate_scores()[token as usize]
                    .checked_add(d)
                    .ok_or_else(|| bad("Generate unary patch overflow"))?;
                changes.entry(row).or_default().push((token, raw));
            }
            let mut staged = Patches::new();
            for &row in map {
                stage_row(row, &changes, caches, &mut staged, reducer)?;
            }
            let next = objective(frames, map, spec, caches, &staged)?;
            let objective_gate = valid_objective(&value, &next)?;
            let mut checked = Vec::new();
            let mut first: Option<Value> = None;
            if objective_gate {
                for row in 0..GUARDS {
                    if changes.contains_key(&row) {
                        stage_row(row, &changes, caches, &mut staged, reducer)?;
                        checked.push(row);
                    }
                    let (_, _, chosen) = view(row, frames[row].target, caches, &staged)?;
                    if chosen != frames[row].target {
                        first = Some(
                            json!({"row":row,"input_index":frames[row].input,"position":frames[row].position,
                            "required":frames[row].target,"chosen":chosen}),
                        );
                        break;
                    }
                }
            }
            row_patches += staged.len() as u64;
            let feasible = objective_gate && first.is_none();
            let compact_digest = digest_staged(&staged, frames, caches)?;
            alternatives.push(json!({"code":proposed,"delta_code":proposed-incumbent,
                "actual_master_delta":f64::from(proposed)*0.25-f64::from(master[coordinate]),
                "combined":next["combined"],"task":next["task"],"reference":next["reference"],
                "objective_masses":next["objective_masses"],"objective_gate":objective_gate,
                "guard_status":if !objective_gate{"NOT_RUN_OBJECTIVE_INELIGIBLE"}else if first.is_some(){"VETO"}else{"PASS"},
                "checked_guard_ids":checked,"first_guard_failure":first,"feasible":feasible,
                "staged_summary_digest":compact_digest,"changed_atoms":changes.values().map(Vec::len).sum::<usize>(),
                "row_patches":staged.len(),"used_full_reduction_count":staged.values().filter(|p|p.summary().used_full_reduction).count(),
                "maximum_scans":staged.values().filter(|p|p.summary().scanned_maximum).count(),
                "winner_scans":staged.values().filter(|p|p.summary().scanned_winner).count(),
                "potential_affected_rows":changes.len(),"changed_atoms_scope":"all CSR affectedatoms; guard-veto/objective-ineligible stage can visit only subset"}));
            if feasible {
                let ce = next["combined"]
                    .as_f64()
                    .ok_or_else(|| bad("Generate alternativeCE absent"))?;
                if best
                    .as_ref()
                    .is_none_or(|b| lower((ce, proposed), (b.0, b.1)))
                {
                    best = Some((ce, proposed, staged, next));
                }
            }
        }
        replay_require(
            alternatives.len() == 14,
            "Generate full legal alternatives count differs",
        )?;
        let selected = if let Some((ce, q, pending, next)) = best {
            // Consumer validates every opaque identity/revision before any infallible application.
            reducer.commit_generate_patch_batch(caches, pending.into_iter().collect())?;
            current[coordinate] = f32::from(q) * 0.25;
            value = next;
            epoch += 1;
            accepted += 1;
            json!({"code":q,"combined":ce,"status":"committed","restaged_rows":0})
        } else {
            json!({"code":incumbent,"status":"unchanged","restaged_rows":0})
        };
        let record = json!({"order":ordinal,"index":coordinate,"incumbent_epoch":old_epoch,
            "incumbent_code":incumbent,"original_master":master[coordinate],"gradient":grad[coordinate],
            "initial_zero_gradient":grad[coordinate]==0.,"alternatives":alternatives,
            "selected":selected,"epoch_after":epoch});
        if ordinal > 0 {
            records.write_all(b",")?;
        }
        serde_json::to_writer(&mut records, &record)?;
        records.flush()?;
    }
    records.write_all(b"],\"summary\":")?;
    serde_json::to_writer(
        &mut records,
        &json!({"coordinates":COUNT,"alternatives":COUNT*14,
        "accepted_coordinates":accepted,"row_patches":row_patches,"initial":baseline,"final":value,
        "revisited_coordinates":0,"selected_pending_restage_count":0}),
    )?;
    records.write_all(b"}\n")?;
    records.flush()?;
    Ok((current, value))
}

fn compact_numeric_projection(prior: &Value) -> Result<u64> {
    // Remove the authenticated previous donor-cache allocation, which does not
    // exist in Generate-only construction. Account for global unary CSR, caches,
    // two simultaneous worst-case full pending batches, and compact trial Values.
    let old = prior["numeric_upper_bound"]
        .as_u64()
        .ok_or_else(|| bad("retained numeric projection absent"))?;
    let donor = prior["donor_cache_cap"]
        .as_u64()
        .ok_or_else(|| bad("retained donor cache projection absent"))?;
    let old_pools = prior["actual_retained_guard_pool_bytes"]
        .as_u64()
        .ok_or_else(|| bad("retained guard pool bound absent"))?;
    // Existing old projection retains both current/staged guard pools. These
    // are released before the all-code constructor; numerical scores live only
    // in opaque caches. Typed immutable donor/state/Copy/U witnesses remain.
    let removed = donor + 2 * old_pools;
    let additions = (UNION*VOCAB*8*4)as u64 // packed unary CSR
        +(UNION*VOCAB*8*3)as u64 // cache Generate/weights/masses
        +2*(UNION*VOCAB*8*3)as u64 // best and currently staged full fallback
        +16*1024*1024 // streamed one-coordinate alternative receipts/buffers
        +8*1024*1024; // offsets/atoms groups/container margin
    old.checked_sub(removed)
        .and_then(|x| x.checked_add(additions))
        .ok_or_else(|| bad("Generate numeric phase bound overflow"))
}
fn resource_projection(a: &Args, c: &Config, frames: &[shared::Frame]) -> Result<()> {
    let previous = read(
        &c.retained_episode_root
            .join("trajectory-resource-projection.json"),
    )?;
    let previous_pre = read(
        &c.retained_episode_root
            .join("episode-pregradient-resource-projection.json"),
    )?;
    let full = frames.iter().try_fold(0u64, |sum, f| {
        Ok::<_, Box<dyn std::error::Error>>(sum + serde_json::to_vec(&f.native)?.len() as u64)
    })?;
    let numeric = compact_numeric_projection(&previous)?;
    let phase_ram = previous_pre["process_ram_projection_bytes"]
        .as_u64()
        .ok_or_else(|| bad("retained measured phase RAM projection absent"))?;
    let process = phase_ram + 64 * 1024 * 1024;
    let previous_total = size(&c.retained_episode_root)?;
    let removed_report = size(&c.retained_episode_root.join("prefix-construction.json"))?;
    let report = previous_total
        .checked_sub(removed_report)
        .and_then(|n| n.checked_add(64 * 1024 * 1024 + 16 * 1024 * 1024))
        .ok_or_else(|| bad("Generate report projection overflow"))?;
    // The new cache/CSR coexistence does not include old donor vectors or graphs.
    let cache = (UNION * VOCAB * 8 * 4 + UNION * VOCAB * 8 * 3 + 2 * UNION * VOCAB * 8 * 3) as u64
        + 8 * 1024 * 1024;
    write(
        a,
        "generate-resource-projection.json",
        &json!({"prior_projection":previous,
        "prior_pregradient_projection":previous_pre,"actual_original31_serialized_bytes":full,
        "numeric_upper_bound":numeric,"numeric_cap":512*1024*1024u64,
        "cache_upper_bound":cache,"cache_cap":CACHE_CAP,"process_ram_projection_bytes":process,
        "process_ram_cap":4*1024*1024*1024u64,"report_projection_bytes":report,"report_cap":a.maximum_report_bytes,
        "retained_previous_output_bytes":previous_total,"replaced_previous_construction_bytes":removed_report,
        "compact_alternative_receipt_reserve":16*1024*1024u64,"construction_receipts":"streamed coordinate records, never retain13440 alternative Values in RAM","projected_alternatives":13440,
        "graph_lifetime":"31 streamed coefficient graphs; Generate/device/prepared tensors dropped before guard380 load",
        "constructor_lifetime":"guard full pool buffers dropped after opaque cache admission, before CSR and all-code construction",
        "cache_scope":"CSR+opaque caches+best pending+current pending+container overhead; model/device RAM is separate",
        "fresh_backwards_completed":0,"estimate_not_hard_stop":true}),
    )?;
    replay_require(
        numeric <= 512 * 1024 * 1024
            && cache <= CACHE_CAP as u64
            && process <= 4 * 1024 * 1024 * 1024
            && report + 1048576 < a.maximum_report_bytes,
        "Generate complete numerical/export phase projection exceeded",
    )?;
    Ok(())
}
fn compact_pool(pool: &mut shared::Pool) {
    pool.trace.actions.clear();
    pool.trace.actions.shrink_to_fit();
}
pub(super) fn run(a: &Args, start: Instant, d: &Device) -> Result<Value> {
    validate_settings(a)?;
    replay_require(!d.is_cpu(), "Fresh Generate coefficients require CUDA")?;
    let c = a
        .generate_episode_learning
        .as_ref()
        .ok_or_else(|| bad("Generate config absent"))?;
    let prior = shared::sealed(
        &c.retained_episode_root,
        &c.expected_report_sha256,
        &c.expected_manifest_sha256,
    )?;
    replay_require(
        prior["status"] == "COMPLETED"
            && prior["mode"] == "prefix_episode_progression_learning"
            && prior["weighted_roles"] == 32
            && prior["unique_objective_frames"] == 31
            && prior["all_original380_preserved"] == true,
        "Generate retained episode authority differs",
    )?;
    replay_require(
        read(&c.retained_episode_root.join("config.json"))?["prefix_fragment_learning"]
            == serde_json::to_value(&c.original_inputs)?,
        "Generate inherited original inputs differ from sealed episode",
    )?;
    let original = ContinuationParent::from_checkpoint(&a.checkpoint)?;
    replay_require(
        original.binding.metadata_sha256 == shared::SOURCE
            && sha256_bytes(&original.generate) == shared::G_SHA
            && sha256_file(&a.checkpoint.join("continuation-field.bin"))? == shared::U_SHA,
        "Generate original Source/G/U differs",
    )?;
    let ng = NativeGeometricGenerate::from_bytes(&original.generate, original.integer.binding())?;
    let field = NativeContinuationField::from_bytes(
        &fs::read(a.checkpoint.join("continuation-field.bin"))?,
        &original.binding,
        &ng,
    )?;
    let (mut objective_frames, spec) =
        prefix::prepare_generate_objective(a, &c.original_inputs, &original)?;
    replay_require(
        objective_frames.len() == 31 && spec.roles.as_ref().is_some_and(|r| r.len() == 32),
        "Generate role/physical population differs",
    )?;
    let mut reducer =
        NativeVocabularyActions::new(original.integer.binding().clone(), &original.exp)?;
    let mut objective_pools = objective_frames
        .iter()
        .map(|f| saved_pool(f, &mut reducer))
        .collect::<Result<Vec<_>>>()?;
    for f in &objective_frames {
        let state = shared::codes(&f.native["continuation"]["state_codes"])?;
        let mut u = vec![0; VOCAB];
        field.score_delta_into(&state, &ng, &mut u, &mut Default::default())?;
        replay_require(u == f.u, "Generate frozen U state/score arithmetic differs")?;
    }
    let baseline = shared::objective_for_spec(&objective_frames, &objective_pools, &spec)?;
    write(a, "initial-original-objective.json", &baseline)?;
    resource_projection(a, c, &objective_frames)?;
    let g = gradient(a, start, &original, &objective_frames, &objective_pools, d)?;
    // Full graph/native parity is complete. Remove redundant action Values before
    // the protected population loader, retaining the p3 addition authority.
    slim(&mut objective_frames)?;
    for pool in &mut objective_pools {
        compact_pool(pool);
    }
    let (mut frames, mut pools, authority) = prefix::prepare_generate_guards(
        a,
        &c.original_inputs,
        &original,
        &objective_frames,
        &spec,
    )?;
    replay_require(
        frames.len() == GUARDS && pools.len() == GUARDS,
        "Generate guard population differs",
    )?;
    let mut map = Vec::with_capacity(31);
    for (f, pool) in objective_frames.into_iter().zip(objective_pools) {
        if let Some(row) = frames
            .iter()
            .position(|g| g.input == f.input && g.position == f.position)
        {
            replay_require(
                frames[row].id == f.id
                    && frames[row].prefix == f.prefix
                    && pools[row].generate == pool.generate
                    && pools[row].copy == pool.copy
                    && pools[row].post == pool.post
                    && pools[row].donor == pool.donor,
                "Generate overlapping objective/guard epoch differs",
            )?;
            frames[row] = f;
            map.push(row);
        } else {
            map.push(frames.len());
            frames.push(f);
            pools.push(pool);
        }
    }
    replay_require(
        frames.len() == UNION
            && pools.len() == UNION
            && frames
                .iter()
                .map(|f| (f.input, f.position))
                .collect::<BTreeSet<_>>()
                .len()
                == UNION,
        "Generate391 unique union differs",
    )?;
    let identities_json=frames.iter().enumerate().map(|(i,f)|json!({"row":i,"input_index":f.input,
        "position":f.position,"id":f.id,"actual_prefix_ids":f.prefix,"target_label_only":f.target,
        "zero_weight_guard":i<GUARDS,"guard_weight":0.,"coalesced_objective_weight":f.weight,
        "post_state":pools[i].post.iter().map(|x|x.index()).collect::<Vec<_>>(),"donor":pools[i].donor}))
        .collect::<Vec<_>>();
    write(
        a,
        "generate-population.json",
        &json!({"rows":identities_json,"objective_row_map":map,
        "roles":spec.roles,"guard_population":authority,"objective_physical_frames":31,
        "weighted_roles":32,"guards":GUARDS,"unique_union":UNION}),
    )?;
    let legal = reducer.legal_token_ids().to_vec();
    let csr = incidence(&ng, &frames, &pools, &legal)?;
    let bytes = csr
        .atoms
        .iter()
        .flat_map(|x| x.to_le_bytes())
        .collect::<Vec<_>>();
    write(
        a,
        "generate-incidence.json",
        &json!({"legal_generate_ids":legal,"postings":csr.atoms.len(),
        "offsets":csr.offsets,"packed_atoms_sha256":sha256_bytes(&bytes),
        "key_encoding":"lane*120+compose(inverse(poststate[lane]),prototype[token,lane]); first8 core factor_incidence_into keys",
        "atom_encoding":"row<<12|token; legal admitted IDs only, full raw4096 arrays retained",
        "source_binding":original.binding,"generate_sha256":shared::G_SHA}),
    )?;
    drop(bytes);
    let mut caches = pools
        .iter()
        .zip(&frames)
        .map(|(p, f)| {
            reducer.prepare_generate_patch_cache(p.generate.clone(), f.ids.clone(), p.copy.clone())
        })
        .collect::<std::result::Result<Vec<_>, _>>()?;
    for (cache, pool) in caches.iter().zip(&pools) {
        replay_require(
            cache.summary().chosen_token_id == pool.trace.summary.chosen_token_id
                && cache.summary().total_weight_q31 == pool.trace.summary.total_weight_q31
                && pool
                    .trace
                    .token_masses
                    .iter()
                    .all(|m| cache.token_masses()[m.token_id as usize] == m.weight_q31),
            "Generate cache full masses differ",
        )?;
    }
    // Save only immutable post/donor witnesses. All current numerical state lives
    // in the target-free caches, not duplicate current/staged Pool buffers.
    let posts = pools.iter().map(|p| p.post.clone()).collect::<Vec<_>>();
    let donors = pools.iter().map(|p| p.donor).collect::<Vec<_>>();
    drop(pools);
    for f in &mut frames {
        let continuation = f.native["continuation"].clone();
        f.native = json!({"continuation":continuation});
        f.prefix_trace = None;
    }
    let initial = objective(&frames, &map, &spec, &caches, &Patches::new())?;
    replay_require(
        (initial["combined"]
            .as_f64()
            .ok_or_else(|| bad("Generate baseline absent"))?
            - baseline["combined"]
                .as_f64()
                .ok_or_else(|| bad("Generate saved baseline absent"))?)
        .abs()
            < 1e-12,
        "Generate cache original objective differs",
    )?;
    let actual = GenerateLearningWeights::from_native(
        original.integer.binding().clone(),
        &ng,
        &Device::Cpu,
    )?;
    restore(
        &a.checkpoint.join("generate-source"),
        &read(&a.checkpoint.join("generate-source/metadata.json"))?["parameters"],
        &actual.parameters(),
        &Device::Cpu,
    )?;
    let all_bits = np::snapshot(&actual.parameters())?;
    let master = all_bits
        .get(NAME)
        .ok_or_else(|| bad("Generate unary master absent"))?
        .clone();
    replay_require(
        master.len() == COUNT && actual.export_native()?.to_bytes()? == original.generate,
        "Generate actual fractional authority differs",
    )?;
    fs::write(
        a.out.join("generate-initial-masters.f32le"),
        master
            .iter()
            .flat_map(|x| x.to_le_bytes())
            .collect::<Vec<_>>(),
    )?;
    write(
        a,
        "generate-original-master-binding.json",
        &json!({"source":a.checkpoint.join("generate-source"),
        "parameters":identities(&actual.parameters())?,"native_sha256":shared::G_SHA,
        "all_original_f32_restored_before_graph":true,"active_parameter_names":[NAME]}),
    )?;
    drop(actual);
    let (current, value) = construct(
        a,
        start,
        &frames,
        &map,
        &spec,
        &master,
        &g,
        &csr,
        &mut caches,
        &mut reducer,
    )?;
    let all_guards = (0..GUARDS).all(|r| caches[r].summary().chosen_token_id == frames[r].target);
    let gate = final_gate(&initial, &value, all_guards)?;
    write(a, "final-objective.json", &value)?;
    write(
        a,
        "final-trajectory-guards.json",
        &json!({"all_original_winners":all_guards,"guards":GUARDS,
        "terms":(0..GUARDS).map(|i|json!({"guard_index":i,"input_index":frames[i].input,
            "position":frames[i].position,"required_original_winner":frames[i].target,
            "chosen":caches[i].summary().chosen_token_id,"pool":caches[i].summary()})).collect::<Vec<_>>()}),
    )?;
    // Coherent native artifact is retained regardless of finite gate outcome;
    // no automatic autoregressive rollout is performed by this runner.
    drop(csr);
    drop(g);
    let loaded = load_joint_continuation(a, &original, &Device::Cpu)?;
    let frozen_source = identities(&loaded.source.parameters())?;
    let params = loaded.generate.parameters();
    let saved = np::snapshot(&params)?;
    replay_require(
        np::same_bits(&saved, &all_bits),
        "Generate independently loaded original masters differ",
    )?;
    let u = ContinuationLearningWeights::from_native(
        &field,
        &ng,
        original.integer.binding(),
        &Device::Cpu,
    )?;
    restore(
        &a.checkpoint.join("continuation-source"),
        &original.receipt["continuation_parameters"],
        &u.parameters(),
        &Device::Cpu,
    )?;
    let u_bits = identities(&u.parameters())?;
    let receipt = np::attempt_restored(&params, &saved, || {
        let mut expected = saved.clone();
        expected.insert(NAME.into(), current.clone());
        np::restore(&params, &expected)?;
        replay_require(
            np::same_bits(&np::snapshot(&params)?, &expected)
                && identities(&loaded.source.parameters())? == frozen_source,
            "Generate sole-family/frozen master guard differs",
        )?;
        let (cp, rebound, mut receipt) = joint_checkpoint(a, 1, &loaded, &u)?;
        replay_require(
            cp.binding == original.binding
                && cp.bridge == original.bridge
                && rebound.packed_unary() == field.packed_unary()
                && identities(&u.parameters())? == u_bits,
            "Generate export changed Source/bridge/U numerical coefficients",
        )?;
        // Generic checkpoint stores sidecar native payloads. Restore the ORIGINAL
        // sidecar floating-master files too, whose Source/Cue bindings are fixed.
        for family in ["cue", "prefix"] {
            let source = a.checkpoint.join(family);
            let dest = a.out.join("checkpoint-0001").join(family);
            for entry in fs::read_dir(&source)? {
                let entry = entry?;
                if entry.file_type()?.is_file() {
                    let leaf = entry.file_name();
                    if dest.join(&leaf).exists() {
                        replay_require(
                            fs::read(dest.join(&leaf))? == fs::read(entry.path())?,
                            "Generate frozen sidecar numerical/native metadata differs",
                        )?;
                    } else {
                        fs::copy(entry.path(), dest.join(leaf))?;
                    }
                }
            }
        }
        for family in ["cue-source", "prefix-source"] {
            if a.checkpoint.join(family).exists() {
                copy_directory(
                    &a.checkpoint.join(family),
                    &a.out.join("checkpoint-0001").join(family),
                )?;
            }
        }
        let mut receipt_expected = expected.clone();
        let disk_g = GenerateLearningWeights::from_native(
            cp.integer.binding().clone(),
            &NativeGeometricGenerate::from_bytes(&cp.generate, cp.integer.binding())?,
            &Device::Cpu,
        )?;
        restore(
            &a.out.join("checkpoint-0001/generate-source"),
            &read(&a.out.join("checkpoint-0001/generate-source/metadata.json"))?["parameters"],
            &disk_g.parameters(),
            &Device::Cpu,
        )?;
        replay_require(
            np::same_bits(&np::snapshot(&disk_g.parameters())?, &receipt_expected),
            "Generate exported all floating masters differ",
        )?;
        receipt_expected.clear();
        drop(disk_g);
        receipt["mode"] = json!("generate_episode_learning");
        receipt["policy"] = policy();
        receipt["active_parameter_names"] = json!([NAME]);
        receipt["optimizer_updates"] = json!(0);
        receipt["fresh_adam"] = json!(false);
        receipt["new_gradients"] = json!(1);
        receipt["coefficient_backward_calls"] = json!(31);
        receipt["credit_scope"]=json!("only Generate unary960 gradients extracted/proposed; fixed factual native state/prototypes coefficient-only Q4 STE; frozen pair/bias graph may receive incidental autodiff, never extracted/updated; no Prefix/Cue/Context/donor surrogate");
        receipt["parent_report_sha256"] = json!(shared::P_REPORT);
        receipt["parent_manifest_sha256"] = json!(shared::P_SEAL);
        receipt["frozen_numerical_scope"]=json!("all Source/Cue/Prefix/bridge/Generate pair+bias+prototypes/U masters; only unary960 float/native codes change; U metadata honestly rebound to new Generate");
        receipt["generate_sha256"] = json!(sha256_bytes(&cp.generate));
        receipt["candidate_native_steps"] = json!(UNION);
        for leaf in [
            "checkpoint-0001/receipt.json",
            "checkpoint-0001/continuation-source/metadata.json",
        ] {
            fs::write(a.out.join(leaf), serde_json::to_vec_pretty(&receipt)?)?;
        }
        let cp = ContinuationParent::from_checkpoint(&a.out.join("checkpoint-0001"))?;
        // Final pool reduction matches the exact patch caches. Recompute each
        // finite full trace once; only one compact pool vector exists per row.
        let mut final_pools = Vec::with_capacity(UNION);
        for (i, cache) in caches.iter().enumerate() {
            let mut trace = reducer.reduce_trace(
                cache.generate_scores(),
                cache.copy_token_ids(),
                cache.copy_scores(),
            )?;
            replay_require(
                trace.summary.chosen_token_id == cache.summary().chosen_token_id
                    && trace.summary.total_weight_q31 == cache.summary().total_weight_q31
                    && trace
                        .token_masses
                        .iter()
                        .all(|m| cache.token_masses()[m.token_id as usize] == m.weight_q31),
                "Generate final authoritative mass parity differs",
            )?;
            trace.actions.clear();
            trace.actions.shrink_to_fit();
            final_pools.push(shared::Pool {
                donor: donors[i],
                post: posts[i].clone(),
                generate: cache.generate_scores().to_vec(),
                copy: cache.copy_scores().to_vec(),
                base_copy: frames[i].base_copy.clone(),
                trace,
            });
        }
        shared::reload_guard_candidate(a, &cp, &rebound, &frames, &final_pools)?;
        Ok(receipt)
    });
    replay_require(
        np::same_bits(&np::snapshot(&params)?, &saved),
        "Generate original parent masters not restored",
    )?;
    let receipt = receipt?;
    Ok(
        json!({"schema":"uor-r4.generate-episode-learning-report/1","status":"COMPLETED","mode":"generate_episode_learning",
        "source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT"),"policy":policy(),"selected_model":false,
        "finite_episode_positive":gate["passed"],"useful_candidate":false,"final_gate":gate,
        "baseline_objective":initial,"candidate_objective":value,"candidate_receipt":receipt,
        "all_original380_preserved":all_guards,"weighted_roles":32,"unique_objective_frames":31,
        "candidate_native_steps":UNION,"prefix_backward_calls":0,"generate_backward_calls":31,
        "new_generate_gradients":1,"optimizer_updates":0,"parent_master_bits_restored":true,
        "autoregressive_rollout":"NOT_RUN_SEPARATE_ARTIFACT_CHECK","actual9":"NOT_RUN","full512":"NOT_RUN",
        "original_episode_authority":c.retained_episode_root,"episode_request":c.original_inputs.episode}),
    )
}

/// Admission for the cheap artifact-only evaluator. Independent arithmetic audit
/// remains a final delivery requirement, not a prerequisite to this cheap check.
pub(super) fn authenticate_positive_artifact(
    a: &Args,
    c: &prefix::ArtifactConfig,
    report: &Value,
) -> Result<()> {
    let pin = c
        .generate_episode_candidate
        .as_ref()
        .ok_or_else(|| bad("Generate artifact authority missing"))?;
    replay_require(
        report["status"] == "COMPLETED"
            && report["mode"] == "generate_episode_learning"
            && report["selected_model"] == false
            && report["finite_episode_positive"] == true
            && report["final_gate"]["passed"] == true
            && report["all_original380_preserved"] == true
            && report["weighted_roles"] == 32
            && report["unique_objective_frames"] == 31
            && report["generate_backward_calls"] == 31
            && report["prefix_backward_calls"] == 0
            && report["candidate_native_steps"] == 391
            && report["optimizer_updates"] == 0,
        "Generate artifact lacks positive complete native construction",
    )?;
    let root = &c.retained_candidate_root;
    let gradients = read(&root.join("generate-gradient-receipt.json"))?;
    replay_require(
        gradients["coefficient_backward_calls"] == 31
            && gradients["weighted_roles"] == 32
            && gradients["active_parameter_names"] == json!([NAME]),
        "Generate gradient inventory scope differs",
    )?;
    let rows = gradients["perterm"]
        .as_array()
        .ok_or_else(|| bad("Generate perterm gradients absent"))?;
    replay_require(rows.len() == 31, "Generate perterm gradient count differs")?;
    for row in rows {
        let leaf = row["file"]
            .as_str()
            .ok_or_else(|| bad("Generate raw gradient filename absent"))?;
        replay_require(
            Path::new(leaf).components().count() == 1
                && fs::metadata(root.join(leaf))?.len() == 3840
                && sha256_file(&root.join(leaf))?
                    == row["sha256"]
                        .as_str()
                        .ok_or_else(|| bad("Generate raw gradient hash absent"))?,
            "Generate raw gradient receipt differs",
        )?;
    }
    replay_require(
        fs::metadata(root.join("generate-gradient.f32le"))?.len() == 3840
            && sha256_file(&root.join("generate-gradient.f32le"))?
                == gradients["aggregate"]["sha256"]
                    .as_str()
                    .ok_or_else(|| bad("Generate aggregate gradient hash absent"))?,
        "Generate aggregate gradient differs",
    )?;
    let construction = read(&root.join("generate-construction.json"))?;
    let coords = construction["coordinate_records"]
        .as_array()
        .ok_or_else(|| bad("Generate coordinate records absent"))?;
    replay_require(
        coords.len() == COUNT
            && construction["summary"]["coordinates"] == 960
            && construction["summary"]["alternatives"] == 13440
            && construction["summary"]["revisited_coordinates"] == 0,
        "Generate exhaustive once-only policy receipt differs",
    )?;
    let mut indices = BTreeSet::new();
    for row in coords {
        let index = shared::idx(&row["index"])?;
        let incumbent = row["incumbent_code"]
            .as_i64()
            .ok_or_else(|| bad("Generate incumbent code missing"))?;
        let alternatives = row["alternatives"]
            .as_array()
            .ok_or_else(|| bad("Generate alternatives missing"))?;
        let codes = alternatives
            .iter()
            .map(|x| {
                x["code"]
                    .as_i64()
                    .ok_or_else(|| bad("Generate alternative code missing"))
            })
            .collect::<Result<BTreeSet<_>>>()?;
        replay_require(
            index < COUNT
                && indices.insert(index)
                && alternatives.len() == 14
                && codes
                    == (-7i64..=7)
                        .filter(|q| *q != incumbent)
                        .collect::<BTreeSet<_>>(),
            "Generate exhaustive code population differs",
        )?;
    }
    drop(construction);
    let guards = read(&root.join("final-trajectory-guards.json"))?;
    let terms = guards["terms"]
        .as_array()
        .ok_or_else(|| bad("Generate guard receipt missing"))?;
    replay_require(
        guards["all_original_winners"] == true
            && terms.len() == GUARDS
            && terms
                .iter()
                .all(|t| t["chosen"] == t["required_original_winner"]),
        "Generate complete original guard retention differs",
    )?;
    replay_require(
        report["candidate_objective"]["all_phase_winners"] == true
            && report["candidate_objective"]["correct_reference_frames"] == 17,
        "Generate final wholeepisode/reference gate differs",
    )?;

    let parent =
        ContinuationParent::from_checkpoint(&c.retained_intermediate_root.join("checkpoint-0001"))?;
    let candidate = ContinuationParent::from_checkpoint(&a.checkpoint)?;
    replay_require(
        parent.binding == candidate.binding
            && parent.bridge == candidate.bridge
            && parent.cue == candidate.cue
            && parent.joint == candidate.joint
            && parent.prefix == candidate.prefix
            && sha256_bytes(&candidate.generate) == pin.expected_generate_sha256
            && sha256_file(&a.checkpoint.join("continuation-field.bin"))?
                == pin.expected_continuation_sha256,
        "Generate candidate frozen native families differ",
    )?;
    let original_native =
        NativeGeometricGenerate::from_bytes(&parent.generate, parent.integer.binding())?;
    let current_native =
        NativeGeometricGenerate::from_bytes(&candidate.generate, candidate.integer.binding())?;
    for leaf in [
        "cue/cue-source-f32.bin",
        "prefix/prefix-source-f32.bin",
        "source/consumer/context.safetensors",
        "source/consumer/potential.safetensors",
    ] {
        let old = c
            .retained_intermediate_root
            .join("checkpoint-0001")
            .join(leaf);
        let new = a.checkpoint.join(leaf);
        if old.is_file() {
            replay_require(
                new.is_file() && sha256_file(&old)? == sha256_file(&new)?,
                "Generate candidate changed a frozen Source/sidecar floating payload",
            )?;
        }
    }
    let original_u = NativeContinuationField::from_bytes(
        &fs::read(
            c.retained_intermediate_root
                .join("checkpoint-0001/continuation-field.bin"),
        )?,
        &parent.binding,
        &original_native,
    )?;
    let current_u = NativeContinuationField::from_bytes(
        &fs::read(a.checkpoint.join("continuation-field.bin"))?,
        &candidate.binding,
        &current_native,
    )?;
    replay_require(
        original_u.packed_unary() == current_u.packed_unary(),
        "Generate rebound U numerical payload changed",
    )?;
    let dev = Device::Cpu;
    let original_weights = GenerateLearningWeights::from_native(
        parent.integer.binding().clone(),
        &original_native,
        &dev,
    )?;
    restore(
        &c.retained_intermediate_root
            .join("checkpoint-0001/generate-source"),
        &read(
            &c.retained_intermediate_root
                .join("checkpoint-0001/generate-source/metadata.json"),
        )?["parameters"],
        &original_weights.parameters(),
        &dev,
    )?;
    let current_weights = GenerateLearningWeights::from_native(
        candidate.integer.binding().clone(),
        &current_native,
        &dev,
    )?;
    restore(
        &a.checkpoint.join("generate-source"),
        &read(&a.checkpoint.join("generate-source/metadata.json"))?["parameters"],
        &current_weights.parameters(),
        &dev,
    )?;
    replay_require(
        current_weights.export_native()?.to_bytes()? == candidate.generate,
        "Generate candidate actual floating masters do not export to native",
    )?;
    let original = np::snapshot(&original_weights.parameters())?;
    let current = np::snapshot(&current_weights.parameters())?;
    replay_require(
        original
            .iter()
            .filter(|(n, _)| n.as_str() != NAME)
            .all(|(n, v)| {
                current.get(n).is_some_and(|x| {
                    v.iter().zip(x).all(|(a, b)| a.to_bits() == b.to_bits()) && v.len() == x.len()
                })
            }),
        "Generate candidate changed a frozen floating family",
    )?;
    let metadata = read(&a.checkpoint.join("generate-source/metadata.json"))?;
    replay_require(
        metadata["parameters"][NAME]["shape"] == json!([8, 120])
            && metadata["parameters"][NAME]["sha256"] == pin.expected_unary_master_sha256,
        "Generate unary actual master identity differs",
    )?;
    let population = read(&root.join("generate-population.json"))?;
    replay_require(
        population["rows"]
            .as_array()
            .is_some_and(|r| r.len() == 391),
        "Generate reloaded population missing",
    )?;
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn full_legal_order_uses_fractional_delta_and_keeps_zero_coordinates() -> Result<()> {
        let mut m = vec![0.; COUNT];
        let mut g = vec![0.; COUNT];
        m[1] = 0.13;
        g[1] = -1.;
        m[2] = -0.13;
        g[2] = 1.;
        let order = coordinate_order(&m, &g)?;
        assert_eq!(order.len(), COUNT);
        assert_eq!(order[0], 1);
        assert_eq!(order[1], 2);
        assert_eq!(order.iter().copied().collect::<BTreeSet<_>>().len(), COUNT);
        assert_eq!(code(0.13)?, 1);
        assert_eq!(
            (-7i8..=7)
                .filter(|q| *q != code(m[1]).unwrap_or(99))
                .count(),
            14
        );
        assert!(coordinate_order(&m, &vec![f32::NAN; COUNT]).is_err());
        Ok(())
    }
    #[test]
    fn native_ce_tie_and_reference_veto_do_not_select_gradient_direction() -> Result<()> {
        assert!(lower((1., -7), (1., 7)));
        assert!(!lower((1., 7), (1., -7)));
        let current = json!({"combined":2.});
        let next = json!({"combined":1.,"correct_reference_frames":16});
        assert!(!valid_objective(&current, &next)?);
        assert!(valid_objective(
            &current,
            &json!({"combined":1.,"correct_reference_frames":17})
        )?);
        assert!(!improves(2., 2. - 1e-12));
        Ok(())
    }
    #[test]
    fn terminal_winner_and_original_guard_are_independent_final_gates() -> Result<()> {
        let original = json!({"combined":4.,"task":2.});
        let wrong = json!({"combined":3.,"task":1.,"all_phase_winners":false,"correct_reference_frames":17});
        assert_eq!(final_gate(&original, &wrong, true)?["passed"], false);
        let right =
            json!({"combined":3.,"task":1.,"all_phase_winners":true,"correct_reference_frames":17});
        assert_eq!(final_gate(&original, &right, false)?["passed"], false);
        assert_eq!(final_gate(&original, &right, true)?["passed"], true);
        Ok(())
    }
    #[test]
    fn late_native_guard_patch_is_discarded_and_same_epoch_alternatives_remain_valid() -> Result<()>
    {
        use uor_r4_integer::geometric_source_actions::SourceActionBinding;
        const TOK:&[u8]=br#"{"pre_tokenizer":{"type":"ByteLevel","add_prefix_space":false},"model":{"type":"BPE","vocab":{"<|bos|>":0,"<|eos|>":1,"<|unk|>":2,".":3,"a":4,"b":5},"merges":[]},"added_tokens":[{"id":0,"content":"<|bos|>"},{"id":1,"content":"<|eos|>"},{"id":2,"content":"<|unk|>"}]}"#;
        let exp = (0..uor_r4_integer::geometric_read::EXP_TABLE_LEN)
            .flat_map(|i| {
                (((-(i as f64) / 256.).exp() * (1u64 << 31) as f64).round() as u32).to_le_bytes()
            })
            .collect::<Vec<_>>();
        let mut reducer = NativeVocabularyActions::new(SourceActionBinding::new(TOK)?, &exp)?;
        let mut caches = (0..GUARDS)
            .map(|_| {
                reducer.prepare_generate_patch_cache(
                    vec![-4 << 24, -4 << 24, -4 << 24, -4 << 24, 0, -1 << 20],
                    vec![4, 4],
                    vec![-3 << 24, -3 << 24],
                )
            })
            .collect::<std::result::Result<Vec<_>, _>>()?;
        let before = caches
            .iter()
            .map(|c| c.generate_scores().to_vec())
            .collect::<Vec<_>>();
        let mut changes = BTreeMap::new();
        for row in 0..GUARDS {
            // The final guard alone changes winner. Generate5 moves from
            // -1 to +2 native units: exp(.125) exceeds 1+2*exp(-3), while
            // the three-unit displacement remains inside legal Q4 bounds.
            let token = if row + 1 == GUARDS { 5 } else { 4 };
            changes.insert(row, vec![(token, 2 << 20)]);
        }
        let mut staged = Patches::new();
        for row in 0..GUARDS {
            stage_row(row, &changes, &caches, &mut staged, &mut reducer)?;
        }
        assert_eq!(staged.len(), GUARDS);
        assert!((0..GUARDS - 1).all(|row| staged[&row].summary().chosen_token_id == 4));
        assert_eq!(staged[&(GUARDS - 1)].summary().chosen_token_id, 5);
        drop(staged);
        assert!(caches
            .iter()
            .zip(&before)
            .all(|(c, b)| c.generate_scores() == b && c.revision() == 0));
        // Another legal alternative at the identical incumbent revision can commit.
        let patch = reducer.evaluate_generate_patch(&caches[0], &[(4, 1 << 20)])?;
        reducer.commit_generate_patch_batch(&mut caches, vec![(0, patch)])?;
        assert_eq!(caches[0].copy_token_ids(), &[4, 4]);
        assert_eq!(caches[0].revision(), 1);
        assert_eq!(caches[GUARDS - 1].revision(), 0);
        Ok(())
    }
}
