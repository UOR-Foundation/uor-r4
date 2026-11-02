use std::path::{Path, PathBuf};
use std::time::Instant;

use serde_json::json;
use uor_r4_cpu_train::{CpuWriterCore, Vector, VOCABULARY};
use uor_r4_discrete::corpus::{sha256, BosIdentity, CorpusSource, FilePin, PreparedCorpus, Split};
use uor_r4_integer::discrete_d11::IntegerDiscreteModel;

const ARTIFACT_SHA256: &str = "04652a3de8e966a23539b04fbd77c7d9872d9f13a541fcb224d48e766e2c147d";
const TOKENS_SHA256: &str = "0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b";
const MASK_SHA256: &str = "a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078";
const DRIVER_TOKENIZER_SHA256: &str =
    "d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89";
const MODEL_TOKENIZER_SHA256: &str =
    "a9fb13f70c69ffe5afed40c8251a8e702adf0b00732d35ccff9457a3309ee377";
const MANIFEST_SHA256: &str = "ab269c9818a713ef41f9aff0fdd8995386d5eb0ebaf8a98636f7766e99f83eab";
const PAYLOAD_TOKENS: u64 = 42_856;
const ELIGIBLE_TARGETS: u64 = 14_287;
const EXPECTED_ADMITTED: u64 = 9_012;
const EXPECTED_TOP1: u64 = 7_002;
const EXPECTED_TOP8: u64 = 7_485;

fn main() -> Result<(), String> {
    let root = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .ok_or_else(|| "usage: cpu-public-discrete-parity <fixture-dir>".to_owned())?;
    let artifact = read_hashed(
        &root.join("t16-frozen-zero-seed1-trunk.d11"),
        ARTIFACT_SHA256,
    )?;
    authenticate_manifest(&root)?;
    authenticate_tokenizers(&root)?;
    let table = discrete_table(&artifact)?;
    let writer = CpuWriterCore::from_codebook(&table, 8).map_err(|error| error.to_string())?;
    let model = IntegerDiscreteModel::from_bytes(&artifact)
        .map_err(|error| format!("public artifact: {error}"))?;
    let corpus = PreparedCorpus::load(
        &CorpusSource {
            split: Split::Dev,
            tokens: FilePin {
                path: root.join("dev-tokens.u16"),
                sha256: TOKENS_SHA256.to_owned(),
            },
            mask: FilePin {
                path: root.join("dev-response-mask.u8"),
                sha256: MASK_SHA256.to_owned(),
            },
            expected_payload_tokens: PAYLOAD_TOKENS,
            expected_eligible_targets: ELIGIBLE_TARGETS,
        },
        VOCABULARY as u32,
        &BosIdentity {
            token: "<|bos|>".to_owned(),
            id: 0,
        },
    )
    .map_err(|error| format!("public DEV panel: {error}"))?;

    let mut result = PanelResult::default();
    let started = Instant::now();
    let mut cursor = 0u64;
    while cursor < ELIGIBLE_TARGETS {
        let batch = corpus
            .batch(cursor, 192, 64)
            .map_err(|error| format!("DEV batch at {cursor}: {error}"))?;
        for (window, &target) in batch.windows.iter().zip(&batch.targets) {
            let prefix: Vec<u32> = window.iter().map(|identity| identity.token).collect();
            let evaluation = model
                .evaluate_prefix(&prefix)
                .map_err(|error| format!("model row {}: {error}", result.rows))?;
            let query: Vector = evaluation
                .read_vector
                .as_slice()
                .try_into()
                .map_err(|_| format!("model row {} query width differs", result.rows))?;
            let decision = writer.decide(&query);
            let cpu_cells: Vec<u32> = decision.admitted[..decision.admitted_count]
                .iter()
                .map(|&cell| u32::from(cell))
                .collect();
            let reading = writer
                .read_target(&query, target)
                .map_err(|error| error.to_string())?;
            result.rows += 1;
            result.cell_mismatches += u64::from(evaluation.admitted_cells != cpu_cells);
            result.top8_mismatches += u64::from(evaluation.top8 != decision.top8);
            result.token_mismatches += u64::from(evaluation.prediction != decision.top8[0]);
            result.admitted += u64::from(reading.target_admitted);
            result.top1 += u64::from(reading.target_top1);
            result.top8 += u64::from(reading.target_top8);
            for &value in &evaluation.read_vector {
                result.checksum = result
                    .checksum
                    .rotate_left(7)
                    .wrapping_add(value as i64 as u64);
            }
            for &token in &evaluation.top8 {
                result.checksum = result
                    .checksum
                    .rotate_left(7)
                    .wrapping_add(u64::from(token));
            }
        }
        cursor = batch.next_target;
    }
    let elapsed = started.elapsed().as_secs_f64();
    if result.rows != ELIGIBLE_TARGETS
        || result.admitted != EXPECTED_ADMITTED
        || result.top1 != EXPECTED_TOP1
        || result.top8 != EXPECTED_TOP8
        || result.cell_mismatches != 0
        || result.top8_mismatches != 0
        || result.token_mismatches != 0
    {
        return Err(format!("parity failure: {result:?}"));
    }
    println!(
        "{}",
        json!({
            "schema": "uor-r4.cpu-public-discrete-parity/1",
            "artifact_sha256": ARTIFACT_SHA256,
            "tokens_sha256": TOKENS_SHA256,
            "mask_sha256": MASK_SHA256,
            "driver_tokenizer_sha256": DRIVER_TOKENIZER_SHA256,
            "model_tokenizer_sha256": MODEL_TOKENIZER_SHA256,
            "manifest_sha256": MANIFEST_SHA256,
            "rows": result.rows,
            "threads": 1,
            "elapsed_ms": elapsed * 1_000.0,
            "rows_per_second": result.rows as f64 / elapsed,
            "cell_mismatches": result.cell_mismatches,
            "top8_mismatches": result.top8_mismatches,
            "token_mismatches": result.token_mismatches,
            "target_admitted": result.admitted,
            "top1": result.top1,
            "top8": result.top8,
            "checksum": format!("{:016x}", result.checksum),
            "peak_rss_kib": peak_rss_kib(),
        })
    );
    Ok(())
}

#[derive(Debug, Default)]
struct PanelResult {
    rows: u64,
    cell_mismatches: u64,
    top8_mismatches: u64,
    token_mismatches: u64,
    admitted: u64,
    top1: u64,
    top8: u64,
    checksum: u64,
}

fn read_hashed(path: &Path, expected: &str) -> Result<Vec<u8>, String> {
    let bytes = std::fs::read(path).map_err(|error| format!("{}: {error}", path.display()))?;
    let actual = sha256(&bytes);
    if actual != expected {
        return Err(format!(
            "{} SHA-256 differs: {actual} != {expected}",
            path.display()
        ));
    }
    Ok(bytes)
}

fn authenticate_manifest(root: &Path) -> Result<(), String> {
    let bytes = read_hashed(&root.join("dev-manifest.json"), MANIFEST_SHA256)?;
    let manifest: serde_json::Value =
        serde_json::from_slice(&bytes).map_err(|error| format!("DEV manifest: {error}"))?;
    if manifest["tokens"].as_u64() != Some(PAYLOAD_TOKENS)
        || manifest["response_tokens"].as_u64() != Some(ELIGIBLE_TARGETS)
        || manifest["tokens_sha256"].as_str() != Some(TOKENS_SHA256)
        || manifest["mask_sha256"].as_str() != Some(MASK_SHA256)
        || manifest["tokenizer"]["sha256"].as_str() != Some(DRIVER_TOKENIZER_SHA256)
    {
        return Err("DEV manifest inventory differs".to_owned());
    }
    Ok(())
}

fn authenticate_tokenizers(root: &Path) -> Result<(), String> {
    read_hashed(&root.join("driver-tokenizer.json"), DRIVER_TOKENIZER_SHA256)?;
    read_hashed(&root.join("tokenizer-seed-1.json"), MODEL_TOKENIZER_SHA256)?;
    Ok(())
}

fn discrete_table(artifact: &[u8]) -> Result<Vec<i32>, String> {
    if artifact.len() < 16 || &artifact[..8] != b"UORLUT01" {
        return Err("artifact magic differs".to_owned());
    }
    let header_len = u64::from_le_bytes(artifact[8..16].try_into().unwrap());
    let header_end = 16usize
        .checked_add(usize::try_from(header_len).map_err(|_| "header length")?)
        .filter(|&end| end <= artifact.len())
        .ok_or_else(|| "header outside artifact".to_owned())?;
    let data_start = header_end.div_ceil(64) * 64;
    let header: serde_json::Value = serde_json::from_slice(&artifact[16..header_end])
        .map_err(|error| format!("artifact header: {error}"))?;
    let table = header["tables"]
        .as_array()
        .and_then(|tables| {
            tables
                .iter()
                .find(|entry| entry["name"].as_str() == Some("discrete.table"))
        })
        .ok_or_else(|| "discrete.table is absent".to_owned())?;
    if table["kind"].as_str() != Some("i32")
        || table["len"].as_u64() != Some((VOCABULARY * 32) as u64)
    {
        return Err("discrete.table kind or length differs".to_owned());
    }
    let offset = usize::try_from(
        table["span"]["offset"]
            .as_u64()
            .ok_or_else(|| "discrete.table offset missing".to_owned())?,
    )
    .map_err(|_| "discrete.table offset".to_owned())?;
    let bytes = usize::try_from(
        table["span"]["bytes"]
            .as_u64()
            .ok_or_else(|| "discrete.table byte length missing".to_owned())?,
    )
    .map_err(|_| "discrete.table byte length".to_owned())?;
    if bytes != VOCABULARY * 32 * 4 {
        return Err("discrete.table byte length differs".to_owned());
    }
    let start = data_start + offset;
    let bytes = artifact
        .get(start..start + bytes)
        .ok_or_else(|| "discrete.table outside artifact".to_owned())?;
    Ok(bytes
        .chunks_exact(4)
        .map(|value| i32::from_le_bytes(value.try_into().unwrap()))
        .collect())
}

fn peak_rss_kib() -> u64 {
    std::fs::read_to_string("/proc/self/status")
        .ok()
        .and_then(|status| {
            status.lines().find_map(|line| {
                line.strip_prefix("VmHWM:")
                    .and_then(|value| value.split_whitespace().next())
                    .and_then(|value| value.parse().ok())
            })
        })
        .unwrap_or(0)
}
