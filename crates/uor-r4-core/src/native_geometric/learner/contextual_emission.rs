//! A learned shared low-bit emission residual for one read -> geometric update -> emit primitive.
//!
//! The frozen reader can only boost the selected payload logit, so it cannot favour a token absent
//! from every admitted payload. This module adds a bounded shared residual
//!
//! ```text
//! q0 = bounded causal query state (frozen)
//! q1 = update algebra applied to (q0, relation, selected payload)
//! z  = z_local + ( W . (R[q1] - R[q0]) ) << shift
//! NoRead / UpdateDisabled: q1 = q0, residual exactly zero
//! ```
//!
//! `R` is a small seeded nonconstant ternary state embedding (`120 x CE_WIDTH`); `W` is a learned
//! ternary output map (`vocab x CE_WIDTH`). All served coefficients are two-bit, the only scale is a
//! declared integer shift, and the residual is a sum of additions/shifts/table reads.
#![forbid(unsafe_code)]

use super::group_table::GROUP_ORDER;
use super::read_conditioned::ReadConditionedParams;

/// Residual feature width.
pub const CE_WIDTH: usize = 16;

/// One development example: everything the residual and optimiser need, with no target inside the
/// prediction path. `z_local` already contains the query row `u(q0)`.
#[derive(Clone, Debug)]
pub struct CeExample {
    pub z_local: Vec<i32>,
    pub q0: usize,
    pub r: usize,
    pub payload: u32,
    pub read: bool,
    pub target: u32,
}

impl CeExample {
    /// The update's end state under one algebra (`cyclic = false` is exact signed-H4).
    pub fn q1(&self, params: &ReadConditionedParams, cyclic: bool) -> usize {
        if !self.read {
            return self.q0;
        }
        if cyclic {
            params.update_cyclic(self.q0, self.r, self.payload)
        } else {
            params.update(self.q0, self.r, self.payload)
        }
    }
}

/// The served residual model.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EmissionResidual {
    /// Seeded nonconstant ternary state embedding, `GROUP_ORDER x CE_WIDTH`.
    pub r_embed: Vec<[i8; CE_WIDTH]>,
    /// Learned ternary output map, `vocab x CE_WIDTH`. Zero rows are legal.
    pub w: Vec<[i8; CE_WIDTH]>,
    /// Declared integer scale applied to the summed residual.
    pub shift: u32,
}

impl EmissionResidual {
    /// Nonconstant ternary state embedding with a zero output map: the exact-baseline start that
    /// still admits a learning signal once the state changes.
    pub fn seeded(vocab: usize, seed: u64) -> Self {
        let mut st = seed | 1;
        let mut next = |m: u32| -> i8 {
            st ^= st << 13;
            st ^= st >> 7;
            st ^= st << 17;
            match st % (m as u64) {
                0 => -1,
                1 => 0,
                _ => 1,
            }
        };
        let r_embed = (0..GROUP_ORDER)
            .map(|_| std::array::from_fn(|_| next(3)))
            .collect();
        EmissionResidual {
            r_embed,
            w: vec![[0i8; CE_WIDTH]; vocab],
            shift: 0,
        }
    }

    /// `R[q1] - R[q0]` in integer units (zero when the state is unchanged).
    pub fn delta(&self, q0: usize, q1: usize) -> [i32; CE_WIDTH] {
        if q0 == q1 {
            return [0i32; CE_WIDTH];
        }
        let a = &self.r_embed[q1.min(GROUP_ORDER - 1)];
        let b = &self.r_embed[q0.min(GROUP_ORDER - 1)];
        std::array::from_fn(|j| i32::from(a[j]) - i32::from(b[j]))
    }

    /// The residual added to every vocabulary logit. Exactly zero when `q0 == q1`.
    pub fn residual_i32(&self, q0: usize, q1: usize) -> Vec<i32> {
        let d = self.delta(q0, q1);
        if d.iter().all(|x| *x == 0) {
            return vec![0i32; self.w.len()];
        }
        let scale = 1i64 << self.shift.min(20);
        self.w
            .iter()
            .map(|row| {
                let mut s = 0i64;
                for j in 0..CE_WIDTH {
                    s += i64::from(row[j]) * i64::from(d[j]);
                }
                ((s * scale).clamp(i64::from(i32::MIN), i64::from(i32::MAX))) as i32
            })
            .collect()
    }

    /// The largest |residual| any state change can produce, in integer logit units:
    /// `CE_WIDTH * max|W| * max|delta_R| * 2^shift`.
    pub fn range_bound(&self) -> i64 {
        let maxw = self
            .w
            .iter()
            .flatten()
            .map(|v| (*v as i64).abs())
            .max()
            .unwrap_or(0);
        let maxr = self
            .r_embed
            .iter()
            .flatten()
            .map(|v| (*v as i64).abs())
            .max()
            .unwrap_or(0);
        (CE_WIDTH as i64) * maxw * 2 * maxr * (1i64 << self.shift.min(20))
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut o = Vec::new();
        o.extend_from_slice(b"RLCE");
        o.extend_from_slice(&1u32.to_le_bytes());
        o.extend_from_slice(&(CE_WIDTH as u32).to_le_bytes());
        o.extend_from_slice(&self.shift.to_le_bytes());
        o.extend_from_slice(&(self.r_embed.len() as u32).to_le_bytes());
        for row in self.r_embed.iter() {
            for v in row.iter() {
                o.push(*v as u8);
            }
        }
        o.extend_from_slice(&(self.w.len() as u32).to_le_bytes());
        for row in self.w.iter() {
            for v in row.iter() {
                o.push(*v as u8);
            }
        }
        o
    }

    /// Independent load with bounded allocations and strict ternary/range checks.
    pub fn from_bytes(bytes: &[u8], max_vocab: usize) -> Result<Self, String> {
        let mut c = 0usize;
        let take = |c: &mut usize, n: usize| -> Result<&[u8], String> {
            let end = c.checked_add(n).ok_or("size overflow")?;
            if end > bytes.len() {
                return Err("truncated emission artifact".into());
            }
            let s = &bytes[*c..end];
            *c = end;
            Ok(s)
        };
        if take(&mut c, 4)? != b"RLCE" {
            return Err("bad emission artifact magic".into());
        }
        let ver = u32::from_le_bytes(take(&mut c, 4)?.try_into().unwrap());
        if ver != 1 {
            return Err("unsupported emission artifact version".into());
        }
        let width = u32::from_le_bytes(take(&mut c, 4)?.try_into().unwrap()) as usize;
        if width != CE_WIDTH {
            return Err("emission artifact width mismatch".into());
        }
        let shift = u32::from_le_bytes(take(&mut c, 4)?.try_into().unwrap());
        if shift > 20 {
            return Err("emission artifact shift out of range".into());
        }
        let n_r = u32::from_le_bytes(take(&mut c, 4)?.try_into().unwrap()) as usize;
        if n_r != GROUP_ORDER {
            return Err("emission state embedding has the wrong row count".into());
        }
        let raw_r = take(&mut c, n_r * CE_WIDTH)?;
        let r_embed: Vec<[i8; CE_WIDTH]> = (0..n_r)
            .map(|i| std::array::from_fn(|j| raw_r[i * CE_WIDTH + j] as i8))
            .collect();
        let n_w = u32::from_le_bytes(take(&mut c, 4)?.try_into().unwrap()) as usize;
        if n_w == 0 || n_w > max_vocab {
            return Err("emission output map has an out-of-range row count".into());
        }
        let raw_w = take(&mut c, n_w * CE_WIDTH)?;
        let w: Vec<[i8; CE_WIDTH]> = (0..n_w)
            .map(|i| std::array::from_fn(|j| raw_w[i * CE_WIDTH + j] as i8))
            .collect();
        if c != bytes.len() {
            return Err("trailing bytes in the emission artifact".into());
        }
        if r_embed
            .iter()
            .chain(w.iter())
            .flatten()
            .any(|v| !(-1..=1).contains(v))
        {
            return Err("emission coefficients outside the declared ternary range".into());
        }
        Ok(EmissionResidual { r_embed, w, shift })
    }
}

/// Full-batch gradient training of the output map `W` under NLL with the state maps fixed.
/// Returns the float map and the initial/final NLL so the run can show that learning occurred.
pub fn train_output_map(
    examples: &[CeExample],
    params: &ReadConditionedParams,
    res: &EmissionResidual,
    cyclic: bool,
    epochs: usize,
    lr: f64,
) -> (Vec<[f32; CE_WIDTH]>, f64, f64) {
    let vocab = res.w.len();
    if examples.is_empty() || vocab == 0 {
        return (vec![[0.0f32; CE_WIDTH]; vocab], f64::NAN, f64::NAN);
    }
    let scale = (1i64 << res.shift.min(20)) as f64;
    // Precompute the feature vector per example (it does not depend on W).
    let feats: Vec<[f32; CE_WIDTH]> = examples
        .iter()
        .map(|ex| {
            let d = res.delta(ex.q0, ex.q1(params, cyclic));
            std::array::from_fn(|j| d[j] as f32 * scale as f32)
        })
        .collect();
    let mut w = vec![[0.0f32; CE_WIDTH]; vocab];
    let nll = |w: &[[f32; CE_WIDTH]]| -> f64 {
        let mut total = 0.0f64;
        for (ex, f) in examples.iter().zip(feats.iter()) {
            let t = (ex.target as usize).min(vocab - 1);
            let mut max = f64::NEG_INFINITY;
            let mut logits = vec![0.0f64; vocab];
            for o in 0..vocab {
                let mut s = 0.0f64;
                for j in 0..CE_WIDTH {
                    s += f64::from(w[o][j]) * f64::from(f[j]);
                }
                logits[o] = ex.z_local[o] as f64 + s;
                max = max.max(logits[o]);
            }
            let mut sum = 0.0f64;
            for v in logits.iter() {
                sum += (v - max).exp();
            }
            total += (max + sum.ln() - logits[t]) / std::f64::consts::LN_2;
        }
        total / examples.len() as f64
    };
    let initial = nll(&w);
    for _ in 0..epochs {
        let mut grad = vec![[0.0f32; CE_WIDTH]; vocab];
        for (ex, f) in examples.iter().zip(feats.iter()) {
            let t = (ex.target as usize).min(vocab - 1);
            let mut max = f64::NEG_INFINITY;
            let mut logits = vec![0.0f64; vocab];
            for o in 0..vocab {
                let mut s = 0.0f64;
                for j in 0..CE_WIDTH {
                    s += f64::from(w[o][j]) * f64::from(f[j]);
                }
                logits[o] = ex.z_local[o] as f64 + s;
                max = max.max(logits[o]);
            }
            let mut sum = 0.0f64;
            for v in logits.iter() {
                sum += (v - max).exp();
            }
            for o in 0..vocab {
                let p = (logits[o] - max).exp() / sum;
                let y = if o == t { 1.0f64 } else { 0.0f64 };
                let g = (p - y) as f32;
                for j in 0..CE_WIDTH {
                    grad[o][j] += g * f[j];
                }
            }
        }
        // The features carry the declared shift, so the step must be scaled by their magnitude or
        // the fit diverges. This is the diagnosed defect from the first pass.
        let fscale = (feats
            .iter()
            .map(|f| f.iter().map(|v| f64::from(*v).abs()).sum::<f64>())
            .sum::<f64>()
            / examples.len() as f64)
            .max(1.0);
        let step = (lr / (examples.len() as f64 * fscale)) as f32;
        for o in 0..vocab {
            for j in 0..CE_WIDTH {
                w[o][j] -= step * grad[o][j];
            }
        }
    }
    let final_nll = nll(&w);
    (w, initial, final_nll)
}

/// Teacher-forced NLL in bits of the **served** (quantized, sparse) residual.
pub fn served_nll_bits(
    examples: &[CeExample],
    params: &ReadConditionedParams,
    res: &EmissionResidual,
    cyclic: bool,
    rows: &[usize],
) -> f64 {
    if examples.is_empty() || res.w.is_empty() {
        return f64::NAN;
    }
    let vocab = res.w.len();
    let scale = 1i64 << res.shift.min(20);
    let mut total = 0.0f64;
    for ex in examples.iter() {
        let q1 = ex.q1(params, cyclic);
        let d = res.delta(ex.q0, q1);
        let mut logits = ex.z_local.clone();
        if d.iter().any(|v| *v != 0) {
            for &o in rows {
                let mut acc = 0i64;
                for j in 0..CE_WIDTH {
                    acc += i64::from(res.w[o][j]) * i64::from(d[j]);
                }
                logits[o] = logits[o].saturating_add(
                    (acc * scale).clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32,
                );
            }
        }
        let t = (ex.target as usize).min(vocab - 1);
        let max = logits.iter().copied().max().unwrap_or(0) as f64;
        let sum: f64 = logits.iter().map(|v| (f64::from(*v) - max).exp()).sum();
        total += (max + sum.ln() - f64::from(logits[t])) / std::f64::consts::LN_2;
    }
    total / examples.len() as f64
}

/// Bounded discrete refinement of the ternary output map: flip single coefficients while the mean
/// margin strictly improves. This is what makes a quantized map trainable.
pub fn refine_ternary_map(
    examples: &[CeExample],
    params: &ReadConditionedParams,
    res: &mut EmissionResidual,
    cyclic: bool,
    rows: &[usize],
    passes: usize,
) -> usize {
    let score = |res: &EmissionResidual| -> f64 {
        let mut total = 0.0f64;
        for ex in examples.iter() {
            let q1 = ex.q1(params, cyclic);
            let d = res.delta(ex.q0, q1);
            let scale = 1i64 << res.shift.min(20);
            let mut z = ex.z_local.clone();
            if d.iter().any(|v| *v != 0) {
                for &o in rows {
                    let mut acc = 0i64;
                    for j in 0..CE_WIDTH {
                        acc += i64::from(res.w[o][j]) * i64::from(d[j]);
                    }
                    z[o] = z[o].saturating_add(
                        (acc * scale).clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32,
                    );
                }
            }
            let t = (ex.target as usize).min(z.len() - 1);
            let best_other = z
                .iter()
                .enumerate()
                .filter(|(i, _)| *i != t)
                .map(|(_, v)| *v)
                .max()
                .unwrap_or(i32::MIN);
            total += f64::from(z[t] - best_other);
        }
        total / examples.len() as f64
    };
    let mut best = score(res);
    let mut accepted = 0usize;
    for _ in 0..passes {
        let mut improved = false;
        for &o in rows.iter() {
            for j in 0..CE_WIDTH {
                let saved = res.w[o][j];
                for cand in [-1i8, 0, 1] {
                    if cand == saved {
                        continue;
                    }
                    res.w[o][j] = cand;
                    let m = score(res);
                    if m > best + 1e-9 {
                        best = m;
                        accepted += 1;
                        improved = true;
                    } else {
                        res.w[o][j] = saved;
                    }
                }
            }
        }
        if !improved {
            break;
        }
    }
    accepted
}

/// Ternary quantization of a trained float map: the sign of each coefficient.
pub fn quantize_ternary(w_f: &[[f32; CE_WIDTH]]) -> Vec<[i8; CE_WIDTH]> {
    w_f.iter()
        .map(|row| {
            std::array::from_fn(|j| {
                if row[j] > 0.0 {
                    1
                } else if row[j] < 0.0 {
                    -1
                } else {
                    0
                }
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disabled_state_gives_an_exactly_zero_residual_and_round_trips() {
        let mut res = EmissionResidual::seeded(64, 0x1234);
        let q0 = 7usize;
        let z = res.residual_i32(q0, q0);
        assert!(z.iter().all(|v| *v == 0), "q1 == q0 must be exactly zero");
        res.w[3] = [1, -1, 0, 1, 0, -1, 1, 0, 0, 1, -1, 0, 1, 0, -1, 1];
        assert!(res.residual_i32(q0, q0).iter().all(|v| *v == 0));
        let bytes = res.to_bytes();
        let back = EmissionResidual::from_bytes(&bytes, 64).unwrap();
        assert_eq!(back, res);
        assert!(
            EmissionResidual::from_bytes(&bytes, 4).is_err(),
            "bounded vocab"
        );
        let mut bad = bytes.clone();
        bad.truncate(bad.len() - 1);
        assert!(EmissionResidual::from_bytes(&bad, 64).is_err());
    }

    #[test]
    fn training_reduces_nll_and_the_update_can_change_the_residual() {
        // Two examples with different features and different targets.
        let params = ReadConditionedParams::identity();
        let res = EmissionResidual::seeded(8, 9);
        let mk = |target: u32, payload: u32, r: usize| CeExample {
            z_local: vec![0; 8],
            q0: 1,
            r,
            payload,
            read: true,
            target,
        };
        let examples = vec![mk(2, 11, 3), mk(5, 12, 4)];
        let mut p = params.clone();
        p.value_domain = vec![11, 12];
        p.value_code = vec![1, 2];
        let (w_f, before, after) = train_output_map(&examples, &p, &res, false, 40, 0.5);
        assert!(after <= before + 1e-9, "training must not increase NLL");
        assert!(
            after < before - 1e-6,
            "training must reduce NLL: {before} -> {after}"
        );
        let _ = quantize_ternary(&w_f);
    }
}
