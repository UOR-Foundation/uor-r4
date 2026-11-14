//! Native Q31 vector transport shared by the first integer CUDA training kernel
//! and its Rust oracle.
//!
//! The state is four signed 64-bit quaternion words at Q31, with each admitted
//! coordinate in the signed 32-bit range including +2^31. Products are
//! signed 32x32-to-64-bit integers, accumulated modulo 2^64, rounded by 2^30,
//! then shifted right by 31. This module defines the reference words; the AOT
//! CUDA module must return the same words.

/// One Q31 quaternion: `(w, x, y, z)`.
pub type Q31Quaternion = [i64; 4];

/// One Q31 quaternion after Hamilton transport.
pub type Q31Transport = [i64; 4];

/// The serialized word layout of one native vector carry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Q31VectorState {
    pub words: Q31Quaternion,
}

impl Q31VectorState {
    /// Lossless little-endian carrier: four consecutive signed 64-bit words.
    pub fn to_bytes(self) -> [u8; 32] {
        let mut bytes = [0; 32];
        for (index, word) in self.words.iter().enumerate() {
            bytes[index * 8..(index + 1) * 8].copy_from_slice(&word.to_le_bytes());
        }
        bytes
    }

    /// Restore raw Q31 words without scale conversion.
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        let mut words = [0; 4];
        for (index, word) in words.iter_mut().enumerate() {
            *word = i64::from_le_bytes(
                bytes[index * 8..(index + 1) * 8]
                    .try_into()
                    .expect("fixed Q31 word width"),
            );
        }
        Self { words }
    }
}

const ROUND_Q30: u64 = 1 << 30;
const Q31_SHIFT: u32 = 31;

#[inline]
fn product(a: i64, b: i64) -> u64 {
    a.wrapping_mul(b) as u64
}

#[inline]
fn q31_word(word: u64) -> i64 {
    ((word.wrapping_add(ROUND_Q30)) as i64) >> Q31_SHIFT
}

/// Exact Hamilton rotation `gate (x) value` under the native Q31 ABI.
///
/// The caller supplies already-normalized gate coordinates. No float,
/// transcendental or normalization fallback exists in this operation.
pub fn q31_hamilton_rotate(gate: Q31Quaternion, value: Q31Quaternion) -> Q31Transport {
    debug_assert!(gate
        .iter()
        .chain(&value)
        .all(|coordinate| coordinate.unsigned_abs() <= 1 << 31));
    let p00 = product(gate[0], value[0]);
    let p01 = product(gate[0], value[1]);
    let p02 = product(gate[0], value[2]);
    let p03 = product(gate[0], value[3]);
    let p10 = product(gate[1], value[0]);
    let p11 = product(gate[1], value[1]);
    let p12 = product(gate[1], value[2]);
    let p13 = product(gate[1], value[3]);
    let p20 = product(gate[2], value[0]);
    let p21 = product(gate[2], value[1]);
    let p22 = product(gate[2], value[2]);
    let p23 = product(gate[2], value[3]);
    let p30 = product(gate[3], value[0]);
    let p31 = product(gate[3], value[1]);
    let p32 = product(gate[3], value[2]);
    let p33 = product(gate[3], value[3]);

    [
        q31_word(p00.wrapping_sub(p11).wrapping_sub(p22).wrapping_sub(p33)),
        q31_word(p01.wrapping_add(p10).wrapping_add(p23).wrapping_sub(p32)),
        q31_word(p02.wrapping_sub(p13).wrapping_add(p20).wrapping_add(p31)),
        q31_word(p03.wrapping_add(p12).wrapping_sub(p21).wrapping_add(p30)),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    const ONE: i64 = 1 << 31;

    #[test]
    fn identity_preserves_each_axis() {
        let identity = [ONE, 0, 0, 0];
        for value in [
            [ONE, 0, 0, 0],
            [0, ONE, 0, 0],
            [0, 0, ONE, 0],
            [0, 0, 0, ONE],
        ] {
            assert_eq!(q31_hamilton_rotate(identity, value), value);
        }
    }

    #[test]
    fn transport_signs_follow_the_hamilton_product() {
        assert_eq!(
            q31_hamilton_rotate([0, ONE, 0, 0], [0, 0, ONE, 0]),
            [0, 0, 0, ONE]
        );
        assert_eq!(
            q31_hamilton_rotate([0, 0, ONE, 0], [0, ONE, 0, 0]),
            [0, 0, 0, -ONE]
        );
    }

    #[test]
    fn serialization_preserves_every_raw_word() {
        let state = Q31VectorState {
            words: [1 << 31, -(1 << 31), -1, 0x0123_4567_89ab_cdef],
        };
        assert_eq!(Q31VectorState::from_bytes(state.to_bytes()), state);
    }
}
