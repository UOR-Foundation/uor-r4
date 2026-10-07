//! Frozen first-entry route capacity; all routes precede evaluator labels.
//! Labels are evaluator-only; native generation sees an owned complete bank.
use serde::Deserialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::io::Write;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};
use uor_r4_core::native_geometric::learner::{
    geometric_generate::{GenerateReadCounts, NativeGeometricGenerate},
    geometric_read_state_bridge::{BridgeReadCounts, NativeGeometricReadStateBridge},
};
use uor_r4_core::{
    answer_oracle::FrozenAnswers,
    native_geometric::learner::native_bank_generate::{
        BankPin, BoundNativeBytes, NativeBankArtifacts, NativeBankGenerateStep,
        NativeBankGenerator, OwnedBankSegment, OwnedBankSource, PinnedBankSnapshot,
        SnapshotSourceStatus,
    },
    report_output,
};
use uor_r4_integer::{
    geometric_cue_carrier::CueCarrierMetadata, geometric_prefix_transport::PrefixTransportMetadata,
    geometric_source_realizer::NativeArtifactBinding,
};
use uor_r4_integer::{
    geometric_vocabulary_actions::{NativeVocabularyActions, VocabularyReduction},
    h4_tables::H4Code,
};
#[path = "../../uor-r4-integer/examples/support/source_probe.rs"]
mod output_support;
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
use uor_r4_tokenizer::ByteBpeTokenizer;
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Config {
    baseline_inputs: PathBuf,
    expected_baseline_inputs_sha256: String,
    model_root: PathBuf,
    expected_model_report_sha256: String,
    expected_model_manifest_sha256: String,
    inputs: PathBuf,
    expected_inputs_sha256: String,
    labels: PathBuf,
    expected_labels_sha256: String,
    output: PathBuf,
    #[serde(default = "default_report_bytes")]
    maximum_report_bytes: u64,
}
fn default_report_bytes() -> u64 {
    // Full traces for 32 cases at the 32-token limit can exceed 2 GiB.
    128 << 20
}
fn write_row(c: &Config, name: &str, row: &Value) -> Result<()> {
    let used = fs::read_dir(&c.output)?.try_fold(0u64, |sum, e| -> Result<u64> {
        Ok(sum
            .checked_add(e?.metadata()?.len())
            .ok_or_else(|| bad("output size overflow"))?)
    })?;
    let bytes = serde_json::to_vec_pretty(row)?;
    if used.saturating_add(bytes.len() as u64) > c.maximum_report_bytes - (1 << 20) {
        return Err(bad(
            "declared report storage boundary reached; no rows truncated",
        ));
    }
    Ok(fs::write(c.output.join(name), bytes)?)
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Panel {
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
    #[serde(default)]
    pair_id: Option<String>,
}
fn bad(s: &str) -> Box<dyn std::error::Error> {
    std::io::Error::other(s).into()
}
fn bytes(p: &Path) -> Result<Vec<u8>> {
    Ok(fs::read(p)?)
}
fn hash(b: &[u8]) -> String {
    hex::encode(Sha256::digest(b))
}
fn file_hash(p: &Path) -> Result<String> {
    Ok(hash(&bytes(p)?))
}
fn read(p: &Path) -> Result<Value> {
    Ok(serde_json::from_slice(&bytes(p)?)?)
}
fn write(p: &Path, v: &Value) -> Result<()> {
    Ok(fs::write(p, serde_json::to_vec_pretty(v)?)?)
}
fn expected_string(v: &Value) -> Result<&str> {
    v.as_str()
        .ok_or_else(|| bad("expected digest/string absent"))
}
fn seal_for(path: &Path) -> Result<PathBuf> {
    path.ancestors()
        .find(|p| p.join("manifest.json").is_file())
        .map(Path::to_path_buf)
        .ok_or_else(|| bad("input has no enclosing report seal"))
}
fn snapshot(p: Packet) -> Result<PinnedBankSnapshot> {
    if !p.actual_prefix_ids.is_empty() {
        return Err(bad("generation must start with empty actual prefix"));
    }
    let mut scope = None;
    let mut commit = 0;
    for s in &p.segments {
        if let Segment::Source {
            scope: q,
            commit: c,
            ..
        } = s
        {
            if q.is_empty() || scope.as_ref().is_some_and(|old| old != q) {
                return Err(bad("panel bank has empty/mixed Source scope"));
            }
            scope = Some(q.clone());
            commit = commit.max(*c);
        }
    }
    let scope = scope.ok_or_else(|| bad("Source substitution panel requires Source records"))?;
    Ok(PinnedBankSnapshot {
        pin: BankPin {
            lineage: 0,
            commit,
            scope: scope.into_bytes(),
        },
        query_ids: p.query_ids,
        segments: p
            .segments
            .into_iter()
            .map(|s| match s {
                Segment::Source {
                    event,
                    record,
                    commit,
                    scope,
                    entity,
                    relation,
                    view,
                    original_source_ids,
                } => OwnedBankSegment::Source(OwnedBankSource {
                    event,
                    record,
                    commit,
                    scope: scope.into_bytes(),
                    entity,
                    relation,
                    view,
                    status: SnapshotSourceStatus::Found,
                    original_token_ids: original_source_ids,
                }),
                Segment::Context {
                    event,
                    role,
                    token_ids,
                } => OwnedBankSegment::Context {
                    event,
                    role,
                    token_ids,
                },
            })
            .collect(),
    })
}
fn compare_step(actual: &NativeBankGenerateStep, expected: &Value) -> Result<()> {
    if expected["actual_prefix_ids"] != json!(actual.actual_prefix_ids)
        || expected["copy_token_ids"] != json!(actual.copy_token_ids)
        || expected["copy_raw_scores_q24"] != json!(actual.copy_raw_scores_q24)
        || expected["retained_state_codes"]
            != json!(actual
                .post_state
                .iter()
                .map(|c| c.index())
                .collect::<Vec<_>>())
        || expected["generate_raw_scores_sha256"]
            != hash(&serde_json::to_vec(&actual.generate_raw_scores_q24)?)
        || expected["pool"]["summary"] != serde_json::to_value(&actual.actions.summary)?
        || expected["generate_costs"] != serde_json::to_value(&actual.generate_counts)?
    {
        return Err(bad(
            "native prefix/Copy/poststate/Generate/reducer summary parity differs",
        ));
    }
    if let Some(trace) = &actual.bank_trace {
        let bank = &trace.cue_bank.bank;
        if expected["source_provenance"]["candidates"] != serde_json::to_value(&bank.candidates)?
            || expected["source_provenance"]["causal_tokens"] != json!(bank.context.tokens)
            || expected["source_provenance"]["legacy_terminal_actions_discarded"] != true
        {
            return Err(bad("full occurrence chronology/provenance parity differs"));
        }
        let count = actual.copy_token_ids.len();
        let sum = |heads: &[Vec<i64>]| -> Result<Vec<i64>> {
            let mut result = vec![0i64; count];
            for head in heads {
                if head.len() != count {
                    return Err(bad("component candidate count differs"));
                }
                for (i, &score) in head.iter().enumerate() {
                    result[i] = result[i]
                        .checked_add(score)
                        .ok_or_else(|| bad("component overflow"))?;
                }
            }
            Ok(result)
        };
        let cue = sum(&trace.cue_bank.carrier.copy_q24)?;
        let prefix = sum(&trace.prefix.copy_q24)?;
        let context = actual
            .copy_raw_scores_q24
            .iter()
            .enumerate()
            .map(|(i, &score)| {
                score
                    .checked_sub(cue[i])
                    .and_then(|v| v.checked_sub(prefix[i]))
                    .ok_or_else(|| bad("attribution overflow"))
            })
            .collect::<Result<Vec<_>>>()?;
        if expected["source_provenance"]["copy_components_q24"]
            != json!({"cue":cue,"prefix":prefix,"contextual":context})
        {
            return Err(bad("Copy component attribution parity differs"));
        }
    } else if expected["source_provenance"]["no_source"] != true {
        return Err(bad("source availability parity differs"));
    }
    match &actual.bridge {
        Some(b) => {
            let saved = &expected["source_provenance"]["read_state_bridge"];
            if saved["selected_ordinal"] != json!(b.selected_ordinal)
                || saved["selected_candidate"] != serde_json::to_value(&b.selected_candidate)?
                || saved["query_state_codes"]
                    != json!(b.query_state.iter().map(|c| c.index()).collect::<Vec<_>>())
                || saved["selected_source_state_codes"]
                    != json!(b.source_state.iter().map(|c| c.index()).collect::<Vec<_>>())
                || saved["action_codes"]
                    != json!(b.action_codes.iter().map(|c| c.index()).collect::<Vec<_>>())
                || saved["action_scores_q24"] != json!(b.action_scores_q24)
                || saved["native_costs"] != serde_json::to_value(&b.counts)?
            {
                return Err(bad("raw-selected categorical bridge parity differs"));
            }
        }
        None if !expected["source_provenance"]["read_state_bridge"].is_null() => {
            return Err(bad("unexpected saved bridge"))
        }
        None => {}
    }
    Ok(())
}
#[derive(serde::Serialize)]
struct RouteFrame {
    cache_id: usize,
    is_query: bool,
    candidate: Value,
    source_state: Vec<u8>,
    action_codes: Vec<u8>,
    action_scores_sha256: Option<String>,
    factual_selected: bool,
    generate_summary: VocabularyReduction,
    full_summary: VocabularyReduction,
}
#[derive(serde::Serialize)]
struct RowFrames {
    id: String,
    query_state: Vec<u8>,
    copy_ids: Vec<u32>,
    copy_scores: Vec<i64>,
    routes: Vec<RouteFrame>,
}
struct CachedScores {
    state: Vec<u8>,
    scores: Vec<i64>,
    bytes_sha256: String,
    offset: u64,
}
fn codes(x: &[H4Code]) -> Vec<u8> {
    x.iter().map(|x| x.index()).collect()
}
fn h4(x: &[u8]) -> Result<Vec<H4Code>> {
    Ok(x.iter()
        .copied()
        .map(H4Code::try_from)
        .collect::<std::result::Result<Vec<_>, _>>()?)
}
fn run(c: &Config) -> Result<Value> {
    report_output::verify(&c.model_root)?;
    if file_hash(&c.model_root.join("report.json"))? != c.expected_model_report_sha256
        || file_hash(&c.model_root.join("manifest.json"))? != c.expected_model_manifest_sha256
    {
        return Err(bad("frozen model report/seal identity differs"));
    }
    let r = read(&c.model_root.join("report.json"))?;
    if r["schema"] != "uor-r4.geometric-prediction-control/1"
        || r["status"] != "COMPLETED"
        || r["updates"] != 0
        || r["native_prediction_control_win"] != true
        || r["resume"]["component_recomposition"] != true
        || r["resume"]["source_step"] != 48
        || r["resume"]["generate_step"] != 64
        || r["resume"]["producer_source_commit"] != "b451571aa638d1f93e3ccf25f718f27ed2ea731f"
    {
        return Err(bad(
            "model is not the completed successful zero-update48/64 recomposition",
        ));
    }
    let cp = c.model_root.join("checkpoint-0000");
    let receipt = read(&cp.join("receipt.json"))?;
    if receipt != r["final_receipt"]
        || receipt != r["initial_receipt"]
        || receipt["step"] != 0
        || receipt["native_independently_reloaded"] != true
        || receipt["masters_independently_reloaded"] != true
    {
        return Err(bad("coherent frozen native checkpoint receipt differs"));
    }
    // This is an external authored panel, not a store enumeration or lineage proof.
    let input_root = seal_for(&c.inputs)?;
    let label_root = seal_for(&c.labels)?;
    report_output::verify(&input_root)?;
    report_output::verify(&label_root)?;
    if file_hash(&c.inputs)? != c.expected_inputs_sha256
        || file_hash(&c.labels)? != c.expected_labels_sha256
    {
        return Err(bad("panel identity differs"));
    }
    let panel: Panel = serde_json::from_slice(&bytes(&c.inputs)?)?;
    if panel.schema != "uor-r4.native-source-bank-probe-input/1" || panel.cases.len() != 32 {
        return Err(bad("capacity requires the complete fresh32 panel"));
    }
    let binding: NativeArtifactBinding = serde_json::from_value(receipt["parent"].clone())?;
    let gen = bytes(&cp.join("generate.bin"))?;
    let bridge = bytes(&cp.join("read-state-bridge-categorical.bin"))?;
    let exp = bytes(&cp.join("native/consumer/exp-q31.bin"))?;
    let exp_hash = hash(&exp);
    let cue_metadata: CueCarrierMetadata =
        serde_json::from_value(read(&cp.join("cue/native-metadata.json"))?)?;
    let prefix_metadata: PrefixTransportMetadata =
        serde_json::from_value(read(&cp.join("prefix/native-metadata.json"))?)?;
    let cue = bytes(&cp.join("cue/cue-q4.bin"))?;
    let prefix = bytes(&cp.join("prefix/prefix-q4.bin"))?;
    let joint = if cp.join("cue/cue-joint-q4.bin").is_file() {
        Some(bytes(&cp.join("cue/cue-joint-q4.bin"))?)
    } else {
        None
    };
    let native_dir = cp.join("native");
    let load_generator = |with_bridge| -> Result<NativeBankGenerator> {
        Ok(NativeBankGenerator::load(NativeBankArtifacts {
            native_directory: &native_dir,
            source_binding: &binding,
            generate: BoundNativeBytes {
                bytes: &gen,
                sha256: expected_string(&receipt["generate_sha256"])?,
            },
            bridge: if with_bridge {
                Some(BoundNativeBytes {
                    bytes: &bridge,
                    sha256: expected_string(&receipt["categorical_sha256"])?,
                })
            } else {
                None
            },
            cue_packed: &cue,
            cue_joint_packed: joint.as_deref(),
            cue_metadata: &cue_metadata,
            prefix_packed: &prefix,
            prefix_metadata: &prefix_metadata,
            exp: BoundNativeBytes {
                bytes: &exp,
                sha256: &exp_hash,
            },
        })?)
    };
    let mut g = load_generator(true)?;
    let mut query_generator = load_generator(false)?;
    let tok =
        ByteBpeTokenizer::from_tokenizer_json_bytes(&bytes(&native_dir.join("tokenizer.json"))?)
            .ok_or_else(|| bad("ByteBPE unavailable"))?;
    let integer = uor_r4_integer::geometric_source_realizer::NativeSourceRealizer::load_native(
        &native_dir,
        &binding,
    )?;
    let native_generate = NativeGeometricGenerate::from_bytes(&gen, integer.binding())?;
    let native_bridge = NativeGeometricReadStateBridge::from_bytes(&bridge, integer.binding())?;
    if native_generate.vocab_size() != 4096 {
        return Err(bad("expected full4096 vocabulary"));
    }
    let mut pool = NativeVocabularyActions::new(integer.binding().clone(), &exp)?;
    let saved = read(&c.model_root.join("prediction-0000.json"))?;
    let saved_refs = saved["rows"]
        .as_array()
        .ok_or_else(|| bad("model baseline rows absent"))?;
    let baseline_root = seal_for(&c.baseline_inputs)?;
    report_output::verify(&baseline_root)?;
    if file_hash(&c.baseline_inputs)? != c.expected_baseline_inputs_sha256 {
        return Err(bad("original input identity differs"));
    }
    let original: Panel = serde_json::from_slice(&bytes(&c.baseline_inputs)?)?;
    if original.cases.len() != 8
        || saved_refs.len() != 8
        || original.schema != "uor-r4.native-source-bank-probe-input/1"
    {
        return Err(bad("original eight baseline coverage differs"));
    }
    for (p, reference) in original.cases.into_iter().zip(saved_refs) {
        if reference["id"] != p.id {
            return Err(bad("original input order differs"));
        }
        let name = expected_string(&reference["row_file"])?;
        if Path::new(name)
            .components()
            .any(|p| !matches!(p, std::path::Component::Normal(_)))
        {
            return Err(bad("unsafe saved row path"));
        }
        let row_path = c.model_root.join(name);
        if file_hash(&row_path)? != reference["row_sha256"] {
            return Err(bad("original saved row hash differs"));
        }
        let row = read(&row_path)?;
        let bank = g.admit_bank(snapshot(p)?)?;
        let factual = g.step(&bank, &[])?;
        compare_step(&factual, &row["generation"][0])?;
    }
    let mut cache = Vec::<CachedScores>::new();
    let mut cache_ids = BTreeMap::<Vec<u8>, usize>::new();
    let mut stream = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(c.output.join("generate-i64le.bin"))?;
    let mut phase1 = Vec::<RowFrames>::new();
    let mut seen = BTreeSet::new();
    // Enumerate every physical candidate and query control across ALL fresh32
    // before deserializing or inspecting any answer/target label.
    for p in panel.cases {
        if p.id.is_empty() || !seen.insert(p.id.clone()) {
            return Err(bad("empty/duplicate caseID"));
        }
        let id = p.id.clone();
        let bank = g.admit_bank(snapshot(p)?)?;
        let factual = g.step(&bank, &[])?;
        let trace = factual
            .bank_trace
            .as_ref()
            .ok_or_else(|| bad("factual Source trace absent"))?;
        let b = &trace.cue_bank.bank;
        let witness = factual
            .bridge
            .as_ref()
            .ok_or_else(|| bad("factual bridge absent"))?;
        let query = witness.query_state.clone();
        let query_step = query_generator.step(&bank, &[])?;
        if query_step.post_state != query
            || query_step.bridge.is_some()
            || query_step.copy_token_ids != factual.copy_token_ids
            || query_step.copy_raw_scores_q24 != factual.copy_raw_scores_q24
            || query_step.bank_trace != factual.bank_trace
        {
            return Err(bad(
                "global query control changed complete factual Source/Copy bank",
            ));
        }
        let actual_query = h4(b
            .context
            .states
            .last()
            .ok_or_else(|| bad("query cumulative state absent"))?)?;
        if query != actual_query || b.candidates.len() != factual.copy_token_ids.len() {
            return Err(bad("physical bank/query carrier mismatch"));
        }
        let mut routes = Vec::new();
        for ordinal in 0..=b.candidates.len() {
            let is_query = ordinal == b.candidates.len();
            let mut post = query.clone();
            let mut actions = vec![H4Code::IDENTITY; query.len()];
            let mut action_scores = vec![0i64; query.len() * 120];
            let mut counts = BridgeReadCounts::default();
            let (source, candidate) = if is_query {
                (Vec::new(), Value::Null)
            } else {
                let candidate = &b.candidates[ordinal];
                let key = h4(b
                    .context
                    .states
                    .get(candidate.context_position)
                    .ok_or_else(|| bad("physical cumulative Source state absent"))?)?;
                native_bridge.apply_into(
                    &query,
                    &key,
                    &mut post,
                    &mut actions,
                    &mut action_scores,
                    &mut counts,
                )?;
                (key, serde_json::to_value(candidate)?)
            };
            let state = codes(&post);
            let cache_id = if let Some(&i) = cache_ids.get(&state) {
                i
            } else {
                let mut scores = vec![0i64; 4096];
                native_generate.score_into(
                    &post,
                    &mut scores,
                    &mut GenerateReadCounts::default(),
                )?;
                let bytes = scores
                    .iter()
                    .flat_map(|v| v.to_le_bytes())
                    .collect::<Vec<_>>();
                let used = fs::read_dir(&c.output)?.try_fold(0u64, |sum, e| -> Result<u64> {
                    Ok(sum + e?.metadata()?.len())
                })?;
                if used + bytes.len() as u64 > c.maximum_report_bytes - (1 << 20) {
                    return Err(bad(
                        "capacity score cache storage boundary; no routes skipped",
                    ));
                }
                stream.write_all(&bytes)?;
                let i = cache.len();
                cache_ids.insert(state.clone(), i);
                cache.push(CachedScores {
                    state: state.clone(),
                    scores,
                    bytes_sha256: hash(&bytes),
                    offset: i as u64 * 32768,
                });
                i
            };
            let scored = &cache[cache_id].scores;
            let mixed = pool.reduce_trace(
                scored,
                &factual.copy_token_ids,
                &factual.copy_raw_scores_q24,
            )?;
            let only = pool.reduce_trace(scored, &[], &[])?;
            if is_query
                && (*scored != query_step.generate_raw_scores_q24 || mixed != query_step.actions)
            {
                return Err(bad(
                    "query control does not match actual bridgeNone native path",
                ));
            }
            let selected = !is_query && ordinal == witness.selected_ordinal;
            if selected
                && (post != factual.post_state
                    || *scored != factual.generate_raw_scores_q24
                    || mixed != factual.actions)
            {
                return Err(bad("factual allGenerate/fullalias route replay differs"));
            }
            routes.push(RouteFrame {
                cache_id,
                is_query,
                candidate,
                source_state: codes(&source),
                action_codes: if is_query {
                    Vec::new()
                } else {
                    codes(&actions)
                },
                action_scores_sha256: if is_query {
                    None
                } else {
                    Some(hash(&serde_json::to_vec(&action_scores)?))
                },
                factual_selected: selected,
                generate_summary: only.summary,
                full_summary: mixed.summary,
            });
        }
        if routes.iter().filter(|r| r.factual_selected).count() != 1 {
            return Err(bad("exactly one factual physical Source route required"));
        }
        phase1.push(RowFrames {
            id,
            query_state: codes(&query),
            copy_ids: factual.copy_token_ids,
            copy_scores: factual.copy_raw_scores_q24,
            routes,
        });
    }
    stream.flush()?;
    drop(stream);
    let phase1_value = serde_json::to_value(&phase1)?;
    write_row(c, "phase1-target-blind.json", &phase1_value)?;
    let cache_index=cache.iter().enumerate().map(|(id,x)|Ok(json!({"cache_id":id,"state_codes":x.state,"offset":x.offset,"length_bytes":32768,"i64le_sha256":x.bytes_sha256,
        "generate_json_sha256":hash(&serde_json::to_vec(&x.scores)?)}))).collect::<Result<Vec<_>>>()?;
    write_row(
        c,
        "cache-index.json",
        &json!({"artifact_generate_sha256":g.generate_sha256(),"score_file":"generate-i64le.bin","score_file_sha256":file_hash(&c.output.join("generate-i64le.bin"))?,"entries":cache_index}),
    )?;
    let labels: Labels = serde_json::from_slice(&bytes(&c.labels)?)?;
    if labels.schema != "uor-r4.native-source-bank-labels/1"
        || labels.protocol != "uor-r4.literal-role-dialogue/2"
        || !labels.membership_only
        || labels.cases.len() != phase1.len()
    {
        return Err(bad("complete evaluator labels differ"));
    }
    let mut rows = Vec::new();
    let mut gen_reachable = 0;
    let mut mixed_reachable = 0;
    let mut source_routes = 0;
    for (index, (frames, label)) in phase1.iter().zip(labels.cases).enumerate() {
        if frames.id != label.id {
            return Err(bad("phase2 label rowID mismatch"));
        }
        label.answers.validate()?;
        let target = tok.encode(&label.answers.accepted[0]);
        if tok.decode_bytes(&target) != label.answers.accepted[0].as_bytes() || target.is_empty() {
            return Err(bad("entry label roundtrip differs"));
        }
        let gold = target[0];
        let mut routes = Vec::new();
        let mut source_gen = false;
        let mut query_gen = false;
        let mut source_mixed = false;
        let mut query_mixed = false;
        for r in &frames.routes {
            let scores = &cache[r.cache_id].scores;
            let mixed = pool.reduce_trace(scores, &frames.copy_ids, &frames.copy_scores)?;
            let only = pool.reduce_trace(scores, &[], &[])?;
            if mixed.summary != r.full_summary || only.summary != r.generate_summary {
                return Err(bad("phase2 reconstructed alias pool differs"));
            }
            let m = mixed
                .token_masses
                .iter()
                .find(|x| x.token_id == gold)
                .ok_or_else(|| bad("gold legal token absent"))?;
            let only_gold = only
                .token_masses
                .iter()
                .find(|x| x.token_id == gold)
                .ok_or_else(|| bad("Generate-only gold absent"))?;
            let gen_win = only.summary.chosen_token_id == gold;
            let full_win = mixed.summary.chosen_token_id == gold;
            if r.is_query {
                query_gen = gen_win;
                query_mixed = full_win;
            } else {
                source_gen |= gen_win;
                source_mixed |= full_win;
                source_routes += 1;
            }
            routes.push(json!({"cache_id":r.cache_id,"query_control":r.is_query,"candidate":r.candidate,"factual_selected":r.factual_selected,
                "gold_generate_raw_q24":scores[gold as usize],"generate_only_winner":only.summary.chosen_token_id,"full_alias_winner":mixed.summary.chosen_token_id,
                "generate_gold_wins":gen_win,"full_alias_gold_wins":full_win,"generate_only_gold_mass_q31":only_gold.weight_q31,"generate_only_denominator_q31":only.summary.total_weight_q31,"gold_generate_mass_q31":m.generate_weight_q31,"gold_copy_mass_q31":m.copy_weight_q31,"gold_total_mass_q31":m.weight_q31,"denominator_q31":mixed.summary.total_weight_q31}));
        }
        gen_reachable += usize::from(source_gen || query_gen);
        mixed_reachable += usize::from(source_mixed || query_mixed);
        let v = json!({"id":frames.id,"entry_target_label_only":gold,"gold_absent_copy":!frames.copy_ids.contains(&gold),"physical_source_routes":frames.routes.len()-1,
            "any_source_generate_gold_wins":source_gen,"query_generate_gold_wins":query_gen,"any_source_full_alias_gold_wins":source_mixed,"query_full_alias_gold_wins":query_mixed,"routes":routes});
        let name = format!("capacity-row-{index:04}.json");
        write_row(c, &name, &v)?;
        rows.push(json!({"id":frames.id,"row_file":name,"row_sha256":file_hash(&c.output.join(&name))?,"row_bytes":fs::metadata(c.output.join(&name))?.len(),
            "generate_gold_reachable":source_gen||query_gen,"mixed_gold_reachable":source_mixed||query_mixed}));
    }
    Ok(
        json!({"schema":"uor-r4.native-bank-entry-capacity/1","status":"COMPLETED","source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT"),"executable_sha256":file_hash(&std::env::current_exe()?)?,
        "model_report_sha256":c.expected_model_report_sha256,"model_manifest_sha256":c.expected_model_manifest_sha256,"component_provenance":r["resume"],"checkpoint_receipt_sha256":file_hash(&cp.join("receipt.json"))?,
        "inputs_sha256":c.expected_inputs_sha256,"labels_sha256":c.expected_labels_sha256,"baseline_inputs_sha256":c.expected_baseline_inputs_sha256,"baseline_firststep_parity_rows":8,
        "config_sha256":file_hash(&c.output.join("config.json"))?,"generate_sha256":g.generate_sha256(),"bridge_sha256":g.bridge_sha256(),"source_binding":g.source_binding(),
        "cases":rows.len(),"physical_source_routes":source_routes,"query_controls":rows.len(),"unique_native_states":cache.len(),"score_cache_bytes":cache.len()*32768,
        "rows_with_generate_gold_reachable":gen_reachable,"rows_with_full_alias_gold_reachable":mixed_reachable,"rows":rows,
        "query_control_scope":"global bridgeNone configuration supported; learned per-step query-vs-Source choice is unintegrated","phase1_sha256":file_hash(&c.output.join("phase1-target-blind.json"))?,"cache_index_sha256":file_hash(&c.output.join("cache-index.json"))?,
        "scope":"first-entry exhaustive physical-source transport and untransported query endpoints under one frozen artifact; Copy bank/scores held factual; diagnostic potential upper bound, not integrated route policy or impossibility beyond these states; no training/future teacher prefix/chat claim"}),
    )
}
fn admit_paths(c: &mut Config) -> Result<()> {
    if c.maximum_report_bytes < 8 << 20 || c.maximum_report_bytes > 4 << 30 {
        return Err(bad("report storage limit must be8MiB..4GiB"));
    }
    for p in [
        &c.model_root,
        &c.inputs,
        &c.labels,
        &c.baseline_inputs,
        &c.output,
    ] {
        if !p.is_absolute()
            || p.components()
                .any(|v| matches!(v, std::path::Component::ParentDir))
        {
            return Err(bad("absolute nontraversing paths required"));
        }
    }
    c.baseline_inputs = fs::canonicalize(&c.baseline_inputs)?;
    c.model_root = fs::canonicalize(&c.model_root)?;
    c.inputs = fs::canonicalize(&c.inputs)?;
    c.labels = fs::canonicalize(&c.labels)?;
    c.output = output_support::prospective_output(&c.output)?;
    for p in [&c.model_root, &c.inputs, &c.labels, &c.baseline_inputs] {
        let seal = seal_for(p)?;
        if c.output.starts_with(p) || p.starts_with(&c.output) || c.output.starts_with(seal) {
            return Err(bad("output/input or sealed ancestry overlap"));
        }
    }
    Ok(())
}
fn main() -> Result<()> {
    let args = std::env::args().collect::<Vec<_>>();
    if args.len() == 3 && args[1] == "verify-report" {
        report_output::verify(Path::new(&args[2]))?;
        return Ok(());
    }
    if args.len() != 2 {
        return Err(bad("usage: native-bank-entry-capacity CONFIG.json"));
    }
    let raw = bytes(Path::new(&args[1]))?;
    let mut c: Config = serde_json::from_slice(&raw)?;
    admit_paths(&mut c)?;
    report_output::claim(&c.output)?;
    write(
        &c.output.join("config.json"),
        &serde_json::from_slice(&raw)?,
    )?;
    let result = run(&c);
    let report = match &result {
        Ok(v) => v.clone(),
        Err(e) => {
            json!({"schema":"uor-r4.native-bank-entry-capacity/1","status":"FAILED","error":e.to_string(),"scope":"execution/instrument failure; no model-quality verdict"})
        }
    };
    write(&c.output.join("report.json"), &report)?;
    report_output::seal(&c.output)?;
    result.map(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn packet() -> Value {
        json!({"id":"fixture","actual_prefix_ids":[],"query_ids":[7],"segments":[
        {"kind":"Context","event":1,"role":1,"token_ids":[4]},
        {"kind":"Source","event":2,"record":9,"commit":3,"scope":"fixture-scope","entity":[4],"relation":2,"view":1,"original_source_ids":[5]}]})
    }
    #[test]
    fn snapshot_preserves_source_occurrences_and_query_with_metadata_only_pin() -> Result<()> {
        let input: Packet = serde_json::from_value(packet())?;
        let bank = snapshot(input)?;
        assert_eq!(bank.query_ids, vec![7]);
        assert_eq!(bank.pin.commit, 3);
        assert_eq!(bank.pin.scope, b"fixture-scope");
        match &bank.segments[1] {
            OwnedBankSegment::Source(s) => {
                assert_eq!(s.original_token_ids, vec![5]);
                assert_eq!(s.record, 9);
            }
            _ => return Err(bad("source absent")),
        }
        let mut malicious = packet();
        malicious["selected_record"] = json!(9);
        assert!(serde_json::from_value::<Packet>(malicious).is_err());
        let mut prefix = packet();
        prefix["actual_prefix_ids"] = json!([2997]);
        assert!(snapshot(serde_json::from_value(prefix)?).is_err());
        Ok(())
    }
    #[test]
    fn different_evaluator_answers_cannot_change_native_snapshot_inputs() -> Result<()> {
        let mut first =
            json!({"id":"fixture","answers":{"intent":"current","accepted":[" first."]}});
        let a: Label = serde_json::from_value(first.clone())?;
        a.answers.validate()?;
        first["answers"]["accepted"] = json!([" second."]);
        let b: Label = serde_json::from_value(first)?;
        b.answers.validate()?;
        assert_ne!(a.answers, b.answers);
        let x = snapshot(serde_json::from_value(packet())?)?;
        let y = snapshot(serde_json::from_value(packet())?)?;
        assert_eq!(x.pin, y.pin);
        assert_eq!(x.query_ids, y.query_ids);
        assert_eq!(x.segments.len(), y.segments.len());
        // Packet deserialization rejects evaluator labels in the runtime input.
        let mut p = packet();
        p["answers"] = json!([" second."]);
        assert!(serde_json::from_value::<Packet>(p).is_err());
        Ok(())
    }
    #[test]
    fn snapshot_rejects_mixed_scope_and_missing_source() -> Result<()> {
        let mut p = packet();
        let mut other = p["segments"][1].clone();
        other["scope"] = json!("other");
        p["segments"]
            .as_array_mut()
            .ok_or_else(|| bad("fixture segments"))?
            .push(other);
        assert!(snapshot(serde_json::from_value(p)?).is_err());
        let mut p = packet();
        p["segments"] = json!([{ "kind":"Context","event":1,"role":1,"token_ids":[4]}]);
        assert!(snapshot(serde_json::from_value(p)?).is_err());
        Ok(())
    }
}
