//! Artifact-bound nearest-example content readout for FBS serving.
//!
//! The readout is a sidecar: it never changes `UORLUT01` bytes. A sidecar binds
//! its rows to one artifact's SHA-256, and `IntegerStackModel::serve_step`
//! consults it only after the native writer and point-key fallback have
//! selected a token.

use std::collections::BTreeMap;
use std::io::Read;
use std::path::Path;

use serde::{Deserialize, Serialize};

use super::point_key::POINT_KEY_BOS;
use super::StackError;

/// Schema of a serving content-readout sidecar.
pub const STACK_CONTENT_READOUT_SCHEMA: &str = "uor-r4.stack-content-readout/1";
/// The only serving signature supported by this sidecar version.
pub const CONTENT_CODE_SIGNATURE_BITS: usize = 2_048;
/// Refuse a sidecar larger than this before JSON parsing.
pub const MAX_CONTENT_READOUT_BYTES: u64 = 64 << 20;

/// A 2,048-bit content signature, packed as 32 little-endian words.
pub type ContentSignature = Vec<u64>;

/// The supported content-signature mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContentSignatureMode {
    Code,
}

/// One signature and its reference-content target.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContentExample {
    pub signature: ContentSignature,
    pub target: u32,
}

/// The validated nearest-example table.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContentReadout {
    pub mode: ContentSignatureMode,
    pub signature_bits: usize,
    pub includes_reference_end: bool,
    pub rows: Vec<ContentExample>,
    pub max_distance: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ContentReadoutSidecar {
    schema: String,
    artifact_sha256: String,
    readout: ContentReadout,
}

/// The result of applying a serving content readout to one writer query.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ContentSelection {
    pub token: u32,
    pub nearest_distance: u32,
    pub readout_fired: bool,
}

impl ContentReadout {
    /// Parse and validate a sidecar against the loaded artifact and vocabulary.
    pub fn from_sidecar(
        bytes: &[u8],
        artifact_sha256: &str,
        vocab: usize,
    ) -> Result<Self, StackError> {
        if bytes.len() as u64 > MAX_CONTENT_READOUT_BYTES {
            return Err(StackError::ContentReadout(format!(
                "sidecar exceeds {MAX_CONTENT_READOUT_BYTES} bytes"
            )));
        }
        let sidecar: ContentReadoutSidecar = serde_json::from_slice(bytes).map_err(|error| {
            StackError::ContentReadout(format!("invalid sidecar JSON: {error}"))
        })?;
        if sidecar.schema != STACK_CONTENT_READOUT_SCHEMA {
            return Err(StackError::ContentReadout(format!(
                "schema {:?} is not {STACK_CONTENT_READOUT_SCHEMA:?}",
                sidecar.schema
            )));
        }
        if sidecar.artifact_sha256.len() != 64
            || !sidecar
                .artifact_sha256
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        {
            return Err(StackError::ContentReadout(
                "artifact_sha256 is not 64 lowercase hex characters".to_owned(),
            ));
        }
        if sidecar.artifact_sha256 != artifact_sha256 {
            return Err(StackError::ContentReadout(format!(
                "sidecar binds artifact {}, loaded artifact is {artifact_sha256}",
                sidecar.artifact_sha256
            )));
        }
        sidecar.readout.validate(vocab)?;
        Ok(sidecar.readout)
    }

    /// Read and validate a sidecar file.
    pub fn load(path: &Path, artifact_sha256: &str, vocab: usize) -> Result<Self, StackError> {
        Self::read_sidecar(path, artifact_sha256, vocab).map(|(_, readout)| readout)
    }

    /// Read a sidecar without allocating beyond the size limit, returning the
    /// exact bytes alongside the validated table so callers can identify the
    /// sidecar without rereading it.
    pub fn read_sidecar(
        path: &Path,
        artifact_sha256: &str,
        vocab: usize,
    ) -> Result<(Vec<u8>, Self), StackError> {
        let file = std::fs::File::open(path).map_err(StackError::Io)?;
        let mut bytes = Vec::new();
        file.take(MAX_CONTENT_READOUT_BYTES + 1)
            .read_to_end(&mut bytes)
            .map_err(StackError::Io)?;
        if bytes.len() as u64 > MAX_CONTENT_READOUT_BYTES {
            return Err(StackError::ContentReadout(format!(
                "sidecar exceeds {MAX_CONTENT_READOUT_BYTES} bytes"
            )));
        }
        let readout = Self::from_sidecar(&bytes, artifact_sha256, vocab)?;
        Ok((bytes, readout))
    }

    /// Validate rows, target ids and the learned Hamming radius.
    pub fn validate(&self, vocab: usize) -> Result<(), StackError> {
        if self.mode != ContentSignatureMode::Code
            || self.signature_bits != CONTENT_CODE_SIGNATURE_BITS
        {
            return Err(StackError::ContentReadout(
                "serving requires the 2,048-bit code signature".to_owned(),
            ));
        }
        if self.includes_reference_end {
            return Err(StackError::ContentReadout(
                "serving content readout must not include terminal BOS rows".to_owned(),
            ));
        }
        if self.rows.is_empty() {
            return Err(StackError::ContentReadout(
                "serving content readout has no rows".to_owned(),
            ));
        }
        if self.max_distance as usize > self.signature_bits {
            return Err(StackError::ContentReadout(
                "content-readout threshold exceeds the signature".to_owned(),
            ));
        }
        for row in &self.rows {
            if row.signature.len() != self.signature_bits / 64 {
                return Err(StackError::ContentReadout(
                    "content-readout row width differs".to_owned(),
                ));
            }
            if row.target as usize >= vocab {
                return Err(StackError::ContentReadout(format!(
                    "content-readout target {} is outside vocabulary {vocab}",
                    row.target
                )));
            }
            if row.target == POINT_KEY_BOS {
                return Err(StackError::ContentReadout(
                    "content-readout serving rows must not target BOS".to_owned(),
                ));
            }
        }
        Ok(())
    }

    /// Pack the low byte of each Q12 coordinate into a 2,048-bit signature.
    pub fn signature(query_q12: &[i32]) -> Result<ContentSignature, StackError> {
        if query_q12.len() * 8 != CONTENT_CODE_SIGNATURE_BITS {
            return Err(StackError::ContentReadout(format!(
                "content readout needs a 256-coordinate query, got {}",
                query_q12.len()
            )));
        }
        let mut signature = vec![0u64; CONTENT_CODE_SIGNATURE_BITS / 64];
        for (index, &value) in query_q12.iter().enumerate() {
            let bit = index * 8;
            signature[bit / 64] |= u64::from(value as u8) << (bit % 64);
        }
        Ok(signature)
    }

    /// Select the nearest target within the learned radius.
    ///
    /// Equal minimum distances vote by target. The commonest target wins, and
    /// an exact vote tie goes to the smaller token id.
    pub fn select(&self, query_q12: &[i32], fallback: u32) -> Result<ContentSelection, StackError> {
        let signature = Self::signature(query_q12)?;
        let mut nearest = u32::MAX;
        let mut votes = BTreeMap::<u32, u64>::new();
        for row in &self.rows {
            let distance: u32 = signature
                .iter()
                .zip(&row.signature)
                .map(|(&left, &right)| (left ^ right).count_ones())
                .sum();
            match distance.cmp(&nearest) {
                std::cmp::Ordering::Less => {
                    nearest = distance;
                    votes.clear();
                    votes.insert(row.target, 1);
                }
                std::cmp::Ordering::Equal => {
                    *votes.entry(row.target).or_default() += 1;
                }
                std::cmp::Ordering::Greater => {}
            }
        }
        let target = votes
            .into_iter()
            .max_by(|left, right| left.1.cmp(&right.1).then_with(|| right.0.cmp(&left.0)))
            .map(|(target, _)| target)
            .ok_or_else(|| {
                StackError::ContentReadout("content readout has no effective target".to_owned())
            })?;
        let fired = nearest <= self.max_distance;
        Ok(ContentSelection {
            token: if fired { target } else { fallback },
            nearest_distance: nearest,
            readout_fired: fired,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn signature(bit: usize) -> ContentSignature {
        let mut signature = vec![0; CONTENT_CODE_SIGNATURE_BITS / 64];
        signature[bit / 64] |= 1u64 << (bit % 64);
        signature
    }

    fn readout(rows: Vec<ContentExample>, max_distance: u32) -> ContentReadout {
        ContentReadout {
            mode: ContentSignatureMode::Code,
            signature_bits: CONTENT_CODE_SIGNATURE_BITS,
            includes_reference_end: false,
            rows,
            max_distance,
        }
    }

    #[test]
    fn minimum_rows_use_commonest_target_then_smaller_token() {
        let query = vec![0; CONTENT_CODE_SIGNATURE_BITS / 64];
        let table = readout(
            vec![
                ContentExample {
                    signature: query.clone(),
                    target: 9,
                },
                ContentExample {
                    signature: query.clone(),
                    target: 7,
                },
                ContentExample {
                    signature: query.clone(),
                    target: 7,
                },
                ContentExample {
                    signature: signature(1),
                    target: 3,
                },
            ],
            0,
        );
        assert_eq!(
            table.select(&vec![0; 256], 99).expect("select"),
            ContentSelection {
                token: 7,
                nearest_distance: 0,
                readout_fired: true,
            }
        );

        let tie = readout(
            vec![
                ContentExample {
                    signature: query.clone(),
                    target: 9,
                },
                ContentExample {
                    signature: query,
                    target: 7,
                },
            ],
            0,
        );
        assert_eq!(tie.select(&vec![0; 256], 99).expect("select").token, 7);
    }

    #[test]
    fn beyond_the_radius_returns_the_native_fallback() {
        let table = readout(
            vec![ContentExample {
                signature: signature(0),
                target: 7,
            }],
            0,
        );
        assert_eq!(
            table.select(&vec![0; 256], 42).expect("select"),
            ContentSelection {
                token: 42,
                nearest_distance: 1,
                readout_fired: false,
            }
        );
    }

    #[test]
    fn sidecar_is_artifact_bound_and_strict() {
        let sha = "a".repeat(64);
        let sidecar = serde_json::json!({
            "schema": STACK_CONTENT_READOUT_SCHEMA,
            "artifact_sha256": sha,
            "readout": readout(
                vec![ContentExample {
                    signature: signature(0),
                    target: 1,
                }],
                8,
            ),
        });
        let bytes = serde_json::to_vec(&sidecar).expect("json");
        assert!(ContentReadout::from_sidecar(&bytes, &"a".repeat(64), 2).is_ok());

        let mut wrong_artifact = sidecar.clone();
        wrong_artifact["artifact_sha256"] = serde_json::json!("b".repeat(64));
        assert!(ContentReadout::from_sidecar(
            &serde_json::to_vec(&wrong_artifact).expect("json"),
            &"a".repeat(64),
            2
        )
        .is_err());

        for field in ["schema", "artifact_sha256", "readout"] {
            let mut missing = sidecar.clone();
            missing.as_object_mut().expect("object").remove(field);
            assert!(ContentReadout::from_sidecar(
                &serde_json::to_vec(&missing).expect("json"),
                &"a".repeat(64),
                2
            )
            .is_err());
        }
        let mut missing_terminal_flag = sidecar.clone();
        missing_terminal_flag["readout"]
            .as_object_mut()
            .expect("readout object")
            .remove("includes_reference_end");
        assert!(ContentReadout::from_sidecar(
            &serde_json::to_vec(&missing_terminal_flag).expect("json"),
            &"a".repeat(64),
            2
        )
        .is_err());
        let mut unknown = sidecar;
        unknown["extra"] = serde_json::json!(true);
        assert!(ContentReadout::from_sidecar(
            &serde_json::to_vec(&unknown).expect("json"),
            &"a".repeat(64),
            2
        )
        .is_err());
    }

    #[test]
    fn validation_refuses_unsafe_serving_tables() {
        let valid = readout(
            vec![ContentExample {
                signature: signature(0),
                target: 1,
            }],
            8,
        );
        assert!(valid.validate(2).is_ok());

        let mut wrong_width = valid.clone();
        wrong_width.rows[0].signature.pop();
        assert!(wrong_width.validate(2).is_err());

        let mut wrong_target = valid.clone();
        wrong_target.rows[0].target = 2;
        assert!(wrong_target.validate(2).is_err());

        let mut bos = valid.clone();
        bos.rows[0].target = POINT_KEY_BOS;
        assert!(bos.validate(2).is_err());

        let mut terminal = valid.clone();
        terminal.includes_reference_end = true;
        assert!(terminal.validate(2).is_err());

        let mut sign = valid.clone();
        sign.signature_bits = 256;
        assert!(sign.validate(2).is_err());

        let mut wide = valid;
        wide.max_distance = CONTENT_CODE_SIGNATURE_BITS as u32 + 1;
        assert!(wide.validate(2).is_err());
    }
}
