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
fn main() -> Result<()> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
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
