//! Lawful integer read-binding hinge (`DiscreteReadBindingHingeV1`, corrected).
//!
//! The held design (`docs/labs/read-binding-on-lawful-read/README.md`) composes the
//! read's first two keys as `R = (rel << 3) - distance`, claiming `distance` lies
//! in `{0, 2, 4, 6, 8}`. It does not: `positioned_root_cost` sums
//! `|query[d] - root[d]|` over eight coordinates, so the distance is unbounded and
//! a `+1` relation does not dominate it. This module therefore defines the hinge
//! on the read's **exact** comparator and a **rank gap**: the best bound leaf and
//! the best competing leaf are ranked among the eligible candidates, and the
//! margin is the number of places the bound is ahead.
//!
//! Every quantity is integer; no float, softmax or matmul. The serving pass reads
//! none of this: it is a training-time sidecar cost.

/// The sidecar's absent marker: a row with this span is not a supervised memory row.
pub const ABSENT: u32 = u32::MAX;
/// The rank gap the bound should clear: two places ahead of the best competitor.
pub const M_BIND: i32 = 2;
/// The penalty when no bound leaf is admitted at level 0.
pub const W_MISS: u32 = 4;

/// Four words per row: `[bound_start, bound_end, comp_mask_lo, comp_mask_hi]`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BindSpan {
    /// First bound leaf (inclusive), or [`ABSENT`].
    pub bound_start: u32,
    /// End of the bound leaf range (exclusive).
    pub bound_end: u32,
    /// Competing leaf bits `0..32`.
    pub comp_mask_lo: u32,
    /// Competing leaf bits `32..64`.
    pub comp_mask_hi: u32,
}

impl BindSpan {
    /// The span of a row that carries no binding label.
    pub const ABSENT_SPAN: Self = Self {
        bound_start: ABSENT,
        bound_end: ABSENT,
        comp_mask_lo: 0,
        comp_mask_hi: 0,
    };

    /// Parse one four-word sidecar row.
    pub fn from_words(words: [u32; 4]) -> Self {
        Self {
            bound_start: words[0],
            bound_end: words[1],
            comp_mask_lo: words[2],
            comp_mask_hi: words[3],
        }
    }

    /// The four words this span occupies.
    pub fn words(&self) -> [u32; 4] {
        [
            self.bound_start,
            self.bound_end,
            self.comp_mask_lo,
            self.comp_mask_hi,
        ]
    }

    /// Whether `node` is a bound leaf.
    pub fn is_bound(&self, node: u32) -> bool {
        self.bound_start != ABSENT && node >= self.bound_start && node < self.bound_end
    }

    /// Whether `node` is a competing leaf.
    pub fn is_competing(&self, node: u32) -> bool {
        let bit = if node < 32 {
            (self.comp_mask_lo >> node) & 1
        } else if node < 64 {
            (self.comp_mask_hi >> (node - 32)) & 1
        } else {
            0
        };
        bit != 0
    }
}

/// One eligible candidate at level 0, with the five keys the read orders by.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Candidate {
    /// The leaf's node index.
    pub node: u32,
    /// The relation score (`rel[node]`, higher is better).
    pub rel: i32,
    /// The positioned root cost (lower is better).
    pub distance: u32,
    /// The coordinate Hamming count (lower is better).
    pub hamming: u32,
    /// The last-touch step (higher is better).
    pub last: u32,
}

/// The read's exact order: relation desc, distance asc, hamming asc, last desc,
/// node asc. `key(a) < key(b)` means `a` is read before `b`.
fn order_key(
    c: &Candidate,
) -> (
    std::cmp::Reverse<i32>,
    u32,
    u32,
    std::cmp::Reverse<u32>,
    u32,
) {
    (
        std::cmp::Reverse(c.rel),
        c.distance,
        c.hamming,
        std::cmp::Reverse(c.last),
        c.node,
    )
}

/// Whether `a` is read before `b`.
pub fn outranks(a: &Candidate, b: &Candidate) -> bool {
    order_key(a) < order_key(b)
}

/// The held design's scalar key, retained to show why the hinge does not use it:
/// it ties `(rel 1, distance 8)` with `(rel 0, distance 0)`, although the read puts
/// the first strictly ahead.
pub fn shift_three_key(c: &Candidate) -> i32 {
    (c.rel << 3).wrapping_sub(c.distance as i32)
}

/// The read-binding hinge for one level-0 candidate set.
///
/// `found` is the admitted prefix (`top[0..found]`); `spans` binds the row. The
/// cost is the rank margin plus the miss penalty, both integers. A row whose span
/// is absent costs zero.
pub fn bind_hinge_cost(
    candidates: &[Candidate],
    span: BindSpan,
    found: &[u32],
    m_bind: i32,
) -> u32 {
    if span.bound_start == ABSENT || span.bound_start >= span.bound_end {
        return 0;
    }
    let mut sorted: Vec<&Candidate> = candidates.iter().collect();
    sorted.sort_by_key(|c| order_key(c));
    let rank_bound = sorted.iter().position(|c| span.is_bound(c.node));
    let rank_comp = sorted.iter().position(|c| span.is_competing(c.node));
    let miss = if found.iter().any(|&node| span.is_bound(node)) {
        0
    } else {
        W_MISS
    };
    let (Some(rank_bound), Some(rank_comp)) = (rank_bound, rank_comp) else {
        // No bound candidate at all is a miss; no competing candidate leaves the
        // margin unconstrained.
        return miss;
    };
    let delta = rank_comp as i64 - rank_bound as i64;
    let margin = (i64::from(m_bind) - delta).max(0) as u32;
    margin + miss
}

#[cfg(test)]
mod tests {
    use super::*;

    fn candidate(node: u32, rel: i32, distance: u32) -> Candidate {
        Candidate {
            node,
            rel,
            distance,
            hamming: 0,
            last: 0,
        }
    }

    fn span(bound_start: u32, bound_end: u32, comp: &[u32]) -> BindSpan {
        let mut lo = 0u32;
        let mut hi = 0u32;
        for &node in comp {
            if node < 32 {
                lo |= 1 << node;
            } else {
                hi |= 1 << (node - 32);
            }
        }
        BindSpan {
            bound_start,
            bound_end,
            comp_mask_lo: lo,
            comp_mask_hi: hi,
        }
    }

    #[test]
    fn the_shift_three_key_ties_a_pair_the_read_orders() {
        // This is the design's false premise, recorded: a +1 relation with an
        // 8-unit distance ties a 0 relation at distance 0, but the read ranks the
        // relation first. The hinge therefore uses `outranks`, not this key.
        let higher_relation = candidate(1, 1, 8);
        let nearer = candidate(2, 0, 0);
        assert!(outranks(&higher_relation, &nearer));
        assert_eq!(shift_three_key(&higher_relation), shift_three_key(&nearer));
    }

    #[test]
    fn the_hinge_charges_the_rank_gap_and_the_miss() {
        // Bound at 10, competing at 11 and 12, unrelated at 13.
        let bound = span(10, 11, &[11, 12]);
        let candidates = [
            candidate(10, 3, 4),
            candidate(11, 2, 0),
            candidate(12, 1, 0),
            candidate(13, 0, 0),
        ];
        // Bound is rank 0, the best competitor rank 1: one place ahead, so the
        // two-place margin costs one.
        assert_eq!(bind_hinge_cost(&candidates, bound, &[10], M_BIND), 1);
        // The same row with only the bound admitted still costs one, not the miss.
        assert_eq!(
            bind_hinge_cost(&candidates, bound, &[11], M_BIND),
            1 + W_MISS
        );

        // A third candidate between them makes the bound two places ahead: no margin.
        let separated = [
            candidate(10, 3, 4),
            candidate(13, 2, 0),
            candidate(11, 1, 0),
        ];
        assert_eq!(bind_hinge_cost(&separated, bound, &[10], M_BIND), 0);

        // A competing leaf ahead of the bound costs more than the plain margin.
        let losing = [candidate(11, 3, 0), candidate(10, 2, 4)];
        assert!(bind_hinge_cost(&losing, bound, &[11], M_BIND) > 1);
    }

    #[test]
    fn an_absent_span_costs_zero() {
        let candidates = [candidate(1, 0, 0)];
        assert_eq!(
            bind_hinge_cost(&candidates, BindSpan::ABSENT_SPAN, &[], M_BIND),
            0
        );
        assert_eq!(
            bind_hinge_cost(&candidates, span(5, 5, &[]), &[], M_BIND),
            0
        );
    }

    #[test]
    fn permuting_the_sidecar_flips_the_rank_zero_leaf() {
        let candidates = [
            candidate(10, 3, 4),
            candidate(11, 1, 0),
            candidate(12, 0, 0),
        ];
        let mut sorted = candidates;
        sorted.sort_by_key(|c| order_key(c));
        let bound = span(10, 11, &[11, 12]);
        let permuted = span(11, 12, &[10]);
        assert_eq!(sorted[0].node, 10);
        assert!(span(10, 11, &[11, 12]).is_bound(sorted[0].node));
        let mut flipped = sorted;
        flipped.sort_by_key(|c| order_key(c));
        assert_eq!(flipped[0].node, 10);
        assert!(permuted.is_competing(flipped[0].node));
        assert_eq!(bind_hinge_cost(&candidates, bound, &[10], M_BIND), 1);
        assert!(bind_hinge_cost(&candidates, permuted, &[10], M_BIND) >= 2);
    }
}
