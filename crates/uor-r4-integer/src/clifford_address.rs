//! `clifford_address`: the address as a **Cayley–Dickson / Clifford binding**, not a vector sum.
//!
//! Why: the measured failure of the learned address was that `colour` and `city` are
//! distributionally identical ("what is X's ___"), so a sum of word vectors cannot separate them —
//! 29/40 on formation and 0/40 on unseen wording (`slot-embed-train`). The bounded algebra fixes that
//! **by construction**: each attribute is a distinct basis blade, and distinct blades are orthogonal
//! under the geometric product.
//!
//! The tower is the reason it works, and it is the project's own geometry:
//!
//! | level | dim | imaginary units |
//! |---|---|---|
//! | R | 1 | 0 |
//! | C | 2 | 1 |
//! | H | 4 | 3 |
//! | O | 8 | 7 |
//! | S | 16 | 15 |
//!
//! Fifteen is `CL06_BIVECTOR_COUNT` — the bivectors of Cl(0,6) — so the algebra gives **15 orthogonal
//! role axes**, one per attribute slot, with no learning and no collision. Order is free above the
//! quaternions: `entity * attribute` and `attribute * entity` differ, which is the role separation
//! the prime-domain scheme was trying to hand-build.
//!
//! Integer only: a blade product is a sign and a mask, and a full product is 64 x 64 sign-adds. No
//! matmul, no squashing, deterministic.
//!
//! Cl(0,6): six generators with `e_k^2 = -1`, 64 basis blades (bitmask over the six generators).

/// The coefficient count of Cl(0,6): 2^6 blades.
pub const DIM: usize = 64;

/// A basis blade: bit `k` set means generator `k` appears.
pub type Blade = u8;

/// The geometric product of two basis blades: `e_a e_b = sign * e_{a xor b}`, sign 0 never occurs.
///
/// Two effects contribute: the `-1` from each generator that appears in both (since `e_k^2 = -1`),
/// and the `-1` from each anti-commutation needed to move `b`'s generators past `a`'s.
pub fn blade_product(a: Blade, b: Blade) -> (i8, Blade) {
    let mut sign: i8 = 1;
    // Each shared generator squares to -1.
    if (a & b).count_ones() % 2 == 1 {
        sign = -sign;
    }
    // Count transpositions: for every generator in b, how many generators of a are larger than it.
    let mut swaps = 0u32;
    for j in 0..6u8 {
        if b & (1 << j) == 0 {
            continue;
        }
        swaps += (a >> (j + 1)).count_ones();
    }
    if swaps % 2 == 1 {
        sign = -sign;
    }
    (sign, a ^ b)
}

/// An element of Cl(0,6): one integer coefficient per basis blade.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Element(pub [i32; DIM]);

impl Element {
    pub fn zero() -> Self {
        Self([0; DIM])
    }

    pub fn blade(mask: Blade) -> Self {
        let mut out = [0; DIM];
        out[mask as usize] = 1;
        Self(out)
    }

    /// The geometric product: `sum over blades a, b of x_a y_b (sign e_{a xor b})`.
    pub fn mul(&self, other: &Self) -> Self {
        let mut out = [0i32; DIM];
        for (a, &xa) in self.0.iter().enumerate() {
            if xa == 0 {
                continue;
            }
            for (b, &yb) in other.0.iter().enumerate() {
                if yb == 0 {
                    continue;
                }
                let (sign, mask) = blade_product(a as Blade, b as Blade);
                out[mask as usize] += i32::from(sign) * xa * yb;
            }
        }
        Self(out)
    }

    /// The reverse: every basis blade keeps its sign when its grade is 0 or 1 mod 4, and flips when
    /// it is 2 or 3.
    pub fn reverse(&self) -> Self {
        let mut out = [0i32; DIM];
        for (i, &x) in self.0.iter().enumerate() {
            let grade = (i as u8).count_ones();
            out[i] = if matches!(grade % 4, 0 | 1) { x } else { -x };
        }
        Self(out)
    }

    /// The inverse of a **versor**: `V V~ = s` is a scalar, so `V^-1 = V~ / s`. An arbitrary
    /// element is not invertible this way, which is why an entity code is built as a versor.
    pub fn inverse(&self) -> Option<Self> {
        let s = self.mul(&self.reverse()).0[0];
        if s == 0 {
            return None;
        }
        let r = self.reverse();
        let mut out = [0i32; DIM];
        for i in 0..DIM {
            if r.0[i] % s != 0 {
                return None; // not a versor: the division is not exact
            }
            out[i] = r.0[i] / s;
        }
        Some(Self(out))
    }

    pub fn is_zero(&self) -> bool {
        self.0.iter().all(|&x| x == 0)
    }
}

/// A stable entity code: a **unit blade** times a sign, chosen by the name's hash.
///
/// A unit blade is a versor with `V V~ = +-1`, so its inverse is `+-V` and the unbinding is exact in
/// integers. (A rotor built from two sign vectors has norm^2 = 36 and its inverse is fractional,
/// which integer arithmetic refuses — that was the first version's bug.) Identity stays lexical and
/// exact; the attribute carries the structure.
pub fn entity_code(name: &str) -> Element {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in name.to_ascii_lowercase().bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x100_0000_01b3);
    }
    let mask = (hash & 0x3f) as u8;
    let mask = if mask == 0 { 1 } else { mask };
    let sign = if (hash >> 6) & 1 == 1 { 1 } else { -1 };
    let mut out = [0i32; DIM];
    out[mask as usize] = sign;
    Element(out)
}

/// The attribute blades: the 15 bivectors `e_i e_j`, `i < j`, in a fixed order. There are exactly as
/// many as `CL06_BIVECTOR_COUNT`, which is why the tower and the project's geometry agree.
pub fn attribute_blades() -> Vec<Element> {
    let mut out = Vec::with_capacity(15);
    for i in 0..6u8 {
        for j in (i + 1)..6u8 {
            out.push(Element::blade((1 << i) | (1 << j)));
        }
    }
    out
}

/// The address of a slot: `entity * attribute_blade`, the binding the sum could not do.
pub fn address(entity: &str, attribute: usize) -> Element {
    let blades = attribute_blades();
    let blade = blades.get(attribute).cloned().unwrap_or_else(|| blades[0].clone());
    entity_code(entity).mul(&blade)
}

/// Recover the attribute index from an address and the entity: `address * blade^-1 * entity^-1`.
///
/// This is the property a tuple cannot offer: the attribute is *read out of the address*, not
/// carried beside it. On an exact address it returns the original blade; on anything else it returns
/// the blade with the largest overlap, which is a lawful nearest, not a normalised distance.
pub fn recover_attribute(entity: &str, address: &Element) -> Option<usize> {
    // `A = E B`, so `E^-1 A = B`. Left multiplication, not right: the product is non-commutative and
    // that is the whole point of the order property.
    let unbind = entity_code(entity).inverse()?.mul(address);
    attribute_blades()
        .into_iter()
        .enumerate()
        .map(|(i, blade)| {
            let overlap: i32 = unbind.0.iter().zip(&blade.0).map(|(a, b)| a * b).sum();
            (i, overlap)
        })
        .max_by_key(|&(_, o)| o)
        .map(|(i, _)| i)
}

/// A **dense** entity code: `+1` or `-1` on every one of the 64 blades, from the name's hash.
///
/// The sparse form (a single unit blade) is exact and invertible but its sign code is one bit, so it
/// cannot be shortlisted by Hamming — measured: unrelated addresses landed within 1 bit. A dense code
/// is what makes the 64-bit word informative; the exact check stays an equality on the element.
pub fn dense_entity_code(name: &str) -> Element {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in name.to_ascii_lowercase().bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x100_0000_01b3);
    }
    let mut out = [0i32; DIM];
    // One independently mixed hash per coefficient (splitmix64). The first version reused rotated
    // 16-bit windows, which correlated the codes: unrelated addresses came within 7 bits.
    for (k, cell) in out.iter_mut().enumerate() {
        let mut z = hash ^ (k as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15);
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^= z >> 31;
        *cell = if z & 1 == 1 { 1 } else { -1 };
    }
    out[0] = 1; // the scalar slot carries the sign of the whole code, keep it positive
    Element(out)
}

/// A dense address: `entity * attribute_blade`, both dense, so the sign code is informative and the
/// geometric product keeps the role order.
pub fn dense_address(entity: &str, attribute: usize) -> Element {
    let blades = attribute_blades();
    let blade = blades.get(attribute).cloned().unwrap_or_else(|| blades[0].clone());
    dense_entity_code(entity).mul(&blade)
}

/// The **sign code** of an element: bit `k` is set when coefficient `k` is negative.
///
/// This is the bridge between the two mechanisms. A Hamming head reads one 64-bit word, and Cl(0,6)
/// has 64 basis blades for the same reason — six binary choices — so a 64-bit code *is* an element's
/// sign pattern. That makes one read do both jobs: popcount for the shortlist, the algebra for the
/// exact answer.
pub fn sign_code(element: &Element) -> u64 {
    let mut code = 0u64;
    for (k, &coefficient) in element.0.iter().enumerate() {
        if coefficient < 0 {
            code |= 1 << k;
        }
    }
    code
}

/// Hamming distance between two sign codes: `popcount(a xor b)`, two integer instructions.
pub fn hamming_distance(a: u64, b: u64) -> u32 {
    (a ^ b).count_ones()
}

/// The shortlist-then-verify read: the stored address nearest to `query` by Hamming over sign codes,
/// returned **only** when it is exactly equal to the query's address.
///
/// This is the literature's "coarse code selects candidates, exact equality returns the value"
/// (`GO-FORWARD` §4) in the form the project can serve: 64-bit popcount for the coarse step, the
/// Clifford element for the exact one. `None` is a typed abstention.
pub fn shortlist_exact(query: &Element, stored: &[Element]) -> Option<usize> {
    let code = sign_code(query);
    let best = stored
        .iter()
        .enumerate()
        .min_by_key(|(_, element)| hamming_distance(code, sign_code(element)))?;
    (best.1 == query).then_some(best.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generators_square_to_minus_one_and_anti_commute() {
        for k in 0..6u8 {
            let e = Element::blade(1 << k);
            assert_eq!(e.mul(&e).0[0], -1, "e{k}^2 must be -1");
        }
        let (i, j) = (Element::blade(1), Element::blade(2));
        assert_eq!(i.mul(&j).0, j.mul(&i).0.map(|x| -x));
    }

    #[test]
    fn fifteen_attribute_blades_are_mutually_orthogonal() {
        let blades = attribute_blades();
        assert_eq!(blades.len(), 15, "Cl(0,6) has 15 bivectors");
        for a in 0..blades.len() {
            for b in (a + 1)..blades.len() {
                // Distinct basis blades have disjoint supports: their coefficient inner product is 0.
                let overlap: i32 = blades[a].0.iter().zip(&blades[b].0).map(|(x, y)| x * y).sum();
                assert_eq!(overlap, 0, "blades {a} and {b} are not orthogonal");
            }
        }
    }

    #[test]
    fn bind_is_ordered_when_the_supports_overlap() {
        // Blades with disjoint supports commute; overlapping ones anti-commute. Role separation from
        // the algebra therefore holds exactly where the entity and the slot share a generator, which
        // is the case the role scheme cares about. Disjoint commutation is a property, not a defect.
        let e0 = Element::blade(1);
        let e0e1 = Element::blade(3);
        assert_ne!(e0.mul(&e0e1).0, e0e1.mul(&e0).0, "overlapping roles must be ordered");
        assert_eq!(
            Element::blade(1).mul(&Element::blade(2)).0,
            Element::blade(2).mul(&Element::blade(1)).0.map(|x| -x),
            "disjoint generators anti-commute"
        );
    }

    #[test]
    fn the_attribute_is_recoverable_from_the_address() {
        for attribute in 0..15 {
            let a = address("alice", attribute);
            assert_eq!(recover_attribute("alice", &a), Some(attribute));
        }
    }

    #[test]
    fn the_dense_sign_code_separates_the_exact_address_from_every_other() {
        // Ten entities x eight attributes as DENSE addresses: the sign code must rank the exact one
        // first, and unrelated addresses must sit far away rather than one bit apart.
        let mut stored = Vec::new();
        for e in 0..10 {
            for a in 0..8 {
                stored.push(dense_address(&format!("entity{e}"), a));
            }
        }
        let mut nearest_other = 64u32;
        for e in 0..10 {
            for a in 0..8 {
                let query = dense_address(&format!("entity{e}"), a);
                let index = shortlist_exact(&query, &stored).expect("exact address is stored");
                assert_eq!(stored[index], query);
                let other = stored
                    .iter()
                    .filter(|s| **s != query)
                    .map(|s| hamming_distance(sign_code(&query), sign_code(s)))
                    .min()
                    .expect("more than one stored");
                nearest_other = nearest_other.min(other);
            }
        }
        assert!(nearest_other >= 8, "unrelated dense addresses sat within {nearest_other} bits");
        println!("nearest unrelated dense address: {nearest_other} of 64 bits");
    }

    #[test]
    fn different_attributes_do_not_collide() {
        let colour = address("alice", 0);
        let city = address("alice", 1);
        assert_ne!(colour.0, city.0);
        // And the entity factor keeps entities apart for the same attribute.
        assert_ne!(address("alice", 0).0, address("bruno", 0).0);
    }
}
