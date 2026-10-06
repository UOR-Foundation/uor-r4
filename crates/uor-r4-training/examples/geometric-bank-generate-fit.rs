//! Real-panel CUDA Copy+Generate learning. No supplied selected record; labels
//! enter only after target-free native scores. Admission never starts a fit.
#![recursion_limit = "256"]

use candle_core::{Device, Tensor, Var};
use candle_nn::{AdamW, Optimizer, ParamsAdamW};
use serde::Deserialize;
use serde_json::{json, Value};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs, io,
    path::{Path, PathBuf},
    time::Instant,
};
use uor_r4_core::{
    answer_oracle::FrozenAnswers,
    native_geometric::learner::{
        geometric_generate::{GenerateReadCounts, NativeGeometricGenerate},
        geometric_read_state_bridge::{BridgeReadCounts, NativeGeometricReadStateBridge},
    },
    report_output,
};
use uor_r4_integer::{
    geometric_context::NativeContextState,
    geometric_cue_carrier::{CueAngularConfig, CueAngularQ4, CueJointMetadata, CueJointQ4},
    geometric_occurrence_read::{FrameMetadata, FrameStatus, SelectedRecordFrame, SourceIdentity},
    geometric_prefix_transport::{PrefixAngularConfig, PrefixAngularQ4},
    geometric_source_emission_view::SourceEmissionView,
    geometric_source_realizer::{
        NativeArtifactBinding, NativeSourceRealizer as IntegerRealizer, SourceBankSegment,
    },
    geometric_vocabulary_actions::NativeVocabularyActions,
};
use uor_r4_tokenizer::ByteBpeTokenizer;
use uor_r4_training::{
    geometric_bank_generate::PreparedBankGenerate,
    geometric_generate_learning::GenerateLearningWeights,
    geometric_occurrence_consumer::{
        source_realizer::{NativeSourceRealizer, SourceRealizerWeights},
        ConsumerIdentity,
    },
    geometric_read_state_bridge::BridgeLearningWeights,
    sha256_bytes, sha256_file,
};
#[path = "../../uor-r4-integer/examples/support/source_probe.rs"]
mod output_support;
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Args {
    mode: String,
    arm: String,
    seed: u64,
    source_weights: PathBuf,
    native_artifact: PathBuf,
    trusted_binding: PathBuf,
    cue_bundle: PathBuf,
    prefix_bundle: PathBuf,
    canonical_exp: PathBuf,
    training_inputs: PathBuf,
    training_labels: PathBuf,
    development_inputs: PathBuf,
    development_labels: PathBuf,
    fresh_inputs: Option<PathBuf>,
    fresh_labels: Option<PathBuf>,
    // A source-input-selected cost probe only; never changes the fit schedule.
    admission_episode_indices: Option<Vec<usize>>,
    #[serde(default = "default_backward_chunk")]
    token_backward_chunk: usize,
    #[serde(default)]
    prefix_temporal_utility: bool,
    #[serde(default)]
    balanced_token_geometry: bool,
    #[serde(default)]
    phase_balanced_token_loss: bool,
    #[serde(default)]
    read_state_bridge: bool,
    #[serde(default = "default_bridge_learning_rate")]
    read_state_bridge_learning_rate: f64,
    updates: usize,
    learning_rate: f64,
    prototype_learning_rate: f64,
    context_learning_rate: f64,
    #[serde(default)]
    potential_learning_rate: Option<f64>,
    maximum_seconds: u64,
    maximum_report_bytes: u64,
    out: PathBuf,
}
fn default_bridge_learning_rate() -> f64 {
    0.002
}
fn default_backward_chunk() -> usize {
    1
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Inputs {
    schema: String,
    cases: Vec<Packet>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Packet {
    id: String,
    segments: Vec<Segment>,
    query_ids: Vec<u32>,
    actual_prefix_ids: Vec<u32>,
}
#[derive(Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
enum Segment {
    Source {
        event: u64,
        record: u64,
        commit: u64,
        scope: String,
        entity: Vec<u32>,
        relation: u32,
        view: u32,
        original_source_ids: Vec<u32>,
    },
    Context {
        event: u64,
        role: u32,
        token_ids: Vec<u32>,
    },
}
impl Segment {
    fn frame(&self) -> Option<SelectedRecordFrame<'_>> {
        match self {
            Self::Source {
                record,
                commit,
                scope,
                entity,
                relation,
                view,
                original_source_ids,
                ..
            } => Some(SelectedRecordFrame {
                identity: SourceIdentity {
                    record: *record,
                    commit: *commit,
                },
                metadata: FrameMetadata {
                    scope: scope.as_bytes(),
                    entity,
                    relation: *relation,
                    view: *view,
                    status: FrameStatus::Found,
                },
                token_ids: original_source_ids,
            }),
            _ => None,
        }
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Labels {
    schema: String,
    protocol: String,
    membership_only: bool,
    cases: Vec<Label>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Label {
    id: String,
    answers: FrozenAnswers,
}
struct Episode {
    packet: Packet,
    answers: FrozenAnswers,
    target: Vec<u32>,
    views: Vec<Option<SourceEmissionView>>,
}
impl Episode {
    fn segments(&self) -> Result<Vec<SourceBankSegment<'_>>> {
        self.packet
            .segments
            .iter()
            .enumerate()
            .map(|(i, s)| {
                Ok(match s {
                    Segment::Source { event, .. } => SourceBankSegment::Source {
                        frame: s.frame().ok_or_else(|| bad("source frame missing"))?,
                        view: self.views[i]
                            .as_ref()
                            .ok_or_else(|| bad("source view missing"))?,
                        event: *event,
                    },
                    Segment::Context {
                        event,
                        role,
                        token_ids,
                    } => SourceBankSegment::Context {
                        event: *event,
                        role: *role,
                        token_ids,
                    },
                })
            })
            .collect()
    }
    fn base_len(&self) -> usize {
        self.packet.query_ids.len()
            + self
                .packet
                .segments
                .iter()
                .zip(&self.views)
                .map(|(s, v)| match (s, v) {
                    (Segment::Source { .. }, Some(v)) => v.emitted_token_ids().len(),
                    (Segment::Context { token_ids, .. }, _) => token_ids.len(),
                    _ => 0,
                })
                .sum::<usize>()
    }
    fn has_source(&self) -> bool {
        self.packet
            .segments
            .iter()
            .any(|s| matches!(s, Segment::Source { .. }))
    }
    fn causal_no_source(&self, prefix: &[u32]) -> Result<Vec<u32>> {
        let mut ids = Vec::new();
        for s in &self.packet.segments {
            match s {
                Segment::Context { token_ids, .. } => ids.extend_from_slice(token_ids),
                _ => return Err(bad("no-source branch contains Source")),
            }
        }
        ids.extend_from_slice(&self.packet.query_ids);
        ids.extend_from_slice(prefix);
        if ids.is_empty() || ids.len() > 128 {
            return Err(bad("no-source causal context outside1..128"));
        }
        Ok(ids)
    }
}
fn bad(s: &str) -> Box<dyn std::error::Error> {
    io::Error::new(io::ErrorKind::InvalidData, s).into()
}
fn read(path: &Path) -> Result<Value> {
    Ok(serde_json::from_slice(&fs::read(path)?)?)
}
fn size(p: &Path) -> Result<u64> {
    let mut total = 0;
    for e in fs::read_dir(p)? {
        let p = e?.path();
        total += if p.is_dir() {
            size(&p)?
        } else {
            fs::metadata(p)?.len()
        };
    }
    Ok(total)
}
fn write(a: &Args, name: &str, v: &Value) -> Result<()> {
    let bytes = serde_json::to_vec(v)?;
    let p = a.out.join(name);
    let previous = fs::metadata(&p).map_or(0, |m| m.len());
    if size(&a.out)? - previous + bytes.len() as u64 > a.maximum_report_bytes - 1_048_576 {
        return Err(bad("report cap"));
    }
    fs::write(p, bytes)?;
    Ok(())
}
fn deadline(a: &Args, start: Instant) -> Result<()> {
    let elapsed = start.elapsed().as_secs_f64();
    if elapsed >= a.maximum_seconds as f64 && !a.out.join("wall-estimate-overrun.json").exists() {
        // Owner rule (6 October): a self-set estimate must not terminate a
        // progressing run. Resource/lease extensions still need ledger visibility
        // from the controlling lab; this notice does not renew a GPU lease.
        write(
            a,
            "wall-estimate-overrun.json",
            &json!({
                "schema":"uor-r4.wall-estimate-overrun/1",
                "declared_estimate_seconds":a.maximum_seconds,
                "first_observed_elapsed_seconds":elapsed,
                "action":"continue; self-set estimate is not a cancellation rule",
                "operator_obligation":"record extension on owning issue and renew active lease; owner spending caps, disk floor and failed/diverged work remain boundaries",
                "preflight_projection_admission":"unchanged"
            }),
        )?;
        eprintln!("Wall estimate exceeded after {elapsed:.3}s; continuing under owner rule. Record extension and renew the active lease.");
    }
    Ok(())
}
fn seal_for(path: &Path) -> Result<PathBuf> {
    for p in path.ancestors() {
        if p.join("manifest.json").is_file() {
            return Ok(p.to_path_buf());
        }
    }
    Err(bad("input has no enclosing seal"))
}
fn args() -> Result<Args> {
    let mut it = std::env::args().skip(1);
    let p = it
        .next()
        .ok_or_else(|| bad("one JSON config path required"))?;
    if it.next().is_some() {
        return Err(bad("one config path only"));
    }
    let a: Args = serde_json::from_slice(&fs::read(p)?)?;
    if a.fresh_inputs.is_some() != a.fresh_labels.is_some() {
        return Err(bad(
            "prospective evaluation input/labels must both be supplied or both absent",
        ));
    }
    if !["admission", "fit"].contains(&a.mode.as_str())
        || !["output-only", "joint", "joint-potential"].contains(&a.arm.as_str())
        || (a.arm == "joint-potential" && a.potential_learning_rate.is_none())
        || a.potential_learning_rate
            .is_some_and(|lr| !lr.is_finite() || lr <= 0. || lr > 0.01)
        || (a.arm != "joint-potential" && a.potential_learning_rate.is_some())
        || !a.read_state_bridge_learning_rate.is_finite()
        || a.read_state_bridge_learning_rate <= 0.
        || a.read_state_bridge_learning_rate > 0.01
        || ![1, 2].contains(&a.token_backward_chunk)
        || a.updates == 0
        || a.updates > 128
        || (a.mode == "fit" && a.updates != 128)
        || !a.learning_rate.is_finite()
        || a.learning_rate <= 0.
        || a.learning_rate > 0.01
        || !a.context_learning_rate.is_finite()
        || a.context_learning_rate <= 0.
        || a.context_learning_rate > 0.01
        || !a.prototype_learning_rate.is_finite()
        || a.prototype_learning_rate <= 0.
        || a.prototype_learning_rate > 0.02
        || a.maximum_seconds == 0
        || a.maximum_seconds > 8100
        || a.maximum_report_bytes < 64 << 20
        || a.maximum_report_bytes > 512 << 20
    {
        return Err(bad("prospective mode/arm/resource bounds"));
    }
    let output = output_support::prospective_output(&a.out)?;
    for p in input_paths(&a) {
        let p = fs::canonicalize(p)?;
        if output.starts_with(&p) || p.starts_with(&output) {
            return Err(bad("output/input overlap"));
        }
        for ancestor in p.ancestors() {
            if ancestor.join("manifest.json").is_file() && output.starts_with(ancestor) {
                return Err(bad("output inside input seal"));
            }
        }
    }
    Ok(a)
}
fn input_paths(a: &Args) -> Vec<&Path> {
    let mut p = vec![
        a.source_weights.as_path(),
        a.native_artifact.as_path(),
        a.trusted_binding.as_path(),
        a.cue_bundle.as_path(),
        a.prefix_bundle.as_path(),
        a.canonical_exp.as_path(),
        a.training_inputs.as_path(),
        a.training_labels.as_path(),
        a.development_inputs.as_path(),
        a.development_labels.as_path(),
    ];
    if let Some(x) = &a.fresh_inputs {
        p.push(x);
    }
    if let Some(x) = &a.fresh_labels {
        p.push(x);
    }
    p
}
#[cfg(feature = "cuda")]
fn cuda() -> Result<Device> {
    Ok(Device::new_cuda(0)?)
}
#[cfg(not(feature = "cuda"))]
fn cuda() -> Result<Device> {
    Err(bad("CUDA feature/device required; no fallback"))
}
fn cue_payload(root: &Path) -> Result<CueAngularQ4> {
    let meta = read(&root.join("native-metadata.json"))?;
    let config: CueAngularConfig = serde_json::from_value(meta["potential"].clone())?;
    let packed = fs::read(root.join("cue-q4.bin"))?;
    let mut q = CueAngularQ4::new(config, &packed)?;
    if let Some(j) = meta.get("joint") {
        let m: CueJointMetadata = serde_json::from_value(j.clone())?;
        let joint = CueJointQ4::new(m.config, fs::read(root.join("cue-joint-q4.bin"))?)?;
        if joint.metadata() != m {
            return Err(bad("cue joint payload receipt"));
        }
        q = q.with_joint(joint)?;
    } else if root.join("cue-joint-q4.bin").exists() {
        return Err(bad("unreceipted cue joint"));
    }
    Ok(q)
}
fn prefix_payload(root: &Path) -> Result<PrefixAngularQ4> {
    let m = read(&root.join("native-metadata.json"))?;
    let c: PrefixAngularConfig = serde_json::from_value(m["potential"].clone())?;
    Ok(PrefixAngularQ4::new(
        c,
        &fs::read(root.join("prefix-q4.bin"))?,
    )?)
}
fn load_panel(
    inputs: &Path,
    labels: &Path,
    integer: &IntegerRealizer,
    tok: &ByteBpeTokenizer,
    legal: &BTreeSet<u32>,
    max: usize,
) -> Result<Vec<Episode>> {
    let i: Inputs = serde_json::from_slice(&fs::read(inputs)?)?;
    let l: Labels = serde_json::from_slice(&fs::read(labels)?)?;
    if i.schema != "uor-r4.native-source-bank-probe-input/1"
        || l.schema != "uor-r4.native-source-bank-labels/1"
        || l.protocol != "uor-r4.literal-role-dialogue/2"
        || !l.membership_only
        || i.cases.is_empty()
        || i.cases.len() > max
        || i.cases.len() != l.cases.len()
    {
        return Err(bad("complete panel/schema/count contract"));
    }
    let mut seen = BTreeSet::new();
    let mut out = Vec::new();
    for (p, l) in i.cases.into_iter().zip(l.cases) {
        l.answers.validate()?;
        if p.id.is_empty()
            || p.id != l.id
            || !seen.insert(p.id.clone())
            || !p.actual_prefix_ids.is_empty()
            || p.query_ids.is_empty()
            || p.segments.len() > 128
            || l.answers.accepted.is_empty()
        {
            return Err(bad("case identity or complete-answer prefix contract"));
        }
        let mut target = tok.encode(&l.answers.accepted[0]);
        if tok.decode_bytes(&target) != l.answers.accepted[0].as_bytes() {
            return Err(bad("canonical answer roundtrip"));
        }
        target.push(integer.binding().eos_token_id());
        if target.len() > 32 {
            return Err(bad("answer exceeds32 token budget"));
        }
        let views = p
            .segments
            .iter()
            .map(|s| {
                s.frame()
                    .map(|f| integer.compile_view(f.token_ids))
                    .transpose()
            })
            .collect::<std::result::Result<Vec<_>, _>>()?;
        let e = Episode {
            packet: p,
            answers: l.answers,
            target,
            views,
        };
        let mut base = e.packet.query_ids.len();
        for (s, v) in e.packet.segments.iter().zip(&e.views) {
            base += match (s, v) {
                (Segment::Source { .. }, Some(v)) => v.emitted_token_ids().len(),
                (Segment::Context { token_ids, .. }, _) => token_ids.len(),
                _ => return Err(bad("view/source mismatch")),
            };
        }
        if base + e.target.len() > 128 {
            return Err(bad("complete teacher-prefix context exceeds128"));
        } // Eligibility is public token membership, never model-quality filtering.
        let mut ids = e.packet.query_ids.clone();
        ids.extend_from_slice(&e.target);
        for (s, v) in e.packet.segments.iter().zip(&e.views) {
            match s {
                Segment::Source {
                    original_source_ids,
                    ..
                } => {
                    ids.extend_from_slice(original_source_ids);
                    ids.extend_from_slice(
                        v.as_ref().ok_or_else(|| bad("view"))?.emitted_token_ids(),
                    );
                }
                Segment::Context { token_ids, .. } => ids.extend_from_slice(token_ids),
            }
        }
        if ids.iter().any(|id| !legal.contains(id)) {
            return Err(bad("panel token outside actual public action binding"));
        }
        out.push(e);
    }
    Ok(out)
}

fn prefix_clone(q: &PrefixAngularQ4) -> Result<PrefixAngularQ4> {
    Ok(PrefixAngularQ4::new(q.config(), q.packed_coefficients())?)
}
fn parameters(
    source: &SourceRealizerWeights,
    g: &GenerateLearningWeights,
    bridge: Option<&BridgeLearningWeights>,
    arm: &str,
) -> BTreeMap<String, Var> {
    let mut p = g.parameters();
    if let Some(bridge) = bridge {
        p.extend(bridge.parameters());
    }
    if matches!(arm, "joint" | "joint-potential") {
        p.extend(source.context_state_parameters());
    }
    if arm == "joint-potential" {
        p.extend(source.potential_parameters());
    }
    p
}
fn compile_learning_source(
    source: &SourceRealizerWeights,
    frozen: &NativeSourceRealizer,
    arm: &str,
) -> Result<NativeSourceRealizer> {
    Ok(if arm == "joint-potential" {
        source.compile_context_potential_rebound(frozen)?
    } else {
        source.compile_context_state_rebound(frozen)?
    })
}
// Native serving selects before consulting any target label; strict comparison
// preserves the first physical occurrence when raw Copy scores tie.
fn earliest_raw_copy_max(scores: &[i64]) -> Result<usize> {
    let mut selected = 0;
    let mut best = *scores
        .first()
        .ok_or_else(|| bad("bridge has no Copy candidates"))?;
    for (i, &score) in scores.iter().enumerate().skip(1) {
        if score > best {
            selected = i;
            best = score;
        }
    }
    Ok(selected)
}
fn native_step(
    model: &IntegerRealizer,
    g: &NativeGeometricGenerate,
    bridge: Option<&NativeGeometricReadStateBridge>,
    pool: &mut NativeVocabularyActions,
    e: &Episode,
    actual: &[u32],
    cueq: &CueAngularQ4,
    prefixq: &PrefixAngularQ4,
) -> Result<Value> {
    use uor_r4_integer::h4_tables::H4Code;
    let (codes, copy_ids, copy_scores, provenance) = if e.has_source() {
        let cue = model.compile_cue_carrier(cueq.clone())?;
        let prefix = model.compile_prefix_transport(&cue, prefix_clone(prefixq)?)?;
        let trace = model.read_bank_with_prefix_transport(
            &e.segments()?,
            &e.packet.query_ids,
            actual,
            &cue,
            &prefix,
        )?;
        let b = &trace.cue_bank.bank;
        let mut codes = b
            .context
            .states
            .last()
            .ok_or_else(|| bad("bank final state absent"))?
            .iter()
            .copied()
            .map(H4Code::try_from)
            .collect::<std::result::Result<Vec<_>, _>>()?;
        let ids = b
            .candidates
            .iter()
            .map(|c| c.occurrence.token_id)
            .collect::<Vec<_>>();
        let mut scores = vec![0i64; ids.len()];
        for h in &b.heads {
            for (i, &s) in h.scores_q24.iter().enumerate() {
                scores[i] = scores[i]
                    .checked_add(s)
                    .ok_or_else(|| bad("Copy score overflow"))?;
            }
        }
        let bridge_trace = if let Some(bridge) = bridge {
            let selected = earliest_raw_copy_max(&scores)?;
            let candidate = &b.candidates[selected];
            let source = b
                .context
                .states
                .get(candidate.context_position)
                .ok_or_else(|| bad("bridge selected source state missing"))?
                .iter()
                .copied()
                .map(H4Code::try_from)
                .collect::<std::result::Result<Vec<_>, _>>()?;
            let mut distinct_source_states = BTreeSet::new();
            for candidate in &b.candidates {
                let state = b
                    .context
                    .states
                    .get(candidate.context_position)
                    .ok_or_else(|| bad("alternative source state missing"))?;
                distinct_source_states.insert(state.clone());
            }
            let query = codes.clone();
            let mut actions = vec![H4Code::IDENTITY; codes.len()];
            let mut action_scores = vec![0i64; codes.len() * 120];
            let mut counts = BridgeReadCounts::default();
            bridge.apply_into(
                &query,
                &source,
                &mut codes,
                &mut actions,
                &mut action_scores,
                &mut counts,
            )?;
            json!({"distinct_candidate_source_states":distinct_source_states.len(),"selected_ordinal":selected,"selected_candidate":candidate,"query_state_codes":query.iter().map(|c|c.index()).collect::<Vec<_>>(),"selected_source_state_codes":source.iter().map(|c|c.index()).collect::<Vec<_>>(),"action_codes":actions.iter().map(|c|c.index()).collect::<Vec<_>>(),"action_scores_q24":action_scores,"native_costs":counts,"payload_sha256":bridge.metadata().payload_sha256})
        } else {
            Value::Null
        };
        let mut components = BTreeMap::new();
        for (name, source) in [
            ("cue", &trace.cue_bank.carrier.copy_q24),
            ("prefix", &trace.prefix.copy_q24),
        ] {
            let mut totals = vec![0i64; ids.len()];
            if source.len() != b.heads.len() {
                return Err(bad("Copy component head count differs"));
            }
            for head in source {
                if head.len() != ids.len() {
                    return Err(bad("Copy component candidate count differs"));
                }
                for (j, &score) in head.iter().enumerate() {
                    totals[j] = totals[j]
                        .checked_add(score)
                        .ok_or_else(|| bad("Copy component overflow"))?;
                }
            }
            components.insert(name, totals);
        }
        let contextual = scores
            .iter()
            .enumerate()
            .map(|(j, &score)| {
                score
                    .checked_sub(components["cue"][j])
                    .and_then(|s| s.checked_sub(components["prefix"][j]))
                    .ok_or_else(|| bad("Copy contextual attribution overflow"))
            })
            .collect::<Result<Vec<_>>>()?;
        components.insert("contextual", contextual);
        let mut provenance = json!({"copy_components_q24":components,"candidates":b.candidates,"causal_tokens":b.context.tokens,"legacy_terminal_actions_discarded":true});
        if bridge.is_some() {
            provenance["read_state_bridge"] = bridge_trace;
        }
        (codes, ids, scores, provenance)
    } else {
        let ids = e.causal_no_source(actual)?;
        let cfg = model.context_config();
        let (tables, geometry) = model.context_encoder_parts();
        let mut st = NativeContextState::new(cfg.heads, cfg.lanes_per_head)?;
        for &id in &ids {
            st.step(id as usize, tables, geometry)?;
        }
        (
            st.states().to_vec(),
            Vec::new(),
            Vec::new(),
            json!({"causal_tokens":ids,"no_source":true}),
        )
    };
    let mut gs = vec![0i64; g.vocab_size()];
    let mut counts = GenerateReadCounts::default();
    g.score_into(&codes, &mut gs, &mut counts)?;
    let actions = pool.reduce_trace(&gs, &copy_ids, &copy_scores)?;
    Ok(
        json!({"actual_prefix_ids":actual,"retained_state_codes":codes.iter().map(|s|s.index()).collect::<Vec<_>>(),"copy_token_ids":copy_ids,"copy_raw_scores_q24":copy_scores,"generate_raw_scores_sha256":sha256_bytes(&serde_json::to_vec(&gs)?),"pool":{"summary":actions.summary,"token_masses":actions.token_masses.iter().map(|m|[m.token_id as u64,m.weight_q31,m.generate_weight_q31,m.copy_weight_q31]).collect::<Vec<_>>()},"source_provenance":provenance,"generate_costs":counts}),
    )
}
fn evaluate(
    a: &Args,
    name: &str,
    model: &IntegerRealizer,
    g: &NativeGeometricGenerate,
    bridge: Option<&NativeGeometricReadStateBridge>,
    exp: &[u8],
    eps: &[Episode],
    tok: &ByteBpeTokenizer,
    cue: &CueAngularQ4,
    prefix: &PrefixAngularQ4,
    start: Instant,
) -> Result<Value> {
    let mut pool = NativeVocabularyActions::new(model.binding().clone(), exp)?;
    let mut rows = Vec::new();
    let mut complete = 0;
    let mut ce = 0.;
    let mut tokens = 0;
    for e in eps {
        deadline(a, start)?;
        let mut canonical = Vec::new();
        let mut rowce = 0.;
        for (t, &target) in e.target.iter().enumerate() {
            let mut step =
                native_step(model, g, bridge, &mut pool, e, &e.target[..t], cue, prefix)?;
            let total = step["pool"]["summary"]["total_weight_q31"]
                .as_u64()
                .ok_or_else(|| bad("native denominator missing"))?;
            let mass = step["pool"]["token_masses"]
                .as_array()
                .ok_or_else(|| bad("native token masses"))?
                .iter()
                .find(|m| m[0].as_u64() == Some(target as u64))
                .and_then(|m| m[1].as_u64())
                .ok_or_else(|| bad("full Generate target mass missing"))?;
            if total == 0 || mass == 0 {
                return Err(bad("positive native support violated"));
            }
            step["pool"]
                .as_object_mut()
                .ok_or_else(|| bad("pool object"))?
                .remove("token_masses");
            let loss = -(mass as f64 / total as f64).ln();
            rowce += loss;
            canonical.push(json!({"target_label_only":target,"native_ce":loss,"native_target_mass":mass,"native_denominator":total,"native":step}));
            tokens += 1;
        }
        ce += rowce / e.target.len() as f64;
        let mut ids = Vec::new();
        let mut generated = Vec::new();
        let mut eos = false;
        let generation_allowance = 32usize.min(128usize.saturating_sub(e.base_len()));
        for _ in 0..generation_allowance {
            deadline(a, start)?;
            let mut step = native_step(model, g, bridge, &mut pool, e, &ids, cue, prefix)?;
            let id = step["pool"]["summary"]["chosen_token_id"]
                .as_u64()
                .ok_or_else(|| bad("native chosen ID"))? as u32;
            step["pool"]
                .as_object_mut()
                .ok_or_else(|| bad("pool object"))?
                .remove("token_masses");
            generated.push(step);
            ids.push(id);
            if id == model.binding().eos_token_id() {
                eos = true;
                break;
            }
        }
        let text = tok.decode(&ids[..ids.len() - usize::from(eos)]);
        let accepted = eos && e.answers.accepts(&text);
        complete += usize::from(accepted);
        let row = json!({"id":e.packet.id,"canonical_target_ids_labels_only":e.target,"native_equal_episode_ce":rowce/e.target.len()as f64,"canonical":canonical,"generation":generated,"generated_ids":ids,"decoded":text,"eos":eos,"generation_allowance":generation_allowance,"cutoff":if eos{None}else if generation_allowance<32{Some("context-cap")}else{Some("generation-cap")},"complete":accepted});
        let filename = format!("{name}-row-{:04}.json", rows.len());
        write(a, &filename, &row)?;
        rows.push(json!({"id":e.packet.id,"native_equal_episode_ce":rowce/e.target.len()as f64,"complete":accepted,"eos":eos,"generated_ids":ids,"row_file":filename,"row_sha256":sha256_file(&a.out.join(filename))?}));
    }
    let result = json!({"cases":eps.len(),"target_positions":tokens,"complete":complete,"native_equal_episode_ce":ce/eps.len()as f64,"rows":rows,"runtime":"integer context/Copy/Generate; one full-vocabulary pool; old terminals excluded; actual emitted feedback","selection_primary":"complete own-prefix answers","selection_tie":"native equal-episode CE; earliest checkpoint"});
    write(a, &format!("{name}.json"), &result)?;
    Ok(result)
}
fn checkpoint(
    a: &Args,
    step: usize,
    source: &SourceRealizerWeights,
    frozen: &NativeSourceRealizer,
    g: &GenerateLearningWeights,
    bridge: Option<&BridgeLearningWeights>,
    cue: &CueAngularQ4,
    prefix: &PrefixAngularQ4,
) -> Result<(
    IntegerRealizer,
    NativeGeometricGenerate,
    Option<NativeGeometricReadStateBridge>,
    Value,
)> {
    let root = a.out.join(format!("checkpoint-{step:04}"));
    let projected_checkpoint = size(&a.source_weights)?
        + size(&a.native_artifact)?
        + g.parameters()
            .values()
            .map(|v| v.elem_count() as u64 * 4)
            .sum::<u64>()
        + bridge.map_or(0, |b| {
            b.parameters()
                .values()
                .map(|v| v.elem_count() as u64 * 4)
                .sum::<u64>()
                + b.padded_native_coefficient_bytes() as u64
                + 65_536
        })
        + 8_388_608;
    if size(&a.out)? + projected_checkpoint > a.maximum_report_bytes - 1_048_576 {
        return Err(bad(
            "checkpoint projected bytes exceed declared cap before export",
        ));
    }
    fs::create_dir(&root)?;
    let native = compile_learning_source(source, frozen, &a.arm)?;
    source.save_source(&root.join("source"))?;
    native.save(&root.join("native"))?;
    let binding = native.execution_binding()?;
    if a.arm == "output-only" && binding != frozen.artifact_binding()? {
        return Err(bad("output-only context source identity changed"));
    }
    let integer = IntegerRealizer::load_native(&root.join("native"), &binding)?;
    let c = integer.compile_cue_carrier(cue.clone())?;
    let p = integer.compile_prefix_transport(&c, prefix_clone(prefix)?)?;
    fs::create_dir(root.join("cue"))?;
    fs::create_dir(root.join("prefix"))?;
    fs::write(
        root.join("cue/native-metadata.json"),
        serde_json::to_vec_pretty(c.metadata())?,
    )?;
    fs::write(root.join("cue/cue-q4.bin"), cue.packed_coefficients())?;
    if let Some(j) = cue.joint() {
        fs::write(root.join("cue/cue-joint-q4.bin"), j.packed_coefficients())?;
    }
    fs::write(
        root.join("prefix/native-metadata.json"),
        serde_json::to_vec_pretty(p.metadata())?,
    )?;
    fs::write(
        root.join("prefix/prefix-q4.bin"),
        prefix.packed_coefficients(),
    )?;
    let mut shadows = BTreeMap::new();
    fs::create_dir(root.join("generate-source"))?;
    for (name, var) in g.parameters() {
        let values = var.flatten_all()?.to_vec1::<f32>()?;
        if values.iter().any(|v| !v.is_finite()) {
            return Err(bad("nonfinite Generate checkpoint master"));
        }
        let raw = values
            .into_iter()
            .flat_map(f32::to_le_bytes)
            .collect::<Vec<_>>();
        fs::write(
            root.join("generate-source").join(format!("{name}.f32le")),
            &raw,
        )?;
        shadows.insert(
            name,
            json!({"shape":var.dims(),"bytes":raw.len(),"sha256":sha256_bytes(&raw)}),
        );
    }
    fs::write(
        root.join("generate-source/metadata.json"),
        serde_json::to_vec_pretty(
            &json!({"parameters":shadows,"tokenizer_sha256":native.binding().tokenizer_sha256(),"protocol":native.binding().protocol(),"seed":a.seed,"lanes":g.lanes(),"scope":"offline source masters; excluded from serving"}),
        )?,
    )?;
    let readcue = cue_payload(&root.join("cue"))?;
    let readprefix = prefix_payload(&root.join("prefix"))?;
    let rec = integer.compile_cue_carrier(readcue)?;
    let rep = integer.compile_prefix_transport(&rec, readprefix)?;
    if serde_json::to_value(rec.metadata())? != read(&root.join("cue/native-metadata.json"))?
        || serde_json::to_value(rep.metadata())? != read(&root.join("prefix/native-metadata.json"))?
    {
        return Err(bad("checkpoint sidecar independent disk reload mismatch"));
    }
    let decoder = g.export_native()?;
    let bytes = decoder.to_bytes()?;
    fs::write(root.join("generate.bin"), &bytes)?;
    let reloaded = NativeGeometricGenerate::from_bytes(
        &fs::read(root.join("generate.bin"))?,
        integer.binding(),
    )?;
    if reloaded.to_bytes()? != bytes {
        return Err(bad("Generate independent reload differs"));
    }
    let (reloaded_bridge, bridge_receipt) = if let Some(bridge) = bridge {
        let native = bridge.export_native()?;
        let bytes = native.to_bytes()?;
        fs::write(root.join("read-state-bridge.bin"), &bytes)?;
        let reload = NativeGeometricReadStateBridge::from_bytes(
            &fs::read(root.join("read-state-bridge.bin"))?,
            integer.binding(),
        )?;
        if reload.to_bytes()? != bytes {
            return Err(bad("bridge independent native reload differs"));
        }
        fs::create_dir(root.join("read-state-bridge-source"))?;
        let mut parameters = BTreeMap::new();
        for (name, var) in bridge.parameters() {
            let values = var.flatten_all()?.to_vec1::<f32>()?;
            if values.iter().any(|v| !v.is_finite()) {
                return Err(bad("nonfinite bridge master"));
            }
            let raw = values
                .into_iter()
                .flat_map(f32::to_le_bytes)
                .collect::<Vec<_>>();
            fs::write(
                root.join("read-state-bridge-source")
                    .join(format!("{name}.f32le")),
                &raw,
            )?;
            parameters.insert(
                name,
                json!({"shape":var.dims(),"bytes":raw.len(),"sha256":sha256_bytes(&raw)}),
            );
        }
        let receipt = json!({"enabled":true,"native_sha256":sha256_bytes(&bytes),"metadata":reload.metadata(),"padded_native_coefficient_bytes":bridge.padded_native_coefficient_bytes(),"packed_bytes_changed_from_zero":reload.packed_bias().iter().chain(reload.packed_relative()).filter(|&&b|b!=0).count(),"independent_disk_reload":true,"source_parameters":parameters,"learning_rate":a.read_state_bridge_learning_rate,"selection":"earliest raw all-head Copy maximum; no label input","resume":"NOT_SUPPORTED; Adam moments not exported"});
        fs::write(
            root.join("read-state-bridge-source/metadata.json"),
            serde_json::to_vec_pretty(&receipt)?,
        )?;
        (Some(reload), receipt)
    } else {
        (None, json!({"enabled":false}))
    };
    let contextbytes = fs::read(root.join("native/consumer/context-q4.bin"))?;
    let oldcontext = fs::read(a.native_artifact.join("consumer/context-q4.bin"))?;
    if contextbytes.len() != oldcontext.len() {
        return Err(bad("native context width changed"));
    }
    let context_changes = contextbytes
        .iter()
        .zip(&oldcontext)
        .filter(|(a, b)| a != b)
        .count();
    let potential_bytes = fs::read(root.join("native/consumer/potential-q4.bin"))?;
    let old_potential = fs::read(a.native_artifact.join("consumer/potential-q4.bin"))?;
    if potential_bytes.len() != old_potential.len() {
        return Err(bad("native potential width changed"));
    }
    let potential_changes = potential_bytes
        .iter()
        .zip(&old_potential)
        .filter(|(a, b)| a != b)
        .count();
    if a.arm != "joint-potential" && potential_changes != 0 {
        return Err(bad("frozen potential payload changed"));
    }
    let receipt = json!({"read_state_bridge":bridge_receipt,"step":step,"parent":binding,"balanced_token_geometry":a.balanced_token_geometry,"phase_balanced_token_loss":a.phase_balanced_token_loss,"read_state_bridge_enabled":a.read_state_bridge,"read_state_bridge_learning_rate":a.read_state_bridge_learning_rate,"training_loss_weight_policy":loss_weight_policy(a.phase_balanced_token_loss),"generate_sha256":sha256_bytes(&bytes),"cue_metadata_sha256":sha256_file(&root.join("cue/native-metadata.json"))?,"prefix_metadata_sha256":sha256_file(&root.join("prefix/native-metadata.json"))?,"cue_payload_sha256":sha256_bytes(cue.packed_coefficients()),"prefix_payload_sha256":sha256_bytes(prefix.packed_coefficients()),"native_independently_reloaded":true,"context_packed_bytes_changed_from_donor":context_changes,"context_packed_sha256":sha256_bytes(&contextbytes),"frozen_scoring_except_context":a.arm != "joint-potential","learned_potential_coefficients":a.arm == "joint-potential","potential_packed_bytes_changed_from_donor":potential_changes,"potential_packed_sha256":sha256_bytes(&potential_bytes),"generation_f32_source_access":false,"sidecars_independently_disk_reloaded_verified":true,"training_resume":"NOT_SUPPORTED; source masters retained, Adam moment states not exported"});
    fs::write(
        root.join("receipt.json"),
        serde_json::to_vec_pretty(&receipt)?,
    )?;
    if size(&a.out)? > a.maximum_report_bytes - 1_048_576 {
        return Err(bad("checkpoint exceeds complete report cap"));
    }
    Ok((integer, reloaded, reloaded_bridge, receipt))
}

const LOSS_PHASE_NAMES: [&str; 3] = ["entry", "later_copy_covered", "later_generate_only"];
fn loss_weight_policy(enabled: bool) -> &'static str {
    if enabled {
        "equal-nonempty-phases-per-episode/1;entry-position0;later-exact-allsource-token-membership;empty-phase-zero;remaining-phases-equal;no-runtime-gate"
    } else {
        "legacy-equal-episode-token-mean/1"
    }
}
struct EpisodeLossWeights {
    counts: [usize; 3],
    phases: Vec<usize>,
    weights: Vec<f64>,
}
/// Offline label weighting only, called after actual target-free action admission.
/// Candidate membership does not imply correct-source identity or force Copy.
fn episode_loss_weights(
    target: &[u32],
    admitted_copy_ids: &BTreeSet<u32>,
    batch_episodes: usize,
    balanced: bool,
) -> Result<EpisodeLossWeights> {
    if target.is_empty() || batch_episodes == 0 {
        return Err(bad("loss weighting requires nonempty episode and batch"));
    }
    let phases = target
        .iter()
        .enumerate()
        .map(|(t, id)| {
            if t == 0 {
                0
            } else if admitted_copy_ids.contains(id) {
                1
            } else {
                2
            }
        })
        .collect::<Vec<_>>();
    let mut counts = [0usize; 3];
    for &phase in &phases {
        counts[phase] += 1;
    }
    let nonempty = counts.iter().filter(|&&n| n != 0).count();
    let weights = phases
        .iter()
        .map(|&phase| {
            if balanced {
                1. / nonempty as f64 / counts[phase] as f64 / batch_episodes as f64
            } else {
                // Keep exact prior floating arithmetic and operation order.
                1. / target.len() as f64 / batch_episodes as f64
            }
        })
        .collect();
    Ok(EpisodeLossWeights {
        counts,
        phases,
        weights,
    })
}

#[derive(Clone, Copy)]
struct BridgeAdmissionControl {
    expect_selected_source_state: bool,
    selector_credit: bool,
}
// A declared algebraic positive control, not a trained checkpoint. Every lane
// uniquely chooses a=d, hence q*(inverse(q)*k)=k in the authenticated H4 frame.
fn source_identity_bridge(
    binding: &uor_r4_integer::geometric_source_actions::SourceActionBinding,
    lanes: usize,
    device: &Device,
) -> Result<BridgeLearningWeights> {
    let bias = vec![0u8; lanes * 60];
    let mut relative = vec![0u8; lanes * 120 * 64];
    for lane in 0..lanes {
        for action in 0..120 {
            let at = (lane * 120 + action) * 128 + action;
            relative[at >> 1] |= 7 << if at & 1 == 0 { 0 } else { 4 };
        }
    }
    let native = NativeGeometricReadStateBridge::compile(binding, lanes, &bias, &relative)?;
    let weights = BridgeLearningWeights::from_native(&native, binding, device)?;
    if weights.export_native()?.to_bytes()? != native.to_bytes()? {
        return Err(bad("positive-control bridge native export differs"));
    }
    Ok(weights)
}
fn selector_gradient_contributions(
    enabled: &BTreeMap<String, Tensor>,
    disabled: &BTreeMap<String, Tensor>,
    device: &Device,
) -> Result<Value> {
    if enabled.keys().ne(disabled.keys()) {
        return Err(bad("selector control gradient connectivity differs"));
    }
    let mut families = BTreeMap::new();
    let mut selector_nonzero_potential = false;
    for (name, on) in enabled {
        let off = &disabled[name];
        if on.shape() != off.shape()
            || !on.device().same_device(device)
            || !off.device().same_device(device)
        {
            return Err(bad("selector control gradient shape/device differs"));
        }
        let delta = (on - off)?.detach();
        let on_l2 = on.sqr()?.sum_all()?.sqrt()?.to_scalar::<f32>()?;
        let off_l2 = off.sqr()?.sum_all()?.sqrt()?.to_scalar::<f32>()?;
        let delta_l2 = delta.sqr()?.sum_all()?.sqrt()?.to_scalar::<f32>()?;
        let delta_max = delta.abs()?.max_all()?.to_scalar::<f32>()?;
        if [on_l2, off_l2, delta_l2, delta_max]
            .iter()
            .any(|x| !x.is_finite())
        {
            return Err(bad("nonfinite selector control device reduction"));
        }
        selector_nonzero_potential |= name.starts_with("consumer.potential.") && delta_max > 0.;
        families.insert(name.clone(),json!({"selector_enabled_l2":on_l2,"selector_disabled_l2":off_l2,"selector_delta_l2":delta_l2,"selector_delta_max_abs":delta_max,"selector_delta_nonzero":delta_max>0.,"elements":on.elem_count(),"device":if device.is_cuda(){"CUDA"}else{"CPU"},"off_scope":if name.starts_with("consumer.potential."){"direct Copy emission coefficient credit"}else{"factual bridge/Generate and direct emission credit; not pool-only"}}));
    }
    Ok(
        json!({"families":families,"selector_nonzero_potential":selector_nonzero_potential,"status":if selector_nonzero_potential{"NONZERO_SELECTOR_POTENTIAL_CREDIT"}else{"NO_SELECTOR_ESCAPE_AT_CONTROL"},"decoder_utility_scope":"positive selector delta witnesses nonconstant decoder utility projected onto occurrence alternatives; a zero aggregate does not distinguish constant utility from cancellation or scorer null directions","scope":"same exact native forward; device subtraction enabled-minus-disabled isolates selector-score adjoint contribution; scalar reductions only; no optimizer use or learned-language claim"}),
    )
}
fn batch(
    a: &Args,
    source: &SourceRealizerWeights,
    frozen: &NativeSourceRealizer,
    g: &GenerateLearningWeights,
    bridge: Option<&BridgeLearningWeights>,
    cueq: &CueAngularQ4,
    prefixq: &PrefixAngularQ4,
    exp: &[u8],
    eps: &[Episode],
    indices: &[usize],
    device: &Device,
    start: Instant,
    independent: Option<&IntegerRealizer>,
    backward_chunk: usize,
    retain_inactive: bool,
    bridge_control: Option<BridgeAdmissionControl>,
) -> Result<(BTreeMap<String, Tensor>, Value)> {
    let begun = Instant::now();
    let stage = Instant::now();
    let current = compile_learning_source(source, frozen, &a.arm)?;
    device.synchronize()?;
    let source_rebound_seconds = stage.elapsed().as_secs_f64();
    let stage = Instant::now();
    let prepared = if a.arm == "joint-potential" {
        source.prepare_context_potential_on_device(&current, device)?
    } else {
        source.prepare_on_device(&current, device)?
    };
    device.synchronize()?;
    let source_device_preparation_seconds = stage.elapsed().as_secs_f64();
    let stage = Instant::now();
    let cue = current.compile_cue_carrier(cueq.clone())?;
    let prefix = current.compile_prefix_transport(&cue, prefix_clone(prefixq)?)?;
    device.synchronize()?;
    let cue_prefix_seconds = stage.elapsed().as_secs_f64();
    let stage = Instant::now();
    let snapshot = g.prepare_native()?;
    device.synchronize()?;
    let generate_native_preparation_seconds = stage.elapsed().as_secs_f64();
    let stage = Instant::now();
    let bridge_snapshot = bridge
        .map(BridgeLearningWeights::prepare_native)
        .transpose()?;
    device.synchronize()?;
    let bridge_native_preparation_seconds = stage.elapsed().as_secs_f64();
    let stage = Instant::now();
    // Keep independent admission intact without allocating diagnostic-only
    // native reloads and vocabulary state for ordinary optimizer batches.
    let mut independent_diagnostics = if independent.is_some() {
        let generate =
            NativeGeometricGenerate::from_bytes(&snapshot.native.to_bytes()?, g.binding())?;
        let bridge = bridge_snapshot
            .as_ref()
            .map(|b| NativeGeometricReadStateBridge::from_bytes(&b.native.to_bytes()?, g.binding()))
            .transpose()?;
        let pool = NativeVocabularyActions::new(g.binding().clone(), exp)?;
        Some((generate, bridge, pool))
    } else {
        None
    };
    device.synchronize()?;
    let independent_diagnostics_seconds = stage.elapsed().as_secs_f64();
    let staged = begun.elapsed().as_secs_f64();
    let staging_subtimings = json!({
        "source_rebound_seconds": source_rebound_seconds,
        "source_device_preparation_seconds": source_device_preparation_seconds,
        "cue_prefix_seconds": cue_prefix_seconds,
        "generate_native_preparation_seconds": generate_native_preparation_seconds,
        "bridge_native_preparation_seconds": bridge_native_preparation_seconds,
        "independent_diagnostics_seconds": independent_diagnostics_seconds,
        "independent_diagnostics_constructed": independent_diagnostics.is_some(),
        "scope": "host wall elapsed with learning device synchronized at each stage endpoint; includes download, validation, compilation, device work and synchronization overhead; not isolated GPU kernel time; first stage may include preceding pending device work",
    });
    let mut learner = PreparedBankGenerate::new(&prepared, g, &snapshot, exp)?
        .with_prefix_temporal_utility(a.prefix_temporal_utility);
    if let (Some(weights), Some(snapshot)) = (bridge, bridge_snapshot.as_ref()) {
        learner = learner.with_read_state_bridge(weights, snapshot)?;
    }
    let selector_credit_enabled = bridge_control.map_or(true, |c| c.selector_credit);
    learner = learner.with_read_selector_credit(selector_credit_enabled);
    let params = parameters(
        source,
        g,
        bridge,
        if a.arm == "joint-potential" {
            "joint-potential"
        } else {
            "joint"
        },
    );
    let active = parameters(source, g, bridge, &a.arm);
    let mut sums = BTreeMap::<String, Tensor>::new();
    let mut nativece = 0.;
    let mut weighted_native_objective = 0.;
    let mut phase_positions = [0usize; 3];
    let mut phase_weighted_native_loss = [0f64; 3];
    let mut episode_phase_counts = Vec::new();
    let mut positions = 0;
    let mut backwardseconds = 0.;
    let mut forwardseconds = 0.;
    let mut pending: Option<Tensor> = None;
    let mut pending_count = 0usize;
    let mut backward_calls = 0usize;
    let mut native_forward_packets = Vec::new();
    let mut bridge_receipts = Vec::new();
    let mut first_context_receipts = Vec::new();
    let mut first_context_backward_seconds = 0.;
    for &index in indices {
        let e = &eps[index];
        let compiled_copy_union = e
            .views
            .iter()
            .flatten()
            .flat_map(|view| view.emitted_token_ids().iter().copied())
            .collect::<BTreeSet<_>>();
        let mut episode_weights: Option<EpisodeLossWeights> = None;
        for (t, &target) in e.target.iter().enumerate() {
            deadline(a, start)?;
            let f = Instant::now();
            let out = if e.has_source() {
                learner.forward_bank(
                    &e.segments()?,
                    &e.packet.query_ids,
                    &e.target[..t],
                    &cue,
                    &prefix,
                )?
            } else {
                learner.forward_no_source(&e.causal_no_source(&e.target[..t])?)?
            };
            let mut distinct_candidate_source_states = Value::Null;
            if let Some(model) = independent {
                let (independent_generate, independent_bridge, independent_pool) =
                    independent_diagnostics
                        .as_mut()
                        .ok_or_else(|| bad("independent admission diagnostics missing"))?;
                let expected = native_step(
                    model,
                    independent_generate,
                    independent_bridge.as_ref(),
                    independent_pool,
                    e,
                    &e.target[..t],
                    cueq,
                    prefixq,
                )?;
                if let Some((ordinal, result)) = &out.read_state_bridge {
                    let native_bridge = &expected["source_provenance"]["read_state_bridge"];
                    distinct_candidate_source_states =
                        native_bridge["distinct_candidate_source_states"].clone();
                    if native_bridge["selected_ordinal"] != json!(ordinal)
                        || native_bridge["action_codes"]
                            != json!(result
                                .action_codes
                                .iter()
                                .map(|c| c.index())
                                .collect::<Vec<_>>())
                        || native_bridge["action_scores_q24"] != json!(result.action_scores_q24)
                        || native_bridge["native_costs"] != serde_json::to_value(result.counts)?
                    {
                        return Err(bad("bridge independent selected occurrence/action/score/cost parity differs"));
                    }
                    // Zero initialization is an exact native escape from the
                    // added route: factual identity transport preserves original
                    // query codes, so Generate scores and the Copy pool agree.
                    let reference_state =
                        if bridge_control.is_some_and(|c| c.expect_selected_source_state) {
                            &native_bridge["selected_source_state_codes"]
                        } else {
                            &native_bridge["query_state_codes"]
                        };
                    if &expected["retained_state_codes"] != reference_state {
                        return Err(bad("bridge admission did not preserve declared identity/selected-source state"));
                    }
                }
                if expected["pool"]["summary"] != serde_json::to_value(&out.actions.summary)?
                    || expected["copy_token_ids"] != json!(out.copy_token_ids)
                    || expected["copy_raw_scores_q24"] != json!(out.copy_scores_q24)
                    || expected["retained_state_codes"]
                        != json!(out
                            .final_state_codes
                            .iter()
                            .map(|s| s.index())
                            .collect::<Vec<_>>())
                    || expected["generate_raw_scores_sha256"]
                        != json!(sha256_bytes(&serde_json::to_vec(&out.generate.scores_q24)?))
                {
                    return Err(bad(
                        "actual initial CPU integer / CUDA composed hard-pool parity differs",
                    ));
                }
            }
            if let Some((ordinal, result)) = &out.read_state_bridge {
                let identity = result
                    .action_codes
                    .iter()
                    .all(|c| *c == uor_r4_integer::h4_tables::H4Code::IDENTITY);
                if independent.is_some() && bridge_control.is_none() && !identity {
                    return Err(bad("zero bridge initial admission changed native state"));
                }
                bridge_receipts.push(json!({"episode_id":e.packet.id,"position":t,"selected_ordinal":ordinal,"action_codes":result.action_codes.iter().map(|c|c.index()).collect::<Vec<_>>(),"distinct_candidate_source_states":distinct_candidate_source_states,"native_costs":result.counts,"selector_native_alternative_evaluations":out.copy_token_ids.len(),"selector_native_alternative_counts_per_evaluation":result.counts,"cost_scope":"factual native apply plus declared equal-shape alternative applies; GPU adjoint/index staging not counted by native reads","identity_actions":identity,"validation_scalar_reads":result.validation_scalar_reads,"credit_scope":result.credit_scope}));
            } else if bridge.is_some() && e.has_source() {
                return Err(bad("source bank omitted enabled bridge"));
            }
            if bridge_control.is_some() {
                native_forward_packets.push(json!({"episode_id":e.packet.id,"position":t,"actual_prefix_ids":&e.target[..t],"copy_ids":out.copy_token_ids,"copy_scores_sha256":sha256_bytes(&serde_json::to_vec(&out.copy_scores_q24)?),"generate_scores_sha256":sha256_bytes(&serde_json::to_vec(&out.generate.scores_q24)?),"pool_summary":out.actions.summary,"poststate":out.final_state_codes.iter().map(|c|c.index()).collect::<Vec<_>>(),"bridge_selected_actions":out.read_state_bridge.as_ref().map(|(j,b)|json!({"ordinal":j,"actions":b.action_codes.iter().map(|c|c.index()).collect::<Vec<_>>()}))}));
            }
            // Bind stable membership to ACTUAL target-free forward admission,
            // not raw Context IDs or a selected-record/label-derived candidate set.
            let actual_copy_union = out.copy_token_ids.iter().copied().collect::<BTreeSet<_>>();
            if actual_copy_union != compiled_copy_union {
                return Err(bad(
                    "phase weighting actual Copy admission differs from compiled all-source union",
                ));
            }
            if episode_weights.is_none() {
                let plan = episode_loss_weights(
                    &e.target,
                    &actual_copy_union,
                    indices.len(),
                    a.phase_balanced_token_loss,
                )?;
                episode_phase_counts.push(json!({"id":e.packet.id,"counts":plan.counts,"nonempty_phases":plan.counts.iter().filter(|&&n| n!=0).count(),"episode_total_weight":plan.weights.iter().sum::<f64>()}));
                episode_weights = Some(plan);
            }
            let plan = episode_weights
                .as_ref()
                .ok_or_else(|| bad("episode weight plan absent"))?;
            let weight = plan.weights[t];
            let phase = plan.phases[t];
            // Targets enter loss/weighting only after native action construction.
            let loss = out.loss(target)?;
            let mass = out
                .actions
                .token_masses
                .iter()
                .find(|m| m.token_id == target)
                .ok_or_else(|| bad("native target support"))?
                .weight_q31;
            let nll = -(mass as f64 / out.actions.summary.total_weight_q31 as f64).ln();
            if !nll.is_finite() {
                return Err(bad("invalid native objective"));
            }
            nativece += nll / e.target.len() as f64 / indices.len() as f64;
            weighted_native_objective += nll * weight;
            phase_positions[phase] += 1;
            phase_weighted_native_loss[phase] += nll * weight;
            let scaled = loss.affine(weight, 0.)?;
            device.synchronize()?;
            forwardseconds += f.elapsed().as_secs_f64();
            // Auxiliary admission measurement only: preserve the same native
            // pool/loss forward but stop direct Copy emission's adjoint. With
            // the bridge enabled, Generate still depends on Copy selector scores:
            // those potential/context paths are intentionally retained. It never
            // supplies optimizer gradients or changes the fitted objective.
            if a.mode == "admission"
                && backward_chunk == 1
                && t == 0
                && !out.copy_token_ids.contains(&target)
            {
                let diagnostic_start = Instant::now();
                let stopped_copy = out.copy.as_ref().map(|c| c.copy_raw.detach());
                let diagnostic =
                    uor_r4_training::geometric_generate_learning::vocabulary_marginal_loss(
                        &out.actions,
                        &out.generate.raw_scores,
                        stopped_copy.as_ref(),
                        target,
                    )?
                    .affine(weight, 0.)?;
                let diagnostic_grads = diagnostic.backward()?;
                let mut context_families = BTreeMap::new();
                for (name, variable) in &params {
                    if !name.starts_with("consumer.context.") {
                        continue;
                    }
                    let entry = if let Some(gradient) = diagnostic_grads.get(variable.as_tensor()) {
                        if !gradient.device().same_device(device) {
                            return Err(bad("first Generate context diagnostic CPU fallback"));
                        }
                        let l2 = gradient.sqr()?.sum_all()?.sqrt()?.to_scalar::<f32>()?;
                        if !l2.is_finite() {
                            return Err(bad("nonfinite first Generate context diagnostic"));
                        }
                        json!({"connected":true,"l2":l2,"elements":variable.elem_count(),"device":if device.is_cuda(){"CUDA"}else{"CPU"}})
                    } else {
                        json!({"connected":false,"l2":0.,"elements":variable.elem_count()})
                    };
                    context_families.insert(name.clone(), entry);
                }
                device.synchronize()?;
                let seconds = diagnostic_start.elapsed().as_secs_f64();
                first_context_backward_seconds += seconds;
                let mut receipt = json!({"id":e.packet.id,"position":0,"gold_absent_copy":true,"auxiliary_backward_seconds":seconds,"direct_copy_emission_adjoint_detached":true,"bridge_selector_copy_score_adjoint_retained":bridge.is_some() && selector_credit_enabled});
                let family_key = if bridge.is_some() && selector_credit_enabled {
                    "generate_branch_including_selector_context_families"
                } else if bridge.is_some() {
                    "generate_branch_without_selector_context_families"
                } else {
                    "generate_only_context_families"
                };
                receipt[family_key] = json!(context_families);
                first_context_receipts.push(receipt);
            }
            pending = Some(match pending.take() {
                Some(previous) => (&previous + &scaled)?,
                None => scaled,
            });
            pending_count += 1;
            if pending_count == backward_chunk {
                let b = Instant::now();
                let measured_first = a.mode == "admission"
                    && backward_chunk == 1
                    && t == 0
                    && !out.copy_token_ids.contains(&target);
                let combined_context = accumulate_backward(
                    pending.take().ok_or_else(|| bad("missing chunk loss"))?,
                    &params,
                    &mut sums,
                    device,
                    measured_first,
                )?;
                if let Some(context) = combined_context {
                    let row = first_context_receipts
                        .last_mut()
                        .ok_or_else(|| bad("missing first diagnostic row"))?;
                    row["whole_pool_context_families"] = context;
                }
                device.synchronize()?;
                backwardseconds += b.elapsed().as_secs_f64();
                backward_calls += 1;
                pending_count = 0;
            }
            positions += 1;
        }
    }
    if let Some(loss) = pending.take() {
        let b = Instant::now();
        accumulate_backward(loss, &params, &mut sums, device, false)?;
        device.synchronize()?;
        backwardseconds += b.elapsed().as_secs_f64();
        backward_calls += 1;
    }
    let mut family = BTreeMap::new();
    let mut norm2 = 0.;
    for (name, var) in &params {
        let entry = if let Some(grad) = sums.get(name) {
            let l2 = grad.sqr()?.sum_all()?.sqrt()?.to_scalar::<f32>()?;
            if !l2.is_finite() {
                return Err(bad("nonfinite gradient"));
            }
            if active.contains_key(name) {
                norm2 += l2 as f64 * l2 as f64;
            }
            json!({"connected":true,"elements":var.elem_count(),"l2":l2,"device":"CUDA","optimizer_active":active.contains_key(name)})
        } else {
            json!({"connected":false,"elements":var.elem_count(),"l2":0.})
        };
        family.insert(name.clone(), entry);
    }
    if !family
        .iter()
        .any(|(n, v)| n.starts_with("generate.") && v["l2"].as_f64().unwrap_or(0.) > 0.)
    {
        return Err(bad("decoder credit disconnected"));
    }
    let first_context_diagnostic = json!({"status":if a.mode != "admission" {"NOT_MEASURED_FIT_UNCHANGED"} else if backward_chunk != 1 {"NOT_MEASURED_REQUIRES_CHUNK1"} else if first_context_receipts.is_empty() {"NOT_MEASURED_NO_ELIGIBLE_FIRST_TARGETS"} else {"MEASURED"},"rows":first_context_receipts,"auxiliary_backward_calls":first_context_receipts.len(),"auxiliary_backward_seconds":first_context_backward_seconds,"scope":if bridge.is_some() && selector_credit_enabled { "absent-Copy first canonical positions; direct Copy emission adjoint detached only in auxiliary backward; Generate branch retains bridge selector Copy-score potential/context paths plus hard120 carrier credit; same native pool and loss weights; no per-root utility measurement or optimizer use" } else if bridge.is_some() { "absent-Copy first canonical positions; direct Copy emission detached in auxiliary backward and selector credit disabled; factual bridge/Generate hard120 carrier credit retained; same native pool and loss weights; no optimizer use" } else { "absent-Copy first canonical positions only; same fullpool forward and declared episode/phase loss weighting, Copy adjoint detached only in auxiliary backward; Generate retained hard120 carrier to actual9context parameter families; no direct per-root utility measurement; no optimizer use or changed objective" }});
    let report = json!({"read_state_bridge":a.read_state_bridge,"bridge_master_download_bytes":bridge_snapshot.as_ref().map_or(0,|b|b.downloaded_master_bytes),"bridge_padded_native_coefficient_bytes":bridge.map_or(0,|b|b.padded_native_coefficient_bytes()),"bridge_forward_receipts":bridge_receipts,"native_forward_packets_sha256":sha256_bytes(&serde_json::to_vec(&native_forward_packets)?),"bridge_zero_escape_native_state_verified":independent.is_some() && bridge.is_some() && bridge_control.is_none(),"bridge_control_selected_source_state_verified":independent.is_some() && bridge_control.is_some_and(|c|c.expect_selected_source_state),"bridge_selector_credit_enabled":bridge.is_some() && bridge_control.map_or(true,|c|c.selector_credit),"bridge_selector_credit_scope":if bridge.is_none() {"NOT_APPLICABLE_NO_BRIDGE"} else if bridge_control.is_some_and(|c|!c.selector_credit) {"selector-adjoint-disabled; factual bridge/Generate and direct emission credit remain"} else {"local detached native occurrence alternatives; factual bridge coefficient/state credit; no derivative of hard argmax and no runtime label"},"episode_indices":indices,"episodes":indices.len(),"target_positions":positions,"native_equal_episode_ce":nativece,"native_weighted_training_objective":weighted_native_objective,"loss_phase_names":LOSS_PHASE_NAMES,"loss_phase_positions":phase_positions,"loss_phase_weighted_native_contributions":phase_weighted_native_loss,"episode_loss_phase_counts":episode_phase_counts,"gradient_families":family,"gradient_global_l2":norm2.sqrt(),"staging_native_snapshot_seconds":staged,"staging_native_snapshot_subtimings":staging_subtimings,"forward_including_host_native_oracle_synced_seconds":forwardseconds,"backward_synced_seconds":backwardseconds,"elapsed_seconds":begun.elapsed().as_secs_f64(),"generate_master_download_bytes":snapshot.downloaded_master_bytes,"potential_credit_scope":if a.arm == "joint-potential" {"opt-in-existing-potential-Q4-selected-all-causal-source-pairs;native-hard-Q24-anchor;device-coefficient-adjoint-plus-unchanged-context-adjoint;cue/prefix-numeric-frozen;no-gate-or-new-normalizer/1"} else {"FROZEN"},"credit_scope":if a.arm == "joint-potential" { "existing-Generate-and-context-credit-plus-learned-geometric-potential-Copy;one-common-fullvocabulary-alias-loss;fixed-cue/prefix" } else if a.prefix_temporal_utility { uor_r4_training::geometric_bank_generate::PREFIX_TEMPORAL_CREDIT_SCOPE } else { uor_r4_training::geometric_bank_generate::CREDIT_SCOPE },"prefix_temporal_utility":a.prefix_temporal_utility,"balanced_token_geometry":a.balanced_token_geometry,"phase_balanced_token_loss":a.phase_balanced_token_loss,"read_state_bridge_enabled":a.read_state_bridge,"read_state_bridge_learning_rate":a.read_state_bridge_learning_rate,"training_loss_weight_policy":loss_weight_policy(a.phase_balanced_token_loss),"token_geometry_initialization":if a.balanced_token_geometry {"balanced-a+j*b-with-existing-offset/1"}else{"legacy-base120-digits-with-existing-offset/1"},"gradient_accumulation":"stream bounded token-chunk backward; declared per-token episode/phase weights applied before sum; detached device F32 gradient accumulation; no host dynamic adjoints","token_backward_chunk":backward_chunk,"backward_calls":backward_calls,"first_generate_context_credit":first_context_diagnostic,"updates":0,"independent_native_hard_pool_parity":independent.is_some(),"output_only_cost_scope":"same composed graph credit is computed for diagnostics; context gradients excluded before global norm/optimizer, context does not update"});
    if !retain_inactive {
        sums.retain(|name, _| active.contains_key(name));
    }
    Ok((sums, report))
}
fn accumulate_backward(
    loss: Tensor,
    params: &BTreeMap<String, Var>,
    sums: &mut BTreeMap<String, Tensor>,
    device: &Device,
    measure_context: bool,
) -> Result<Option<Value>> {
    let grads = loss.backward()?;
    let mut measured = BTreeMap::new();
    for (name, var) in params {
        if measure_context && name.starts_with("consumer.context.") {
            let entry = if let Some(gradient) = grads.get(var.as_tensor()) {
                if !gradient.device().same_device(device) {
                    return Err(bad("first whole-pool context diagnostic CPU fallback"));
                }
                let l2 = gradient.sqr()?.sum_all()?.sqrt()?.to_scalar::<f32>()?;
                if !l2.is_finite() {
                    return Err(bad("nonfinite whole-pool first context diagnostic"));
                }
                json!({"connected":true,"l2":l2,"elements":var.elem_count(),"device":if device.is_cuda(){"CUDA"}else{"CPU"}})
            } else {
                json!({"connected":false,"l2":0.,"elements":var.elem_count()})
            };
            measured.insert(name.clone(), entry);
        }
        if let Some(grad) = grads.get(var.as_tensor()) {
            if !grad.device().same_device(device) {
                return Err(bad("gradient CPU fallback"));
            }
            if !grad.sqr()?.sum_all()?.to_scalar::<f32>()?.is_finite() {
                return Err(bad(&format!(
                    "nonfinite device gradient before accumulation: {name}"
                )));
            }
            let next = match sums.get(name) {
                Some(old) => old.add(grad)?.detach(),
                None => grad.detach(),
            };
            sums.insert(name.clone(), next);
        }
    }
    Ok(if measure_context {
        Some(json!(measured))
    } else {
        None
    })
}
fn chunk_gradient_parity(
    reference: &BTreeMap<String, Tensor>,
    candidate: &BTreeMap<String, Tensor>,
) -> Result<Value> {
    if reference.keys().ne(candidate.keys()) {
        return Err(bad("chunk gradient connectivity differs"));
    }
    let mut families = BTreeMap::new();
    let mut passed = true;
    for (name, r) in reference {
        let c = &candidate[name];
        if r.shape() != c.shape() || !r.device().same_device(c.device()) {
            return Err(bad("chunk gradient shape/device differs"));
        }
        let delta = (c - r)?.abs()?;
        let max_abs = delta.max_all()?.to_scalar::<f32>()?;
        let max_rel = delta
            .broadcast_div(&r.abs()?.clamp(1e-6f32, f32::MAX)?)?
            .max_all()?
            .to_scalar::<f32>()?;
        let envelope = r.abs()?.affine(2e-3, 2e-5)?;
        let ratio = delta
            .broadcast_div(&envelope)?
            .max_all()?
            .to_scalar::<f32>()?;
        let finite = max_abs.is_finite() && max_rel.is_finite() && ratio.is_finite();
        passed &= finite && ratio <= 1.;
        families.insert(name.clone(), json!({"max_absolute_difference":max_abs,"max_relative_difference_floor1e6":max_rel,"maximum_envelope_ratio":ratio,"finite":finite}));
    }
    Ok(
        json!({"status":if passed { "PASS" } else { "FAIL" },"families":families,"criterion":"elementwise abs(delta)<=2e-5+2e-3*abs(reference); maxrel denominator floor1e-6; f32 summation order may differ","device_reductions_only":true,"host_full_adjoints":false}),
    )
}
fn apply(
    optimizer: &mut AdamW,
    params: &BTreeMap<String, Var>,
    gradients: &BTreeMap<String, Tensor>,
    denominator: &Tensor,
) -> Result<()> {
    // A zero scalar graph creates an empty GradStore on the same device. Only the
    // admitted parameter group is inserted; separate optimizers share one clip.
    let first = params
        .values()
        .next()
        .ok_or_else(|| bad("empty optimizer group"))?;
    let mut store = first.as_tensor().sum_all()?.affine(0., 0.)?.backward()?;
    for (name, var) in params {
        if let Some(g) = gradients.get(name) {
            store.insert(var.as_tensor(), g.broadcast_div(denominator)?.detach());
        }
    }
    optimizer.step(&store)?;
    Ok(())
}
fn comparison(a: &Args, new: &Value, old: &Value) -> Result<Value> {
    let n = new["rows"]
        .as_array()
        .ok_or_else(|| bad("comparison rows"))?;
    let o = old["rows"]
        .as_array()
        .ok_or_else(|| bad("comparison rows"))?;
    if n.len() != o.len() {
        return Err(bad("comparison row count"));
    }
    let mut rows = Vec::new();
    for (n, o) in n.iter().zip(o) {
        if n["id"] != o["id"] {
            return Err(bad("comparison row IDs"));
        }
        let nr = read(
            &a.out
                .join(n["row_file"].as_str().ok_or_else(|| bad("row file"))?),
        )?;
        let or = read(
            &a.out
                .join(o["row_file"].as_str().ok_or_else(|| bad("row file"))?),
        )?;
        let nc = nr["canonical"].as_array().ok_or_else(|| bad("canonical"))?;
        let oc = or["canonical"].as_array().ok_or_else(|| bad("canonical"))?;
        if nc.len() != oc.len() {
            return Err(bad("canonical length differs"));
        }
        let mut changedstates = 0;
        let mut changedcopy = 0;
        for (n, o) in nc.iter().zip(oc) {
            if n["target_label_only"] != o["target_label_only"]
                || n["native"]["actual_prefix_ids"] != o["native"]["actual_prefix_ids"]
            {
                return Err(bad("fixed canonical inputs changed"));
            }
            changedstates += usize::from(
                n["native"]["retained_state_codes"] != o["native"]["retained_state_codes"],
            );
            changedcopy += usize::from(
                n["native"]["copy_raw_scores_q24"] != o["native"]["copy_raw_scores_q24"],
            );
        }
        rows.push(json!({"id":n["id"],"old_complete":o["complete"],"new_complete":n["complete"],"old_ce":o["native_equal_episode_ce"],"new_ce":n["native_equal_episode_ce"],"fixedprefix_retained_state_changes":changedstates,"fixedprefix_raw_Copy_score_changes":changedcopy}));
    }
    Ok(
        json!({"rows":rows,"scope":"same complete canonical prefixes; actualgeneration answer counts separately; scores at diverged generated prefixes are not fixed-input causal comparisons"}),
    )
}
fn improved(new: &Value, old: &Value) -> Result<bool> {
    let n = new["complete"]
        .as_u64()
        .ok_or_else(|| bad("selection complete"))?;
    let o = old["complete"]
        .as_u64()
        .ok_or_else(|| bad("selection complete"))?;
    let nc = new["native_equal_episode_ce"]
        .as_f64()
        .ok_or_else(|| bad("selection CE"))?;
    let oc = old["native_equal_episode_ce"]
        .as_f64()
        .ok_or_else(|| bad("selection CE"))?;
    Ok(n > o || (n == o && nc < oc))
}
fn quarter_margins(params: &BTreeMap<String, Var>) -> Result<Value> {
    let mut out = BTreeMap::new();
    for (name, v) in params {
        let vals = v.flatten_all()?.to_vec1::<f32>()?;
        let min = vals
            .iter()
            .map(|&x| {
                let q = x as f64 * 4.;
                (0.5 - (q - q.round()).abs()).max(0.) / 4.
            })
            .fold(f64::INFINITY, f64::min);
        out.insert(name.clone(),json!({"elements":vals.len(),"minimum_quarter_crossing_shadow_distance":min,"host_download_scope":"once initial margin admission; excludes prototype argmax masters"}));
    }
    Ok(json!(out))
}
fn input_hash(p: &Path) -> Result<String> {
    if p.is_file() {
        return Ok(sha256_file(p)?);
    }
    let mut files = Vec::new();
    fn walk(root: &Path, p: &Path, out: &mut Vec<(String, String)>) -> Result<()> {
        for e in fs::read_dir(p)? {
            let path = e?.path();
            if path.is_dir() {
                walk(root, &path, out)?;
            } else {
                out.push((
                    path.strip_prefix(root)?.display().to_string(),
                    sha256_file(&path)?,
                ));
            }
        }
        Ok(())
    }
    walk(p, p, &mut files)?;
    files.sort();
    Ok(sha256_bytes(&serde_json::to_vec(&files)?))
}
fn run(a: &Args, start: Instant) -> Result<Value> {
    let mut seals = BTreeSet::new();
    for p in input_paths(a) {
        seals.insert(seal_for(p)?);
    }
    for root in &seals {
        report_output::verify(root)?;
    }
    let before = seals
        .iter()
        .map(|r| Ok((r.clone(), sha256_file(&r.join("manifest.json"))?)))
        .collect::<Result<BTreeMap<_, _>>>()?;
    let inputs = input_paths(a)
        .iter()
        .map(|p| Ok((p.display().to_string(), input_hash(p)?)))
        .collect::<Result<BTreeMap<_, _>>>()?;
    let device = cuda()?;
    let expected: NativeArtifactBinding = serde_json::from_slice(&fs::read(&a.trusted_binding)?)?;
    let integer = IntegerRealizer::load_native(&a.native_artifact, &expected)?;
    let tokenizerbytes = fs::read(a.native_artifact.join("tokenizer.json"))?;
    let tok = ByteBpeTokenizer::from_tokenizer_json_bytes(&tokenizerbytes)
        .ok_or_else(|| bad("ByteBPE tokenizer unavailable"))?;
    let lanes = integer
        .context_config()
        .heads
        .checked_mul(integer.context_config().lanes_per_head)
        .ok_or_else(|| bad("donor lane width overflow"))?;
    if integer.binding().vocab_size() > 4096 || !(1..=8).contains(&lanes) {
        return Err(bad(
            "actual donor exceeds native decoder vocabulary/lane envelope",
        ));
    }
    let exp = fs::read(&a.canonical_exp)?;
    let public = NativeVocabularyActions::new(integer.binding().clone(), &exp)?;
    let legal = public
        .legal_token_ids()
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    let source = if a.arm == "joint-potential" {
        SourceRealizerWeights::load_context_potential_on_device(
            &a.source_weights,
            &tokenizerbytes,
            &device,
        )?
    } else {
        SourceRealizerWeights::load_source_on_device(&a.source_weights, &tokenizerbytes, &device)?
    };
    let meta = read(&a.native_artifact.join("metadata.json"))?;
    let identity: ConsumerIdentity = serde_json::from_value(meta["identity"].clone())?;
    let frozen = NativeSourceRealizer::load(&a.native_artifact, &source, &identity)?;
    if frozen.artifact_binding()? != expected {
        return Err(bad("source/native/trusted parent mismatch"));
    }
    let cue = cue_payload(&a.cue_bundle)?;
    let prefix = prefix_payload(&a.prefix_bundle)?;
    let nativecue = integer.compile_cue_carrier(cue.clone())?;
    let nativeprefix = integer.compile_prefix_transport(&nativecue, prefix_clone(&prefix)?)?;
    if serde_json::to_value(nativecue.metadata())?
        != read(&a.cue_bundle.join("native-metadata.json"))?
        || serde_json::to_value(nativeprefix.metadata())?
            != read(&a.prefix_bundle.join("native-metadata.json"))?
    {
        return Err(bad("frozen donor sidecar binding differs"));
    }
    let train = load_panel(
        &a.training_inputs,
        &a.training_labels,
        &integer,
        &tok,
        &legal,
        512,
    )?;
    let dev = load_panel(
        &a.development_inputs,
        &a.development_labels,
        &integer,
        &tok,
        &legal,
        512,
    )?;
    if train.len() != 512 || dev.len() != 512 {
        return Err(bad(
            "complete declared 512-row training/open-development panels required",
        ));
    }
    // A changed answer frame does not certify untouched source composition.
    write(
        a,
        "input-admission.json",
        &json!({"input_sha256":inputs,"input_manifests":before,"balanced_token_geometry":a.balanced_token_geometry,"phase_balanced_token_loss":a.phase_balanced_token_loss,"read_state_bridge_enabled":a.read_state_bridge,"read_state_bridge_learning_rate":a.read_state_bridge_learning_rate,"training_loss_weight_policy":loss_weight_policy(a.phase_balanced_token_loss),"token_geometry_initialization":if a.balanced_token_geometry {"balanced-a+j*b-with-existing-offset/1"}else{"legacy-base120-digits-with-existing-offset/1"},"training_cases":train.len(),"development_cases":dev.len(),"prospective_evaluation_supplied_not_loaded":a.fresh_inputs.is_some(),"training_no_source_cases":train.iter().filter(|e|!e.has_source()).count(),"public_legal_ids":legal.len(),"vocab":integer.binding().vocab_size(),"lanes":lanes,"source_parent":expected,"cue_payload_sha256":sha256_bytes(cue.packed_coefficients()),"prefix_payload_sha256":sha256_bytes(prefix.packed_coefficients()),"development_scope":"open construction panel; source overlap allowed and reported; not independent generalization","data_quality_scope":"schema/public-alphabet/causal-context/canonical-roundtrip admission; panel's independent answerability receipt remains required; renamed labels alone are not untouched data"}),
    )?;
    write(
        a,
        "frozen-development-answer-oracles.json",
        &json!({"rule":"input FrozenAnswers used exactly; canonical first accepted bytes plus EOS; no runtime formatting or trimming","cases":dev.iter().map(|e|json!({"id":e.packet.id,"answers":e.answers,"canonical_ids_labels_only":e.target})).collect::<Vec<_>>()}),
    )?;
    let generate = if a.balanced_token_geometry {
        GenerateLearningWeights::seeded_balanced_token_geometry(
            integer.binding().clone(),
            lanes,
            a.seed,
            &device,
        )?
    } else {
        GenerateLearningWeights::seeded(integer.binding().clone(), lanes, a.seed, &device)?
    };
    let bridge = if a.read_state_bridge {
        Some(BridgeLearningWeights::zeroed(
            integer.binding(),
            lanes,
            &device,
        )?)
    } else {
        None
    };
    let bridge_parameters = bridge
        .as_ref()
        .map(BridgeLearningWeights::parameters)
        .unwrap_or_default();
    let gparams = generate.parameters();
    let (prototype, coefficients): (BTreeMap<_, _>, BTreeMap<_, _>) = gparams
        .clone()
        .into_iter()
        .partition(|(n, _)| n == "generate.prototype_choices");
    let context = source.context_state_parameters();
    let potential = source.potential_parameters();
    let bridge_margins = if a.read_state_bridge {
        Some(quarter_margins(&bridge_parameters)?)
    } else {
        None
    };
    let coeffmargins = quarter_margins(&coefficients)?;
    let contextmargins = quarter_margins(&context)?;
    let potentialmargins = if a.arm == "joint-potential" {
        Some(quarter_margins(&potential)?)
    } else {
        None
    };
    write(
        a,
        "optimizer-design.json",
        &json!({"balanced_token_geometry":a.balanced_token_geometry,"phase_balanced_token_loss":a.phase_balanced_token_loss,"read_state_bridge_enabled":a.read_state_bridge,"read_state_bridge_learning_rate":a.read_state_bridge_learning_rate,"training_loss_weight_policy":loss_weight_policy(a.phase_balanced_token_loss),"token_geometry_initialization":if a.balanced_token_geometry {"balanced-a+j*b-with-existing-offset/1"}else{"legacy-base120-digits-with-existing-offset/1"},"bridge_optimizer_active":a.read_state_bridge,"bridge_initial_quarter_margins":bridge_margins,"bridge_initialization":"zero coefficients; identity-first native action ties preserve query exactly","initialization_scope":"label-free tokenID geometry only; same energy seed/pairgraph/coefficient margins/prototype gap; no semantic-distance claim","generate_coefficient_lr":a.learning_rate,"prototype_lr":a.prototype_learning_rate,"context_lr":a.context_learning_rate,"potential_lr":a.potential_learning_rate,"potential_initial_quarter_margins":potentialmargins,"potential_optimizer_active":a.arm == "joint-potential","potential_possible_nonzero_families_on_absent_content":["context_unary","context_radius","context_presence","content_presence"],"potential_structurally_zero_families_on_absent_content":["content_unary","content_radius","pair"],"potential_presence_scope":"content_presence cell0 is a shared baseline; context_presence uses authentic categorical endpoint cells; source-covered Copy retention required","beta1":0.9,"beta2":0.999,"eps":1e-8,"weight_decay":0.,"same_global_clip_l2":1.,"coefficient_initial_quarter_margins":coeffmargins,"context_initial_quarter_margins":contextmargins,"prototype_initial_winner_gap":2.,"reachability_upper_bound_multiplier_128":227.47318,"reachability_scope":"upper bound permits crossings at declared rates; does not guarantee changes or benefit; no initialization-margin manipulation"}),
    )?;
    let first = a
        .admission_episode_indices
        .clone()
        .unwrap_or_else(|| (0..8).collect());
    if first.len() != 8
        || first.iter().any(|i| *i >= train.len())
        || first
            .iter()
            .copied()
            .collect::<std::collections::BTreeSet<_>>()
            .len()
            != 8
    {
        return Err(bad(
            "admission cost sample must name eight distinct training indices",
        ));
    }
    write(
        a,
        "admission-cost-sample.json",
        &json!({"indices":first,"ids":first.iter().map(|i|&train[*i].packet.id).collect::<Vec<_>>(),"scope":"cost probe only; fixed sequential B8 fit schedule unchanged; admission quality subset remains first eight development rows"}),
    )?;
    let (admission_grads, admission) = batch(
        a,
        &source,
        &frozen,
        &generate,
        bridge.as_ref(),
        &cue,
        &prefix,
        &exp,
        &train,
        &first,
        &device,
        start,
        Some(&integer),
        a.token_backward_chunk,
        true,
        None,
    )?;
    if a.arm == "joint-potential" {
        let families = admission["gradient_families"]
            .as_object()
            .ok_or_else(|| bad("potential admission gradient inventory absent"))?;
        if !families.iter().any(|(name, receipt)| {
            name.starts_with("consumer.potential.")
                && receipt["connected"] == true
                && receipt["optimizer_active"] == true
                && receipt["l2"]
                    .as_f64()
                    .is_some_and(|v| v.is_finite() && v > 0.)
        }) {
            return Err(bad(
                "joint-potential initial admission has no finite nonzero potential credit",
            ));
        }
    }
    if a.read_state_bridge {
        let families = admission["gradient_families"]
            .as_object()
            .ok_or_else(|| bad("bridge admission inventory absent"))?;
        if !families.iter().any(|(name, r)| {
            name.starts_with("read_state_bridge.")
                && r["connected"] == true
                && r["optimizer_active"] == true
                && r["l2"].as_f64().is_some_and(|x| x.is_finite() && x > 0.)
        }) {
            return Err(bad(
                "zero bridge admission has no finite nonzero coefficient credit",
            ));
        }
    }
    write(a, "first-b8-admission.json", &admission)?;
    if a.mode == "admission" && a.read_state_bridge {
        write(
            a,
            "bridge-control-projection.json",
            &json!({"additional_passes":2,"representative_b8_indices":first,"measured_original_b8_seconds":admission["elapsed_seconds"],"additional_projected_seconds":admission["elapsed_seconds"].as_f64().ok_or_else(||bad("admission timing absent"))?*2.5,"policy":"estimated two same B8 forward/backward passes plus25percent reserve; healthy work not killed at estimate; no fit or optimizer updates","positive_control":"bias zero; strict native T[a,d]=7 iff a=d, else0; poststate equals selected source retained state","native_padded_bytes":lanes*7740}),
        )?;
        let control_bridge = source_identity_bridge(integer.binding(), lanes, &device)?;
        let mut results = Vec::new();
        let mut gradients = Vec::new();
        for selector_credit in [true, false] {
            let (grad, report) = batch(
                a,
                &source,
                &frozen,
                &generate,
                Some(&control_bridge),
                &cue,
                &prefix,
                &exp,
                &train,
                &first,
                &device,
                start,
                Some(&integer),
                a.token_backward_chunk,
                true,
                Some(BridgeAdmissionControl {
                    expect_selected_source_state: true,
                    selector_credit,
                }),
            )?;
            if !report["bridge_forward_receipts"]
                .as_array()
                .ok_or_else(|| bad("positive-control bridge receipts absent"))?
                .iter()
                .any(|r| r["identity_actions"] == false)
            {
                return Err(bad("actual positive-control bank has no nonidentity action; discriminator unavailable"));
            }
            results.push(report);
            gradients.push(grad);
        }
        if results[0]["native_forward_packets_sha256"]
            != results[1]["native_forward_packets_sha256"]
            || results[0]["native_equal_episode_ce"] != results[1]["native_equal_episode_ce"]
            || results[0]["native_weighted_training_objective"]
                != results[1]["native_weighted_training_objective"]
        {
            return Err(bad(
                "selector diagnostic changed actual native bank forward",
            ));
        }
        let contributions = selector_gradient_contributions(&gradients[0], &gradients[1], &device)?;
        write(
            a,
            "bridge-controlled-bank-admission.json",
            &json!({"selector_enabled":results[0],"selector_disabled":results[1],"contributions":contributions,"controlled_bridge_native_sha256":control_bridge.payload_sha256()?,"initial_fit_bridge_unchanged":true,"updates":0,"scope":"declared algebraic actual-bank positive control only; no training benefit/attention/language result"}),
        )?;
    }

    if a.mode == "admission" && a.token_backward_chunk == 2 {
        let (reference_grads, reference) = batch(
            a,
            &source,
            &frozen,
            &generate,
            bridge.as_ref(),
            &cue,
            &prefix,
            &exp,
            &train,
            &first,
            &device,
            start,
            Some(&integer),
            1,
            true,
            None,
        )?;
        if admission["native_equal_episode_ce"] != reference["native_equal_episode_ce"]
            || admission["target_positions"] != reference["target_positions"]
        {
            return Err(bad("chunk native objective differs"));
        }
        write(a, "chunk1-reference-admission.json", &reference)?;
        let parity = chunk_gradient_parity(&reference_grads, &admission_grads)?;
        write(
            a,
            "token-backward-chunk-parity.json",
            &json!({"parity":parity,"chunk2":admission,"chunk1":reference,"scope":"same parameters/data/indices, zero updates; both arms independently verify every native hard pool; no changed training dose; peak GPU memory measured externally"}),
        )?;
        if parity["status"] != "PASS" {
            return Err(bad(
                "token backward chunk gradient parity exceeded declared envelope",
            ));
        }
    }
    drop(admission_grads);
    let (base_native, base_gen, base_bridge, base_receipt) = checkpoint(
        a,
        0,
        &source,
        &frozen,
        &generate,
        bridge.as_ref(),
        &cue,
        &prefix,
    )?;
    let baseline = evaluate(
        a,
        "development-0000",
        &base_native,
        &base_gen,
        base_bridge.as_ref(),
        &exp,
        if a.mode == "admission" {
            &dev[..dev.len().min(8)]
        } else {
            &dev
        },
        &tok,
        &cue,
        &prefix,
        start,
    )?;
    let mut stages = vec![json!({"step":0,"evaluation":baseline,"checkpoint":base_receipt})];
    let mut selected = 0;
    let mut best = baseline.clone();
    let mut updates = Vec::new();
    if a.mode == "fit" {
        // Explicit fit mode is root's already authorized switch, not automatic
        // conversion of a successful zero-update admission into paid work.
        let minimum_context_margin = contextmargins
            .as_object()
            .ok_or_else(|| bad("context margins"))?
            .values()
            .filter_map(|v| v["minimum_quarter_crossing_shadow_distance"].as_f64())
            .fold(f64::INFINITY, f64::min);
        if (matches!(a.arm.as_str(), "joint" | "joint-potential")
            && a.context_learning_rate * 227.47318 <= minimum_context_margin)
            || 2. * a.prototype_learning_rate * 227.47318 <= 2.
        {
            return Err(bad("declared learning rates cannot reach an initial native context/prototype boundary within128steps"));
        }
        if let Some(margins) = &potentialmargins {
            let minimum = margins
                .as_object()
                .ok_or_else(|| bad("potential margins"))?
                .iter()
                .filter(|(name, _)| {
                    !matches!(
                        name.as_str(),
                        "consumer.potential.content_unary"
                            | "consumer.potential.content_radius"
                            | "consumer.potential.pair"
                    )
                })
                .filter_map(|(_, v)| v["minimum_quarter_crossing_shadow_distance"].as_f64())
                .fold(f64::INFINITY, f64::min);
            if a.potential_learning_rate
                .ok_or_else(|| bad("missing potential rate"))?
                * 227.47318
                <= minimum
            {
                return Err(bad("declared potential rate cannot reach any initially active native coefficient boundary within128steps"));
            }
        }
        let projection = admission["elapsed_seconds"]
            .as_f64()
            .ok_or_else(|| bad("B8 timing missing"))?
            * a.updates as f64
            * 1.25
            + start.elapsed().as_secs_f64() * 2.
            + 60.;
        write(
            a,
            "fit-resource-projection.json",
            &json!({"first_b8_seconds":admission["elapsed_seconds"],"updates":a.updates,"complete_projected_seconds":projection,"declared_maximum_seconds":a.maximum_seconds,"basis":"measured full native/forward/backward first B8, 25percent update reserve, two baseline-sized checkpoint/evaluation reserves+60seconds; optimizer and late ownprefix costs uncertain"}),
        )?;
        if projection > a.maximum_seconds as f64 {
            return Err(bad(
                "measured full fit projection exceeds declared budget before optimizer1",
            ));
        }
        let adam = |vars: &BTreeMap<String, Var>, lr: f64| -> Result<AdamW> {
            Ok(AdamW::new(
                vars.values().cloned().collect(),
                ParamsAdamW {
                    lr,
                    beta1: 0.9,
                    beta2: 0.999,
                    eps: 1e-8,
                    weight_decay: 0.,
                },
            )?)
        };
        let mut co = adam(&coefficients, a.learning_rate)?;
        let mut po = adam(&prototype, a.prototype_learning_rate)?;
        let mut cx = if matches!(a.arm.as_str(), "joint" | "joint-potential") {
            Some(adam(&context, a.context_learning_rate)?)
        } else {
            None
        };
        let mut cp = if a.arm == "joint-potential" {
            Some(adam(
                &potential,
                a.potential_learning_rate
                    .ok_or_else(|| bad("missing potential learning rate"))?,
            )?)
        } else {
            None
        };
        let mut bridge_optimizer = if a.read_state_bridge {
            Some(adam(&bridge_parameters, a.read_state_bridge_learning_rate)?)
        } else {
            None
        };
        for update in 0..a.updates {
            deadline(a, start)?;
            let indices = (0..8)
                .map(|i| (update * 8 + i) % train.len())
                .collect::<Vec<_>>();
            let (grads, receipt) = batch(
                a,
                &source,
                &frozen,
                &generate,
                bridge.as_ref(),
                &cue,
                &prefix,
                &exp,
                &train,
                &indices,
                &device,
                start,
                None,
                a.token_backward_chunk,
                false,
                None,
            )?;
            let squares = grads
                .values()
                .map(|g| g.sqr()?.sum_all())
                .collect::<candle_core::Result<Vec<_>>>()?;
            let norm = Tensor::stack(&squares, 0)?.sum_all()?.sqrt()?;
            let scalar_norm = norm.to_scalar::<f32>()?;
            if !scalar_norm.is_finite() {
                return Err(bad("nonfinite global gradient norm before clipping"));
            }
            let denominator = norm.clamp(1., 1e30)?;
            apply(&mut co, &coefficients, &grads, &denominator)?;
            apply(&mut po, &prototype, &grads, &denominator)?;
            if let Some(o) = cx.as_mut() {
                apply(o, &context, &grads, &denominator)?;
                for v in context.values() {
                    v.set(&v.as_tensor().clamp(-1.75, 1.75)?)?;
                }
            }
            if let Some(o) = cp.as_mut() {
                apply(o, &potential, &grads, &denominator)?;
                source.project_potential_range()?;
            }
            if let (Some(optimizer), Some(weights)) = (bridge_optimizer.as_mut(), bridge.as_ref()) {
                apply(optimizer, &bridge_parameters, &grads, &denominator)?;
                weights.project_coefficients()?;
            }
            generate.project_shadow_range()?;
            device.synchronize()?;
            updates.push(json!({"step":update+1,"before_update_batch":receipt}));
            write(a, "updates.json", &json!(updates))?;
            let step = update + 1;
            if step == a.updates {
                let (n, g, native_bridge, c) = checkpoint(
                    a,
                    step,
                    &source,
                    &frozen,
                    &generate,
                    bridge.as_ref(),
                    &cue,
                    &prefix,
                )?;
                let e = evaluate(
                    a,
                    &format!("development-{step:04}"),
                    &n,
                    &g,
                    native_bridge.as_ref(),
                    &exp,
                    &dev,
                    &tok,
                    &cue,
                    &prefix,
                    start,
                )?;
                write(
                    a,
                    "final-versus-initial.json",
                    &comparison(a, &e, &baseline)?,
                )?;
                if improved(&e, &best)? {
                    best = e.clone();
                    selected = step;
                }
                stages.push(json!({"step":step,"evaluation":e,"checkpoint":c}));
                write(
                    a,
                    "checkpoint-selection.json",
                    &json!({"stages":stages,"selected_step":selected,"primary":"complete ownprefix answer count","tie_only":"equalepisode native CE; earliest ties","fresh_predictions":0}),
                )?;
            }
        }
    }
    let mut freshresult = Value::Null;
    if a.mode == "fit" && a.fresh_inputs.is_some() {
        let fresh = load_panel(
            a.fresh_inputs.as_ref().ok_or_else(|| bad("fresh input"))?,
            a.fresh_labels.as_ref().ok_or_else(|| bad("fresh labels"))?,
            &integer,
            &tok,
            &legal,
            64,
        )?;
        let cp = a.out.join(format!("checkpoint-{selected:04}"));
        let r = read(&cp.join("receipt.json"))?;
        let b: NativeArtifactBinding = serde_json::from_value(r["parent"].clone())?;
        let n = IntegerRealizer::load_native(&cp.join("native"), &b)?;
        let g =
            NativeGeometricGenerate::from_bytes(&fs::read(cp.join("generate.bin"))?, n.binding())?;
        let native_bridge = if a.read_state_bridge {
            Some(NativeGeometricReadStateBridge::from_bytes(
                &fs::read(cp.join("read-state-bridge.bin"))?,
                n.binding(),
            )?)
        } else {
            None
        };
        let selectedfresh = evaluate(
            a,
            "fresh-selected",
            &n,
            &g,
            native_bridge.as_ref(),
            &exp,
            &fresh,
            &tok,
            &cue,
            &prefix,
            start,
        )?;
        let parentfresh = if selected == 0 {
            selectedfresh.clone()
        } else {
            evaluate(
                a,
                "fresh-parent",
                &base_native,
                &base_gen,
                base_bridge.as_ref(),
                &exp,
                &fresh,
                &tok,
                &cue,
                &prefix,
                start,
            )?
        };
        freshresult = json!({"selected":selectedfresh,"parent":parentfresh,"scope":"panel source-exposure provenance controls novelty; generated answer framing is not fresh source data"});
    }
    for (root, sha) in &before {
        report_output::verify(root)?;
        if sha256_file(&root.join("manifest.json"))? != *sha {
            return Err(bad("sealed input changed"));
        }
    }
    Ok(
        json!({"schema":"uor-r4.geometric-bank-generate-fit/1","status":"COMPLETED","mode":a.mode,"arm":a.arm,"seed":a.seed,"balanced_token_geometry":a.balanced_token_geometry,"phase_balanced_token_loss":a.phase_balanced_token_loss,"read_state_bridge_enabled":a.read_state_bridge,"read_state_bridge_learning_rate":a.read_state_bridge_learning_rate,"training_loss_weight_policy":loss_weight_policy(a.phase_balanced_token_loss),"token_geometry_initialization":if a.balanced_token_geometry {"balanced-a+j*b-with-existing-offset/1"}else{"legacy-base120-digits-with-existing-offset/1"},"source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT"),"executable_sha256":sha256_file(&std::env::current_exe()?)?,"report_scope":"initial/final checkpoints only; native full512 ownprefix selection; admission8subset explicitly not full512 quality verdict","updates":if a.mode=="fit"{a.updates}else{0},"selected_step":selected,"stages":stages,"fresh":freshresult,"elapsed_seconds":start.elapsed().as_secs_f64(),"wall_time_estimate_seconds":a.maximum_seconds,"wall_time_estimate_exceeded":start.elapsed().as_secs_f64()>=a.maximum_seconds as f64,"runtime_scope":"actual all-source Copy plus full legal native geometric Generate; integer CPU oracle/serving and CUDA learning; no legacy terminal mass","language_scope":"bounded framed grounded answers; not general chat/reasoning; no-source path available but not qualified if panel lacks such rows"}),
    )
}
fn main() -> Result<()> {
    let a = args()?;
    report_output::claim(&a.out)?;
    let start = Instant::now();
    let result = run(&a, start);
    let report = match &result {
        Ok(v) => v.clone(),
        Err(e) => {
            json!({"schema":"uor-r4.geometric-bank-generate-fit/1","status":"FAILED","error":e.to_string(),"elapsed_seconds":start.elapsed().as_secs_f64(),"wall_time_estimate_seconds":a.maximum_seconds,"wall_time_estimate_exceeded":start.elapsed().as_secs_f64()>=a.maximum_seconds as f64,"model_verdict":"UNQUALIFIED; preserve completed rows/checkpoints; unfinished fit is not model failure"})
        }
    };
    write(&a, "report.json", &report)?;
    report_output::seal(&a.out)?;
    report_output::verify(&a.out)?;
    result.map(|_| ())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn selector_gradient_difference_reports_zero_without_claiming_escape() -> Result<()> {
        let name = "consumer.potential.context_presence".to_owned();
        let tensor = Tensor::from_vec(vec![1f32, -1.], 2, &Device::Cpu)?;
        let on = BTreeMap::from([(name.clone(), tensor.clone())]);
        let off = BTreeMap::from([(name.clone(), tensor)]);
        let equal = selector_gradient_contributions(&on, &off, &Device::Cpu)?;
        assert_eq!(equal["status"], json!("NO_SELECTOR_ESCAPE_AT_CONTROL"));
        assert_eq!(equal["families"][&name]["selector_delta_l2"], json!(0.));
        let changed = BTreeMap::from([(
            name.clone(),
            Tensor::from_vec(vec![2f32, -2.], 2, &Device::Cpu)?,
        )]);
        let result = selector_gradient_contributions(&changed, &off, &Device::Cpu)?;
        assert_eq!(result["status"], json!("NONZERO_SELECTOR_POTENTIAL_CREDIT"));
        assert!(result["families"][&name]["selector_delta_l2"]
            .as_f64()
            .is_some_and(|x| x > 1.4 && x < 1.5));
        Ok(())
    }
    #[test]
    fn bridge_raw_read_selects_first_physical_occurrence_without_token_aliasing() -> Result<()> {
        assert_eq!(earliest_raw_copy_max(&[4, 4, 3])?, 0);
        assert_eq!(earliest_raw_copy_max(&[-9, -4, -4])?, 1);
        assert_eq!(earliest_raw_copy_max(&[i64::MIN, i64::MAX])?, 1);
        assert!(earliest_raw_copy_max(&[]).is_err());
        Ok(())
    }
    #[test]
    fn wall_estimate_overrun_continues_and_preserves_first_receipt() -> Result<()> {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|_| bad("test clock predates epoch"))?
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "uor-wall-estimate-test-{}-{nonce}",
            std::process::id()
        ));
        let a: Args = serde_json::from_value(json!({
            "mode":"admission", "arm":"joint-potential", "seed":1,
            "source_weights":".", "native_artifact":".", "trusted_binding":".",
            "cue_bundle":".", "prefix_bundle":".", "canonical_exp":".",
            "training_inputs":".", "training_labels":".",
            "development_inputs":".", "development_labels":".",
            "updates":0, "learning_rate":0.003, "prototype_learning_rate":0.01,
            "context_learning_rate":0.002, "maximum_seconds":1,
            "maximum_report_bytes":2_097_152, "out":root
        }))?;
        report_output::claim(&a.out)?;
        let start = Instant::now()
            .checked_sub(std::time::Duration::from_secs(2))
            .ok_or_else(|| bad("test Instant cannot subtract two seconds"))?;
        deadline(&a, start)?;
        let path = a.out.join("wall-estimate-overrun.json");
        let first = fs::read(&path)?;
        let receipt: Value = serde_json::from_slice(&first)?;
        assert_eq!(receipt["declared_estimate_seconds"], json!(1));
        assert!(
            receipt["first_observed_elapsed_seconds"]
                .as_f64()
                .ok_or_else(|| bad("overrun elapsed absent"))?
                >= 2.
        );
        deadline(&a, start)?;
        assert_eq!(fs::read(&path)?, first);
        fs::remove_dir_all(&a.out)?;
        Ok(())
    }
    #[test]
    fn accumulated_nonfinite_credit_is_rejected_before_optimizer_input() -> Result<()> {
        let var = Var::new(1f32, &Device::Cpu)?;
        let params = BTreeMap::from([(
            "consumer.potential.context_presence".to_owned(),
            var.clone(),
        )]);
        let loss = var.as_tensor().affine(f64::NAN, 0.)?;
        let mut sums = BTreeMap::new();
        assert!(accumulate_backward(loss, &params, &mut sums, &Device::Cpu, false).is_err());
        assert!(sums.is_empty());
        assert_eq!(var.to_scalar::<f32>()?, 1.);
        Ok(())
    }

    #[test]
    fn chunk_gradient_parity_rejects_changed_gradient() -> Result<()> {
        let reference =
            BTreeMap::from([("x".to_owned(), Tensor::new(&[0.01f32, 0.02], &Device::Cpu)?)]);
        let changed =
            BTreeMap::from([("x".to_owned(), Tensor::new(&[0.01f32, 0.04], &Device::Cpu)?)]);
        assert_eq!(
            chunk_gradient_parity(&reference, &changed)?["status"],
            "FAIL"
        );
        Ok(())
    }
    #[test]
    fn first_context_receipt_filters_decoder_and_preserves_accumulation() -> Result<()> {
        let context = Var::new(2f32, &Device::Cpu)?;
        let decoder = Var::new(3f32, &Device::Cpu)?;
        let absent = Var::new(1f32, &Device::Cpu)?;
        let params = BTreeMap::from([
            ("consumer.context.token_transition".into(), context.clone()),
            ("consumer.context.self_root".into(), absent),
            ("generate.unary".into(), decoder.clone()),
        ]);
        let loss = (context.as_tensor().sqr()? + decoder.as_tensor().sqr()?)?;
        let mut measured = BTreeMap::new();
        let receipt =
            accumulate_backward(loss.clone(), &params, &mut measured, &Device::Cpu, true)?
                .ok_or_else(|| bad("missing diagnostic"))?;
        let mut plain = BTreeMap::new();
        assert!(accumulate_backward(loss, &params, &mut plain, &Device::Cpu, false)?.is_none());
        assert_eq!(
            receipt
                .as_object()
                .ok_or_else(|| bad("receipt object"))?
                .len(),
            2
        );
        assert_eq!(receipt["consumer.context.token_transition"]["l2"], 4.);
        assert_eq!(receipt["consumer.context.self_root"]["connected"], false);
        assert!(receipt.get("generate.unary").is_none());
        for (name, g) in measured {
            assert_eq!(g.to_scalar::<f32>()?, plain[&name].to_scalar::<f32>()?);
        }
        Ok(())
    }
    #[test]
    fn weighted_chunk2_shared_graph_matches_chunk1_vjp() -> Result<()> {
        let v = Var::new(0.7f32, &Device::Cpu)?;
        let shared = v.as_tensor().sqr()?;
        let params = BTreeMap::from([("shared".to_owned(), v)]);
        let losses = [
            shared.affine(1. / 6., 0.)?,
            shared.sqr()?.affine(1. / 8., 0.)?,
            shared.affine(1. / 10., 0.)?,
        ];
        let mut reference = BTreeMap::new();
        for loss in &losses {
            accumulate_backward(loss.clone(), &params, &mut reference, &Device::Cpu, false)?;
        }
        let mut chunked = BTreeMap::new();
        accumulate_backward(
            (&losses[0] + &losses[1])?,
            &params,
            &mut chunked,
            &Device::Cpu,
            false,
        )?;
        accumulate_backward(
            losses[2].clone(),
            &params,
            &mut chunked,
            &Device::Cpu,
            false,
        )?;
        assert_eq!(
            chunk_gradient_parity(&reference, &chunked)?["status"],
            "PASS"
        );
        let expected = 2. * 0.7 * (1. / 6. + 1. / 10.) + 4. * 0.7f32.powi(3) / 8.;
        assert!((chunked["shared"].to_scalar::<f32>()? - expected).abs() < 1e-6);
        Ok(())
    }
    #[test]
    fn complete_answer_primary_over_ce() {
        let old = json!({"complete":1,"native_equal_episode_ce":1.});
        assert!(improved(&json!({"complete":2,"native_equal_episode_ce":9.}), &old).unwrap());
        assert!(!improved(&json!({"complete":1,"native_equal_episode_ce":1.}), &old).unwrap());
    }
    #[test]
    fn no_source_preserves_actual_history() {
        let e = Episode {
            packet: Packet {
                id: "n".into(),
                segments: vec![Segment::Context {
                    event: 1,
                    role: 1,
                    token_ids: vec![4, 5],
                }],
                query_ids: vec![6],
                actual_prefix_ids: vec![],
            },
            answers: FrozenAnswers {
                accepted: vec!["a.".into()],
                intent: uor_r4_core::answer_oracle::RecordedValueIntent::Current,
            },
            target: vec![7, 1],
            views: vec![None],
        };
        assert_eq!(e.causal_no_source(&[8]).unwrap(), vec![4, 5, 6, 8]);
    }
    #[test]
    fn phase_weights_partition_and_normalize_with_exact_legacy_default() -> Result<()> {
        let copy = BTreeSet::from([4, 5]);
        let target = [4, 4, 5, 7, 7, 1];
        let p = episode_loss_weights(&target, &copy, 8, true)?;
        assert_eq!(p.counts, [1, 2, 3]);
        assert_eq!(p.phases, [0, 1, 1, 2, 2, 2]);
        for phase in 0..3 {
            let weight = p
                .weights
                .iter()
                .zip(&p.phases)
                .filter(|(_, k)| **k == phase)
                .map(|(w, _)| *w)
                .sum::<f64>();
            assert!((weight - 1. / 24.).abs() < 1e-15);
        }
        assert!((p.weights.iter().sum::<f64>() - 1. / 8.).abs() < 1e-15);
        let old = episode_loss_weights(&target, &copy, 8, false)?;
        for w in old.weights {
            assert_eq!(w.to_bits(), (1. / target.len() as f64 / 8f64).to_bits());
        }
        Ok(())
    }
    #[test]
    fn phase_weights_empty_strata_singleton_and_no_source_are_explicit() -> Result<()> {
        for (target, copy, expected) in [
            (vec![4], BTreeSet::from([4]), [1, 0, 0]),
            (vec![4, 4, 5], BTreeSet::from([4, 5]), [1, 2, 0]),
            (vec![7, 7, 1], BTreeSet::new(), [1, 0, 2]),
        ] {
            let p = episode_loss_weights(&target, &copy, 2, true)?;
            assert_eq!(p.counts, expected);
            assert!(p.weights.iter().all(|w| w.is_finite() && *w > 0.));
            assert!((p.weights.iter().sum::<f64>() - 0.5).abs() < 1e-15);
        }
        assert!(episode_loss_weights(&[], &BTreeSet::new(), 1, true).is_err());
        assert!(episode_loss_weights(&[1], &BTreeSet::new(), 0, true).is_err());
        Ok(())
    }
}
