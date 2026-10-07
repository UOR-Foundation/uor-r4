//! RECORD: shared-parameter native-loss direction packet, not a fit.
//! Authenticated retained loaders/optimizer/export helpers from7fbfc940; fixed RawIdentity credit in both arms.
//! Targets enter only the token-alias objective; native source selection stays target-free.
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
    geometric_generate_learning::{GenerateLearningWeights, VocabularyScoreAdjoint},
    geometric_occurrence_consumer::{
        source_realizer::{NativeSourceRealizer, SourceRealizerWeights},
        ConsumerIdentity,
    },
    geometric_read_state_bridge::{BridgeLearningWeights, PreparedCategoricalBridge},
    sha256_bytes, sha256_file,
};
#[path = "../../uor-r4-integer/examples/support/source_probe.rs"]
mod output_support;
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum Mode {
    Admission,
    Fit,
}
#[derive(Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum Credit {
    Clipped,
    RawIdentity,
}
impl Credit {
    fn policy(self) -> VocabularyScoreAdjoint {
        match self {
            Self::Clipped => VocabularyScoreAdjoint::Clipped,
            Self::RawIdentity => VocabularyScoreAdjoint::RawIdentity,
        }
    }
    fn name(self) -> &'static str {
        match self {
            Self::Clipped => "clipped",
            Self::RawIdentity => "raw_identity",
        }
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Args {
    mode: Mode,
    credit: Credit,
    seed: u64,
    checkpoint: PathBuf,
    saved_fit: PathBuf,
    categorical: PathBuf,
    parent_config: PathBuf,
    training_inputs: PathBuf,
    training_labels: PathBuf,
    development_inputs: PathBuf,
    development_labels: PathBuf,
    maximum_seconds: u64,
    maximum_report_bytes: u64,
    out: PathBuf,
    #[serde(default)]
    baseline: Option<PathBuf>,
}
const INPUT_SHA: &str = "b9661606b280884217a64e0a5b643f8324a90390e47ade7241da0889a5f7c86a";
const LABEL_SHA: &str = "84991e0657b5697c0e061eaa3fe86e4a0ec7ce6bc2be8371b62698c6b8126155";
const CP_RECEIPT_SHA: &str = "9196520209cb8ed172fea65a60be0a1f2fa788e40d5857c65bf93eccb658b816";
const CAT_SHA: &str = "af5039e74c6cd92e2fa0bf4c72c315ef34f912d77b09fc607215a18f86cd1d0e";
const QUARTER_SHA: &str = "d7e56ed124ad80619dad5b6bfb90dec5f78fe855f43ef4bfd9011e2e31f5fccf";
const GENERATE_SHA: &str = "41c7beae25e98fdc658ff667959a7c02a6b916b04298a425f7f7f48285d21511";
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
    let path = it.next().ok_or_else(|| bad("one JSON config required"))?;
    if it.next().is_some() {
        return Err(bad("one JSON config only"));
    }
    let a: Args = serde_json::from_slice(&fs::read(path)?)?;
    if ![1001, 1002, 1003].contains(&a.seed)
        || a.maximum_seconds == 0
        || a.maximum_report_bytes < (64 << 20)
        || a.maximum_report_bytes > (2 << 30)
        || (a.baseline.is_some() && a.mode != Mode::Fit)
    {
        return Err(bad("fixed order seeds/resource admission"));
    }
    let output = output_support::prospective_output(&a.out)?;
    for path in [
        &a.checkpoint,
        &a.saved_fit,
        &a.categorical,
        &a.parent_config,
        &a.training_inputs,
        &a.training_labels,
        &a.development_inputs,
        &a.development_labels,
    ]
    .into_iter()
    .chain(a.baseline.iter())
    {
        let input = fs::canonicalize(path)?;
        if output.starts_with(&input) || input.starts_with(&output) {
            return Err(bad("output/input overlap"));
        }
        for ancestor in input.ancestors() {
            if ancestor.join("manifest.json").exists() && output.starts_with(ancestor) {
                return Err(bad("output beneath input seal"));
            }
        }
    }
    Ok(a)
}
#[cfg(feature = "cuda")]
fn cuda() -> Result<Device> {
    Ok(Device::new_cuda(0)?)
}
#[cfg(not(feature = "cuda"))]
fn cuda() -> Result<Device> {
    Err(bad("CUDA feature/device required; no CPU fallback"))
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
fn restore(
    root: &Path,
    inventory: &Value,
    vars: &BTreeMap<String, Var>,
    device: &Device,
) -> Result<()> {
    let map = inventory
        .as_object()
        .ok_or_else(|| bad("master inventory absent"))?;
    if map.keys().collect::<BTreeSet<_>>() != vars.keys().collect::<BTreeSet<_>>() {
        return Err(bad("master parameter inventory differs"));
    }
    for (name, var) in vars {
        if !name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '_')
        {
            return Err(bad("unsafe master name"));
        }
        let m = &map[name];
        let shape: Vec<usize> = serde_json::from_value(m["shape"].clone())?;
        let bytes = fs::read(root.join(format!("{name}.f32le")))?;
        if shape != var.dims()
            || bytes.len()
                != var
                    .elem_count()
                    .checked_mul(4)
                    .ok_or_else(|| bad("master size overflow"))?
            || m["bytes"].as_u64() != Some(bytes.len() as u64)
            || m["sha256"].as_str() != Some(sha256_bytes(&bytes).as_str())
        {
            return Err(bad("master shape/hash/bytes differ"));
        }
        let values = bytes
            .chunks_exact(4)
            .map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]]))
            .collect::<Vec<_>>();
        if values
            .iter()
            .any(|x| !x.is_finite() || (!name.ends_with("prototype_choices") && x.abs() > 1.75))
        {
            return Err(bad("master finite/range admission"));
        }
        var.set(&Tensor::from_vec(values, shape.as_slice(), device)?)?;
    }
    Ok(())
}
fn identities(vars: &BTreeMap<String, Var>) -> Result<BTreeMap<String, String>> {
    vars.iter()
        .map(|(n, v)| {
            let b = v
                .flatten_all()?
                .to_vec1::<f32>()?
                .into_iter()
                .flat_map(f32::to_le_bytes)
                .collect::<Vec<_>>();
            Ok((n.clone(), sha256_bytes(&b)))
        })
        .collect()
}
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

struct Loaded {
    source: SourceRealizerWeights,
    frozen: NativeSourceRealizer,
    integer: IntegerRealizer,
    tokenizer: ByteBpeTokenizer,
    generate: GenerateLearningWeights,
    original_bridge: BridgeLearningWeights,
    marker: BridgeLearningWeights,
    cue: CueAngularQ4,
    prefix: PrefixAngularQ4,
    exp: Vec<u8>,
    receipt: Value,
    categorical_receipt: Value,
}
fn authenticate(a: &Args) -> Result<()> {
    report_output::verify(&a.saved_fit)?;
    report_output::verify(&a.categorical)?;
    for p in [
        &a.training_inputs,
        &a.training_labels,
        &a.development_inputs,
        &a.development_labels,
    ] {
        report_output::verify(&seal_for(p)?)?;
    }
    let fit = read(&a.saved_fit.join("report.json"))?;
    let receipt = read(&a.checkpoint.join("receipt.json"))?;
    if fit["schema"] != "uor-r4.geometric-bank-generate-fit/1"
        || fit["status"] != "COMPLETED"
        || fit["mode"] != "fit"
        || fit["arm"] != "joint-potential"
        || fit["updates"] != 128
        || fs::canonicalize(a.saved_fit.join("checkpoint-0128"))?
            != fs::canonicalize(&a.checkpoint)?
        || sha256_file(&a.checkpoint.join("receipt.json"))? != CP_RECEIPT_SHA
        || fit["stages"]
            .as_array()
            .and_then(|v| v.last())
            .map(|v| &v["checkpoint"])
            != Some(&receipt)
    {
        return Err(bad("exact retained final checkpoint admission"));
    }
    for (p, expected) in [
        (&a.training_inputs, INPUT_SHA),
        (&a.development_inputs, INPUT_SHA),
        (&a.training_labels, LABEL_SHA),
        (&a.development_labels, LABEL_SHA),
    ] {
        if sha256_file(p)? != expected {
            return Err(bad("fixed 512-row panel hash differs"));
        }
    }
    let parent = read(&a.parent_config)?;
    for (key, expected) in [
        ("updates", json!(128)),
        ("learning_rate", json!(0.003)),
        ("prototype_learning_rate", json!(0.01)),
        ("context_learning_rate", json!(0.002)),
        ("potential_learning_rate", json!(0.003)),
        ("token_backward_chunk", json!(1)),
        ("phase_balanced_token_loss", json!(true)),
        ("prefix_temporal_utility", json!(false)),
        ("arm", json!("joint-potential")),
    ] {
        if parent[key] != expected {
            return Err(bad("retained fit configuration differs"));
        }
    }
    // Parent-config absolute paths are historical. Data/artifact identities above,
    // not guessed replacement ancestry, determine admission.
    Ok(())
}
fn load(a: &Args, d: &Device) -> Result<Loaded> {
    authenticate(a)?;
    let cp = &a.checkpoint;
    let receipt = read(&cp.join("receipt.json"))?;
    let binding: NativeArtifactBinding = serde_json::from_value(receipt["parent"].clone())?;
    let integer = IntegerRealizer::load_native(&cp.join("native"), &binding)?;
    let tokbytes = fs::read(cp.join("native/tokenizer.json"))?;
    let tokenizer = ByteBpeTokenizer::from_tokenizer_json_bytes(&tokbytes)
        .ok_or_else(|| bad("ByteBPE unavailable"))?;
    let source =
        SourceRealizerWeights::load_context_potential_on_device(&cp.join("source"), &tokbytes, d)?;
    let meta = read(&cp.join("native/metadata.json"))?;
    let identity: ConsumerIdentity = serde_json::from_value(meta["identity"].clone())?;
    let frozen = NativeSourceRealizer::load(&cp.join("native"), &source, &identity)?;
    if frozen.artifact_binding()? != binding {
        return Err(bad("source/native binding differs"));
    }
    let genbytes = fs::read(cp.join("generate.bin"))?;
    if sha256_bytes(&genbytes) != GENERATE_SHA {
        return Err(bad("Generate donor identity"));
    }
    let gn = NativeGeometricGenerate::from_bytes(&genbytes, integer.binding())?;
    let generate = GenerateLearningWeights::from_native(integer.binding().clone(), &gn, d)?;
    let gm = read(&cp.join("generate-source/metadata.json"))?;
    if gm["tokenizer_sha256"] != sha256_bytes(&tokbytes)
        || gm["protocol"] != serde_json::to_value(integer.binding().protocol())?
        || gm["lanes"] != json!(generate.lanes())
    {
        return Err(bad("Generate masters binding"));
    }
    restore(
        &cp.join("generate-source"),
        &gm["parameters"],
        &generate.parameters(),
        d,
    )?;
    if generate.export_native()?.to_bytes()? != genbytes {
        return Err(bad("restored Generate export differs"));
    }
    let bbytes = fs::read(cp.join("read-state-bridge.bin"))?;
    if sha256_bytes(&bbytes) != QUARTER_SHA {
        return Err(bad("quarter bridge donor identity"));
    }
    let bn = NativeGeometricReadStateBridge::from_bytes(&bbytes, integer.binding())?;
    let original_bridge = BridgeLearningWeights::from_native(&bn, integer.binding(), d)?;
    let bmbytes = fs::read(cp.join("read-state-bridge-source/metadata.json"))?;
    let bm: Value = serde_json::from_slice(&bmbytes)?;
    if bm["metadata"] != serde_json::to_value(bn.metadata())? || bm != receipt["read_state_bridge"]
    {
        return Err(bad("bridge donor master receipt differs"));
    }
    restore(
        &cp.join("read-state-bridge-source"),
        &bm["source_parameters"],
        &original_bridge.parameters(),
        d,
    )?;
    if original_bridge.export_native()?.to_bytes()? != bbytes {
        return Err(bad("original quarter export differs"));
    }
    let rebuilt = original_bridge.export_categorical_actions()?;
    let cb = fs::read(a.categorical.join("read-state-bridge-categorical.bin"))?;
    let probe = read(&a.categorical.join("probe.json"))?;
    if sha256_bytes(&cb) != CAT_SHA
        || rebuilt.native.to_bytes()? != cb
        || probe["status"] != "COMPLETED"
        || probe["mode"] != "categorical"
        || probe["schema"] != "uor-r4.read-state-export-probe/1"
        || probe["checkpoint_receipt_sha256"] != CP_RECEIPT_SHA
        || probe["parent_manifest_sha256"] != sha256_file(&a.saved_fit.join("manifest.json"))?
        || probe["master_metadata_sha256"] != sha256_bytes(&bmbytes)
        || probe["master_identities"] != bm["source_parameters"]
        || probe["original_native_sha256"] != QUARTER_SHA
        || probe["derived_native_sha256"] != CAT_SHA
        || probe["derived_payload_sha256"] != rebuilt.native.metadata().payload_sha256
        || probe["original_coarse_reproduction"] != "BYTE_IDENTICAL"
        || probe["parent_binding"] != receipt["parent"]
        || probe["all_query_frame_mismatches"] != 0
        || probe["all_query_frame_checks"] != json!(generate.lanes() * 120 * 120)
    {
        return Err(bad(
            "categorical artifact does not bind exact retained masters",
        ));
    }
    // This separate marker object supplies a truthful quarter graph for its
    // categorical native energies. Its Vars are never in an optimizer or clip.
    let marker = BridgeLearningWeights::from_native(&rebuilt.native, integer.binding(), d)?;
    if marker.prepare_native()?.native.to_bytes()? != cb {
        return Err(bad("frozen marker/native energy snapshot differs"));
    }
    let cue = cue_payload(&cp.join("cue"))?;
    let prefix = prefix_payload(&cp.join("prefix"))?;
    let cc = frozen.compile_cue_carrier(cue.clone())?;
    let pp = frozen.compile_prefix_transport(&cc, prefix_clone(&prefix)?)?;
    if serde_json::to_value(cc.metadata())? != read(&cp.join("cue/native-metadata.json"))?
        || serde_json::to_value(pp.metadata())? != read(&cp.join("prefix/native-metadata.json"))?
    {
        return Err(bad("donor cue/prefix receipts differ"));
    }
    let exp = fs::read(cp.join("native/consumer/exp-q31.bin"))?;
    Ok(Loaded {
        source,
        frozen,
        integer,
        tokenizer,
        generate,
        original_bridge,
        marker,
        cue,
        prefix,
        exp,
        receipt,
        categorical_receipt: serde_json::to_value(rebuilt.receipt)?,
    })
}
fn active(
    source: &SourceRealizerWeights,
    g: &GenerateLearningWeights,
) -> Result<BTreeMap<String, Var>> {
    let mut vars = g.parameters();
    vars.extend(source.context_state_parameters());
    vars.extend(source.potential_parameters());
    if vars.keys().any(|n| n.starts_with("read_state_bridge.")) {
        return Err(bad("frozen bridge admitted to optimizer"));
    }
    Ok(vars)
}
fn batch(
    a: &Args,
    l: &Loaded,
    eps: &[Episode],
    indices: &[usize],
    d: &Device,
    start: Instant,
    independent: Option<&IntegerRealizer>,
    pullback: Pullback,
) -> Result<(BTreeMap<String, Tensor>, Value)> {
    let current = l.source.compile_context_potential_rebound(&l.frozen)?;
    let prepared = l.source.prepare_context_potential_on_device(&current, d)?;
    let cue = current.compile_cue_carrier(l.cue.clone())?;
    let prefix = current.compile_prefix_transport(&cue, prefix_clone(&l.prefix)?)?;
    let gs = l.generate.prepare_native()?;
    let bs = l.marker.prepare_native()?;
    if sha256_bytes(&bs.native.to_bytes()?) != CAT_SHA {
        return Err(bad("frozen categorical map changed"));
    }
    let categorical_bytes = bs.native.to_bytes()?;
    let categorical = PreparedCategoricalBridge::from_bytes(
        &categorical_bytes,
        l.generate.binding(),
        CAT_SHA,
        d,
    )?;
    let learner = PreparedBankGenerate::new(&prepared, &l.generate, &gs, &l.exp)?
        .with_prefix_temporal_utility(false);
    let mut learner = match pullback {
        Pullback::Legacy => learner.with_read_state_bridge(&l.marker, &bs)?,
        Pullback::Categorical => learner.with_categorical_read_state_bridge(&categorical)?,
    }
    .with_read_selector_credit(true);
    let params = active(&l.source, &l.generate)?;
    let mut sums = BTreeMap::<String, Tensor>::new();
    let mut rows = Vec::new();
    let mut total = 0.;
    let mut positions = 0;
    let mut phase_positions = [0usize; 3];
    let mut phase_losses = [0.; 3];
    let mut pool = NativeVocabularyActions::new(l.generate.binding().clone(), &l.exp)?;
    for &index in indices {
        let e = eps.get(index).ok_or_else(|| bad("batch index absent"))?;
        let expected_union = e
            .views
            .iter()
            .flatten()
            .flat_map(|v| v.emitted_token_ids().iter().copied())
            .collect::<BTreeSet<_>>();
        let mut plan = None;
        let mut ep_loss = 0.;
        for (t, &target) in e.target.iter().enumerate() {
            deadline(a, start)?;
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
            let union = out.copy_token_ids.iter().copied().collect::<BTreeSet<_>>();
            if union != expected_union {
                return Err(bad("actual all-source Copy union changed"));
            }
            if let Some(model) = independent {
                let expected = native_step(
                    model,
                    &gs.native,
                    Some(&bs.native),
                    &mut pool,
                    e,
                    &e.target[..t],
                    &l.cue,
                    &l.prefix,
                )?;
                let masses = out
                    .actions
                    .token_masses
                    .iter()
                    .map(|m| {
                        [
                            m.token_id as u64,
                            m.weight_q31,
                            m.generate_weight_q31,
                            m.copy_weight_q31,
                        ]
                    })
                    .collect::<Vec<_>>();
                if expected["pool"]["summary"] != serde_json::to_value(&out.actions.summary)?
                    || expected["pool"]["token_masses"] != json!(masses)
                    || expected["copy_token_ids"] != json!(out.copy_token_ids)
                    || expected["copy_raw_scores_q24"] != json!(out.copy_scores_q24)
                    || expected["retained_state_codes"]
                        != json!(out
                            .final_state_codes
                            .iter()
                            .map(|c| c.index())
                            .collect::<Vec<_>>())
                    || expected["generate_raw_scores_sha256"]
                        != sha256_bytes(&serde_json::to_vec(&out.generate.scores_q24)?)
                {
                    return Err(bad(
                        "independent native categorical bank/Generate/pool parity differs",
                    ));
                }
                let old = out
                    .loss_with_credit(target, VocabularyScoreAdjoint::Clipped)?
                    .to_scalar::<f32>()?;
                let raw = out
                    .loss_with_credit(target, VocabularyScoreAdjoint::RawIdentity)?
                    .to_scalar::<f32>()?;
                if old.to_bits() != raw.to_bits() || !old.is_finite() {
                    return Err(bad("matched credits have different forward losses"));
                }
                let (_, bridge) = out
                    .read_state_bridge
                    .as_ref()
                    .ok_or_else(|| bad("categorical bridge absent"))?;
                let nb = &expected["source_provenance"]["read_state_bridge"];
                if nb["selected_ordinal"] != json!(out.read_state_bridge.as_ref().map(|v| v.0))
                    || nb["action_codes"]
                        != json!(bridge
                            .action_codes
                            .iter()
                            .map(|c| c.index())
                            .collect::<Vec<_>>())
                    || nb["action_scores_q24"] != json!(bridge.action_scores_q24)
                {
                    return Err(bad(
                        "independent categorical selected/action parity differs",
                    ));
                }
            }
            // Labels enter only after the complete target-free native pool.
            if plan.is_none() {
                plan = Some(episode_loss_weights(
                    &e.target,
                    &union,
                    indices.len(),
                    true,
                )?);
            }
            let plan = plan.as_ref().ok_or_else(|| bad("loss phases absent"))?;
            let weight = plan.weights[t];
            let phase = plan.phases[t];
            let mass = out
                .actions
                .token_masses
                .iter()
                .find(|m| m.token_id == target)
                .ok_or_else(|| bad("target support absent"))?
                .weight_q31;
            let nll = -(mass as f64 / out.actions.summary.total_weight_q31 as f64).ln();
            if !nll.is_finite() {
                return Err(bad("nonfinite native token objective"));
            }
            total += nll * weight;
            ep_loss += nll * weight;
            positions += 1;
            phase_positions[phase] += 1;
            phase_losses[phase] += nll * weight;
            let grads = out
                .loss_with_credit(target, a.credit.policy())?
                .affine(weight, 0.)?
                .backward()?;
            // One token graph at a time; no detached context carrier is inserted.
            for (name, var) in &params {
                if let Some(g) = grads.get(var.as_tensor()) {
                    if !g.device().same_device(d)
                        || !g
                            .abs()?
                            .flatten_all()?
                            .max(0)?
                            .to_scalar::<f32>()?
                            .is_finite()
                    {
                        return Err(bad("gradient device/finite admission"));
                    }
                    let sum = if let Some(old) = sums.get(name) {
                        old.add(g)?.detach()
                    } else {
                        g.detach()
                    };
                    sums.insert(name.clone(), sum);
                }
            }
        }
        rows.push(
            json!({"id":e.packet.id,"index":index,"weighted_native_loss":ep_loss,
            "phase_counts":plan.map(|p|p.counts)}),
        );
    }
    Ok((
        sums,
        json!({"indices":indices,"rows":rows,"positions":positions,"weighted_native_loss":total,
        "phase_positions":phase_positions,"phase_losses":phase_losses,
        "independent_native_parity":independent.is_some(),
        "matched_forward_loss_bit_equal":independent.is_some(),
        "credit":a.credit.name(),"token_backward_chunk":1,"frozen_bridge_excluded_from_gradient_accumulation":true}),
    ))
}
fn save_masters(root: &Path, vars: &BTreeMap<String, Var>) -> Result<Value> {
    fs::create_dir(root)?;
    let mut inventory = BTreeMap::new();
    for (name, var) in vars {
        let values = var.flatten_all()?.to_vec1::<f32>()?;
        if values.iter().any(|v| !v.is_finite()) {
            return Err(bad("nonfinite checkpoint master"));
        }
        let bytes = values
            .into_iter()
            .flat_map(f32::to_le_bytes)
            .collect::<Vec<_>>();
        fs::write(root.join(format!("{name}.f32le")), &bytes)?;
        inventory.insert(
            name,
            json!({"shape":var.dims(),"bytes":bytes.len(),"sha256":sha256_bytes(&bytes)}),
        );
    }
    Ok(json!(inventory))
}
fn copy_directory(from: &Path, to: &Path) -> Result<()> {
    fs::create_dir(to)?;
    for entry in fs::read_dir(from)? {
        let entry = entry?;
        if !entry.file_type()?.is_file() {
            return Err(bad("unexpected donor master subdirectory"));
        }
        fs::copy(entry.path(), to.join(entry.file_name()))?;
    }
    Ok(())
}
fn checkpoint(
    a: &Args,
    step: usize,
    l: &Loaded,
) -> Result<(
    IntegerRealizer,
    NativeGeometricGenerate,
    NativeGeometricReadStateBridge,
    Value,
)> {
    let estimated = size(&a.checkpoint.join("source"))?
        + size(&a.checkpoint.join("native"))?
        + l.generate
            .parameters()
            .values()
            .map(|v| v.elem_count() as u64 * 4)
            .sum::<u64>()
        + 4_194_304;
    if size(&a.out)? + estimated > a.maximum_report_bytes - 1_048_576 {
        return Err(bad("checkpoint storage admission"));
    }
    let root = a.out.join(format!("checkpoint-{step:04}"));
    fs::create_dir(&root)?;
    let native = l.source.compile_context_potential_rebound(&l.frozen)?;
    l.source.save_source(&root.join("source"))?;
    native.save(&root.join("native"))?;
    let binding = native.execution_binding()?;
    let integer = IntegerRealizer::load_native(&root.join("native"), &binding)?;
    let tokbytes = fs::read(root.join("native/tokenizer.json"))?;
    let disk_source = SourceRealizerWeights::load_context_potential_on_device(
        &root.join("source"),
        &tokbytes,
        &Device::Cpu,
    )?;
    let disk_native = NativeSourceRealizer::load(
        &root.join("native"),
        &disk_source,
        &serde_json::from_value::<ConsumerIdentity>(
            read(&root.join("native/metadata.json"))?["identity"].clone(),
        )?,
    )?;
    if disk_native.artifact_binding()? != binding {
        return Err(bad("independent source/native master reload differs"));
    }
    let gm = save_masters(&root.join("generate-source"), &l.generate.parameters())?;
    fs::write(
        root.join("generate-source/metadata.json"),
        serde_json::to_vec_pretty(&json!({
        "parameters":gm,"tokenizer_sha256":integer.binding().tokenizer_sha256(),
        "protocol":integer.binding().protocol(),"lanes":l.generate.lanes(),"order_seed":a.seed}))?,
    )?;
    let genbytes = l.generate.export_native()?.to_bytes()?;
    fs::write(root.join("generate.bin"), &genbytes)?;
    let generate = NativeGeometricGenerate::from_bytes(
        &fs::read(root.join("generate.bin"))?,
        integer.binding(),
    )?;
    let disk_gen =
        GenerateLearningWeights::from_native(integer.binding().clone(), &generate, &Device::Cpu)?;
    restore(
        &root.join("generate-source"),
        &gm,
        &disk_gen.parameters(),
        &Device::Cpu,
    )?;
    if disk_gen.export_native()?.to_bytes()? != genbytes {
        return Err(bad("saved Generate masters/native reload differs"));
    }
    let quarter = l.original_bridge.export_native()?.to_bytes()?;
    if sha256_bytes(&quarter) != QUARTER_SHA {
        return Err(bad("original quarter bridge changed"));
    }
    fs::write(root.join("read-state-bridge.bin"), &quarter)?;
    copy_directory(
        &a.checkpoint.join("read-state-bridge-source"),
        &root.join("read-state-bridge-source"),
    )?;
    let cat = l.marker.export_native()?.to_bytes()?;
    if sha256_bytes(&cat) != CAT_SHA {
        return Err(bad("categorical frozen map changed"));
    }
    fs::write(root.join("read-state-bridge-categorical.bin"), &cat)?;
    let bridge = NativeGeometricReadStateBridge::from_bytes(
        &fs::read(root.join("read-state-bridge-categorical.bin"))?,
        integer.binding(),
    )?;
    let rebind = NativeGeometricReadStateBridge::compile(
        integer.binding(),
        bridge.lanes(),
        bridge.packed_bias(),
        bridge.packed_relative(),
    )?;
    if rebind.to_bytes()? != cat {
        return Err(bad(
            "source action binding would change frozen categorical artifact",
        ));
    }
    let original_reload = NativeGeometricReadStateBridge::from_bytes(&quarter, integer.binding())?;
    let original_masters =
        BridgeLearningWeights::from_native(&original_reload, integer.binding(), &Device::Cpu)?;
    let bm = read(&root.join("read-state-bridge-source/metadata.json"))?;
    restore(
        &root.join("read-state-bridge-source"),
        &bm["source_parameters"],
        &original_masters.parameters(),
        &Device::Cpu,
    )?;
    if original_masters.export_native()?.to_bytes()? != quarter
        || original_masters
            .export_categorical_actions()?
            .native
            .to_bytes()?
            != cat
    {
        return Err(bad("independently restored frozen original/map differs"));
    }
    let cue = integer.compile_cue_carrier(l.cue.clone())?;
    let prefix = integer.compile_prefix_transport(&cue, prefix_clone(&l.prefix)?)?;
    fs::create_dir(root.join("cue"))?;
    fs::create_dir(root.join("prefix"))?;
    fs::write(
        root.join("cue/native-metadata.json"),
        serde_json::to_vec_pretty(cue.metadata())?,
    )?;
    fs::write(root.join("cue/cue-q4.bin"), l.cue.packed_coefficients())?;
    if let Some(j) = l.cue.joint() {
        fs::write(root.join("cue/cue-joint-q4.bin"), j.packed_coefficients())?;
    }
    fs::write(
        root.join("prefix/native-metadata.json"),
        serde_json::to_vec_pretty(prefix.metadata())?,
    )?;
    fs::write(
        root.join("prefix/prefix-q4.bin"),
        l.prefix.packed_coefficients(),
    )?;
    let dc = integer.compile_cue_carrier(cue_payload(&root.join("cue"))?)?;
    let dp = integer.compile_prefix_transport(&dc, prefix_payload(&root.join("prefix"))?)?;
    if serde_json::to_value(dc.metadata())? != read(&root.join("cue/native-metadata.json"))?
        || serde_json::to_value(dp.metadata())? != read(&root.join("prefix/native-metadata.json"))?
        || fs::read(root.join("native/consumer/exp-q31.bin"))? != l.exp
    {
        return Err(bad("frozen sidecar/exp payload reload differs"));
    }
    let old_binding = &l.receipt["parent"];
    let receipt = json!({"step":step,"parent":binding,"original_parent":old_binding,
        "source_metadata_rebound":serde_json::to_value(&binding)?!=*old_binding,
        "binding_scope":"source context/potential metadata honestly rebound; typed action bindings checked against current artifact; unchanged bridge coefficients, original masters and map bytes independently verified",
        "generate_sha256":sha256_bytes(&genbytes),"original_quarter_sha256":QUARTER_SHA,"categorical_sha256":CAT_SHA,
        "categorical_receipt":l.categorical_receipt,"credit":a.credit.name(),"order_seed":a.seed,
        "native_independently_reloaded":true,"masters_independently_reloaded":true,
        "fresh_adam":"moments zero-initialized; not historical optimizer continuation",
        "frozen_bridge_training":"marker parameters excluded from Adam and clip; original masters unchanged"});
    let mut receipt = receipt;
    receipt["checkpoint_index_not_optimizer_iteration"] = json!(true);
    receipt["step_scope"] =
        json!("export index only; packet candidates independently reset to common parent");
    fs::write(
        root.join("receipt.json"),
        serde_json::to_vec_pretty(&receipt)?,
    )?;
    Ok((integer, generate, bridge, receipt))
}

fn optimizer(vars: &BTreeMap<String, Var>, lr: f64) -> Result<AdamW> {
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
}
fn clip_denominator(grads: &BTreeMap<String, Tensor>, d: &Device) -> Result<(Tensor, f64)> {
    if grads.is_empty() {
        return Err(bad("no active parameter gradients"));
    }
    let mut scale = 0f64;
    for g in grads.values() {
        let value = g.abs()?.flatten_all()?.max(0)?.to_scalar::<f32>()? as f64;
        if !value.is_finite() {
            return Err(bad("nonfinite accumulated gradient"));
        }
        scale = scale.max(value);
    }
    let norm = if scale == 0. {
        0.
    } else {
        let terms = grads
            .values()
            .map(|g| g.affine(1. / scale, 0.)?.sqr()?.sum_all())
            .collect::<candle_core::Result<Vec<_>>>()?;
        scale * (Tensor::stack(&terms, 0)?.sum_all()?.to_scalar::<f32>()? as f64).sqrt()
    };
    if !norm.is_finite() {
        return Err(bad("nonfinite global gradient norm"));
    }
    let denominator = norm.max(1.) as f32;
    if !denominator.is_finite() {
        return Err(bad("gradient clip denominator exceeds F32"));
    }
    Ok((Tensor::new(denominator, d)?, norm))
}

#[derive(Clone, Copy, Debug)]
enum Pullback {
    Legacy,
    Categorical,
}
impl Pullback {
    fn name(self) -> &'static str {
        match self {
            Self::Legacy => "legacy",
            Self::Categorical => "categorical",
        }
    }
}
const PACKET_ROWS: [usize; 4] = [0, 4, 8, 12];
const AMPLITUDES: [f64; 6] = [0., 0.25, 1., 4., 16., -1.];
type Snapshot = BTreeMap<String, Tensor>;
fn snapshot(vars: &BTreeMap<String, Var>) -> Result<Snapshot> {
    vars.iter()
        .map(|(name, var)| Ok((name.clone(), var.as_tensor().detach().copy()?)))
        .collect()
}
fn reset(vars: &BTreeMap<String, Var>, base: &Snapshot) -> Result<()> {
    if vars.keys().ne(base.keys()) {
        return Err(bad("reset parameter inventory differs"));
    }
    for (name, var) in vars {
        var.set(base.get(name).ok_or_else(|| bad("reset base missing"))?)?;
    }
    Ok(())
}
fn candidate(
    vars: &BTreeMap<String, Var>,
    base: &Snapshot,
    delta: &Snapshot,
    alpha: f64,
) -> Result<()> {
    if !AMPLITUDES.contains(&alpha) || vars.keys().ne(base.keys()) || vars.keys().ne(delta.keys()) {
        return Err(bad("candidate amplitude/inventory differs"));
    }
    // Always use copied original base, never the preceding candidate.
    for (name, var) in vars {
        var.set(&base[name].add(&delta[name].affine(alpha, 0.)?)?)?;
    }
    Ok(())
}
fn project(l: &Loaded) -> Result<()> {
    l.generate.project_shadow_range()?; // Deliberately excludes prototype_choices.
    for var in l.source.context_state_parameters().values() {
        var.set(&var.as_tensor().clamp(-1.75, 1.75)?)?;
    }
    l.source.project_potential_range()?;
    Ok(())
}
fn tensor_inventory(tensors: &Snapshot) -> Result<Value> {
    let mut rows = Vec::new();
    for (name, t) in tensors {
        let values = t.flatten_all()?.to_vec1::<f32>()?;
        if values.iter().any(|x| !x.is_finite()) {
            return Err(bad("nonfinite parameter/direction"));
        }
        let max = values.iter().map(|v| v.abs() as f64).fold(0., f64::max);
        let norm = if max == 0. {
            0.
        } else {
            max * values
                .iter()
                .map(|v| (*v as f64 / max).powi(2))
                .sum::<f64>()
                .sqrt()
        };
        let bytes = values
            .iter()
            .flat_map(|v| v.to_le_bytes())
            .collect::<Vec<_>>();
        rows.push(json!({"parameter":name,"shape":t.dims(),"sha256":sha256_bytes(&bytes),"maximum_abs":max,"l2_norm":norm,"nonzero":values.iter().filter(|v|**v!=0.).count(),"activity":if max==0.{"INACTIVE"}else{"ACTIVE"}}));
    }
    Ok(json!(rows))
}
fn proposal(l: &Loaded, grads: &Snapshot, d: &Device) -> Result<(Snapshot, Value)> {
    let gp = l.generate.parameters();
    let (prototype, coefficients): (BTreeMap<_, _>, BTreeMap<_, _>) = gp
        .into_iter()
        .partition(|(name, _)| name == "generate.prototype_choices");
    let context = l.source.context_state_parameters();
    let potential = l.source.potential_parameters();
    let vars = active(&l.source, &l.generate)?;
    let base = snapshot(&vars)?;
    let (denominator, norm) = clip_denominator(grads, d)?;
    // Each arm gets fresh moments, one identical declared joint update.
    let mut go = optimizer(&coefficients, 0.003)?;
    let mut po = optimizer(&prototype, 0.01)?;
    let mut co = optimizer(&context, 0.002)?;
    let mut vo = optimizer(&potential, 0.003)?;
    apply(&mut go, &coefficients, grads, &denominator)?;
    apply(&mut po, &prototype, grads, &denominator)?;
    apply(&mut co, &context, grads, &denominator)?;
    apply(&mut vo, &potential, grads, &denominator)?;
    let delta = vars
        .iter()
        .map(|(name, var)| {
            Ok((
                name.clone(),
                var.as_tensor().sub(&base[name])?.detach().copy()?,
            ))
        })
        .collect::<Result<Snapshot>>()?;
    let receipt = json!({"fresh_adam_proposal_updates":1,"global_gradient_norm":norm,"clip_denominator":denominator.to_scalar::<f32>()?,"gradients":tensor_inventory(grads)?,"unprojected_displacement":tensor_inventory(&delta)?,"rates":{"generate":0.003,"prototype":0.01,"context":0.002,"potential":0.003},"amplitude_scope":"scale resulting parameter displacement; do not scale gradients before Adam","limitation":"fresh first Adam proposal often resembles sign gradient; magnitude or one candidate gain does not establish mechanism advantage"});
    reset(&vars, &base)?;
    Ok((delta, receipt))
}
fn displacement(vars: &BTreeMap<String, Var>, base: &Snapshot) -> Result<Value> {
    let delta = vars
        .iter()
        .map(|(name, var)| Ok((name.clone(), var.as_tensor().sub(&base[name])?.detach())))
        .collect::<Result<Snapshot>>()?;
    let mut bounds = BTreeMap::new();
    for (name, var) in vars {
        let values = var.flatten_all()?.to_vec1::<f32>()?;
        if values.iter().any(|v| !v.is_finite()) {
            return Err(bad("nonfinite projected master"));
        }
        if name != "generate.prototype_choices" {
            if values.iter().any(|v| v.abs() > 1.75) {
                return Err(bad("projected bounded master outside strict range"));
            }
            bounds.insert(
                name.clone(),
                values.iter().filter(|v| v.abs() == 1.75).count(),
            );
        }
    }
    Ok(
        json!({"projected_displacement":tensor_inventory(&delta)?,"bounded_parameters_at_endpoint":bounds,"prototype_bounds":"unbounded; original Generate projection excludes prototype_choices"}),
    )
}
fn native_hashes(root: &Path, g: &NativeGeometricGenerate) -> Result<Value> {
    Ok(
        json!({"native_metadata":sha256_file(&root.join("native/metadata.json"))?,"context_q4":sha256_file(&root.join("native/consumer/context-q4.bin"))?,"potential_q4":sha256_file(&root.join("native/consumer/potential-q4.bin"))?,"generate_artifact":sha256_file(&root.join("generate.bin"))?,"generate_operator_payload":g.metadata().payload_sha256,"categorical_artifact":sha256_file(&root.join("read-state-bridge-categorical.bin"))?,"cue_payload":sha256_file(&root.join("cue/cue-q4.bin"))?,"prefix_payload":sha256_file(&root.join("prefix/prefix-q4.bin"))?,"scope":"metadata/source-binding changes are distinct from discrete operator payload crossings"}),
    )
}
fn native_packet(
    a: &Args,
    model: &IntegerRealizer,
    g: &NativeGeometricGenerate,
    bridge: &NativeGeometricReadStateBridge,
    l: &Loaded,
    episodes: &[Episode],
) -> Result<Value> {
    let mut pool = NativeVocabularyActions::new(model.binding().clone(), &l.exp)?;
    let mut rows = Vec::new();
    let mut total = 0.;
    let mut phase_loss = [0.; 3];
    let mut counts = [0usize; 3];
    for &index in &PACKET_ROWS {
        let e = &episodes[index];
        let mut plan = None;
        let mut positions = Vec::new();
        for (t, &target) in e.target.iter().enumerate() {
            // Teacher prefix is the declared offline objective, not an oracle-selected source.
            let native = native_step(
                model,
                g,
                Some(bridge),
                &mut pool,
                e,
                &e.target[..t],
                &l.cue,
                &l.prefix,
            )?;
            let copy_ids: Vec<u32> = serde_json::from_value(native["copy_token_ids"].clone())?;
            if plan.is_none() {
                plan = Some(episode_loss_weights(
                    &e.target,
                    &copy_ids.into_iter().collect(),
                    PACKET_ROWS.len(),
                    true,
                )?);
            }
            let weights = plan
                .as_ref()
                .ok_or_else(|| bad("packet phase plan missing"))?;
            let denominator = native["pool"]["summary"]["total_weight_q31"]
                .as_u64()
                .ok_or_else(|| bad("native denominator missing"))?;
            let masses = native["pool"]["token_masses"]
                .as_array()
                .ok_or_else(|| bad("native decomposition missing"))?;
            let mass = masses
                .iter()
                .find(|m| m[0].as_u64() == Some(target as u64))
                .ok_or_else(|| bad("native target mass missing"))?;
            let all = mass[1]
                .as_u64()
                .ok_or_else(|| bad("native target mass type"))?;
            if all == 0 || denominator == 0 || all > denominator {
                return Err(bad("native positive support violated"));
            }
            let nll = -(all as f64 / denominator as f64).ln();
            let weighted = nll * weights.weights[t];
            let phase = weights.phases[t];
            if !weighted.is_finite() {
                return Err(bad("nonfinite native aliasloss"));
            }
            total += weighted;
            phase_loss[phase] += weighted;
            counts[phase] += 1;
            positions.push(json!({"position":t,"phase":phase,"label_only_target":target,"native_alias_loss":nll,"phase_weight":weights.weights[t],"target_total_q31":all,"target_generate_q31":mass[2],"target_copy_q31":mass[3],"denominator_q31":denominator,"target_probability":all as f64/denominator as f64,"native":native}));
        }
        rows.push(json!({"index":index,"id":e.packet.id,"positions":positions}));
    }
    if counts != [4, 22, 16] {
        return Err(bad("audited packet phase coverage differs"));
    }
    Ok(
        json!({"weighted_native_alias_loss":total,"phase_losses":phase_loss,"phase_counts":counts,"rows":rows,"scope":"all42 canonical positions; ordinary clipped native complete-alias objective; no gate/selected label; no heldout/own-feedback claim"}),
    )
}
fn disk_floor(a: &Args) -> Result<()> {
    let output = std::process::Command::new("df")
        .arg("-Pk")
        .arg(&a.out)
        .output()?;
    if !output.status.success() {
        return Err(bad("disk-floor observation unavailable"));
    }
    let text = std::str::from_utf8(&output.stdout)?;
    let available = text
        .lines()
        .last()
        .and_then(|l| l.split_whitespace().nth(3))
        .and_then(|n| n.parse::<u64>().ok())
        .ok_or_else(|| bad("disk-floor parse"))?;
    if available < 128 * 1024 {
        return Err(bad("128MiB disk stop margin reached"));
    }
    Ok(())
}

fn packet_contrast(base: &Value, current: &Value) -> Result<Value> {
    let mut positions = Vec::new();
    let mut route_changes = 0;
    let mut state_changes = 0;
    let mut score_changes = 0;
    let oldrows = base["rows"]
        .as_array()
        .ok_or_else(|| bad("base packet rows"))?;
    let newrows = current["rows"]
        .as_array()
        .ok_or_else(|| bad("candidate packet rows"))?;
    if oldrows.len() != newrows.len() {
        return Err(bad("packet row coverage changed"));
    }
    for (old, new) in oldrows.iter().zip(newrows) {
        if old["id"] != new["id"] {
            return Err(bad("packet row identity changed"));
        }
        let op = old["positions"]
            .as_array()
            .ok_or_else(|| bad("base positions"))?;
        let np = new["positions"]
            .as_array()
            .ok_or_else(|| bad("candidate positions"))?;
        if op.len() != np.len() {
            return Err(bad("position coverage changed"));
        }
        for (a, b) in op.iter().zip(np) {
            if a["position"] != b["position"] || a["label_only_target"] != b["label_only_target"] {
                return Err(bad("position/target identity changed"));
            }
            let route = a["native"]["source_provenance"]["read_state_bridge"]["selected_candidate"]
                != b["native"]["source_provenance"]["read_state_bridge"]["selected_candidate"];
            let state = a["native"]["retained_state_codes"] != b["native"]["retained_state_codes"];
            let score = a["native"]["generate_raw_scores_sha256"]
                != b["native"]["generate_raw_scores_sha256"]
                || a["native"]["copy_raw_scores_q24"] != b["native"]["copy_raw_scores_q24"];
            route_changes += usize::from(route);
            state_changes += usize::from(state);
            score_changes += usize::from(score);
            positions.push(json!({"row_id":new["id"],"position":b["position"],"phase":b["phase"],"target_probability_delta":b["target_probability"].as_f64().ok_or_else(||bad("candidate probability"))?-a["target_probability"].as_f64().ok_or_else(||bad("base probability"))?,"route_changed":route,"state_changed":state,"scores_changed":score,"canonical_token_changed":a["native"]["pool"]["summary"]["chosen_token_id"]!=b["native"]["pool"]["summary"]["chosen_token_id"]}));
        }
    }
    Ok(
        json!({"native_loss_delta":current["weighted_native_alias_loss"].as_f64().ok_or_else(||bad("candidate loss"))?-base["weighted_native_alias_loss"].as_f64().ok_or_else(||bad("base loss"))?,"route_changes":route_changes,"state_changes":state_changes,"score_changes":score_changes,"positions":positions,"unchanged_scope":"no discrete behavior crossing is a plateau, not a mechanism failure"}),
    )
}
fn direction_run(a: &Args, start: Instant) -> Result<Value> {
    if a.mode != Mode::Admission || a.credit != Credit::RawIdentity || a.baseline.is_some() {
        return Err(bad("direction requires admission mode, fixed raw_identity score adjoint, no baseline reuse"));
    }
    let d = cuda()?;
    let l = load(a, &d)?;
    let public = NativeVocabularyActions::new(l.integer.binding().clone(), &l.exp)?;
    let legal = public
        .legal_token_ids()
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    let episodes = load_panel(
        &a.training_inputs,
        &a.training_labels,
        &l.integer,
        &l.tokenizer,
        &legal,
        512,
    )?;
    if episodes.len() != 512 {
        return Err(bad("full authenticated512 panel required"));
    }
    let vars = active(&l.source, &l.generate)?;
    let base = snapshot(&vars)?;
    let base_ids = identities(&vars)?;
    let original_ids = identities(&l.original_bridge.parameters())?;
    let marker_ids = identities(&l.marker.parameters())?;
    disk_floor(a)?;
    let (native, g, bridge, initial_receipt) = checkpoint(a, 0, &l)?;
    let base_packet = native_packet(a, &native, &g, &bridge, &l, &episodes)?;
    let original_generate = NativeGeometricGenerate::from_bytes(
        &fs::read(a.checkpoint.join("generate.bin"))?,
        l.integer.binding(),
    )?;
    let original_categorical = l.marker.export_native()?;
    let original_packet = native_packet(
        a,
        &l.integer,
        &original_generate,
        &original_categorical,
        &l,
        &episodes,
    )?;
    if base_packet != original_packet
        || initial_receipt["parent"] != l.receipt["parent"]
        || g.to_bytes()? != fs::read(a.checkpoint.join("generate.bin"))?
    {
        return Err(bad(
            "alpha-zero export differs from exact parent master/native behavior",
        ));
    }
    let base_hashes = native_hashes(&a.out.join("checkpoint-0000"), &g)?;
    write(a, "packet-base.json", &base_packet)?;
    let mut candidates = vec![
        json!({"arm":"shared_parent","alpha":0.,"checkpoint":"checkpoint-0000","native_hashes":base_hashes,"native_loss":base_packet["weighted_native_alias_loss"],"master_identity":base_ids,"baseline_true_parent":true}),
    ];
    let mut proposals = Vec::new();
    let mut index = 0;
    let mut directions = Vec::new();
    let mut gradient_packets = Vec::new();
    for pullback in [Pullback::Legacy, Pullback::Categorical] {
        reset(&vars, &base)?;
        if identities(&vars)? != base_ids {
            return Err(bad("gradient arm reset changed parent"));
        }
        let (grads, before) = batch(
            a,
            &l,
            &episodes,
            &PACKET_ROWS,
            &d,
            start,
            Some(&native),
            pullback,
        )?;
        let (delta, receipt) = proposal(&l, &grads, &d)?;
        if identities(&vars)? != base_ids {
            return Err(bad("Adam proposal reset changed parent"));
        }
        proposals.push(json!({"arm":pullback.name(),"before":before,"proposal":receipt}));
        gradient_packets.push(grads);
        directions.push((pullback, delta));
    }
    let differences = directions[0]
        .1
        .iter()
        .map(|(name, t)| Ok((name.clone(), t.sub(&directions[1].1[name])?)))
        .collect::<Result<Snapshot>>()?;
    let mut family_comparison = Vec::new();
    for (name, keys) in [
        (
            "Context",
            l.source
                .context_state_parameters()
                .keys()
                .cloned()
                .collect::<BTreeSet<_>>(),
        ),
        (
            "Potential",
            l.source.potential_parameters().keys().cloned().collect(),
        ),
        (
            "Generate",
            l.generate.parameters().keys().cloned().collect(),
        ),
    ] {
        let proposal_diff = differences
            .iter()
            .filter(|(key, _)| keys.contains(*key))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect::<Snapshot>();
        let mut gradient_diff = Snapshot::new();
        let mut gradient_inventory_equal = true;
        for key in &keys {
            match (gradient_packets[0].get(key), gradient_packets[1].get(key)) {
                (Some(a), Some(b)) => {
                    gradient_diff.insert(key.clone(), a.sub(b)?);
                }
                (None, None) => {}
                _ => {
                    gradient_inventory_equal = false;
                }
            }
        }
        family_comparison.push(json!({"family":name,"gradient_inventory_equal":gradient_inventory_equal,"gradient_difference":tensor_inventory(&gradient_diff)?,"proposal_difference":tensor_inventory(&proposal_diff)?}));
    }
    write(
        a,
        "proposal-directions.json",
        &json!({"arms":proposals,"families":family_comparison,"legacy_minus_categorical_proposal":tensor_inventory(&differences)?,"attribution":"common Generate bias and freshAdam sign-like proposals can yield shared gains; report families and native phase effects, not causal credit from aggregate gain alone"}),
    )?;
    for (pullback, delta) in directions {
        for alpha in AMPLITUDES.into_iter().filter(|v| *v != 0.) {
            deadline(a, start)?;
            reset(&vars, &base)?;
            if identities(&vars)? != base_ids {
                return Err(bad("candidate reset changed parent"));
            }
            candidate(&vars, &base, &delta, alpha)?;
            project(&l)?;
            let projected = displacement(&vars, &base)?;
            if identities(&l.original_bridge.parameters())? != original_ids
                || identities(&l.marker.parameters())? != marker_ids
            {
                return Err(bad("frozen transport master changed"));
            }
            disk_floor(a)?;
            index += 1;
            let (n, g, b, receipt) = checkpoint(a, index, &l)?;
            let packet = native_packet(a, &n, &g, &b, &l, &episodes)?;
            let hashes = native_hashes(&a.out.join(format!("checkpoint-{index:04}")), &g)?;
            let contrast = packet_contrast(&base_packet, &packet)?;
            let name = format!("packet-{}-alpha-{alpha}.json", pullback.name());
            write(a, &name, &packet)?;
            candidates.push(json!({"arm":pullback.name(),"alpha":alpha,"checkpoint":format!("checkpoint-{index:04}"),"checkpoint_index_not_optimizer_iteration":true,"native_hashes":hashes,"operator_hash_crossings":{"context":hashes["context_q4"]!=base_hashes["context_q4"],"potential":hashes["potential_q4"]!=base_hashes["potential_q4"],"generate":hashes["generate_operator_payload"]!=base_hashes["generate_operator_payload"]},"projected":projected,"native_contrast":contrast,"packet_file":name,"packet_sha256":sha256_file(&a.out.join(&name))?,"export_receipt":receipt}));
            write(a, "candidates.json", &json!(candidates))?;
        }
    }
    reset(&vars, &base)?;
    if identities(&vars)? != base_ids || index != 10 {
        return Err(bad("final true-parent reset/11exports differs"));
    }
    Ok(
        json!({"schema":"uor-r4.categorical-native-direction/1","status":"COMPLETED","source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT").unwrap_or("UNBOUND"),"rows":PACKET_ROWS,"phase_counts":[4,22,16],"amplitudes":AMPLITUDES,"bridge_credit_arms":["legacy","categorical"],"score_adjoint":"raw_identity both arms; native/scalar alias objective clipped unchanged","fresh_adam_proposals":2,"chained_training_updates":0,"native_exports":11,"initial_receipt":initial_receipt,"initial_master_identity":base_ids,"final_master_reset_byte_identical":true,"categorical_sha256":CAT_SHA,"data_sha256":INPUT_SHA,"label_sha256":LABEL_SHA,"checkpoint_receipt_sha256":CP_RECEIPT_SHA,"candidates":candidates,"elapsed_seconds":start.elapsed().as_secs_f64(),"scope":"shared legal parameter direction packet; allphysicalbank independently reloaded for every candidate; no semantic/runtime gate or selected-label injection; no fit/heldout/chat/geometric-advantage claim; freshAdam sign-like normalization limits attribution; canonical objective only, ownfeedback NOT_RUN"}),
    )
}
fn main() -> Result<()> {
    let a = args()?;
    if a.mode != Mode::Admission || a.credit != Credit::RawIdentity || a.baseline.is_some() {
        return Err(bad("fixed direction admission/raw_identity config only"));
    }
    report_output::claim(&a.out)?;
    let start = Instant::now();
    let result = direction_run(&a, start);
    let report = match &result {
        Ok(v) => v.clone(),
        Err(e) => {
            json!({"schema":"uor-r4.categorical-native-direction/1","status":"FAILED","error":e.to_string(),"elapsed_seconds":start.elapsed().as_secs_f64(),"model_verdict":"UNQUALIFIED; preserve partial candidates; instrument failure is not quality evidence"})
        }
    };
    fs::write(
        a.out.join("report.json"),
        serde_json::to_vec_pretty(&report)?,
    )?;
    report_output::seal(&a.out)?;
    report_output::verify(&a.out)?;
    result.map(|_| ())
}
#[cfg(test)]
mod direction_tests {
    use super::*;
    #[test]
    fn snapshots_are_owned_and_candidates_reset_instead_of_accumulating() -> Result<()> {
        let v = Var::from_vec(vec![1f32, -1.], 2, &Device::Cpu)?;
        let vars = BTreeMap::from([("x".into(), v.clone())]);
        let base = snapshot(&vars)?;
        let delta = BTreeMap::from([(
            "x".into(),
            Tensor::from_vec(vec![0.5f32, 0.25], 2, &Device::Cpu)?,
        )]);
        candidate(&vars, &base, &delta, 4.)?;
        assert_eq!(v.to_vec1::<f32>()?, vec![3., 0.]);
        candidate(&vars, &base, &delta, 1.)?;
        assert_eq!(v.to_vec1::<f32>()?, vec![1.5, -0.75]);
        reset(&vars, &base)?;
        assert_eq!(v.to_vec1::<f32>()?, vec![1., -1.]);
        candidate(&vars, &base, &delta, 0.)?;
        assert_eq!(v.to_vec1::<f32>()?, vec![1., -1.]);
        assert!(candidate(&vars, &base, &delta, 2.).is_err());
        Ok(())
    }
    #[test]
    fn audited_phase_weights_preserve_equal_nonempty_mass() -> Result<()> {
        for (copy_count, rest_count) in [(6, 4), (5, 4)] {
            let mut target = vec![2997];
            target.extend(vec![617; copy_count]);
            target.extend(vec![200; rest_count]);
            let p = episode_loss_weights(&target, &BTreeSet::from([617]), 4, true)?;
            assert_eq!(p.counts, [1, copy_count, rest_count]);
            for phase in 0..3 {
                let mass = p
                    .weights
                    .iter()
                    .zip(&p.phases)
                    .filter(|(_, q)| **q == phase)
                    .map(|(w, _)| w)
                    .sum::<f64>();
                assert!((mass - 1. / 12.).abs() < 1e-14);
            }
        }
        Ok(())
    }
    #[test]
    fn actual_generate_projection_clamps_coefficients_but_preserves_prototypes() -> Result<()> {
        let tok=br#"{"pre_tokenizer":{"type":"ByteLevel","add_prefix_space":false},"model":{"type":"BPE","vocab":{"<|bos|>":0,"<|eos|>":1,"<|unk|>":2,".":3,"a":4},"merges":[]},"added_tokens":[{"id":0,"content":"<|bos|>"},{"id":1,"content":"<|eos|>"},{"id":2,"content":"<|unk|>"}]}"#;
        let binding = uor_r4_integer::geometric_source_actions::SourceActionBinding::new(tok)?;
        let g = GenerateLearningWeights::seeded(binding, 1, 1, &Device::Cpu)?;
        let params = g.parameters();
        for v in params.values() {
            if v.elem_count() > 0 {
                v.set(
                    &Tensor::ones(v.shape(), candle_core::DType::F32, &Device::Cpu)?
                        .affine(4., 0.)?,
                )?;
            }
        }
        g.project_shadow_range()?;
        for (name, v) in params {
            let expected = if name == "generate.prototype_choices" {
                4.
            } else {
                1.75
            };
            assert!(v
                .flatten_all()?
                .to_vec1::<f32>()?
                .iter()
                .all(|x| *x == expected));
        }
        Ok(())
    }
    #[test]
    fn native_contrast_separates_probability_from_token_flip() -> Result<()> {
        let packet = |prob: f64, choice: u64| json!({"weighted_native_alias_loss":-prob.ln(),"rows":[{"id":"x","positions":[{"position":0,"label_only_target":1,"phase":0,"target_probability":prob,"native":{"source_provenance":{"read_state_bridge":{"selected_candidate":{"record":1}}},"retained_state_codes":[2],"generate_raw_scores_sha256":"same","copy_raw_scores_q24":[0],"pool":{"summary":{"chosen_token_id":choice}}}}]}]});
        let same = packet_contrast(&packet(0.1, 2), &packet(0.1, 2))?;
        assert_eq!(same["native_loss_delta"], 0.);
        let better = packet_contrast(&packet(0.1, 2), &packet(0.2, 2))?;
        assert!(
            better["native_loss_delta"]
                .as_f64()
                .ok_or_else(|| bad("loss"))?
                < 0.
        );
        assert_eq!(better["positions"][0]["canonical_token_changed"], false);
        Ok(())
    }
}
