use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Hypervector(pub [u64; 16]); // 16 * 64 bits = 1024 dimensions

impl Hypervector {
    pub fn zero() -> Self {
        Self([0u64; 16])
    }

    /// bind(A, B) = A ^ B (XOR is its own inverse for binary hypervectors)
    pub fn bind(&self, other: &Self) -> Self {
        Self(std::array::from_fn(|i| self.0[i] ^ other.0[i]))
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
        for (i, word) in res.iter_mut().enumerate() {
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
            *word = accumulated;
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
            for word in res.iter_mut() {
                let next_carry = *word >> (64 - bit_shift);
                *word = (*word << bit_shift) | carry;
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

/// Encode a subject-predicate-object statement semantic triple
pub fn encode_statement(
    subject_cid: &str,
    predicate_cid: &str,
    object_cid: &str,
    space: &str,
) -> Hypervector {
    let r_subj = expand_atom("role", "subject", space);
    let r_pred = expand_atom("role", "predicate", space);
    let r_obj = expand_atom("role", "object", space);

    let h_subj = expand_atom("entity", subject_cid, space);
    let h_pred = expand_atom("relation", predicate_cid, space);
    let h_obj = expand_atom("entity", object_cid, space);

    let b_subj = r_subj.bind(&h_subj);
    let b_pred = r_pred.bind(&h_pred);
    let b_obj = r_obj.bind(&h_obj);

    Hypervector::bundle(&[b_subj, b_pred, b_obj])
}

/// Encode a temporal event linking subject, action, timestamp, and location
pub fn encode_event(
    subject_cid: &str,
    action_cid: &str,
    timestamp_cid: &str,
    location_cid: &str,
    space: &str,
) -> Hypervector {
    let r_subj = expand_atom("role", "subject", space);
    let r_act = expand_atom("role", "action", space);
    let r_time = expand_atom("role", "time", space);
    let r_loc = expand_atom("role", "location", space);

    let h_subj = expand_atom("entity", subject_cid, space);
    let h_act = expand_atom("relation", action_cid, space);
    let h_time = expand_atom("temporal", timestamp_cid, space);
    let h_loc = expand_atom("entity", location_cid, space);

    let b_subj = r_subj.bind(&h_subj);
    let b_act = r_act.bind(&h_act);
    let b_time = r_time.bind(&h_time);
    let b_loc = r_loc.bind(&h_loc);

    Hypervector::bundle(&[b_subj, b_act, b_time, b_loc])
}

/// Encode a directed target graph relationship
pub fn encode_graph_edge(
    source_cid: &str,
    relation_cid: &str,
    target_cid: &str,
    space: &str,
) -> Hypervector {
    let r_src = expand_atom("role", "source", space);
    let r_rel = expand_atom("role", "relation", space);
    let r_tgt = expand_atom("role", "target", space);

    let h_src = expand_atom("entity", source_cid, space);
    let h_rel = expand_atom("relation", relation_cid, space);
    let h_tgt = expand_atom("entity", target_cid, space);

    let b_src = r_src.bind(&h_src);
    let b_rel = r_rel.bind(&h_rel);
    // Target is permuted by 1 to enforce edge directionality
    let b_tgt = r_tgt.bind(&h_tgt).permute(1);

    Hypervector::bundle(&[b_src, b_rel, b_tgt])
}
