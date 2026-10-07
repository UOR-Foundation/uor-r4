//! Offline retained-artifact route-capacity discriminator. No optimizer or runtime selector.
use candle_core::Device;
use serde::Deserialize;
use std::time::Instant;
use uor_r4_training::geometric_generate_learning::GenerateLearningWeights;
#[path = "native-entry-route-capacity/certificate.rs"]
mod certificate;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::{Path, PathBuf},
};
use uor_r4_core::{
    native_geometric::learner::native_bank_generate::{
        BankPin, BoundNativeBytes, GenerationLimits, GenerationStop, NativeBankArtifacts,
        NativeBankGenerateStep, NativeBankGenerator, OwnedBankSegment, OwnedBankSource,
        PinnedBankSnapshot, SnapshotSourceStatus,
    },
    native_geometric::learner::{
        geometric_generate::{GenerateReadCounts, NativeGeometricGenerate},
        geometric_read_state_bridge::{BridgeReadCounts, NativeGeometricReadStateBridge},
    },
    report_output,
};
use uor_r4_integer::{
    geometric_cue_carrier::CueCarrierMetadata,
    geometric_prefix_transport::PrefixTransportMetadata,
    geometric_source_actions::SourceActionBinding,
    geometric_source_realizer::NativeArtifactBinding,
    geometric_vocabulary_actions::{NativeVocabularyActions, VocabularyActionTrace},
    h4_tables::H4Code,
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
const PRODUCER: &str = "7fbfc940ce4abfb056e0a094bf82d1df5d9bc9ae";
const BASE_REPORT: &str = "49c78d15029486c07c0ffca052fe1410bb4ff3f52424227e5d844490722e3b57";
const CP_RECEIPT: &str = "76cfdbfc7504180f325cac2aa6250fb117529f3ab99d99a815900682847531e3";
const INPUT_SHA: &str = "b9661606b280884217a64e0a5b643f8324a90390e47ade7241da0889a5f7c86a";
const GEN_SHA: &str = "41c7beae25e98fdc658ff667959a7c02a6b916b04298a425f7f7f48285d21511";
const BRIDGE_SHA: &str = "af5039e74c6cd92e2fa0bf4c72c315ef34f912d77b09fc607215a18f86cd1d0e";
const EXP_SHA: &str = "79485d6e63cc28f5e01d98c5d73abe021db33fa7fef368142ae6592d06b4817f";
const ROWS: [usize; 4] = [0, 8, 4, 12];
const POSITIONS: [usize; 4] = [0, 3, 4, 5];

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
struct RouteReplay {
    ordinal: usize,
    source_state: Vec<H4Code>,
    post_state: Vec<H4Code>,
    scores: Vec<i64>,
    full: VocabularyActionTrace,
    generate_only: VocabularyActionTrace,
    evidence: Value,
}
struct PositionReplay {
    position: usize,
    factual: Value,
    copy_ids: Vec<u32>,
    routes: Vec<RouteReplay>,
}
struct RowReplay {
    index: usize,
    id: String,
    saved_hash: String,
    saved: Value,
    positions: Vec<PositionReplay>,
}

fn run(c: &Config) -> Result<Value> {
    let total_start = Instant::now();
    if c.expected_baseline_report_sha256 != BASE_REPORT
        || c.expected_checkpoint_receipt_sha256 != CP_RECEIPT
        || c.pin_lineage != 1
        || c.pin_commit != 2
        || c.pin_scope != "compiler-probe"
    {
        return Err(bad("config differs from prospective retained-parent scope"));
    }
    report_output::verify(&c.baseline)?;
    checked_hash(&c.baseline.join("report.json"), BASE_REPORT)?;
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
            "fixed baseline completion/producer/input identity differs",
        ));
    }
    let input_bytes = checked_hash(&c.inputs, INPUT_SHA)?;
    report_output::verify(
        c.inputs
            .parent()
            .ok_or_else(|| bad("input parent absent"))?,
    )?;
    let panel: Panel = serde_json::from_slice(&input_bytes)?;
    if panel.schema != "uor-r4.native-source-bank-probe-input/1" || panel.cases.len() != 512 {
        return Err(bad("fixed input panel incomplete"));
    }
    let cp = c.baseline.join("checkpoint-0000");
    let receipt: Value =
        serde_json::from_slice(&checked_hash(&cp.join("receipt.json"), CP_RECEIPT)?)?;
    if receipt["step"] != 0 || receipt["native_independently_reloaded"] != true {
        return Err(bad("checkpoint initialization/reload identity differs"));
    }
    let binding: NativeArtifactBinding = serde_json::from_value(receipt["parent"].clone())?;
    let gen_bytes = checked_hash(&cp.join("generate.bin"), GEN_SHA)?;
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
    let mut generator = NativeBankGenerator::load(NativeBankArtifacts {
        native_directory: &native_dir,
        source_binding: &binding,
        generate: BoundNativeBytes {
            bytes: &gen_bytes,
            sha256: GEN_SHA,
        },
        bridge: Some(BoundNativeBytes {
            bytes: &bridge_bytes,
            sha256: BRIDGE_SHA,
        }),
        cue_packed: &cue,
        cue_joint_packed: joint.as_deref(),
        cue_metadata: &cue_metadata,
        prefix_packed: &prefix,
        prefix_metadata: &prefix_metadata,
        exp: BoundNativeBytes {
            bytes: &exp_bytes,
            sha256: EXP_SHA,
        },
    })?;
    let development = read(&c.baseline.join("development-0000.json"))?;
    let saved_rows = development["rows"]
        .as_array()
        .ok_or_else(|| bad("row index absent"))?;
    if saved_rows.len() != 512 {
        return Err(bad("retained row coverage"));
    }
    let device = Device::new_cuda(0)?;
    let prepare_start = Instant::now();
    let weights = GenerateLearningWeights::from_native(token_binding, &native_generate, &device)?;
    let prepared = weights.prepare_native()?;
    device.synchronize()?;
    if prepared.native.to_bytes()? != gen_bytes {
        return Err(bad("CUDA preparation changes Generate artifact"));
    }
    let prepare_seconds = prepare_start.elapsed().as_secs_f64();
    let mut replay = Vec::new();
    let mut enumerated = 0usize;
    let mut cuda_seconds = 0f64;
    let mut cpu_reference_seconds = 0f64;
    let mut factual_replay_seconds = 0f64;
    let mut pool_seconds = 0f64;
    // Phase one enumerates routes without target labels under retained teacher prefixes.
    // These later-position prefixes contain gold tokens; entry has an empty prefix.
    // All physical sources and fixed positions are
    // enumerated before any target_label_only field is inspected.
    for &index in &ROWS {
        let packet = panel
            .cases
            .get(index)
            .ok_or_else(|| bad("input row absent"))?
            .clone();
        let rowref = &saved_rows[index];
        if rowref["id"] != packet.id {
            return Err(bad("ordered row ID mismatch"));
        }
        let name = expected_string(&rowref["row_file"])?;
        if Path::new(name).components().count() != 1 {
            return Err(bad("row reference must be leaf"));
        }
        let saved_path = c.baseline.join(name);
        let saved_hash = file_hash(&saved_path)?;
        if rowref["row_sha256"] != saved_hash {
            return Err(bad("saved row digest mismatch"));
        }
        let saved = read(&saved_path)?;
        if saved["id"] != packet.id {
            return Err(bad("saved row ID mismatch"));
        }
        let id = packet.id.clone();
        let bank = generator.admit_bank(snapshot(packet, c)?)?;
        let canonical = saved["canonical"]
            .as_array()
            .ok_or_else(|| bad("canonical evidence absent"))?;
        let mut positions = Vec::new();
        for &position in &POSITIONS {
            let expected = canonical
                .get(position)
                .ok_or_else(|| bad("canonical position absent"))?;
            let actual_prefix: Vec<u32> =
                serde_json::from_value(expected["native"]["actual_prefix_ids"].clone())?;
            if actual_prefix.len() != position {
                return Err(bad("canonical position/prefix length mismatch"));
            }
            let start = Instant::now();
            let step = generator.step(&bank, &actual_prefix)?;
            compare_step(&step, &expected["native"])?;
            factual_replay_seconds += start.elapsed().as_secs_f64();
            let b = &step
                .bank_trace
                .as_ref()
                .ok_or_else(|| bad("full bank trace absent"))?
                .cue_bank
                .bank;
            let factual_bridge = step
                .bridge
                .as_ref()
                .ok_or_else(|| bad("categorical witness absent"))?;
            let query = h4_codes(
                b.context
                    .states
                    .last()
                    .ok_or_else(|| bad("query state absent"))?,
            )?;
            if query != factual_bridge.query_state || b.candidates.len() != 11 {
                return Err(bad("factual query or complete physical bank differs"));
            }
            let mut routes = Vec::new();
            for (ordinal, candidate) in b.candidates.iter().enumerate() {
                let source_state = h4_codes(
                    b.context
                        .states
                        .get(candidate.context_position)
                        .ok_or_else(|| bad("cumulative source state absent"))?,
                )?;
                let mut post = query.clone();
                let mut actions = vec![H4Code::IDENTITY; query.len()];
                let mut action_scores = vec![0; query.len() * 120];
                let mut bridge_counts = BridgeReadCounts::default();
                bridge.apply_into(
                    &query,
                    &source_state,
                    &mut post,
                    &mut actions,
                    &mut action_scores,
                    &mut bridge_counts,
                )?;
                let start = Instant::now();
                let output = weights.forward_prepared_coefficients_only(&prepared, &post)?;
                device.synchronize()?;
                let gpu_time = start.elapsed().as_secs_f64();
                cuda_seconds += gpu_time;
                if output.costs.hard_score_backend != "cuda-authenticated-i64-factors" {
                    return Err(bad("hard scorer did not execute CUDA"));
                }
                let start = Instant::now();
                let mut cpu = vec![0; native_generate.vocab_size()];
                let mut counts = GenerateReadCounts::default();
                native_generate.score_into(&post, &mut cpu, &mut counts)?;
                let cpu_time = start.elapsed().as_secs_f64();
                cpu_reference_seconds += cpu_time;
                if cpu != output.scores_q24 {
                    return Err(bad("complete CPU/CUDA Generate vector mismatch"));
                }
                let selected = ordinal == factual_bridge.selected_ordinal;
                if selected
                    && (post != step.post_state
                        || source_state != factual_bridge.source_state
                        || actions != factual_bridge.action_codes
                        || action_scores != factual_bridge.action_scores_q24
                        || output.scores_q24 != step.generate_raw_scores_q24)
                {
                    return Err(bad(
                        "factual route does not reproduce native bridge/Generate",
                    ));
                }
                let start = Instant::now();
                let full = pool.reduce_trace(
                    &output.scores_q24,
                    &step.copy_token_ids,
                    &step.copy_raw_scores_q24,
                )?;
                let generate_only = pool.reduce_trace(&output.scores_q24, &[], &[])?;
                pool_seconds += start.elapsed().as_secs_f64();
                if selected && full != step.actions {
                    return Err(bad("factual full alias-pool parity mismatch"));
                }
                let evidence = json!({"ordinal":ordinal,"physical_candidate":candidate,
                    "is_factual_selected":selected,"query_state_codes":codes(&query),
                    "source_state_codes":codes(&source_state),"post_state_codes":codes(&post),
                    "action_codes":codes(&actions),"action_scores_q24":action_scores,"bridge_costs":bridge_counts,
                    "generate_raw_scores_sha256":hash(&serde_json::to_vec(&output.scores_q24)?),
                    "cpu_native_counts":counts,"cuda_costs":output.costs,
                    "cuda_forward_including_sync_download_and_coefficient_graph_seconds":gpu_time,
                    "cpu_native_score_seconds":cpu_time,"full_pool_factual_copy_summary":full.summary,
                    "generate_only_diagnostic_summary":generate_only.summary,"complete_vector_cpu_cuda_parity":"PASS"});
                routes.push(RouteReplay {
                    ordinal,
                    source_state,
                    post_state: post,
                    scores: output.scores_q24,
                    full,
                    generate_only,
                    evidence,
                });
                enumerated += 1;
            }
            positions.push(PositionReplay { position, factual:json!({"actual_prefix_ids":actual_prefix,
                "chosen_token_id":step.actions.summary.chosen_token_id,"selected_ordinal":factual_bridge.selected_ordinal,
                "causal_tokens":b.context.tokens,"copy_token_ids":step.copy_token_ids,
                "copy_raw_scores_q24":step.copy_raw_scores_q24,"saved_step_parity":"PASS"}),
                copy_ids:step.copy_token_ids,routes });
        }
        replay.push(RowReplay {
            index,
            id,
            saved_hash,
            saved,
            positions,
        });
    }
    if enumerated != 176 {
        return Err(bad("prospective physical route coverage differs"));
    }
    // Phase two reads labels only for mass/rank/certificate analysis.
    let mut rows = Vec::new();
    let mut entry_excluded = 0usize;
    let mut entry_uncertified = 0usize;
    for row in replay {
        let mut positions = Vec::new();
        for mut p in row.positions {
            let target: u32 = serde_json::from_value(
                row.saved["canonical"][p.position]["target_label_only"].clone(),
            )?;
            let in_copy = p.copy_ids.contains(&target);
            if (p.position == 0 && (in_copy || target != 2997)) || (p.position != 0 && !in_copy) {
                return Err(bad(
                    "prospective entry/retention target availability differs",
                ));
            }
            let mut routes = Vec::new();
            let mut distinct = std::collections::BTreeSet::new();
            for r in &mut p.routes {
                distinct.insert(codes(&r.post_state));
                let full_target = r
                    .full
                    .token_masses
                    .iter()
                    .find(|m| m.token_id == target)
                    .ok_or_else(|| bad("target is not legal"))?;
                let generate_target = r
                    .generate_only
                    .token_masses
                    .iter()
                    .find(|m| m.token_id == target)
                    .ok_or_else(|| bad("Generate target absent"))?;
                let certificate = if !in_copy {
                    certificate::certify_generate_dominance(&r.scores, &legal, target, &exp)?
                } else {
                    None
                };
                if p.position == 0 {
                    if certificate.is_some() {
                        entry_excluded += 1;
                    } else {
                        entry_uncertified += 1;
                    }
                }
                let mut evidence = r.evidence.clone();
                evidence["generate_raw_scores_q24"] = json!(r.scores);
                evidence["target_generate_raw_score_q24"] = json!(r.scores[target as usize]);
                evidence["full_factual_copy_target_mass"] = serde_json::to_value(full_target)?;
                evidence["generate_only_target_mass"] = serde_json::to_value(generate_target)?;
                evidence["target_generate_clipped_rank_with_smallest_id_ties"] = json!(
                    1 + legal
                        .iter()
                        .filter(|&&j| {
                            let s = r.scores[j as usize].clamp(-(8 << 24), 8 << 24);
                            let t = r.scores[target as usize].clamp(-(8 << 24), 8 << 24);
                            s > t || (s == t && j < target)
                        })
                        .count()
                );
                evidence["entry_exclusion_certificate"] = serde_json::to_value(certificate)?;
                evidence["certificate_applicable"] = json!(!in_copy);
                evidence["ordinal"] = json!(r.ordinal);
                evidence["source_state_codes"] = json!(codes(&r.source_state));
                routes.push(evidence);
            }
            positions.push(json!({"position":p.position,"target_label_only":target,"target_in_copy":in_copy,
                "factual":p.factual,"physical_routes":routes,"distinct_poststates":distinct.len(),
                "counterfactual_scope":"all physical source poststates; arbitrary-source upper bound, not reachability under a shared legal Potential; full-pool summaries keep factual Copy scores only diagnostically"}));
        }
        rows.push(json!({"index":row.index,"id":row.id,"saved_row_sha256":row.saved_hash,"positions":positions}));
    }
    if checked_hash(&cp.join("generate.bin"), GEN_SHA)? != gen_bytes
        || checked_hash(&cp.join("read-state-bridge-categorical.bin"), BRIDGE_SHA)? != bridge_bytes
        || prepared.native.to_bytes()? != gen_bytes
    {
        return Err(bad("fixed native inputs changed during diagnostic"));
    }
    report_output::verify(&c.baseline)?;
    let excluded = entry_excluded == 44;
    Ok(
        json!({"schema":"uor-r4.native-entry-route-capacity/1","status":"COMPLETED",
        "source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT"),"executable_sha256":file_hash(&std::env::current_exe()?)?,
        "baseline":c.baseline,"baseline_producer":PRODUCER,"baseline_report_sha256":BASE_REPORT,
        "checkpoint_receipt_sha256":CP_RECEIPT,"inputs_sha256":INPUT_SHA,
        "generate_sha256":GEN_SHA,"categorical_sha256":BRIDGE_SHA,"exp_sha256":EXP_SHA,
        "native_source_binding":generator.source_binding(),"device":"cuda:0","cuda_visible_devices":std::env::var("CUDA_VISIBLE_DEVICES").ok(),
        "physical_routes_executed":enumerated,"entry_routes_excluded":entry_excluded,"entry_routes_uncertified":entry_uncertified,
        "all_entry_poststates_excluded":excluded,"legal_token_ids":legal,"rows":rows,"updates":0,
        "costs":{"total_seconds":total_start.elapsed().as_secs_f64(),"cuda_preparation_seconds":prepare_seconds,
            "cuda_forward_sync_download_coefficient_graph_seconds":cuda_seconds,"cpu_native_generate_reference_seconds":cpu_reference_seconds,
            "factual_native_context_bank_bridge_generate_replay_seconds":factual_replay_seconds,"cpu_full_alias_pool_seconds":pool_seconds,
            "prepared_master_download_bytes":prepared.downloaded_master_bytes},
        "decision":if excluded {"Potential-only entry fix excluded for this fixed context/map/Generate parent and four construction rows; next joint geometric readout/context discriminator"}
            else {"No complete exclusion certificate; full propagated shared legal Potential scan warranted; uncertified is not proof of viable or reachable"},
        "scope":"zero-update common-parent construction diagnosis; later positions use retained teacher prefixes; entry prefix empty; no optimizer, runtime selector, held-out, general attention/chat or family impossibility claim; CUDA hard Generate with host native reference/context/pool and explicit snapshot/coefficient-graph overhead"}),
    )
}
fn admit_paths(c: &mut Config) -> Result<()> {
    if c.expected_baseline_report_sha256 != BASE_REPORT
        || c.expected_checkpoint_receipt_sha256 != CP_RECEIPT
        || c.pin_lineage != 1
        || c.pin_commit != 2
        || c.pin_scope != "compiler-probe"
    {
        return Err(bad("config differs from prospective retained-parent scope"));
    }
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
    let result = run(&c);
    let report = match &result {
        Ok(v) => v.clone(),
        Err(e) => {
            json!({"schema":"uor-r4.native-entry-route-capacity/1","status":"FAILED","error":e.to_string(),
            "source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT"),"scope":"execution/parity failure; no model quality verdict"})
        }
    };
    write(
        &c.output.join("config.json"),
        &serde_json::from_slice(&config_bytes)?,
    )?;
    write(&c.output.join("report.json"), &report)?;
    report_output::seal(&c.output)?;
    result.map(|_| ())
}

#[cfg(test)]
mod argument_tests {
    use super::*;
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
                expected_baseline_report_sha256: BASE_REPORT.into(),
                expected_checkpoint_receipt_sha256: CP_RECEIPT.into(),
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
