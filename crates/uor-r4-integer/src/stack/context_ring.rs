//! The `Context` stage's bounded ring and settlement gate (ADR-0007, I2).
//!
//! This is the transferable part of CIRISKernel, promoted from the independent probe in
//! `tests/context_window_probe.rs` (codex-3). The probe stays as it is on purpose: a gate that
//! shared its code with the implementation would only prove itself.
//!
//! The rule is one sentence: **a span leaves a bounded ring only when it is settled**, and the stub
//! it leaves behind keeps exactly what the declared read consumes.
//!
//! - A **fold** read (a commutative sum) survives a sum stub exactly.
//! - A **selection** read (top-k) does not survive a sum stub, and does survive a mergeable
//!   bounded top-k sketch of at least its `k`.
//!
//! An unsettled eviction is refused and the ring is unchanged, because two histories the stub
//! cannot distinguish would be sent to different answers by a later read — the witness pair.
//!
//! What this module is not: the wiring that lets a served read consume the stub. The native
//! first-pick read ranks candidates against the *current* query, so its sketch sufficiency is not
//! the static one proved here; that is the remaining half of I2 and is ordered, not assumed.

/// One context span: a position and the value it contributes to the declared reads.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ContextSpan {
    /// The span's absolute position in the context.
    pub position: u32,
    /// The value the fold read sums and the selection read ranks.
    pub value: i64,
}

/// The read's order over spans: larger value first, ties to the earlier position.
fn better(a: &ContextSpan, b: &ContextSpan) -> bool {
    a.value > b.value || (a.value == b.value && a.position < b.position)
}

/// A mergeable bounded top-k: the largest `k` values, ties to the earlier position.
///
/// Insertion into a bounded top-k is a join, so `merge` is associative and commutative and the
/// stub's contents do not depend on the order spans were dropped in.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TopK {
    k: usize,
    entries: Vec<ContextSpan>,
}

impl TopK {
    /// A sketch that keeps the `k` best spans.
    pub fn new(k: usize) -> Self {
        Self {
            k,
            entries: Vec::new(),
        }
    }

    /// Fold one span into the sketch.
    pub fn insert(&mut self, span: ContextSpan) {
        if self.k == 0 {
            return;
        }
        self.entries.push(span);
        self.entries.sort_by(|a, b| match (better(a, b), better(b, a)) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => std::cmp::Ordering::Equal,
        });
        self.entries.truncate(self.k);
    }

    /// The join of two sketches: the same set whichever side is the receiver.
    pub fn merge(&self, other: &TopK) -> TopK {
        let mut out = self.clone();
        for span in &other.entries {
            out.insert(*span);
        }
        out
    }

    /// The kept spans, best first.
    pub fn entries(&self) -> &[ContextSpan] {
        &self.entries
    }

    /// The kept positions, best first.
    pub fn positions(&self) -> Vec<u32> {
        self.entries.iter().map(|span| span.position).collect()
    }

    /// The sketch's capacity.
    pub fn k(&self) -> usize {
        self.k
    }
}

/// A bounded ring of spans, with a stub per dropped span.
pub struct ContextRing {
    slots: Vec<Option<ContextSpan>>,
    settled: Vec<bool>,
    cursor: usize,
    folded_sum: i128,
    folded_top: TopK,
}

impl ContextRing {
    /// `capacity` live slots and a sketch that keeps `sketch` dropped candidates.
    pub fn new(capacity: usize, sketch: usize) -> Self {
        assert!(capacity > 0, "a bounded ring needs a slot");
        Self {
            slots: vec![None; capacity],
            settled: vec![false; capacity],
            cursor: 0,
            folded_sum: 0,
            folded_top: TopK::new(sketch),
        }
    }

    /// Write `span` into the next slot.
    ///
    /// When the slot holds an **unsettled** span the write is refused and the blocking span's
    /// position is returned, with the ring unchanged. A settled span is folded into the stub and
    /// dropped.
    pub fn push(&mut self, span: ContextSpan, settled: bool) -> Result<(), u32> {
        let slot = self.cursor % self.slots.len();
        if let Some(old) = self.slots[slot] {
            if !self.settled[slot] {
                return Err(old.position);
            }
            self.folded_sum += i128::from(old.value);
            self.folded_top.insert(old);
        }
        self.slots[slot] = Some(span);
        self.settled[slot] = settled;
        self.cursor += 1;
        Ok(())
    }

    /// Settle the span at `position`, so a later push may evict it. Idempotent: settling a
    /// settled span is not an error, and an absent position settles nothing (`false`).
    ///
    /// This is the `settle` half of the ADR's seam that only marks settlement; merging the stub
    /// into the model's state is the ordered follow-up.
    pub fn settle(&mut self, position: u32) -> bool {
        for (slot, span) in self.slots.iter().enumerate() {
            if span.is_some_and(|span| span.position == position) {
                self.settled[slot] = true;
                return true;
            }
        }
        false
    }

    /// The fold read's value over everything the ring has seen: the stub plus the live slots.
    pub fn sum(&self) -> i128 {
        self.folded_sum
            + self
                .slots
                .iter()
                .flatten()
                .map(|span| i128::from(span.value))
                .sum::<i128>()
    }

    /// The selection read's top-`k` over everything the ring has seen: the fuse of the stub and the
    /// live slots. The sketch must keep at least `k` or a dropped candidate cannot answer.
    pub fn top(&self, k: usize) -> TopK {
        let mut out = TopK::new(k);
        for span in self.folded_top.entries() {
            out.insert(*span);
        }
        for span in self.slots.iter().flatten() {
            out.insert(*span);
        }
        out
    }

    /// The live slots.
    pub fn live(&self) -> impl Iterator<Item = ContextSpan> + '_ {
        self.slots.iter().flatten().copied()
    }

    /// The live span count.
    pub fn live_len(&self) -> usize {
        self.slots.iter().flatten().count()
    }

    /// The dropped spans the stub holds, best first (the audit view of the sketch).
    pub fn stub(&self) -> &TopK {
        &self.folded_top
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The unbounded record the ring is compared against.
    fn history(n: u32) -> Vec<ContextSpan> {
        (0..n)
            .map(|step| ContextSpan {
                position: step,
                value: i128::from((step * 37) % 23) as i64 - 11,
            })
            .collect()
    }

    fn history_sum(spans: &[ContextSpan]) -> i128 {
        spans.iter().map(|span| i128::from(span.value)).sum()
    }

    fn history_top(spans: &[ContextSpan], k: usize) -> Vec<u32> {
        let mut sketch = TopK::new(k);
        for &span in spans {
            sketch.insert(span);
        }
        sketch.positions()
    }

    #[test]
    fn the_fold_and_the_sketch_survive_every_settled_wrap() {
        let (capacity, sketch) = (8, 4);
        let mut ring = ContextRing::new(capacity, sketch);
        let spans = history(40);
        for (step, &span) in spans.iter().enumerate() {
            ring.push(span, true).expect("every span is settled");
            assert_eq!(
                ring.sum(),
                history_sum(&spans[..=step]),
                "fold read at step {step}"
            );
            assert_eq!(
                ring.top(3).positions(),
                history_top(&spans[..=step], 3),
                "selection read at step {step}"
            );
            assert!(ring.live_len() <= capacity, "the ring stayed bounded");
        }
        assert_eq!(ring.stub().entries().len(), sketch);
    }

    #[test]
    fn an_unsettled_push_is_refused_and_leaves_the_ring_unchanged() {
        let mut ring = ContextRing::new(2, 1);
        let span = |position, value| ContextSpan { position, value };
        ring.push(span(1, 3), true).expect("settled");
        ring.push(span(2, -1), false).expect("live");
        // The third span evicts the settled first one; the fourth meets the unsettled second.
        ring.push(span(3, 5), true).expect("settled eviction");
        let before = (ring.sum(), ring.live_len(), ring.top(1));
        let blocked = ring
            .push(span(4, 7), true)
            .expect_err("an unsettled span is not evicted");
        assert_eq!(blocked, 2, "the blocking span's position");
        assert_eq!((ring.sum(), ring.live_len(), ring.top(1)), before);
        // Settling it lets the same push through.
        assert!(ring.settle(2));
        ring.push(span(4, 7), true).expect("settled eviction");
        assert_eq!(ring.stub().positions(), vec![1]);
    }

    #[test]
    fn the_stub_merge_is_order_independent() {
        // A mergeable top-k is a join, so the stub does not depend on the drop order.
        let spans = history(12);
        let split = 5;
        let mut left = TopK::new(3);
        let mut right = TopK::new(3);
        for &span in &spans[..split] {
            left.insert(span);
        }
        for &span in &spans[split..] {
            right.insert(span);
        }
        assert_eq!(left.merge(&right), right.merge(&left));
        assert_eq!(left.merge(&right).positions(), history_top(&spans, 3));
    }

    #[test]
    fn a_settled_span_can_be_settled_only_once_by_position() {
        let mut ring = ContextRing::new(2, 2);
        ring.push(
            ContextSpan {
                position: 7,
                value: 1,
            },
            false,
        )
        .expect("live");
        assert!(ring.settle(7));
        assert!(ring.settle(7), "settling is idempotent");
        assert!(!ring.settle(8), "an absent position settles nothing");
    }
}
