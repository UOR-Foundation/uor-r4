//! ScoreQ: Quantized fixed-point log-domain score representation (Q16.16 in i32).
//!
//! The single canonical definition lives in `uor_r4_graph_format::ScoreQ`
//! (re-exported here for compatibility): Q16.16 in i32, multiplication-free
//! saturating add/sub arithmetic, replacing floating-point route scores in
//! deployed inference artifacts.

pub use uor_r4_graph_format::ScoreQ;

/// Dyadic storage descriptor for compact residual table decoding (runtime
/// decode side): `{ width_bits: 8|16|32, shift, zero_point }`.
///
/// NOTE: this is the runtime decode helper, distinct from the *wire*
/// `uor_r4_graph_format::StorageDescriptor` (4 bytes: width tag, shift,
/// i16 zero point) used in EMIT/EXCT sections. Phase 4 will map wire
/// descriptors onto this decode form when residual tables land.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StorageDescriptor {
    pub width_bits: u8,
    pub shift: i8,
    pub zero_point: i32,
}

impl StorageDescriptor {
    pub const fn new(width_bits: u8, shift: i8, zero_point: i32) -> Self {
        StorageDescriptor {
            width_bits,
            shift,
            zero_point,
        }
    }

    /// Decode raw integer entry into ScoreQ using shift + zero_point (mul-free).
    pub fn decode(&self, raw_entry: i32) -> ScoreQ {
        let centered = raw_entry.saturating_sub(self.zero_point);
        let raw_score = if self.shift >= 0 {
            centered.checked_shl(self.shift as u32).unwrap_or_else(|| {
                if centered.is_negative() {
                    i32::MIN
                } else {
                    i32::MAX
                }
            })
        } else {
            centered
                .checked_shr((-self.shift) as u32)
                .unwrap_or_else(|| if centered.is_negative() { -1 } else { 0 })
        };
        ScoreQ(raw_score)
    }
}
