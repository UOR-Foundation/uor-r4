//! Offline CPU solver for Claude-330's per-token ask offsets.
//!
//! The found-copy pull depends on token ids, the shared writer table and key
//! rotation alone. This module optimizes checkpoint-compatible ask rows against
//! supplied `(question, target key)` pulls without running the trunk.

use std::collections::{BTreeMap, BTreeSet};
use std::ops::Range;

use rayon::prelude::*;
use safetensors::tensor::TensorView;
use safetensors::{serialize, Dtype, SafeTensors};
use serde::{Deserialize, Serialize};
use uor_r4_integer::bitcode::popcount_swar;
use uor_r4_integer::fbs_snapshot::{hamilton_gate_f32, pack_signs_f32};

pub const PULL_CORPUS_SCHEMA: &str = "uor-r4.ask-pull-corpus/1";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AskPull {
    pub question: u32,
    pub key: u32,
    #[serde(default = "unit_weight")]
    pub weight: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AskPullCorpus {
    pub schema: String,
    pub pulls: Vec<AskPull>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AskOffsetSolveReading {
    pub question: u32,
    pub depth: usize,
    pub pulls: usize,
    pub pull_weight: u64,
    pub target_keys: usize,
    pub zero_weighted_bits: u64,
    pub solved_weighted_bits: u64,
    pub improvement_bits: u64,
    pub evaluations: u64,
    pub offset_changed_coordinates: usize,
    pub offset_abs_sum_q16: u64,
}

pub struct SolvedAskOffsets {
    pub hamming_bytes: Vec<u8>,
    pub readings: Vec<AskOffsetSolveReading>,
}

#[derive(Clone)]
struct NamedTensor {
    name: String,
    dtype: Dtype,
    shape: Vec<usize>,
    data: Vec<u8>,
}

#[derive(Clone)]
struct SolveRead {
    depth: usize,
    key_rotation: Vec<f32>,
}

pub struct AskOffsetSolveBasis {
    table: Vec<f32>,
    vocab: usize,
    width: usize,
    heads: usize,
    reads: Vec<SolveRead>,
    original_tensors: Vec<NamedTensor>,
    metadata: Option<std::collections::HashMap<String, String>>,
}

impl AskPullCorpus {
    pub fn from_json(bytes: &[u8]) -> Result<Self, String> {
        let corpus: Self =
            serde_json::from_slice(bytes).map_err(|error| format!("pull corpus: {error}"))?;
        if corpus.schema != PULL_CORPUS_SCHEMA {
            return Err(format!(
                "pull corpus schema is {:?}, not {PULL_CORPUS_SCHEMA:?}",
                corpus.schema
            ));
        }
        Ok(corpus)
    }

    pub fn b2_relations(relations: Range<u32>, query_shift: u32) -> Self {
        Self {
            schema: PULL_CORPUS_SCHEMA.to_owned(),
            pulls: relations
                .map(|key| AskPull {
                    question: key + query_shift,
                    key,
                    weight: 1,
                })
                .collect(),
        }
    }

    fn validate(&self, vocab: usize) -> Result<(), String> {
        if self.pulls.is_empty() {
            return Err("pull corpus is empty".to_owned());
        }
        for (index, pull) in self.pulls.iter().enumerate() {
            if pull.weight == 0 {
                return Err(format!("pull {index} has zero weight"));
            }
            for (role, token) in [("question", pull.question), ("key", pull.key)] {
                if token as usize >= vocab {
                    return Err(format!(
                        "pull {index} {role} token {token} is outside vocabulary {vocab}"
                    ));
                }
            }
        }
        Ok(())
    }
}

impl AskOffsetSolveBasis {
    pub fn from_bytes(model: &[u8], hamming: &[u8]) -> Result<Self, String> {
        let model_tensors = SafeTensors::deserialize(model)
            .map_err(|error| format!("model.safetensors: {error}"))?;
        let table_tensor = model_tensors
            .tensor("embedding.weight")
            .map_err(|error| format!("embedding.weight: {error}"))?;
        let (vocab, width) = match table_tensor.shape() {
            [vocab, width] => (*vocab, *width),
            shape => {
                return Err(format!(
                    "embedding.weight is {shape:?}, not a two-dimensional table"
                ))
            }
        };
        let table = f32_values(&table_tensor, "embedding.weight", &[vocab, width])?;

        let hamming_tensors =
            SafeTensors::deserialize(hamming).map_err(|error| format!("hamming_read: {error}"))?;
        let metadata = safetensors_metadata(hamming)?;
        let head_tensor = hamming_tensors
            .tensor("heads")
            .map_err(|error| format!("heads: {error}"))?;
        let heads = *u32_values(&head_tensor, "heads", &[1])?
            .first()
            .ok_or_else(|| "heads is empty".to_owned())? as usize;
        if heads < 2 || !heads.is_multiple_of(2) || !width.is_multiple_of(heads) {
            return Err(format!("{heads} heads over width {width} is invalid"));
        }
        let head_width = width / heads;
        if head_width == 0 || !head_width.is_multiple_of(4) || head_width > 64 {
            return Err(format!(
                "head width {head_width} is not a quaternion multiple within one word"
            ));
        }
        if hamming_tensors
            .names()
            .iter()
            .any(|name| name.ends_with(".step_rotation"))
        {
            return Err("ask heads and the type spin are mutually exclusive".to_owned());
        }

        let mut depths = BTreeSet::new();
        for name in hamming_tensors.names() {
            if let Some((depth, _)) = name
                .strip_prefix('d')
                .and_then(|name| name.split_once('.'))
                .and_then(|(depth, part)| depth.parse::<usize>().ok().map(|depth| (depth, part)))
            {
                depths.insert(depth);
            }
        }
        if depths.is_empty() {
            return Err("checkpoint carries no Hamming depths".to_owned());
        }

        let mut reads = Vec::with_capacity(depths.len());
        for depth in depths {
            let name = format!("d{depth}.key_rotation");
            let rotation = hamming_tensors
                .tensor(&name)
                .map_err(|error| format!("{name}: {error}"))?;
            reads.push(SolveRead {
                depth,
                key_rotation: f32_values(&rotation, &name, &[width])?,
            });
        }

        let mut names = hamming_tensors.names().to_vec();
        names.sort_unstable();
        let original_tensors = names
            .into_iter()
            .filter(|name| !name.ends_with(".ask"))
            .map(|name| {
                let tensor = hamming_tensors
                    .tensor(name)
                    .map_err(|error| format!("{name}: {error}"))?;
                Ok(NamedTensor {
                    name: name.to_owned(),
                    dtype: tensor.dtype(),
                    shape: tensor.shape().to_vec(),
                    data: tensor.data().to_vec(),
                })
            })
            .collect::<Result<Vec<_>, String>>()?;

        Ok(Self {
            table,
            vocab,
            width,
            heads,
            reads,
            original_tensors,
            metadata,
        })
    }

    pub fn solve(
        &self,
        corpus: &AskPullCorpus,
        max_evaluations_per_cell: u64,
    ) -> Result<SolvedAskOffsets, String> {
        corpus.validate(self.vocab)?;
        if max_evaluations_per_cell == 0 {
            return Err("max evaluations must be positive".to_owned());
        }
        let mut groups: BTreeMap<u32, Vec<&AskPull>> = BTreeMap::new();
        for pull in &corpus.pulls {
            groups.entry(pull.question).or_default().push(pull);
        }
        let tasks = self
            .reads
            .iter()
            .enumerate()
            .flat_map(|(read_index, read)| {
                groups
                    .iter()
                    .map(move |(&question, pulls)| (read_index, read, question, pulls))
            })
            .collect::<Vec<_>>();
        let cells = tasks
            .into_par_iter()
            .map(|(read_index, read, question, pulls)| {
                let (offset, evaluations, zero_bits, solved_bits) =
                    self.solve_question(read, question, pulls, max_evaluations_per_cell)?;
                let offset_changed_coordinates =
                    offset.iter().filter(|&&value| value != 0.0).count();
                let offset_abs_sum_q16 = offset_abs_sum_q16(&offset)?;
                Ok((
                    read_index,
                    question,
                    offset,
                    AskOffsetSolveReading {
                        question,
                        depth: read.depth,
                        pulls: pulls.len(),
                        pull_weight: pulls
                            .iter()
                            .try_fold(0u64, |total, pull| {
                                total.checked_add(u64::from(pull.weight))
                            })
                            .ok_or_else(|| "pull weight total overflows".to_owned())?,
                        target_keys: pulls
                            .iter()
                            .map(|pull| pull.key)
                            .collect::<BTreeSet<_>>()
                            .len(),
                        zero_weighted_bits: zero_bits,
                        solved_weighted_bits: solved_bits,
                        improvement_bits: zero_bits.saturating_sub(solved_bits),
                        evaluations,
                        offset_changed_coordinates,
                        offset_abs_sum_q16,
                    },
                ))
            })
            .collect::<Result<Vec<_>, String>>()?;
        let mut tables = self
            .reads
            .iter()
            .map(|read| (read.depth, vec![0.0f32; self.vocab * self.width]))
            .collect::<BTreeMap<_, _>>();
        let mut readings = Vec::with_capacity(cells.len());
        for (read_index, question, offset, reading) in cells {
            let depth = self.reads[read_index].depth;
            tables
                .get_mut(&depth)
                .ok_or_else(|| format!("depth {depth} table disappeared"))?
                [question as usize * self.width..][..self.width]
                .copy_from_slice(&offset);
            readings.push(reading);
        }
        readings.sort_by_key(|reading| (reading.question, reading.depth));
        Ok(SolvedAskOffsets {
            hamming_bytes: self.serialize_hamming(&tables)?,
            readings,
        })
    }

    fn solve_question(
        &self,
        read: &SolveRead,
        question: u32,
        pulls: &[&AskPull],
        max_evaluations: u64,
    ) -> Result<(Vec<f32>, u64, u64, u64), String> {
        let zero = vec![0.0f32; self.width];
        let zero_bits = self.objective(read, question, &zero, pulls)?;
        let question_point = self.table_row(question);
        let targets = pulls.iter().map(|pull| pull.key).collect::<BTreeSet<_>>();
        let mut candidates = vec![zero.clone()];
        for key in &targets {
            let key_point = self.table_row(*key);
            let offset = question_point
                .iter()
                .zip(key_point)
                .map(|(&question, &key)| key - question)
                .collect::<Vec<_>>();
            ensure_finite(&offset, "exact candidate")?;
            candidates.push(offset);
        }
        candidates.push(self.weighted_median_offset(question, pulls)?);
        let mut candidates = candidates
            .into_iter()
            .map(|offset| {
                let bits = self.objective(read, question, &offset, pulls)?;
                Ok((offset, bits))
            })
            .collect::<Result<Vec<_>, String>>()?;
        candidates
            .sort_by(|(a, a_bits), (b, b_bits)| a_bits.cmp(b_bits).then_with(|| offset_cmp(a, b)));
        candidates.dedup_by(|(a, _), (b, _)| a == b);

        let mut evaluations = 0u64;
        let mut best = zero.clone();
        let mut best_bits = zero_bits;
        for (start, start_bits) in candidates.into_iter().take(4) {
            if evaluations >= max_evaluations {
                break;
            }
            let mut offset = start;
            let mut bits = start_bits;
            evaluations += 1;
            if bits == 0 {
                best = offset;
                best_bits = 0;
                break;
            }
            for step in [16384.0f32, 4096.0, 1024.0, 256.0, 64.0, 16.0, 4.0, 1.0] {
                loop {
                    let mut improved = false;
                    for coordinate in 0..self.width {
                        for delta in [step, -step] {
                            if evaluations >= max_evaluations {
                                break;
                            }
                            let previous = offset[coordinate];
                            let candidate = previous + delta;
                            if !candidate.is_finite() || candidate == previous {
                                continue;
                            }
                            offset[coordinate] = candidate;
                            evaluations += 1;
                            let candidate_bits = self.objective(read, question, &offset, pulls)?;
                            if candidate_bits < bits {
                                bits = candidate_bits;
                                improved = true;
                            } else {
                                offset[coordinate] = previous;
                            }
                        }
                    }
                    if !improved || evaluations >= max_evaluations {
                        break;
                    }
                }
            }
            if bits < best_bits || (bits == best_bits && offset_cmp(&offset, &best).is_lt()) {
                best = offset;
                best_bits = bits;
            }
        }
        Ok((best, evaluations, zero_bits, best_bits))
    }

    fn objective(
        &self,
        read: &SolveRead,
        question: u32,
        offset: &[f32],
        pulls: &[&AskPull],
    ) -> Result<u64, String> {
        if offset.len() != self.width {
            return Err("ask offset width differs".to_owned());
        }
        let query = self
            .table_row(question)
            .iter()
            .zip(offset)
            .map(|(&point, &offset)| point + offset)
            .collect::<Vec<_>>();
        ensure_finite(&query, "question point plus offset")?;
        let mut total = 0u128;
        for pull in pulls {
            let key = self.table_row(pull.key);
            let mut bits = 0u64;
            for head in self.heads / 2..self.heads {
                bits += u64::from(popcount_swar(
                    self.head_code(read, &query, head)? ^ self.head_code(read, key, head)?,
                ));
            }
            total = total
                .checked_add(u128::from(bits) * u128::from(pull.weight))
                .ok_or_else(|| "pull objective overflows".to_owned())?;
        }
        u64::try_from(total).map_err(|_| "pull objective exceeds u64".to_owned())
    }

    fn weighted_median_offset(
        &self,
        question: u32,
        pulls: &[&AskPull],
    ) -> Result<Vec<f32>, String> {
        let question_point = self.table_row(question);
        let mut offset = Vec::with_capacity(self.width);
        for (coordinate, &question_value) in question_point.iter().enumerate() {
            let mut values = pulls
                .iter()
                .map(|pull| (self.table_row(pull.key)[coordinate], pull.weight))
                .collect::<Vec<_>>();
            values.sort_by(|a, b| a.0.total_cmp(&b.0).then_with(|| a.1.cmp(&b.1)));
            let total = values
                .iter()
                .try_fold(0u64, |sum, (_, weight)| sum.checked_add(u64::from(*weight)))
                .ok_or_else(|| "weighted median overflows".to_owned())?;
            let mut cumulative = 0u64;
            let median = values
                .iter()
                .find_map(|(value, weight)| {
                    cumulative += u64::from(*weight);
                    (cumulative.saturating_mul(2) >= total).then_some(*value)
                })
                .ok_or_else(|| "weighted median is empty".to_owned())?;
            let value = median - question_value;
            if !value.is_finite() {
                return Err("weighted median offset is not finite".to_owned());
            }
            offset.push(value);
        }
        Ok(offset)
    }

    fn head_code(&self, read: &SolveRead, point: &[f32], head: usize) -> Result<u64, String> {
        let head_width = self.width / self.heads;
        let start = head * head_width;
        let rotated = hamilton_gate_f32(
            &read.key_rotation[start..start + head_width],
            &point[start..start + head_width],
        );
        if let Some(coordinate) = rotated.iter().position(|value| !value.is_finite()) {
            return Err(format!(
                "depth {} head {head} rotated coordinate {coordinate} is not finite",
                read.depth
            ));
        }
        Ok(pack_signs_f32(&rotated)[0])
    }

    fn table_row(&self, token: u32) -> &[f32] {
        &self.table[token as usize * self.width..][..self.width]
    }

    fn serialize_hamming(&self, tables: &BTreeMap<usize, Vec<f32>>) -> Result<Vec<u8>, String> {
        let mut tensors = self.original_tensors.clone();
        for (&depth, ask) in tables {
            tensors.push(NamedTensor {
                name: format!("d{depth}.ask"),
                dtype: Dtype::F32,
                shape: vec![self.vocab, self.width],
                data: f32_bytes(ask),
            });
        }
        tensors.sort_by(|a, b| a.name.cmp(&b.name));
        let views = tensors
            .iter()
            .map(|tensor| {
                Ok((
                    tensor.name.clone(),
                    TensorView::new(tensor.dtype, tensor.shape.clone(), &tensor.data)
                        .map_err(|error| format!("{}: {error}", tensor.name))?,
                ))
            })
            .collect::<Result<Vec<_>, String>>()?;
        let bytes = serialize(views, self.metadata.clone()).map_err(|error| error.to_string())?;
        canonicalize_safetensors_header(bytes)
    }
}

fn unit_weight() -> u32 {
    1
}

fn f32_values(tensor: &TensorView<'_>, name: &str, shape: &[usize]) -> Result<Vec<f32>, String> {
    if tensor.dtype() != Dtype::F32 || tensor.shape() != shape {
        return Err(format!(
            "{name} is {:?} {:?}, not F32 {shape:?}",
            tensor.dtype(),
            tensor.shape()
        ));
    }
    let values = tensor
        .data()
        .chunks_exact(4)
        .map(|bytes| f32::from_le_bytes(bytes.try_into().expect("four bytes")))
        .collect::<Vec<_>>();
    ensure_finite(&values, name)?;
    Ok(values)
}

fn u32_values(tensor: &TensorView<'_>, name: &str, shape: &[usize]) -> Result<Vec<u32>, String> {
    if tensor.dtype() != Dtype::U32 || tensor.shape() != shape {
        return Err(format!(
            "{name} is {:?} {:?}, not U32 {shape:?}",
            tensor.dtype(),
            tensor.shape()
        ));
    }
    Ok(tensor
        .data()
        .chunks_exact(4)
        .map(|bytes| u32::from_le_bytes(bytes.try_into().expect("four bytes")))
        .collect())
}

fn f32_bytes(values: &[f32]) -> Vec<u8> {
    values
        .iter()
        .flat_map(|value| value.to_le_bytes())
        .collect()
}

fn ensure_finite(values: &[f32], what: &str) -> Result<(), String> {
    if let Some(index) = values.iter().position(|value| !value.is_finite()) {
        return Err(format!("{what}[{index}] is not finite"));
    }
    Ok(())
}

fn safetensors_metadata(
    bytes: &[u8],
) -> Result<Option<std::collections::HashMap<String, String>>, String> {
    let len = bytes
        .get(..8)
        .and_then(|bytes| bytes.try_into().ok())
        .map(u64::from_le_bytes)
        .ok_or_else(|| "safetensors header has no length".to_owned())?;
    let len = usize::try_from(len).map_err(|_| "safetensors header is too large".to_owned())?;
    let header = bytes
        .get(8..8usize.saturating_add(len))
        .ok_or_else(|| "safetensors header is truncated".to_owned())?;
    let value: serde_json::Value =
        serde_json::from_slice(header).map_err(|error| format!("safetensors header: {error}"))?;
    value
        .get("__metadata__")
        .map(|metadata| {
            serde_json::from_value(metadata.clone())
                .map_err(|error| format!("safetensors metadata: {error}"))
        })
        .transpose()
}

fn canonicalize_safetensors_header(bytes: Vec<u8>) -> Result<Vec<u8>, String> {
    let len = bytes
        .get(..8)
        .and_then(|bytes| bytes.try_into().ok())
        .map(u64::from_le_bytes)
        .ok_or_else(|| "safetensors header has no length".to_owned())?;
    let len = usize::try_from(len).map_err(|_| "safetensors header is too large".to_owned())?;
    let data_start = 8usize
        .checked_add(len)
        .filter(|&end| end <= bytes.len())
        .ok_or_else(|| "safetensors header is truncated".to_owned())?;
    let value: serde_json::Value = serde_json::from_slice(&bytes[8..data_start])
        .map_err(|error| format!("safetensors header: {error}"))?;
    let mut header =
        serde_json::to_vec(&value).map_err(|error| format!("safetensors header: {error}"))?;
    let aligned_len = header.len().next_multiple_of(8);
    header.resize(aligned_len, b' ');
    let aligned_len =
        u64::try_from(aligned_len).map_err(|_| "safetensors header is too large".to_owned())?;
    let mut canonical = Vec::with_capacity(8 + header.len() + bytes.len() - data_start);
    canonical.extend_from_slice(&aligned_len.to_le_bytes());
    canonical.extend_from_slice(&header);
    canonical.extend_from_slice(&bytes[data_start..]);
    Ok(canonical)
}

fn offset_cmp(a: &[f32], b: &[f32]) -> std::cmp::Ordering {
    a.iter()
        .map(|value| value.to_bits())
        .cmp(b.iter().map(|value| value.to_bits()))
}

fn offset_abs_sum_q16(offset: &[f32]) -> Result<u64, String> {
    offset.iter().try_fold(0u64, |sum, &value| {
        let q16 = (f64::from(value).abs() * 65_536.0).round();
        if !q16.is_finite() || q16 > u64::MAX as f64 {
            return Err("ask offset magnitude is unrepresentable".to_owned());
        }
        sum.checked_add(q16 as u64)
            .ok_or_else(|| "ask offset magnitude overflows".to_owned())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ask_offset_audit::TrainedAskOffsets;

    fn u32_bytes(values: &[u32]) -> Vec<u8> {
        values
            .iter()
            .flat_map(|value| value.to_le_bytes())
            .collect()
    }

    fn tensor_file(tensors: Vec<(&str, Dtype, Vec<usize>, Vec<u8>)>) -> Vec<u8> {
        tensor_file_with_metadata(tensors, None)
    }

    fn tensor_file_with_metadata(
        tensors: Vec<(&str, Dtype, Vec<usize>, Vec<u8>)>,
        metadata: Option<std::collections::HashMap<String, String>>,
    ) -> Vec<u8> {
        let tensors = tensors
            .into_iter()
            .map(|(name, dtype, shape, data)| (name.to_owned(), dtype, shape, data))
            .collect::<Vec<_>>();
        let views = tensors
            .iter()
            .map(|(name, dtype, shape, data)| {
                (
                    name.clone(),
                    TensorView::new(*dtype, shape.clone(), data).unwrap(),
                )
            })
            .collect::<Vec<_>>();
        serialize(views, metadata).unwrap()
    }

    fn fixture() -> (Vec<u8>, Vec<u8>, AskPullCorpus) {
        let (vocab, width) = (64usize, 8usize);
        let mut table = vec![1.0f32; vocab * width];
        table[17 * width..18 * width].fill(-1.0);
        let model = tensor_file(vec![(
            "embedding.weight",
            Dtype::F32,
            vec![vocab, width],
            f32_bytes(&table),
        )]);
        let rotation = (0..width / 4)
            .flat_map(|_| [1.0f32, 0.0, 0.0, 0.0])
            .collect::<Vec<_>>();
        let hamming = tensor_file(vec![
            (
                "heads",
                Dtype::U32,
                vec![1],
                [2u32].into_iter().flat_map(u32::to_le_bytes).collect(),
            ),
            (
                "d2.key_rotation",
                Dtype::F32,
                vec![width],
                f32_bytes(&rotation),
            ),
            (
                "d6.key_rotation",
                Dtype::F32,
                vec![width],
                f32_bytes(&rotation),
            ),
        ]);
        let corpus = AskPullCorpus {
            schema: PULL_CORPUS_SCHEMA.to_owned(),
            pulls: vec![AskPull {
                question: 33,
                key: 17,
                weight: 1,
            }],
        };
        (model, hamming, corpus)
    }

    #[test]
    fn solves_b2_style_pull_and_preserves_hamming_tensors() {
        let (model, hamming, corpus) = fixture();
        let basis = AskOffsetSolveBasis::from_bytes(&model, &hamming).unwrap();
        let solved = basis.solve(&corpus, 100_000).unwrap();
        assert_eq!(solved.readings.len(), 2);
        for reading in &solved.readings {
            assert_eq!(reading.zero_weighted_bits, 4);
            assert_eq!(reading.solved_weighted_bits, 0);
            assert_eq!(reading.improvement_bits, 4);
        }
        let parsed = TrainedAskOffsets::from_bytes(&model, &solved.hamming_bytes).unwrap();
        let audit = parsed.audit_relations(16..32, 16).unwrap();
        let target = audit.iter().find(|reading| reading.relation == 17).unwrap();
        assert!(target.cells.iter().all(|cell| cell.trained_bits == 0));
    }

    #[test]
    fn repeated_solve_is_byte_identical_and_multiple_targets_do_not_regress() {
        let (model, hamming, mut corpus) = fixture();
        corpus.pulls.push(AskPull {
            question: 33,
            key: 18,
            weight: 3,
        });
        let basis = AskOffsetSolveBasis::from_bytes(&model, &hamming).unwrap();
        let first = basis.solve(&corpus, 20_000).unwrap();
        let second = basis.solve(&corpus, 20_000).unwrap();
        assert_eq!(first.hamming_bytes, second.hamming_bytes);
        let limited = basis.solve(&corpus, 1).unwrap();
        assert!(limited
            .readings
            .iter()
            .all(|reading| reading.evaluations <= 1));
        assert!(basis.solve(&corpus, 0).is_err());
        assert!(first
            .readings
            .iter()
            .all(|reading| reading.solved_weighted_bits <= reading.zero_weighted_bits));
        assert!(first
            .readings
            .iter()
            .any(|reading| reading.target_keys == 2));
    }

    #[test]
    fn malformed_corpora_are_refused() {
        let (model, hamming, _) = fixture();
        let basis = AskOffsetSolveBasis::from_bytes(&model, &hamming).unwrap();
        for pull in [
            AskPull {
                question: 64,
                key: 17,
                weight: 1,
            },
            AskPull {
                question: 33,
                key: 17,
                weight: 0,
            },
        ] {
            let corpus = AskPullCorpus {
                schema: PULL_CORPUS_SCHEMA.to_owned(),
                pulls: vec![pull],
            };
            assert!(basis.solve(&corpus, 10).is_err());
        }
        assert!(basis
            .solve(
                &AskPullCorpus {
                    schema: PULL_CORPUS_SCHEMA.to_owned(),
                    pulls: vec![],
                },
                10,
            )
            .is_err());
    }

    #[test]
    fn malformed_and_nonfinite_checkpoints_are_refused() {
        let (vocab, width, heads) = (4usize, 8usize, 2usize);
        let table = vec![0.5f32; vocab * width];
        let model = tensor_file(vec![(
            "embedding.weight",
            Dtype::F32,
            vec![vocab, width],
            f32_bytes(&table),
        )]);
        let rotation = (0..width / 4)
            .flat_map(|_| [1.0f32, 0.0, 0.0, 0.0])
            .collect::<Vec<_>>();
        let valid_hamming = |head_count: u32| {
            tensor_file(vec![
                ("heads", Dtype::U32, vec![1], u32_bytes(&[head_count])),
                (
                    "d2.key_rotation",
                    Dtype::F32,
                    vec![width],
                    f32_bytes(&rotation),
                ),
            ])
        };
        assert!(AskOffsetSolveBasis::from_bytes(&model, &valid_hamming(heads as u32)).is_ok());

        let mut nonfinite_model_values = table.clone();
        nonfinite_model_values[0] = f32::NAN;
        let nonfinite_model = tensor_file(vec![(
            "embedding.weight",
            Dtype::F32,
            vec![vocab, width],
            f32_bytes(&nonfinite_model_values),
        )]);
        assert!(
            AskOffsetSolveBasis::from_bytes(&nonfinite_model, &valid_hamming(heads as u32))
                .err()
                .expect("non-finite table")
                .contains("embedding.weight[0] is not finite")
        );

        assert!(AskOffsetSolveBasis::from_bytes(&model, &valid_hamming(1))
            .err()
            .expect("one head")
            .contains("1 heads over width 8 is invalid"));

        let mut nonfinite_rotation = rotation.clone();
        nonfinite_rotation[0] = f32::INFINITY;
        let nonfinite_hamming = tensor_file(vec![
            ("heads", Dtype::U32, vec![1], u32_bytes(&[heads as u32])),
            (
                "d2.key_rotation",
                Dtype::F32,
                vec![width],
                f32_bytes(&nonfinite_rotation),
            ),
        ]);
        assert!(AskOffsetSolveBasis::from_bytes(&model, &nonfinite_hamming)
            .err()
            .expect("non-finite rotation")
            .contains("d2.key_rotation[0] is not finite"));

        let partial_hamming = tensor_file(vec![
            ("heads", Dtype::U32, vec![1], u32_bytes(&[heads as u32])),
            (
                "d2.key_rotation",
                Dtype::F32,
                vec![width],
                f32_bytes(&rotation),
            ),
            ("d4.layer", Dtype::U32, vec![1], u32_bytes(&[7])),
        ]);
        assert!(AskOffsetSolveBasis::from_bytes(&model, &partial_hamming)
            .err()
            .expect("partial depth")
            .contains("d4.key_rotation"));

        let type_spin_hamming = tensor_file(vec![
            ("heads", Dtype::U32, vec![1], u32_bytes(&[heads as u32])),
            (
                "d2.key_rotation",
                Dtype::F32,
                vec![width],
                f32_bytes(&rotation),
            ),
            (
                "d2.step_rotation",
                Dtype::F32,
                vec![width],
                f32_bytes(&rotation),
            ),
        ]);
        assert!(AskOffsetSolveBasis::from_bytes(&model, &type_spin_hamming)
            .err()
            .expect("type spin")
            .contains("mutually exclusive"));
    }

    #[test]
    fn metadata_is_preserved_and_serialized_canonically() {
        let (model, hamming, corpus) = fixture();
        let original = SafeTensors::deserialize(&hamming).unwrap();
        let tensors = original.names().into_iter().map(|name| {
            let tensor = original.tensor(name).unwrap();
            (
                name,
                tensor.dtype(),
                tensor.shape().to_vec(),
                tensor.data().to_vec(),
            )
        });
        let metadata = Some(std::collections::HashMap::from([
            ("z-last".to_owned(), "2".to_owned()),
            ("a-first".to_owned(), "1".to_owned()),
        ]));
        let hamming = tensor_file_with_metadata(tensors.collect(), metadata);
        let basis = AskOffsetSolveBasis::from_bytes(&model, &hamming).unwrap();
        let solved = basis.solve(&corpus, 10_000).unwrap();
        assert_eq!(
            safetensors_metadata(&solved.hamming_bytes).unwrap(),
            Some(std::collections::HashMap::from([
                ("z-last".to_owned(), "2".to_owned()),
                ("a-first".to_owned(), "1".to_owned()),
            ]))
        );
        assert_eq!(
            canonicalize_safetensors_header(solved.hamming_bytes.clone()).unwrap(),
            solved.hamming_bytes
        );
    }

    #[test]
    #[ignore = "release benchmark"]
    fn release_width_256_two_depth_solver_benchmark() {
        let (vocab, width, heads) = (4_096usize, 256usize, 4usize);
        let table = (0..vocab * width)
            .map(|index| ((index.wrapping_mul(37).wrapping_add(11) % 257) as f32 - 128.0) / 64.0)
            .collect::<Vec<_>>();
        let model = tensor_file(vec![(
            "embedding.weight",
            Dtype::F32,
            vec![vocab, width],
            f32_bytes(&table),
        )]);
        let rotation = (0..width / 4)
            .flat_map(|_| [1.0f32, 0.0, 0.0, 0.0])
            .collect::<Vec<_>>();
        let mut tensors = vec![(
            "heads",
            Dtype::U32,
            vec![1],
            [heads as u32]
                .into_iter()
                .flat_map(u32::to_le_bytes)
                .collect(),
        )];
        let rotation_names = [format!("d2.key_rotation"), format!("d6.key_rotation")];
        for depth in [2usize, 6] {
            tensors.push((
                rotation_names[if depth == 2 { 0 } else { 1 }].as_str(),
                Dtype::F32,
                vec![width],
                f32_bytes(&rotation),
            ));
        }
        let hamming = tensor_file(tensors);
        let basis = AskOffsetSolveBasis::from_bytes(&model, &hamming).unwrap();
        let mut corpus = AskPullCorpus::b2_relations(16..32, 16);
        let second_targets = corpus
            .pulls
            .iter()
            .map(|pull| AskPull {
                question: pull.question,
                key: pull.key + 16,
                weight: 1,
            })
            .collect::<Vec<_>>();
        corpus.pulls.extend(second_targets);
        let started = std::time::Instant::now();
        let solved = basis.solve(&corpus, 200_000).unwrap();
        let elapsed = started.elapsed();
        assert!(solved
            .readings
            .iter()
            .all(|reading| reading.solved_weighted_bits <= reading.zero_weighted_bits));
        eprintln!(
            "width=256 depths=2 questions=16 evaluations={} wall_ms={}",
            solved
                .readings
                .iter()
                .map(|reading| reading.evaluations)
                .sum::<u64>(),
            elapsed.as_millis()
        );
    }
}
