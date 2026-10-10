//! Current-parent D22 credit. No saved historical Frame.u or state adjoint.
//! Native scoring is authoritative; donor utilities are an explicitly local STE.
use super::context_cue_coadapt as shared;
use super::*;
use uor_r4_core::native_geometric::learner::geometric_continuation_field::ContinuationReadCounts;
use uor_r4_integer::{
    geometric_prefix_transport::PrefixTransportTrace,
    geometric_vocabulary_actions::VocabularyActionTrace, h4_tables::H4Code,
};
use uor_r4_training::{
    geometric_generate_learning::{
        vocabulary_log_mass_margin_with_credit, PreparedGenerateLearning,
    },
    geometric_occurrence_consumer::source_realizer::PrefixAngularWeights,
};

pub(super) const WIDTH: usize = 1920;
const FAMILY: usize = 960;
const SCALE: f64 = 16777216.;
#[derive(Clone, Debug, serde::Serialize, Deserialize)]
pub(super) struct Position {
    pub input: usize,
    pub position: usize,
    pub prefix: Vec<u32>,
    pub target: u32,
}
#[derive(Clone)]
pub(super) struct Frame {
    pub position: Position,
    pub id: String,
    prefix_trace: PrefixTransportTrace,
    ids: Vec<u32>,
    base_copy: Vec<i64>,
    query: Vec<H4Code>,
    sources: Vec<Vec<H4Code>>,
    local: Vec<H4Code>,
    epoch_sha256: String,
    teacher: bool,
    target_count: usize,
}
struct Pool {
    donor: usize,
    post: Vec<H4Code>,
    generate: Vec<i64>,
    copy: Vec<i64>,
    u: Vec<i64>,
    trace: VocabularyActionTrace,
}
#[derive(serde::Serialize)]
pub(super) struct NativeEvaluation {
    pub objective: f64,
    pub guard_margins: Vec<f64>,
    pub guard_winners: Vec<u32>,
    pub guard_preserved: bool,
    pub target_positions: usize,
}
pub(super) struct Credit {
    pub objective: f64,
    pub gradient: Vec<f64>,
    pub guard_margins: Vec<f64>,
    pub jacobian: Vec<Vec<f64>>,
    pub receipt: Value,
}
/// Complete canonical targets including EOS, in input/position order.
pub(super) fn objective_positions(episodes: &[Episode]) -> Result<Vec<Position>> {
    replay_require(
        episodes.len() == 512,
        "constructor objective requires frozen512",
    )?;
    let mut positions = Vec::new();
    for (input, e) in episodes.iter().enumerate() {
        replay_require(
            !e.target.is_empty() && e.target.len() <= 32,
            "constructor canonical target extent",
        )?;
        for (position, &target) in e.target.iter().enumerate() {
            positions.push(Position {
                input,
                position,
                prefix: e.target[..position].to_vec(),
                target,
            });
        }
    }
    Ok(positions)
}
fn epoch(p: &ContinuationParent, field: &NativeContinuationField) -> Result<String> {
    Ok(sha256_bytes(&serde_json::to_vec(
        &json!({"source":p.binding,
        "generate":sha256_bytes(&p.generate),"prefix":sha256_bytes(&p.prefix),
        "bridge":sha256_bytes(&p.bridge),"cue":sha256_bytes(&p.cue),"joint":p.joint,
        "field":sha256_bytes(&field.to_bytes()?)}),
    )?))
}
fn earliest(scores: &[i64]) -> Result<usize> {
    let mut best = 0;
    let mut score = *scores
        .first()
        .ok_or_else(|| bad("constructor physical pool empty"))?;
    for (i, &s) in scores.iter().enumerate().skip(1) {
        if s > score {
            best = i;
            score = s;
        }
    }
    Ok(best)
}
fn checked_add(a: &[i64], b: &[i64]) -> Result<Vec<i64>> {
    replay_require(a.len() == b.len(), "constructor score shape")?;
    a.iter()
        .zip(b)
        .map(|(a, b)| {
            a.checked_add(*b)
                .ok_or_else(|| bad("constructor score overflow"))
        })
        .collect()
}
/// Every donor gets its own factual post, joint field and Copy alias energies.
fn pool(
    f: &Frame,
    _p: &ContinuationParent,
    field: &NativeContinuationField,
    generate: &NativeGeometricGenerate,
    bridge: &NativeGeometricReadStateBridge,
    reducer: &mut NativeVocabularyActions,
    donor: usize,
) -> Result<Pool> {
    let source = f
        .sources
        .get(donor)
        .ok_or_else(|| bad("constructor donor missing"))?;
    let mut post = vec![H4Code::IDENTITY; 8];
    let mut actions = post.clone();
    bridge.apply_into(
        &f.query,
        source,
        &mut post,
        &mut actions,
        &mut vec![0; 960],
        &mut BridgeReadCounts::default(),
    )?;
    let (g, copy, u) = joint_scores(&post, &f.local, generate, field, &f.ids, &f.base_copy)?;
    let trace = reducer.reduce_trace(&g, &f.ids, &copy)?;
    Ok(Pool {
        donor,
        post,
        generate: g,
        copy,
        u,
        trace,
    })
}
fn joint_scores(
    post: &[H4Code],
    local: &[H4Code],
    generate: &NativeGeometricGenerate,
    field: &NativeContinuationField,
    ids: &[u32],
    base_copy: &[i64],
) -> Result<(Vec<i64>, Vec<i64>, Vec<i64>)> {
    replay_require(ids.len() == base_copy.len(), "constructor alias extent")?;
    let mut base_g = vec![0; generate.vocab_size()];
    generate.score_into(post, &mut base_g, &mut GenerateReadCounts::default())?;
    let mut u = vec![0; generate.vocab_size()];
    field.score_delta_with_factual_into(
        post,
        local,
        generate,
        &mut u,
        &mut ContinuationReadCounts::default(),
    )?;
    let g = checked_add(&base_g, &u)?;
    let alias_u = ids
        .iter()
        .map(|&id| {
            u.get(id as usize)
                .copied()
                .ok_or_else(|| bad("constructor alias token out of range"))
        })
        .collect::<Result<Vec<_>>>()?;
    let copy = checked_add(base_copy, &alias_u)?;
    Ok((g, copy, u))
}
fn snapshot(step: &NativeBankGenerateStep) -> Result<Value> {
    let u = step
        .continuation
        .as_ref()
        .ok_or_else(|| bad("constructor field witness absent"))?;
    let bridge = step
        .bridge
        .as_ref()
        .ok_or_else(|| bad("constructor bridge witness absent"))?;
    Ok(
        json!({"generate_q24":step.generate_raw_scores_q24,"copy_ids":step.copy_token_ids,
        "copy_q24":step.copy_raw_scores_q24,"post_state":step.post_state.iter().map(|x|x.index()).collect::<Vec<_>>(),
        "pool":step.actions,"bank_trace":step.bank_trace,
        "bridge":{"selected_ordinal":bridge.selected_ordinal,"query_state":bridge.query_state.iter().map(|x|x.index()).collect::<Vec<_>>()},
        "continuation":{"delta_scores_q24":u.delta_scores_q24}}),
    )
}
/// Positions are explicit: objective teacher prefixes and guard own prefixes
/// must be supplied separately. Inputs are never modified or selected by loss.
pub(super) fn capture(
    a: &Args,
    start: Instant,
    p: &ContinuationParent,
    field: &NativeContinuationField,
    episodes: &[Episode],
    positions: &[Position],
) -> Result<Vec<Frame>> {
    replay_require(
        field.is_cross_state() && !positions.is_empty(),
        "constructor requires cross field and positions",
    )?;
    let bytes = field.to_bytes()?;
    let hash = sha256_bytes(&bytes);
    let mut native = p.generator()?.with_continuation_field(BoundNativeBytes {
        bytes: &bytes,
        sha256: &hash,
    })?;
    let g = NativeGeometricGenerate::from_bytes(&p.generate, p.integer.binding())?;
    let bridge = NativeGeometricReadStateBridge::from_bytes(&p.bridge, p.integer.binding())?;
    let mut reducer = NativeVocabularyActions::new(p.integer.binding().clone(), &p.exp)?;
    let binding = epoch(p, field)?;
    let mut frames = Vec::with_capacity(positions.len());
    let mut seen = BTreeSet::new();
    // Reuse each admitted input bank; never retain full native step JSON.
    let mut previous = None;
    let mut bank = None;
    for pos in positions {
        deadline(a, start)?;
        replay_require(
            pos.position == pos.prefix.len()
                && pos.position < 32
                && pos.target < 4096
                && seen.insert((pos.input, pos.position)),
            "constructor position duplicate/invalid",
        )?;
        let e = episodes
            .get(pos.input)
            .ok_or_else(|| bad("constructor input index"))?;
        if previous != Some(pos.input) {
            bank = Some(native.admit_bank(continuation_snapshot(&e.packet)?)?);
            previous = Some(pos.input);
        }
        let step = native.step(
            bank.as_ref()
                .ok_or_else(|| bad("constructor admitted bank absent"))?,
            &pos.prefix,
        )?;
        let v = snapshot(&step)?;
        let old = shared::parse_saved_native_frame(
            pos.input,
            pos.position,
            e.packet.id.clone(),
            pos.prefix.clone(),
            pos.target,
            1.,
            &v,
            p,
            true,
        )?;
        let frame = Frame {
            position: pos.clone(),
            id: e.packet.id.clone(),
            prefix_trace: old
                .prefix_trace
                .ok_or_else(|| bad("constructor Prefix trace missing"))?,
            ids: old.ids,
            base_copy: old.base_copy,
            query: old.query,
            sources: old.sources,
            local: step
                .continuation
                .as_ref()
                .ok_or_else(|| bad("constructor local witness"))?
                .state_codes
                .clone(),
            epoch_sha256: binding.clone(),
            teacher: e.target.get(pos.position) == Some(&pos.target)
                && e.target.get(..pos.position) == Some(pos.prefix.as_slice()),
            target_count: e.target.len(),
        };
        let factual = pool(
            &frame,
            p,
            field,
            &g,
            &bridge,
            &mut reducer,
            earliest(&frame.base_copy)?,
        )?;
        replay_require(
            factual.post == step.post_state
                && factual.generate == step.generate_raw_scores_q24
                && factual.copy == step.copy_raw_scores_q24
                && factual.donor
                    == step
                        .bridge
                        .as_ref()
                        .ok_or_else(|| bad("constructor donor witness"))?
                        .selected_ordinal
                && serde_json::to_value(&factual.trace)? == serde_json::to_value(&step.actions)?,
            "constructor dynamic factual/native complete pool mismatch",
        )?;
        frames.push(frame);
    }
    Ok(frames)
}
fn rival(trace: &VocabularyActionTrace, target: u32) -> Result<u32> {
    let mut best: Option<(u32, u64)> = None;
    let mut found = false;
    for m in &trace.token_masses {
        replay_require(
            m.weight_q31 > 0 && m.weight_q31 <= trace.summary.total_weight_q31,
            "constructor native mass invalid",
        )?;
        if m.token_id == target {
            found = true;
            continue;
        }
        if best.is_none_or(|(id, mass)| {
            m.weight_q31 > mass || (m.weight_q31 == mass && m.token_id < id)
        }) {
            best = Some((m.token_id, m.weight_q31));
        }
    }
    replay_require(found, "constructor target absent from native pool")?;
    best.map(|x| x.0)
        .ok_or_else(|| bad("constructor rival absent"))
}
fn margin(trace: &VocabularyActionTrace, target: u32, wrong: u32) -> Result<f64> {
    let mass = |id| {
        trace
            .token_masses
            .iter()
            .find(|m| m.token_id == id)
            .map(|m| m.weight_q31)
            .filter(|&v| v > 0)
            .ok_or_else(|| bad("constructor contrast mass missing"))
    };
    Ok((mass(target)? as f64 / mass(wrong)? as f64).ln())
}
fn softplus(z: f64) -> f64 {
    z.max(0.) + (-z.abs()).exp().ln_1p()
}
fn rank(trace: &VocabularyActionTrace, target: u32) -> Result<f64> {
    // Match the existing native-margin F32 anchor and stable softplus F32 output.
    Ok(softplus(-(margin(trace, target, rival(trace, target)?)? as f32 as f64)) as f32 as f64)
}
fn zero_utility(raw: &Tensor, values: &[f32]) -> Result<Tensor> {
    replay_require(
        raw.dims() == [values.len()] && !values.is_empty() && values.iter().all(|x| x.is_finite()),
        "constructor donor utility shape/finite",
    )?;
    let weights = candle_nn::ops::softmax(raw, 0)?;
    let s = weights
        .mul(&Tensor::from_vec(values.to_vec(), values.len(), raw.device())?.detach())?
        .sum_all()?;
    Ok((&s - s.detach())?)
}
fn tensor_scores(v: &[i64], d: &Device) -> Result<Tensor> {
    Ok(Tensor::from_vec(
        v.iter()
            .map(|&x| (x as f64 / SCALE) as f32)
            .collect::<Vec<_>>(),
        v.len(),
        d,
    )?)
}
fn graph(
    f: &Frame,
    factual: &Pool,
    values: &[f32],
    pw: &PrefixAngularWeights,
    g: &GenerateLearningWeights,
    prepared: &PreparedGenerateLearning,
    guard: bool,
) -> Result<Tensor> {
    let d = g.device();
    let pc = pw.coefficient_credit(&f.prefix_trace, d)?;
    let raw = (tensor_scores(&f.base_copy, d)? + (&pc - pc.detach())?)?;
    let out = g.forward_prepared_coefficients_only(prepared, &factual.post)?;
    replay_require(
        checked_add(&out.scores_q24, &factual.u)? == factual.generate,
        "constructor Generate native graph mismatch",
    )?;
    let joined_g =
        (tensor_scores(&factual.generate, d)? + (&out.raw_scores - out.raw_scores.detach())?)?;
    let joined_c = (tensor_scores(&factual.copy, d)? + (&pc - pc.detach())?)?;
    let direct = if guard {
        vocabulary_log_mass_margin_with_credit(
            &factual.trace,
            &joined_g,
            Some(&joined_c),
            f.position.target,
            rival(&factual.trace, f.position.target)?,
            VocabularyScoreAdjoint::RawIdentity,
        )?
    } else {
        cross_state_completion::rank_loss_from_pool(
            &factual.trace,
            &joined_g,
            Some(&joined_c),
            f.position.target,
        )?
    };
    Ok((&direct + zero_utility(&raw, values)?)?)
}
fn masters(pw: &PrefixAngularWeights, g: &GenerateLearningWeights) -> Result<(Var, Var)> {
    let prefix = pw
        .parameters()
        .remove("prefix.coefficients")
        .ok_or_else(|| bad("constructor Prefix master absent"))?;
    replay_require(
        prefix.elem_count() == FAMILY && g.unary.elem_count() == FAMILY,
        "constructor expected two960 master families",
    )?;
    Ok((prefix, g.unary.clone()))
}
fn gradient(loss: &Tensor, params: &(Var, Var)) -> Result<(Vec<f64>, [bool; 2])> {
    let grads = loss.backward()?;
    let mut flat = Vec::with_capacity(WIDTH);
    let mut presence = [false; 2];
    for (i, p) in [&params.0, &params.1].into_iter().enumerate() {
        if let Some(v) = grads.get(p) {
            presence[i] = true;
            flat.extend(
                v.flatten_all()?
                    .to_vec1::<f32>()?
                    .into_iter()
                    .map(f64::from),
            );
        } else {
            flat.extend(vec![0.; FAMILY]);
        }
    }
    replay_require(
        flat.len() == WIDTH && flat.iter().all(|x| x.is_finite()),
        "constructor gradient nonfinite/shape",
    )?;
    Ok((flat, presence))
}
fn donors(
    f: &Frame,
    p: &ContinuationParent,
    field: &NativeContinuationField,
    g: &NativeGeometricGenerate,
    b: &NativeGeometricReadStateBridge,
    r: &mut NativeVocabularyActions,
    factual: &Pool,
    guard: bool,
) -> Result<Vec<f32>> {
    let wrong = rival(&factual.trace, f.position.target)?;
    let base = if guard {
        margin(&factual.trace, f.position.target, wrong)?
    } else {
        rank(&factual.trace, f.position.target)?
    };
    (0..f.ids.len())
        .map(|j| {
            let trial = pool(f, p, field, g, b, r, j)?;
            // Guard rival stays fixed within its Jacobian; objective rival is chosen afresh.
            let value = if guard {
                margin(&trial.trace, f.position.target, wrong)?
            } else {
                rank(&trial.trace, f.position.target)?
            };
            let delta = (value - base) as f32;
            replay_require(delta.is_finite(), "constructor donor contrast nonfinite")?;
            Ok(delta)
        })
        .collect()
}
fn check_frames(frames: &[Frame], binding: &str) -> Result<()> {
    replay_require(
        !frames.is_empty() && frames.iter().all(|f| f.epoch_sha256 == binding),
        "constructor frames belong to another parent epoch",
    )
}
fn episode_groups(frames: &[Frame]) -> Result<Vec<std::ops::Range<usize>>> {
    let mut groups = Vec::new();
    let mut start = 0;
    let mut seen = BTreeSet::new();
    while start < frames.len() {
        let input = frames[start].position.input;
        replay_require(
            seen.insert(input),
            "constructor objective input noncontiguous",
        )?;
        let mut end = start;
        while end < frames.len() && frames[end].position.input == input {
            replay_require(
                frames[end].teacher && frames[end].position.position == end - start,
                "constructor objective target position gap or nonteacher prefix",
            )?;
            end += 1;
        }
        replay_require(
            end - start <= 32 && end - start == frames[start].target_count,
            "constructor episode must include all targets and EOS",
        )?;
        groups.push(start..end);
        start = end;
    }
    replay_require(
        groups.len() == 512 && seen.iter().copied().eq(0..512),
        "constructor objective must cover each frozen input exactly once",
    )?;
    Ok(groups)
}
/// Stream scalar losses/1920 gradients. No full-bank computational graph survives a token.
pub(super) fn credit(
    a: &Args,
    start: Instant,
    p: &ContinuationParent,
    field: &NativeContinuationField,
    pw: &PrefixAngularWeights,
    g: &GenerateLearningWeights,
    objective: &[Frame],
    guards: &[Frame],
) -> Result<Credit> {
    let binding = epoch(p, field)?;
    check_frames(objective, &binding)?;
    check_frames(guards, &binding)?;
    replay_require(
        guards.len() == 380,
        "constructor requires registered380 guards",
    )?;
    replay_require(
        pw.packed_coefficients()? == p.prefix && g.export_native()?.to_bytes()? == p.generate,
        "constructor credit masters/native epoch mismatch",
    )?;
    let groups = episode_groups(objective)?;
    let ng = NativeGeometricGenerate::from_bytes(&p.generate, p.integer.binding())?;
    let b = NativeGeometricReadStateBridge::from_bytes(&p.bridge, p.integer.binding())?;
    let mut r = NativeVocabularyActions::new(p.integer.binding().clone(), &p.exp)?;
    let prepared = g.prepare_native()?;
    let params = masters(pw, g)?;
    let mut sum = vec![0.; WIDTH];
    let mut total = 0.;
    let mut presence = [0usize; 2];
    let mut donor_count = 0usize;
    for range in &groups {
        let mut bottleneck = cross_state_completion::EpisodeBottleneck::default();
        for f in &objective[range.clone()] {
            deadline(a, start)?;
            let factual = pool(f, p, field, &ng, &b, &mut r, earliest(&f.base_copy)?)?;
            let value = rank(&factual.trace, f.position.target)?;
            let utilities = donors(f, p, field, &ng, &b, &mut r, &factual, false)?;
            donor_count += utilities.len();
            let loss = graph(f, &factual, &utilities, pw, g, &prepared, false)?;
            replay_require(
                loss.to_scalar::<f32>()?.to_bits() == (value as f32).to_bits(),
                "constructor rank objective differs from existing hard-forward loss",
            )?;
            let grads = loss.backward()?;
            let mut active = BTreeMap::new();
            for (i, (name, var)) in [
                ("prefix.coefficients", &params.0),
                ("generate.unary", &params.1),
            ]
            .into_iter()
            .enumerate()
            {
                let gradient = if let Some(v) = grads.get(var) {
                    presence[i] += 1;
                    v.clone()
                } else {
                    var.zeros_like()?
                };
                active.insert(name.to_owned(), gradient);
            }
            bottleneck.push(value, active)?;
        }
        let (value, active) = bottleneck.finish(range.len(), groups.len())?;
        total += value;
        let flat = ["prefix.coefficients", "generate.unary"]
            .iter()
            .map(|name| {
                Ok(active
                    .get(*name)
                    .ok_or_else(|| bad("constructor episode gradient missing"))?
                    .flatten_all()?
                    .to_vec1::<f32>()?)
            })
            .collect::<Result<Vec<_>>>()?;
        for (s, n) in sum.iter_mut().zip(flat.into_iter().flatten()) {
            *s += f64::from(n);
        }
    }
    let mut margins = Vec::new();
    let mut jacobian = Vec::new();
    for f in guards {
        deadline(a, start)?;
        let factual = pool(f, p, field, &ng, &b, &mut r, earliest(&f.base_copy)?)?;
        let wrong = rival(&factual.trace, f.position.target)?;
        margins.push(margin(&factual.trace, f.position.target, wrong)?);
        let utilities = donors(f, p, field, &ng, &b, &mut r, &factual, true)?;
        donor_count += utilities.len();
        let quantity = graph(f, &factual, &utilities, pw, g, &prepared, true)?;
        jacobian.push(gradient(&quantity, &params)?.0);
    }
    replay_require(
        presence.iter().all(|&n| n > 0) && total.is_finite() && sum.iter().all(|x| x.is_finite()),
        "constructor aggregate nonfinite",
    )?;
    Ok(Credit {
        objective: total,
        gradient: sum,
        guard_margins: margins,
        jacobian,
        receipt: json!({"schema":"uor-r4.constructor-current-credit/1","epoch_sha256":binding,
            "episodes":groups.len(),"target_positions":objective.len(),"guard_positions":guards.len(),
            "active_order":["prefix.coefficients","generate.unary"],"width":WIDTH,
            "objective":"equal episode logmeanexp(native pooled strongest-wrong softplus(-logmassmargin));margin0",
            "donor_credit":"zero-forward softmax physical donor native fullpool quantity; dynamic factual/local U on all aliases",
            "state_adjoint":false,"parameter_gradient_presence":presence,"forced_donor_reductions":donor_count,
            "backward_calls":objective.len()+guards.len(),"parent_epoch_recaptured":true,
            "rival_tie_policy":"highest integer pooled mass; smallest token ID",
            "donor_tie_policy":"earliest physical occurrence among maximum raw Copy scores",
            "disconnected_token_family_policy":"explicit zero; aggregate each family must connect"}),
    })
}
/// Native teacher objective and selected guard outcomes; own-prefix full panels are root-owned.
pub(super) fn evaluate_frames(
    a: &Args,
    start: Instant,
    p: &ContinuationParent,
    field: &NativeContinuationField,
    objective: &[Frame],
    guards: &[Frame],
) -> Result<NativeEvaluation> {
    let binding = epoch(p, field)?;
    check_frames(objective, &binding)?;
    check_frames(guards, &binding)?;
    let groups = episode_groups(objective)?;
    let g = NativeGeometricGenerate::from_bytes(&p.generate, p.integer.binding())?;
    let b = NativeGeometricReadStateBridge::from_bytes(&p.bridge, p.integer.binding())?;
    let mut r = NativeVocabularyActions::new(p.integer.binding().clone(), &p.exp)?;
    let mut total = 0.;
    for range in &groups {
        let mut maximum = f64::NEG_INFINITY;
        let mut norm = 0.;
        for f in &objective[range.clone()] {
            deadline(a, start)?;
            let factual = pool(f, p, field, &g, &b, &mut r, earliest(&f.base_copy)?)?;
            let value = rank(&factual.trace, f.position.target)?;
            let next = maximum.max(value);
            norm = norm * (maximum - next).exp() + (value - next).exp();
            maximum = next;
        }
        total += (maximum + norm.ln() - (range.len() as f64).ln()) / groups.len() as f64;
    }
    let mut margins = Vec::new();
    let mut winners = Vec::new();
    let mut preserved = true;
    for f in guards {
        deadline(a, start)?;
        let factual = pool(f, p, field, &g, &b, &mut r, earliest(&f.base_copy)?)?;
        margins.push(margin(
            &factual.trace,
            f.position.target,
            rival(&factual.trace, f.position.target)?,
        )?);
        winners.push(factual.trace.summary.chosen_token_id);
        preserved &= factual.trace.summary.chosen_token_id == f.position.target;
    }
    Ok(NativeEvaluation {
        objective: total,
        guard_margins: margins,
        guard_winners: winners,
        guard_preserved: preserved,
        target_positions: objective.len(),
    })
}
/// Build an in-memory candidate. This is NOT independent disk export/reload.
pub(super) fn candidate_parent(
    p: &ContinuationParent,
    field: &NativeContinuationField,
    l: &Loaded,
    pw: &PrefixAngularWeights,
    g: &GenerateLearningWeights,
) -> Result<(ContinuationParent, NativeContinuationField)> {
    replay_require(field.is_cross_state(), "constructor cross field required")?;
    let native = g.export_native()?;
    let old = NativeGeometricGenerate::from_bytes(&p.generate, p.integer.binding())?;
    replay_require(
        native.prototypes() == old.prototypes()
            && native.packed_biases() == old.packed_biases()
            && serde_json::to_value(native.energy())?["pair_packed"]
                == serde_json::to_value(old.energy())?["pair_packed"]
            && native.energy().edges() == old.energy().edges(),
        "constructor prototypes changed",
    )?;
    let carrier = l.frozen.compile_cue_carrier(l.cue.clone())?;
    replay_require(
        carrier.packed_coefficients() == p.cue && carrier.metadata() == &p.cue_metadata,
        "constructor frozen Cue mismatch",
    )?;
    let prefix = l.frozen.compile_prefix_transport(&carrier, pw.native()?)?;
    let generate = native.to_bytes()?;
    let rebound = NativeContinuationField::compile_cross_state(
        &p.binding,
        &native,
        field
            .packed_cross_state()
            .ok_or_else(|| bad("constructor cross coefficients missing"))?,
    )?;
    let cp = ContinuationParent {
        binding: p.binding.clone(),
        integer: IntegerRealizer::load_native(&p.native_directory, &p.binding)?,
        tokenizer: ByteBpeTokenizer::from_tokenizer_json_bytes(&fs::read(
            p.native_directory.join("tokenizer.json"),
        )?)
        .ok_or_else(|| bad("constructor tokenizer reload"))?,
        native_directory: p.native_directory.clone(),
        generate_sha256: sha256_bytes(&generate),
        generate,
        bridge: p.bridge.clone(),
        bridge_sha256: p.bridge_sha256.clone(),
        cue: p.cue.clone(),
        joint: p.joint.clone(),
        cue_metadata: p.cue_metadata.clone(),
        prefix: prefix.packed_coefficients().to_vec(),
        prefix_metadata: prefix.metadata().clone(),
        exp: p.exp.clone(),
        exp_sha256: p.exp_sha256.clone(),
        receipt: p.receipt.clone(),
    };
    Ok((cp, rebound))
}

#[cfg(test)]
mod tests {
    use super::*;
    use uor_r4_core::native_geometric::learner::integrated_attention::geometry::EnergyTables;
    use uor_r4_integer::{
        geometric_source_actions::SourceActionBinding, geometric_source_realizer::ArtifactIdentity,
    };
    const TOK:&[u8]=br#"{"pre_tokenizer":{"type":"ByteLevel","add_prefix_space":false},"model":{"type":"BPE","vocab":{"<|bos|>":0,"<|eos|>":1,"<|unk|>":2,".":3,"a":4,"b":5},"merges":[]},"added_tokens":[{"id":0,"content":"<|bos|>"},{"id":1,"content":"<|eos|>"},{"id":2,"content":"<|unk|>"}]}"#;
    fn fixture() -> Result<(
        NativeArtifactBinding,
        NativeGeometricGenerate,
        NativeVocabularyActions,
    )> {
        let source = SourceActionBinding::new(TOK)?;
        let g = NativeGeometricGenerate::compile(
            &source,
            2,
            &[1, 1, 0, 1, 3, 5, 5, 3, 31, 57, 31, 57],
            &[0; 3],
            EnergyTables::zeroed(2, vec![])?,
        )?;
        let binding = NativeArtifactBinding {
            metadata_sha256: "a".repeat(64),
            identity: ArtifactIdentity {
                tokenizer_sha256: g.metadata().tokenizer_sha256.clone(),
                parent_checkpoint_manifest_sha256: "b".repeat(64),
                parent_model_sha256: "c".repeat(64),
                parent_config_sha256: "d".repeat(64),
            },
        };
        let exp = (0..uor_r4_integer::geometric_read::EXP_TABLE_LEN)
            .flat_map(|i| {
                (((-(i as f64) / 256.).exp() * (1u64 << 31) as f64).round() as u32).to_le_bytes()
            })
            .collect::<Vec<_>>();
        Ok((binding, g, NativeVocabularyActions::new(source, &exp)?))
    }
    #[test]
    fn constructor_dynamic_joint_field_rescores_all_copy_aliases_and_poststate() -> Result<()> {
        let (binding, g, mut reducer) = fixture()?;
        let local = vec![H4Code::IDENTITY; 2];
        let factual = local.clone();
        let changed = vec![H4Code::try_from(2)?, H4Code::IDENTITY];
        let proto = g.prototypes()[4 * 2];
        let old = g
            .algebra()
            .compose(g.algebra().inverse(factual[0].index())?, proto)?;
        let new = g
            .algebra()
            .compose(g.algebra().inverse(changed[0].index())?, proto)?;
        let loc = g
            .algebra()
            .compose(g.algebra().inverse(local[0].index())?, proto)?;
        assert_ne!(old, new);
        let mut packed = vec![0u8; 2 << 13];
        let index = ((new as usize) << 6) | ((loc as usize) >> 1);
        packed[index] |= 7 << ((loc & 1) << 2);
        let field = NativeContinuationField::compile_cross_state(&binding, &g, &packed)?;
        let ids = [4, 4, 5];
        let base = [1i64, 2, 3];
        let (oldg, oldc, oldu) = joint_scores(&factual, &local, &g, &field, &ids, &base)?;
        let (newg, newc, newu) = joint_scores(&changed, &local, &g, &field, &ids, &base)?;
        assert_eq!(oldu[4], 0);
        assert_eq!(newu[4], 7 << 22);
        assert_eq!(newc[0] - oldc[0], 7 << 22);
        assert_eq!(newc[1] - oldc[1], 7 << 22);
        assert_eq!(newg[4] - oldg[4], 7 << 22);
        let actual = reducer.reduce_trace(&newg, &ids, &newc)?;
        let stale = reducer.reduce_trace(&newg, &ids, &oldc)?;
        assert_ne!(serde_json::to_value(actual)?, serde_json::to_value(stale)?);
        let zero = NativeContinuationField::compile_cross_state(&binding, &g, &vec![0; 2 << 13])?;
        let (z, copy, u) = joint_scores(&factual, &local, &g, &zero, &ids, &base)?;
        assert_eq!(copy, base);
        assert!(u.iter().all(|&x| x == 0));
        let mut raw = vec![0; g.vocab_size()];
        g.score_into(&factual, &mut raw, &mut Default::default())?;
        assert_eq!(z, raw);
        assert!(joint_scores(&factual, &local, &g, &zero, &[999], &[0]).is_err());
        Ok(())
    }
    #[test]
    fn constructor_rank_native_anchor_matches_existing_graph_alias_tie_and_refresh() -> Result<()> {
        let (_, g, mut reducer) = fixture()?;
        let raw = vec![0i64; g.vocab_size()];
        let ids = [4, 4, 5];
        let copy = [0i64; 3];
        let trace = reducer.reduce_trace(&raw, &ids, &copy)?;
        assert_eq!(rival(&trace, 1)?, 4); // EOS target, pooled duplicate wins.
        assert_eq!(rival(&trace, 4)?, 5);
        let joinedg = tensor_scores(&raw, &Device::Cpu)?;
        let joinedc = tensor_scores(&copy, &Device::Cpu)?;
        let graph =
            cross_state_completion::rank_loss_from_pool(&trace, &joinedg, Some(&joinedc), 1)?;
        assert_eq!(
            (rank(&trace, 1)? as f32).to_bits(),
            graph.to_scalar::<f32>()?.to_bits()
        );
        let tie = reducer.reduce_trace(&raw, &[], &[])?;
        assert_eq!(rival(&tie, 1)?, 0);
        let mut raised = raw.clone();
        raised[3] = 4 << 24;
        let next = reducer.reduce_trace(&raised, &ids, &copy)?;
        assert_eq!(rival(&next, 1)?, 3);
        assert!(rival(&trace, 999).is_err());
        Ok(())
    }
    #[test]
    fn constructor_donor_utility_zero_forward_and_guard_objective_opposite_credit() -> Result<()> {
        let raw = Var::from_vec(vec![0f32; 3], 3, &Device::Cpu)?;
        let objective = zero_utility(raw.as_tensor(), &[0., 0., -0.4])?;
        let guard = zero_utility(raw.as_tensor(), &[0., 0., 0.4])?;
        assert_eq!(objective.to_scalar::<f32>()?, 0.);
        assert_eq!(guard.to_scalar::<f32>()?, 0.);
        let get = |t: &Tensor| -> Result<Vec<f32>> {
            Ok(t.backward()?
                .get(raw.as_tensor())
                .ok_or_else(|| bad("constructor fixture gradient absent"))?
                .to_vec1::<f32>()?)
        };
        let a = get(&objective)?;
        let b = get(&guard)?;
        assert!(a[0] > 0. && a[2] < 0.);
        assert_eq!(a[0], a[1]);
        for (x, y) in a.iter().zip(b) {
            assert!((x + y).abs() < 1e-7);
        }
        assert!(zero_utility(raw.as_tensor(), &[f32::NAN, 0., 0.]).is_err());
        assert!(zero_utility(raw.as_tensor(), &[0., 0.]).is_err());
        Ok(())
    }
    #[test]
    fn constructor_existing_bottleneck_composes_direct_and_donor_credit_once() -> Result<()> {
        let raw = Var::from_vec(vec![0f32; 3], 3, &Device::Cpu)?;
        let mut accumulator = cross_state_completion::EpisodeBottleneck::default();
        for loss in [1f32, 2., 3.] {
            let direct = raw.sum_all()?.affine(loss as f64, loss as f64)?;
            let utility = zero_utility(raw.as_tensor(), &[0., 0., -0.4])?;
            let graph = (&direct + utility)?;
            let grad = graph
                .backward()?
                .get(raw.as_tensor())
                .ok_or_else(|| bad("constructor composition gradient"))?
                .clone();
            accumulator.push(loss as f64, BTreeMap::from([("x".into(), grad)]))?;
        }
        let (j, grads) = accumulator.finish(3, 2)?;
        let denom = (1f64).exp() + 2f64.exp() + 3f64.exp();
        assert!((j - (denom / 3.).ln() / 2.).abs() < 1e-12);
        let mean = (1f64.exp() + 2. * 2f64.exp() + 3. * 3f64.exp()) / denom;
        let actual = grads["x"].to_vec1::<f32>()?;
        assert!((actual[0] as f64 - (mean + 0.4 / 9.) / 2.).abs() < 1e-6);
        assert!((actual[2] as f64 - (mean - 0.8 / 9.) / 2.).abs() < 1e-6);
        Ok(())
    }
    #[test]
    fn constructor_rebinding_preserves_cross_payload_and_rejects_stale_generate() -> Result<()> {
        let (binding, g, _) = fixture()?;
        let field = NativeContinuationField::compile_cross_state(&binding, &g, &vec![0; 2 << 13])?;
        let source = SourceActionBinding::new(TOK)?;
        let mut energy = g.energy().clone();
        energy.set_unary(0, 3, 1)?;
        let changed = NativeGeometricGenerate::compile(
            &source,
            2,
            g.prototypes(),
            g.packed_biases(),
            energy,
        )?;
        assert!(
            NativeContinuationField::from_bytes(&field.to_bytes()?, &binding, &changed).is_err()
        );
        let rebound = NativeContinuationField::compile_cross_state(
            &binding,
            &changed,
            field
                .packed_cross_state()
                .ok_or_else(|| bad("fixture packed field"))?,
        )?;
        assert_eq!(rebound.packed_cross_state(), field.packed_cross_state());
        assert_ne!(rebound.to_bytes()?, field.to_bytes()?);
        assert_eq!(
            NativeContinuationField::from_bytes(&rebound.to_bytes()?, &binding, &changed)?
                .to_bytes()?,
            rebound.to_bytes()?
        );
        Ok(())
    }
}
