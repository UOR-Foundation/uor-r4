//! Exact offline CUDA counterpart of the admitted integer alias reducer.
//! Device carriers are privately constructed; no caller-supplied masses or
//! probability anchor are admitted. Portable serving remains unchanged.
use crate::{
    invalid,
    native_geometric_cuda_kernels::{launch, Arg},
    Result,
};
use candle_core::{op::BackpropOp, CudaStorage, DType, Device, Storage, Tensor};
use std::sync::Arc;
use uor_r4_integer::{
    geometric_source_actions::SourceActionBinding,
    geometric_vocabulary_actions::{
        NativeVocabularyActions, VocabularyAction, VocabularyActionMass, VocabularyActionTrace,
        VocabularyReduction, VocabularyTokenMass, POLICY,
    },
};

fn storage(s: &Storage) -> Result<&CudaStorage> {
    match s {
        Storage::Cuda(c) => Ok(c),
        _ => Err(invalid("alias reducer requires CUDA storage")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometric_generate_learning::{
        vocabulary_marginal_loss_device_with_credit, vocabulary_marginal_loss_with_credit,
        VocabularyScoreAdjoint,
    };
    use candle_core::Var;
    fn cuda_required() -> Result<Device> {
        if std::env::var("UOR_R4_CUDA_REQUIRED").as_deref() != Ok("1") {
            return Err(invalid(
                "set UOR_R4_CUDA_REQUIRED=1 for explicit CUDA checks",
            ));
        }
        Ok(Device::new_cuda(0)?)
    }
    fn binding(vocab: usize, sparse: bool) -> Result<SourceActionBinding> {
        let mut map = serde_json::Map::new();
        let mut added = vec![
            serde_json::json!({"id":0,"content":"<|bos|>"}),
            serde_json::json!({"id":1,"content":"<|eos|>"}),
            serde_json::json!({"id":2,"content":"<|unk|>"}),
        ];
        for id in 0..vocab {
            if sparse && id == 7 {
                continue;
            }
            let name = match id {
                0 => "<|bos|>".into(),
                1 => "<|eos|>".into(),
                2 => "<|unk|>".into(),
                3 => ".".into(),
                _ => format!("x{id}"),
            };
            if sparse && id > 7 {
                added.push(serde_json::json!({"id":id,"content":name}));
            } else {
                map.insert(name, serde_json::json!(id));
            }
        }
        let tok = serde_json::to_vec(
            &serde_json::json!({"pre_tokenizer":{"type":"ByteLevel","add_prefix_space":false},
            "model":{"type":"BPE","vocab":map,"merges":[]},"added_tokens":added}),
        )?;
        SourceActionBinding::new(&tok).map_err(|e| invalid(e.to_string()))
    }
    fn exp() -> Vec<u8> {
        (0..uor_r4_integer::geometric_read::EXP_TABLE_LEN)
            .flat_map(|i| {
                (((-(i as f64) / 256.).exp() * (1u64 << 31) as f64).round() as u32).to_le_bytes()
            })
            .collect()
    }
    #[test]
    #[ignore = "explicit CUDA required; set UOR_R4_CUDA_REQUIRED=1, no CPU fallback"]
    fn native_alias_cuda_exact_full_trace_edges_and_repeated_calls() -> Result<()> {
        let d = cuda_required()?;
        for (v, sparse) in [(12, false), (12, true), (4096, false)] {
            let b = binding(v, sparse)?;
            let bytes = exp();
            let prepared = PreparedNativeVocabularyCuda::new(b.clone(), &bytes, &d)?;
            let mut cpu =
                NativeVocabularyActions::new(b, &bytes).map_err(|e| invalid(e.to_string()))?;
            let extremes = [
                i64::MIN,
                -(8 << 24) - 1,
                -(8 << 24),
                -65537,
                -65536,
                -1,
                0,
                1,
                65535,
                65536,
                65537,
                (8 << 24),
                (8 << 24) + 1,
                (1i64 << 62) + (1i64 << 38) + 1,
                i64::MAX,
            ];
            for round in 0..extremes.len() + 2 {
                let gen = (0..v)
                    .map(|i| {
                        if round < extremes.len() {
                            extremes[(i + round) % extremes.len()]
                        } else {
                            0
                        }
                    })
                    .collect::<Vec<_>>();
                let ids = if round == 0 {
                    vec![]
                } else if v == 4096 {
                    vec![4; 128]
                } else {
                    vec![4, 4, 5, 1, 3]
                };
                let scores = (0..ids.len())
                    .map(|i| {
                        if round < extremes.len() {
                            extremes[(2 * i + round) % extremes.len()]
                        } else {
                            0
                        }
                    })
                    .collect::<Vec<_>>();
                let expected = cpu
                    .reduce_trace(&gen, &ids, &scores)
                    .map_err(|e| invalid(e.to_string()))?;
                let pool =
                    prepared.reduce(&Tensor::from_vec(gen.clone(), v, &d)?, &ids, &scores)?;
                assert_eq!(
                    pool.trace()?,
                    expected,
                    "v={v} sparse={sparse} round={round}"
                );
                assert_eq!(
                    pool.raw_action_scores().to_vec1::<f32>()?,
                    expected
                        .actions
                        .iter()
                        .map(|a| (a.raw_score_q24 as f64 / 16777216.) as f32)
                        .collect::<Vec<_>>()
                );
                let target = 4;
                let mass = expected
                    .token_masses
                    .iter()
                    .find(|m| m.token_id == target)
                    .ok_or_else(|| invalid("test target absent"))?
                    .weight_q31;
                assert_eq!(
                    pool.native_probability(target)?.to_bits(),
                    ((mass as f64 / expected.summary.total_weight_q31 as f64) as f32).to_bits()
                );
            }
        }
        Ok(())
    }
    #[test]
    #[ignore = "explicit CUDA required; set UOR_R4_CUDA_REQUIRED=1, no CPU fallback"]
    fn native_alias_cuda_admission_and_device_loss_credit_parity() -> Result<()> {
        let d = cuda_required()?;
        let b = binding(12, true)?;
        let bytes = exp();
        let prepared = PreparedNativeVocabularyCuda::new(b.clone(), &bytes, &d)?;
        assert!(PreparedNativeVocabularyCuda::new(b.clone(), &bytes, &Device::Cpu).is_err());
        let mut bad = bytes.clone();
        bad[4] ^= 1;
        assert!(PreparedNativeVocabularyCuda::new(b.clone(), &bad, &d).is_err());
        assert!(prepared
            .reduce(&Tensor::zeros(12, DType::F32, &d)?, &[], &[])
            .is_err());
        assert!(prepared
            .reduce(&Tensor::zeros(12, DType::I64, &Device::Cpu)?, &[], &[])
            .is_err());
        let gen = vec![0i64; 12];
        let hard = Tensor::from_vec(gen.clone(), 12, &d)?;
        let mut padded = vec![i64::MAX];
        padded.extend_from_slice(&gen);
        let offset = Tensor::from_vec(padded, 13, &d)?.narrow(0, 1, 12)?;
        assert_eq!(
            prepared.reduce(&offset, &[], &[])?.trace()?,
            prepared.reduce(&hard, &[], &[])?.trace()?
        );
        assert!(prepared.reduce(&hard, &[7], &[0]).is_err());
        assert!(prepared.reduce(&hard, &[4], &[]).is_err());
        assert!(prepared
            .reduce(&hard, &vec![4; 129], &vec![0; 129])
            .is_err());
        let ids = [4, 4, 5];
        let copy = [9 << 24, -65537, 1];
        let pool = prepared.reduce(&hard, &ids, &copy)?;
        assert!(pool.native_probability(7).is_err());
        let mut cpu =
            NativeVocabularyActions::new(b, &bytes).map_err(|e| invalid(e.to_string()))?;
        let trace = cpu
            .reduce_trace(&gen, &ids, &copy)
            .map_err(|e| invalid(e.to_string()))?;
        let gv = Var::from_vec(vec![0f32; 12], 12, &d)?;
        let cv = Var::from_vec(
            copy.iter()
                .map(|&x| (x as f64 / 16777216.) as f32)
                .collect::<Vec<_>>(),
            3,
            &d,
        )?;
        for credit in [
            VocabularyScoreAdjoint::Clipped,
            VocabularyScoreAdjoint::RawIdentity,
        ] {
            let old = vocabulary_marginal_loss_with_credit(
                &trace,
                gv.as_tensor(),
                Some(cv.as_tensor()),
                4,
                credit,
            )?;
            let new = vocabulary_marginal_loss_device_with_credit(
                &pool,
                gv.as_tensor(),
                Some(cv.as_tensor()),
                4,
                credit,
            )?;
            assert_eq!(
                old.to_scalar::<f32>()?.to_bits(),
                new.to_scalar::<f32>()?.to_bits()
            );
            let (og, ng) = (old.backward()?, new.backward()?);
            for var in [&gv, &cv] {
                assert_eq!(
                    og.get(var.as_tensor())
                        .ok_or_else(|| invalid("old grad absent"))?
                        .to_vec1::<f32>()?,
                    ng.get(var.as_tensor())
                        .ok_or_else(|| invalid("new grad absent"))?
                        .to_vec1::<f32>()?
                );
            }
        }
        assert!(vocabulary_marginal_loss_device_with_credit(
            &pool,
            &gv.as_tensor().affine(1., 1.)?,
            Some(cv.as_tensor()),
            4,
            VocabularyScoreAdjoint::Clipped
        )
        .is_err());
        assert!(vocabulary_marginal_loss_device_with_credit(
            &pool,
            &Tensor::from_vec(vec![f32::NAN; 12], 12, &d)?,
            Some(cv.as_tensor()),
            4,
            VocabularyScoreAdjoint::Clipped
        )
        .is_err());
        Ok(())
    }
}
fn wrap_i(slice: cudarc::driver::CudaSlice<i64>, d: &candle_core::CudaDevice, n: usize) -> Tensor {
    Tensor::from_storage(
        Storage::Cuda(CudaStorage::wrap_cuda_slice(slice, d.clone())),
        n,
        BackpropOp::none(),
        false,
    )
}
fn wrap_f(slice: cudarc::driver::CudaSlice<f32>, d: &candle_core::CudaDevice, n: usize) -> Tensor {
    Tensor::from_storage(
        Storage::Cuda(CudaStorage::wrap_cuda_slice(slice, d.clone())),
        n,
        BackpropOp::none(),
        false,
    )
}

pub struct PreparedNativeVocabularyCuda {
    admission: NativeVocabularyActions,
    binding: Arc<SourceActionBinding>,
    legal_ids: Arc<[u32]>,
    legal: Tensor,
    exp: Tensor,
    device: Device,
    exp_len: usize,
}
impl PreparedNativeVocabularyCuda {
    pub fn new(binding: SourceActionBinding, exp_bytes: &[u8], device: &Device) -> Result<Self> {
        if !device.is_cuda() {
            return Err(invalid("native alias CUDA explicitly requires CUDA"));
        }
        let admission =
            NativeVocabularyActions::new(binding, exp_bytes).map_err(|e| invalid(e.to_string()))?;
        // Parse only after complete canonical authentication by the native owner.
        let exp = exp_bytes
            .chunks_exact(4)
            .map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
            .collect::<Vec<_>>();
        let exp_len = exp.len();
        let legal = admission.legal_token_ids().to_vec();
        Ok(Self {
            binding: Arc::new(admission.binding().clone()),
            legal_ids: legal.clone().into(),
            admission,
            legal: Tensor::from_vec(legal.clone(), legal.len(), device)?,
            exp: Tensor::from_vec(exp, exp_len, device)?,
            device: device.clone(),
            exp_len,
        })
    }
    pub fn binding(&self) -> &SourceActionBinding {
        self.admission.binding()
    }
    pub fn device(&self) -> &Device {
        &self.device
    }
    pub fn reduce(
        &self,
        generate_q24: &Tensor,
        copy_ids: &[u32],
        copy_q24: &[i64],
    ) -> Result<DeviceVocabularyReduction> {
        let vocab = self.admission.vocab_size();
        let n = self
            .admission
            .action_count(copy_ids.len())
            .map_err(|e| invalid(e.to_string()))?;
        if generate_q24.dtype() != DType::I64
            || generate_q24.dims() != [vocab]
            || !generate_q24.device().same_device(&self.device)
            || copy_ids.len() != copy_q24.len()
        {
            return Err(invalid(
                "native alias CUDA input shape/dtype/device differs",
            ));
        }
        if copy_ids.iter().any(|&id| !self.binding().admits_token(id)) {
            return Err(invalid(
                "native alias CUDA Copy token is unknown or a sparse hole",
            ));
        }
        // Kernel arguments address storage from zero, whereas contiguous narrow
        // views may retain a nonzero offset. Copy only those or strided inputs.
        let generate_q24 =
            if generate_q24.is_contiguous() && generate_q24.layout().start_offset() == 0 {
                generate_q24.clone()
            } else {
                generate_q24.force_contiguous()?
            };
        let ng = self.admission.legal_token_ids().len();
        let nc = copy_ids.len();
        let Device::Cuda(d) = &self.device else {
            return Err(invalid("alias CUDA device absent"));
        };
        let copies = Tensor::from_vec(
            if nc == 0 { vec![0] } else { copy_ids.to_vec() },
            nc.max(1),
            &self.device,
        )?;
        let copy_scores = Tensor::from_vec(
            if nc == 0 {
                vec![0i64]
            } else {
                copy_q24.to_vec()
            },
            nc.max(1),
            &self.device,
        )?;
        let action_ids = if nc == 0 {
            self.legal.clone()
        } else {
            Tensor::cat(&[&self.legal, &copies], 0)?
        };
        let refs = d.alloc_zeros::<i64>(2)?;
        let weights = d.alloc_zeros::<i64>(n)?;
        let masses = d.alloc_zeros::<i64>(vocab)?;
        let gm = d.alloc_zeros::<i64>(vocab)?;
        let rm = d.alloc_zeros::<i64>(vocab)?;
        let hard = d.alloc_zeros::<f32>(n)?;
        let rawf = d.alloc_zeros::<f32>(n)?;
        let summary = d.alloc_zeros::<i64>(19)?;
        let (g, _) = generate_q24.storage_and_layout();
        let (l, _) = self.legal.storage_and_layout();
        let (ci, _) = copies.storage_and_layout();
        let (cs, _) = copy_scores.storage_and_layout();
        let (e, _) = self.exp.storage_and_layout();
        launch(
            d,
            "native_alias_reference",
            1,
            &[
                Arg::I(storage(&g)?.as_cuda_slice::<i64>()?.as_view()),
                Arg::U(storage(&l)?.as_cuda_slice::<u32>()?.as_view()),
                Arg::I(storage(&cs)?.as_cuda_slice::<i64>()?.as_view()),
                Arg::I(refs.as_view()),
                Arg::N(ng as u32),
                Arg::N(nc as u32),
            ],
        )?;
        launch(
            d,
            "native_alias_weights",
            n,
            &[
                Arg::I(storage(&g)?.as_cuda_slice::<i64>()?.as_view()),
                Arg::U(storage(&l)?.as_cuda_slice::<u32>()?.as_view()),
                Arg::U(storage(&ci)?.as_cuda_slice::<u32>()?.as_view()),
                Arg::I(storage(&cs)?.as_cuda_slice::<i64>()?.as_view()),
                Arg::U(storage(&e)?.as_cuda_slice::<u32>()?.as_view()),
                Arg::I(refs.as_view()),
                Arg::I(weights.as_view()),
                Arg::I(masses.as_view()),
                Arg::I(gm.as_view()),
                Arg::I(rm.as_view()),
                Arg::F(hard.as_view()),
                Arg::F(rawf.as_view()),
                Arg::N(ng as u32),
                Arg::N(nc as u32),
                Arg::N(self.exp_len as u32),
            ],
        )?;
        launch(
            d,
            "native_alias_summary",
            1,
            &[
                Arg::I(storage(&g)?.as_cuda_slice::<i64>()?.as_view()),
                Arg::U(storage(&l)?.as_cuda_slice::<u32>()?.as_view()),
                Arg::I(storage(&cs)?.as_cuda_slice::<i64>()?.as_view()),
                Arg::I(refs.as_view()),
                Arg::I(masses.as_view()),
                Arg::I(gm.as_view()),
                Arg::I(rm.as_view()),
                Arg::I(summary.as_view()),
                Arg::N(ng as u32),
                Arg::N(nc as u32),
            ],
        )?;
        // Guards own storage references; drop before moving the immutable input.
        drop((g, l, ci, cs, e));
        Ok(DeviceVocabularyReduction {
            binding: self.binding.clone(),
            legal_ids: self.legal_ids.clone(),
            copy_ids: copy_ids.to_vec(),
            copy_scores: copy_q24.to_vec(),
            generate_q24,
            action_ids,
            weights: wrap_i(weights, d, n),
            masses: wrap_i(masses, d, vocab),
            generate_masses: wrap_i(gm, d, vocab),
            hard: wrap_f(hard, d, n),
            raw: wrap_f(rawf, d, n),
            summary: wrap_i(summary, d, 19),
        })
    }
}

/// Privately admitted exact reduction. Only trace() downloads vocabulary arrays;
/// summary/probability helpers transfer small scalar evidence explicitly.
pub struct DeviceVocabularyReduction {
    binding: Arc<SourceActionBinding>,
    legal_ids: Arc<[u32]>,
    copy_ids: Vec<u32>,
    copy_scores: Vec<i64>,
    generate_q24: Tensor,
    action_ids: Tensor,
    weights: Tensor,
    masses: Tensor,
    generate_masses: Tensor,
    hard: Tensor,
    raw: Tensor,
    summary: Tensor,
}
impl DeviceVocabularyReduction {
    pub fn binding(&self) -> &SourceActionBinding {
        &self.binding
    }
    pub fn legal_generate_count(&self) -> usize {
        self.legal_ids.len()
    }
    pub fn copy_count(&self) -> usize {
        self.copy_ids.len()
    }
    pub fn action_token_ids(&self) -> &Tensor {
        &self.action_ids
    }
    pub fn hard_action_scores(&self) -> &Tensor {
        &self.hard
    }
    pub fn raw_action_scores(&self) -> &Tensor {
        &self.raw
    }
    pub fn action_weights_q31(&self) -> &Tensor {
        &self.weights
    }
    pub fn token_masses_q31(&self) -> &Tensor {
        &self.masses
    }
    pub fn generate_masses_q31(&self) -> &Tensor {
        &self.generate_masses
    }
    pub fn legal_generate_ids(&self) -> Result<Tensor> {
        Ok(self.action_ids.narrow(0, 0, self.legal_ids.len())?)
    }
    pub fn target_mask(&self, target: u32) -> Result<Tensor> {
        self.admit_target(target)?;
        Ok(self.action_ids.eq(target)?.to_dtype(DType::F32)?)
    }
    fn admit_target(&self, target: u32) -> Result<()> {
        if self.legal_ids.binary_search(&target).is_err() {
            return Err(invalid("native alias target not legal"));
        }
        Ok(())
    }
    pub fn target_mass(&self, target: u32) -> Result<u64> {
        self.admit_target(target)?;
        let mass = self
            .masses
            .narrow(0, target as usize, 1)?
            .reshape(())?
            .to_scalar::<i64>()?;
        u64::try_from(mass).map_err(|_| invalid("native alias negative target mass"))
    }
    pub fn native_probability(&self, target: u32) -> Result<f32> {
        let mass = self.target_mass(target)?;
        let total = self
            .summary
            .narrow(0, 3, 1)?
            .reshape(())?
            .to_scalar::<i64>()?;
        if mass == 0 || total <= 0 || mass > total as u64 {
            return Err(invalid("native alias target mass invalid"));
        }
        // Preserve authoritative CPU F64 division followed by F32 rounding.
        let p = (mass as f64 / total as f64) as f32;
        if !p.is_finite() || p <= 0. {
            return Err(invalid("native alias probability invalid"));
        }
        Ok(p)
    }
    pub fn summary(&self) -> Result<VocabularyReduction> {
        let a = self.summary.to_vec1::<i64>()?;
        let u = |i: usize| {
            u64::try_from(a[i]).map_err(|_| invalid("native alias summary negative mass/count"))
        };
        Ok(VocabularyReduction {
            legal_generate_actions: u(0)? as usize,
            copy_actions: u(1)? as usize,
            max_score_q24: a[2],
            total_weight_q31: u(3)?,
            chosen_token_id: u(4)? as u32,
            chosen_weight_q31: u(5)?,
            generate_weight_q31: u(6)?,
            copy_weight_q31: u(7)?,
            chosen_generate_weight_q31: u(8)?,
            chosen_copy_weight_q31: u(9)?,
            clipped_low_actions: u(10)? as usize,
            clipped_high_actions: u(11)? as usize,
            raw_max_score_q24: a[12],
            raw_total_weight_q31: u(13)?,
            raw_chosen_token_id: u(14)? as u32,
            raw_chosen_weight_q31: u(15)?,
            chosen_raw_mass_rank: u(16)? as usize,
            raw_chosen_clipped_mass_rank: u(17)? as usize,
            token_winner_changed_by_clip: a[18] != 0,
        })
    }
    pub fn trace(&self) -> Result<VocabularyActionTrace> {
        let summary = self.summary()?;
        let gen = self.generate_q24.to_vec1::<i64>()?;
        let weights = self.weights.to_vec1::<i64>()?;
        let masses = self.masses.to_vec1::<i64>()?;
        let gm = self.generate_masses.to_vec1::<i64>()?;
        let mut actions = Vec::with_capacity(weights.len());
        for (i, &w) in weights.iter().enumerate() {
            let (action, id, raw) = if i < self.legal_ids.len() {
                let id = self.legal_ids[i];
                (
                    VocabularyAction::Generate { token_id: id },
                    id,
                    gen[id as usize],
                )
            } else {
                let j = i - self.legal_ids.len();
                (
                    VocabularyAction::Copy { source_offset: j },
                    self.copy_ids[j],
                    self.copy_scores[j],
                )
            };
            if w <= 0 {
                return Err(invalid("native alias nonpositive action weight"));
            }
            actions.push(VocabularyActionMass {
                action,
                action_offset: i,
                token_id: id,
                raw_score_q24: raw,
                score_q24: raw.clamp(-(8 << 24), 8 << 24),
                weight_q31: w as u64,
            });
        }
        let mut token_masses = Vec::with_capacity(self.legal_ids.len());
        for &id in self.legal_ids.iter() {
            let (m, g) = (masses[id as usize], gm[id as usize]);
            if m < g || g <= 0 {
                return Err(invalid("native alias token decomposition invalid"));
            }
            token_masses.push(VocabularyTokenMass {
                token_id: id,
                weight_q31: m as u64,
                generate_weight_q31: g as u64,
                copy_weight_q31: (m - g) as u64,
            });
        }
        Ok(VocabularyActionTrace {
            policy: POLICY,
            tokenizer_sha256: self.binding.tokenizer_sha256().into(),
            period_token_id: self.binding.period_token_id(),
            eos_token_id: self.binding.eos_token_id(),
            summary,
            actions,
            token_masses,
        })
    }
}
