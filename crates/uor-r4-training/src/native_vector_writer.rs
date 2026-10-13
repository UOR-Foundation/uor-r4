//! One learned vector table for token input, addressed output and carried tokens.
//! Model arithmetic is CUDA-only. Product addresses are identity metadata, not
//! a semantic distance inferred from token numbers. Geometry scores displacement
//! in learned vectors; fixed SpiralCore atlas ports admit a bounded candidate set.
use crate::{invalid, sha256_bytes, Result};
use candle_core::{DType, IndexOp, Tensor};
use serde::{Deserialize, Serialize};

pub const SCHEMA: &str = "uor-r4.shared-native-vector-writer/1";
pub const CELLS: usize = 256;
pub const MAX_PER_CELL: usize = 32;
pub const DEFAULT_KEPT_CELLS: usize = 8;
const ABSENT_SCORE: f32 = -1.0e30;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WriterSpec {
    pub vocabulary: usize,
    pub width: usize,
    pub kept_cells: usize,
}
impl WriterSpec {
    pub fn validate(&self) -> Result<()> {
        if !(CELLS..=CELLS * MAX_PER_CELL).contains(&self.vocabulary)
            || self.width == 0
            || self.width % 8 != 0
            || self.width > 1024
            || !(1..=16).contains(&self.kept_cells)
        {
            return Err(invalid("native vector writer needs 256..8192 identities, whole 8D blocks <=1024, and 1..16 product cells"));
        }
        Ok(())
    }
    pub fn binding(&self) -> Result<serde_json::Value> {
        self.validate()?;
        let plan = crate::spiralcore_fbs::ProjectionPlan::new(8, 8)?;
        Ok(serde_json::json!({"schema":SCHEMA, "spec":self,
            "source_sha256":sha256_bytes(include_bytes!("native_vector_writer.rs")),
            "geometry":plan.fingerprint,
            "identity_partition":"token-id-modulo-256; quotient is collision slot",
            "query_ports":"first-and-last-whole-8D-block; alias at width8",
            "score":"negative-mean-L1-displacement; raw",
            "address_credit":"target-cell-versus-admitted-cell-margin; surrogate"}))
    }
}

#[derive(Clone)]
pub struct NativeVectorWriter {
    table: Tensor,
    spec: WriterSpec,
    roots: Tensor,
    slots: usize,
    ids: Tensor,
    valid: Tensor,
}
/// IDs and masks accompany bounded raw scores. No omitted target is inserted.
pub struct VectorCandidates {
    pub token_ids: Tensor,
    pub valid: Tensor,
    pub raw_scores: Tensor,
    pub cell_ids: Tensor,
    pub address_scores: Tensor,
    query: Tensor,
    table_id: usize,
}
/// Exact occurrence/version identity stays separate from its geometric vector.
#[derive(Clone)]
pub struct CarriedVectorToken {
    pub token_id: u32,
    pub occurrence: u64,
    pub version: u64,
    vector: Tensor,
    table_id: usize,
}
impl CarriedVectorToken {
    pub fn vector(&self) -> &Tensor {
        &self.vector
    }
}
impl NativeVectorWriter {
    pub fn new(table: &Tensor, spec: WriterSpec) -> Result<Self> {
        // Refuse before any model arithmetic, including validation reductions.
        if !table.device().is_cuda() {
            return Err(invalid(
                "native vector writer requires CUDA; no host fallback",
            ));
        }
        spec.validate()?;
        if table.dtype() != DType::F32 || table.dims() != [spec.vocabulary, spec.width] {
            return Err(invalid("shared vector table shape/dtype differs"));
        }
        let g = crate::spiralcore_fbs::ProjectionPlan::new(8, 8)?;
        let roots = Tensor::from_vec(
            g.geometry
                .roots
                .iter()
                .flatten()
                .map(|v| f32::from(*v))
                .collect::<Vec<_>>(),
            (16, 8),
            table.device(),
        )?;
        let slots = spec.vocabulary.div_ceil(CELLS);
        let mut ids = Vec::with_capacity(CELLS * slots);
        let mut valid = Vec::with_capacity(CELLS * slots);
        for cell in 0..CELLS {
            for slot in 0..slots {
                let id = cell + CELLS * slot;
                ids.push(if id < spec.vocabulary { id as u32 } else { 0 });
                valid.push(u8::from(id < spec.vocabulary));
            }
        }
        Ok(Self {
            table: table.clone(),
            roots,
            slots,
            ids: Tensor::from_vec(ids, (CELLS, slots), table.device())?,
            valid: Tensor::from_vec(valid, (CELLS, slots), table.device())?,
            spec,
        })
    }
    pub fn spec(&self) -> &WriterSpec {
        &self.spec
    }
    pub fn table(&self) -> &Tensor {
        &self.table
    }
    pub fn table_id(&self) -> usize {
        self.table.id()
    }
    fn check(&self, value: &Tensor) -> Result<()> {
        if !value.device().is_cuda()
            || !value.device().same_device(self.table.device())
            || value.dtype() != DType::F32
        {
            return Err(invalid(
                "native vector arithmetic needs same-device CUDA F32",
            ));
        }
        // Only a validation status crosses to the host; no scores or gradients.
        if value.abs()?.le(1.0e6f32)?.min_all()?.to_scalar::<u8>()? == 0 {
            return Err(invalid(
                "native vector nonfinite or outside declared arithmetic bound",
            ));
        }
        Ok(())
    }
    pub fn encode(&self, ids: &[u32]) -> Result<Tensor> {
        if ids.is_empty() || ids.iter().any(|&id| id as usize >= self.spec.vocabulary) {
            return Err(invalid("native vector token inventory invalid"));
        }
        let at = Tensor::from_vec(ids.to_vec(), ids.len(), self.table.device())?;
        let vectors = self.table.index_select(&at, 0)?;
        self.check(&vectors)?;
        Ok(vectors)
    }
    pub fn carry(
        &self,
        token_id: u32,
        occurrence: u64,
        version: u64,
    ) -> Result<CarriedVectorToken> {
        Ok(CarriedVectorToken {
            token_id,
            occurrence,
            version,
            vector: self.encode(&[token_id])?,
            table_id: self.table.id(),
        })
    }
    pub fn read_carried(&self, token: &CarriedVectorToken) -> Result<Tensor> {
        if token.table_id != self.table.id() || token.token_id as usize >= self.spec.vocabulary {
            return Err(invalid(
                "carried token belongs to another shared parameter revision",
            ));
        }
        self.check(&token.vector)?;
        Ok(token.vector.clone())
    }
    fn address_scores(&self, query: &Tensor) -> Result<Tensor> {
        let rows = query.dim(0)?;
        let port = |start| -> Result<Tensor> {
            Ok(query
                .narrow(1, start, 8)?
                .unsqueeze(1)?
                .broadcast_sub(&self.roots.unsqueeze(0)?)?
                .abs()?
                .mean(2)?
                .neg()?)
        };
        let a = port(0)?;
        let b = port(self.spec.width - 8)?;
        Ok(a.unsqueeze(2)?
            .broadcast_add(&b.unsqueeze(1)?)?
            .reshape((rows, CELLS))?)
    }
    pub fn candidates(&self, query: &Tensor) -> Result<VectorCandidates> {
        if query.dims().len() != 2
            || query.dim(0)? == 0
            || query.dim(0)? > 4096
            || query.dim(1)? != self.spec.width
        {
            return Err(invalid(
                "native writer query needs 1..4096 rows of complete vector blocks",
            ));
        }
        self.check(query)?;
        let query = query.to_dtype(DType::F32)?;
        let rows = query.dim(0)?;
        let address_scores = self.address_scores(&query)?;
        let addresses = Tensor::from_vec(
            (0..CELLS as u32).collect::<Vec<_>>(),
            CELLS,
            self.table.device(),
        )?
        .unsqueeze(0)?
        .broadcast_as((rows, CELLS))?;
        let mut left = address_scores.detach();
        let mut cells = Vec::new();
        for _ in 0..self.spec.kept_cells {
            // Exact score tie -> smallest cell identity, never a score epsilon.
            let best = left.max_keepdim(1)?;
            let tied = left.broadcast_eq(&best)?;
            let absent = Tensor::full(u32::MAX, (rows, CELLS), self.table.device())?;
            let cell = tied.where_cond(&addresses, &absent)?.min_keepdim(1)?;
            left = addresses.broadcast_eq(&cell)?.where_cond(
                &Tensor::full(ABSENT_SCORE, (rows, CELLS), self.table.device())?,
                &left,
            )?;
            cells.push(cell);
        }
        let cell_ids = Tensor::cat(&cells, 1)?;
        let flat = cell_ids.flatten_all()?;
        let count = self.spec.kept_cells * self.slots;
        let token_ids = self.ids.index_select(&flat, 0)?.reshape((rows, count))?;
        let valid = self.valid.index_select(&flat, 0)?.reshape((rows, count))?;
        let vectors = self
            .table
            .index_select(&token_ids.flatten_all()?, 0)?
            .reshape((rows, count, self.spec.width))?;
        self.check(&vectors)?;
        let scores = query
            .unsqueeze(1)?
            .broadcast_sub(&vectors)?
            .abs()?
            .mean(2)?
            .neg()?;
        let raw_scores = valid.where_cond(
            &scores,
            &Tensor::full(ABSENT_SCORE, (rows, count), self.table.device())?,
        )?;
        Ok(VectorCandidates {
            token_ids,
            valid,
            raw_scores,
            cell_ids,
            address_scores,
            query,
            table_id: self.table.id(),
        })
    }
    pub fn write(&self, query: &Tensor) -> Result<Tensor> {
        let c = self.candidates(query)?;
        let best = c.raw_scores.max_keepdim(1)?;
        let tied = c.raw_scores.broadcast_eq(&best)?.mul(&c.valid)?;
        Ok(tied
            .where_cond(
                &c.token_ids,
                &Tensor::full(u32::MAX, c.token_ids.shape(), self.table.device())?,
            )?
            .min(1)?)
    }
    /// Native loss explicitly scores an omitted target but never changes admission.
    /// Admission margin supplies a labelled auxiliary surrogate, not hard-topK's derivative.
    pub fn margin(&self, c: &VectorCandidates, targets: &[u32], rivals: usize) -> Result<Tensor> {
        let rows = c.query.dim(0)?;
        if c.table_id != self.table.id() || targets.len() != rows || !(1..=8).contains(&rivals) {
            return Err(invalid(
                "native vector objective binding/target/rival inventory differs",
            ));
        }
        let target_vectors = self.encode(targets)?;
        let target_score = c
            .query
            .sub(&target_vectors)?
            .abs()?
            .mean(1)?
            .neg()?
            .unsqueeze(1)?;
        let target_ids = Tensor::from_vec(targets.to_vec(), (rows, 1), self.table.device())?;
        let available = c.token_ids.broadcast_ne(&target_ids)?.mul(&c.valid)?;
        if available
            .to_dtype(DType::U32)?
            .sum(1)?
            .ge(rivals as u32)?
            .min_all()?
            .to_scalar::<u8>()?
            == 0
        {
            return Err(invalid(
                "declared rivals exceed admitted non-target identities",
            ));
        }
        // For V>=256 each cell contains at least one real token; exclude self.
        let mut selected = available.where_cond(
            &c.raw_scores,
            &Tensor::full(ABSENT_SCORE, c.raw_scores.shape(), self.table.device())?,
        )?;
        let mut losses = Vec::new();
        let indices = Tensor::from_vec(
            (0..selected.dim(1)? as u32).collect::<Vec<_>>(),
            selected.dim(1)?,
            self.table.device(),
        )?
        .unsqueeze(0)?
        .broadcast_as(selected.shape())?;
        for _ in 0..rivals {
            let at = selected.argmax_keepdim(1)?;
            let rival = selected.gather(&at, 1)?;
            let hinge = rival
                .broadcast_sub(&target_score)?
                .affine(1., 1.)?
                .maximum(&Tensor::zeros((rows, 1), DType::F32, self.table.device())?)?;
            losses.push(hinge);
            selected = indices.broadcast_eq(&at)?.where_cond(
                &Tensor::full(ABSENT_SCORE, selected.shape(), self.table.device())?,
                &selected,
            )?;
        }
        let target_cells = Tensor::from_vec(
            targets
                .iter()
                .map(|id| id % CELLS as u32)
                .collect::<Vec<_>>(),
            (rows, 1),
            self.table.device(),
        )?;
        let target_address = c.address_scores.gather(&target_cells, 1)?;
        let admitted = c.address_scores.gather(&c.cell_ids, 1)?;
        let other = c.cell_ids.broadcast_ne(&target_cells)?;
        let address_rival = other
            .where_cond(
                &admitted,
                &Tensor::full(ABSENT_SCORE, admitted.shape(), self.table.device())?,
            )?
            .max_keepdim(1)?;
        let address_loss = address_rival
            .sub(&target_address)?
            .affine(1., 1.)?
            .maximum(&Tensor::zeros((rows, 1), DType::F32, self.table.device())?)?;
        Ok(Tensor::cat(&losses, 1)?
            .mean_all()?
            .add(&address_loss.mean_all()?)?)
    }
}

impl NativeVectorWriter {
    /// Full-V compatibility allocation, with no additional vector comparisons
    /// or parameter scan. The native margin never consumes omitted sentinels.
    pub fn compatibility_scores(&self, query: &Tensor) -> Result<Tensor> {
        let c = self.candidates(query)?;
        let rows = query.dim(0)?;
        let zero = Tensor::zeros(c.raw_scores.shape(), DType::F32, self.table.device())?;
        let values = c.valid.where_cond(&c.raw_scores, &zero)?;
        let base = Tensor::zeros(
            (rows, self.spec.vocabulary),
            DType::F32,
            self.table.device(),
        )?;
        let scores = base.scatter_add(&c.token_ids, &values, 1)?;
        let present = base
            .scatter_add(&c.token_ids, &c.valid.to_dtype(DType::F32)?, 1)?
            .gt(0f32)?;
        Ok(present.where_cond(
            &scores,
            &Tensor::full(ABSENT_SCORE, scores.shape(), self.table.device())?,
        )?)
    }
}

/// The compiler/store/emitter path borrows the very same learned parameter
/// tensor as the stack. It cannot rebuild another prototype table or silently
/// reinterpret an old packed/probability artifact. Exact store control is host
/// metadata; encode, carrier transport, selection and learning stay on CUDA.
#[derive(Clone)]
pub struct NativeVectorChannel {
    writer: NativeVectorWriter,
}
impl NativeVectorChannel {
    pub fn new(writer: NativeVectorWriter) -> Self {
        Self { writer }
    }
    pub fn writer(&self) -> &NativeVectorWriter {
        &self.writer
    }
    pub fn compile_tokens(
        &self,
        ids: &[u32],
        first_occurrence: u64,
        version: u64,
    ) -> Result<Vec<CarriedVectorToken>> {
        let vectors = self.writer.encode(ids)?;
        ids.iter()
            .enumerate()
            .map(|(i, &token_id)| {
                Ok(CarriedVectorToken {
                    token_id,
                    occurrence: first_occurrence
                        .checked_add(i as u64)
                        .ok_or_else(|| invalid("vector occurrence overflow"))?,
                    version,
                    vector: vectors.narrow(0, i, 1)?,
                    table_id: self.writer.table_id(),
                })
            })
            .collect()
    }
    pub fn store_read(
        &self,
        token: &CarriedVectorToken,
        occurrence: u64,
        version: u64,
    ) -> Result<Tensor> {
        if token.occurrence != occurrence || token.version != version {
            return Err(invalid("vector occurrence/version witness differs"));
        }
        self.writer.read_carried(token)
    }
    pub fn transport(&self, token: &CarriedVectorToken, queries: &Tensor) -> Result<Tensor> {
        let carrier = self.writer.read_carried(token)?;
        crate::spiralcore_fbs::ProjectionPlan::new(self.writer.spec.width, self.writer.spec.width)?
            .forward(queries, &carrier)
    }
    pub fn emit(&self, vector: &Tensor) -> Result<Tensor> {
        self.writer.write(vector)
    }
    pub fn loss(&self, vector: &Tensor, targets: &[u32], rivals: usize) -> Result<Tensor> {
        self.writer
            .margin(&self.writer.candidates(vector)?, targets, rivals)
    }
}

#[cfg(test)]
mod contract_tests {
    use super::*;
    #[test]
    fn binding_and_cpu_refusal() -> Result<()> {
        let spec = WriterSpec {
            vocabulary: 4096,
            width: 8,
            kept_cells: 8,
        };
        let binding = spec.binding()?;
        assert_eq!(binding["schema"], SCHEMA);
        assert_eq!(binding, spec.binding()?);
        assert!(WriterSpec {
            width: 7,
            ..spec.clone()
        }
        .validate()
        .is_err());
        assert!(WriterSpec {
            vocabulary: 8193,
            ..spec.clone()
        }
        .validate()
        .is_err());
        let table = Tensor::zeros((4096, 8), DType::F32, &candle_core::Device::Cpu)?;
        assert!(NativeVectorWriter::new(&table, spec).is_err());
        Ok(())
    }
}
#[cfg(all(test, feature = "cuda"))]
mod cuda_tests {
    use super::*;
    use candle_core::{Device, Var};
    fn fixture(vocab: usize, width: usize, kept: usize) -> Result<(Var, NativeVectorWriter)> {
        let device = Device::new_cuda(0)?; // Missing CUDA is a failure, not a skipped PASS.
                                           // Fixed input constants, not a host-side forward/reference calculation.
        let table = Var::from_tensor(&Tensor::zeros((vocab, width), DType::F32, &device)?)?;
        let writer = NativeVectorWriter::new(
            table.as_tensor(),
            WriterSpec {
                vocabulary: vocab,
                width,
                kept_cells: kept,
            },
        )?;
        Ok((table, writer))
    }
    #[test]
    fn ties_identity_and_shared_channel() -> Result<()> {
        let (_, w) = fixture(257, 8, 8)?;
        let q = Tensor::zeros((1, 8), DType::F32, w.table.device())?;
        let c = w.candidates(&q)?;
        assert_eq!(
            c.cell_ids.to_vec2::<u32>()?[0],
            vec![0, 1, 2, 3, 4, 5, 6, 7]
        );
        assert_eq!(w.write(&q)?.to_vec1::<u32>()?, vec![0]);
        // Repeated identities at collision slots survive; padding never emits.
        assert_eq!(c.token_ids.to_vec2::<u32>()?[0][1], 256);
        assert_eq!(c.valid.to_vec2::<u8>()?[0][3], 0);
        let channel = NativeVectorChannel::new(w.clone());
        assert_eq!(channel.writer.table_id(), w.table_id());
        let tokens = channel.compile_tokens(&[256, 0], 42, 9)?;
        assert_eq!(tokens[0].token_id, 256);
        assert_eq!(tokens[1].occurrence, 43);
        assert_eq!(
            channel.store_read(&tokens[0], 42, 9)?.to_vec2::<f32>()?,
            vec![vec![0.; 8]]
        );
        assert!(channel.store_read(&tokens[0], 42, 8).is_err());
        assert_eq!(
            channel
                .emit(&channel.store_read(&tokens[0], 42, 9)?)?
                .to_vec1::<u32>()?,
            w.write(&q)?.to_vec1::<u32>()?
        );
        let (_, other) = fixture(257, 8, 8)?;
        assert!(other.read_carried(&tokens[0]).is_err());
        Ok(())
    }
    #[test]
    fn local_distance_adjoints_and_omitted_target() -> Result<()> {
        let (table, w) = fixture(512, 16, 8)?;
        let q = Var::from_tensor(&Tensor::full(0.5f32, (1, 16), w.table.device())?)?;
        let c = w.candidates(q.as_tensor())?;
        // Fixed analytic expectation: displacement .5 in each coordinate.
        for value in c.raw_scores.to_vec2::<f32>()?[0].iter() {
            assert_eq!(*value, -0.5);
        }
        let grads = c.raw_scores.i((0, 0))?.backward()?;
        assert_eq!(
            grads
                .get(q.as_tensor())
                .ok_or_else(|| invalid("query credit absent"))?
                .to_vec2::<f32>()?,
            vec![vec![-0.0625; 16]]
        );
        let tg = grads
            .get(table.as_tensor())
            .ok_or_else(|| invalid("table credit absent"))?
            .to_vec2::<f32>()?;
        let first = c.token_ids.to_vec2::<u32>()?[0][0] as usize;
        assert_eq!(tg[first], vec![0.0625; 16]);
        assert_eq!(tg.iter().filter(|r| r.iter().any(|x| *x != 0.)).count(), 1);
        let admitted = c.token_ids.to_vec2::<u32>()?[0].clone();
        let target = (0..512u32)
            .find(|id| !admitted.contains(id))
            .ok_or_else(|| invalid("fixture needs omitted target"))?;
        let before = c.token_ids.to_vec2::<u32>()?;
        let loss = w.margin(&c, &[target], 1)?;
        assert!(loss.to_scalar::<f32>()?.is_finite());
        let gradients = loss.backward()?;
        assert!(
            gradients
                .get(table.as_tensor())
                .ok_or_else(|| invalid("omitted target credit absent"))?
                .i(target as usize)?
                .abs()?
                .sum_all()?
                .to_scalar::<f32>()?
                > 0.
        );
        assert_eq!(c.token_ids.to_vec2::<u32>()?, before); // No target-injected inference.
        assert_eq!(
            NativeVectorChannel::new(w.clone())
                .loss(q.as_tensor(), &[target], 1)?
                .to_scalar::<f32>()?,
            loss.to_scalar::<f32>()?
        );
        Ok(())
    }
    #[test]
    fn numerical_and_device_boundaries() -> Result<()> {
        let (_, w) = fixture(256, 8, 8)?;
        let dev = w.table.device();
        assert!(w.candidates(&Tensor::full(f32::NAN, (1, 8), dev)?).is_err());
        assert!(w
            .candidates(&Tensor::zeros((1, 7), DType::F32, dev)?)
            .is_err());
        assert!(w
            .candidates(&Tensor::zeros((1, 8), DType::F32, &Device::Cpu)?)
            .is_err());
        assert!(w.encode(&[256]).is_err());
        assert!(w.encode(&[]).is_err());
        Ok(())
    }
}
