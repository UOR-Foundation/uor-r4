//! Native Q31 vector transport shared by the first integer CUDA training kernel
//! and its Rust oracle.
//!
//! The state is four signed 64-bit quaternion words at Q31, with each admitted
//! coordinate in the signed 32-bit range including +2^31. Products are
//! sign-extended 64-bit integer multiplies of those admitted coordinates,
//! accumulated modulo 2^64, rounded by 2^30, then shifted right by 31 and
//! saturated to the same closed domain. This module defines the reference
//! words; the AOT CUDA module must return the same words.

use crate::{Error, Result};

/// One Q31 quaternion: `(w, x, y, z)`.
pub type Q31Quaternion = [i64; 4];

/// One Q31 quaternion after Hamilton transport.
pub type Q31Transport = [i64; 4];

const Q31_MIN: i64 = -(1 << 31);
const Q31_MAX: i64 = 1 << 31;
/// Largest admitted batch: the last `item * 4` index remains in the lower 32 bits.
pub const MAX_Q31_BATCH_ITEMS: usize = 1 << 30;

/// Whether one word is inside the closed native Q31 domain.
pub fn q31_coordinate_admitted(word: i64) -> bool {
    (Q31_MIN..=Q31_MAX).contains(&word)
}

/// Whether every word is inside the closed native Q31 domain.
pub fn q31_quaternion_admitted(value: Q31Quaternion) -> bool {
    value.into_iter().all(q31_coordinate_admitted)
}

/// Whether one batch fits the native launch-index contract.
pub fn q31_batch_len_admitted(items: usize) -> bool {
    items <= MAX_Q31_BATCH_ITEMS
}

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

    /// Restore raw Q31 words without scale conversion, refusing out-of-domain state.
    pub fn from_bytes(bytes: [u8; 32]) -> Result<Self> {
        let mut words = [0; 4];
        for (index, word) in words.iter_mut().enumerate() {
            *word = i64::from_le_bytes(
                bytes[index * 8..(index + 1) * 8]
                    .try_into()
                    .expect("fixed Q31 word width"),
            );
        }
        if !q31_quaternion_admitted(words) {
            return Err(Error::Invalid(
                "serialized Q31 words leave -2^31..=2^31".into(),
            ));
        }
        Ok(Self { words })
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
    (((word.wrapping_add(ROUND_Q30)) as i64) >> Q31_SHIFT).clamp(Q31_MIN, Q31_MAX)
}

/// Hamilton rotation `gate (x) value` under the closed native Q31 ABI.
///
/// Invalid state words are refused consistently on CPU and GPU. Output words
/// are saturated to the same range, so each result is admissible as the next
/// native state. No float, transcendental or normalization fallback exists.
pub fn q31_hamilton_rotate(gate: Q31Quaternion, value: Q31Quaternion) -> Result<Q31Transport> {
    if !q31_quaternion_admitted(gate) || !q31_quaternion_admitted(value) {
        return Err(Error::Invalid(
            "native Q31 coordinates must be in -2^31..=2^31".into(),
        ));
    }
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

    Ok([
        q31_word(p00.wrapping_sub(p11).wrapping_sub(p22).wrapping_sub(p33)),
        q31_word(p01.wrapping_add(p10).wrapping_add(p23).wrapping_sub(p32)),
        q31_word(p02.wrapping_sub(p13).wrapping_add(p20).wrapping_add(p31)),
        q31_word(p03.wrapping_add(p12).wrapping_sub(p21).wrapping_add(p30)),
    ])
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
            assert_eq!(q31_hamilton_rotate(identity, value).unwrap(), value);
        }
    }

    #[test]
    fn transport_signs_follow_the_hamilton_product() {
        assert_eq!(
            q31_hamilton_rotate([0, ONE, 0, 0], [0, 0, ONE, 0]).unwrap(),
            [0, 0, 0, ONE]
        );
        assert_eq!(
            q31_hamilton_rotate([0, 0, ONE, 0], [0, ONE, 0, 0]).unwrap(),
            [0, 0, 0, -ONE]
        );
    }

    #[test]
    fn serialization_preserves_every_raw_word() {
        let state = Q31VectorState {
            words: [1 << 31, -(1 << 31), -1, 0x0123_4567],
        };
        assert_eq!(Q31VectorState::from_bytes(state.to_bytes()).unwrap(), state);
    }

    #[test]
    fn out_of_domain_state_is_refused_consistently() {
        let invalid = [Q31_MAX + 1, 0, 0, 0];
        assert!(q31_hamilton_rotate(invalid, [ONE, 0, 0, 0]).is_err());
        assert!(q31_hamilton_rotate([ONE, 0, 0, 0], invalid).is_err());
        let state = Q31VectorState { words: invalid };
        assert!(Q31VectorState::from_bytes(state.to_bytes()).is_err());
    }

    #[test]
    fn batch_indices_are_bounded_before_launch() {
        assert!(q31_batch_len_admitted(MAX_Q31_BATCH_ITEMS));
        assert!(!q31_batch_len_admitted(MAX_Q31_BATCH_ITEMS + 1));
    }

    #[test]
    fn saturated_output_stays_in_the_next_state_domain() {
        let edge = [ONE, ONE, 0, 0];
        let output = q31_hamilton_rotate(edge, edge).unwrap();
        assert!(q31_quaternion_admitted(output));
    }
}
