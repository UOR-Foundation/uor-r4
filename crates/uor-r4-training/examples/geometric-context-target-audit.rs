//! Frozen native context information audit, not a JEPA learner or language test.
use serde::Deserialize;
use serde_json::{json, Value};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs, io,
    path::{Path, PathBuf},
    time::Instant,
};
use uor_r4_core::report_output;
use uor_r4_integer::{
    geometric_context::NativeContextState,
    geometric_source_realizer::{NativeArtifactBinding, NativeSourceRealizer},
};
use uor_r4_tokenizer::ByteBpeTokenizer;
use uor_r4_training::{sha256_bytes, sha256_file};
#[path = "../../uor-r4-integer/examples/support/source_probe.rs"]
mod output_support;
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
fn bad(s: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, s)
}
struct Args {
    fit_root: PathBuf,
    checkpoint_step: usize,
    input: PathBuf,
    out: PathBuf,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Input {
    schema: String,
    pairs: Vec<Pair>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Pair {
    id: String,
    kind: String,
    a: String,
    b: String,
}
fn args() -> Result<Args> {
    let mut fields = BTreeMap::new();
    let mut it = std::env::args().skip(1);
    while let Some(key) = it.next() {
        if !["--fit-root", "--checkpoint-step", "--pairs", "--out"].contains(&key.as_str()) {
            return Err(bad("unknown flag").into());
        }
        let value = it.next().ok_or_else(|| bad("missing flag value"))?;
        if fields.insert(key, value).is_some() {
            return Err(bad("duplicate flag").into());
        }
    }
    let mut take = |key: &str| -> Result<PathBuf> {
        Ok(fs::canonicalize(
            fields
                .remove(key)
                .ok_or_else(|| bad("required flag absent"))?,
        )?)
    };
    let fit_root = take("--fit-root")?;
    let input = take("--pairs")?;
    let checkpoint_step: usize = fields
        .remove("--checkpoint-step")
        .unwrap_or_else(|| "0".into())
        .parse()?;
    if ![0, 128].contains(&checkpoint_step) {
        return Err(bad("checkpoint-step must be declared0 or separately requested128").into());
    }
    let out = output_support::prospective_output(Path::new(
        &fields
            .remove("--out")
            .ok_or_else(|| bad("--out required"))?,
    ))?;
    if out.starts_with(&fit_root) || fit_root.starts_with(&out) || input.starts_with(&out) {
        return Err(bad("input/output overlap").into());
    }
    Ok(Args {
        fit_root,
        checkpoint_step,
        input,
        out,
    })
}
fn checkpoints(length: usize) -> Result<[usize; 3]> {
    if !(1..=128).contains(&length) {
        return Err(bad("span requires 1..=128 actual tokenizer tokens").into());
    }
    Ok([length.div_ceil(3), (2 * length).div_ceil(3), length])
}
fn snapshot_key(trajectory: &[Vec<u8>], at: [usize; 3]) -> Result<Vec<u8>> {
    let mut result = Vec::new();
    for n in at {
        result.extend_from_slice(
            trajectory
                .get(n.checked_sub(1).ok_or_else(|| bad("zero checkpoint"))?)
                .ok_or_else(|| bad("checkpoint outside trajectory"))?,
        );
    }
    Ok(result)
}
struct Encoded {
    value: Value,
    trajectory: Vec<Vec<u8>>,
    endpoint: Vec<u8>,
    triple: Vec<u8>,
    ids: Vec<u32>,
}
fn encode(model: &NativeSourceRealizer, tok: &ByteBpeTokenizer, text: &str) -> Result<Encoded> {
    if text.is_empty() {
        return Err(bad("empty exact text span").into());
    }
    let ids = tok.encode(text);
    let at = checkpoints(ids.len())?;
    if tok.decode_bytes(&ids) != text.as_bytes()
        || ids.iter().any(|&t| !model.binding().admits_token(t))
    {
        return Err(bad("text/tokenizer exact-byte or token admission mismatch").into());
    }
    let cfg = model.context_config();
    let (tables, geometry) = model.context_encoder_parts();
    let mut state = NativeContextState::new(cfg.heads, cfg.lanes_per_head)?;
    let width = cfg
        .heads
        .checked_mul(cfg.lanes_per_head)
        .ok_or_else(|| bad("lane width overflow"))?;
    let mut trajectory = Vec::new();
    let mut actions = Vec::new();
    let mut reads = 0usize;
    for &id in &ids {
        let step = state.step(id as usize, tables, geometry)?;
        let codes = state.states().iter().map(|c| c.index()).collect::<Vec<_>>();
        if codes
            != step.states[..width]
                .iter()
                .map(|c| c.index())
                .collect::<Vec<_>>()
        {
            return Err(bad("native step/state receipt mismatch").into());
        }
        trajectory.push(codes);
        actions.push(
            step.actions[..width]
                .iter()
                .map(|c| c.index())
                .collect::<Vec<_>>(),
        );
        reads = reads
            .checked_add(step.coefficient_reads)
            .ok_or_else(|| bad("logical read count overflow"))?;
    }
    let endpoint = trajectory
        .last()
        .ok_or_else(|| bad("empty native trajectory"))?
        .clone();
    let triple = snapshot_key(&trajectory, at)?;
    let snapshots = at
        .iter()
        .map(|&n| trajectory[n - 1].clone())
        .collect::<Vec<_>>();
    let value = json!({"text_bytes_sha256":sha256_bytes(text.as_bytes()),"exact_text":text,"token_ids":ids,"token_count":ids.len(),"signed_h4_state_codes_by_token":trajectory,"signed_h4_actions_by_token":actions,"checkpoint_positions_1based":at,"duplicate_checkpoint_positions":at.iter().collect::<BTreeSet<_>>().len()!=3,"checkpoint_signed_states":snapshots,"endpoint_signed_states":endpoint,"context_coefficient_reads":reads});
    Ok(Encoded {
        value,
        trajectory,
        endpoint,
        triple,
        ids,
    })
}
fn write(root: &Path, name: &str, value: &Value) -> Result<()> {
    let bytes = serde_json::to_vec(value)?;
    if bytes.len() > 64 << 20 {
        return Err(bad("report file exceeds64MiB").into());
    }
    fs::write(root.join(name), bytes)?;
    Ok(())
}
fn run(a: &Args, start: Instant) -> Result<Value> {
    let inputbytes = fs::read(&a.input)?;
    if inputbytes.len() > 4 << 20 {
        return Err(bad("pair input exceeds4MiB").into());
    }
    let input: Input = serde_json::from_slice(&inputbytes)?;
    if input.schema != "uor-r4.contextual-target-pairs/1"
        || input.pairs.is_empty()
        || input.pairs.len() > 512
    {
        return Err(bad("schema/count require 1..=512 predeclared pairs").into());
    }
    report_output::verify(&a.fit_root)?;
    let manifest_sha = sha256_file(&a.fit_root.join("manifest.json"))?;
    let fitbytes = fs::read(a.fit_root.join("report.json"))?;
    let fit: Value = serde_json::from_slice(&fitbytes)?;
    if fit["status"] != json!("COMPLETED") {
        return Err(
            bad("requires completed sealed original fit; live unsealed fits unsupported").into(),
        );
    }
    let checkpoint = a
        .fit_root
        .join(format!("checkpoint-{:04}", a.checkpoint_step));
    let receiptbytes = fs::read(checkpoint.join("receipt.json"))?;
    let receipt: Value = serde_json::from_slice(&receiptbytes)?;
    let stage = fit["stages"]
        .as_array()
        .ok_or_else(|| bad("fit stages absent"))?
        .iter()
        .find(|stage| stage["checkpoint"]["step"] == json!(a.checkpoint_step))
        .ok_or_else(|| bad("selected checkpoint not in authenticated fit report"))?;
    if receipt["step"] != json!(a.checkpoint_step) || stage["checkpoint"] != receipt {
        return Err(bad("fit/checkpoint receipt identity mismatch").into());
    }
    let trusted: NativeArtifactBinding = serde_json::from_value(receipt["parent"].clone())?;
    let model = NativeSourceRealizer::load_native(&checkpoint.join("native"), &trusted)?;
    let tokenizerbytes = fs::read(checkpoint.join("native/tokenizer.json"))?;
    let tok = ByteBpeTokenizer::from_tokenizer_json_bytes(&tokenizerbytes)
        .ok_or_else(|| bad("invalid bound tokenizer"))?;
    if model.binding().tokenizer_sha256() != sha256_bytes(&tokenizerbytes) {
        return Err(bad("native tokenizer binding differs").into());
    }
    let mut seen = BTreeSet::new();
    let mut endpoints = BTreeMap::<Vec<u8>, usize>::new();
    let mut triples = BTreeMap::<Vec<u8>, usize>::new();
    let width = model.context_config().heads * model.context_config().lanes_per_head;
    let mut occupancy = vec![BTreeSet::new(); width];
    let mut groups = BTreeMap::<String, [usize; 8]>::new();
    let mut rows = Vec::new();
    let mut span_count = 0;
    let mut tokens = 0;
    for p in &input.pairs {
        if p.id.is_empty()
            || p.kind.is_empty()
            || !seen.insert(p.id.clone())
            || p.a.as_bytes() == p.b.as_bytes()
        {
            return Err(bad(
                "pair IDs/kinds must be nonempty and unique; paired exact bytes must differ",
            )
            .into());
        }
        let aa = encode(&model, &tok, &p.a)?;
        let bb = encode(&model, &tok, &p.b)?;
        if aa.ids == bb.ids {
            return Err(bad("different paired bytes must retain different exact token IDs").into());
        }
        let endpoint_collision = aa.endpoint == bb.endpoint;
        let triple_collision = aa.triple == bb.triple;
        let same_length = aa.ids.len() == bb.ids.len();
        let mut aid = aa.ids.clone();
        let mut bid = bb.ids.clone();
        aid.sort_unstable();
        bid.sort_unstable();
        let same_token_multiset = aid == bid;
        // Vec equality includes length: never truncate unequal trajectories.
        let full_trajectory_collision = aa.trajectory == bb.trajectory;
        let counts = groups.entry(p.kind.clone()).or_default();
        counts[0] += 1;
        counts[1] += usize::from(same_length);
        counts[2] += usize::from(endpoint_collision);
        counts[3] += usize::from(triple_collision);
        counts[4] += usize::from(endpoint_collision && same_length);
        counts[5] += usize::from(triple_collision && same_length);
        counts[6] += usize::from(same_token_multiset);
        counts[7] += usize::from(full_trajectory_collision);
        for e in [&aa, &bb] {
            *endpoints.entry(e.endpoint.clone()).or_default() += 1;
            *triples.entry(e.triple.clone()).or_default() += 1;
            span_count += 1;
            tokens += e.ids.len();
            for state in &e.trajectory {
                for (lane, &code) in state.iter().enumerate() {
                    occupancy[lane].insert(code);
                }
            }
        }
        rows.push(json!({"id":p.id,"kind":p.kind,"a":aa.value,"b":bb.value,"same_token_length":same_length,"same_token_multiset":same_token_multiset,"full_signed_trajectory_exact_collision":full_trajectory_collision,"endpoint_exact_collision":endpoint_collision,"three_checkpoint_exact_collision":triple_collision,"length_confound":"different lengths change relative checkpoint positions and may explain separation; report separately"}));
    }
    if fs::read(&a.input)? != inputbytes {
        return Err(bad("audit input identity changed during read").into());
    }
    report_output::verify(&a.fit_root)?;
    if sha256_file(&a.fit_root.join("manifest.json"))? != manifest_sha {
        return Err(bad("checkpoint seal changed").into());
    }
    write(
        &a.out,
        "pairs.json",
        &json!({"schema":"uor-r4.context-target-collision-rows/1","signed_code_policy":"all120 historical H4 codes; antipodes/fiber are not quotiented; no observation-root substitution","pairs":rows}),
    )?;
    Ok(
        json!({"schema":"uor-r4.context-target-collision-audit/1","status":"COMPLETED","candidate_scope":"offline frozen-target information audit only; no JEPA adoption, student or new loss","language_verdict":"NOT_MEASURED; collision is not a language failure; separation is not grammar or prediction capability","model_training":"NOT_RUN","native_context_encoding":"ACTUALLY_EXECUTED","runtime_selected_record":"NONE; plain-text spans with fresh native identity state each","input_sha256":sha256_bytes(&inputbytes),"fit_report_sha256":sha256_bytes(&fitbytes),"checkpoint_step":a.checkpoint_step,"native_context_payload_sha256":sha256_file(&checkpoint.join("native/consumer/context-q4.bin"))?,"checkpoint_receipt_sha256":sha256_bytes(&receiptbytes),"checkpoint_manifest_sha256":manifest_sha,"tokenizer_sha256":sha256_bytes(&tokenizerbytes),"artifact_binding":trusted,"context_config":model.context_config(),"pairs":input.pairs.len(),"span_occurrences":span_count,"tokens_encoded":tokens,"checkpoint_policy":"ceil(m/3),ceil(2m/3),m; repeated short-span checkpoints explicitly retained","kind_counts_fields":["pairs","same_length","endpoint_collision","three_checkpoint_collision","same_length_endpoint_collision","same_length_three_checkpoint_collision","same_token_multiset","full_signed_trajectory_collision"],"kind_counts":groups,"occupancy":{"distinct_endpoint_tuples":endpoints.len(),"distinct_three_checkpoint_tuples":triples.len(),"endpoint_tuple_occurrence_multiplicities":endpoints.values().copied().collect::<Vec<_>>(),"three_checkpoint_tuple_occurrence_multiplicities":triples.values().copied().collect::<Vec<_>>(),"per_lane_signed_codes_over_all_tokens":occupancy.iter().map(|v|v.len()).collect::<Vec<_>>()},"source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT"),"executable_sha256":sha256_file(&std::env::current_exe()?)?,"elapsed_seconds":start.elapsed().as_secs_f64(),"wall_policy":"no self-estimate cancellation; finite pair/token bounds; owner resource policy applies"}),
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
            json!({"schema":"uor-r4.context-target-collision-audit/1","status":"FAILED","error":e.to_string(),"language_verdict":"NOT_APPLICABLE","elapsed_seconds":start.elapsed().as_secs_f64()})
        }
    };
    write(&a.out, "report.json", &report)?;
    report_output::seal(&a.out)?;
    report_output::verify(&a.out)?;
    result.map(|_| ())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn checkpoint_positions_short_and_maximal() -> Result<()> {
        assert_eq!(checkpoints(1)?, [1, 1, 1]);
        assert_eq!(checkpoints(2)?, [1, 2, 2]);
        assert_eq!(checkpoints(3)?, [1, 2, 3]);
        assert_eq!(checkpoints(128)?, [43, 86, 128]);
        assert!(checkpoints(0).is_err());
        assert!(checkpoints(129).is_err());
        Ok(())
    }
    #[test]
    fn checkpoint_key_preserves_duplicate_positions_and_signed_codes() -> Result<()> {
        let t = vec![vec![0, 119], vec![1, 118]];
        assert_eq!(snapshot_key(&t, checkpoints(2)?)?, [0, 119, 1, 118, 1, 118]);
        assert_ne!(snapshot_key(&t, [1, 1, 2])?, snapshot_key(&t, [1, 2, 2])?);
        assert!(snapshot_key(&t, [0, 1, 2]).is_err());
        assert!(snapshot_key(&t, [1, 2, 3]).is_err());
        Ok(())
    }
    #[test]
    fn full_trajectory_collision_does_not_truncate_unequal_lengths() {
        let short = vec![vec![1, 119]];
        let long = vec![vec![1, 119], vec![1, 119]];
        assert_ne!(short, long);
        assert_eq!(short.last(), long.last());
    }
}
