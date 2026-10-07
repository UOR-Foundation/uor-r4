//! RECORD-only zero-update full-bank categorical pullback parity.
//! Retained fitter loaders/helpers copied unchanged from base4c3fe850; no optimizer is invoked.
//! Targets enter only the token-alias objective; native source selection stays target-free.
#![recursion_limit = "256"]
use candle_core::{Device, Tensor, Var};
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
    geometric_bank_generate::{BankGenerateOutput, PreparedBankGenerate},
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
fn bank_snapshot(out: &BankGenerateOutput) -> Result<Value> {
    let copy = out
        .copy
        .as_ref()
        .ok_or_else(|| bad("actual Copy bank absent"))?;
    let bank = &copy.trace.cue_bank.bank;
    let (selected, bridge) = out
        .read_state_bridge
        .as_ref()
        .ok_or_else(|| bad("bridge absent"))?;
    Ok(json!({
        "causal_tokens":out.causal_token_ids,
        "candidates":bank.candidates,
        "context_states":bank.context.states,
        "selected_ordinal":selected,
        "post_state_codes":out.final_state_codes.iter().map(|c|c.index()).collect::<Vec<_>>(),
        "bridge_action_codes":bridge.action_codes.iter().map(|c|c.index()).collect::<Vec<_>>(),
        "bridge_action_scores_q24":bridge.action_scores_q24,
        "copy_token_ids":out.copy_token_ids,"copy_raw_scores_q24":out.copy_scores_q24,
        "generate_scores_q24":out.generate.scores_q24,
        "pool_summary":out.actions.summary,
        "token_masses":out.actions.token_masses.iter().map(|m|[m.token_id as u64,m.weight_q31,m.generate_weight_q31,m.copy_weight_q31]).collect::<Vec<_>>()
    }))
}
fn compare_independent(snapshot: &Value, native: &Value) -> Result<()> {
    for (a, b) in [
        ("post_state_codes", "retained_state_codes"),
        ("copy_token_ids", "copy_token_ids"),
        ("copy_raw_scores_q24", "copy_raw_scores_q24"),
    ] {
        if snapshot[a] != native[b] {
            return Err(bad("independent native bank vectors differ"));
        }
    }
    if snapshot["pool_summary"] != native["pool"]["summary"]
        || snapshot["token_masses"] != native["pool"]["token_masses"]
        || sha256_bytes(&serde_json::to_vec(&snapshot["generate_scores_q24"])?)
            != native["generate_raw_scores_sha256"]
        || snapshot["causal_tokens"] != native["source_provenance"]["causal_tokens"]
        || snapshot["candidates"] != native["source_provenance"]["candidates"]
        || snapshot["selected_ordinal"]
            != native["source_provenance"]["read_state_bridge"]["selected_ordinal"]
        || snapshot["bridge_action_codes"]
            != native["source_provenance"]["read_state_bridge"]["action_codes"]
        || snapshot["bridge_action_scores_q24"]
            != native["source_provenance"]["read_state_bridge"]["action_scores_q24"]
    {
        return Err(bad(
            "independent native complete alias/bridge parity differs",
        ));
    }
    Ok(())
}
fn compare_saved(native: &Value, saved: &Value) -> Result<()> {
    // Producer intentionally removed token_masses: authenticate/replay available
    // fields, and compare complete masses only against fresh independent native.
    for field in [
        "actual_prefix_ids",
        "retained_state_codes",
        "copy_token_ids",
        "copy_raw_scores_q24",
        "generate_raw_scores_sha256",
        "source_provenance",
    ] {
        if native[field] != saved[field] {
            return Err(bad("saved factual native replay differs"));
        }
    }
    if native["pool"]["summary"] != saved["pool"]["summary"] {
        return Err(bad("saved native pool summary differs"));
    }
    Ok(())
}
fn context_gradient_receipt(
    grads: &candle_core::backprop::GradStore,
    vars: &BTreeMap<String, Var>,
    d: &Device,
) -> Result<Value> {
    let mut rows = Vec::new();
    let mut any = false;
    for (name, var) in vars {
        let g = grads
            .get(var.as_tensor())
            .ok_or_else(|| bad("actual context Var gradient absent"))?;
        if !g.device().same_device(d) {
            return Err(bad("context gradient changed device"));
        }
        let maximum = g.abs()?.flatten_all()?.max(0)?.to_scalar::<f32>()?;
        if !maximum.is_finite() {
            return Err(bad("context gradient nonfinite"));
        }
        // Check the sum even for inactive gradients: a backend maximum could
        // ignore a NaN among zeros, while this reduction retains it.
        let scaled = if maximum == 0. {
            g.clone()
        } else {
            (g / maximum as f64)?
        };
        let square = scaled.sqr()?.sum_all()?.to_scalar::<f32>()?;
        if !square.is_finite() || square < 0. {
            return Err(bad("scaled context gradient norm nonfinite"));
        }
        let norm = maximum as f64 * (square as f64).sqrt();
        any |= maximum > 0.;
        rows.push(json!({"parameter":name,"shape":g.dims(),"maximum_abs":maximum,"l2_norm":norm,"activity":if maximum==0.{"INACTIVE"}else{"ACTIVE"}}));
    }
    Ok(
        json!({"parameters":rows,"any_active":any,"scope":"actual Context parameters; finite/device validation; no equality requirement across distinct pullbacks"}),
    )
}
fn parity_run(a: &Args, baseline: &Path, start: Instant) -> Result<Value> {
    if a.mode != Mode::Admission || a.baseline.is_some() {
        return Err(bad("parity requires admission config without fit baseline"));
    }
    report_output::verify(baseline)?;
    let base = read(&baseline.join("report.json"))?;
    let admission = read(&baseline.join("admission.json"))?;
    let expected_manifest = std::env::var("UOR_CATEGORICAL_PARITY_BASELINE_MANIFEST_SHA256")
        .map_err(|_| bad("pinned baseline manifest SHA256 required"))?;
    if sha256_file(&baseline.join("manifest.json"))? != expected_manifest
        || base["status"] != "COMPLETED"
        || base["mode"] != "admission"
        || base["updates"] != 0
        || admission["source_commit"].as_str().is_none()
    {
        return Err(bad("pinned completed sealed zero-update baseline required"));
    }
    let device = cuda()?;
    let l = load(a, &device)?;
    let initial = identities(&active(&l.source, &l.generate)?)?;
    let bridge_initial = identities(&l.original_bridge.parameters())?;
    let marker_initial = identities(&l.marker.parameters())?;
    let public = NativeVocabularyActions::new(l.generate.binding().clone(), &l.exp)?;
    let legal = public
        .legal_token_ids()
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    let episodes = load_panel(
        &a.development_inputs,
        &a.development_labels,
        &l.integer,
        &l.tokenizer,
        &legal,
        512,
    )?;
    if episodes.len() != 512 {
        return Err(bad("complete512 admitted panel required"));
    }
    let current = l.source.compile_context_potential_rebound(&l.frozen)?;
    let prepared = l
        .source
        .prepare_context_potential_on_device(&current, &device)?;
    let cue = current.compile_cue_carrier(l.cue.clone())?;
    let prefix = current.compile_prefix_transport(&cue, prefix_clone(&l.prefix)?)?;
    let gs = l.generate.prepare_native()?;
    let bs = l.marker.prepare_native()?;
    let bytes = bs.native.to_bytes()?;
    let categorical =
        PreparedCategoricalBridge::from_bytes(&bytes, l.generate.binding(), CAT_SHA, &device)?;
    let mut pool = NativeVocabularyActions::new(l.generate.binding().clone(), &l.exp)?;
    let mut rows = Vec::new();
    let mut any_categorical_active = false;
    for row_index in [0usize, 8] {
        let e = &episodes[row_index];
        let saved_path = baseline.join(format!("development-0000-row-{row_index:04}.json"));
        let saved_bytes = fs::read(&saved_path)?;
        let saved: Value = serde_json::from_slice(&saved_bytes)?;
        if saved["id"] != e.packet.id {
            return Err(bad("saved row ID differs"));
        }
        for step_index in [0usize, 3] {
            for prefix_scope in ["teacher_canonical", "actual_emitted"] {
                let saved_step = if prefix_scope == "teacher_canonical" {
                    &saved["canonical"][step_index]["native"]
                } else {
                    &saved["generation"][step_index]
                };
                if saved_step.is_null() {
                    return Err(bad("required saved physical position missing"));
                }
                let actual: Vec<u32> =
                    serde_json::from_value(saved_step["actual_prefix_ids"].clone())?;
                if actual.len() != step_index {
                    return Err(bad("saved prefix position differs"));
                }
                let independent = native_step(
                    &l.integer,
                    &gs.native,
                    Some(&bs.native),
                    &mut pool,
                    e,
                    &actual,
                    &l.cue,
                    &l.prefix,
                )?;
                compare_saved(&independent, saved_step)?;
                for selector in [false, true] {
                    deadline(a, start)?;
                    let mut old = PreparedBankGenerate::new(&prepared, &l.generate, &gs, &l.exp)?
                        .with_prefix_temporal_utility(false)
                        .with_read_state_bridge(&l.marker, &bs)?
                        .with_read_selector_credit(selector);
                    let mut new = PreparedBankGenerate::new(&prepared, &l.generate, &gs, &l.exp)?
                        .with_prefix_temporal_utility(false)
                        .with_categorical_read_state_bridge(&categorical)?
                        .with_read_selector_credit(selector);
                    // Both complete target-free forwards precede this position's label.
                    let oldout = old.forward_bank(
                        &e.segments()?,
                        &e.packet.query_ids,
                        &actual,
                        &cue,
                        &prefix,
                    )?;
                    let newout = new.forward_bank(
                        &e.segments()?,
                        &e.packet.query_ids,
                        &actual,
                        &cue,
                        &prefix,
                    )?;
                    let snapshot = bank_snapshot(&newout)?;
                    if bank_snapshot(&oldout)? != snapshot {
                        return Err(bad("legacy/categorical native fullbank forward differs"));
                    }
                    compare_independent(&snapshot, &independent)?;
                    let target = *e
                        .target
                        .get(step_index)
                        .ok_or_else(|| bad("label position missing"))?;
                    let mut credit_rows = Vec::new();
                    for credit in [
                        VocabularyScoreAdjoint::Clipped,
                        VocabularyScoreAdjoint::RawIdentity,
                    ] {
                        let oldloss = oldout.loss_with_credit(target, credit)?;
                        let newloss = newout.loss_with_credit(target, credit)?;
                        let oldscalar = oldloss.to_scalar::<f32>()?;
                        let newscalar = newloss.to_scalar::<f32>()?;
                        if !oldscalar.is_finite() || oldscalar.to_bits() != newscalar.to_bits() {
                            return Err(bad("same-forward alias scalar loss differs"));
                        }
                        let oldgrads = oldloss.backward()?;
                        let newgrads = newloss.backward()?;
                        let vars = l.source.context_state_parameters();
                        let oldreceipt = context_gradient_receipt(&oldgrads, &vars, &device)?;
                        let newreceipt = context_gradient_receipt(&newgrads, &vars, &device)?;
                        any_categorical_active |= newreceipt["any_active"] == true;
                        credit_rows.push(json!({"credit":format!("{credit:?}"),"label_only_target":target,"scalar_loss":newscalar,"legacy_context":oldreceipt,"categorical_context":newreceipt}));
                    }
                    let name=format!("row-{row_index:04}-step-{step_index}-{prefix_scope}-selector-{selector}.json");
                    let row = json!({"row_id":e.packet.id,"row_index":row_index,"step_index":step_index,"prefix_scope":prefix_scope,"actual_prefix_ids":actual,"selector":selector,"saved_row_sha256":sha256_bytes(&saved_bytes),"native":snapshot,"independent_native":independent,"legacy_scope":oldout.credit_scope,"categorical_scope":newout.credit_scope,"credit":credit_rows});
                    write(a, &name, &row)?;
                    rows.push(json!({"row":name,"sha256":sha256_file(&a.out.join(&name))?}));
                }
            }
        }
    }
    if !any_categorical_active {
        return Err(bad("all actual categorical Context adjoints inactive"));
    }
    if initial != identities(&active(&l.source, &l.generate)?)?
        || bridge_initial != identities(&l.original_bridge.parameters())?
        || marker_initial != identities(&l.marker.parameters())?
    {
        return Err(bad("zero-update master identities changed"));
    }
    Ok(
        json!({"schema":"uor-r4.categorical-bank-zero-update/1","status":"COMPLETED","source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT").unwrap_or("UNBOUND"),"device":"cuda","optimizer_updates":0,"baseline":baseline,"baseline_manifest_sha256":sha256_file(&baseline.join("manifest.json"))?,"baseline_report_sha256":sha256_file(&baseline.join("report.json"))?,"baseline_producer":admission["source_commit"],"config_inputs_sha256":INPUT_SHA,"checkpoint_receipt_sha256":CP_RECEIPT_SHA,"categorical_sha256":CAT_SHA,"generate_sha256":GENERATE_SHA,"initial_master_identities":initial,"masters_unchanged":true,"positions":rows.len(),"rows":rows,"any_categorical_context_active":any_categorical_active,"elapsed_seconds":start.elapsed().as_secs_f64(),"scope":"zero-update actual physical allbank; native hardforward/fullalias equality; actual Context gradient instrument; no fit/chat/generalization result; actual-emitted off-correct-prefix labels are diagnostic only"}),
    )
}
fn main() -> Result<()> {
    let a = args()?;
    let baseline = fs::canonicalize(
        std::env::var("UOR_CATEGORICAL_PARITY_BASELINE")
            .map_err(|_| bad("UOR_CATEGORICAL_PARITY_BASELINE required"))?,
    )?;
    let output = output_support::prospective_output(&a.out)?;
    if output.starts_with(&baseline) || baseline.starts_with(&output) {
        return Err(bad("baseline/output overlap"));
    }
    for ancestor in baseline.ancestors() {
        if ancestor.join("manifest.json").exists() && output.starts_with(ancestor) {
            return Err(bad("output beneath baseline seal"));
        }
    }
    if a.mode != Mode::Admission || a.baseline.is_some() {
        return Err(bad("admission config only"));
    }
    report_output::claim(&a.out)?;
    let start = Instant::now();
    let result = parity_run(&a, &baseline, start);
    let report = match &result {
        Ok(v) => v.clone(),
        Err(e) => {
            json!({"schema":"uor-r4.categorical-bank-zero-update/1","status":"FAILED","error":e.to_string(),"optimizer_updates":0,"elapsed_seconds":start.elapsed().as_secs_f64(),"model_verdict":"UNQUALIFIED; execution failure is not model-quality evidence"})
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
