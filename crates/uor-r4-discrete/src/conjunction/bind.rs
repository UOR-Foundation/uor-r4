//! The octonion bind of two tokens: the product of their codebook blocks as
//! integral octonions (the Cayley-Dickson step above the quaternions), one
//! block at a time. The product is undone by the conjugate, so the bound
//! address still holds both tokens. Integer arithmetic only.
//!
//! The table is the canonical Fano orientation of
//! `uor-r4-core::spiralcore_operator::OCTONION_FANO_CYCLES`.

use super::invalid;
use crate::Result;
use std::collections::{HashMap, HashSet};

/// `e_a e_b = e_c` for each cyclic turn of a line `[a, b, c]`.
pub const FANO_CYCLES: [[usize; 3]; 7] = [
    [1, 2, 4],
    [2, 3, 5],
    [3, 4, 6],
    [4, 5, 7],
    [5, 6, 1],
    [6, 7, 2],
    [7, 1, 3],
];

/// The basis product `e_left e_right`: its sign and its unit.
fn unit(left: usize, right: usize) -> (i64, usize) {
    if left == 0 {
        return (1, right);
    }
    if right == 0 {
        return (1, left);
    }
    if left == right {
        return (-1, 0);
    }
    for [a, b, c] in FANO_CYCLES {
        for [x, y, z] in [[a, b, c], [b, c, a], [c, a, b]] {
            if left == x && right == y {
                return (1, z);
            }
            if left == y && right == x {
                return (-1, z);
            }
        }
    }
    unreachable!("every pair of imaginary units lies on one Fano line")
}

/// The octonion product `a b`.
pub fn product(a: &[i64; 8], b: &[i64; 8]) -> [i64; 8] {
    let mut out = [0i64; 8];
    for (left, &x) in a.iter().enumerate() {
        if x == 0 {
            continue;
        }
        for (right, &y) in b.iter().enumerate() {
            if y == 0 {
                continue;
            }
            let (sign, at) = unit(left, right);
            out[at] += sign * x * y;
        }
    }
    out
}

/// The conjugate: the real part kept, the seven others negated.
pub fn conjugate(a: &[i64; 8]) -> [i64; 8] {
    let mut out = a.map(|x| -x);
    out[0] = a[0];
    out
}

pub fn norm(a: &[i64; 8]) -> i64 {
    a.iter().map(|x| x * x).sum()
}

/// What the bind of every ordered token pair looks like over one codebook.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct Census {
    pub tokens: u64,
    pub blocks: u64,
    /// Distinct block vectors, and distinct products of two of them.
    pub block_vectors: u64,
    pub block_products: u64,
    /// Products of two block vectors that are a block vector again, up to
    /// the scale of the norm, of all ordered pairs of block vectors.
    pub products_on_a_block_vector: u64,
    pub block_vector_pairs: u64,
    /// Ordered token pairs, and their distinct bound addresses.
    pub pairs: u64,
    pub addresses: u64,
    /// Pairs that share their address with an earlier pair.
    pub collisions: u64,
    /// Unbinding checked on every pair of block vectors:
    /// `conjugate(a) (a b) = norm(a) b`.
    pub unbinds: bool,
}

/// Binds every ordered pair of tokens of a codebook table (`blocks` blocks
/// of eight coordinates a token) and counts the distinct addresses.
pub fn census(table: &[i32], vocabulary: usize, blocks: usize) -> Result<Census> {
    if blocks == 0 || blocks > 4 || vocabulary == 0 || table.len() != vocabulary * blocks * 8 {
        return Err(invalid(
            "the codebook table is not vocabulary x blocks (at most 4) x 8",
        ));
    }
    let mut ids: HashMap<[i64; 8], usize> = HashMap::new();
    let mut vectors: Vec<[i64; 8]> = Vec::new();
    let mut tokens: Vec<[usize; 4]> = Vec::with_capacity(vocabulary);
    for token in table.chunks_exact(blocks * 8) {
        let mut own = [0usize; 4];
        for (block, coords) in token.chunks_exact(8).enumerate() {
            let vector: [i64; 8] = std::array::from_fn(|place| i64::from(coords[place]));
            own[block] = *ids.entry(vector).or_insert_with(|| {
                vectors.push(vector);
                vectors.len() - 1
            });
        }
        tokens.push(own);
    }
    let count = vectors.len();
    if count > 256 {
        return Err(invalid("more than 256 distinct block vectors"));
    }
    // Every product of two block vectors, numbered by its value.
    let mut product_ids: HashMap<[i64; 8], u16> = HashMap::new();
    let mut of = vec![0u16; count * count];
    let mut on_a_vector = 0u64;
    let mut unbinds = true;
    for (a, left) in vectors.iter().enumerate() {
        for (b, right) in vectors.iter().enumerate() {
            let bound = product(left, right);
            let scale = norm(left);
            unbinds &= product(&conjugate(left), &bound) == right.map(|x| x * scale);
            // `a b` has norm `norm(a) norm(b)`; a block vector of norm
            // `norm(b)` scaled by a whole root of `norm(a)` would equal it.
            on_a_vector += u64::from(vectors.iter().any(|vector| {
                (1..=scale).any(|by| by * by == scale && vector.map(|x| x * by) == bound)
            }));
            let next = product_ids.len() as u16;
            of[a * count + b] = *product_ids.entry(bound).or_insert(next);
        }
    }
    if product_ids.len() > 1 << 16 {
        return Err(invalid("more than 65536 distinct block products"));
    }
    let mut addresses: HashSet<u64> = HashSet::with_capacity(vocabulary * vocabulary);
    for left in &tokens {
        for right in &tokens {
            let mut address = 0u64;
            for block in 0..blocks {
                address = (address << 16) | u64::from(of[left[block] * count + right[block]]);
            }
            addresses.insert(address);
        }
    }
    let pairs = (vocabulary * vocabulary) as u64;
    Ok(Census {
        tokens: vocabulary as u64,
        blocks: blocks as u64,
        block_vectors: count as u64,
        block_products: product_ids.len() as u64,
        products_on_a_block_vector: on_a_vector,
        block_vector_pairs: (count * count) as u64,
        pairs,
        addresses: addresses.len() as u64,
        collisions: pairs - addresses.len() as u64,
        unbinds,
    })
}

/// The bind as a table: each token's four block vectors by number (four
/// bits each) and the number of every product of two block vectors. The
/// bound address of two tokens is their four block products.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BindTable {
    pub words: Vec<u16>,
    pub count: usize,
    pub of: Vec<u16>,
}

impl BindTable {
    /// From a codebook table of four blocks a token with at most sixteen
    /// distinct block vectors, numbered as they first appear.
    pub fn from_codebook(table: &[i32], vocabulary: usize, blocks: usize) -> Result<Self> {
        if blocks != 4 || vocabulary == 0 || table.len() != vocabulary * 32 {
            return Err(invalid("the bind table needs vocabulary x 4 blocks x 8"));
        }
        let mut ids: HashMap<[i64; 8], usize> = HashMap::new();
        let mut vectors: Vec<[i64; 8]> = Vec::new();
        let mut words = Vec::with_capacity(vocabulary);
        for token in table.chunks_exact(32) {
            let mut word = 0u16;
            for coords in token.chunks_exact(8) {
                let vector: [i64; 8] = std::array::from_fn(|place| i64::from(coords[place]));
                let id = *ids.entry(vector).or_insert_with(|| {
                    vectors.push(vector);
                    vectors.len() - 1
                });
                if id >= 16 {
                    return Err(invalid("more than sixteen distinct block vectors"));
                }
                word = (word << 4) | id as u16;
            }
            words.push(word);
        }
        let count = vectors.len();
        let mut product_ids: HashMap<[i64; 8], u16> = HashMap::new();
        let mut of = Vec::with_capacity(count * count);
        for left in &vectors {
            for right in &vectors {
                let next = product_ids.len() as u16;
                of.push(*product_ids.entry(product(left, right)).or_insert(next));
            }
        }
        Ok(Self { words, count, of })
    }
    /// Checks a table read from a file.
    pub fn checked(self) -> Result<Self> {
        let small =
            |word: &u16| (0..4).all(|block| usize::from((word >> (4 * block)) & 15) < self.count);
        if self.count > 16
            || self.of.len() != self.count * self.count
            || !self.words.iter().all(small)
        {
            return Err(invalid("the bind table does not agree with itself"));
        }
        Ok(self)
    }
    /// The bound address of `left` then `right`; `None` past the table.
    pub fn address(&self, left: u32, right: u32) -> Option<u64> {
        let a = *self.words.get(left as usize)?;
        let b = *self.words.get(right as usize)?;
        let mut address = 0u64;
        for block in (0..4).rev() {
            let x = usize::from((a >> (4 * block)) & 15);
            let y = usize::from((b >> (4 * block)) & 15);
            address = (address << 16) | u64::from(self.of[x * self.count + y]);
        }
        Some(address)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn basis(at: usize) -> [i64; 8] {
        let mut out = [0i64; 8];
        out[at] = 1;
        out
    }

    fn sum(a: &[i64; 8], b: &[i64; 8]) -> [i64; 8] {
        std::array::from_fn(|place| a[place] + b[place])
    }

    /// The case the draft table of 9 October failed: `x (x y) = (x x) y`
    /// for `x = e1 + e2`, `y = e4`.
    #[test]
    fn the_product_is_alternative_and_keeps_the_norm() {
        let x = sum(&basis(1), &basis(2));
        let y = basis(4);
        assert_eq!(product(&x, &product(&x, &y)), product(&product(&x, &x), &y));
        let a = [1, -2, 0, 3, 1, 0, -1, 2];
        let b = [2, 0, 1, -1, 0, 4, 1, -3];
        assert_eq!(product(&a, &product(&a, &b)), product(&product(&a, &a), &b));
        assert_eq!(product(&product(&a, &b), &b), product(&a, &product(&b, &b)));
        assert_eq!(norm(&product(&a, &b)), norm(&a) * norm(&b));
        // Not associative: the order of binding is part of the address.
        let c = basis(3);
        assert_ne!(
            product(&product(&basis(1), &basis(2)), &c),
            product(&basis(1), &product(&basis(2), &c))
        );
    }

    #[test]
    fn the_conjugate_undoes_the_bind() {
        let a = [0, 2, -2, 0, 0, 0, 0, 0];
        let b = [0, 0, 0, 2, 0, 0, 2, 0];
        let bound = product(&a, &b);
        assert_eq!(product(&conjugate(&a), &bound), b.map(|x| x * norm(&a)));
        assert_eq!(product(&bound, &conjugate(&b)), a.map(|x| x * norm(&b)));
    }

    /// Two tokens that differ in one block bind to different addresses with
    /// any partner, and a census counts every ordered pair once.
    #[test]
    fn a_census_counts_each_pair_of_a_small_codebook() -> Result<()> {
        let rows: [[i32; 8]; 4] = [
            [2, 2, 0, 0, 0, 0, 0, 0],
            [0, 2, -2, 0, 0, 0, 0, 0],
            [0, 0, 0, 2, 0, 0, 2, 0],
            [2, 0, 0, 0, 0, -2, 0, 0],
        ];
        let table: Vec<i32> = rows.iter().flatten().copied().collect();
        let counted = census(&table, 4, 1)?;
        assert_eq!(counted.pairs, 16);
        assert_eq!(counted.addresses + counted.collisions, 16);
        assert!(counted.unbinds);
        assert!(census(&table, 3, 1).is_err());
        // The bind table: with one partner fixed, two tokens never share an
        // address (the product can be divided out).
        let four: Vec<i32> = rows
            .iter()
            .flat_map(|row| [row, &rows[0], &rows[1], row])
            .flatten()
            .copied()
            .collect();
        let bound = BindTable::from_codebook(&four, 4, 4)?.checked()?;
        for partner in 0..4u32 {
            let seen: HashSet<_> = (0..4u32)
                .map(|token| bound.address(token, partner))
                .collect();
            assert_eq!(seen.len(), 4);
        }
        assert_eq!(bound.address(4, 0), None);
        Ok(())
    }
}
