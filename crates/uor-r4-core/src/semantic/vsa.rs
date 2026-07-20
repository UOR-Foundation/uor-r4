use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Hypervector(pub [u64; 16]); // 16 * 64 bits = 1024 dimensions

impl Hypervector {
    pub fn zero() -> Self {
        Self([0u64; 16])
    }

    /// bind(A, B) = A ^ B (XOR is its own inverse for binary hypervectors)
    pub fn bind(&self, other: &Self) -> Self {
        let mut res = [0u64; 16];
        for i in 0..16 {
            res[i] = self.0[i] ^ other.0[i];
        }
        Self(res)
    }

    /// unbind(A, AxB) = B (same as bind)
    pub fn unbind(&self, other: &Self) -> Self {
        self.bind(other)
    }

    /// bundle(Vec<Hypervector>): Component-wise majority aggregation.
    pub fn bundle(vectors: &[Self]) -> Self {
        if vectors.is_empty() {
            return Self::zero();
        }
        let mut res = [0u64; 16];
        let half = vectors.len() / 2;
        // Count 1s at each bit position
        for i in 0..16 {
            let mut accumulated = 0u64;
            for bit in 0..64 {
                let mut ones = 0;
                for v in vectors {
                    if (v.0[i] & (1u64 << bit)) != 0 {
                        ones += 1;
                    }
                }
                if ones > half {
                    accumulated |= 1u64 << bit;
                }
            }
            res[i] = accumulated;
        }
        Self(res)
    }

    /// permute(A, shift): Circular bit shift of the entire 1024-bit vector
    pub fn permute(&self, shift: usize) -> Self {
        let shift = shift % 1024;
        if shift == 0 {
            return *self;
        }
        let word_shift = shift / 64;
        let bit_shift = shift % 64;

        let mut res = [0u64; 16];
        for i in 0..16 {
            // Calculate destination index in circular buffer
            let dest = (i + word_shift) % 16;
            res[dest] = self.0[i];
        }

        if bit_shift > 0 {
            let mut carry = 0u64;
            for i in 0..16 {
                let next_carry = res[i] >> (64 - bit_shift);
                res[i] = (res[i] << bit_shift) | carry;
                carry = next_carry;
            }
            res[0] |= carry;
        }
        Self(res)
    }

    /// similarity(A, B): Normalized Hamming similarity
    pub fn similarity(&self, other: &Self) -> f32 {
        let mut matches = 0;
        for i in 0..16 {
            let matching_bits = !(self.0[i] ^ other.0[i]);
            matches += matching_bits.count_ones() as usize;
        }
        matches as f32 / 1024.0
    }
}

/// Deterministically expand a CID into a pseudo-orthogonal 1024-bit hypervector
pub fn expand_atom(prefix: &str, cid: &str, space: &str) -> Hypervector {
    let mut hasher = blake3::Hasher::new();
    hasher.update(prefix.as_bytes());
    hasher.update(cid.as_bytes());
    hasher.update(space.as_bytes());
    let hash = hasher.finalize();

    let seed_bytes = hash.as_bytes();
    let mut state = [
        u64::from_le_bytes(seed_bytes[0..8].try_into().unwrap()),
        u64::from_le_bytes(seed_bytes[8..16].try_into().unwrap()),
        u64::from_le_bytes(seed_bytes[16..24].try_into().unwrap()),
        u64::from_le_bytes(seed_bytes[24..32].try_into().unwrap()),
    ];

    let mut data = [0u64; 16];
    for val in &mut data {
        // xorshift128plus step
        let mut x = state[0];
        let y = state[1];
        state[0] = y;
        x ^= x << 23;
        state[1] = x ^ y ^ (x >> 17) ^ (y >> 26);
        *val = state[1].wrapping_add(y);
    }
    Hypervector(data)
}
