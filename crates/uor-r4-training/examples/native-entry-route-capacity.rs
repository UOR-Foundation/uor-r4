#![recursion_limit = "256"]
//! Offline trained-checkpoint entry route capacity. No optimizer or runtime selector.
use candle_core::Device;
use serde::Deserialize;
use std::time::Instant;
use uor_r4_training::geometric_generate_learning::GenerateLearningWeights;
#[path = "native-entry-route-capacity/certificate.rs"]
mod certificate;
#[path = "native-entry-route-capacity/score_codec.rs"]
mod score_codec;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::{Path, PathBuf},
};
use uor_r4_core::{
    native_geometric::learner::native_bank_generate::{
        BankPin, BoundNativeBytes, NativeBankArtifacts, NativeBankGenerateStep,
        NativeBankGenerator, OwnedBankSegment, OwnedBankSource, PinnedBankSnapshot,
        SnapshotSourceStatus,
    },
    native_geometric::learner::{
        geometric_generate::{GenerateReadCounts, NativeGeometricGenerate},
        geometric_read_state_bridge::{BridgeReadCounts, NativeGeometricReadStateBridge},
    },
    report_output,
};
use uor_r4_integer::{
    geometric_cue_carrier::CueCarrierMetadata, geometric_prefix_transport::PrefixTransportMetadata,
    geometric_source_actions::SourceActionBinding,
    geometric_source_realizer::NativeArtifactBinding,
    geometric_vocabulary_actions::NativeVocabularyActions, h4_tables::H4Code,
};
#[path = "../../uor-r4-integer/examples/support/source_probe.rs"]
mod output_support;
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Config {
    baseline: PathBuf,
    expected_baseline_report_sha256: String,
    expected_checkpoint_receipt_sha256: String,
    expected_generate_sha256: String,
    inputs: PathBuf,
    output: PathBuf,
    pin_lineage: u64,
    pin_commit: u64,
    pin_scope: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Panel {
    schema: String,
    cases: Vec<Packet>,
}
#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct Packet {
    id: String,
    segments: Vec<Segment>,
    query_ids: Vec<u32>,
    actual_prefix_ids: Vec<u32>,
}
#[derive(Clone, Deserialize)]
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
fn snapshot(p: Packet, c: &Config) -> Result<PinnedBankSnapshot> {
    if !p.actual_prefix_ids.is_empty() {
        return Err(bad("panel free generation must start with empty prefix"));
    }
    Ok(PinnedBankSnapshot {
        pin: BankPin {
            lineage: c.pin_lineage,
            commit: c.pin_commit,
            scope: c.pin_scope.as_bytes().to_vec(),
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
fn compare_token_masses(
    actual: &[uor_r4_integer::geometric_vocabulary_actions::VocabularyTokenMass],
    saved: &Value,
) -> Result<()> {
    let rows = actual
        .iter()
        .map(|m| {
            [
                u64::from(m.token_id),
                m.weight_q31,
                m.generate_weight_q31,
                m.copy_weight_q31,
            ]
        })
        .collect::<Vec<_>>();
    if serde_json::to_value(rows)? != *saved {
        return Err(bad(
            "complete saved token alias masses/Generate/Copy decomposition parity differs",
        ));
    }
    Ok(())
}
fn compare_step(actual: &NativeBankGenerateStep, expected: &Value) -> Result<()> {
    compare_token_masses(
        &actual.actions.token_masses,
        &expected["pool"]["token_masses"],
    )?;
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

const PRODUCER: &str = "7fbfc940ce4abfb056e0a094bf82d1df5d9bc9ae";
const INPUT_SHA: &str = "b9661606b280884217a64e0a5b643f8324a90390e47ade7241da0889a5f7c86a";
const BRIDGE_SHA: &str = "af5039e74c6cd92e2fa0bf4c72c315ef34f912d77b09fc607215a18f86cd1d0e";
const EXP_SHA: &str = "79485d6e63cc28f5e01d98c5d73abe021db33fa7fef368142ae6592d06b4817f";
// Six complete attempts fit the prospectively extended combined 1 GiB.
// Each has 160 MiB for evidence plus a separate 1 MiB failure/seal reserve;
// no route may be omitted to satisfy admission.
const MAX_ATTEMPT_BYTES: u64 = 160 << 20;

fn checked_hash(p: &Path, expected: &str) -> Result<Vec<u8>> {
    let b = bytes(p)?;
    if hash(&b) != expected {
        return Err(bad("fixed retained artifact digest differs"));
    }
    Ok(b)
}
fn codes(c: &[H4Code]) -> Vec<u8> {
    c.iter().map(|v| v.index()).collect()
}
fn h4_codes(c: &[u8]) -> Result<Vec<H4Code>> {
    Ok(c.iter()
        .copied()
        .map(H4Code::try_from)
        .collect::<std::result::Result<Vec<_>, _>>()?)
}
fn valid_digest(s: &str) -> bool {
    s.len() == 64 && s.bytes().all(|b| b.is_ascii_hexdigit())
}
fn admit_config(c: &Config) -> Result<()> {
    if !valid_digest(&c.expected_baseline_report_sha256)
        || !valid_digest(&c.expected_checkpoint_receipt_sha256)
        || !valid_digest(&c.expected_generate_sha256)
        || c.pin_lineage != 1
        || c.pin_commit != 2
        || c.pin_scope != "compiler-probe"
    {
        return Err(bad("configuration digest/pinned bank scope differs"));
    }
    Ok(())
}
fn admit_receipt(report: &Value, receipt: &Value) -> Result<()> {
    if receipt["step"] != 128
        || receipt["native_independently_reloaded"] != true
        || receipt["masters_independently_reloaded"] != true
        || receipt["source_metadata_rebound"] != true
        || report["final_receipt"] != *receipt
    {
        return Err(bad(
            "trained final checkpoint receipt/reload equality differs",
        ));
    }
    Ok(())
}
fn output_bytes(root: &Path) -> Result<u64> {
    let mut total = 0u64;
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let m = entry.metadata()?;
        if !m.is_file() {
            return Err(bad("unexpected output directory/non-file"));
        }
        total = total
            .checked_add(m.len())
            .ok_or_else(|| bad("output size overflow"))?;
    }
    Ok(total)
}
fn write_bounded(root: &Path, name: &str, value: &Value) -> Result<()> {
    let encoded = serde_json::to_vec(value)?;
    if output_bytes(root)?
        .checked_add(encoded.len() as u64)
        .is_none_or(|n| n > MAX_ATTEMPT_BYTES)
    {
        return Err(bad(
            "complete report storage admission exceeded; no routes were intentionally skipped",
        ));
    }
    fs::write(root.join(name), encoded)?;
    Ok(())
}
struct PendingRow {
    index: usize,
    id: String,
    saved_path: PathBuf,
    saved_hash: String,
    phase1_file: String,
}
fn run(c: &Config) -> Result<Value> {
    let total_start = Instant::now();
    admit_config(c)?;
    report_output::verify(&c.baseline)?;
    checked_hash(
        &c.baseline.join("report.json"),
        &c.expected_baseline_report_sha256,
    )?;
    let report = read(&c.baseline.join("report.json"))?;
    let admission = read(&c.baseline.join("admission.json"))?;
    if report["status"] != "COMPLETED"
        || report["source_commit"] != PRODUCER
        || report["mode"] != "fit"
        || report["updates"] != 128
        || admission["source_commit"] != PRODUCER
        || admission["training_input_sha256"] != INPUT_SHA
    {
        return Err(bad(
            "trained baseline completion/producer/input identity differs",
        ));
    }
    let input_bytes = checked_hash(&c.inputs, INPUT_SHA)?;
    report_output::verify(
        c.inputs
            .parent()
            .ok_or_else(|| bad("input parent absent"))?,
    )?;
    let panel: Panel = serde_json::from_slice(&input_bytes)?;
    if panel.schema != "uor-r4.native-source-bank-probe-input/1"
        || panel.cases.len() != 512
        || panel.cases.iter().any(|p| !p.actual_prefix_ids.is_empty())
        || panel
            .cases
            .iter()
            .map(|p| p.id.as_str())
            .collect::<std::collections::BTreeSet<_>>()
            .len()
            != 512
    {
        return Err(bad(
            "complete unique empty-prefix 512-row input panel differs",
        ));
    }
    let cp = c.baseline.join("checkpoint-0128");
    let receipt: Value = serde_json::from_slice(&checked_hash(
        &cp.join("receipt.json"),
        &c.expected_checkpoint_receipt_sha256,
    )?)?;
    admit_receipt(&report, &receipt)?;
    if receipt["generate_sha256"] != c.expected_generate_sha256
        || receipt["categorical_sha256"] != BRIDGE_SHA
    {
        return Err(bad("final decoder/map receipt differs"));
    }
    // Use the UPDATED parent receipt, never the donor/original_parent binding.
    let binding: NativeArtifactBinding = serde_json::from_value(receipt["parent"].clone())?;
    let gen_bytes = checked_hash(&cp.join("generate.bin"), &c.expected_generate_sha256)?;
    let bridge_bytes = checked_hash(&cp.join("read-state-bridge-categorical.bin"), BRIDGE_SHA)?;
    let exp_bytes = checked_hash(&cp.join("native/consumer/exp-q31.bin"), EXP_SHA)?;
    if exp_bytes.len() % 4 != 0 {
        return Err(bad("exp byte alignment"));
    }
    let exp = exp_bytes
        .chunks_exact(4)
        .map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
        .collect::<Vec<_>>();
    let token_binding = SourceActionBinding::new(&bytes(&cp.join("native/tokenizer.json"))?)?;
    let native_generate = NativeGeometricGenerate::from_bytes(&gen_bytes, &token_binding)?;
    let bridge = NativeGeometricReadStateBridge::from_bytes(&bridge_bytes, &token_binding)?;
    let mut pool = NativeVocabularyActions::new(token_binding.clone(), &exp_bytes)?;
    let legal = pool.legal_token_ids().to_vec();
    let cue_metadata: CueCarrierMetadata =
        serde_json::from_value(read(&cp.join("cue/native-metadata.json"))?)?;
    let prefix_metadata: PrefixTransportMetadata =
        serde_json::from_value(read(&cp.join("prefix/native-metadata.json"))?)?;
    let cue = bytes(&cp.join("cue/cue-q4.bin"))?;
    let joint_path = cp.join("cue/cue-joint-q4.bin");
    let joint = if joint_path.exists() {
        Some(bytes(&joint_path)?)
    } else {
        None
    };
    let prefix = bytes(&cp.join("prefix/prefix-q4.bin"))?;
    let native_dir = cp.join("native");
    let artifacts = |bridge_enabled: bool| NativeBankArtifacts {
        native_directory: &native_dir,
        source_binding: &binding,
        generate: BoundNativeBytes {
            bytes: &gen_bytes,
            sha256: &c.expected_generate_sha256,
        },
        bridge: if bridge_enabled {
            Some(BoundNativeBytes {
                bytes: &bridge_bytes,
                sha256: BRIDGE_SHA,
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
            bytes: &exp_bytes,
            sha256: EXP_SHA,
        },
    };
    let mut generator = NativeBankGenerator::load(artifacts(true))?;
    // Existing global bridge=None control. This is not an integrated per-step gate.
    let mut query_generator = NativeBankGenerator::load(artifacts(false))?;
    let development = read(&c.baseline.join("development-0128.json"))?;
    let saved_rows = development["rows"]
        .as_array()
        .ok_or_else(|| bad("trained row index absent"))?;
    if saved_rows.len() != 512 {
        return Err(bad("trained row coverage incomplete"));
    }
    let device = Device::new_cuda(0)?;
    let prepare_start = Instant::now();
    let weights = GenerateLearningWeights::from_native(token_binding, &native_generate, &device)?;
    let prepared = weights.prepare_native()?;
    device.synchronize()?;
    if prepared.native.to_bytes()? != gen_bytes {
        return Err(bad("CUDA preparation changes trained decoder"));
    }
    let prepare_seconds = prepare_start.elapsed().as_secs_f64();
    let mut cache = score_codec::ScoreCache::new(
        &c.output.join("generate-scores.i16le"),
        &c.expected_generate_sha256,
        native_generate.vocab_size(),
        native_generate.lanes(),
        MAX_ATTEMPT_BYTES,
    )?;
    let mut pending = Vec::new();
    let mut physical_count = 0usize;
    let mut query_count = 0usize;
    let mut cuda_seconds = 0.;
    let mut cpu_seconds = 0.;
    let mut factual_seconds = 0.;
    let mut pool_seconds = 0.;
    let mut candidate_histogram = std::collections::BTreeMap::<usize, usize>::new();
    // Phase one: ALL512 rows and ALL physical source ordinals + one query control
    // per row precede inspection of ANY target_label_only value.
    for (index, packet) in panel.cases.into_iter().enumerate() {
        let rowref = &saved_rows[index];
        if rowref["id"] != packet.id {
            return Err(bad("ordered trained row ID differs"));
        }
        let name = expected_string(&rowref["row_file"])?;
        if Path::new(name).components().count() != 1
            || !matches!(
                Path::new(name).components().next(),
                Some(std::path::Component::Normal(_))
            )
        {
            return Err(bad("saved row must be a leaf"));
        }
        let saved_path = c.baseline.join(name);
        let saved_hash = file_hash(&saved_path)?;
        if rowref["row_sha256"] != saved_hash {
            return Err(bad("trained saved row hash differs"));
        }
        let saved = read(&saved_path)?;
        if saved["id"] != packet.id {
            return Err(bad("trained saved row ID differs"));
        }
        let expected = saved["canonical"]
            .as_array()
            .and_then(|a| a.first())
            .ok_or_else(|| bad("trained entry absent"))?;
        let prefix_ids: Vec<u32> =
            serde_json::from_value(expected["native"]["actual_prefix_ids"].clone())?;
        if !prefix_ids.is_empty() {
            return Err(bad("trained entry prefix is nonempty"));
        }
        let id = packet.id.clone();
        let bank = generator.admit_bank(snapshot(packet.clone(), c)?)?;
        let query_bank = query_generator.admit_bank(snapshot(packet, c)?)?;
        let start = Instant::now();
        let factual = generator.step(&bank, &[])?;
        compare_step(&factual, &expected["native"])?;
        let query_step = query_generator.step(&query_bank, &[])?;
        factual_seconds += start.elapsed().as_secs_f64();
        let bank_trace = &factual
            .bank_trace
            .as_ref()
            .ok_or_else(|| bad("actual source bank absent"))?
            .cue_bank
            .bank;
        let factual_bridge = factual
            .bridge
            .as_ref()
            .ok_or_else(|| bad("actual categorical witness absent"))?;
        let query = h4_codes(
            bank_trace
                .context
                .states
                .last()
                .ok_or_else(|| bad("query state absent"))?,
        )?;
        if query != factual_bridge.query_state
            || query_step.post_state != query
            || query_step.bridge.is_some()
            || query_step.copy_token_ids != factual.copy_token_ids
            || query_step.copy_raw_scores_q24 != factual.copy_raw_scores_q24
            || query_step.bank_trace != factual.bank_trace
        {
            return Err(bad(
                "query-preserving global control changed source/Copy/context bank",
            ));
        }
        *candidate_histogram
            .entry(bank_trace.candidates.len())
            .or_default() += 1;
        let mut routes = Vec::new();
        for ordinal in 0..=bank_trace.candidates.len() {
            let is_query = ordinal == bank_trace.candidates.len();
            let (source_state, post, actions, action_scores, bridge_counts) = if is_query {
                (
                    Vec::new(),
                    query.clone(),
                    Vec::new(),
                    Vec::new(),
                    BridgeReadCounts::default(),
                )
            } else {
                let candidate = &bank_trace.candidates[ordinal];
                let source = h4_codes(
                    bank_trace
                        .context
                        .states
                        .get(candidate.context_position)
                        .ok_or_else(|| bad("physical cumulative source state absent"))?,
                )?;
                let mut post = query.clone();
                let mut actions = vec![H4Code::IDENTITY; query.len()];
                let mut action_scores = vec![0; query.len() * 120];
                let mut counts = BridgeReadCounts::default();
                bridge.apply_into(
                    &query,
                    &source,
                    &mut post,
                    &mut actions,
                    &mut action_scores,
                    &mut counts,
                )?;
                (source, post, actions, action_scores, counts)
            };
            let state_codes = codes(&post);
            let cache_id = if let Some(id) = cache.lookup(&state_codes) {
                id
            } else {
                let start = Instant::now();
                let output = weights.forward_prepared_coefficients_only(&prepared, &post)?;
                device.synchronize()?;
                cuda_seconds += start.elapsed().as_secs_f64();
                if output.costs.hard_score_backend != "cuda-authenticated-i64-factors" {
                    return Err(bad("hard Generate did not use CUDA"));
                }
                let start = Instant::now();
                let mut cpu = vec![0; native_generate.vocab_size()];
                let mut counts = GenerateReadCounts::default();
                native_generate.score_into(&post, &mut cpu, &mut counts)?;
                cpu_seconds += start.elapsed().as_secs_f64();
                if cpu != output.scores_q24 {
                    return Err(bad("unique-state full CUDA/CPU integer vector mismatch"));
                }
                let additional = (cpu.len() as u64)
                    .checked_mul(2)
                    .ok_or_else(|| bad("score record size overflow"))?;
                if output_bytes(&c.output)?
                    .checked_add(additional)
                    .is_none_or(|n| n > MAX_ATTEMPT_BYTES)
                {
                    return Err(bad("exact score append storage admission exceeded; complete coverage was not truncated"));
                }
                cache.insert(&state_codes, &cpu)?
            };
            let scores = cache.read(cache_id)?;
            let start = Instant::now();
            let full = pool.reduce_trace(
                &scores,
                &factual.copy_token_ids,
                &factual.copy_raw_scores_q24,
            )?;
            pool_seconds += start.elapsed().as_secs_f64();
            let selected = !is_query && ordinal == factual_bridge.selected_ordinal;
            if selected
                && (source_state != factual_bridge.source_state
                    || post != factual.post_state
                    || actions != factual_bridge.action_codes
                    || action_scores != factual_bridge.action_scores_q24
                    || scores != factual.generate_raw_scores_q24
                    || full != factual.actions)
            {
                return Err(bad("factual physical route/full alias pool parity differs"));
            }
            if is_query
                && (scores != query_step.generate_raw_scores_q24 || full != query_step.actions)
            {
                return Err(bad("global bridge=None query/full pool parity differs"));
            }
            routes.push(json!({"kind":if is_query{"query_preserving"}else{"physical_source"},
                "ordinal":if is_query{Value::Null}else{json!(ordinal)},
                "physical_candidate":if is_query{Value::Null}else{serde_json::to_value(&bank_trace.candidates[ordinal])?},
                "is_factual_selected":selected,"query_state_codes":codes(&query),
                "source_state_codes":codes(&source_state),"post_state_codes":state_codes,
                "action_codes":codes(&actions),"action_scores_q24_sha256":hash(&serde_json::to_vec(&action_scores)?),
                "bridge_costs":bridge_counts,"score_cache_id":cache_id,
                "full_factual_copy_summary":full.summary,"unique_state_complete_cpu_cuda_parity":"PASS"}));
            if is_query {
                query_count += 1;
            } else {
                physical_count += 1;
            }
        }
        let phase1_file = format!("phase1-row-{index:04}.json");
        write_bounded(
            &c.output,
            &phase1_file,
            &json!({"index":index,"id":id,"saved_row_sha256":saved_hash,
            "position":0,"actual_prefix_ids":[],"factual_selected_ordinal":factual_bridge.selected_ordinal,
            "physical_source_count":bank_trace.candidates.len(),"factual_native_step_parity":"PASS",
            "causal_tokens":bank_trace.context.tokens,"copy_token_ids":factual.copy_token_ids,
            "copy_raw_scores_q24":factual.copy_raw_scores_q24,"routes":routes}),
        )?;
        pending.push(PendingRow {
            index,
            id,
            saved_path,
            saved_hash,
            phase1_file,
        });
    }
    if pending.len() != 512 || query_count != 512 {
        return Err(bad("complete phase-one coverage differs"));
    }
    let cache_manifest = cache.finish()?;
    write_bounded(&c.output, "score-cache-index.json", &cache_manifest)?;
    let mut rows = Vec::new();
    let mut all_source_excluded = 0usize;
    let mut query_certified = 0usize;
    let mut query_generate_wins = 0usize;
    let mut query_full_wins = 0usize;
    let mut factual_entry_wins = 0usize;
    let mut target_histogram = std::collections::BTreeMap::<u32, usize>::new();
    // Phase two: labels affect only target mass/rank/certificate analysis.
    for p in pending {
        if file_hash(&p.saved_path)? != p.saved_hash {
            return Err(bad("trained label record changed"));
        }
        let saved = read(&p.saved_path)?;
        let target: u32 =
            serde_json::from_value(saved["canonical"][0]["target_label_only"].clone())?;
        *target_histogram.entry(target).or_default() += 1;
        let phase1 = read(&c.output.join(&p.phase1_file))?;
        let copy_ids: Vec<u32> = serde_json::from_value(phase1["copy_token_ids"].clone())?;
        let copy_scores: Vec<i64> = serde_json::from_value(phase1["copy_raw_scores_q24"].clone())?;
        let in_copy = copy_ids.contains(&target);
        let mut routes = Vec::new();
        let mut excluded = 0usize;
        let mut source_routes = 0usize;
        let mut query_analysis = Value::Null;
        for original in phase1["routes"]
            .as_array()
            .ok_or_else(|| bad("phase-one routes absent"))?
        {
            let cache_id = original["score_cache_id"]
                .as_u64()
                .ok_or_else(|| bad("cache ID absent"))? as usize;
            let record = cache.record(cache_id)?;
            if original["post_state_codes"] != json!(record.post_state_codes) {
                return Err(bad("route/cache state differs"));
            }
            let scores = cache.read(cache_id)?;
            let start = Instant::now();
            let full = pool.reduce_trace(&scores, &copy_ids, &copy_scores)?;
            let generate_only = pool.reduce_trace(&scores, &[], &[])?;
            pool_seconds += start.elapsed().as_secs_f64();
            if serde_json::to_value(&full.summary)? != original["full_factual_copy_summary"] {
                return Err(bad(
                    "phase-two full pool differs from target-free phase one",
                ));
            }
            let full_target = full
                .token_masses
                .iter()
                .find(|m| m.token_id == target)
                .ok_or_else(|| bad("target illegal"))?;
            let gen_target = generate_only
                .token_masses
                .iter()
                .find(|m| m.token_id == target)
                .ok_or_else(|| bad("Generate target absent"))?;
            let cert = if in_copy {
                None
            } else {
                certificate::certify_generate_dominance(&scores, &legal, target, &exp)?
            };
            let has_certificate = cert.is_some();
            let mut evidence = original.clone();
            evidence["score_record"] = serde_json::to_value(record)?;
            evidence["target_generate_raw_score_q24"] = json!(scores[target as usize]);
            evidence["full_factual_copy_target_mass"] = serde_json::to_value(full_target)?;
            evidence["full_factual_copy_denominator"] = json!(full.summary.total_weight_q31);
            evidence["generate_only_target_mass"] = serde_json::to_value(gen_target)?;
            evidence["generate_only_denominator"] = json!(generate_only.summary.total_weight_q31);
            evidence["generate_only_winner"] = json!(generate_only.summary.chosen_token_id);
            evidence["full_factual_copy_winner"] = json!(full.summary.chosen_token_id);
            evidence["entry_exclusion_certificate"] = serde_json::to_value(cert)?;
            evidence["certificate_applicable"] = json!(!in_copy);
            if original["kind"] == "physical_source" {
                source_routes += 1;
                if has_certificate {
                    excluded += 1;
                }
                if original["is_factual_selected"] == true && full.summary.chosen_token_id == target
                {
                    factual_entry_wins += 1;
                }
            } else if original["kind"] == "query_preserving" {
                query_certified += usize::from(has_certificate);
                query_generate_wins += usize::from(generate_only.summary.chosen_token_id == target);
                query_full_wins += usize::from(full.summary.chosen_token_id == target);
                query_analysis = json!({"certificate":evidence["entry_exclusion_certificate"],
                    "generate_only_winner":generate_only.summary.chosen_token_id,
                    "full_factual_copy_winner":full.summary.chosen_token_id,
                    "target_generate_only_wins":generate_only.summary.chosen_token_id==target,
                    "target_full_factual_copy_wins":full.summary.chosen_token_id==target,
                    "integration":"global bridge=None validated; per-step learned choice UNINTEGRATED"});
            } else {
                return Err(bad("unknown retained route kind"));
            }
            routes.push(evidence);
        }
        if source_routes
            != phase1["physical_source_count"]
                .as_u64()
                .ok_or_else(|| bad("source count absent"))? as usize
            || query_analysis.is_null()
        {
            return Err(bad("phase-two physical/query coverage differs"));
        }
        let source_all_excluded = source_routes > 0 && excluded == source_routes;
        all_source_excluded += usize::from(source_all_excluded);
        let row_file = format!("row-{index:04}.json", index = p.index);
        let row = json!({"index":p.index,"id":p.id,"saved_row_sha256":p.saved_hash,"position":0,
            "target_label_only":target,"target_in_copy":in_copy,"physical_source_routes":source_routes,
            "source_routes_certified_excluded":excluded,"source_all_certified_excluded":source_all_excluded,
            "query_preserving":query_analysis,"phase1_file":p.phase1_file,"routes":routes,
            "scope":"all physical sources at fixed trained entry context; arbitrary-source upper bound, not shared-Potential reachability; full-pool counterfactuals retain factual Copy scores"});
        write_bounded(&c.output, &row_file, &row)?;
        rows.push(json!({"index":p.index,"id":row["id"],"row_file":row_file,
            "row_sha256":file_hash(&c.output.join(&row_file))?,"source_all_certified_excluded":source_all_excluded,
            "query_preserving":row["query_preserving"]}));
    }
    checked_hash(&cp.join("generate.bin"), &c.expected_generate_sha256)?;
    checked_hash(&cp.join("read-state-bridge-categorical.bin"), BRIDGE_SHA)?;
    if prepared.native.to_bytes()? != gen_bytes {
        return Err(bad("fixed decoder changed"));
    }
    report_output::verify(&c.baseline)?;
    let retained = output_bytes(&c.output)?;
    Ok(
        json!({"schema":"uor-r4.native-trained-entry-route-capacity/1","status":"COMPLETED",
        "source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT"),"executable_sha256":file_hash(&std::env::current_exe()?)?,
        "baseline":c.baseline,"baseline_producer":PRODUCER,"baseline_report_sha256":c.expected_baseline_report_sha256,
        "checkpoint_receipt_sha256":c.expected_checkpoint_receipt_sha256,"checkpoint":"checkpoint-0128","checkpoint_receipt":receipt,
        "inputs_sha256":INPUT_SHA,"generate_sha256":c.expected_generate_sha256,"categorical_sha256":BRIDGE_SHA,"exp_sha256":EXP_SHA,
        "native_source_binding":generator.source_binding(),"device":"cuda:0","cuda_visible_devices":std::env::var("CUDA_VISIBLE_DEVICES").ok(),
        "completed_rows":rows.len(),"physical_source_routes_executed":physical_count,"query_preserving_controls":query_count,
        "physical_source_count_histogram":candidate_histogram,"target_id_histogram":target_histogram,
        "rows_all_source_routes_certified_excluded":all_source_excluded,"query_controls_certified_excluded":query_certified,
        "query_generate_only_target_wins":query_generate_wins,"query_full_factual_copy_target_wins":query_full_wins,
        "factual_entry_target_wins":factual_entry_wins,"legal_token_ids":legal,"score_cache_index":"score-cache-index.json",
        "unique_generate_states":cache_manifest["unique_states"],"score_cache_bytes":cache_manifest["bytes"],
        "score_cache_sha256":cache_manifest["file_sha256"],"rows":rows,"updates":0,
        "storage":{"bytes_before_final_report":retained,"maximum_attempt_bytes":MAX_ATTEMPT_BYTES,"failure_seal_reserve_bytes":1u64<<20,"six_attempt_combined_bytes":1u64<<30},
        "costs":{"total_seconds":total_start.elapsed().as_secs_f64(),"cuda_preparation_seconds":prepare_seconds,
            "cuda_unique_forward_sync_download_coefficient_graph_seconds":cuda_seconds,
            "cpu_unique_native_generate_reference_seconds":cpu_seconds,
            "factual_and_query_control_native_bank_replay_seconds":factual_seconds,"cpu_full_alias_pool_seconds":pool_seconds,
            "prepared_master_download_bytes":prepared.downloaded_master_bytes},
        "decision":"per-row source exclusion and independent query-preserving controls; uncertified is UNRESOLVED, never viable/reachable; no automatic fit authorized",
        "scope":"zero-update trained-checkpoint development diagnosis over all512 empty-prefix entries; no optimizer, runtime selector, held-out, chat/general attention or family impossibility claim; global query-pass control supported but per-step choice UNINTEGRATED; CUDA integer Generate and host native context/bridge/full alias pool"}),
    )
}
fn admit_paths(c: &mut Config) -> Result<()> {
    admit_config(c)?;
    if [&c.baseline, &c.inputs, &c.output].iter().any(|p| {
        !p.is_absolute()
            || p.components()
                .any(|v| matches!(v, std::path::Component::ParentDir))
    }) || c.pin_scope.is_empty()
    {
        return Err(bad(
            "paths must be absolute without parent traversal; pin scope must be nonempty",
        ));
    }
    c.baseline = fs::canonicalize(&c.baseline)?;
    c.inputs = fs::canonicalize(&c.inputs)?;
    // Resolve symlinked existing parents and reject ANY sealed ancestor before
    // claim creates a missing parent or writes its attempt sentinel.
    c.output = output_support::prospective_output(&c.output)?;
    let input_parent = c
        .inputs
        .parent()
        .ok_or_else(|| bad("input parent absent"))?;
    if c.output.starts_with(&c.baseline)
        || c.baseline.starts_with(&c.output)
        || c.output.starts_with(input_parent)
        || input_parent.starts_with(&c.output)
    {
        return Err(bad("output overlaps canonical baseline/input ancestry"));
    }
    Ok(())
}
fn main() -> Result<()> {
    let argv = std::env::args().collect::<Vec<_>>();
    if argv.len() == 3 && argv[1] == "verify-report" {
        report_output::verify(Path::new(&argv[2]))?;
        println!("{}", json!({"status":"VERIFIED","report_root":argv[2]}));
        return Ok(());
    }
    if argv.len() != 2 {
        return Err(bad("usage: native-entry-route-capacity CONFIG.json"));
    }
    let config_bytes = bytes(Path::new(&argv[1]))?;
    let mut c: Config = serde_json::from_slice(&config_bytes)?;
    admit_paths(&mut c)?;
    #[cfg(not(feature = "cuda"))]
    return Err(bad(
        "this diagnostic requires --features cuda; no CPU fallback",
    ));
    #[allow(unreachable_code)]
    report_output::claim(&c.output)?;
    write(
        &c.output.join("config.json"),
        &serde_json::from_slice(&config_bytes)?,
    )?;
    // A final report admission failure is also sealed as FAILED; successful
    // compute is never silently relabelled as a mechanism negative.
    let result = run(&c).and_then(|report| {
        write_bounded(&c.output, "report.json", &report)?;
        Ok(report)
    });
    if let Err(e) = &result {
        write(
            &c.output.join("report.json"),
            &json!({
                "schema":"uor-r4.native-trained-entry-route-capacity/1","status":"FAILED",
                "error":e.to_string(),"source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT"),
                "scope":"execution/parity/storage failure; no model quality verdict"
            }),
        )?;
    }
    report_output::seal(&c.output)?;
    result.map(|_| ())
}

#[cfg(test)]
mod argument_tests {
    use super::*;
    #[test]
    fn factual_pool_parity_rejects_hidden_alias_decomposition_change() -> Result<()> {
        use uor_r4_integer::geometric_vocabulary_actions::VocabularyTokenMass;
        let actual = vec![
            VocabularyTokenMass {
                token_id: 7,
                weight_q31: 100,
                generate_weight_q31: 40,
                copy_weight_q31: 60,
            },
            VocabularyTokenMass {
                token_id: 9,
                weight_q31: 50,
                generate_weight_q31: 50,
                copy_weight_q31: 0,
            },
        ];
        let saved = json!([[7, 100, 40, 60], [9, 50, 50, 0]]);
        compare_token_masses(&actual, &saved)?;
        let mut changed = saved.clone();
        changed[0] = json!([7, 100, 41, 59]); // Same token totals/winner/denominator.
        assert!(compare_token_masses(&actual, &changed).is_err());
        changed = saved.clone();
        changed[1] = json!([8, 50, 50, 0]);
        assert!(compare_token_masses(&actual, &changed).is_err());
        assert!(compare_token_masses(&actual, &Value::Null).is_err());
        Ok(())
    }
    #[test]
    fn trained_admission_requires_exact_final_receipt_and_digest_shape() -> Result<()> {
        let receipt = json!({"step":128,"native_independently_reloaded":true,"masters_independently_reloaded":true,"source_metadata_rebound":true,"parent":{"metadata_sha256":"rebound"}});
        let report = json!({"final_receipt":receipt});
        admit_receipt(&report, &receipt)?;
        let mut not_rebound = receipt.clone();
        not_rebound["source_metadata_rebound"] = json!(false);
        let not_rebound_report = json!({"final_receipt":not_rebound});
        assert!(admit_receipt(&not_rebound_report, &not_rebound).is_err());
        let mut wrong = receipt.clone();
        wrong["step"] = json!(0);
        assert!(admit_receipt(&report, &wrong).is_err());
        let mut wrong = receipt.clone();
        wrong["parent"] = json!({"metadata_sha256":"donor"});
        assert!(admit_receipt(&report, &wrong).is_err());
        assert!(valid_digest(&"a".repeat(64)));
        assert!(!valid_digest("a"));
        assert!(!valid_digest(&"z".repeat(64)));
        Ok(())
    }
    #[test]
    fn parity_paths_protect_sealed_input_ancestors_and_preserve_missing_parents() -> Result<()> {
        let root = std::env::temp_dir().join(format!(
            "native-bank-parity-paths-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_nanos()
        ));
        fs::create_dir(&root)?;
        let result = (|| -> Result<()> {
            let baseline = root.join("baseline");
            fs::create_dir(&baseline)?;
            let inputs_root = root.join("inputs");
            fs::create_dir(&inputs_root)?;
            let inputs = inputs_root.join("inputs.json");
            fs::write(&inputs, b"{}")?;
            fs::write(inputs_root.join(report_output::MANIFEST_FILE), b"{}")?;
            let config = |output| Config {
                baseline: baseline.clone(),
                expected_baseline_report_sha256: "a".repeat(64),
                expected_checkpoint_receipt_sha256: "b".repeat(64),
                expected_generate_sha256: "c".repeat(64),
                inputs: inputs.clone(),
                output,
                pin_lineage: 1,
                pin_commit: 2,
                pin_scope: "compiler-probe".into(),
            };
            let mut nested = config(inputs_root.join("missing/attempt"));
            assert!(admit_paths(&mut nested).is_err());
            assert!(!inputs_root.join("missing").exists());
            #[cfg(unix)]
            {
                let alias = root.join("alias");
                std::os::unix::fs::symlink(&inputs_root, &alias)?;
                let mut linked = config(alias.join("missing/attempt"));
                assert!(admit_paths(&mut linked).is_err());
                assert!(!inputs_root.join("missing").exists());
            }
            let mut ancestor = config(root.clone());
            assert!(admit_paths(&mut ancestor).is_err());
            let mut safe = config(root.join("outputs/missing/attempt"));
            admit_paths(&mut safe)?;
            assert_eq!(
                safe.output,
                fs::canonicalize(&root)?.join("outputs/missing/attempt")
            );
            assert!(!root.join("outputs").exists());
            Ok(())
        })();
        fs::remove_dir_all(&root)?;
        result
    }
}
