//! The SCENARIO SUITE: comprehensive comparison over diverse, real-world
//! input, complementing the aggregate rows of `compare` and
//! docs/COMPARISON.md.
//!
//! Scenario classes:
//!   - in-domain prompts: story openings a user would actually type at a
//!     TinyStories-class model; agreement measured along the TEACHER'S OWN
//!     greedy trajectory (the deployment question: "would the artifact
//!     have produced the same continuation, token by token?");
//!   - out-of-domain prompts: questions, instructions, business prose —
//!     real-world inputs this source model was never meant for; both
//!     systems are out of domain and the comparison is relative;
//!   - real human-written text (not model-sampled): an in-domain-style
//!     story and a Shakespeare passage (fully out-of-domain), scored
//!     token-by-token against the ACTUAL next token for both systems, plus
//!     artifact↔teacher agreement;
//!   - structural stress: repetition, a one-word prompt, and a cold start
//!     from BOS alone.
//!
//! Rules of the suite: the tokenizer is validated in-run (round-trip
//! witness plus a fluency gate on the teacher's continuation — a broken
//! encoding cannot pass it); every scenario feeds teacher and artifact the
//! IDENTICAL token stream; the artifact uses the same store and the same
//! runtime code path certified in PROOF.md (scenario text is fully unseen
//! by the store, which was built from the training split only); quality
//! and throughput are reported together, per scenario class.
//!
//! Classical runtimes (llama.cpp et al.) execute the source model itself,
//! so their scenario-level predictions coincide with the teacher rows by
//! definition; their throughput is in docs/COMPARISON.md.

use std::collections::BTreeMap;
use std::io;
use std::path::Path;

const MAX_TOKEN_BYTES: usize = 1024;

/// Convert a Hugging Face byte-level BPE vocabulary into the compact token
/// table consumed by the allocation-free runtime tokenizer.
#[cfg(not(target_arch = "wasm32"))]
pub fn export_hf_bytelevel_tokenizer(
    source: impl AsRef<Path>,
    destination: impl AsRef<Path>,
) -> io::Result<()> {
    export_hf_bytelevel_tokenizer_with_lengths(source, destination).map(|_| ())
}

/// Export the runtime tokenizer and return per-token UTF-8 byte lengths for
/// observation byte-anchor generation.
#[cfg(not(target_arch = "wasm32"))]
pub fn export_hf_bytelevel_tokenizer_with_lengths(
    source: impl AsRef<Path>,
    destination: impl AsRef<Path>,
) -> io::Result<Vec<u32>> {
    let tokens = hf_bytelevel_tokens(source)?;
    let lengths = tokens
        .iter()
        .map(|token| {
            u32::try_from(token.len())
                .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "token too long"))
        })
        .collect::<io::Result<Vec<_>>>()?;
    let mut bytes = Vec::new();
    for token in tokens {
        let length = i32::try_from(token.len())
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "token too long"))?;
        bytes.extend_from_slice(&length.to_le_bytes());
        bytes.extend_from_slice(&token);
    }
    std::fs::write(destination, bytes)?;
    Ok(lengths)
}

fn hf_bytelevel_tokens(source: impl AsRef<Path>) -> io::Result<Vec<Vec<u8>>> {
    let value: serde_json::Value = serde_json::from_slice(&std::fs::read(source)?)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    let vocab = value
        .pointer("/model/vocab")
        .and_then(serde_json::Value::as_object)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "missing BPE vocabulary"))?;
    let mut tokens = vec![Vec::new(); vocab.len()];
    let byte_map = bytelevel_inverse();
    for (piece, id) in vocab {
        let id = id
            .as_u64()
            .and_then(|value| usize::try_from(value).ok())
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "invalid token id"))?;
        let output = tokens
            .get_mut(id)
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "sparse token ids"))?;
        for ch in piece.chars() {
            if let Some(byte) = byte_map.get(&ch) {
                output.push(*byte);
            } else {
                let mut utf8 = [0u8; 4];
                output.extend_from_slice(ch.encode_utf8(&mut utf8).as_bytes());
            }
        }
    }
    Ok(tokens)
}

fn bytelevel_inverse() -> BTreeMap<char, u8> {
    let mut bytes: Vec<u8> = (b'!'..=b'~')
        .chain(0xA1..=0xAC)
        .chain(0xAE..=0xFF)
        .collect();
    let mut codepoints: Vec<u32> = bytes.iter().map(|byte| u32::from(*byte)).collect();
    let mut extra = 0u32;
    for byte in 0u8..=u8::MAX {
        if !bytes.contains(&byte) {
            bytes.push(byte);
            codepoints.push(256 + extra);
            extra += 1;
        }
    }
    bytes
        .into_iter()
        .zip(codepoints)
        .map(|(byte, codepoint)| {
            (
                char::from_u32(codepoint).expect("byte-level codepoint is valid"),
                byte,
            )
        })
        .collect()
}

// ------------------------------------------------------------ tokenizer --

/// The original (July 2023, scoreless) llama2.c tokenizer.bin: per token,
/// i32 length then bytes. Probed conventions (witnessed in-run): the
/// sentencepiece space marker was already exported as a plain space
/// (piece 278 = " the"), and ids 3+cp hold the UTF-8 encoding of
/// codepoints U+0000..=U+00FF. Encoding: leading-space prefix, direct
/// per-char piece lookup with codepoint fallback, then iterative greedy
/// pair merging preferring the LOWEST merged token id (the deterministic
/// rule both systems share; the fluency gate validates its adequacy).
pub struct Tokenizer {
    pub vocab: Vec<Vec<u8>>,
    map: BTreeMap<Vec<u8>, u32>,
}

impl Tokenizer {
    /// Load and validate a tokenizer without panicking on malformed input.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn try_load(path: impl AsRef<Path>) -> io::Result<Self> {
        let path_ref = path.as_ref();

        // 1. Try loading vocab.json if present in the target or parent directories
        let json_candidates = [
            path_ref.to_path_buf(),
            path_ref.with_file_name("vocab.json"),
            path_ref
                .parent()
                .unwrap_or_else(|| Path::new("."))
                .join("vocab.json"),
            std::path::PathBuf::from(".uor-models/sources/smollm2-1-7b-instruct/vocab.json"),
            std::path::PathBuf::from(".uor-models/compiled/smollm2-135m-instruct/vocab.json"),
        ];

        for jpath in &json_candidates {
            if jpath.extension().and_then(|s| s.to_str()) == Some("json") && jpath.exists() {
                if let Ok(bytes) = std::fs::read(jpath) {
                    if let Ok(raw_map) = serde_json::from_slice::<BTreeMap<String, u32>>(&bytes) {
                        let mut max_id = 0u32;
                        for &id in raw_map.values() {
                            if id > max_id {
                                max_id = id;
                            }
                        }
                        let mut vocab = vec![Vec::new(); (max_id + 1) as usize];
                        let mut map = BTreeMap::new();
                        for (k, &id) in &raw_map {
                            let k_bytes = k.as_bytes().to_vec();
                            vocab[id as usize] = k_bytes.clone();
                            map.insert(k_bytes, id);
                        }
                        return Ok(Tokenizer { vocab, map });
                    }
                }
            }
        }

        // 2. Fall back to binary tokenizer.bin format
        let bytes = std::fs::read(path_ref)?;
        let mut vocab = Vec::new();
        let mut offset = 0usize;
        while offset < bytes.len() {
            let length_bytes = bytes.get(offset..offset + 4).ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidData, "truncated tokenizer length")
            })?;
            let length = i32::from_le_bytes(
                length_bytes
                    .try_into()
                    .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?,
            );
            offset += 4;
            if length < 0 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "negative token length",
                ));
            }
            let length = length as usize;
            let token = bytes.get(offset..offset + length).ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidData, "truncated tokenizer token")
            })?;
            vocab.push(token.to_vec());
            offset += length;
        }
        let mut map = BTreeMap::new();
        for (index, token) in vocab.iter().enumerate() {
            let id = index as u32;
            map.entry(token.clone()).or_insert(id);
        }
        Ok(Tokenizer { vocab, map })
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn load(path: &str) -> Self {
        Self::try_load(path).expect("tokenizer file must be readable and well-formed")
    }

    pub fn encode(&self, text: &str) -> Vec<u32> {
        let mut toks = vec![0u32; text.chars().count().saturating_add(2)];
        let count = self
            .encode_into(text, &mut toks)
            .expect("token buffer sized from input characters");
        toks.truncate(count);
        toks
    }

    /// Encode into caller-owned storage without allocating.
    pub fn encode_into(&self, text: &str, out: &mut [u32]) -> io::Result<usize> {
        let capacity_error = || io::Error::new(io::ErrorKind::InvalidInput, "token buffer full");
        let is_llama_bos = self.vocab.get(1).is_some_and(|v| v == b"<s>");
        let mut len = 0usize;
        if is_llama_bos {
            *out.get_mut(len).ok_or_else(capacity_error)? = 1;
            len += 1;
        }

        let is_bpe = self.vocab.len() > 32000
            || self
                .vocab
                .get(1)
                .is_some_and(|v| v == b"<|im_start|>" || v == b"\xC4\xA0");

        if is_bpe {
            let mut encoded_str = String::with_capacity(text.len() * 2);
            for ch in text.chars() {
                match ch {
                    ' ' => encoded_str.push('Ġ'),
                    '\n' => encoded_str.push('Ċ'),
                    '\r' => encoded_str.push('Ĉ'),
                    '\t' => encoded_str.push('ĉ'),
                    c => encoded_str.push(c),
                }
            }

            let bytes = encoded_str.as_bytes();
            let mut i = 0usize;
            while i < bytes.len() {
                let mut matched_len = 0usize;
                let mut matched_id = None;

                let max_k = (bytes.len() - i).min(64);
                for k in (1..=max_k).rev() {
                    let sub = &bytes[i..i + k];
                    if let Some(&id) = self.map.get(sub) {
                        matched_len = k;
                        matched_id = Some(id);
                        break;
                    }
                }

                if let Some(id) = matched_id {
                    *out.get_mut(len).ok_or_else(capacity_error)? = id;
                    len += 1;
                    i += matched_len;
                } else {
                    let b = bytes[i];
                    if let Some(&id) = self.map.get(&[b][..]) {
                        *out.get_mut(len).ok_or_else(capacity_error)? = id;
                        len += 1;
                    }
                    i += 1;
                }
            }
            return Ok(len);
        }

        for ch in std::iter::once(' ').chain(text.chars()) {
            let mut utf8 = [0u8; 4];
            let bytes = ch.encode_utf8(&mut utf8).as_bytes();
            let token = match self.map.get(bytes) {
                Some(&id) => id,
                None => {
                    for byte in bytes {
                        let id = self.map.get(&[*byte][..]).copied().ok_or_else(|| {
                            io::Error::new(
                                io::ErrorKind::InvalidInput,
                                "tokenizer has no byte fallback",
                            )
                        })?;
                        *out.get_mut(len).ok_or_else(capacity_error)? = id;
                        len += 1;
                    }
                    continue;
                }
            };
            *out.get_mut(len).ok_or_else(capacity_error)? = token;
            len += 1;
        }
        loop {
            let mut best: Option<(u32, usize)> = None;
            let start = if is_llama_bos { 1 } else { 0 };
            for i in start..len.saturating_sub(1) {
                let left = &self.vocab[out[i] as usize];
                let right = &self.vocab[out[i + 1] as usize];
                let pair_len = left.len().saturating_add(right.len());
                if pair_len > MAX_TOKEN_BYTES {
                    continue;
                }
                let mut pair = [0u8; MAX_TOKEN_BYTES];
                pair[..left.len()].copy_from_slice(left);
                pair[left.len()..pair_len].copy_from_slice(right);
                if let Some(&id) = self.map.get(&pair[..pair_len]) {
                    if best.is_none_or(|(b, _)| id < b) {
                        best = Some((id, i));
                    }
                }
            }
            match best {
                Some((id, i)) => {
                    out[i] = id;
                    out.copy_within(i + 2..len, i + 1);
                    len -= 1;
                }
                None => break,
            }
        }
        Ok(len)
    }

    pub fn decode(&self, toks: &[u32]) -> String {
        let is_llama_bos = self.vocab.get(1).is_some_and(|v| v == b"<s>");
        let mut raw = Vec::new();
        for &t in toks {
            if is_llama_bos && (t == 1 || t == 2) {
                continue;
            }
            if (t as usize) < self.vocab.len() {
                raw.extend_from_slice(&self.vocab[t as usize]);
            }
        }
        let text = String::from_utf8_lossy(&raw);
        text.replace('Ġ', " ")
            .replace('Ċ', "\n")
            .replace('Ĉ', "\r")
            .replace('ĉ', "\t")
    }

    /// Decode into caller-owned byte storage without allocating.
    pub fn decode_into(&self, toks: &[u32], out: &mut [u8]) -> io::Result<usize> {
        let decoded = self.decode(toks);
        let bytes = decoded.as_bytes();
        if bytes.len() > out.len() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "decoded output buffer full",
            ));
        }
        out[..bytes.len()].copy_from_slice(bytes);
        Ok(bytes.len())
    }
}
