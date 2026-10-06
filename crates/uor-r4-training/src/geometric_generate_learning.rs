//! Offline learning of token H4 tuples and shared native Generate factors.
//!
//! Serving is the core integer/table model. This graph anchors its exact Q24
//! scores and differentiates selected quarter-grid coefficients separately
//! from conditional full-120 state and prototype choices. Other lanes and the
//! opposite endpoint stay factual. This is a local normalized-choice surrogate,
//! not a derivative of argmax or a full recurrent joint posterior. Earlier
//! context recurrence retains its own declared approximation.
//!
//! `prepare_native` is explicit source/export admission: it downloads masters
//! and selects hard prototypes. Refresh it and all graphs after every optimizer
//! update. CUDA forward never downloads dynamic probabilities or adjoints.
//! The caller binds state codes and state_logits to the same fresh causal replay;
//! this module cannot establish the token history from a tensor shape.

use std::collections::BTreeMap;

use candle_core::{DType, Device, Tensor, Var};
use serde::Serialize;
use sha2::{Digest, Sha256};
use uor_r4_core::native_geometric::learner::{
    geometric_generate::{GenerateReadCounts, NativeGeometricGenerate, MAX_LANES, SCORE_SHIFT},
    integrated_attention::geometry::{EnergyTables, LanePair},
};
use uor_r4_integer::{
    geometric_source_actions::SourceActionBinding,
    geometric_vocabulary_actions::{VocabularyAction, VocabularyActionTrace},
    h4_tables::{H4Code, ROOT_COUNT},
};

use crate::{
    geometric_context::{q4_project_tensor, q4_shadow_ste},
    invalid, Result,
};

pub const SURROGATE: &str = "native-Q24-hard-anchor;selected-quarter-grid-coefficient-STE;detached-factors-full120-local-state-and-token-code-choice-credit;other-lanes-and-opposite-endpoint-factual;existing-softmax-choice-differential;common-final-clip8nat;fresh-export-required/1";
const Q24: f64 = 16_777_216.;
// Quarter-valued shadows times this factor equal integer nibble <<20 /2^24.
const QUARTER_TO_NATS: f64 = 0.25;

pub struct GenerateLearningWeights {
    binding: SourceActionBinding,
    lanes: usize,
    edges: Vec<LanePair>,
    /// Unquantized offline choice logits, never included in native serving.
    pub prototype_choices: Var,
    /// Quarter-valued master coefficients; strict projected range +/-1.75.
    pub unary: Var,
    pub pair: Var,
    pub bias: Var,
}

/// Frozen export admission. Its native model, not live float argmax, chooses
/// prototypes and supplies factual scores. An optimizer update invalidates it.
pub struct PreparedGenerateLearning {
    pub native: NativeGeometricGenerate,
    pub downloaded_master_bytes: usize,
    pub prototype_choice_sha256: String,
    cache: GenerateDeviceIndexCache,
}

struct GenerateDeviceIndexCache {
    payload_sha256: String,
    // Rows are factual state IDs, columns are prototype IDs, values inv(s)*p.
    relative: Tensor,
    index_multiplier: Tensor,
    prototypes: Vec<Tensor>,
    // Rows token IDs, columns alternative state IDs; independent of position.
    state_relatives: Vec<Tensor>,
    code_probability: Tensor,
    staged_bytes: usize,
    device_index_bytes: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct GenerateLearningCosts {
    pub vocabulary_rows: usize,
    pub lanes: usize,
    pub ordered_pairs: usize,
    pub conditional_choice_rows: usize,
    /// Static CPU geometry indices staged to the live device, not adjoints.
    pub staged_index_bytes: usize,
    pub staged_hard_score_bytes: usize,
    pub export_master_download_bytes: usize,
    pub max_conditional_utility_elements: usize,
    /// Zero CUDA dynamic-vector downloads; one scalar finite-status read in
    /// loss is reported separately from explicit prepare/export downloads.
    pub loss_status_scalars: usize,
    pub snapshot_staged_index_bytes: usize,
    pub snapshot_device_index_bytes: usize,
    pub per_position_device_index_elements: usize,
    pub snapshot_choice_probability_bytes: usize,
    pub generate_only_loss_staged_bytes_upper: usize,
}

pub struct GenerateLearningOutput {
    /// Full token-ID order including sparse holes; pool owns legal admission.
    pub raw_scores: Tensor,
    pub clipped_scores: Tensor,
    pub scores_q24: Vec<i64>,
    pub native_counts: GenerateReadCounts,
    pub costs: GenerateLearningCosts,
}

fn device_admit(device: &Device) -> Result<()> {
    if !device.is_cpu() && !device.is_cuda() {
        return Err(invalid(
            "Generate learning supports CPU reference or CUDA only",
        ));
    }
    #[cfg(not(feature = "cuda"))]
    if device.is_cuda() {
        return Err(invalid("Generate CUDA requested without cuda feature"));
    }
    Ok(())
}

fn quarter_codes(value: &Var) -> Result<Vec<i8>> {
    let values = value.flatten_all()?.to_vec1::<f32>()?;
    if values.iter().any(|x| !x.is_finite() || x.abs() > 1.75) {
        return Err(invalid(
            "Generate export coefficient outside strict quarter range",
        ));
    }
    Ok(values.into_iter().map(|x| (x * 4.).round() as i8).collect())
}

fn packed_nibbles(values: &[i8]) -> Result<Vec<u8>> {
    if values.iter().any(|x| !(-7..=7).contains(x)) {
        return Err(invalid(
            "Generate bias coefficient outside strict nibble range",
        ));
    }
    let mut out = vec![0; values.len().div_ceil(2)];
    for (i, &v) in values.iter().enumerate() {
        out[i / 2] |= ((v as u8) & 15) << ((i & 1) * 4);
    }
    Ok(out)
}

fn choice_digest(values: &[f32]) -> String {
    let mut hash = Sha256::new();
    for value in values {
        hash.update(value.to_bits().to_le_bytes());
    }
    hex::encode(hash.finalize())
}

impl GenerateLearningWeights {
    /// Seeded nonzero shared potentials and distinct base-120 tuples where
    /// cardinality permits. Arbitrary initialization has no semantic claim.
    pub fn seeded(
        binding: SourceActionBinding,
        lanes: usize,
        seed: u64,
        device: &Device,
    ) -> Result<Self> {
        device_admit(device)?;
        if !(1..=MAX_LANES).contains(&lanes) || !(1..=4096).contains(&binding.vocab_size()) {
            return Err(invalid("Generate seed configuration differs"));
        }
        let edges = (0..lanes / 2)
            .map(|i| LanePair {
                left: (2 * i) as u8,
                right: (2 * i + 1) as u8,
            })
            .collect::<Vec<_>>();
        let mut rng = seed ^ 0x9e3779b97f4a7c15;
        if rng == 0 {
            rng = 1;
        }
        let mut draw = || {
            rng ^= rng << 13;
            rng ^= rng >> 7;
            rng ^= rng << 17;
            (rng % 5) as i8 - 2
        };
        let mut energy =
            EnergyTables::zeroed(lanes as u8, edges).map_err(|e| invalid(e.to_string()))?;
        for l in 0..lanes {
            for r in 0..ROOT_COUNT {
                energy
                    .set_unary(l as u8, r as u8, draw())
                    .map_err(|e| invalid(e.to_string()))?;
            }
        }
        for e in 0..energy.edges().len() {
            for a in 0..ROOT_COUNT {
                for b in 0..ROOT_COUNT {
                    energy
                        .set_pair(e, a as u8, b as u8, draw())
                        .map_err(|e| invalid(e.to_string()))?;
                }
            }
        }
        let v = binding.vocab_size();
        let mut prototypes = Vec::with_capacity(v * lanes);
        for token in 0..v {
            let mut digits = token;
            for l in 0..lanes {
                let offset = ((seed as usize % ROOT_COUNT) + l * 17) % ROOT_COUNT;
                prototypes.push(((digits % ROOT_COUNT + offset) % ROOT_COUNT) as u8);
                digits /= ROOT_COUNT;
            }
        }
        let native = NativeGeometricGenerate::compile(
            &binding,
            lanes,
            &prototypes,
            &vec![0; v.div_ceil(2)],
            energy,
        )
        .map_err(|e| invalid(e.to_string()))?;
        Self::from_native(binding, &native, device)
    }

    pub fn from_native(
        binding: SourceActionBinding,
        native: &NativeGeometricGenerate,
        device: &Device,
    ) -> Result<Self> {
        device_admit(device)?;
        // Re-admit exact bytes to establish tokenizer/algebra/payload identity.
        let native = NativeGeometricGenerate::from_bytes(
            &native.to_bytes().map_err(|e| invalid(e.to_string()))?,
            &binding,
        )
        .map_err(|e| invalid(e.to_string()))?;
        let v = native.vocab_size();
        let l = native.lanes();
        let edges = native.energy().edges().to_vec();
        let mut choices = vec![0f32; v * l * ROOT_COUNT];
        for (i, &r) in native.prototypes().iter().enumerate() {
            choices[i * ROOT_COUNT + usize::from(r)] = 2.;
        }
        let mut unary = Vec::with_capacity(l * ROOT_COUNT);
        for lane in 0..l {
            for r in 0..ROOT_COUNT {
                unary.push(
                    f32::from(
                        native
                            .energy()
                            .get_unary(lane as u8, r as u8)
                            .map_err(|e| invalid(e.to_string()))?,
                    ) * 0.25,
                );
            }
        }
        let mut pair = Vec::with_capacity(edges.len() * ROOT_COUNT * ROOT_COUNT);
        for e in 0..edges.len() {
            for a in 0..ROOT_COUNT {
                for b in 0..ROOT_COUNT {
                    pair.push(
                        f32::from(
                            native
                                .energy()
                                .get_pair(e, a as u8, b as u8)
                                .map_err(|e| invalid(e.to_string()))?,
                        ) * 0.25,
                    );
                }
            }
        }
        let bias = (0..v)
            .map(|t| {
                native
                    .token_bias(t)
                    .map(|x| f32::from(x) * 0.25)
                    .map_err(|e| invalid(e.to_string()))
            })
            .collect::<Result<Vec<_>>>()?;
        Ok(Self {
            binding,
            lanes: l,
            edges,
            prototype_choices: Var::from_vec(choices, (v, l, ROOT_COUNT), device)?,
            unary: Var::from_vec(unary, (l, ROOT_COUNT), device)?,
            pair: Var::from_vec(
                pair,
                (native.energy().edges().len(), ROOT_COUNT, ROOT_COUNT),
                device,
            )?,
            bias: Var::from_vec(bias, v, device)?,
        })
    }

    pub fn device(&self) -> &Device {
        self.unary.device()
    }
    pub fn lanes(&self) -> usize {
        self.lanes
    }
    pub fn vocab_size(&self) -> usize {
        self.binding.vocab_size()
    }
    pub fn binding(&self) -> &SourceActionBinding {
        &self.binding
    }
    pub fn parameters(&self) -> BTreeMap<String, Var> {
        [
            ("generate.prototype_choices", self.prototype_choices.clone()),
            ("generate.unary", self.unary.clone()),
            ("generate.pair", self.pair.clone()),
            ("generate.bias", self.bias.clone()),
        ]
        .into_iter()
        .map(|(k, v)| (k.to_owned(), v))
        .collect()
    }
    /// Project coefficient masters only. Prototype logits are unquantized;
    /// explicit export rejects nonfinite values before native admission.
    pub fn project_shadow_range(&self) -> Result<()> {
        let updates = [&self.unary, &self.pair, &self.bias]
            .into_iter()
            .filter(|v| v.elem_count() != 0)
            .map(|v| Ok((v, q4_project_tensor(v.as_tensor())?)))
            .collect::<Result<Vec<_>>>()?;
        for (var, value) in updates {
            var.set(&value)?;
        }
        Ok(())
    }
    pub fn export_native(&self) -> Result<NativeGeometricGenerate> {
        Ok(self.export_native_with_choice_digest()?.0)
    }
    fn export_native_with_choice_digest(&self) -> Result<(NativeGeometricGenerate, String)> {
        self.validate_shapes()?;
        let choices = self.prototype_choices.flatten_all()?.to_vec1::<f32>()?;
        if choices.iter().any(|x| !x.is_finite()) {
            return Err(invalid("Generate prototype master nonfinite"));
        }
        let choice_digest = choice_digest(&choices);
        let mut prototypes = Vec::with_capacity(self.vocab_size() * self.lanes);
        for row in choices.chunks_exact(ROOT_COUNT) {
            let mut chosen = 0;
            for r in 1..ROOT_COUNT {
                if row[r] > row[chosen] {
                    chosen = r;
                }
            }
            prototypes.push(chosen as u8);
        }
        let u = quarter_codes(&self.unary)?;
        let p = quarter_codes(&self.pair)?;
        let b = quarter_codes(&self.bias)?;
        let mut energy = EnergyTables::zeroed(self.lanes as u8, self.edges.clone())
            .map_err(|e| invalid(e.to_string()))?;
        for l in 0..self.lanes {
            for r in 0..ROOT_COUNT {
                energy
                    .set_unary(l as u8, r as u8, u[l * ROOT_COUNT + r])
                    .map_err(|e| invalid(e.to_string()))?;
            }
        }
        for e in 0..self.edges.len() {
            for a in 0..ROOT_COUNT {
                for b in 0..ROOT_COUNT {
                    energy
                        .set_pair(
                            e,
                            a as u8,
                            b as u8,
                            p[(e * ROOT_COUNT + a) * ROOT_COUNT + b],
                        )
                        .map_err(|e| invalid(e.to_string()))?;
                }
            }
        }
        let native = NativeGeometricGenerate::compile(
            &self.binding,
            self.lanes,
            &prototypes,
            &packed_nibbles(&b)?,
            energy,
        )
        .map_err(|e| invalid(e.to_string()))?;
        Ok((native, choice_digest))
    }
    pub fn prepare_native(&self) -> Result<PreparedGenerateLearning> {
        let (native, prototype_choice_sha256) = self.export_native_with_choice_digest()?;
        let v = self.vocab_size();
        let mut relation = Vec::with_capacity(ROOT_COUNT * ROOT_COUNT);
        for state in 0..ROOT_COUNT {
            for prototype in 0..ROOT_COUNT {
                relation.push(u32::from(
                    native
                        .algebra()
                        .relative(state as u8, native.algebra().identity(), prototype as u8)
                        .map_err(|e| invalid(e.to_string()))?,
                ));
            }
        }
        let relative = Tensor::from_vec(relation, (ROOT_COUNT, ROOT_COUNT), self.device())?;
        let transposed = relative.t()?.contiguous()?;
        let mut prototypes = Vec::with_capacity(self.lanes);
        let mut state_relatives = Vec::with_capacity(self.lanes);
        for lane in 0..self.lanes {
            let ids = (0..v)
                .map(|token| u32::from(native.prototypes()[token * self.lanes + lane]))
                .collect::<Vec<_>>();
            let ids = Tensor::from_vec(ids, v, self.device())?;
            state_relatives.push(transposed.index_select(&ids, 0)?);
            prototypes.push(ids);
        }
        let payload_sha256 = native.metadata().payload_sha256.clone();
        Ok(PreparedGenerateLearning {
            native,
            prototype_choice_sha256,
            downloaded_master_bytes: 4
                * (self.prototype_choices.elem_count()
                    + self.unary.elem_count()
                    + self.pair.elem_count()
                    + self.bias.elem_count()),
            cache: GenerateDeviceIndexCache {
                payload_sha256,
                relative,
                index_multiplier: Tensor::new(ROOT_COUNT as u32, self.device())?,
                prototypes,
                state_relatives,
                code_probability: candle_nn::ops::softmax(self.prototype_choices.as_tensor(), 2)?,
                staged_bytes: 4 * (ROOT_COUNT * ROOT_COUNT + v * self.lanes + 1),
                device_index_bytes: 4
                    * (ROOT_COUNT * ROOT_COUNT + v * self.lanes + v * self.lanes * ROOT_COUNT + 1),
            },
        })
    }
    fn validate_shapes(&self) -> Result<()> {
        device_admit(self.device())?;
        if self.prototype_choices.dims() != [self.vocab_size(), self.lanes, ROOT_COUNT]
            || self.unary.dims() != [self.lanes, ROOT_COUNT]
            || self.pair.dims() != [self.edges.len(), ROOT_COUNT, ROOT_COUNT]
            || self.bias.dims() != [self.vocab_size()]
            || self
                .parameters()
                .values()
                .any(|v| v.dtype() != DType::F32 || !v.device().same_device(self.device()))
        {
            return Err(invalid("Generate master shape/device differs"));
        }
        Ok(())
    }

    /// No labels enter scoring or admission. CUDA snapshot freshness and the
    /// causal state graph binding are explicit caller responsibilities.
    pub fn forward_prepared(
        &self,
        prepared: &PreparedGenerateLearning,
        state: &[H4Code],
        state_logits: &Tensor,
    ) -> Result<GenerateLearningOutput> {
        self.validate_shapes()?;
        let native = &prepared.native;
        let cache = &prepared.cache;
        if native.lanes() != self.lanes
            || native.vocab_size() != self.vocab_size()
            || native.metadata().score_shift != SCORE_SHIFT
            || native.energy().edges() != self.edges
            || native.metadata().tokenizer_sha256 != self.binding.tokenizer_sha256()
            || state.len() != self.lanes
            || state_logits.dims() != [self.lanes, ROOT_COUNT]
            || state_logits.dtype() != DType::F32
            || !state_logits.device().same_device(self.device())
            || !cache.relative.device().same_device(self.device())
            || native.metadata().payload_sha256 != cache.payload_sha256
        {
            return Err(invalid(
                "Generate cached prepared state/binding/device differs",
            ));
        }
        if self.device().is_cpu() {
            if choice_digest(&self.prototype_choices.flatten_all()?.to_vec1::<f32>()?)
                != prepared.prototype_choice_sha256
            {
                return Err(invalid("Generate cached choice probabilities are stale"));
            }
            if self
                .export_native()?
                .to_bytes()
                .map_err(|e| invalid(e.to_string()))?
                != native.to_bytes().map_err(|e| invalid(e.to_string()))?
            {
                return Err(invalid("Generate prepared payload is stale"));
            }
            if state_logits
                .flatten_all()?
                .to_vec1::<f32>()?
                .iter()
                .any(|x| !x.is_finite())
            {
                return Err(invalid("Generate state choices nonfinite"));
            }
        }
        let v = self.vocab_size();
        let mut counts = GenerateReadCounts::default();
        let mut hard = vec![0i64; v];
        native
            .score_into(state, &mut hard, &mut counts)
            .map_err(|e| invalid(e.to_string()))?;
        let u = (q4_shadow_ste(self.unary.as_tensor())? * QUARTER_TO_NATS)?;
        let p = if self.edges.is_empty() {
            self.pair.as_tensor().clone()
        } else {
            (q4_shadow_ste(self.pair.as_tensor())? * QUARTER_TO_NATS)?
        };
        let mut coefficient = (q4_shadow_ste(self.bias.as_tensor())? * QUARTER_TO_NATS)?;
        let mut factual_relatives = Vec::with_capacity(self.lanes);
        let mut code_relatives = Vec::with_capacity(self.lanes);
        let mut device_index_elements = 0usize;
        for lane in 0..self.lanes {
            let row = cache
                .relative
                .narrow(0, usize::from(state[lane].index()), 1)?
                .reshape(ROOT_COUNT)?;
            let factual = row.index_select(&cache.prototypes[lane], 0)?;
            coefficient = (&coefficient
                + u.narrow(0, lane, 1)?
                    .reshape(ROOT_COUNT)?
                    .index_select(&factual, 0)?)?;
            device_index_elements += v;
            factual_relatives.push(factual);
            code_relatives.push(row);
        }
        for (e, edge) in self.edges.iter().enumerate() {
            let ids = pair_device_indices(
                &factual_relatives[usize::from(edge.left)],
                &factual_relatives[usize::from(edge.right)],
                &cache.index_multiplier,
            )?;
            coefficient = (&coefficient
                + p.narrow(0, e, 1)?
                    .reshape(ROOT_COUNT * ROOT_COUNT)?
                    .index_select(&ids, 0)?)?;
            device_index_elements += v;
        }
        let state_probability = candle_nn::ops::softmax(state_logits, 1)?;
        let ud = u.detach();
        let pd = p.detach();
        let mut input = Tensor::zeros(v, DType::F32, self.device())?;
        for lane in 0..self.lanes {
            let sr = &cache.state_relatives[lane];
            let cr = code_relatives[lane].reshape((1, ROOT_COUNT))?;
            let unary = ud.narrow(0, lane, 1)?.reshape(ROOT_COUNT)?;
            let mut su = unary
                .index_select(&sr.flatten_all()?, 0)?
                .reshape((v, ROOT_COUNT))?;
            // Unary code utility is shared across every token: no V-fold copy
            // of its index map or utility is needed until a pair term varies.
            let mut cu = unary
                .index_select(&cr.flatten_all()?, 0)?
                .reshape((1, ROOT_COUNT))?;
            for (e, edge) in self.edges.iter().enumerate() {
                let left = usize::from(edge.left);
                let right = usize::from(edge.right);
                if lane != left && lane != right {
                    continue;
                }
                let other = if lane == left { right } else { left };
                let other = factual_relatives[other].reshape((v, 1))?;
                let (sa, sb, ca, cb) = if lane == left {
                    (sr, &other, &cr, &other)
                } else {
                    (&other, sr, &other, &cr)
                };
                let si = pair_device_indices(sa, sb, &cache.index_multiplier)?.flatten_all()?;
                let ci = pair_device_indices(ca, cb, &cache.index_multiplier)?.flatten_all()?;
                let table = pd.narrow(0, e, 1)?.reshape(ROOT_COUNT * ROOT_COUNT)?;
                su = (&su + table.index_select(&si, 0)?.reshape((v, ROOT_COUNT))?)?;
                cu = cu.broadcast_add(&table.index_select(&ci, 0)?.reshape((v, ROOT_COUNT))?)?;
                device_index_elements += 2 * v * ROOT_COUNT;
            }
            let sp = state_probability.narrow(0, lane, 1)?;
            let cp = cache
                .code_probability
                .narrow(1, lane, 1)?
                .reshape((v, ROOT_COUNT))?;
            input = (&input + su.broadcast_mul(&sp)?.sum(1)?)?;
            input = (&input + cu.broadcast_mul(&cp)?.sum(1)?)?;
        }
        let anchor = Tensor::from_vec(
            hard.iter()
                .map(|&x| (x as f64 / Q24) as f32)
                .collect::<Vec<_>>(),
            v,
            self.device(),
        )?;
        let raw_scores =
            ((&anchor + (&coefficient - coefficient.detach())?)? + (&input - input.detach())?)?;
        let clipped_scores = raw_scores.clamp(-8f32, 8f32)?;
        Ok(GenerateLearningOutput {
            raw_scores,
            clipped_scores,
            scores_q24: hard,
            native_counts: counts,
            costs: GenerateLearningCosts {
                vocabulary_rows: v,
                lanes: self.lanes,
                ordered_pairs: self.edges.len(),
                conditional_choice_rows: 2 * v * self.lanes,
                staged_index_bytes: 0,
                staged_hard_score_bytes: 4 * v,
                export_master_download_bytes: prepared.downloaded_master_bytes,
                max_conditional_utility_elements: v * ROOT_COUNT,
                loss_status_scalars: 2,
                snapshot_staged_index_bytes: cache.staged_bytes,
                snapshot_device_index_bytes: cache.device_index_bytes,
                per_position_device_index_elements: device_index_elements,
                snapshot_choice_probability_bytes: 4 * v * self.lanes * ROOT_COUNT,
                generate_only_loss_staged_bytes_upper: 16 * v + 4,
            },
        })
    }

    /// Deliberately uncached CPU reference for exact index/gradient parity.
    pub fn forward_prepared_uncached_reference(
        &self,
        prepared: &PreparedGenerateLearning,
        state: &[H4Code],
        state_logits: &Tensor,
    ) -> Result<GenerateLearningOutput> {
        if !self.device().is_cpu() {
            return Err(invalid("uncached Generate path is CPU reference only"));
        }
        self.validate_shapes()?;
        let native = &prepared.native;
        if native.lanes() != self.lanes
            || native.vocab_size() != self.vocab_size()
            || native.metadata().score_shift != SCORE_SHIFT
            || native.energy().edges() != self.edges
            || native.metadata().tokenizer_sha256 != self.binding.tokenizer_sha256()
            || state.len() != self.lanes
            || state_logits.dims() != [self.lanes, ROOT_COUNT]
            || state_logits.dtype() != DType::F32
            || !state_logits.device().same_device(self.device())
        {
            return Err(invalid("Generate prepared state/binding/shape differs"));
        }
        if self.device().is_cpu() {
            if self
                .export_native()?
                .to_bytes()
                .map_err(|e| invalid(e.to_string()))?
                != native.to_bytes().map_err(|e| invalid(e.to_string()))?
            {
                return Err(invalid("Generate prepared payload is stale"));
            }
            if state_logits
                .flatten_all()?
                .to_vec1::<f32>()?
                .iter()
                .any(|x| !x.is_finite())
            {
                return Err(invalid("Generate state choices nonfinite"));
            }
        }
        let v = self.vocab_size();
        let mut counts = GenerateReadCounts::default();
        let mut hard = vec![0i64; v];
        native
            .score_into(state, &mut hard, &mut counts)
            .map_err(|e| invalid(e.to_string()))?;
        let u = (q4_shadow_ste(self.unary.as_tensor())? * QUARTER_TO_NATS)?.flatten_all()?;
        let p = if self.edges.is_empty() {
            self.pair.as_tensor().flatten_all()?
        } else {
            (q4_shadow_ste(self.pair.as_tensor())? * QUARTER_TO_NATS)?.flatten_all()?
        };
        let b = (q4_shadow_ste(self.bias.as_tensor())? * QUARTER_TO_NATS)?;
        let mut indices_bytes = 0;
        let mut relative = vec![0u8; v * self.lanes];
        for token in 0..v {
            for lane in 0..self.lanes {
                relative[token * self.lanes + lane] = native
                    .algebra()
                    .relative(
                        state[lane].index(),
                        native.algebra().identity(),
                        native.prototypes()[token * self.lanes + lane],
                    )
                    .map_err(|e| invalid(e.to_string()))?;
            }
        }
        // Selected hard coefficient gradient only. Input utility factors below
        // are detached so alternative choices cannot double-count coefficients.
        let mut coefficient = b;
        for lane in 0..self.lanes {
            let ids = (0..v)
                .map(|t| (lane * ROOT_COUNT + usize::from(relative[t * self.lanes + lane])) as u32)
                .collect::<Vec<_>>();
            coefficient = (&coefficient + gather(&u, ids, self.device(), &mut indices_bytes)?)?;
        }
        for (e, edge) in self.edges.iter().enumerate() {
            let ids = (0..v)
                .map(|t| {
                    ((e * ROOT_COUNT
                        + usize::from(relative[t * self.lanes + usize::from(edge.left)]))
                        * ROOT_COUNT
                        + usize::from(relative[t * self.lanes + usize::from(edge.right)]))
                        as u32
                })
                .collect::<Vec<_>>();
            coefficient = (&coefficient + gather(&p, ids, self.device(), &mut indices_bytes)?)?;
        }
        let ud = u.detach();
        let pd = p.detach();
        let state_probability = candle_nn::ops::softmax(state_logits, 1)?;
        let code_probability = candle_nn::ops::softmax(self.prototype_choices.as_tensor(), 2)?;
        let mut input = Tensor::zeros(v, DType::F32, self.device())?;
        for lane in 0..self.lanes {
            // Each utility is V x120, not a four-dimensional projection. Only
            // terms touching this lane vary; common terms cancel in pullback.
            let mut state_ids = Vec::with_capacity(v * ROOT_COUNT);
            let mut code_ids = Vec::with_capacity(v * ROOT_COUNT);
            let mut sr = Vec::with_capacity(v * ROOT_COUNT);
            let mut cr = Vec::with_capacity(v * ROOT_COUNT);
            for token in 0..v {
                for r in 0..ROOT_COUNT {
                    let srel = native
                        .algebra()
                        .relative(
                            r as u8,
                            native.algebra().identity(),
                            native.prototypes()[token * self.lanes + lane],
                        )
                        .map_err(|e| invalid(e.to_string()))?;
                    let crel = native
                        .algebra()
                        .relative(state[lane].index(), native.algebra().identity(), r as u8)
                        .map_err(|e| invalid(e.to_string()))?;
                    state_ids.push((lane * ROOT_COUNT + usize::from(srel)) as u32);
                    code_ids.push((lane * ROOT_COUNT + usize::from(crel)) as u32);
                    sr.push(srel);
                    cr.push(crel);
                }
            }
            let mut su = gather(&ud, state_ids, self.device(), &mut indices_bytes)?
                .reshape((v, ROOT_COUNT))?;
            let mut cu = gather(&ud, code_ids, self.device(), &mut indices_bytes)?
                .reshape((v, ROOT_COUNT))?;
            for (e, edge) in self.edges.iter().enumerate() {
                let left = usize::from(edge.left);
                let right = usize::from(edge.right);
                if lane != left && lane != right {
                    continue;
                }
                let map = |alternatives: &[u8]| {
                    (0..v * ROOT_COUNT)
                        .map(|i| {
                            let t = i / ROOT_COUNT;
                            let a = if lane == left {
                                alternatives[i]
                            } else {
                                relative[t * self.lanes + left]
                            };
                            let b = if lane == right {
                                alternatives[i]
                            } else {
                                relative[t * self.lanes + right]
                            };
                            ((e * ROOT_COUNT + usize::from(a)) * ROOT_COUNT + usize::from(b)) as u32
                        })
                        .collect::<Vec<_>>()
                };
                su = (&su
                    + gather(&pd, map(&sr), self.device(), &mut indices_bytes)?
                        .reshape((v, ROOT_COUNT))?)?;
                cu = (&cu
                    + gather(&pd, map(&cr), self.device(), &mut indices_bytes)?
                        .reshape((v, ROOT_COUNT))?)?;
            }
            let sp = state_probability.narrow(0, lane, 1)?;
            let cp = code_probability
                .narrow(1, lane, 1)?
                .reshape((v, ROOT_COUNT))?;
            input = (&input + su.broadcast_mul(&sp)?.sum(1)?)?;
            input = (&input + (cu * cp)?.sum(1)?)?;
        }
        let anchor = Tensor::from_vec(
            hard.iter()
                .map(|&x| (x as f64 / Q24) as f32)
                .collect::<Vec<_>>(),
            v,
            self.device(),
        )?;
        let coefficient_delta = (&coefficient - coefficient.detach())?;
        let input_delta = (&input - input.detach())?;
        let raw_scores = ((&anchor + coefficient_delta)? + input_delta)?;
        let clipped_scores = raw_scores.clamp(-8f32, 8f32)?;
        Ok(GenerateLearningOutput {
            raw_scores,
            clipped_scores,
            scores_q24: hard,
            native_counts: counts,
            costs: GenerateLearningCosts {
                vocabulary_rows: v,
                lanes: self.lanes,
                ordered_pairs: self.edges.len(),
                conditional_choice_rows: v * self.lanes * 2,
                staged_index_bytes: indices_bytes,
                staged_hard_score_bytes: v * 4,
                export_master_download_bytes: prepared.downloaded_master_bytes,
                max_conditional_utility_elements: v * ROOT_COUNT,
                loss_status_scalars: 2,
                snapshot_staged_index_bytes: prepared.cache.staged_bytes,
                snapshot_device_index_bytes: prepared.cache.device_index_bytes,
                per_position_device_index_elements: 0,
                snapshot_choice_probability_bytes: 4 * v * self.lanes * ROOT_COUNT,
                generate_only_loss_staged_bytes_upper: 16 * v + 4,
            },
        })
    }
}

// Offline device U32 arithmetic only. All inputs are exact group IDs <120,
// hence resulting flat indices are <14400 and overflow cannot occur. Pair
// orientation stays declared left then right. No float index approximation.
fn pair_device_indices(left: &Tensor, right: &Tensor, multiplier: &Tensor) -> Result<Tensor> {
    Ok(left.broadcast_mul(multiplier)?.broadcast_add(right)?)
}

fn gather(table: &Tensor, ids: Vec<u32>, device: &Device, bytes: &mut usize) -> Result<Tensor> {
    *bytes = bytes
        .checked_add(ids.len() * 4)
        .ok_or_else(|| invalid("Generate staged index size overflow"))?;
    let n = ids.len();
    Ok(table.index_select(&Tensor::from_vec(ids, n, device)?, 0)?)
}

/// Flat action marginal with labels used only after a target-free native pool.
/// Native token mass is aggregated before the sole float probability boundary.
/// Copy scores may be frozen or carry separately implemented input credit.
/// No separate branch gate or independent Generate normalization is introduced.
pub fn vocabulary_marginal_loss(
    trace: &VocabularyActionTrace,
    generate_raw: &Tensor,
    copy_raw: Option<&Tensor>,
    target: u32,
) -> Result<Tensor> {
    device_admit(generate_raw.device())?;
    if generate_raw.rank() != 1
        || generate_raw.dtype() != DType::F32
        || trace.policy != uor_r4_integer::geometric_vocabulary_actions::POLICY
    {
        return Err(invalid("Generate loss full score shape differs"));
    }
    let mut generate_indices = Vec::new();
    let mut copy_indices = Vec::new();
    let mut seen_copy = false;
    let mut summed_mass = 0u64;
    let mut target_action_mass = 0u64;
    for (offset, action) in trace.actions.iter().enumerate() {
        if action.action_offset != offset
            || action.score_q24 != action.raw_score_q24.clamp(-(8 << 24), 8 << 24)
            || action.weight_q31 == 0
        {
            return Err(invalid("Generate loss native action trace differs"));
        }
        summed_mass = summed_mass
            .checked_add(action.weight_q31)
            .ok_or_else(|| invalid("Generate loss action mass overflow"))?;
        if action.token_id == target {
            target_action_mass = target_action_mass
                .checked_add(action.weight_q31)
                .ok_or_else(|| invalid("Generate loss alias mass overflow"))?;
        }
        match action.action {
            VocabularyAction::Generate { token_id } => {
                if seen_copy
                    || token_id != action.token_id
                    || token_id as usize >= generate_raw.elem_count()
                    || generate_indices
                        .last()
                        .is_some_and(|previous| *previous >= token_id)
                {
                    return Err(invalid("Generate loss action order/identity differs"));
                }
                generate_indices.push(token_id);
            }
            VocabularyAction::Copy { source_offset } => {
                seen_copy = true;
                if source_offset != copy_indices.len() {
                    return Err(invalid("Generate loss Copy occurrence order differs"));
                }
                copy_indices.push(source_offset as u32);
            }
        }
    }
    if generate_indices.is_empty() || summed_mass != trace.summary.total_weight_q31 {
        return Err(invalid(
            "Generate loss empty admission or denominator differs",
        ));
    }
    let generate = generate_raw.index_select(
        &Tensor::from_vec(
            generate_indices.clone(),
            generate_indices.len(),
            generate_raw.device(),
        )?,
        0,
    )?;
    let combined = if copy_indices.is_empty() {
        if copy_raw.is_some_and(|c| c.elem_count() != 0) {
            return Err(invalid("Generate loss unexpected Copy scores"));
        }
        generate
    } else {
        let copy = copy_raw.ok_or_else(|| invalid("Generate loss missing Copy scores"))?;
        if copy.dims() != [copy_indices.len()]
            || copy.dtype() != DType::F32
            || !copy.device().same_device(generate_raw.device())
        {
            return Err(invalid("Generate loss Copy shape/device differs"));
        }
        Tensor::cat(&[&generate, copy], 0)?
    };
    let native_raw = Tensor::from_vec(
        trace
            .actions
            .iter()
            .map(|a| (a.raw_score_q24 as f64 / Q24) as f32)
            .collect::<Vec<_>>(),
        trace.actions.len(),
        generate_raw.device(),
    )?;
    // Bind this loss to the supplied native pool at the declared f32 boundary.
    // This scalar checks every raw action before the hard anchor could conceal
    // an unrelated trace. Exact integer provenance remains the caller's role.
    if (&combined - &native_raw)?
        .abs()?
        .max(0)?
        .to_scalar::<f32>()?
        != 0.
    {
        return Err(invalid(
            "Generate loss raw score graph differs from native pool",
        ));
    }
    let bounded = combined.clamp(-8f32, 8f32)?;
    // A single reduced status scalar is permitted and counted; never download
    // the live CUDA score vector or its adjoints for validation.
    if !combined.sqr()?.sum_all()?.to_scalar::<f32>()?.is_finite() {
        return Err(invalid("Generate marginal score status is nonfinite"));
    }
    let mass = trace
        .token_masses
        .iter()
        .find(|m| m.token_id == target)
        .ok_or_else(|| invalid("Generate target is not admitted by full legal pool"))?
        .weight_q31;
    let denominator = trace.summary.total_weight_q31;
    if mass == 0 || denominator == 0 || mass > denominator || mass != target_action_mass {
        return Err(invalid("Generate native target mass is invalid"));
    }
    let native_probability = (mass as f64 / denominator as f64) as f32;
    if !native_probability.is_finite() || native_probability <= 0. {
        return Err(invalid("Generate native probability cannot be represented"));
    }
    let hard_scores = Tensor::from_vec(
        trace
            .actions
            .iter()
            .map(|a| (a.score_q24 as f64 / Q24) as f32)
            .collect::<Vec<_>>(),
        trace.actions.len(),
        generate_raw.device(),
    )?;
    let scores = (&hard_scores + (&bounded - bounded.detach())?)?;
    let probability = candle_nn::ops::softmax(&scores, 0)?;
    let mask = Tensor::from_vec(
        trace
            .actions
            .iter()
            .map(|a| if a.token_id == target { 1f32 } else { 0f32 })
            .collect::<Vec<_>>(),
        trace.actions.len(),
        generate_raw.device(),
    )?;
    let soft_target = (probability * mask)?.sum_all()?;
    let anchored = (Tensor::new(native_probability, generate_raw.device())?
        + (&soft_target - soft_target.detach())?)?;
    Ok(anchored.log()?.neg()?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use uor_r4_integer::geometric_no_read::CANONICAL_BASIS_Q25;
    const TOK: &str = r#"{"pre_tokenizer":{"type":"ByteLevel","add_prefix_space":false},"model":{"type":"BPE","vocab":{"<|bos|>":0,"<|eos|>":1,"<|unk|>":2,".":3,"a":4,"b":5,"Ġ":6,"Ġa":7},"merges":["Ġ a"]},"added_tokens":[{"id":0,"content":"<|bos|>"},{"id":1,"content":"<|eos|>"},{"id":2,"content":"<|unk|>"}]}"#;
    fn binding() -> Result<SourceActionBinding> {
        SourceActionBinding::new(TOK.as_bytes()).map_err(|e| invalid(e.to_string()))
    }
    fn code(x: u8) -> Result<H4Code> {
        H4Code::try_from(x).map_err(|e| invalid(e.to_string()))
    }
    fn grad(g: &candle_core::backprop::GradStore, t: &Tensor) -> Result<Vec<f32>> {
        Ok(g.get(t)
            .ok_or_else(|| invalid("Generate test missing gradient"))?
            .flatten_all()?
            .to_vec1::<f32>()?)
    }
    #[test]
    fn generate_learning_hard_forward_export_and_selected_coefficient_credit() -> Result<()> {
        for seed in [1, 2, 3] {
            let w = GenerateLearningWeights::seeded(binding()?, 2, seed, &Device::Cpu)?;
            let snapshot = w.prepare_native()?;
            let states = [code(4)?, code(5)?];
            let z = Var::zeros((2, ROOT_COUNT), DType::F32, &Device::Cpu)?;
            let out = w.forward_prepared(&snapshot, &states, z.as_tensor())?;
            assert_eq!(
                out.raw_scores.to_vec1::<f32>()?,
                out.scores_q24
                    .iter()
                    .map(|&s| (s as f64 / Q24) as f32)
                    .collect::<Vec<_>>()
            );
            let g = out.raw_scores.sum_all()?.backward()?;
            // Counterfactual paths detach factors: bias receives exactly one
            // selected coefficient derivative per vocabulary row, not twice.
            assert!(grad(&g, w.bias.as_tensor())?.iter().all(|&x| x == 0.25));
            assert!(grad(&g, z.as_tensor())?.iter().any(|x| x.abs() > 1e-6));
            assert!(grad(&g, w.prototype_choices.as_tensor())?
                .iter()
                .any(|x| x.abs() > 1e-6));
            let reload = NativeGeometricGenerate::from_bytes(
                &snapshot
                    .native
                    .to_bytes()
                    .map_err(|e| invalid(e.to_string()))?,
                w.binding(),
            )
            .map_err(|e| invalid(e.to_string()))?;
            let restored = GenerateLearningWeights::from_native(binding()?, &reload, &Device::Cpu)?;
            assert_eq!(
                restored
                    .export_native()?
                    .to_bytes()
                    .map_err(|e| invalid(e.to_string()))?,
                snapshot
                    .native
                    .to_bytes()
                    .map_err(|e| invalid(e.to_string()))?
            );
        }
        Ok(())
    }
    #[test]
    fn generate_learning_cached_u32_indices_match_uncached_scores_and_all_gradients() -> Result<()>
    {
        let w = GenerateLearningWeights::seeded(binding()?, 4, 3, &Device::Cpu)?;
        let prepared = w.prepare_native()?;
        let z = Var::zeros((4, ROOT_COUNT), DType::F32, &Device::Cpu)?;
        let states = [code(4)?, code(5)?, code(8)?, code(13)?];
        let cached = w.forward_prepared(&prepared, &states, z.as_tensor())?;
        let reference = w.forward_prepared_uncached_reference(&prepared, &states, z.as_tensor())?;
        assert_eq!(
            cached.raw_scores.to_vec1::<f32>()?,
            reference.raw_scores.to_vec1::<f32>()?
        );
        assert_eq!(cached.costs.staged_index_bytes, 0);
        assert!(reference.costs.staged_index_bytes > 0);
        let a = cached.raw_scores.sqr()?.sum_all()?.backward()?;
        let b = reference.raw_scores.sqr()?.sum_all()?.backward()?;
        let mut parameters = w.parameters();
        parameters.insert("state".into(), z);
        for (name, var) in parameters {
            for (x, y) in grad(&a, var.as_tensor())?
                .into_iter()
                .zip(grad(&b, var.as_tensor())?)
            {
                assert!((x - y).abs() < 1e-6, "{name}: {x} {y}");
            }
        }
        // Repeated positions reuse the same snapshot and choice graph; code
        // probabilities must not be carried beyond an optimizer update.
        let changed = [code(13)?, code(8)?, code(5)?, code(4)?];
        let next = w.forward_prepared(
            &prepared,
            &changed,
            &Tensor::zeros((4, ROOT_COUNT), DType::F32, &Device::Cpu)?,
        )?;
        assert_eq!(next.costs.staged_index_bytes, 0);
        assert_ne!(cached.scores_q24, next.scores_q24);
        // Logits may change without moving native argmax, still invalidating
        // the cached normalized choice differential.
        let before = w
            .export_native()?
            .to_bytes()
            .map_err(|e| invalid(e.to_string()))?;
        w.prototype_choices
            .set(&(w.prototype_choices.as_tensor() * 1.5)?)?;
        assert_eq!(
            before,
            w.export_native()?
                .to_bytes()
                .map_err(|e| invalid(e.to_string()))?
        );
        assert!(w
            .forward_prepared(
                &prepared,
                &states,
                &Tensor::zeros((4, ROOT_COUNT), DType::F32, &Device::Cpu)?
            )
            .is_err());
        Ok(())
    }
    #[test]
    fn generate_learning_full120_preserves_even_harmonic_state_credit() -> Result<()> {
        let b = binding()?;
        let mut energy = EnergyTables::zeroed(1, vec![]).map_err(|e| invalid(e.to_string()))?;
        for (r, axes) in CANONICAL_BASIS_Q25.iter().enumerate() {
            let value = if axes[0].unsigned_abs() > 16_777_216 {
                1
            } else {
                -1
            };
            energy
                .set_unary(0, r as u8, value)
                .map_err(|e| invalid(e.to_string()))?;
        }
        let n = NativeGeometricGenerate::compile(
            &b,
            1,
            &vec![1; b.vocab_size()],
            &vec![0; b.vocab_size().div_ceil(2)],
            energy,
        )
        .map_err(|e| invalid(e.to_string()))?;
        let w = GenerateLearningWeights::from_native(b, &n, &Device::Cpu)?;
        let prepared = w.prepare_native()?;
        let z = Var::zeros((1, ROOT_COUNT), DType::F32, &Device::Cpu)?;
        let out = w.forward_prepared(&prepared, &[code(1)?], z.as_tensor())?;
        let g = out.raw_scores.sum_all()?.backward()?;
        let dz = grad(&g, z.as_tensor())?;
        assert!(dz.iter().any(|x| x.abs() > 1e-4));
        // Even antipodal utility has zero four-coordinate projection while
        // retaining a nonzero full120 differential.
        for axis in 0..4 {
            let moment = dz
                .iter()
                .zip(CANONICAL_BASIS_Q25.iter())
                .map(|(&x, b)| f64::from(x) * f64::from(b[axis]) / 33_554_432.)
                .sum::<f64>();
            assert!(moment.abs() < 1e-6, "axis {axis}: {moment}");
        }
        let mut counter = [0i64; ROOT_COUNT];
        n.state_conditional_scores_into(
            &[code(1)?],
            0,
            0,
            &mut counter,
            &mut GenerateReadCounts::default(),
        )
        .map_err(|e| invalid(e.to_string()))?;
        let utilities = counter.map(|x| x as f64 / Q24);
        let mean = utilities.iter().sum::<f64>() / ROOT_COUNT as f64;
        for (actual, u) in dz.iter().zip(utilities) {
            let expected = (u - mean) * w.vocab_size() as f64 / ROOT_COUNT as f64;
            assert!((f64::from(*actual) - expected).abs() < 1e-6);
        }
        Ok(())
    }
    #[test]
    fn generate_learning_zero_tables_disconnect_choices_and_stale_snapshot_rejects() -> Result<()> {
        let b = binding()?;
        let n = NativeGeometricGenerate::compile(
            &b,
            2,
            &vec![1; b.vocab_size() * 2],
            &vec![0; b.vocab_size().div_ceil(2)],
            EnergyTables::zeroed(2, vec![LanePair { left: 0, right: 1 }])
                .map_err(|e| invalid(e.to_string()))?,
        )
        .map_err(|e| invalid(e.to_string()))?;
        let w = GenerateLearningWeights::from_native(b, &n, &Device::Cpu)?;
        let p = w.prepare_native()?;
        let z = Var::zeros((2, ROOT_COUNT), DType::F32, &Device::Cpu)?;
        let out = w.forward_prepared(&p, &[code(1)?, code(1)?], z.as_tensor())?;
        let g = out.raw_scores.sum_all()?.backward()?;
        assert!(grad(&g, z.as_tensor())?.iter().all(|x| *x == 0.));
        assert!(grad(&g, w.prototype_choices.as_tensor())?
            .iter()
            .all(|x| *x == 0.));
        w.bias.set(&Tensor::from_vec(
            vec![0.25f32; w.vocab_size()],
            w.vocab_size(),
            &Device::Cpu,
        )?)?;
        assert!(w
            .forward_prepared(&p, &[code(1)?, code(1)?], z.as_tensor())
            .is_err());
        w.bias.set(&Tensor::from_vec(
            vec![f32::NAN; w.vocab_size()],
            w.vocab_size(),
            &Device::Cpu,
        )?)?;
        assert!(w.export_native().is_err());
        Ok(())
    }
    #[test]
    fn generate_learning_loss_binds_raw_scores_and_clip_has_zero_saturated_credit() -> Result<()> {
        use uor_r4_integer::geometric_vocabulary_actions::{
            VocabularyActionMass, VocabularyReduction, VocabularyTokenMass,
        };
        let b = binding()?;
        let v = b.vocab_size();
        let one = 1u64 << 31;
        // Exact canonical-exp equal-zero-score case: every unnormalized weight
        // is 2^31, so this fixture needs no authored probability approximation.
        let actions = (0..v)
            .map(|i| VocabularyActionMass {
                action: VocabularyAction::Generate { token_id: i as u32 },
                action_offset: i,
                token_id: i as u32,
                raw_score_q24: 0,
                score_q24: 0,
                weight_q31: one,
            })
            .collect();
        let masses = (0..v)
            .map(|i| VocabularyTokenMass {
                token_id: i as u32,
                weight_q31: one,
                generate_weight_q31: one,
                copy_weight_q31: 0,
            })
            .collect();
        let trace = VocabularyActionTrace {
            policy: uor_r4_integer::geometric_vocabulary_actions::POLICY,
            tokenizer_sha256: b.tokenizer_sha256().into(),
            period_token_id: b.period_token_id(),
            eos_token_id: b.eos_token_id(),
            actions,
            token_masses: masses,
            summary: VocabularyReduction {
                legal_generate_actions: v,
                copy_actions: 0,
                max_score_q24: 0,
                total_weight_q31: one * v as u64,
                chosen_token_id: 0,
                chosen_weight_q31: one,
                generate_weight_q31: one * v as u64,
                copy_weight_q31: 0,
                chosen_generate_weight_q31: one,
                chosen_copy_weight_q31: 0,
                clipped_low_actions: 0,
                clipped_high_actions: 0,
                raw_max_score_q24: 0,
                raw_total_weight_q31: one * v as u64,
                raw_chosen_token_id: 0,
                raw_chosen_weight_q31: one,
                chosen_raw_mass_rank: 1,
                raw_chosen_clipped_mass_rank: 1,
                token_winner_changed_by_clip: false,
            },
        };
        let scores = Var::zeros(v, DType::F32, &Device::Cpu)?;
        let loss = vocabulary_marginal_loss(&trace, scores.as_tensor(), None, 4)?;
        assert!((loss.to_scalar::<f32>()? - (v as f32).ln()).abs() < 1e-6);
        assert!(grad(&loss.backward()?, scores.as_tensor())?
            .iter()
            .any(|x| x.abs() > 1e-6));
        let mut wrong = vec![0f32; v];
        wrong[4] = 0.0625;
        assert!(vocabulary_marginal_loss(
            &trace,
            &Tensor::from_vec(wrong, v, &Device::Cpu)?,
            None,
            4
        )
        .is_err());
        let clip = Var::from_vec(vec![-9f32, 0., 9.], 3, &Device::Cpu)?;
        let g = clip.clamp(-8f32, 8f32)?.sum_all()?.backward()?;
        assert_eq!(grad(&g, clip.as_tensor())?, vec![0., 1., 0.]);
        Ok(())
    }
    #[cfg(feature = "cuda")]
    #[test]
    fn generate_learning_cuda_explicit_hard_and_full_choice_gradient_parity() -> Result<()> {
        // Explicit CUDA test: no missing-device skip can be mistaken for PASS.
        let gpu = Device::new_cuda(0)?;
        let cpu = GenerateLearningWeights::seeded(binding()?, 2, 7, &Device::Cpu)?;
        let native = cpu.export_native()?;
        let cuda = GenerateLearningWeights::from_native(binding()?, &native, &gpu)?;
        let states = [code(4)?, code(5)?];
        let zc = Var::zeros((2, ROOT_COUNT), DType::F32, &Device::Cpu)?;
        let zg = Var::zeros((2, ROOT_COUNT), DType::F32, &gpu)?;
        let c = cpu.forward_prepared(&cpu.prepare_native()?, &states, zc.as_tensor())?;
        let d = cuda.forward_prepared(&cuda.prepare_native()?, &states, zg.as_tensor())?;
        assert_eq!(
            c.raw_scores.to_vec1::<f32>()?,
            d.raw_scores.to_vec1::<f32>()?
        );
        let cg = c.raw_scores.sqr()?.sum_all()?.backward()?;
        let dg = d.raw_scores.sqr()?.sum_all()?.backward()?;
        let mut cp = cpu.parameters();
        let mut dp = cuda.parameters();
        cp.insert("state".into(), zc);
        dp.insert("state".into(), zg);
        for (name, a) in cp {
            let b = dp
                .get(&name)
                .ok_or_else(|| invalid("CUDA test parameter name missing"))?;
            for (x, y) in grad(&cg, a.as_tensor())?
                .into_iter()
                .zip(grad(&dg, b.as_tensor())?)
            {
                assert!((x - y).abs() <= 2e-4 + 2e-4 * x.abs(), "{name}: {x} {y}");
            }
        }
        Ok(())
    }
}
