//! Cuts a leading whole-record slice from a pinned UORT token stream and its
//! response mask, so a trainer can hold a part of a corpus larger than host
//! memory. The source files are only read. Identity work only: no scores.
#![forbid(unsafe_code)]
use crate::corpus::{sha256, UORT_HEADER_BYTES};
use crate::{Error, Result};
use serde_json::{json, Value};
use std::{
    fs,
    io::Read,
    path::Path,
};

/// The most tokens one slice may hold (the loader keeps 4 bytes per token and
/// 8 per eligible target on the host).
pub const MAX_SLICE_TOKENS: u64 = 1 << 30;

fn invalid(message: impl Into<String>) -> Error {
    Error::Invalid(format!("corpus slice: {}", message.into()))
}

fn read_prefix(path: &Path, bytes: u64) -> Result<Vec<u8>> {
    let file = fs::File::open(path).map_err(|e| invalid(format!("open {}: {e}", path.display())))?;
    let mut out = Vec::new();
    file.take(bytes)
        .read_to_end(&mut out)
        .map_err(|e| invalid(format!("read {}: {e}", path.display())))?;
    if out.len() as u64 != bytes {
        return Err(invalid(format!("{} is shorter than its header says", path.display())));
    }
    Ok(out)
}

/// The cut: the largest `n <= max_tokens` where token `n` starts a record (is
/// BOS) or `n` is the stream length. Returns the token count kept.
pub fn cut_point(ids: &[u16], total: u64, max_tokens: u64, bos: u16) -> Result<u64> {
    if ids.first() != Some(&bos) {
        return Err(invalid("stream must begin at BOS"));
    }
    if total <= max_tokens && ids.len() as u64 == total {
        return Ok(total);
    }
    let last = usize::try_from(max_tokens.min(ids.len() as u64 - 1))
        .map_err(|_| invalid("limit exceeds address space"))?;
    let cut = (1..=last).rev().find(|&p| ids[p] == bos);
    cut.map(|p| p as u64)
        .ok_or_else(|| invalid("no whole record fits under the limit"))
}

/// Writes `<out>/tokens.u16` and `<out>/response_mask.u8` and returns their
/// pins and counts in the shape of a driver `CorpusSource`.
pub fn run(tokens: &Path, mask: &Path, max_tokens: u64, bos: u16, split: &str, out: &Path) -> Result<Value> {
    if !(2..=MAX_SLICE_TOKENS).contains(&max_tokens) {
        return Err(invalid("limit must be 2..=2^30 tokens"));
    }
    let header = read_prefix(tokens, UORT_HEADER_BYTES as u64)?;
    if &header[..4] != b"UORT" || header[4..8] != 1u32.to_le_bytes() || header[20..].iter().any(|&b| b != 0) {
        return Err(invalid("source is not a canonical UORT version1 stream"));
    }
    let mut count = [0u8; 8];
    count.copy_from_slice(&header[8..16]);
    let total = u64::from_le_bytes(count);
    let physical = fs::metadata(tokens).map_err(|e| invalid(e.to_string()))?.len();
    let mask_len = fs::metadata(mask).map_err(|e| invalid(e.to_string()))?.len();
    if physical != total * 2 + UORT_HEADER_BYTES as u64 || mask_len != total || total < 2 {
        return Err(invalid("token and mask lengths differ from the header count"));
    }
    // One token past the limit is read so a record ending exactly at the limit is kept.
    let want = total.min(max_tokens + 1);
    let raw = read_prefix(tokens, UORT_HEADER_BYTES as u64 + want * 2)?;
    let ids = raw[UORT_HEADER_BYTES..]
        .chunks_exact(2)
        .map(|w| u16::from_le_bytes([w[0], w[1]]))
        .collect::<Vec<_>>();
    let kept = cut_point(&ids, total, max_tokens, bos)?;
    let kept_usize = usize::try_from(kept).map_err(|_| invalid("slice exceeds address space"))?;
    let mask_bytes = read_prefix(mask, kept)?;
    if mask_bytes.iter().any(|&m| m > 1) {
        return Err(invalid("mask must be binary"));
    }
    let mut token_bytes = header.clone();
    token_bytes[8..16].copy_from_slice(&kept.to_le_bytes());
    token_bytes.extend_from_slice(&raw[UORT_HEADER_BYTES..UORT_HEADER_BYTES + kept_usize * 2]);
    let eligible = ids[..kept_usize]
        .iter()
        .zip(&mask_bytes)
        .enumerate()
        .filter(|(p, (&id, &m))| *p > 0 && id != bos && m == 1)
        .count() as u64;
    let records = ids[..kept_usize].iter().filter(|&&id| id == bos).count() as u64;
    if eligible == 0 {
        return Err(invalid("slice has no eligible target"));
    }
    if out.exists() && fs::read_dir(out).map_err(|e| invalid(e.to_string()))?.next().is_some() {
        return Err(invalid("output folder must be empty"));
    }
    fs::create_dir_all(out).map_err(|e| invalid(e.to_string()))?;
    let token_path = out.join("tokens.u16");
    let mask_path = out.join("response_mask.u8");
    fs::write(&token_path, &token_bytes).map_err(|e| invalid(e.to_string()))?;
    fs::write(&mask_path, &mask_bytes).map_err(|e| invalid(e.to_string()))?;
    Ok(json!({
        "schema": "uor-r4.discrete-corpus-slice/1",
        "source_tokens": tokens,
        "source_total_tokens": total,
        "limit_tokens": max_tokens,
        "records": records,
        "whole_stream": kept == total,
        "source": {
            "split": split,
            "tokens": {"path": token_path, "sha256": sha256(&token_bytes)},
            "mask": {"path": mask_path, "sha256": sha256(&mask_bytes)},
            "expected_payload_tokens": kept,
            "expected_eligible_targets": eligible
        }
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stream(ids: &[u16]) -> Vec<u8> {
        let mut bytes = vec![0u8; UORT_HEADER_BYTES];
        bytes[..4].copy_from_slice(b"UORT");
        bytes[4..8].copy_from_slice(&1u32.to_le_bytes());
        bytes[8..16].copy_from_slice(&(ids.len() as u64).to_le_bytes());
        bytes[16..20].copy_from_slice(&16u32.to_le_bytes());
        for id in ids {
            bytes.extend_from_slice(&id.to_le_bytes());
        }
        bytes
    }

    fn scratch(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("uor-corpus-slice-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn cut_lands_on_a_record_start() {
        let ids = [0u16, 5, 6, 0, 7, 8, 9, 0, 3];
        assert_eq!(cut_point(&ids[..6], 9, 5, 0).unwrap(), 3);
        assert_eq!(cut_point(&ids[..8], 9, 7, 0).unwrap(), 7);
        assert_eq!(cut_point(&ids, 9, 20, 0).unwrap(), 9);
        assert!(cut_point(&ids[..3], 9, 2, 0).is_err());
        assert!(cut_point(&ids[1..], 8, 4, 0).is_err());
    }

    #[test]
    fn slice_loads_through_the_driver_corpus_loader() {
        let dir = scratch("load");
        let ids = [0u16, 5, 6, 0, 7, 8, 9, 0, 3];
        let mask = [0u8, 0, 1, 0, 1, 1, 0, 0, 1];
        fs::write(dir.join("t.u16"), stream(&ids)).unwrap();
        fs::write(dir.join("m.u8"), mask).unwrap();
        let out = dir.join("out");
        let report = run(&dir.join("t.u16"), &dir.join("m.u8"), 8, 0, "train", &out).unwrap();
        assert_eq!(report["source"]["expected_payload_tokens"], 7);
        assert_eq!(report["source"]["expected_eligible_targets"], 3);
        assert_eq!(report["records"], 2);
        let source: crate::corpus::CorpusSource = serde_json::from_value(report["source"].clone()).unwrap();
        let bos = crate::corpus::BosIdentity { token: "<|bos|>".into(), id: 0 };
        let loaded = crate::corpus::PreparedCorpus::load(&source, 16, &bos).unwrap();
        assert_eq!(loaded.eligible_len(), 3);
        assert!(run(&dir.join("t.u16"), &dir.join("m.u8"), 8, 0, "train", &out).is_err());
        fs::remove_dir_all(&dir).unwrap();
    }
}
