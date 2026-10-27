//! Hamming flock attention inside the in-space trunk (in-space-trunk-to-text, claude-146).
//!
//! The trunk is a selective quaternion recurrence: a token 16 back moves the prediction by 1–2%
//! of what the last token does. claude-144 and claude-145 each added one read, with credit only
//! to the few positions it picked. This read gives the trunk several cheap look-backs per layer,
//! with graded credit and position, by the owner's direction of 27 Oct (#2028).
//!
//! After layers 2 and 4 of 6 ([`HAMMING_DEPTHS`]), for each position `t` of a window with residual
//! point `x_t`, and for each of `heads` heads on its own block of coordinates:
//! 1. **Points.** The query and the key are `x` rotated lane by lane by learned unit quaternions
//!    (Hamilton products), then spun by their own position: lane `j` of a head turns by
//!    `t · SPIN_BASE^(−j / lanes)`. The spin carries position, RoPE's role. The spins share one
//!    axis, so they keep L2 distances a function of the offset `s − t` alone, but the sign codes
//!    below are not rotation-invariant: a head's Hamming distance depends on both positions, not
//!    only their offset. The value is `x` rotated by a third learned quaternion.
//! 2. **Codes.** Each coordinate's bit is 1 at or above 0, as [`pack_signs`] packs it. The forward
//!    pass compares the ±1 codes, so the distance is exactly the Hamming distance
//!    `popcount(q XOR k)` that serving computes. The backward pass treats the code as the identity
//!    (straight through), so the distance's gradient `(a − b) / 2` reaches the rotated points.
//! 3. **Choice.** The [`HAMMING_KEPT`] earlier positions `s < t` nearest in Hamming distance,
//!    hard and detached (claude-144's choice).
//! 4. **Weights.** `max(0, m − d) / HAMMING_KEPT`, with `d` in bits and a learned margin `m` per
//!    head: graded, so every kept candidate inside the margin gets credit, and nothing exponential.
//! 5. **Payload.** Even heads bring back the kept position's value, `v_s`. Odd heads bring its
//!    successor's, `v_{s+1}`, a known input because `s + 1 ≤ t`: "what followed the earlier match",
//!    a one-step induction head.
//! 6. **Add.** `x_t + w ⊙ c_t`, the weight `w` starting at 0, so at step 0 the trunk is unchanged.
//!
//! The reads are active for the thread that sets them ([`set_hamming_reads`]); every caller of
//! `StackModel::hidden` on that thread gets them. They train with claude-144's AdamW.
use std::cell::RefCell;
use std::path::Path;

use candle_core::{DType, Device, Tensor, Var};

use crate::flock_read::choose_nearest;
use crate::geometric_stack::hamilton_gate;
use crate::{invalid, Result};

/// Each read runs after this many of the trunk's 6 layers.
pub const HAMMING_DEPTHS: [usize; 2] = [2, 4];
/// Earlier positions each head keeps.
pub const HAMMING_KEPT: usize = 16;
/// Lane `j` of a head spins by `SPIN_BASE^(−j / lanes)` radians per position, RoPE's spacing.
const SPIN_BASE: f64 = 10_000.0;
const FILE: &str = "hamming_read.safetensors";

thread_local! {
    static ACTIVE: RefCell<Option<HammingReads>> = const { RefCell::new(None) };
}

/// Make `reads` the Hamming reads `StackModel::hidden` applies on this thread, or none.
pub fn set_hamming_reads(reads: Option<HammingReads>) {
    ACTIVE.with(|cell| *cell.borrow_mut() = reads);
}

/// The Hamming reads `StackModel::hidden` applies on this thread, if set.
pub fn hamming_reads() -> Option<HammingReads> {
    ACTIVE.with(|cell| cell.borrow().clone())
}

/// Each coordinate's sign bit, packed 64 to a word in coordinate order: the serving form of a
/// head's code, compared by `(a ^ b).count_ones()`. A coordinate at or above 0 is a 1.
pub fn pack_signs(coordinates: &[f32]) -> Vec<u64> {
    coordinates
        .chunks(64)
        .map(|chunk| {
            chunk
                .iter()
                .enumerate()
                .fold(0u64, |word, (i, &c)| word | (u64::from(c >= 0.0) << i))
        })
        .collect()
}

/// One read: learned per-lane rotations for the query, key and value, a margin per head (bits),
/// and the per-coordinate weight of what it adds.
#[derive(Clone)]
pub struct HammingRead {
    heads: usize,
    pub(crate) query_rotation: Var,
    pub(crate) key_rotation: Var,
    pub(crate) value_rotation: Var,
    pub(crate) margin: Var,
    pub(crate) weight: Var,
}

/// The reads at every depth in [`HAMMING_DEPTHS`], in that order.
#[derive(Clone)]
pub struct HammingReads {
    pub reads: Vec<HammingRead>,
}

impl HammingReads {
    /// Reads with `heads` heads on points of `width` coordinates: identity rotations, margins at
    /// half a head's bits, and zero weights, so they start as no read at all.
    pub fn new(heads: usize, width: usize, device: &Device) -> Result<Self> {
        let reads = HAMMING_DEPTHS
            .iter()
            .map(|_| HammingRead::new(heads, width, device))
            .collect::<Result<Vec<_>>>()?;
        Ok(Self { reads })
    }

    /// The read after `depth` layers, if one runs there.
    pub fn at_depth(&self, depth: usize) -> Option<&HammingRead> {
        HAMMING_DEPTHS
            .iter()
            .position(|&d| d == depth)
            .and_then(|i| self.reads.get(i))
    }

    /// Every learned parameter, in a fixed order for the optimizer.
    pub fn variables(&self) -> Vec<&Var> {
        self.reads.iter().flat_map(|r| r.variables()).collect()
    }

    /// The mean absolute value of every read's weight, as a scalar tensor.
    pub fn weight_mean_abs(&self) -> Result<Tensor> {
        let parts = self
            .reads
            .iter()
            .map(|r| r.weight.as_tensor().abs()?.mean_all())
            .collect::<candle_core::Result<Vec<_>>>()?;
        Ok(Tensor::stack(&parts, 0)?.mean_all()?)
    }

    /// Every read's margins in bits, read by read and head by head.
    pub fn margins(&self) -> Result<Vec<f64>> {
        let mut margins = Vec::new();
        for read in &self.reads {
            for m in read.margin.as_tensor().to_vec1::<f32>()? {
                margins.push(f64::from(m));
            }
        }
        Ok(margins)
    }

    /// Write the reads to `directory/hamming_read.safetensors`, with their head count.
    pub fn save(&self, directory: &Path) -> Result<()> {
        std::fs::create_dir_all(directory)?;
        let mut tensors = std::collections::HashMap::new();
        for (depth, read) in HAMMING_DEPTHS.iter().zip(&self.reads) {
            for (name, var) in read.named() {
                tensors.insert(format!("d{depth}.{name}"), var.as_tensor().clone());
            }
        }
        let heads = self.reads.first().map_or(0, |r| r.heads) as u32;
        let device = self.reads[0].weight.device();
        tensors.insert("heads".to_owned(), Tensor::new(&[heads], device)?);
        candle_core::safetensors::save(&tensors, directory.join(FILE))?;
        Ok(())
    }

    /// The reads saved in `directory`, or none when the checkpoint has none.
    pub fn load(directory: &Path, device: &Device) -> Result<Option<Self>> {
        let path = directory.join(FILE);
        if !path.exists() {
            return Ok(None);
        }
        let tensors = candle_core::safetensors::load(path, device)?;
        let get = |name: &str| -> Result<Var> {
            let tensor = tensors
                .get(name)
                .ok_or_else(|| invalid(format!("the saved Hamming read has no {name}")))?;
            Ok(Var::from_tensor(tensor)?)
        };
        let heads = get("heads")?.as_tensor().to_vec1::<u32>()?;
        let heads = *heads
            .first()
            .ok_or_else(|| invalid("the saved head count is empty"))? as usize;
        let mut reads = Vec::with_capacity(HAMMING_DEPTHS.len());
        for depth in HAMMING_DEPTHS {
            reads.push(HammingRead {
                heads,
                query_rotation: get(&format!("d{depth}.query_rotation"))?,
                key_rotation: get(&format!("d{depth}.key_rotation"))?,
                value_rotation: get(&format!("d{depth}.value_rotation"))?,
                margin: get(&format!("d{depth}.margin"))?,
                weight: get(&format!("d{depth}.weight"))?,
            });
        }
        Ok(Some(Self { reads }))
    }
}

impl HammingRead {
    fn new(heads: usize, width: usize, device: &Device) -> Result<Self> {
        if heads == 0 || !width.is_multiple_of(heads) {
            return Err(invalid(
                "a Hamming read needs a head count that divides the width",
            ));
        }
        let head_width = width / heads;
        if !head_width.is_multiple_of(4) || head_width > 64 {
            return Err(invalid(
                "a Hamming head needs whole 4-lane blocks and at most 64 coordinates (one word)",
            ));
        }
        let lanes = width / 4;
        let identity: Vec<f32> = (0..lanes).flat_map(|_| [1.0f32, 0.0, 0.0, 0.0]).collect();
        let rotation = || -> Result<Var> {
            Ok(Var::from_tensor(&Tensor::from_vec(
                identity.clone(),
                (lanes, 4),
                device,
            )?)?)
        };
        Ok(Self {
            heads,
            query_rotation: rotation()?,
            key_rotation: rotation()?,
            value_rotation: rotation()?,
            margin: Var::from_tensor(&Tensor::full((head_width / 2) as f32, heads, device)?)?,
            weight: Var::zeros(width, DType::F32, device)?,
        })
    }

    fn named(&self) -> [(&'static str, &Var); 5] {
        [
            ("query_rotation", &self.query_rotation),
            ("key_rotation", &self.key_rotation),
            ("value_rotation", &self.value_rotation),
            ("margin", &self.margin),
            ("weight", &self.weight),
        ]
    }

    fn variables(&self) -> [&Var; 5] {
        self.named().map(|(_, var)| var)
    }

    /// `points` rows rotated lane by lane by `rotation`'s unit quaternions.
    fn rotate(rotation: &Var, points: &Tensor) -> Result<Tensor> {
        let gate = rotation
            .as_tensor()
            .reshape((1, points.dim(1)?))?
            .broadcast_as(points.shape())?
            .contiguous()?;
        hamilton_gate(&gate, points)
    }

    /// The position spin for `batch` windows of `time` positions, as unit quaternions per lane,
    /// rows `(batch * time, width)`: lane `j` of each head turns by `t · SPIN_BASE^(−j / lanes)`.
    fn spin(&self, batch: usize, time: usize, width: usize, device: &Device) -> Result<Tensor> {
        let head_lanes = width / self.heads / 4;
        let mut gate = Vec::with_capacity(time * width);
        for t in 0..time {
            for lane in 0..width / 4 {
                let j = (lane % head_lanes) as f64;
                let angle = t as f64 * SPIN_BASE.powf(-j / head_lanes as f64);
                gate.extend_from_slice(&[angle.cos() as f32, angle.sin() as f32, 0.0, 0.0]);
            }
        }
        let gate = Tensor::from_vec(gate, (1, time, width), device)?;
        Ok(gate
            .broadcast_as((batch, time, width))?
            .reshape((batch * time, width))?)
    }

    /// The read applied to the residual `(batch, time, width)`; returns the same shape.
    pub fn apply(&self, residual: &Tensor) -> Result<Tensor> {
        let (batch, time, width) = residual.dims3()?;
        // A window of one token has no earlier position to read.
        if time < 2 {
            return Ok(residual.clone());
        }
        let device = residual.device();
        let rows = batch * time;
        let head_width = width / self.heads;
        let points = residual.reshape((rows, width))?;
        let spin = self.spin(batch, time, width, device)?;
        let query = hamilton_gate(&spin, &Self::rotate(&self.query_rotation, &points)?)?;
        let key = hamilton_gate(&spin, &Self::rotate(&self.key_rotation, &points)?)?;
        let value = Self::rotate(&self.value_rotation, &points)?;
        // ±1 by `p ≥ 0` forward, as `pack_signs` packs it, so 0 is a 1 bit; the identity backward.
        let code = |p: &Tensor| -> Result<Tensor> {
            let bits = p.ge(0.0)?.to_dtype(DType::F32)?.affine(2.0, -1.0)?;
            Ok(p.add(&bits.sub(p)?.detach())?)
        };
        let (query, key) = (code(&query)?, code(&key)?);
        let starts: Vec<u32> = (0..rows).map(|r| ((r / time) * time) as u32).collect();
        let starts = Tensor::from_vec(starts, (rows, 1), device)?;
        let one = Tensor::new(&[1u32], device)?;
        let margins = self.margin.as_tensor();
        let mut heads = Vec::with_capacity(self.heads);
        for h in 0..self.heads {
            let q = query.narrow(1, h * head_width, head_width)?.contiguous()?;
            let k = key.narrow(1, h * head_width, head_width)?.contiguous()?;
            let (picks, real) = choose_nearest(&q, &k, batch, time, HAMMING_KEPT)?;
            // A pick that is not real points at its window's start, whose successor is still
            // inside the window, and carries weight 0.
            let picks = real.where_cond(&picks, &starts.broadcast_as(picks.shape())?)?;
            let kept = picks.dim(1)?;
            let flat = picks.flatten_all()?;
            let kept_keys = k
                .index_select(&flat, 0)?
                .reshape((rows, kept, head_width))?;
            let bits = hamming_bits(&q.unsqueeze(1)?, &kept_keys)?;
            let weights = margins
                .narrow(0, h, 1)?
                .reshape((1, 1))?
                .broadcast_sub(&bits)?
                .relu()?
                .affine(1.0 / HAMMING_KEPT as f64, 0.0)?
                .mul(&real.to_dtype(DType::F32)?)?;
            let sources = if h % 2 == 1 {
                flat.broadcast_add(&one)?
            } else {
                flat
            };
            let payload = value
                .narrow(1, h * head_width, head_width)?
                .contiguous()?
                .index_select(&sources, 0)?
                .reshape((rows, kept, head_width))?;
            heads.push(payload.broadcast_mul(&weights.unsqueeze(2)?)?.sum(1)?);
        }
        let consensus = Tensor::cat(&heads, 1)?;
        let added = consensus.broadcast_mul(&self.weight.as_tensor().unsqueeze(0)?)?;
        Ok(points.add(&added)?.reshape((batch, time, width))?)
    }
}

/// The Hamming distance between ±1 codes, `(a − b)² / 4` summed over the last dimension: the
/// popcount of the XOR forward. Its gradient is zero on agreeing bits and pulls a disagreeing bit
/// toward the other code.
pub(crate) fn hamming_bits(a: &Tensor, b: &Tensor) -> Result<Tensor> {
    let last = b.rank() - 1;
    Ok(a.broadcast_sub(b)?.sqr()?.sum(last)?.affine(0.25, 0.0)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packed_sign_codes_pack_one_bit_per_coordinate_in_order() {
        let mut coordinates = vec![-1.0f32; 70];
        coordinates[0] = 0.5;
        coordinates[63] = 0.0;
        coordinates[65] = 2.0;
        assert_eq!(pack_signs(&coordinates), vec![1 | (1 << 63), 1 << 1]);
    }

    #[cfg(feature = "cuda")]
    #[test]
    fn the_forward_distance_is_the_popcount_of_the_xor_of_the_sign_codes() {
        let device = Device::new_cuda(0).unwrap();
        // Two 64-coordinate points with no zero coordinate.
        let a: Vec<f32> = (0..64).map(|i| ((i * 37 + 11) % 19) as f32 - 9.5).collect();
        let b: Vec<f32> = (0..64).map(|i| ((i * 53 + 7) % 23) as f32 - 11.5).collect();
        let code = |v: &[f32]| {
            Tensor::from_vec(v.to_vec(), (1, 64), &device)
                .unwrap()
                .sign()
                .unwrap()
        };
        let bits = hamming_bits(&code(&a), &code(&b))
            .unwrap()
            .to_vec1::<f32>()
            .unwrap()[0];
        let popcount = (pack_signs(&a)[0] ^ pack_signs(&b)[0]).count_ones();
        assert_eq!(bits, popcount as f32);
        assert!(popcount > 0 && popcount < 64);
    }

    #[cfg(feature = "cuda")]
    fn residual(batch: usize, time: usize, width: usize, device: &Device) -> Tensor {
        let values: Vec<f32> = (0..batch * time * width)
            .map(|i| (((i * 2_654_435_761) % 1_000) as f32 / 500.0) - 0.999)
            .collect();
        Tensor::from_vec(values, (batch, time, width), device).unwrap()
    }

    #[cfg(feature = "cuda")]
    #[test]
    fn a_zero_weight_leaves_the_residual_unchanged() {
        let device = Device::new_cuda(0).unwrap();
        let reads = HammingReads::new(4, 256, &device).unwrap();
        let x = residual(2, 24, 256, &device);
        let out = reads.reads[0].apply(&x).unwrap();
        assert_eq!(
            out.flatten_all().unwrap().to_vec1::<f32>().unwrap(),
            x.flatten_all().unwrap().to_vec1::<f32>().unwrap()
        );
    }

    #[cfg(feature = "cuda")]
    #[test]
    fn each_row_reads_only_earlier_rows_of_its_own_window() {
        let device = Device::new_cuda(0).unwrap();
        let reads = HammingReads::new(4, 256, &device).unwrap();
        let read = &reads.reads[0];
        read.weight
            .set(&Tensor::ones(256, DType::F32, &device).unwrap())
            .unwrap();
        // Every candidate inside the margin: 64 bits is the most a head can differ.
        read.margin
            .set(&Tensor::full(65.0f32, 4, &device).unwrap())
            .unwrap();
        // Two windows of two positions. Window 1 is window 0 scaled by 3, so its codes are the
        // same and every weight matches. Row 1 of each window may keep only row 0 of its own.
        let rows = [[1.0f32; 256], [-1.0; 256], [3.0; 256], [-3.0; 256]];
        let flat: Vec<f32> = rows.iter().flatten().copied().collect();
        let x = Tensor::from_vec(flat, (2, 2, 256), &device).unwrap();
        let added = read
            .apply(&x)
            .unwrap()
            .sub(&x)
            .unwrap()
            .reshape((4, 256))
            .unwrap()
            .to_vec2::<f32>()
            .unwrap();
        // Head 0 (coordinates 0..64) carries the kept row's value: row 0 (+1), not row 1 itself
        // (−1). Head 1 (64..128) carries the kept row's successor, row 1 (−1).
        assert!(
            added[1][0] > 0.0 && added[1][64] < 0.0,
            "{:?}",
            &added[1][..2]
        );
        // Window 1 reads its own row 0 (+3), not window 0's (+1): three times window 0's read.
        for c in [0usize, 64, 128, 192] {
            assert!(
                (added[3][c] - 3.0 * added[1][c]).abs() < 1e-4,
                "coordinate {c}"
            );
        }
        // Row 0 of each window has no earlier row and reads nothing.
        assert!(added[0].iter().chain(&added[2]).all(|&v| v == 0.0));
    }

    #[cfg(feature = "cuda")]
    #[test]
    fn no_position_reads_a_later_one_and_a_zero_margin_reads_nothing() {
        let device = Device::new_cuda(0).unwrap();
        let reads = HammingReads::new(4, 256, &device).unwrap();
        let read = &reads.reads[0];
        read.weight
            .set(&Tensor::ones(256, DType::F32, &device).unwrap())
            .unwrap();
        let x = residual(1, 24, 256, &device);
        let base = read
            .apply(&x)
            .unwrap()
            .squeeze(0)
            .unwrap()
            .to_vec2::<f32>()
            .unwrap();
        // Change only the last position: every earlier output must stay exactly as it was.
        let last = Tensor::ones((1, 1, 256), DType::F32, &device).unwrap();
        let changed = Tensor::cat(&[&x.narrow(1, 0, 23).unwrap(), &last], 1).unwrap();
        let moved = read
            .apply(&changed)
            .unwrap()
            .squeeze(0)
            .unwrap()
            .to_vec2::<f32>()
            .unwrap();
        assert_eq!(&moved[..23], &base[..23]);
        assert_ne!(moved[23], base[23]);
        // A margin of 0 bits weighs every candidate at 0: the read adds nothing.
        read.margin
            .set(&Tensor::zeros(4, DType::F32, &device).unwrap())
            .unwrap();
        let none = read.apply(&x).unwrap();
        assert_eq!(
            none.flatten_all().unwrap().to_vec1::<f32>().unwrap(),
            x.flatten_all().unwrap().to_vec1::<f32>().unwrap()
        );
    }

    #[cfg(feature = "cuda")]
    #[test]
    fn a_helpful_payload_raises_the_margin_under_descent() {
        let device = Device::new_cuda(0).unwrap();
        let reads = HammingReads::new(4, 256, &device).unwrap();
        let read = &reads.reads[0];
        read.weight
            .set(&Tensor::ones(256, DType::F32, &device).unwrap())
            .unwrap();
        // A residual of all-positive points: every value is positive, so a loss that rewards the
        // sum of the read's output rewards every payload it brings.
        let x = residual(1, 24, 256, &device)
            .abs()
            .unwrap()
            .affine(1.0, 0.1)
            .unwrap();
        let out = read.apply(&x).unwrap();
        let loss = out.sum_all().unwrap().neg().unwrap();
        let grads = loss.backward().unwrap();
        let margin = grads
            .get(read.margin.as_tensor())
            .unwrap()
            .to_vec1::<f32>()
            .unwrap();
        // A wider margin weighs the helpful payloads more, so descent raises every head's margin.
        assert!(margin.iter().all(|&g| g < 0.0), "{margin:?}");
    }

    #[cfg(feature = "cuda")]
    #[test]
    fn the_trunk_with_zero_weight_reads_is_unchanged_and_moved_reads_change_it() {
        let device = Device::new_cuda(0).unwrap();
        let model = crate::spiralcore_fbs_sheet::build_fbs_model(
            crate::spiralcore_fbs_arms::FbsArm::OneAtomActive,
            1,
            &device,
        )
        .unwrap();
        let ids: Vec<u32> = (0..32u32).map(|i| (i * 37 + 5) % 4096).collect();
        let hidden = |m: &crate::geometric_stack::StackModel| {
            crate::hamilton_sheet::with_gate_mode(
                crate::hamilton_sheet::HamiltonArm::HamiltonActive,
                1,
                0,
                false,
                || m.hidden(&ids, 1, 32),
            )
            .unwrap()
            .flatten_all()
            .unwrap()
            .to_vec1::<f32>()
            .unwrap()
        };
        let plain = hidden(&model);
        let reads = HammingReads::new(4, crate::hamilton_sheet::SHEET_WIDTH, &device).unwrap();
        set_hamming_reads(Some(reads.clone()));
        let zero = hidden(&model);
        for read in &reads.reads {
            read.weight
                .set(
                    &Tensor::ones(crate::hamilton_sheet::SHEET_WIDTH, DType::F32, &device).unwrap(),
                )
                .unwrap();
        }
        let moved = hidden(&model);
        set_hamming_reads(None);
        assert_eq!(zero, plain);
        assert_ne!(moved, plain);
    }
}
