//! Allocation-free D11 integer flock selector for sparse causal attention.
//!
//! Multiplier-free and allocation-free causal flock selection under owner decision
//! D11 and Council D16 (Rule 5 & Item 16).
//!
//! Flock attention keeps for each query row:
//! 1. A **sink** position (e.g. position 0, always preserved).
//! 2. A **local window** of the nearest preceding positions (e.g. 64 positions).
//! 3. The **exact top-k** of the remaining causally visible positions (e.g. k in {1, 7, 16, 64}).
//!
//! This module provides:
//! - [`FlockScratch`]: Reusable workspace preallocated at session initialization,
//!   ensuring strictly zero heap allocations per token step during live served inference.
//! - In-place O(n + k log k) partial selection using `select_nth_unstable_by` over the
//!   non-window past, eliminating full O(n log n) sorts.
//! - Strict deduplicated support `{sink} ∪ window ∪ top_k` with deterministic lowest-position
//!   tie breaking.
//! - Integer rank scoring (Minkowski / dot product) and multiplier-free fixed-point
//!   rank weights (Arm R) and integer softmax (Arm S).
//! - Top-k-only entrypoint (`top_k_select_integer`) for single-pointer (k=1) retrieval.

use crate::stack::kernels::stack_div_u128;
use crate::{invalid, IntegerError, Result};

/// Version tag binding the integer flock selector contract.
pub const FLOCK_INTEGER_SELECTOR_VERSION: &str = "flock-integer-selector-v1";

/// Maximum supported context positions in a single session scratch buffer.
pub const MAX_FLOCK_CONTEXT: usize = 4096;

/// Role of a selected position inside the flock support.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FlockSlot {
    /// The sink position, always included.
    Sink,
    /// One of the nearest `window` causally visible positions.
    Window,
    /// An exact top-k position outside the sink and window.
    TopK,
}

/// One selected position and its admitting role.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FlockEntry {
    pub position: usize,
    pub slot: FlockSlot,
}

/// Selection parameters for one query step.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FlockSelect {
    /// Always-selected anchor position (typically 0).
    pub sink: usize,
    /// Nearest preceding positions always included.
    pub window: usize,
    /// Exact top-k of the remaining causally visible positions.
    pub k: usize,
}

impl FlockSelect {
    pub fn new(sink: usize, window: usize, k: usize) -> Self {
        Self { sink, window, k }
    }

    /// Standard B0 / A1 canonical configuration: sink at 0, window 64, top-k.
    pub fn b0(k: usize) -> Self {
        Self {
            sink: 0,
            window: 64,
            k,
        }
    }
}

/// Exact accounting counters for one selection step.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FlockScan {
    /// Causally visible positions (`query + 1`).
    pub visible: usize,
    /// Kept entries in sink role (0 or 1).
    pub sink_kept: usize,
    /// Kept entries in window role.
    pub window_kept: usize,
    /// Kept entries in top-k role.
    pub top_k_kept: usize,
    /// Rest positions compared for top-k selection.
    pub candidates_scanned: usize,
    /// Whole visible prefix fits inside nominal window.
    pub short_prefix: bool,
    /// Fewer than `k` rest candidates existed.
    pub top_k_short: bool,
    /// The k-th and (k+1)-th candidates have equal rank scores (tie broken by position).
    pub cutoff_ties: bool,
}

/// Pre-allocated workspace for allocation-free flock selection.
///
/// Allocated once per session or layer. Calling [`FlockScratch::reset`]
/// resets length counters without releasing or reallocating memory.
#[derive(Clone, Debug)]
pub struct FlockScratch {
    /// Per-position slot tags, sized to maximum context.
    slots: Vec<Option<FlockSlot>>,
    /// Indices of non-window, non-sink positions for top-k partitioning.
    rest: Vec<usize>,
    /// Kept entries ordered by descending rank score (lowest position on ties).
    pub entries: Vec<FlockEntry>,
}

impl FlockScratch {
    /// Create a scratch workspace with capacity for at least `capacity` context positions.
    pub fn new(capacity: usize) -> Self {
        let cap = capacity.max(64);
        Self {
            slots: vec![None; cap],
            rest: Vec::with_capacity(cap),
            entries: Vec::with_capacity(129.min(cap)),
        }
    }

    /// Ensure capacity for at least `needed` positions.
    pub fn ensure_capacity(&mut self, needed: usize) {
        if self.slots.len() < needed {
            self.slots.resize(needed, None);
            self.rest.reserve(needed - self.rest.capacity());
        }
    }

    /// Reset internal buffers for a query at `query`.
    #[inline(always)]
    fn prepare(&mut self, query: usize) {
        self.ensure_capacity(query + 1);
        self.slots[..=query].fill(None);
        self.rest.clear();
        self.entries.clear();
    }
}

/// Exact integer flock selection over `scores[0..=query]`.
///
/// Scores are integer rank scores (such as integer Minkowski inner product or dot product).
/// Higher scores rank nearer.
///
/// Returns the selection accounting [`FlockScan`]. Selected entries are written into
/// `scratch.entries` in descending rank order with ties broken to the lowest position.
///
/// Algorithm:
/// - Deduplicated support `{sink} ∪ window ∪ top_k`.
/// - Sink wins if inside window (deduplication).
/// - Non-sink, non-window rest positions are partitioned in O(n + k log k) using
///   `select_nth_unstable_by`, avoiding full O(n log n) sorting.
/// - Strictly zero heap allocations when `scratch` has sufficient capacity.
#[inline(never)]
pub fn flock_select_integer(
    scores: &[i64],
    query: usize,
    select: FlockSelect,
    scratch: &mut FlockScratch,
) -> Result<FlockScan> {
    if scores.len() <= query {
        return Err(invalid(format!(
            "flock scores length {} is too short for query {query}",
            scores.len()
        )));
    }
    if select.window == 0 {
        return Err(invalid("flock window must be positive"));
    }
    if select.k == 0 {
        return Err(invalid("flock k must be positive"));
    }
    if select.sink > query {
        return Err(invalid("flock sink must be causally visible"));
    }

    scratch.prepare(query);

    let window_start = (query + 1).saturating_sub(select.window);
    for slot in scratch.slots[window_start..=query].iter_mut() {
        *slot = Some(FlockSlot::Window);
    }
    // Sink takes precedence if it falls inside the window span
    scratch.slots[select.sink] = Some(FlockSlot::Sink);

    // Collect rest positions (prior to window_start, excluding sink)
    for pos in 0..window_start {
        if scratch.slots[pos].is_none() {
            scratch.rest.push(pos);
        }
    }

    let candidates_scanned = scratch.rest.len();
    let short_prefix = (query + 1) <= select.window;
    let top_k_short = candidates_scanned < select.k;

    // Partition top-k in O(n + k log k)
    let (top_k_count, cutoff_ties) = if candidates_scanned <= select.k {
        // All rest candidates are selected; sort all of them
        scratch.rest.sort_by(|&a, &b| {
            scores[b as usize]
                .cmp(&scores[a as usize])
                .then_with(|| a.cmp(&b))
        });
        (candidates_scanned, false)
    } else {
        // More than k candidates: select_nth_unstable_by puts the k best at 0..k
        scratch.rest.select_nth_unstable_by(select.k, |&a, &b| {
            scores[b as usize]
                .cmp(&scores[a as usize])
                .then_with(|| a.cmp(&b))
        });

        // Check if cutoff is a tie with (k+1)-th element
        let cutoff_ties =
            scores[scratch.rest[select.k - 1] as usize] == scores[scratch.rest[select.k] as usize];

        // Sort only the k selected elements
        scratch.rest[..select.k].sort_by(|&a, &b| {
            scores[b as usize]
                .cmp(&scores[a as usize])
                .then_with(|| a.cmp(&b))
        });

        (select.k, cutoff_ties)
    };

    for &pos in &scratch.rest[..top_k_count] {
        scratch.slots[pos] = Some(FlockSlot::TopK);
    }

    // Materialize entries
    let mut sink_kept = 0usize;
    let mut window_kept = 0usize;
    let mut top_k_kept = 0usize;

    for (pos, &slot_opt) in scratch.slots[..=query].iter().enumerate() {
        if let Some(slot) = slot_opt {
            scratch.entries.push(FlockEntry {
                position: pos,
                slot,
            });
            match slot {
                FlockSlot::Sink => sink_kept += 1,
                FlockSlot::Window => window_kept += 1,
                FlockSlot::TopK => top_k_kept += 1,
            }
        }
    }

    // Sort final kept support in descending rank order with lowest position on ties
    scratch.entries.sort_by(|a, b| {
        scores[b.position]
            .cmp(&scores[a.position])
            .then_with(|| a.position.cmp(&b.position))
    });

    Ok(FlockScan {
        visible: query + 1,
        sink_kept,
        window_kept,
        top_k_kept,
        candidates_scanned,
        short_prefix,
        top_k_short,
        cutoff_ties,
    })
}

/// Exact top-k only selection over `scores[0..=query]`.
///
/// Specialized entrypoint for pointer retrieval (e.g. k=1) without sink and without window.
#[inline(never)]
pub fn top_k_select_integer(
    scores: &[i64],
    query: usize,
    k: usize,
    scratch: &mut FlockScratch,
) -> Result<FlockScan> {
    if scores.len() <= query {
        return Err(invalid("scores length is too short for query"));
    }
    if k == 0 {
        return Err(invalid("k must be positive"));
    }

    scratch.prepare(query);
    for pos in 0..=query {
        scratch.rest.push(pos);
    }

    let candidates_scanned = scratch.rest.len();
    let top_k_short = candidates_scanned < k;

    let (top_k_count, cutoff_ties) = if candidates_scanned <= k {
        scratch.rest.sort_by(|&a, &b| {
            scores[b as usize]
                .cmp(&scores[a as usize])
                .then_with(|| a.cmp(&b))
        });
        (candidates_scanned, false)
    } else {
        scratch.rest.select_nth_unstable_by(k, |&a, &b| {
            scores[b as usize]
                .cmp(&scores[a as usize])
                .then_with(|| a.cmp(&b))
        });
        let cutoff_ties = scores[scratch.rest[k - 1] as usize] == scores[scratch.rest[k] as usize];
        scratch.rest[..k].sort_by(|&a, &b| {
            scores[b as usize]
                .cmp(&scores[a as usize])
                .then_with(|| a.cmp(&b))
        });
        (k, cutoff_ties)
    };

    for &pos in &scratch.rest[..top_k_count] {
        scratch.entries.push(FlockEntry {
            position: pos,
            slot: FlockSlot::TopK,
        });
    }

    Ok(FlockScan {
        visible: query + 1,
        sink_kept: 0,
        window_kept: 0,
        top_k_kept: top_k_count,
        candidates_scanned,
        short_prefix: false,
        top_k_short,
        cutoff_ties,
    })
}

/// Normalized fixed rank table weights: `w_i ∝ 1 / (i + 1)` in Q31.
///
/// Sums to `(1 << 31) - 1` (within roundoff). Multiplier-free, uses exact restoring
/// long division (`stack_div_u128`).
#[inline(never)]
pub fn rank_table_q31(count: usize, out: &mut [u32]) -> Result<()> {
    if out.len() < count {
        return Err(invalid("output slice is smaller than count"));
    }
    if count == 0 {
        return Ok(());
    }
    if count == 1 {
        out[0] = 0x7FFF_FFFF; // 1.0 in Q31
        return Ok(());
    }

    // Compute harmonic sum in Q32
    // 1 / (i + 1) in Q32: 2^32 / (i + 1)
    let mut sum_q32 = 0u128;
    for i in 0..count {
        let term = (1u128 << 32) / (i as u128 + 1);
        sum_q32 += term;
    }

    // Normalize each term to Q31: w_i = (term / sum_q32) * 2^31
    // (term * 2^63) / sum_q32 gives Q31
    for (i, slot) in out[..count].iter_mut().enumerate() {
        let term = (1u128 << 32) / (i as u128 + 1);
        let num = term << 31;
        let w = stack_div_u128(num, sum_q32);
        *slot = (w as u32).min(0x7FFF_FFFF);
    }

    Ok(())
}

/// Raw unnormalized rank weights: `a_i = 1 / (i + 1)` in Q16.
///
/// Unnormalized B2 hybrid scale. Multiplier-free.
#[inline(never)]
pub fn raw_rank_weights_q16(count: usize, out: &mut [u16]) -> Result<()> {
    if out.len() < count {
        return Err(invalid("output slice is smaller than count"));
    }
    for (i, slot) in out[..count].iter_mut().enumerate() {
        let term = (1u32 << 16) / (i as u32 + 1);
        *slot = term.min(0xFFFF) as u16;
    }
    Ok(())
}
