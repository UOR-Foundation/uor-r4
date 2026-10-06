//! Offline first-token checkpoint crosses, using authentic empty-prefix packets.
//! Surgical retained-state/Copy swaps are not runnable context checkpoints.
use serde_json::{json, Value};
use std::{
    collections::BTreeSet,
    fs, io,
    path::{Component, Path, PathBuf},
    time::Instant,
};
use uor_r4_core::{
    native_geometric::learner::geometric_generate::{GenerateReadCounts, NativeGeometricGenerate},
    report_output,
};
use uor_r4_integer::{
    geometric_source_actions::SourceActionBinding,
    geometric_source_realizer::{NativeArtifactBinding, NativeSourceRealizer},
    geometric_vocabulary_actions::{NativeVocabularyActions, VocabularyActionTrace},
    h4_tables::H4Code,
};
use uor_r4_training::sha256_bytes;
#[path = "../../uor-r4-integer/examples/support/source_probe.rs"]
mod output_support;
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
const MAX_SECONDS: u64 = 90;
const MAX_REPORT_BYTES: usize = 16 << 20;
fn bad(s: &str) -> Box<dyn std::error::Error> {
    io::Error::new(io::ErrorKind::InvalidData, s).into()
}
fn read(p: &Path) -> Result<Value> {
    Ok(serde_json::from_slice(&fs::read(p)?)?)
}
fn array<'a>(v: &'a Value, k: &str) -> Result<&'a Vec<Value>> {
    v[k].as_array()
        .ok_or_else(|| bad(&format!("missing array {k}")))
}
fn uint(v: &Value, k: &str) -> Result<u64> {
    v[k].as_u64()
        .ok_or_else(|| bad(&format!("missing uint {k}")))
}
fn ints(v: &Value) -> Result<Vec<i64>> {
    v.as_array()
        .ok_or_else(|| bad("missing integer array"))?
        .iter()
        .map(|x| x.as_i64().ok_or_else(|| bad("invalid integer")))
        .collect()
}
fn relative(root: &Path, name: &str) -> Result<PathBuf> {
    let p = Path::new(name);
    if p.as_os_str().is_empty() || p.components().any(|c| !matches!(c, Component::Normal(_))) {
        return Err(bad("unsafe row path"));
    }
    Ok(root.join(p))
}
fn deadline(start: Instant) -> Result<()> {
    if start.elapsed().as_secs() >= MAX_SECONDS {
        Err(bad("90-second audit ceiling reached"))
    } else {
        Ok(())
    }
}
fn first_packet(row: &Value) -> Result<&Value> {
    let packet = array(row, "generation")?
        .first()
        .ok_or_else(|| bad("first generation missing"))?;
    if !array(packet, "actual_prefix_ids")?.is_empty() {
        return Err(bad("first generation prefix nonempty"));
    }
    let canonical = &array(row, "canonical")?
        .first()
        .ok_or_else(|| bad("first canonical missing"))?["native"];
    if !array(canonical, "actual_prefix_ids")?.is_empty() {
        return Err(bad("first canonical prefix nonempty"));
    }
    for k in [
        "retained_state_codes",
        "generate_raw_scores_sha256",
        "copy_token_ids",
        "copy_raw_scores_q24",
        "pool",
        "source_provenance",
    ] {
        if packet.get(k).is_none() || packet[k] != canonical[k] {
            return Err(bad(&format!("first canonical/generation mismatch {k}")));
        }
    }
    let chosen = uint(&packet["pool"]["summary"], "chosen_token_id")?;
    if array(row, "generated_ids")?.first().and_then(Value::as_u64) != Some(chosen) {
        return Err(bad("saved emitted first token differs from diagonal"));
    }
    Ok(packet)
}
fn check_matched_packets(a: &Value, b: &Value) -> Result<()> {
    if a["copy_token_ids"] != b["copy_token_ids"] {
        return Err(bad("cross-context Copy occurrence IDs/order differ"));
    }
    for k in ["candidates", "causal_tokens"] {
        if a["source_provenance"].get(k).is_none()
            || a["source_provenance"][k] != b["source_provenance"][k]
        {
            return Err(bad("cross-context bank identity/causal tokens differ"));
        }
    }
    Ok(())
}
struct Packet {
    states: Vec<H4Code>,
    ids: Vec<u32>,
    scores: Vec<i64>,
}
fn packet(v: &Value) -> Result<Packet> {
    let states = ints(&v["retained_state_codes"])?
        .into_iter()
        .map(|x| Ok(H4Code::try_from(u8::try_from(x)?)?))
        .collect::<Result<Vec<_>>>()?;
    let ids = ints(&v["copy_token_ids"])?
        .into_iter()
        .map(|x| Ok(u32::try_from(x)?))
        .collect::<Result<Vec<_>>>()?;
    let scores = ints(&v["copy_raw_scores_q24"])?;
    if ids.len() != scores.len() {
        return Err(bad("Copy shapes differ"));
    }
    Ok(Packet {
        states,
        ids,
        scores,
    })
}
fn diagonal(v: &Value, scores: &[i64], trace: &VocabularyActionTrace) -> Result<()> {
    if v["generate_raw_scores_sha256"].as_str()
        != Some(sha256_bytes(&serde_json::to_vec(scores)?).as_str())
        || v["pool"]["summary"] != serde_json::to_value(&trace.summary)?
    {
        return Err(bad("diagonal rawscore SHA/fullpool summary mismatch"));
    }
    Ok(())
}
fn metrics(
    trace: &VocabularyActionTrace,
    scores: &[i64],
    pool: &NativeVocabularyActions,
    gold: u32,
) -> Result<Value> {
    let target = trace
        .token_masses
        .iter()
        .find(|m| m.token_id == gold)
        .ok_or_else(|| bad("gold outside legal pool"))?;
    let competitor = trace
        .token_masses
        .iter()
        .filter(|m| m.token_id != gold)
        .max_by(|a, b| {
            a.weight_q31
                .cmp(&b.weight_q31)
                .then_with(|| b.token_id.cmp(&a.token_id))
        })
        .ok_or_else(|| bad("no nongold competitor"))?;
    let mut best = None;
    for &id in pool.legal_token_ids() {
        if best.is_none_or(|b: u32| {
            scores[id as usize].clamp(-(8 << 24), 8 << 24)
                > scores[b as usize].clamp(-(8 << 24), 8 << 24)
        }) {
            best = Some(id);
        }
    }
    let best = best.ok_or_else(|| bad("empty legal Generate"))?;
    let best_other_generate = pool
        .legal_token_ids()
        .iter()
        .copied()
        .filter(|&id| id != gold)
        .max_by(|a, b| {
            scores[*a as usize]
                .clamp(-(8 << 24), 8 << 24)
                .cmp(&scores[*b as usize].clamp(-(8 << 24), 8 << 24))
                .then_with(|| b.cmp(a))
        })
        .ok_or_else(|| bad("no other legal Generate token"))?;
    let gs = *scores
        .get(gold as usize)
        .ok_or_else(|| bad("gold out of score domain"))?;
    let rank = 1 + pool
        .legal_token_ids()
        .iter()
        .filter(|&&id| {
            scores[id as usize].clamp(-(8 << 24), 8 << 24) > gs.clamp(-(8 << 24), 8 << 24)
                || (scores[id as usize].clamp(-(8 << 24), 8 << 24) == gs.clamp(-(8 << 24), 8 << 24)
                    && id < gold)
        })
        .count();
    let best_component = trace
        .token_masses
        .iter()
        .max_by(|a, b| {
            a.generate_weight_q31
                .cmp(&b.generate_weight_q31)
                .then_with(|| b.token_id.cmp(&a.token_id))
        })
        .ok_or_else(|| bad("empty Generate component"))?;
    let component_rank = 1 + trace
        .token_masses
        .iter()
        .filter(|m| {
            m.generate_weight_q31 > target.generate_weight_q31
                || (m.generate_weight_q31 == target.generate_weight_q31 && m.token_id < gold)
        })
        .count();
    Ok(
        json!({"chosen_token_id":trace.summary.chosen_token_id,"first_correct":trace.summary.chosen_token_id==gold,"gold_mass_q31":target.weight_q31,"gold_generate_mass_q31":target.generate_weight_q31,"gold_copy_mass_q31":target.copy_weight_q31,"total_mass_q31":trace.summary.total_weight_q31,"native_first_nll":-(target.weight_q31 as f64/trace.summary.total_weight_q31 as f64).ln(),"best_nongold_token_id":competitor.token_id,"best_nongold_mass_q31":competitor.weight_q31,"gold_vs_best_nongold_margin_q31":i128::from(target.weight_q31)-i128::from(competitor.weight_q31),"gold_vs_best_nongold_probability_margin":(target.weight_q31 as f64-competitor.weight_q31 as f64)/trace.summary.total_weight_q31 as f64,"best_generate_raw_score_token_id":best,"best_other_generate_token_id":best_other_generate,"gold_generate_raw_score_q24":gs,"gold_generate_raw_score_rank_with_smallest_id_ties":rank,"gold_vs_best_generate_raw_margin_q24":gs.checked_sub(scores[best as usize]).ok_or_else(||bad("score margin overflow"))?,"gold_vs_best_other_generate_raw_margin_q24":gs.checked_sub(scores[best_other_generate as usize]).ok_or_else(||bad("score margin overflow"))?,"best_common_reference_generate_component_token_id":best_component.token_id,"gold_common_reference_generate_component_rank_with_smallest_id_ties":component_rank,"Generate_component_scope":"weights at fullpool reference, not a separately normalized Generate-only pool","winner_generate_mass_q31":trace.summary.chosen_generate_weight_q31,"winner_copy_mass_q31":trace.summary.chosen_copy_weight_q31}),
    )
}
fn audit(fit: &Path, out: &Path, start: Instant) -> Result<()> {
    report_output::verify(fit)?;
    deadline(start)?;
    let report = read(&fit.join("report.json"))?;
    if report["schema"] != "uor-r4.geometric-bank-generate-fit/1"
        || report["status"] != "COMPLETED"
        || report["mode"] != "fit"
        || report["balanced_token_geometry"] != true
    {
        return Err(bad("requires completed balanced fit"));
    }
    let stages = array(&report, "stages")?;
    if stages.len() != 2
        || uint(&stages[0], "step")? != 0
        || uint(&stages[1], "step")? != uint(&report, "updates")?
        || uint(&report, "updates")? == 0
    {
        return Err(bad("requires matched initial/final stages"));
    }
    let admission = read(&fit.join("input-admission.json"))?;
    let hashes = admission["input_sha256"]
        .as_object()
        .ok_or_else(|| bad("original input hash inventory absent"))?;
    if hashes.is_empty()
        || hashes.values().any(|v| {
            v.as_str()
                .is_none_or(|s| s.len() != 64 || !s.bytes().all(|b| b.is_ascii_hexdigit()))
        })
    {
        return Err(bad("invalid original input identity"));
    }
    let oracles = read(&fit.join("frozen-development-answer-oracles.json"))?;
    let oracle_rows = array(&oracles, "cases")?;
    if oracle_rows.len() != 512 {
        return Err(bad("requires original512 oracle identities"));
    }
    let mut bindings = Vec::new();
    let mut models = Vec::new();
    let mut cps = Vec::new();
    let mut context_metadata = Vec::new();
    let mut generation_sha = Vec::new();
    for stage in stages {
        let cp = fit.join(format!("checkpoint-{:04}", uint(stage, "step")?));
        let receipt = read(&cp.join("receipt.json"))?;
        if stage["checkpoint"] != receipt {
            return Err(bad("stage/checkpoint receipt differs"));
        }
        let expected: NativeArtifactBinding = serde_json::from_value(receipt["parent"].clone())?;
        let native = NativeSourceRealizer::load_native(&cp.join("native"), &expected)?;
        let tok = fs::read(cp.join("native/tokenizer.json"))?;
        let binding = SourceActionBinding::new(&tok)?;
        if native.binding() != &binding {
            return Err(bad("native/Generate tokenizer mismatch"));
        }
        let bytes = fs::read(cp.join("generate.bin"))?;
        let sha = sha256_bytes(&bytes);
        if receipt["generate_sha256"].as_str() != Some(sha.as_str()) {
            return Err(bad("Generate receipt SHA mismatch"));
        }
        let model = NativeGeometricGenerate::from_bytes(&bytes, &binding)?;
        if model.to_bytes()? != bytes
            || model.lanes()
                != native.context_config().heads * native.context_config().lanes_per_head
        {
            return Err(bad("Generate reload/context dimensions differ"));
        }
        context_metadata.push(json!({"execution_binding":expected,"cue_metadata_sha256":sha256_bytes(&fs::read(cp.join("cue/native-metadata.json"))?),"prefix_metadata_sha256":sha256_bytes(&fs::read(cp.join("prefix/native-metadata.json"))?),"native_metadata_sha256":sha256_bytes(&fs::read(cp.join("native/metadata.json"))?),"context_packed_sha256":sha256_bytes(&fs::read(cp.join("native/consumer/context-q4.bin"))?)}));
        generation_sha.push(sha);
        bindings.push(binding);
        models.push(model);
        cps.push(cp);
        deadline(start)?;
    }
    if bindings[0] != bindings[1]
        || models[0].lanes() != models[1].lanes()
        || models[0].metadata().h4_mathematical_sha256
            != models[1].metadata().h4_mathematical_sha256
    {
        return Err(bad("cross-checkpoint vocabulary/geometry frame mismatch"));
    }
    for name in [
        "native/tokenizer.json",
        "native/period-q4.bin",
        "native/consumer/potential-q4.bin",
        "native/consumer/no-read-q4.bin",
        "native/consumer/exp-q31.bin",
        "cue/cue-q4.bin",
        "prefix/prefix-q4.bin",
    ] {
        if fs::read(cps[0].join(name))? != fs::read(cps[1].join(name))? {
            return Err(bad("noncontext frozen operator changed"));
        }
    }
    for name in ["cue/native-metadata.json", "prefix/native-metadata.json"] {
        let a = read(&cps[0].join(name))?;
        let b = read(&cps[1].join(name))?;
        for key in ["schema", "policy", "context", "potential", "algebra_sha256"] {
            if a.get(key).is_none() || a[key] != b[key] {
                return Err(bad("carrier configuration/geometry changed"));
            }
        }
    }
    let joint0 = cps[0].join("cue/cue-joint-q4.bin");
    let joint1 = cps[1].join("cue/cue-joint-q4.bin");
    let cue0 = read(&cps[0].join("cue/native-metadata.json"))?;
    let cue1 = read(&cps[1].join("cue/native-metadata.json"))?;
    if cue0["joint"] != cue1["joint"]
        || cue0["joint"].is_object() != joint0.exists()
        || cue1["joint"].is_object() != joint1.exists()
    {
        return Err(bad("frozen joint cue configuration/basis metadata changed"));
    }
    if joint0.exists()
        && cue0["joint"]["packed_sha256"].as_str()
            != Some(sha256_bytes(&fs::read(&joint0)?).as_str())
    {
        return Err(bad("joint cue metadata/payload digest mismatch"));
    }
    if joint0.exists() != joint1.exists()
        || (joint0.exists() && fs::read(joint0)? != fs::read(joint1)?)
    {
        return Err(bad("frozen joint cue changed"));
    }
    let refs0 = array(&stages[0]["evaluation"], "rows")?;
    let refs1 = array(&stages[1]["evaluation"], "rows")?;
    if refs0.len() != 512
        || refs1.len() != 512
        || uint(&stages[0]["evaluation"], "cases")? != 512
        || uint(&stages[1]["evaluation"], "cases")? != 512
    {
        return Err(bad("requires512 matched rows"));
    }
    let mut pool = NativeVocabularyActions::new(
        bindings[0].clone(),
        &fs::read(cps[0].join("native/consumer/exp-q31.bin"))?,
    )?;
    let cells = [
        ("C0G0", 0, 0, 0),
        ("C0G1", 0, 0, 1),
        ("C1G0", 1, 1, 0),
        ("C1G1", 1, 1, 1),
        ("G1S0Copy1", 0, 1, 1),
        ("G1S1Copy0", 1, 0, 1),
    ];
    let mut rows = Vec::new();
    let mut seen = BTreeSet::new();
    let mut correct = [0usize; 6];
    let mut loss = [0f64; 6];
    let mut mass_margin = [0i128; 6];
    for (i, (r0, r1)) in refs0.iter().zip(refs1).enumerate() {
        deadline(start)?;
        if r0["id"] != r1["id"]
            || r0["id"] != oracle_rows[i]["id"]
            || r0["id"]
                .as_str()
                .is_none_or(|id| !seen.insert(id.to_owned()))
        {
            return Err(bad("row order/ID/oracle identity mismatch"));
        }
        let mut loaded = Vec::new();
        let mut row_shas = Vec::new();
        for r in [r0, r1] {
            let p = relative(
                fit,
                r["row_file"]
                    .as_str()
                    .ok_or_else(|| bad("row path missing"))?,
            )?;
            let bytes = fs::read(p)?;
            let sha = sha256_bytes(&bytes);
            if r["row_sha256"].as_str() != Some(sha.as_str()) {
                return Err(bad("sealed row SHA mismatch"));
            }
            let row: Value = serde_json::from_slice(&bytes)?;
            if row["id"] != r["id"] || row["generated_ids"] != r["generated_ids"] {
                return Err(bad("loaded row ID mismatch"));
            }
            row_shas.push(sha);
            loaded.push(row);
        }
        let v0 = first_packet(&loaded[0])?;
        let v1 = first_packet(&loaded[1])?;
        check_matched_packets(v0, v1)?;
        let p0 = packet(v0)?;
        let p1 = packet(v1)?;
        let ps = [&p0, &p1];
        let mut gen = Vec::new();
        for (s, g) in [(0, 0), (0, 1), (1, 0), (1, 1)] {
            let mut scores = vec![0; models[g].vocab_size()];
            models[g].score_into(
                &ps[s].states,
                &mut scores,
                &mut GenerateReadCounts::default(),
            )?;
            gen.push(scores);
        }
        let mut traces = Vec::new();
        for (_, s, c, g) in cells {
            traces.push(pool.reduce_trace(&gen[s * 2 + g], &ps[c].ids, &ps[c].scores)?);
        }
        diagonal(v0, &gen[0], &traces[0])?;
        diagonal(v1, &gen[3], &traces[3])?;
        // Labels are consulted only after all six target-free pools exist.
        let labels0 = array(&loaded[0], "canonical_target_ids_labels_only")?;
        let labels1 = array(&loaded[1], "canonical_target_ids_labels_only")?;
        if labels0.is_empty()
            || labels0 != labels1
            || Value::Array(labels0.clone()) != oracle_rows[i]["canonical_ids_labels_only"]
        {
            return Err(bad("cross-stage/oracle labels mismatch"));
        }
        let gold = u32::try_from(
            labels0[0]
                .as_u64()
                .ok_or_else(|| bad("first gold missing"))?,
        )?;
        if !bindings[0].admits_token(gold) {
            return Err(bad("gold not legal"));
        }
        for (c, row) in loaded.iter().enumerate() {
            let first = &array(row, "canonical")?[0];
            if uint(first, "target_label_only")? != u64::from(gold)
                || uint(first, "native_denominator")? != traces[c * 3].summary.total_weight_q31
                || uint(first, "native_target_mass")?
                    != traces[c * 3]
                        .token_masses
                        .iter()
                        .find(|m| m.token_id == gold)
                        .ok_or_else(|| bad("diagonal gold missing"))?
                        .weight_q31
            {
                return Err(bad("diagonal canonical mass/label mismatch"));
            }
        }
        let mut m = serde_json::Map::new();
        for (j, (name, s, _, g)) in cells.iter().enumerate() {
            let v = metrics(&traces[j], &gen[s * 2 + g], &pool, gold)?;
            correct[j] += usize::from(traces[j].summary.chosen_token_id == gold);
            loss[j] += v["native_first_nll"]
                .as_f64()
                .ok_or_else(|| bad("NLL missing"))?;
            mass_margin[j] += v["gold_vs_best_nongold_margin_q31"]
                .as_i64()
                .ok_or_else(|| bad("margin outside i64"))? as i128;
            m.insert((*name).to_owned(), v);
        }
        rows.push(json!({"id":r0["id"],"source_row_sha256":row_shas,"gold_label_only":gold,"state_changed":p0.states!=p1.states,"copy_scores_changed":p0.scores!=p1.scores,"copy_occurrence_ids_sha256":sha256_bytes(&serde_json::to_vec(&p0.ids)?),"causal_input_sha256":sha256_bytes(&serde_json::to_vec(&v0["source_provenance"])?),"arms":m}));
    }
    let mut summaries = serde_json::Map::new();
    for (j, (name, _, _, _)) in cells.iter().enumerate() {
        summaries.insert((*name).to_owned(),json!({"first_correct":correct[j],"mean_native_first_nll":loss[j]/512.,"mean_gold_vs_best_nongold_mass_margin_q31":mass_margin[j] as f64/512.}));
    }
    let trade = |a: &str, b: &str| json!({"from":a,"to":b,"corrected":rows.iter().filter(|r|r["arms"][a]["first_correct"]==false&&r["arms"][b]["first_correct"]==true).map(|r|r["id"].clone()).collect::<Vec<_>>(),"regressed":rows.iter().filter(|r|r["arms"][a]["first_correct"]==true&&r["arms"][b]["first_correct"]==false).map(|r|r["id"].clone()).collect::<Vec<_>>()});
    report_output::verify(fit)?;
    deadline(start)?;
    let value = json!({"schema":"uor-r4.geometric-generate-checkpoint-cross/1","status":"COMPLETED","scope":"first target-free empty-prefix pools only; C context swaps change retained state and factual Copy together; extra G1 mediator swaps are surgical diagnostics, not runnable contexts","complete_ownprefix":"NOT_MEASURED","context_recomputed":false,"updates":0,"cases":512,"source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT"),"executable_sha256":sha256_bytes(&fs::read(std::env::current_exe()?)?),"input_report_sha256":sha256_bytes(&fs::read(fit.join("report.json"))?),"input_manifest_sha256":sha256_bytes(&fs::read(fit.join("manifest.json"))?),"input_admission_sha256":sha256_bytes(&fs::read(fit.join("input-admission.json"))?),"original_input_sha256":hashes,"original_input_scope":"sealed parent's bound input inventory; external original files not reloaded","context_artifact_bindings":context_metadata,"generate_sha256":generation_sha,"diagonal_fullpool_verified":true,"arms":summaries,"context_tradeoffs_fixed_G0":trade("C0G0","C1G0"),"context_tradeoffs_fixed_G1":trade("C0G1","C1G1"),"retained_state_tradeoff_fixed_G1_Copy0":trade("C0G1","G1S1Copy0"),"Copy_tradeoff_fixed_G1_S0":trade("C0G1","G1S0Copy1"),"rows":rows,"elapsed_seconds":start.elapsed().as_secs_f64(),"maximum_seconds":MAX_SECONDS,"maximum_report_bytes":MAX_REPORT_BYTES});
    let bytes = serde_json::to_vec_pretty(&value)?;
    if bytes.len() > MAX_REPORT_BYTES {
        return Err(bad("report exceeds16MiB"));
    }
    fs::write(out.join("report.json"), bytes)?;
    Ok(())
}
fn admit_output(fit: &Path, out: &Path) -> Result<()> {
    let prospective = output_support::prospective_output(out)?;
    let input = fs::canonicalize(fit)?;
    if prospective.starts_with(&input) || input.starts_with(&prospective) {
        return Err(bad("output/input overlap"));
    }
    Ok(())
}
/// Explicit native diagnostic controls. This is separate from the strict legacy
/// first-token cross; every control is compiled and independently reloaded.
mod potential_attribution {
    use super::*;
    use candle_core::{Device, Tensor};
    use serde::Deserialize;
    use std::collections::BTreeMap;
    use uor_r4_core::answer_oracle::FrozenAnswers;
    use uor_r4_integer::{
        geometric_context::NativeContextState,
        geometric_cue_carrier::{CueAngularConfig, CueAngularQ4, CueJointMetadata, CueJointQ4},
        geometric_occurrence_read::{
            FrameMetadata, FrameStatus, SelectedRecordFrame, SourceIdentity,
        },
        geometric_potential_q4::{
            unpack_coefficients, PotentialQ4Config, FAMILY_COUNTS, FAMILY_NAMES,
        },
        geometric_prefix_transport::{PrefixAngularConfig, PrefixAngularQ4},
        geometric_source_emission_view::SourceEmissionView,
        geometric_source_realizer::SourceBankSegment,
    };
    use uor_r4_tokenizer::ByteBpeTokenizer;
    use uor_r4_training::geometric_occurrence_consumer::{
        source_realizer::{NativeSourceRealizer as LearningNative, SourceRealizerWeights},
        ConsumerIdentity,
    };

    const ARMS: [&str; 4] = ["P0", "P1", "P1_restore_shared0", "P0_take_shared0"];
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Inputs {
        schema: String,
        cases: Vec<Input>,
    }
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Input {
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
                Self::Context { .. } => None,
            }
        }
    }
    struct Episode {
        input: Input,
        views: Vec<Option<SourceEmissionView>>,
    }
    impl Episode {
        fn segments(&self) -> Result<Vec<SourceBankSegment<'_>>> {
            self.input
                .segments
                .iter()
                .zip(&self.views)
                .map(|(segment, view)| {
                    Ok(match segment {
                        Segment::Source { event, .. } => SourceBankSegment::Source {
                            frame: segment.frame().ok_or_else(|| bad("source frame missing"))?,
                            view: view.as_ref().ok_or_else(|| bad("source view missing"))?,
                            event: *event,
                        },
                        Segment::Context {
                            token_ids,
                            event,
                            role,
                        } => SourceBankSegment::Context {
                            token_ids,
                            event: *event,
                            role: *role,
                        },
                    })
                })
                .collect()
        }
        fn has_source(&self) -> bool {
            self.views.iter().any(Option::is_some)
        }
        fn base_len(&self) -> usize {
            self.input.query_ids.len()
                + self
                    .input
                    .segments
                    .iter()
                    .zip(&self.views)
                    .map(|(s, v)| match (s, v) {
                        (Segment::Context { token_ids, .. }, _) => token_ids.len(),
                        (_, Some(v)) => v.emitted_token_ids().len(),
                        _ => 0,
                    })
                    .sum::<usize>()
        }
        fn no_source_tokens(&self, prefix: &[u32]) -> Result<Vec<u32>> {
            let mut ids = Vec::new();
            for s in &self.input.segments {
                match s {
                    Segment::Context { token_ids, .. } => ids.extend_from_slice(token_ids),
                    _ => return Err(bad("no-source sequence contains source")),
                }
            }
            ids.extend_from_slice(&self.input.query_ids);
            ids.extend_from_slice(prefix);
            if ids.is_empty() || ids.len() > 128 {
                return Err(bad("causal no-source context outside1..128"));
            }
            Ok(ids)
        }
    }
    struct Envelope {
        start: Instant,
        seconds: u64,
        bytes: u64,
        output: PathBuf,
        written: u64,
    }
    impl Envelope {
        fn check(&self) -> Result<()> {
            if self.start.elapsed().as_secs() >= self.seconds {
                Err(bad("potential attribution declared time ceiling reached"))
            } else {
                Ok(())
            }
        }
        fn write(&mut self, name: &str, value: &Value) -> Result<String> {
            self.check()?;
            let bytes = serde_json::to_vec_pretty(value)?;
            if self
                .written
                .saturating_add(bytes.len() as u64)
                .saturating_add(1 << 20)
                > self.bytes
            {
                return Err(bad("potential attribution complete report cap reached"));
            }
            fs::write(self.output.join(name), &bytes)?;
            self.written += bytes.len() as u64;
            Ok(sha256_bytes(&bytes))
        }
    }
    fn tree_bytes(path: &Path) -> Result<u64> {
        let mut n = 0;
        for e in fs::read_dir(path)? {
            let p = e?.path();
            n += if p.is_dir() {
                tree_bytes(&p)?
            } else {
                fs::metadata(p)?.len()
            };
        }
        Ok(n)
    }
    fn cue_payload(cp: &Path) -> Result<CueAngularQ4> {
        let meta = read(&cp.join("cue/native-metadata.json"))?;
        let config: CueAngularConfig = serde_json::from_value(meta["potential"].clone())?;
        let mut q = CueAngularQ4::new(config, &fs::read(cp.join("cue/cue-q4.bin"))?)?;
        if let Some(j) = meta.get("joint") {
            let m: CueJointMetadata = serde_json::from_value(j.clone())?;
            let joint = CueJointQ4::new(m.config, fs::read(cp.join("cue/cue-joint-q4.bin"))?)?;
            if joint.metadata() != m {
                return Err(bad("joint cue payload metadata differs"));
            }
            q = q.with_joint(joint)?;
        } else if cp.join("cue/cue-joint-q4.bin").exists() {
            return Err(bad("unreceipted joint cue payload"));
        }
        Ok(q)
    }
    fn prefix_payload(cp: &Path) -> Result<PrefixAngularQ4> {
        let config: PrefixAngularConfig = serde_json::from_value(
            read(&cp.join("prefix/native-metadata.json"))?["potential"].clone(),
        )?;
        Ok(PrefixAngularQ4::new(
            config,
            &fs::read(cp.join("prefix/prefix-q4.bin"))?,
        )?)
    }
    // Packed family-major layout. Only cell0 of content_presence in each H×L
    // lane is shared for this content-absent bank; context_presence is untouched.
    fn controlled_values(
        arm: &str,
        config: PotentialQ4Config,
        p0: &[i8],
        p1: &[i8],
    ) -> Result<BTreeMap<String, Vec<f32>>> {
        let lanes = config
            .heads
            .checked_mul(config.lanes_per_head)
            .ok_or_else(|| bad("potential lane overflow"))?;
        let expected = config.coefficient_count()?;
        if p0.len() != expected || p1.len() != expected || !ARMS.contains(&arm) {
            return Err(bad("potential control shape/arm mismatch"));
        }
        let mut result = BTreeMap::new();
        let mut at = 0;
        for (family, &count) in FAMILY_NAMES.iter().zip(&FAMILY_COUNTS) {
            let length = lanes * count;
            let mut chosen = if arm.starts_with("P0") {
                p0[at..at + length].to_vec()
            } else {
                p1[at..at + length].to_vec()
            };
            if *family == "content_presence" {
                for lane in 0..lanes {
                    let j = lane * count;
                    if arm == "P1_restore_shared0" {
                        chosen[j] = p0[at + j];
                    }
                    if arm == "P0_take_shared0" {
                        chosen[j] = p1[at + j];
                    }
                }
            }
            result.insert(
                format!("consumer.potential.{family}"),
                chosen.into_iter().map(|q| f32::from(q) * 0.25).collect(),
            );
            at += length;
        }
        Ok(result)
    }
    fn native_packet(
        model: &NativeSourceRealizer,
        generate: &NativeGeometricGenerate,
        pool: &mut NativeVocabularyActions,
        e: &Episode,
        own: &[u32],
        cueq: &CueAngularQ4,
        prefixq: &PrefixAngularQ4,
    ) -> Result<Value> {
        let (states, ids, scores, components, geometry, provenance) = if e.has_source() {
            let cue = model.compile_cue_carrier(cueq.clone())?;
            let prefix = model.compile_prefix_transport(
                &cue,
                PrefixAngularQ4::new(prefixq.config(), prefixq.packed_coefficients())?,
            )?;
            let trace = model.read_bank_with_prefix_transport(
                &e.segments()?,
                &e.input.query_ids,
                own,
                &cue,
                &prefix,
            )?;
            let bank = &trace.cue_bank.bank;
            let states = bank
                .context
                .states
                .last()
                .ok_or_else(|| bad("bank retained state missing"))?
                .iter()
                .copied()
                .map(H4Code::try_from)
                .collect::<std::result::Result<Vec<_>, _>>()?;
            let ids = bank
                .candidates
                .iter()
                .map(|c| c.occurrence.token_id)
                .collect::<Vec<_>>();
            let mut contextual = vec![0i64; ids.len()];
            let mut cues = contextual.clone();
            let mut prefixes = contextual.clone();
            let mut scores = contextual.clone();
            let width = bank.context.heads * bank.context.lanes_per_head;
            let query_at = (bank.context.tokens.len() - 1) * width;
            let query_codes = &bank.context.codes[query_at..query_at + width];
            let mut fingerprint = Vec::new();
            for h in 0..bank.heads.len() {
                for j in 0..ids.len() {
                    let c = trace.cue_bank.carrier.copy_q24[h][j];
                    let p = trace.prefix.copy_q24[h][j];
                    let raw = bank.heads[h].scores_q24[j];
                    let r = raw
                        .checked_sub(c)
                        .and_then(|v| v.checked_sub(p))
                        .ok_or_else(|| bad("context score subtraction overflow"))?;
                    scores[j] = scores[j]
                        .checked_add(raw)
                        .ok_or_else(|| bad("total Copy overflow"))?;
                    cues[j] = cues[j]
                        .checked_add(c)
                        .ok_or_else(|| bad("cue Copy overflow"))?;
                    prefixes[j] = prefixes[j]
                        .checked_add(p)
                        .ok_or_else(|| bad("prefix Copy overflow"))?;
                    contextual[j] = contextual[j]
                        .checked_add(r)
                        .ok_or_else(|| bad("context Copy overflow"))?;
                }
            }
            for candidate in &bank.candidates {
                let k = candidate.context_position * width;
                let codes = &bank.context.codes[k..k + width];
                let cells = query_codes
                    .iter()
                    .zip(codes)
                    .map(|(q, k)| 2 * usize::from(q.present) + usize::from(k.present))
                    .collect::<Vec<_>>();
                fingerprint.push(json!({"candidate":candidate,"query_observation_codes":query_codes,"candidate_observation_codes":codes,"context_presence_cells":cells}));
            }
            for j in 0..ids.len() {
                if contextual[j]
                    .checked_add(cues[j])
                    .and_then(|v| v.checked_add(prefixes[j]))
                    != Some(scores[j])
                {
                    return Err(bad("Copy component parity failed"));
                }
            }
            (
                states,
                ids,
                scores,
                json!({"contextual":contextual,"cue":cues,"prefix":prefixes}),
                json!(fingerprint),
                json!({"candidates":bank.candidates,"causal_tokens":bank.context.tokens}),
            )
        } else {
            let ids = e.no_source_tokens(own)?;
            let c = model.context_config();
            let (tables, geometry) = model.context_encoder_parts();
            let mut state = NativeContextState::new(c.heads, c.lanes_per_head)?;
            for &token in &ids {
                state.step(token as usize, tables, geometry)?;
            }
            (
                state.states().to_vec(),
                Vec::new(),
                Vec::new(),
                json!({"contextual":[],"cue":[],"prefix":[]}),
                json!([]),
                json!({"causal_tokens":ids,"no_source":true}),
            )
        };
        let mut gs = vec![0; generate.vocab_size()];
        let mut counts = GenerateReadCounts::default();
        generate.score_into(&states, &mut gs, &mut counts)?;
        let actions = pool.reduce_trace(&gs, &ids, &scores)?;
        Ok(
            json!({"actual_prefix_ids":own,"retained_state_codes":states.iter().map(|s|s.index()).collect::<Vec<_>>(),"copy_token_ids":ids,"copy_raw_scores_q24":scores,"copy_components_q24":components,"context_presence_fingerprints":geometry,"generate_raw_scores_sha256":sha256_bytes(&serde_json::to_vec(&gs)?),"pool":{"summary":actions.summary,"token_masses":actions.token_masses.iter().map(|m|[m.token_id as u64,m.weight_q31,m.generate_weight_q31,m.copy_weight_q31]).collect::<Vec<_>>()},"source_provenance":provenance,"generate_costs":counts}),
        )
    }
    fn target_stats(packet: &Value, target: u32) -> Result<Value> {
        let masses = array(&packet["pool"], "token_masses")?;
        let target_mass = masses
            .iter()
            .find(|m| m[0].as_u64() == Some(u64::from(target)))
            .and_then(|m| m[1].as_u64())
            .ok_or_else(|| bad("target mass missing"))?;
        let total = uint(&packet["pool"]["summary"], "total_weight_q31")?;
        if target_mass == 0 || total == 0 {
            return Err(bad("nonpositive target probability"));
        }
        let ids = ints(&packet["copy_token_ids"])?;
        let raw = ints(&packet["copy_raw_scores_q24"])?;
        let components = &packet["copy_components_q24"];
        let mut contrast = serde_json::Map::new();
        for name in ["contextual", "cue", "prefix"] {
            let values = ints(&components[name])?;
            let selected = values
                .iter()
                .zip(&ids)
                .filter(|(_, id)| **id == i64::from(target))
                .map(|(s, _)| *s)
                .max();
            let other = values
                .iter()
                .zip(&ids)
                .filter(|(_, id)| **id != i64::from(target))
                .map(|(s, _)| *s)
                .max();
            contrast.insert(name.into(), json!({"gold_token_occurrence_max_q24":selected,"nongold_token_occurrence_max_q24":other,"max_difference_q24":selected.zip(other).map(|(a,b)|a-b)}));
        }
        Ok(
            json!({"target_label_only":target,"source_covered_token":ids.contains(&i64::from(target)),"first_correct":uint(&packet["pool"]["summary"],"chosen_token_id")? == u64::from(target),"native_nll":-(target_mass as f64/total as f64).ln(),"native_target_mass":target_mass,"native_denominator":total,"gold_token_vs_nongold_occurrence_component_contrast":contrast,"copy_raw_max_q24":raw.into_iter().max(),"correct_record_discrimination":"NOT_MEASURED; input panel supplies no expected source fingerprint; token match is not record identity"}),
        )
    }
    fn assert_saved_diagonal(actual: &Value, saved: &Value) -> Result<()> {
        for key in [
            "actual_prefix_ids",
            "retained_state_codes",
            "copy_token_ids",
            "copy_raw_scores_q24",
            "generate_raw_scores_sha256",
            "pool",
        ] {
            if key == "pool" {
                if actual[key]["summary"] != saved[key]["summary"] {
                    return Err(bad("P1 saved diagonal pool mismatch"));
                }
            } else if actual[key] != saved[key] {
                return Err(bad(&format!("P1 saved diagonal mismatch {key}")));
            }
        }
        for key in ["candidates", "causal_tokens"] {
            if actual["source_provenance"][key] != saved["source_provenance"][key] {
                return Err(bad(
                    "P1 saved diagonal source occurrence/chronology mismatch",
                ));
            }
        }
        Ok(())
    }
    pub fn run(
        fit: &Path,
        dev_inputs: &Path,
        out: &Path,
        maximum_seconds: u64,
        maximum_bytes: u64,
    ) -> Result<()> {
        let mut envelope = Envelope {
            start: Instant::now(),
            seconds: maximum_seconds,
            bytes: maximum_bytes,
            output: out.to_owned(),
            written: 0,
        };
        report_output::verify(fit)?;
        let report = read(&fit.join("report.json"))?;
        if report["schema"] != "uor-r4.geometric-bank-generate-fit/1"
            || report["status"] != "COMPLETED"
            || report["mode"] != "fit"
            || report["arm"] != "joint-potential"
        {
            return Err(bad(
                "potential attribution requires completed explicit joint-potential fit",
            ));
        }
        let stages = array(&report, "stages")?;
        if stages.len() != 2
            || uint(&stages[0], "step")? != 0
            || uint(&stages[1], "step")? != uint(&report, "updates")?
            || uint(&report, "updates")? == 0
        {
            return Err(bad("matched initial/final checkpoints required"));
        }
        let cps = [
            fit.join("checkpoint-0000"),
            fit.join(format!("checkpoint-{:04}", uint(&stages[1], "step")?)),
        ];
        for (stage, cp) in stages.iter().zip(&cps) {
            if uint(&stage["evaluation"], "cases")? != 512
                || array(&stage["evaluation"], "rows")?.len() != 512
            {
                return Err(bad(
                    "attribution requires complete initial/final512 evaluation receipts",
                ));
            }
            if stage["checkpoint"] != read(&cp.join("receipt.json"))? {
                return Err(bad("stage/checkpoint receipt mismatch"));
            }
        }
        for name in [
            "native/tokenizer.json",
            "native/period-q4.bin",
            "native/consumer/no-read-q4.bin",
            "native/consumer/exp-q31.bin",
            "cue/cue-q4.bin",
            "prefix/prefix-q4.bin",
        ] {
            if fs::read(cps[0].join(name))? != fs::read(cps[1].join(name))? {
                return Err(bad("attribution frozen nonpotential operator changed"));
            }
        }
        let cq0 = cue_payload(&cps[0])?;
        let cq1 = cue_payload(&cps[1])?;
        if cq0.config() != cq1.config()
            || cq0.packed_coefficients() != cq1.packed_coefficients()
            || cq0.joint().map(|j| j.metadata()) != cq1.joint().map(|j| j.metadata())
            || cq0.joint().map(|j| j.packed_coefficients())
                != cq1.joint().map(|j| j.packed_coefficients())
        {
            return Err(bad("frozen cue numeric payload/config changed"));
        }
        let pq0 = prefix_payload(&cps[0])?;
        let pq1 = prefix_payload(&cps[1])?;
        if pq0.config() != pq1.config() {
            return Err(bad("frozen prefix configuration changed"));
        }
        let admission = read(&fit.join("input-admission.json"))?;
        let hashes = admission["input_sha256"]
            .as_object()
            .ok_or_else(|| bad("input hash inventory missing"))?;
        let dev_bytes = fs::read(dev_inputs)?;
        let dev_sha = sha256_bytes(&dev_bytes);
        if !hashes
            .values()
            .any(|v| v.as_str() == Some(dev_sha.as_str()))
        {
            return Err(bad("supplied development inputs not bound by fit"));
        }
        let inputs: Inputs = serde_json::from_slice(&dev_bytes)?;
        let frozen_oracles = read(&fit.join("frozen-development-answer-oracles.json"))?;
        let oracles = array(&frozen_oracles, "cases")?;
        let final_refs = array(&stages[1]["evaluation"], "rows")?;
        if inputs.schema != "uor-r4.native-source-bank-probe-input/1"
            || inputs.cases.len() != 512
            || final_refs.len() != 512
            || oracles.len() != 512
        {
            return Err(bad("complete512 bound development panel required"));
        }
        let donor_binding: NativeArtifactBinding =
            serde_json::from_value(read(&cps[0].join("receipt.json"))?["parent"].clone())?;
        let donor_native =
            NativeSourceRealizer::load_native(&cps[0].join("native"), &donor_binding)?;
        let final_binding: NativeArtifactBinding =
            serde_json::from_value(read(&cps[1].join("receipt.json"))?["parent"].clone())?;
        let admitted_final =
            NativeSourceRealizer::load_native(&cps[1].join("native"), &final_binding)?;
        if donor_native.binding() != admitted_final.binding()
            || donor_native.context_config() != admitted_final.context_config()
        {
            return Err(bad(
                "donor/final native vocabulary/context configuration differs",
            ));
        }
        let final_context_config = admitted_final.context_config();
        drop(donor_native);
        drop(admitted_final);
        let tokbytes = fs::read(cps[1].join("native/tokenizer.json"))?;
        let tok = ByteBpeTokenizer::from_tokenizer_json_bytes(&tokbytes)
            .ok_or_else(|| bad("tokenizer unavailable"))?;
        let binding = SourceActionBinding::new(&tokbytes)?;
        let final_source = SourceRealizerWeights::load_source(&cps[1].join("source"), &tokbytes)?;
        let identity: ConsumerIdentity = serde_json::from_value(
            read(&cps[1].join("native/metadata.json"))?["identity"].clone(),
        )?;
        let final_native = LearningNative::load(&cps[1].join("native"), &final_source, &identity)?;
        if serde_json::to_value(final_native.artifact_binding()?)?
            != read(&cps[1].join("receipt.json"))?["parent"]
        {
            return Err(bad("final native source/checkpoint identity differs"));
        }
        let generate_bytes = fs::read(cps[1].join("generate.bin"))?;
        if read(&cps[1].join("receipt.json"))?["generate_sha256"] != sha256_bytes(&generate_bytes) {
            return Err(bad("final Generate receipt differs"));
        }
        let generate = NativeGeometricGenerate::from_bytes(&generate_bytes, &binding)?;
        if generate.to_bytes()? != generate_bytes
            || generate.lanes() != final_context_config.heads * final_context_config.lanes_per_head
        {
            return Err(bad("final Generate reload/context lane dimensions differ"));
        }
        let config0: PotentialQ4Config = serde_json::from_value(
            read(&cps[0].join("native/consumer/metadata.json"))?["potential"].clone(),
        )?;
        let config: PotentialQ4Config = serde_json::from_value(
            read(&cps[1].join("native/consumer/metadata.json"))?["potential"].clone(),
        )?;
        if config0 != config {
            return Err(bad("potential configuration changed"));
        }
        let coefficient_count = config.coefficient_count()?;
        let p0 = unpack_coefficients(
            coefficient_count,
            &fs::read(cps[0].join("native/consumer/potential-q4.bin"))?,
        )?;
        let p1 = unpack_coefficients(
            coefficient_count,
            &fs::read(cps[1].join("native/consumer/potential-q4.bin"))?,
        )?;
        let params = final_source.potential_parameters();
        let original = params
            .iter()
            .map(|(n, v)| Ok((n.clone(), v.flatten_all()?.to_vec1::<f32>()?)))
            .collect::<Result<BTreeMap<_, _>>>()?;
        let projected_native_bytes = tree_bytes(&cps[1].join("native"))?
            .saturating_mul(4)
            .saturating_add(1 << 20);
        if projected_native_bytes > maximum_bytes.saturating_sub(1 << 20) {
            return Err(bad("native artifact projection exceeds report cap"));
        }
        let mut controls = Vec::new();
        let mut control_receipts = Vec::new();
        fs::create_dir(out.join("controls"))?;
        for arm in ARMS {
            envelope.check()?;
            for (name, var) in &params {
                var.set(&Tensor::from_vec(
                    original[name].clone(),
                    var.shape(),
                    &Device::Cpu,
                )?)?;
            }
            if arm != "P1" {
                let values = controlled_values(arm, config, &p0, &p1)?;
                for (name, var) in &params {
                    if arm == "P1_restore_shared0" {
                        if name == "consumer.potential.content_presence" {
                            let mut values0 = original[name].clone();
                            for lane in 0..config.heads * config.lanes_per_head {
                                values0[lane * 4] = values[name][lane * 4];
                            }
                            var.set(&Tensor::from_vec(values0, var.shape(), &Device::Cpu)?)?;
                        }
                    } else {
                        var.set(&Tensor::from_vec(
                            values[name].clone(),
                            var.shape(),
                            &Device::Cpu,
                        )?)?;
                    }
                }
            }
            let rebuilt = final_source.compile_context_potential_rebound(&final_native)?;
            let root = out.join("controls").join(arm);
            fs::create_dir(&root)?;
            rebuilt.save(&root.join("native"))?;
            let actual_binding = rebuilt.execution_binding()?;
            if arm == "P1" && actual_binding != final_native.artifact_binding()? {
                return Err(bad("P1 exact trained source execution identity changed"));
            }
            let model = NativeSourceRealizer::load_native(&root.join("native"), &actual_binding)?;
            for name in [
                "tokenizer.json",
                "period-q4.bin",
                "consumer/context-q4.bin",
                "consumer/no-read-q4.bin",
                "consumer/exp-q31.bin",
            ] {
                if fs::read(root.join("native").join(name))?
                    != fs::read(cps[1].join("native").join(name))?
                {
                    return Err(bad(
                        "attribution changed final context or frozen numerical payload",
                    ));
                }
            }
            let cue = model.compile_cue_carrier(cq1.clone())?;
            let prefix = model.compile_prefix_transport(
                &cue,
                PrefixAngularQ4::new(pq1.config(), pq1.packed_coefficients())?,
            )?;
            if cue.packed_coefficients() != cq1.packed_coefficients()
                || prefix.packed_coefficients() != pq1.packed_coefficients()
                || cue.joint().map(|j| j.packed_coefficients())
                    != cq1.joint().map(|j| j.packed_coefficients())
            {
                return Err(bad("control rebind changed cue/prefix numeric bytes"));
            }
            fs::write(
                root.join("cue-native-metadata.json"),
                serde_json::to_vec_pretty(cue.metadata())?,
            )?;
            fs::write(
                root.join("prefix-native-metadata.json"),
                serde_json::to_vec_pretty(prefix.metadata())?,
            )?;
            let packed = fs::read(root.join("native/consumer/potential-q4.bin"))?;
            let expected_values = controlled_values(arm, config, &p0, &p1)?;
            let expected_quarters = FAMILY_NAMES
                .iter()
                .flat_map(|family| {
                    expected_values[&format!("consumer.potential.{family}")]
                        .iter()
                        .map(|v| (v * 4.).round() as i8)
                })
                .collect::<Vec<_>>();
            if unpack_coefficients(coefficient_count, &packed)? != expected_quarters {
                return Err(bad(
                    "compiled control packed coefficient assignment differs",
                ));
            }
            control_receipts.push(json!({"arm":arm,"actual_artifact_binding":actual_binding,"potential_packed_sha256":sha256_bytes(&packed),"cue_metadata":cue.metadata(),"prefix_metadata":prefix.metadata(),"numeric_cue_prefix_sha256":[sha256_bytes(cue.packed_coefficients()),sha256_bytes(prefix.packed_coefficients())],"source_scope":if arm=="P1" {"exact trained final source"} else {"diagnostic source coefficient rebind; same exact final context; no learned or transplanted metadata"},"independent_native_reload":true}));
            drop(prefix);
            drop(cue);
            controls.push(model);
            envelope.written = tree_bytes(out)?;
            if envelope.written.saturating_add(1 << 20) > maximum_bytes {
                return Err(bad("control artifacts exceed declared cap"));
            }
        }
        let exp = fs::read(cps[1].join("native/consumer/exp-q31.bin"))?;
        let mut seen = BTreeSet::new();
        let mut row_refs = Vec::new();
        let mut counts = vec![
            json!({"complete":0,"first_correct":0,"canonical_source_covered_positions":0,"canonical_generate_only_positions":0,"canonical_source_covered_correct":0,"canonical_generate_only_correct":0,"canonical_nll_sum":0.0,"ownprefix_source_covered_emissions":0,"ownprefix_generate_only_emissions":0});
            4
        ];
        for (index, input) in inputs.cases.into_iter().enumerate() {
            envelope.check()?;
            if input.id.is_empty()
                || !seen.insert(input.id.clone())
                || oracles[index]["id"] != input.id
                || final_refs[index]["id"] != input.id
                || !input.actual_prefix_ids.is_empty()
                || input.query_ids.is_empty()
            {
                return Err(bad("bound panel/oracle/final row identity mismatch"));
            }
            let path = relative(
                fit,
                final_refs[index]["row_file"]
                    .as_str()
                    .ok_or_else(|| bad("saved row path missing"))?,
            )?;
            let bytes = fs::read(path)?;
            if final_refs[index]["row_sha256"] != sha256_bytes(&bytes) {
                return Err(bad("saved final row SHA mismatch"));
            }
            let saved: Value = serde_json::from_slice(&bytes)?;
            if saved["id"] != input.id {
                return Err(bad("saved row ID differs"));
            }
            let targets = ints(&oracles[index]["canonical_ids_labels_only"])?
                .into_iter()
                .map(u32::try_from)
                .collect::<std::result::Result<Vec<_>, _>>()?;
            if targets.is_empty()
                || targets.len() > 32
                || targets.last().copied() != Some(binding.eos_token_id())
                || json!(targets) != saved["canonical_target_ids_labels_only"]
            {
                return Err(bad("bound canonical labels differ"));
            }
            let answers: FrozenAnswers = serde_json::from_value(oracles[index]["answers"].clone())?;
            answers.validate()?;
            let views = input
                .segments
                .iter()
                .map(|s| {
                    s.frame()
                        .map(|f| controls[1].compile_view(f.token_ids))
                        .transpose()
                })
                .collect::<std::result::Result<Vec<_>, _>>()?;
            let episode = Episode { input, views };
            if episode.base_len() + targets.len() > 128 {
                return Err(bad("full canonical prefix exceeds admitted context"));
            }
            let mut arms = serde_json::Map::new();
            let mut canonical_reference = Vec::new();
            for (arm_index, arm) in ARMS.iter().enumerate() {
                let mut pool = NativeVocabularyActions::new(binding.clone(), &exp)?;
                let mut canonical = Vec::new();
                let mut row_nll = 0.;
                for (t, &target) in targets.iter().enumerate() {
                    envelope.check()?;
                    let mut packet = native_packet(
                        &controls[arm_index],
                        &generate,
                        &mut pool,
                        &episode,
                        &targets[..t],
                        &cq1,
                        &pq1,
                    )?;
                    if arm_index == 0 {
                        canonical_reference.push((
                            packet["retained_state_codes"].clone(),
                            packet["generate_raw_scores_sha256"].clone(),
                            packet["copy_token_ids"].clone(),
                        ));
                    } else if canonical_reference[t]
                        != (
                            packet["retained_state_codes"].clone(),
                            packet["generate_raw_scores_sha256"].clone(),
                            packet["copy_token_ids"].clone(),
                        )
                    {
                        return Err(bad(
                            "fixed-prefix control context/Generate/occurrence identity differs",
                        ));
                    }
                    if arm_index == 1 {
                        assert_saved_diagonal(&packet, &array(&saved, "canonical")?[t]["native"])?;
                    }
                    let stats = target_stats(&packet, target)?;
                    let covered = stats["source_covered_token"] == true;
                    let group = if covered {
                        "canonical_source_covered_positions"
                    } else {
                        "canonical_generate_only_positions"
                    };
                    counts[arm_index][group] = json!(uint(&counts[arm_index], group)? + 1);
                    if stats["first_correct"] == true {
                        let group = if covered {
                            "canonical_source_covered_correct"
                        } else {
                            "canonical_generate_only_correct"
                        };
                        counts[arm_index][group] = json!(uint(&counts[arm_index], group)? + 1);
                    }
                    row_nll += stats["native_nll"]
                        .as_f64()
                        .ok_or_else(|| bad("canonical NLL missing"))?;
                    packet["pool"]
                        .as_object_mut()
                        .ok_or_else(|| bad("pool missing"))?
                        .remove("token_masses");
                    canonical.push(json!({"stats_labels_only":stats,"native":packet}));
                }
                counts[arm_index]["canonical_nll_sum"] = json!(
                    counts[arm_index]["canonical_nll_sum"]
                        .as_f64()
                        .ok_or_else(|| bad("summary NLL missing"))?
                        + row_nll / targets.len() as f64
                );
                counts[arm_index]["first_correct"] = json!(
                    uint(&counts[arm_index], "first_correct")?
                        + u64::from(canonical[0]["stats_labels_only"]["first_correct"] == true)
                );
                let allowance = 32usize.min(128usize.saturating_sub(episode.base_len()));
                let mut own = Vec::new();
                let mut generated = Vec::new();
                let mut eos = false;
                for t in 0..allowance {
                    envelope.check()?;
                    let mut packet = native_packet(
                        &controls[arm_index],
                        &generate,
                        &mut pool,
                        &episode,
                        &own,
                        &cq1,
                        &pq1,
                    )?;
                    if arm_index == 1 {
                        assert_saved_diagonal(&packet, &array(&saved, "generation")?[t])?;
                    }
                    let chosen =
                        u32::try_from(uint(&packet["pool"]["summary"], "chosen_token_id")?)?;
                    let chosen_mass = array(&packet["pool"], "token_masses")?
                        .iter()
                        .find(|m| m[0].as_u64() == Some(u64::from(chosen)))
                        .ok_or_else(|| bad("chosen alias mass missing"))?
                        .clone();
                    let covered = ints(&packet["copy_token_ids"])?.contains(&i64::from(chosen));
                    let group = if covered {
                        "ownprefix_source_covered_emissions"
                    } else {
                        "ownprefix_generate_only_emissions"
                    };
                    counts[arm_index][group] = json!(uint(&counts[arm_index], group)? + 1);
                    packet["emission_attribution"] = json!({"chosen_source_covered_token":covered,"chosen_generate_component_mass_q31":chosen_mass[2],"chosen_copy_component_mass_q31":chosen_mass[3],"scope":"actual emitted token support, not correct-source identity or correctness at diverged prefixes"});
                    packet["pool"]
                        .as_object_mut()
                        .ok_or_else(|| bad("pool missing"))?
                        .remove("token_masses");
                    // Keep typed observations on canonical prefixes; own-prefix
                    // records retain actual numeric components and their digest.
                    let fingerprint = packet
                        .as_object_mut()
                        .ok_or_else(|| bad("packet missing"))?
                        .remove("context_presence_fingerprints")
                        .ok_or_else(|| bad("fingerprint missing"))?;
                    packet["context_presence_fingerprints_sha256"] =
                        json!(sha256_bytes(&serde_json::to_vec(&fingerprint)?));
                    generated.push(packet);
                    own.push(chosen);
                    if chosen == binding.eos_token_id() {
                        eos = true;
                        break;
                    }
                }
                let text = tok.decode(&own[..own.len() - usize::from(eos)]);
                let complete = eos && answers.accepts(&text);
                counts[arm_index]["complete"] =
                    json!(uint(&counts[arm_index], "complete")? + u64::from(complete));
                if arm_index == 1
                    && (saved["generated_ids"] != json!(own) || saved["complete"] != complete)
                {
                    return Err(bad("P1 own-prefix saved output differs"));
                }
                arms.insert((*arm).to_owned(),json!({"canonical":canonical,"native_equal_episode_ce":row_nll/targets.len() as f64,"generation":generated,"generated_ids":own,"decoded":text,"eos":eos,"complete":complete,"generation_allowance":allowance,"cutoff":if eos {None} else if allowance<32 {Some("context-cap")} else {Some("generation-cap")}}));
            }
            let name = format!("attribution-row-{index:04}.json");
            let sha=envelope.write(&name,&json!({"id":episode.input.id,"saved_final_row_sha256":sha256_bytes(&bytes),"arms":arms}))?;
            row_refs.push(json!({"id":episode.input.id,"row_file":name,"row_sha256":sha}));
        }
        report_output::verify(fit)?;
        if sha256_bytes(&fs::read(dev_inputs)?) != dev_sha {
            return Err(bad("bound development inputs changed during audit"));
        }
        let summaries = ARMS
            .iter()
            .enumerate()
            .map(|(i, name)| ((*name).to_owned(), counts[i].clone()))
            .collect::<BTreeMap<_, _>>();
        let value = json!({"schema":"uor-r4.geometric-potential-attribution/1","status":"COMPLETED","updates":0,"cases":512,"scope":"four independently compiled native potential controls at exact final context+Generate; all source occurrences retained; unchanged numeric cue/prefix rebound to actual parents; common native full-vocabulary alias pool; canonical-prefix attribution and actual-ownprefix outputs separate","control_scope":"cell0 of content_presence in every H×L lane is shared for content-absent endpoints; context_presence cells are preserved and their actual endpoint fingerprints recorded; not assumed uniform","correct_record_discrimination":"NOT_MEASURED; gold token occurrence versus nongold contrast does not establish correct-source identity","source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT"),"executable_sha256":sha256_bytes(&fs::read(std::env::current_exe()?)?),"fit_report_sha256":sha256_bytes(&fs::read(fit.join("report.json"))?),"fit_manifest_sha256":sha256_bytes(&fs::read(fit.join("manifest.json"))?),"bound_development_inputs_sha256":dev_sha,"final_generate_sha256":sha256_bytes(&generate_bytes),"controls":control_receipts,"P1_all_canonical_and_ownprefix_diagonals_verified":true,"arms":summaries,"rows":row_refs,"elapsed_seconds":envelope.start.elapsed().as_secs_f64(),"maximum_seconds":maximum_seconds,"maximum_report_bytes":maximum_bytes,"report_bytes_before_summary":envelope.written});
        envelope.write("report.json", &value)?;
        Ok(())
    }
    #[cfg(test)]
    mod tests {
        use super::*;
        #[test]
        fn shared_cell_ablation_separates_constant_offset_from_context_presence() -> Result<()> {
            use uor_r4_integer::{
                geometric_potential::AddressLane,
                geometric_potential_q4::{pack_coefficients, NativePotentialQ4},
                h4_tables::HistoricalH4Tables,
            };
            let config = PotentialQ4Config {
                heads: 1,
                lanes_per_head: 2,
            };
            let n = config.coefficient_count()?;
            let p0 = vec![0; n];
            let mut p1 = p0.clone();
            let cp_at = FAMILY_COUNTS[..5].iter().sum::<usize>() * 2;
            let rp_at = cp_at + FAMILY_COUNTS[5] * 2;
            for lane in 0..2 {
                p1[cp_at + lane * 4] = -5;
                p1[rp_at + lane * 4 + 1] = 2;
                p1[rp_at + lane * 4 + 3] = -3;
            }
            let models = ARMS
                .iter()
                .map(|arm| {
                    let values = controlled_values(arm, config, &p0, &p1)?;
                    let quarters = FAMILY_NAMES
                        .iter()
                        .flat_map(|family| {
                            values[&format!("consumer.potential.{family}")]
                                .iter()
                                .map(|v| (v * 4.).round() as i8)
                        })
                        .collect::<Vec<_>>();
                    Ok(NativePotentialQ4::new(
                        config,
                        &pack_coefficients(&quarters)?,
                    )?)
                })
                .collect::<Result<Vec<_>>>()?;
            let geometry = HistoricalH4Tables::from_bytes(include_bytes!(
                "../../uor-r4-integer/fixtures/historical-h4-tables-v1.bin"
            ))?;
            let absent = AddressLane::new(H4Code::IDENTITY.index(), 0, false)?;
            let present = AddressLane::new(9, 7, true)?;
            let mut selective = Vec::new();
            for query in [absent, present] {
                let scores = models
                    .iter()
                    .map(|model| {
                        Ok(model.score(
                            0,
                            &[absent; 2],
                            &[absent; 2],
                            &[query; 2],
                            &[present; 2],
                            &geometry,
                        )?)
                    })
                    .collect::<Result<Vec<_>>>()?;
                assert_eq!(scores[1] - scores[2], -(10i64 << 22));
                assert_eq!(scores[3] - scores[0], -(10i64 << 22));
                selective.push(scores[2]);
            }
            assert_ne!(
                selective[0], selective[1],
                "restoring shared content presence must retain selective context-presence cells"
            );
            Ok(())
        }

        #[test]
        fn shared_cell_controls_leave_context_presence_and_other_lanes_cells_intact() -> Result<()>
        {
            let config = PotentialQ4Config {
                heads: 2,
                lanes_per_head: 2,
            };
            let count = config.coefficient_count()?;
            let p0 = vec![-1; count];
            let p1 = vec![2; count];
            let restored = controlled_values("P1_restore_shared0", config, &p0, &p1)?;
            let isolated = controlled_values("P0_take_shared0", config, &p0, &p1)?;
            for (name, values) in restored {
                let other = &isolated[&name];
                for (i, &value) in values.iter().enumerate() {
                    let shared = name == "consumer.potential.content_presence" && i % 4 == 0;
                    assert_eq!(value, if shared { -0.25 } else { 0.5 });
                    assert_eq!(other[i], if shared { 0.5 } else { -0.25 });
                }
            }
            assert!(controlled_values("bad", config, &p0, &p1).is_err());
            Ok(())
        }
    }
}

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.first().is_some_and(|a| a == "--potential-attribution") {
        if args.len() != 6 {
            return Err(bad("usage: --potential-attribution FIT_ROOT BOUND_DEV_INPUTS OUTPUT MAX_SECONDS MAX_REPORT_BYTES"));
        }
        let fit = PathBuf::from(&args[1]);
        let dev = PathBuf::from(&args[2]);
        let out = PathBuf::from(&args[3]);
        let seconds = args[4]
            .to_str()
            .ok_or_else(|| bad("invalid seconds"))?
            .parse::<u64>()?;
        let bytes = args[5]
            .to_str()
            .ok_or_else(|| bad("invalid report bytes"))?
            .parse::<u64>()?;
        if seconds == 0
            || seconds > 5100
            || bytes < 64 << 20
            || bytes > 512 << 20
            || !fit.is_dir()
            || !dev.is_file()
        {
            return Err(bad("potential attribution declared resource/input bounds"));
        }
        admit_output(&fit, &out)?;
        let prospective = output_support::prospective_output(&out)?;
        let original = fs::canonicalize(&dev)?;
        if prospective.starts_with(&original) || original.starts_with(&prospective) {
            return Err(bad("attribution output/development input overlap"));
        }
        for ancestor in original.ancestors() {
            if ancestor.join("manifest.json").is_file() && prospective.starts_with(ancestor) {
                return Err(bad("attribution output inside sealed input"));
            }
        }
        report_output::claim(&out)?;
        let result = potential_attribution::run(&fit, &dev, &out, seconds, bytes);
        if let Err(error) = &result {
            fs::write(
                out.join("failure.json"),
                serde_json::to_vec_pretty(
                    &json!({"status":"FAILED","error":error.to_string(),"scope":"attribution instrument failure, not model verdict"}),
                )?,
            )?;
        }
        report_output::seal(&out)?;
        report_output::verify(&out)?;
        return result;
    }
    if args.len() != 2 {
        return Err(bad(
            "usage: geometric-generate-checkpoint-cross FIT_ROOT REPORT_OUTPUT",
        ));
    }
    let fit = PathBuf::from(&args[0]);
    let out = PathBuf::from(&args[1]);
    if !fit.is_dir() {
        return Err(bad("fit root missing"));
    }
    admit_output(&fit, &out)?;
    report_output::claim(&out)?;
    let result = audit(&fit, &out, Instant::now());
    if let Err(e) = &result {
        fs::write(
            out.join("failure.json"),
            serde_json::to_vec_pretty(
                &json!({"status":"FAILED","error":e.to_string(),"scope":"audit failure, not a model verdict"}),
            )?,
        )?;
    }
    report_output::seal(&out)?;
    report_output::verify(&out)?;
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn overlapping_output_rejected_without_mutation() -> Result<()> {
        let root =
            std::env::temp_dir().join(format!("checkpoint-cross-overlap-{}", std::process::id()));
        fs::create_dir(&root)?;
        let result = (|| {
            let fit = root.join("fit");
            fs::create_dir(&fit)?;
            for out in [root.clone(), fit.clone(), fit.join("missing/attempt")] {
                assert!(admit_output(&fit, &out).is_err());
            }
            assert!(!fit.join("missing").exists());
            assert!(!fit.join("attempt.json").exists());
            fs::write(fit.join("manifest.json"), b"{}")?;
            assert!(admit_output(&fit, &fit.join("missing/attempt")).is_err());
            assert!(!fit.join("missing").exists());
            Ok(())
        })();
        fs::remove_dir_all(root)?;
        result
    }
    const TOK: &str = r#"{"pre_tokenizer":{"type":"ByteLevel","add_prefix_space":false},"model":{"type":"BPE","vocab":{"<|bos|>":0,"<|eos|>":1,"<|unk|>":2,".":3,"b":4},"merges":[]},"added_tokens":[{"id":0,"content":"<|bos|>"},{"id":1,"content":"<|eos|>"},{"id":2,"content":"<|unk|>"}]}"#;
    #[test]
    fn alias_competition_survives_surgical_copy_swap() -> Result<()> {
        let binding = SourceActionBinding::new(TOK.as_bytes())?;
        // Offline fixture formula; the public constructor authenticates it.
        let table = (0..uor_r4_integer::geometric_read::EXP_TABLE_LEN)
            .flat_map(|i| {
                (((-(i as f64) / 256.).exp() * (1u64 << 31) as f64).round() as u32).to_le_bytes()
            })
            .collect::<Vec<_>>();
        let mut pool = NativeVocabularyActions::new(binding, &table)?;
        let gen = [-4 << 24, -4 << 24, -4 << 24, 1 << 24, 0];
        let no_alias = pool.reduce_trace(&gen, &[4], &[0])?;
        let duplicate = pool.reduce_trace(&gen, &[4, 4, 4], &[0, 0, 0])?;
        assert_eq!(no_alias.summary.chosen_token_id, 3);
        assert_eq!(duplicate.summary.chosen_token_id, 4);
        let m = metrics(&duplicate, &gen, &pool, 3)?;
        assert_eq!(m["best_generate_raw_score_token_id"], 3);
        assert_eq!(m["first_correct"], false);
        assert!(m["gold_vs_best_nongold_margin_q31"]
            .as_i64()
            .is_some_and(|v| v < 0));
        Ok(())
    }
    #[test]
    fn first_packet_and_occurrence_admission_reject_mismatch() -> Result<()> {
        let p = json!({"actual_prefix_ids":[],"retained_state_codes":[1],"generate_raw_scores_sha256":"bound","copy_token_ids":[3],"copy_raw_scores_q24":[0],"pool":{"summary":{"chosen_token_id":3}},"source_provenance":{"candidates":[{"event":1}],"causal_tokens":[3]}});
        let mut r = json!({"generation":[p.clone()],"canonical":[{"native":p.clone()}],"generated_ids":[3]});
        assert!(first_packet(&r).is_ok());
        r["generation"][0]["actual_prefix_ids"] = json!([3]);
        assert!(first_packet(&r).is_err());
        r["generation"][0]["actual_prefix_ids"] = Value::Null;
        assert!(first_packet(&r).is_err());
        r["generation"][0] = p.clone();
        r["canonical"][0]["native"]["copy_raw_scores_q24"] = json!([1]);
        assert!(first_packet(&r).is_err());
        let mut other = p.clone();
        other["copy_token_ids"] = json!([4]);
        assert!(check_matched_packets(&p, &other).is_err());
        other = p.clone();
        other["source_provenance"]["candidates"][0]["event"] = json!(2);
        assert!(check_matched_packets(&p, &other).is_err());
        Ok(())
    }
    #[test]
    fn raw_score_rank_and_common_reference_component_ties_are_distinct() -> Result<()> {
        let binding = SourceActionBinding::new(TOK.as_bytes())?;
        let table = (0..uor_r4_integer::geometric_read::EXP_TABLE_LEN)
            .flat_map(|i| {
                (((-(i as f64) / 256.).exp() * (1u64 << 31) as f64).round() as u32).to_le_bytes()
            })
            .collect::<Vec<_>>();
        let mut pool = NativeVocabularyActions::new(binding, &table)?;
        let gen = [
            -(8 << 24),
            8 << 24,
            -(8 << 24),
            -(8 << 24) + 1,
            -(8 << 24) + 2,
        ];
        let trace = pool.reduce_trace(&gen, &[], &[])?;
        let m = metrics(&trace, &gen, &pool, 4)?;
        assert!(
            m["gold_common_reference_generate_component_rank_with_smallest_id_ties"].as_u64()
                > m["gold_generate_raw_score_rank_with_smallest_id_ties"].as_u64()
        );
        Ok(())
    }
}
