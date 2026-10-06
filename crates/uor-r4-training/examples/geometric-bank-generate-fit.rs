//! Real-panel CUDA Copy+Generate learning. No supplied selected record; labels
//! enter only after target-free native scores. Admission never starts a fit.
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
    native_geometric::learner::geometric_generate::{GenerateReadCounts, NativeGeometricGenerate},
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
    updates: usize,
    learning_rate: f64,
    prototype_learning_rate: f64,
    context_learning_rate: f64,
    maximum_seconds: u64,
    maximum_report_bytes: u64,
    out: PathBuf,
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
    if start.elapsed().as_secs() >= a.maximum_seconds {
        return Err(bad("declared wall bound"));
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
        || !["output-only", "joint"].contains(&a.arm.as_str())
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
        || a.maximum_seconds > 3600
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
    arm: &str,
) -> BTreeMap<String, Var> {
    let mut p = g.parameters();
    if arm == "joint" {
        p.extend(source.context_state_parameters());
    }
    p
}
fn native_step(
    model: &IntegerRealizer,
    g: &NativeGeometricGenerate,
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
        let codes = b
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
        (
            codes,
            ids,
            scores,
            json!({"candidates":b.candidates,"causal_tokens":b.context.tokens,"legacy_terminal_actions_discarded":true}),
        )
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
            let mut step = native_step(model, g, &mut pool, e, &e.target[..t], cue, prefix)?;
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
            let mut step = native_step(model, g, &mut pool, e, &ids, cue, prefix)?;
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
    cue: &CueAngularQ4,
    prefix: &PrefixAngularQ4,
) -> Result<(IntegerRealizer, NativeGeometricGenerate, Value)> {
    let root = a.out.join(format!("checkpoint-{step:04}"));
    let projected_checkpoint = size(&a.source_weights)?
        + size(&a.native_artifact)?
        + g.parameters()
            .values()
            .map(|v| v.elem_count() as u64 * 4)
            .sum::<u64>()
        + 8_388_608;
    if size(&a.out)? + projected_checkpoint > a.maximum_report_bytes - 1_048_576 {
        return Err(bad(
            "checkpoint projected bytes exceed declared cap before export",
        ));
    }
    fs::create_dir(&root)?;
    let native = source.compile_context_state_rebound(frozen)?;
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
            &json!({"parameters":shadows,"tokenizer_sha256":native.binding().tokenizer_sha256(),"protocol":native.binding().protocol(),"seed":a.seed,"lanes":8,"scope":"offline source masters; excluded from serving"}),
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
    let receipt = json!({"step":step,"parent":binding,"generate_sha256":sha256_bytes(&bytes),"cue_metadata_sha256":sha256_file(&root.join("cue/native-metadata.json"))?,"prefix_metadata_sha256":sha256_file(&root.join("prefix/native-metadata.json"))?,"cue_payload_sha256":sha256_bytes(cue.packed_coefficients()),"prefix_payload_sha256":sha256_bytes(prefix.packed_coefficients()),"native_independently_reloaded":true,"context_packed_bytes_changed_from_donor":context_changes,"context_packed_sha256":sha256_bytes(&contextbytes),"frozen_scoring_except_context":true,"generation_f32_source_access":false,"sidecars_independently_disk_reloaded_verified":true,"training_resume":"NOT_SUPPORTED; source masters retained, Adam moment states not exported"});
    fs::write(
        root.join("receipt.json"),
        serde_json::to_vec_pretty(&receipt)?,
    )?;
    if size(&a.out)? > a.maximum_report_bytes - 1_048_576 {
        return Err(bad("checkpoint exceeds complete report cap"));
    }
    Ok((integer, reloaded, receipt))
}

fn batch(
    a: &Args,
    source: &SourceRealizerWeights,
    frozen: &NativeSourceRealizer,
    g: &GenerateLearningWeights,
    cueq: &CueAngularQ4,
    prefixq: &PrefixAngularQ4,
    exp: &[u8],
    eps: &[Episode],
    indices: &[usize],
    device: &Device,
    start: Instant,
    independent: Option<&IntegerRealizer>,
) -> Result<(BTreeMap<String, Tensor>, Value)> {
    let begun = Instant::now();
    let current = source.compile_context_state_rebound(frozen)?;
    let prepared = source.prepare_on_device(&current, device)?;
    let cue = current.compile_cue_carrier(cueq.clone())?;
    let prefix = current.compile_prefix_transport(&cue, prefix_clone(prefixq)?)?;
    let snapshot = g.prepare_native()?;
    let independent_generate =
        NativeGeometricGenerate::from_bytes(&snapshot.native.to_bytes()?, g.binding())?;
    let mut independent_pool = NativeVocabularyActions::new(g.binding().clone(), exp)?;
    let staged = begun.elapsed().as_secs_f64();
    let mut learner = PreparedBankGenerate::new(&prepared, g, &snapshot, exp)?;
    let params = parameters(source, g, "joint");
    let active = parameters(source, g, &a.arm);
    let mut sums = BTreeMap::<String, Tensor>::new();
    let mut nativece = 0.;
    let mut positions = 0;
    let mut backwardseconds = 0.;
    let mut forwardseconds = 0.;
    for &index in indices {
        let e = &eps[index];
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
            if let Some(model) = independent {
                let expected = native_step(
                    model,
                    &independent_generate,
                    &mut independent_pool,
                    e,
                    &e.target[..t],
                    cueq,
                    prefixq,
                )?;
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
            // No target exists in either forward API; it first enters here.
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
            let scaled = loss.affine(1. / e.target.len() as f64 / indices.len() as f64, 0.)?;
            device.synchronize()?;
            forwardseconds += f.elapsed().as_secs_f64();
            let b = Instant::now();
            let grads = scaled.backward()?;
            device.synchronize()?;
            backwardseconds += b.elapsed().as_secs_f64();
            for (name, var) in &params {
                if let Some(grad) = grads.get(var.as_tensor()) {
                    if !grad.device().same_device(device) {
                        return Err(bad("gradient CPU fallback"));
                    }
                    let next = if let Some(old) = sums.get(name) {
                        old.add(grad)?.detach()
                    } else {
                        grad.detach()
                    };
                    sums.insert(name.clone(), next);
                }
            }
            positions += 1;
        }
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
    let report = json!({"episode_indices":indices,"episodes":indices.len(),"target_positions":positions,"native_equal_episode_ce":nativece,"gradient_families":family,"gradient_global_l2":norm2.sqrt(),"staging_native_snapshot_seconds":staged,"forward_including_host_native_oracle_synced_seconds":forwardseconds,"backward_synced_seconds":backwardseconds,"elapsed_seconds":begun.elapsed().as_secs_f64(),"generate_master_download_bytes":snapshot.downloaded_master_bytes,"credit_scope":uor_r4_training::geometric_bank_generate::CREDIT_SCOPE,"gradient_accumulation":"stream per-token backward; detached device F32 sums scaled equal-episode mean; filter before global clip; no host dynamic adjoints","updates":0,"independent_native_hard_pool_parity":independent.is_some(),"output_only_cost_scope":"same composed graph credit is computed for diagnostics; context gradients excluded before global norm/optimizer, context does not update"});
    sums.retain(|name, _| active.contains_key(name));
    Ok((sums, report))
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
    if integer.binding().vocab_size() != 1024
        || integer.context_config().heads * integer.context_config().lanes_per_head != 8
    {
        return Err(bad("actual donor must be V1024/eight lanes"));
    }
    let exp = fs::read(&a.canonical_exp)?;
    let public = NativeVocabularyActions::new(integer.binding().clone(), &exp)?;
    let legal = public
        .legal_token_ids()
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    let source =
        SourceRealizerWeights::load_source_on_device(&a.source_weights, &tokenizerbytes, &device)?;
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
        &json!({"input_sha256":inputs,"input_manifests":before,"training_cases":train.len(),"development_cases":dev.len(),"prospective_evaluation_supplied_not_loaded":a.fresh_inputs.is_some(),"training_no_source_cases":train.iter().filter(|e|!e.has_source()).count(),"public_legal_ids":legal.len(),"vocab":1024,"lanes":8,"source_parent":expected,"cue_payload_sha256":sha256_bytes(cue.packed_coefficients()),"prefix_payload_sha256":sha256_bytes(prefix.packed_coefficients()),"development_scope":"open construction panel; source overlap allowed and reported; not independent generalization","data_quality_scope":"schema/public-alphabet/causal-context/canonical-roundtrip admission; panel's independent answerability receipt remains required; renamed labels alone are not untouched data"}),
    )?;
    write(
        a,
        "frozen-development-answer-oracles.json",
        &json!({"rule":"input FrozenAnswers used exactly; canonical first accepted bytes plus EOS; no runtime formatting or trimming","cases":dev.iter().map(|e|json!({"id":e.packet.id,"answers":e.answers,"canonical_ids_labels_only":e.target})).collect::<Vec<_>>()}),
    )?;
    let generate = GenerateLearningWeights::seeded(integer.binding().clone(), 8, a.seed, &device)?;
    let gparams = generate.parameters();
    let (prototype, coefficients): (BTreeMap<_, _>, BTreeMap<_, _>) = gparams
        .clone()
        .into_iter()
        .partition(|(n, _)| n == "generate.prototype_choices");
    let context = source.context_state_parameters();
    let coeffmargins = quarter_margins(&coefficients)?;
    let contextmargins = quarter_margins(&context)?;
    write(
        a,
        "optimizer-design.json",
        &json!({"generate_coefficient_lr":a.learning_rate,"prototype_lr":a.prototype_learning_rate,"context_lr":a.context_learning_rate,"beta1":0.9,"beta2":0.999,"eps":1e-8,"weight_decay":0.,"same_global_clip_l2":1.,"coefficient_initial_quarter_margins":coeffmargins,"context_initial_quarter_margins":contextmargins,"prototype_initial_winner_gap":2.,"reachability_upper_bound_multiplier_128":227.47318,"reachability_scope":"upper bound permits crossings at declared rates; does not guarantee changes or benefit; no initialization-margin manipulation"}),
    )?;
    let first = (0..8).collect::<Vec<_>>();
    let (_, admission) = batch(
        a,
        &source,
        &frozen,
        &generate,
        &cue,
        &prefix,
        &exp,
        &train,
        &first,
        &device,
        start,
        Some(&integer),
    )?;
    write(a, "first-b8-admission.json", &admission)?;
    let (base_native, base_gen, base_receipt) =
        checkpoint(a, 0, &source, &frozen, &generate, &cue, &prefix)?;
    let baseline = evaluate(
        a,
        "development-0000",
        &base_native,
        &base_gen,
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
        if (a.arm == "joint" && a.context_learning_rate * 227.47318 <= minimum_context_margin)
            || 2. * a.prototype_learning_rate * 227.47318 <= 2.
        {
            return Err(bad("declared learning rates cannot reach an initial native context/prototype boundary within128steps"));
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
        let mut cx = if a.arm == "joint" {
            Some(adam(&context, a.context_learning_rate)?)
        } else {
            None
        };
        for update in 0..a.updates {
            deadline(a, start)?;
            let indices = (0..8)
                .map(|i| (update * 8 + i) % train.len())
                .collect::<Vec<_>>();
            let (grads, receipt) = batch(
                a, &source, &frozen, &generate, &cue, &prefix, &exp, &train, &indices, &device,
                start, None,
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
            generate.project_shadow_range()?;
            device.synchronize()?;
            updates.push(json!({"step":update+1,"before_update_batch":receipt}));
            write(a, "updates.json", &json!(updates))?;
            let step = update + 1;
            if step == a.updates {
                let (n, g, c) = checkpoint(a, step, &source, &frozen, &generate, &cue, &prefix)?;
                let e = evaluate(
                    a,
                    &format!("development-{step:04}"),
                    &n,
                    &g,
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
        let selectedfresh = evaluate(
            a,
            "fresh-selected",
            &n,
            &g,
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
        json!({"schema":"uor-r4.geometric-bank-generate-fit/1","status":"COMPLETED","mode":a.mode,"arm":a.arm,"seed":a.seed,"source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT"),"executable_sha256":sha256_file(&std::env::current_exe()?)?,"report_scope":"initial/final checkpoints only; native full512 ownprefix selection; admission8subset explicitly not full512 quality verdict","updates":if a.mode=="fit"{a.updates}else{0},"selected_step":selected,"stages":stages,"fresh":freshresult,"elapsed_seconds":start.elapsed().as_secs_f64(),"runtime_scope":"actual all-source Copy plus full legal native geometric Generate; integer CPU oracle/serving and CUDA learning; no legacy terminal mass","language_scope":"bounded framed grounded answers; not general chat/reasoning; no-source path available but not qualified if panel lacks such rows"}),
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
            json!({"schema":"uor-r4.geometric-bank-generate-fit/1","status":"FAILED","error":e.to_string(),"elapsed_seconds":start.elapsed().as_secs_f64(),"model_verdict":"UNQUALIFIED; preserve completed rows/checkpoints; unfinished fit is not model failure"})
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
}
