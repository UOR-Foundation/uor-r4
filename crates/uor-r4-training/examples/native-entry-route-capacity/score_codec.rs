//! Exact durable cache for one authenticated Generate artifact. No floats,
//! score clipping, token filtering or implicit fallback enters this codec.
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs::{File, OpenOptions},
    io::{Read, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
};
pub const UNIT_Q24: i64 = 1 << 16;
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
fn bad(s: &str) -> Box<dyn std::error::Error> {
    std::io::Error::other(s).into()
}
fn hash(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}
pub fn encode(scores: &[i64]) -> Result<Vec<u8>> {
    let mut out = Vec::with_capacity(
        scores
            .len()
            .checked_mul(2)
            .ok_or_else(|| bad("score length overflow"))?,
    );
    for &score in scores {
        if score % UNIT_Q24 != 0 {
            return Err(bad("score is not aligned to exact i16 codec unit"));
        }
        let value = i16::try_from(score / UNIT_Q24)
            .map_err(|_| bad("score exceeds exact i16 codec range"))?;
        out.extend_from_slice(&value.to_le_bytes());
    }
    Ok(out)
}
pub fn decode(bytes: &[u8], vocab: usize) -> Result<Vec<i64>> {
    if vocab.checked_mul(2) != Some(bytes.len()) {
        return Err(bad("score record length differs"));
    }
    Ok(bytes
        .chunks_exact(2)
        .map(|b| i64::from(i16::from_le_bytes([b[0], b[1]])) * UNIT_Q24)
        .collect())
}
#[derive(Clone, Serialize)]
pub struct ScoreRecord {
    pub cache_id: usize,
    pub post_state_codes: Vec<u8>,
    pub file_offset_bytes: u64,
    pub byte_length: usize,
    pub encoded_sha256: String,
    pub full_q24_json_sha256: String,
}
pub struct ScoreCache {
    path: PathBuf,
    writer: File,
    artifact_sha256: String,
    vocab: usize,
    lanes: usize,
    maximum_bytes: u64,
    bytes: u64,
    lookup: BTreeMap<Vec<u8>, usize>,
    records: Vec<ScoreRecord>,
}
impl ScoreCache {
    pub fn new(
        path: &Path,
        artifact_sha256: &str,
        vocab: usize,
        lanes: usize,
        maximum_bytes: u64,
    ) -> Result<Self> {
        if vocab == 0
            || lanes == 0
            || lanes > 8
            || maximum_bytes == 0
            || artifact_sha256.len() != 64
            || !artifact_sha256.bytes().all(|b| b.is_ascii_hexdigit())
        {
            return Err(bad("score cache admission differs"));
        }
        let writer = OpenOptions::new().write(true).create_new(true).open(path)?;
        Ok(Self {
            path: path.into(),
            writer,
            artifact_sha256: artifact_sha256.into(),
            vocab,
            lanes,
            maximum_bytes,
            bytes: 0,
            lookup: BTreeMap::new(),
            records: Vec::new(),
        })
    }
    pub fn lookup(&self, state: &[u8]) -> Option<usize> {
        self.lookup.get(state).copied()
    }
    pub fn insert(&mut self, state: &[u8], scores: &[i64]) -> Result<usize> {
        if state.len() != self.lanes
            || state.iter().any(|&c| c >= 120)
            || scores.len() != self.vocab
        {
            return Err(bad("score cache state/vector differs"));
        }
        if let Some(id) = self.lookup(state) {
            if self.read(id)? != scores {
                return Err(bad("same state produced a different exact vector"));
            }
            return Ok(id);
        }
        let encoded = encode(scores)?;
        let next = self
            .bytes
            .checked_add(encoded.len() as u64)
            .ok_or_else(|| bad("cache bytes overflow"))?;
        if next > self.maximum_bytes {
            return Err(bad("score cache durable-byte admission exceeded"));
        }
        let id = self.records.len();
        self.writer.write_all(&encoded)?;
        self.writer.flush()?;
        let record = ScoreRecord {
            cache_id: id,
            post_state_codes: state.to_vec(),
            file_offset_bytes: self.bytes,
            byte_length: encoded.len(),
            encoded_sha256: hash(&encoded),
            full_q24_json_sha256: hash(&serde_json::to_vec(scores)?),
        };
        self.records.push(record);
        self.lookup.insert(state.to_vec(), id);
        self.bytes = next;
        Ok(id)
    }
    pub fn read(&self, id: usize) -> Result<Vec<i64>> {
        let r = self
            .records
            .get(id)
            .ok_or_else(|| bad("unknown score cache record"))?;
        // Independently opened reader: cloned descriptors share seek offsets and
        // could otherwise corrupt the next append position during phase one.
        let mut reader = File::open(&self.path)?;
        reader.seek(SeekFrom::Start(r.file_offset_bytes))?;
        let mut bytes = vec![0; r.byte_length];
        reader.read_exact(&mut bytes)?;
        if hash(&bytes) != r.encoded_sha256 {
            return Err(bad("cached exact score record digest differs"));
        }
        let scores = decode(&bytes, self.vocab)?;
        if hash(&serde_json::to_vec(&scores)?) != r.full_q24_json_sha256 {
            return Err(bad("decoded native score vector digest differs"));
        }
        Ok(scores)
    }
    pub fn finish(&mut self) -> Result<serde_json::Value> {
        self.writer.sync_all()?;
        if std::fs::metadata(&self.path)?.len() != self.bytes {
            return Err(bad("score file length differs"));
        }
        let mut reader = File::open(&self.path)?;
        let mut hasher = Sha256::new();
        let mut buffer = [0u8; 65536];
        loop {
            let n = reader.read(&mut buffer)?;
            if n == 0 {
                break;
            }
            hasher.update(&buffer[..n]);
        }
        Ok(
            serde_json::json!({"codec":"signed-i16-le-times-65536-Q24/1","file":"generate-scores.i16le",
            "artifact_sha256":self.artifact_sha256,"vocab_size":self.vocab,"lanes":self.lanes,"bytes":self.bytes,
            "file_sha256":hex::encode(hasher.finalize()),"unique_states":self.records.len(),"records":self.records}),
        )
    }
    pub fn record(&self, id: usize) -> Result<&ScoreRecord> {
        self.records
            .get(id)
            .ok_or_else(|| bad("unknown score record"))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn codec_roundtrip_signed_bounds_and_rejects_lossy_inputs() -> Result<()> {
        let input = [
            i64::from(i16::MIN) * UNIT_Q24,
            -UNIT_Q24,
            0,
            UNIT_Q24,
            i64::from(i16::MAX) * UNIT_Q24,
        ];
        assert_eq!(decode(&encode(&input)?, input.len())?, input);
        assert!(encode(&[1]).is_err());
        assert!(encode(&[(i64::from(i16::MAX) + 1) * UNIT_Q24]).is_err());
        assert!(decode(&[0], 1).is_err());
        assert!(decode(&[0, 0], 2).is_err());
        Ok(())
    }
    #[test]
    fn cache_deduplicates_exact_states_preserves_seek_and_rejects_mismatch() -> Result<()> {
        let path = std::env::temp_dir().join(format!(
            "route-score-cache-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_nanos()
        ));
        let result = (|| -> Result<()> {
            let mut cache = ScoreCache::new(&path, &"a".repeat(64), 2, 2, 8)?;
            assert!(cache.insert(&[1], &[0, UNIT_Q24]).is_err());
            let a = cache.insert(&[1, 3], &[0, UNIT_Q24])?;
            assert_eq!(cache.lookup(&[1, 3]), Some(a));
            assert_eq!(cache.lookup(&[3, 1]), None);
            assert_eq!(cache.read(a)?, vec![0, UNIT_Q24]);
            let b = cache.insert(&[3, 1], &[-UNIT_Q24, 0])?;
            assert_eq!(cache.read(a)?, vec![0, UNIT_Q24]);
            assert_eq!(cache.read(b)?, vec![-UNIT_Q24, 0]);
            assert!(cache.insert(&[1, 3], &[UNIT_Q24, 0]).is_err());
            assert!(cache.insert(&[2, 2], &[0, 0]).is_err());
            assert_eq!(cache.finish()?["unique_states"], 2);
            Ok(())
        })();
        if path.exists() {
            std::fs::remove_file(&path)?;
        }
        result
    }
}
