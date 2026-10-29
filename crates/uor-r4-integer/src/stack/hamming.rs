//! claude-146's induction-keyed Hamming reads in the integer artifact (claude-188): their
//! parameters as a `uor-r4.lut-stack/6` container stores them, one read per depth.
//!
//! Serving the reads is a later unit of `integer-hamming-read`. Until then
//! [`super::IntegerStackModel::parse`] refuses such a container, so a read is never dropped.

use super::format::{Container, StackHamming};
use super::kernels::{stack_div_u64, stack_hamilton};
use super::session::{stack_rotation, GATE_EPSILON};
use super::StackError;

/// One Hamming read's integers, at the exponents its [`StackHamming`] record names.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HammingReadImage {
    /// After this many layers, or on the readout at the layer count.
    pub depth: usize,
    /// `width / 4` raw quaternions `[w, x, y, z]` at `2^rotation_exp`: the type heads' query.
    pub query_rotation: Vec<i32>,
    /// `width / 4` raw quaternions: the key, which the copy heads' queries share.
    pub key_rotation: Vec<i32>,
    /// `width / 4` raw quaternions: the type heads' value.
    pub value_rotation: Vec<i32>,
    /// `heads` margins in bits at `2^margin_exp`, each positive.
    pub margin: Vec<i32>,
    /// `2 * width` at `2^weight_exp`: row 0 the copy heads' weights, row 1 the type heads'.
    pub weight: Vec<i32>,
}

/// A stack's Hamming reads: the header record and one image per depth, in depth order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HammingImage {
    pub record: StackHamming,
    pub reads: Vec<HammingReadImage>,
}

/// The Hamming reads of a `UORLUT01` stack artifact, with every section at the length its shape
/// and record imply, or `None` for a stack without them.
pub fn read_hamming_uorlut01(bytes: &[u8]) -> Result<Option<HammingImage>, StackError> {
    hamming_image(&Container::parse(bytes)?)
}

/// The Hamming reads of a parsed container, as [`read_hamming_uorlut01`].
pub(crate) fn hamming_image(artifact: &Container<'_>) -> Result<Option<HammingImage>, StackError> {
    let Some(record) = artifact.hamming.clone() else {
        return Ok(None);
    };
    let width = artifact.shape.width;
    let mut reads = Vec::with_capacity(record.depths.len());
    for &depth in &record.depths {
        let section = |part: &str, len: usize| -> Result<Vec<i32>, StackError> {
            let name = format!("hamming.d{depth}.{part}");
            let values = artifact.table_i32(&name)?;
            if values.len() != len {
                return Err(StackError::TableLength {
                    name,
                    len: values.len(),
                    expected: len,
                });
            }
            Ok(values)
        };
        let margin = section("margin", record.heads)?;
        if margin.iter().any(|&m| m <= 0) {
            return Err(StackError::Numerics(format!(
                "hamming.d{depth}.margin: a margin is not positive"
            )));
        }
        reads.push(HammingReadImage {
            depth,
            query_rotation: section("query_rotation", width)?,
            key_rotation: section("key_rotation", width)?,
            value_rotation: section("value_rotation", width)?,
            margin,
            weight: section("weight", 2 * width)?,
        });
    }
    Ok(Some(HammingImage { record, reads }))
}

/// The exponent of a copy mean: a Q12 table point times a Q31 weight.
pub const COPY_MEAN_EXP: i32 = -43;

/// The weight of a candidate at `distance` bits under a head's margin `margin_q16` (bits at
/// Q16), at Q31: `max(0, m − d) / max(m, 1)`, as the float read's
/// `margin.broadcast_sub(bits).relu().broadcast_div(margin.maximum(1.0))` (claude-189).
pub fn copy_weight_q31(margin_q16: i32, distance: u32) -> u64 {
    let margin = i64::from(margin_q16);
    let over = margin - (i64::from(distance) << 16);
    if over <= 0 {
        return 0;
    }
    // `over <= margin < 2^31`, so the shifted numerator stays below 2^62.
    stack_div_u64((over as u64) << 31, margin.max(1 << 16) as u64)
}

/// One read's copy heads, built for serving (claude-189): every vocabulary entry's sign code
/// for each copy head, and the copy heads' margins.
///
/// A copy head's query at `t` is the code of `x_t`'s table point through the key rotation, and
/// its key at `s` is the code of `x_{s−1}`'s point through the same rotation, so every code is
/// a vocabulary entry's. The rotation is the unit `g / |g|` of each raw key quaternion, as
/// `stack_rotation` normalizes the unsnapped gates, applied by `stack_hamilton` on the left,
/// as the float `hamilton_gate`. A bit is 1 at or above 0, 64 coordinates to a word in
/// coordinate order, as the float `pack_signs`.
pub struct CopyHeads {
    copy: usize,
    kept: usize,
    width: usize,
    /// `vocab * copy` codes: entry `v`'s code for copy head `h` is at `v * copy + h`.
    codes: Vec<u64>,
    /// The copy heads' margins, in bits at Q16.
    margins: Vec<i32>,
}

impl CopyHeads {
    /// The copy heads of `read`, a read of a stack with `record`, over the Q12 `table` of
    /// `vocab` rows of `width`.
    pub fn new(record: &StackHamming, read: &HammingReadImage, table: &[i16], vocab: usize, width: usize) -> Self {
        let copy = record.heads / 2;
        let head_width = width / record.heads;
        let units: Vec<[i64; 4]> = read
            .key_rotation
            .chunks_exact(4)
            .map(|g| stack_rotation([g[0], g[1], g[2], g[3]], 1u64 << 31, GATE_EPSILON))
            .collect();
        let mut codes = vec![0u64; vocab * copy];
        for (v, row) in table.chunks_exact(width).take(vocab).enumerate() {
            for h in 0..copy {
                let mut word = 0u64;
                for c in (0..head_width).step_by(4) {
                    let at = h * head_width + c;
                    let point = [
                        i64::from(row[at]),
                        i64::from(row[at + 1]),
                        i64::from(row[at + 2]),
                        i64::from(row[at + 3]),
                    ];
                    let rotated = stack_hamilton(units[at / 4], point);
                    for (i, &value) in rotated.iter().enumerate() {
                        if value >= 0 {
                            word |= 1u64 << (c + i);
                        }
                    }
                }
                codes[v * copy + h] = word;
            }
        }
        Self {
            copy,
            kept: record.kept,
            width,
            codes,
            margins: read.margin[..copy].to_vec(),
        }
    }

    /// Copy head `h`'s code for vocabulary entry `token`.
    pub fn code(&self, token: u32, h: usize) -> u64 {
        self.codes[token as usize * self.copy + h]
    }

    /// Copy head `h`'s kept positions at the last row `t` of `tokens` (`x_0..=x_t`): the
    /// `min(kept, t − 1)` positions `1 ≤ s < t` nearest by `popcount(q XOR k_s)`, with their
    /// distances, in ascending distance and then ascending `s` (the design's tie rule; the
    /// float `choose_sorted` leaves the order of ties open). Position 0 has no previous token.
    pub fn choose(&self, h: usize, tokens: &[u32]) -> Vec<(usize, u32)> {
        let t = tokens.len().saturating_sub(1);
        let query = self.code(tokens[t], h);
        let mut candidates: Vec<(u32, usize)> = (1..t)
            .map(|s| ((query ^ self.code(tokens[s - 1], h)).count_ones(), s))
            .collect();
        candidates.sort_unstable();
        candidates.truncate(self.kept);
        candidates.into_iter().map(|(d, s)| (s, d)).collect()
    }

    /// The copy heads' mean at the last row of `tokens`, at [`COPY_MEAN_EXP`], or `None` when no
    /// copy head kept a candidate inside its margin.
    ///
    /// Each head's mean is `Σ w_s · table[x_s] / max(1, #{w_s > 0})` over its kept positions:
    /// the position after the earlier match brings back what followed it, as the very point the
    /// nearest readout names it by. The copy mean is the mean of the head means over the heads
    /// that found a candidate, so a head that found nothing does not dilute one that did. Both
    /// quotients truncate toward zero.
    pub fn mean(&self, tokens: &[u32], table: &[i16]) -> Option<Vec<i128>> {
        let width = self.width;
        let mut sum = vec![0i128; width];
        let mut found = 0i128;
        for h in 0..self.copy {
            let mut head = vec![0i128; width];
            let mut count = 0i128;
            for (s, d) in self.choose(h, tokens) {
                let weight = copy_weight_q31(self.margins[h], d);
                if weight == 0 {
                    continue;
                }
                count += 1;
                let row = &table[tokens[s] as usize * width..][..width];
                for (acc, &p) in head.iter_mut().zip(row) {
                    *acc += i128::from(p) * i128::from(weight);
                }
            }
            if count == 0 {
                continue;
            }
            found += 1;
            for (acc, value) in sum.iter_mut().zip(head) {
                *acc += value / count;
            }
        }
        (found > 0).then(|| sum.into_iter().map(|value| value / found).collect())
    }
}
