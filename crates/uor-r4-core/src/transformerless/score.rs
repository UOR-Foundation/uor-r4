//! Phase 4 of the graph-compiler plan (§5): semantic transitions and
//! ScoreQ residual emission onto the induced cover, plus the Gate C
//! measurement harness. This module is the COMPILER side — it may use
//! floats (documented below); the scoring path it emits for is the
//! integer-only reference scorer in [`super::score_runtime`].
//!
//! # What is compiled
//!
//! - **Forward transitions E_f** ([`compile_transitions`]): over
//!   consecutive train positions `(t, t+1)` in one story, the active
//!   cloud `A_t` (top-M memberships at each depth — the same
//!   `binary_memberships` semantics the cover and the scorer use) is
//!   crossed with `A_{t+1}` to accumulate `(src region, dst region)`
//!   counts per depth. Per source region the out-degree is bounded
//!   ([`ScoreConfig::transition_out_degree`], default 8) by the
//!   canonical order (weight desc, then dst asc); the edge weight is
//!   `ScoreQ::from_logprob(ln(count / src_total))` with `src_total` the
//!   pre-truncation evidence total. The reverse index E_b is built by
//!   sorting canonical edge IDs by `(dst, src, kind)` — Theorem 7
//!   consistency by construction, verified by
//!   [`verify_theorem_7_wired`] (the full per-node wiring, stronger
//!   than the format's v0 existence approximation).
//! - **Emission residuals ΔE** ([`compile_emissions`]): per region n
//!   with parent p(n),
//!   `ΔE(n,v) = ScoreQ::from_logprob(ln P(v|n) − ln P(v|p(n)))` where
//!   `P(v|n)` is the region's corpus next-token distribution — the
//!   store's top-3 teacher-weighted evidence over the train
//!   observations whose **covered** binary top-1 membership is n:
//!   within the region's calibrated radius, with no nearest-region
//!   fallback assignment (the backoff floor is a routing behavior, not
//!   region content — with fallback assignment, every observation would
//!   land in some region at every depth and deep regions' distributions
//!   would collect the whole corpus; see
//!   [`score_runtime::binary_top1_covered`]). The rule is deterministic
//!   and identical under re-induction and `--cover` reload. Add-one
//!   smoothing over the compiled vocabulary:
//!   `P(v|n) = (count_n(v) + 1) / (total_n + V)`. The root prior B(v)
//!   is the level-0 store distribution with the same quantization and
//!   smoothing; the smoothing floor `ln(1/(total + V))` is baked into
//!   the EMIT root header. Each region's emission list is sparse and
//!   bounded: the top-E tokens by residual score
//!   ([`ScoreConfig::emission_entries`], default 64; selection order
//!   score desc then token asc, storage ascending token). The induced
//!   cover has no explicit overlap nodes, so no interaction residuals
//!   exist this phase — Theorem 10 non-duplication holds by
//!   construction (root-plus-residual decomposition; each contribution
//!   attached to exactly one node).
//! - **EXCT**: the existing TLS1 graded store bytes verbatim after the
//!   v0 storage descriptor `{width: i32, shift: 0, zero_point: 0}` —
//!   the converter's migration carryover convention. ΔX lookup
//!   semantics are the existing prefix probe from
//!   `runtime::predict_witness_plain` (deepest populated prefix), with
//!   `ΔX(X,v) = ScoreQ::from_logprob(ln P(v|X)) − B(v)` quantized from
//!   the probed counts at score time (the reference scorer's single
//!   documented compiler-side float; residualized EXCT tables are the
//!   Phase-5 migration that removes it).
//!
//! # Wire layout (EMIT remainder)
//!
//! ```text
//! root prior block:  u32 entry_count | u32 total_count
//!                    | i32 floor_score_q | u32 reserved(0)
//!                    entry_count × (i32 token, i32 score_q)   [token asc]
//! region lists:      per region (ascending region id), wired by the
//!                    PackedNode emission ranges (emission_start = byte
//!                    offset into the remainder, emission_len = entry
//!                    count ≤ HEAD E), each
//!                    emission_len × (i32 token, i32 score_q)  [token asc]
//! ```
//!
//! EDGE kind tags distinguish E_r (0), E_o (1), E_f (2); the canonical
//! edge array is sorted by `(src, kind, dst)` so each node's refinement
//! children stay contiguous (the `convert_r4g1`/`cover` convention).
//!
//! # Quantization and platform pinning
//!
//! All `ln` quantization is compiler-side f64→f32 through
//! `ScoreQ::from_logprob` — libm-sensitive cross-platform, **macOS-
//! pinned**, exactly the status of the existing κ baseline and the
//! cover's f64 entropy (the D2 canonical deterministic compile mode
//! resolves cross-platform byte equality later). The scoring path
//! itself is integer-only (see `score_runtime`).
//!
//! # Gate C
//!
//! [`evaluate_gate_c`] measures top-1 agreement with the corpus's
//! recorded teacher argmax and bits/token on the held-out partition
//! for (a) the graph scorer without EXCT, (b) with EXCT, and (c) the
//! TLA3 store baseline (`runtime::predict_witness_plain` on the same
//! positions, Witten-Bell bits as in `evaluate-report`). This is the
//! M.V.G. checkpoint's fidelity input.

use serde::Serialize;
use std::collections::BTreeMap;

use super::compiler::{self, Corpus, SIG_BYTES, SIG_WORDS, STAGES};
use super::cover::{self, Observation};
use super::runtime::{self, Store};
use super::score_runtime::{
    binary_memberships, binary_top1_covered, regions_from_view, structural_edges_from_view,
    verify_witness_replay, GraphScorer, RegionParams, StructuralEdge, EDGE_KIND_FORWARD,
    EDGE_KIND_NEIGHBOR, EDGE_KIND_REFINEMENT,
};
use uor_r4_graph_format::ScoreQ;

/// Default per-source out-degree cap for E_f edges.
pub const DEFAULT_TRANSITION_OUT_DEGREE: usize = 8;
/// Default per-region emission list bound (top-E by residual score).
pub const DEFAULT_EMISSION_ENTRIES: usize = 64;
/// Default number of root-prior tokens admitted to the candidate set.
pub const DEFAULT_ROOT_TOP_B: usize = 64;
/// Default number of EXCT probe tokens admitted to the candidate set.
pub const DEFAULT_EXCT_TOP_X: usize = 64;
/// Default number of held-out positions whose witnesses are replayed
/// during the Gate C evaluation.
pub const DEFAULT_WITNESS_SAMPLE: usize = 64;

/// HEAD defaults reused from `convert_r4g1`/`cover` (RFC §4 starting
/// defaults; honest observed maxima replace the floors when larger).
const DEFAULT_MAX_FRONTIER_WIDTH: u16 = 32;
const MAX_CANDIDATES: u16 = 16;
const DEFAULT_MAX_EMISSION_ENTRIES: u32 = 64;
const SHORTLIST_SIZE: u16 = 8;
const MAX_PROGRAM_STEPS: u32 = 64;

/// blake3 input labeling this compiler as the compiler of record.
const COMPILER_VERSION_LABEL: &[u8] = b"uor-r4-core score v0";

/// Configuration of one scored-graph compilation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScoreConfig {
    /// Per-source out-degree cap for forward transition edges.
    pub transition_out_degree: usize,
    /// Per-region emission list bound (top-E by residual score).
    pub emission_entries: usize,
    /// Number of root-prior tokens admitted to the candidate set.
    pub root_top_b: usize,
    /// Number of EXCT probe tokens admitted to the candidate set.
    pub exct_top_x: usize,
    /// Held-out positions whose witnesses are replayed in Gate C.
    pub witness_sample: usize,
}

impl Default for ScoreConfig {
    fn default() -> Self {
        Self {
            transition_out_degree: DEFAULT_TRANSITION_OUT_DEGREE,
            emission_entries: DEFAULT_EMISSION_ENTRIES,
            root_top_b: DEFAULT_ROOT_TOP_B,
            exct_top_x: DEFAULT_EXCT_TOP_X,
            witness_sample: DEFAULT_WITNESS_SAMPLE,
        }
    }
}

/// One compiled forward transition edge (E_f): artifact node ids, the
/// raw evidence count (report side), and the quantized log weight.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TransitionEdge {
    pub src: u32,
    pub dst: u32,
    pub count: u32,
    pub score: ScoreQ,
}

/// Compile forward transitions E_f from consecutive train positions
/// (module docs). `regions` are the scoring region parameters; the
/// active clouds come from [`binary_memberships`] — one code path with
/// the scorer. Edges come out sorted by `(src, dst)`.
pub fn compile_transitions(
    corpus: &Corpus,
    regions: &[RegionParams],
    train: &[Observation],
    max_depth: usize,
    out_degree: usize,
) -> Vec<TransitionEdge> {
    let pop = runtime::derive_popcount_table();
    let mut k = runtime::OpKernel::default();
    // Canonical observation order (content-addressed positions, §4.1):
    // the caller's slice order never reaches the counts, so a shuffled
    // observation/shard order compiles to identical edges.
    let mut ordered: Vec<&Observation> = train.iter().collect();
    ordered.sort_by_key(|o| o.position);
    // memberships[depth] of the previous adjacent position, carried.
    let mut previous: Option<(u32, Vec<Vec<u32>>)> = None;
    let mut counts: BTreeMap<(u32, u32), u64> = BTreeMap::new();
    let mut src_totals: BTreeMap<u32, u64> = BTreeMap::new();
    for observation in ordered {
        let position = observation.position;
        let mut memberships: Vec<Vec<u32>> = Vec::with_capacity(max_depth);
        for depth in 1..=max_depth {
            let at_depth: Vec<u32> =
                binary_memberships(&mut k, &pop, regions, depth, &observation.sig)
                    .into_iter()
                    .map(|(region, _)| region)
                    .collect();
            memberships.push(at_depth);
        }
        if let Some((prev_position, prev_memberships)) = previous {
            let adjacent = position == prev_position + 1
                && corpus.story[position as usize] == corpus.story[prev_position as usize];
            if adjacent {
                for depth in 0..max_depth {
                    for &src in &prev_memberships[depth] {
                        for &dst in &memberships[depth] {
                            let edge = (src + 1, dst + 1); // region -> node id
                            *counts.entry(edge).or_insert(0) += 1;
                            *src_totals.entry(edge.0).or_insert(0) += 1;
                        }
                    }
                }
            }
        }
        previous = Some((position, memberships));
    }

    // Per source: canonical order (weight desc, then dst asc), bounded.
    let mut by_src: BTreeMap<u32, Vec<(u32, u64)>> = BTreeMap::new();
    for (&(src, dst), &count) in &counts {
        by_src.entry(src).or_default().push((dst, count));
    }
    let mut edges = Vec::new();
    for (src, mut dsts) in by_src {
        dsts.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        let total = src_totals[&src];
        for &(dst, count) in dsts.iter().take(out_degree) {
            // Compiler-side f64 ln quantization (macOS-pinned; module docs).
            let p = count as f64 / total as f64;
            let score = ScoreQ::from_logprob(p.ln() as f32);
            edges.push(TransitionEdge {
                src,
                dst,
                count: count.min(u32::MAX as u64) as u32,
                score,
            });
        }
    }
    edges.sort_by_key(|e| (e.src, e.dst));
    edges
}

/// The compiled residual tables: the ScoreQ root prior (full block) and
/// each region's bounded ΔE list (ascending token storage order).
#[derive(Debug, Clone)]
pub struct EmissionTables {
    /// B(v) for every token observed at level 0 (ascending token).
    pub root_prior: BTreeMap<u32, ScoreQ>,
    /// The add-one smoothing floor `ScoreQ(ln(1/(total + V)))`.
    pub root_floor: ScoreQ,
    /// Level-0 evidence total (the smoothing denominator source).
    pub root_total: u64,
    /// Per region id: `(token, ΔE)` ascending token, bounded to top-E.
    pub region_lists: Vec<Vec<(u32, ScoreQ)>>,
}

/// ln of an add-one-smoothed probability (compiler-side f64; module
/// docs for the platform pinning).
fn smoothed_ln(count: u64, total: u64, vocab: u32) -> f32 {
    ((count as f64 + 1.0) / (total as f64 + f64::from(vocab))).ln() as f32
}

/// Compile the root prior and per-region emission residuals (module
/// docs). The evidence model matches the store's exactly (top-3
/// teacher-weighted counts over train positions); the root distribution
/// is the level-0 store distribution.
pub fn compile_emissions(
    corpus: &Corpus,
    _store: &Store,
    regions: &[RegionParams],
    train: &[Observation],
    max_depth: usize,
    vocab: u32,
    config: &ScoreConfig,
) -> EmissionTables {
    let pop = runtime::derive_popcount_table();
    let mut k = runtime::OpKernel::default();
    // Weighted evidence per region: the covered binary top-1 membership
    // at each depth (within the calibrated radius — the backoff floor is
    // a routing behavior, never region content; see `binary_top1_covered`).
    let mut evidence: Vec<BTreeMap<u32, u64>> = vec![BTreeMap::new(); regions.len()];
    // The root prior must use the same teacher top-3 evidence as region
    // residuals. Using sampled `next` counts here mixes two distributions
    // and makes every residual compare against the wrong parent model.
    let mut root_dist: BTreeMap<u32, u64> = BTreeMap::new();
    for observation in train {
        let i = observation.position as usize;
        for k_idx in 0..3 {
            let token = corpus.top_tokens[i][k_idx];
            let weight = corpus.top_weights[i][k_idx];
            if weight > 0 {
                *root_dist.entry(token).or_insert(0) += u64::from(weight);
            }
        }
        for depth in 1..=max_depth {
            let Some((top1, _)) =
                binary_top1_covered(&mut k, &pop, regions, depth, &observation.sig)
            else {
                continue;
            };
            let dist = &mut evidence[top1 as usize];
            for k_idx in 0..3 {
                let token = corpus.top_tokens[i][k_idx];
                let weight = corpus.top_weights[i][k_idx];
                if weight > 0 {
                    *dist.entry(token).or_insert(0) += u64::from(weight);
                }
            }
        }
    }

    // Root prior B(v) from the same teacher-weighted evidence stream.
    let root_total: u64 = root_dist.values().sum();
    let root_floor = ScoreQ::from_logprob(smoothed_ln(0, root_total, vocab));
    let root_prior: BTreeMap<u32, ScoreQ> = root_dist
        .iter()
        .map(|(&t, &c)| (t, ScoreQ::from_logprob(smoothed_ln(c, root_total, vocab))))
        .collect();

    let mut region_lists = Vec::with_capacity(regions.len());
    for (region_id, region) in regions.iter().enumerate() {
        let dist = &evidence[region_id];
        let total: u64 = dist.values().sum();
        // Parent distribution: the parent region's evidence, or the
        // level-0 root distribution at depth 1.
        let (parent_dist, parent_total): (&BTreeMap<u32, u64>, u64) = match region.parent {
            Some(parent) => {
                let parent_dist = &evidence[parent as usize];
                (parent_dist, parent_dist.values().sum())
            }
            None => (&root_dist, root_total),
        };
        let mut residuals: Vec<(u32, ScoreQ)> = dist
            .iter()
            .map(|(&token, &count)| {
                let lp_n = smoothed_ln(count, total, vocab);
                let lp_p = smoothed_ln(
                    parent_dist.get(&token).copied().unwrap_or(0),
                    parent_total,
                    vocab,
                );
                (token, ScoreQ::from_logprob(lp_n - lp_p))
            })
            .collect();
        // Top-E by residual score (score desc, token asc), stored
        // ascending token.
        residuals.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        residuals.truncate(config.emission_entries);
        residuals.sort_by_key(|&(token, _)| token);
        region_lists.push(residuals);
    }

    EmissionTables {
        root_prior,
        root_floor,
        root_total,
        region_lists,
    }
}

/// One canonical edge of the scored graph during emission.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WireEdge {
    pub src: u32,
    pub kind: u8,
    pub dst: u32,
    pub score_q: ScoreQ,
}

/// Theorem 7 (full wiring, stronger than the format's v0 existence
/// approximation): the reverse index is a permutation of the canonical
/// edge IDs, sorted by `(dst, src, kind)`; per-dst runs are contiguous
/// and exactly match the PackedNode forward ranges.
pub fn verify_theorem_7_wired(
    edges: &[WireEdge],
    reverse: &[u32],
    forward_start: &[u32],
    forward_len: &[u16],
) -> Result<(), String> {
    if reverse.len() != edges.len() {
        return Err("Theorem 7 violation: reverse index does not cover all edges".to_owned());
    }
    let mut seen = vec![false; edges.len()];
    for &id in reverse {
        let Some(slot) = seen.get_mut(id as usize) else {
            return Err(
                "Theorem 7 violation: invalid canonical edge ID in reverse index".to_owned(),
            );
        };
        if *slot {
            return Err("Theorem 7 violation: duplicate edge ID in reverse index".to_owned());
        }
        *slot = true;
    }
    if seen.iter().any(|s| !s) {
        return Err("Theorem 7 violation: reverse index is not a permutation".to_owned());
    }
    for pair in reverse.windows(2) {
        let a = edges[pair[0] as usize];
        let b = edges[pair[1] as usize];
        if (a.dst, a.src, a.kind) > (b.dst, b.src, b.kind) {
            return Err(
                "Theorem 7 violation: reverse index not sorted by (dst, src, kind)".to_owned(),
            );
        }
    }
    for (node, (&start, &len)) in forward_start.iter().zip(forward_len.iter()).enumerate() {
        let end = start as usize + len as usize;
        if end > reverse.len() {
            return Err("Theorem 7 violation: forward range out of bounds".to_owned());
        }
        for &id in &reverse[start as usize..end] {
            if edges[id as usize].dst != node as u32 {
                return Err("Theorem 7 violation: reverse range target mismatched node".to_owned());
            }
        }
    }
    Ok(())
}

/// What an [`emit_scored_r4g1`] call produced, for the report and tests.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScoredGraphInfo {
    pub node_count: u32,
    pub edge_count: u32,
    pub refinement_edges: u32,
    pub neighbor_edges: u32,
    pub forward_edges: u32,
    pub depth_count: u8,
    pub max_frontier_width: u16,
    pub max_emission_entries: u32,
    pub root_prior_entries: u32,
    pub emission_list_entries: u32,
    pub exct_bytes: u32,
    pub artifact_bytes: usize,
}

/// Data bundle for [`emit_scored_r4g1`] (keeps the argument list
/// focused): the graph content that becomes the NODE/EDGE/EMIT/EXCT
/// sections.
pub struct ScoredGraphSections<'a> {
    /// Region parameters, ascending region id.
    pub regions: &'a [RegionParams],
    /// Structural (E_r/E_o) edges with their stored scores.
    pub structural: &'a [StructuralEdge],
    /// Compiled forward transition edges (E_f).
    pub transitions: &'a [TransitionEdge],
    /// Root prior + per-region residual lists.
    pub emissions: &'a EmissionTables,
    /// Raw TLS1 container bytes (the EXCT carryover).
    pub exct_tls1: &'a [u8],
}

/// Emit the scored graph as an R4G1 container: the cover's HEAD/NODE/
/// ROUT conventions with E_f merged into EDGE (kind tags distinguish
/// E_r/E_o/E_f), the EMIT residual tables with per-node ranges wired,
/// and the TLS1 EXCT carryover. Fails closed: Theorem 7 is verified
/// before serialization, and the bytes are re-validated with
/// `GraphView::parse` + `verify_cids` before they are returned.
pub fn emit_scored_r4g1(
    artifact_container: &[u8],
    corpus_cid_material: (&[u8], &[u8]),
    vocab_size: u32,
    sections: &ScoredGraphSections,
) -> Result<(Vec<u8>, ScoredGraphInfo), String> {
    let ScoredGraphSections {
        regions,
        structural,
        transitions,
        emissions,
        exct_tls1,
    } = *sections;
    if regions.len() != emissions.region_lists.len() {
        return Err("emission lists do not match the region count".to_owned());
    }
    let node_count = 1 + regions.len() as u32;
    let max_depth = regions.iter().map(|r| r.depth as usize).max().unwrap_or(1);
    let depth_count = (max_depth + 1) as u8;

    // Canonical edge array: structural (E_r/E_o) + forward (E_f),
    // sorted by (src, kind, dst) — refinement children contiguous.
    let mut edges: Vec<WireEdge> = structural
        .iter()
        .map(|e| WireEdge {
            src: e.src,
            kind: e.kind,
            dst: e.dst,
            score_q: e.score_q,
        })
        .collect();
    for t in transitions {
        edges.push(WireEdge {
            src: t.src,
            kind: EDGE_KIND_FORWARD,
            dst: t.dst,
            score_q: t.score,
        });
    }
    edges.sort_by_key(|e| (e.src, e.kind, e.dst));
    edges.dedup_by_key(|e| (e.src, e.kind, e.dst));
    let edge_count = edges.len() as u32;
    let refinement_edges = edges
        .iter()
        .filter(|e| e.kind == EDGE_KIND_REFINEMENT)
        .count() as u32;
    let neighbor_edges = edges
        .iter()
        .filter(|e| e.kind == EDGE_KIND_NEIGHBOR)
        .count() as u32;
    let forward_edges = edge_count - refinement_edges - neighbor_edges;

    // Per-node refinement child ranges + reverse index and per-dst
    // forward ranges (cover/convert conventions).
    let node_total = node_count as usize;
    let mut child_start = vec![0u32; node_total];
    let mut child_len = vec![0u16; node_total];
    for (i, edge) in edges.iter().enumerate() {
        if edge.kind != EDGE_KIND_REFINEMENT {
            continue;
        }
        if child_len[edge.src as usize] == 0 {
            child_start[edge.src as usize] = i as u32;
        }
        child_len[edge.src as usize] += 1;
    }
    let max_child_len = child_len[1..].iter().copied().max().unwrap_or(0);
    let max_frontier_width = DEFAULT_MAX_FRONTIER_WIDTH.max(max_child_len);
    let mut reverse: Vec<u32> = (0..edge_count).collect();
    reverse.sort_by_key(|&id| {
        let e = edges[id as usize];
        (e.dst, e.src, e.kind)
    });
    let mut forward_start = vec![0u32; node_total];
    let mut forward_len = vec![0u16; node_total];
    for (i, &id) in reverse.iter().enumerate() {
        let dst = edges[id as usize].dst as usize;
        if forward_len[dst] == 0 {
            forward_start[dst] = i as u32;
        }
        forward_len[dst] += 1;
    }
    verify_theorem_7_wired(&edges, &reverse, &forward_start, &forward_len)?;

    // EMIT: descriptor + root prior block + per-region lists; wire the
    // per-node ranges as we lay the lists down.
    let mut emit = vec![2u8, 0, 0, 0]; // {width: i32, shift: 0, zero_point: 0}
    let root_entry_count = u32::try_from(emissions.root_prior.len())
        .map_err(|_| "root prior exceeds u32 entries".to_owned())?;
    emit.extend_from_slice(&root_entry_count.to_le_bytes());
    let root_total = emissions.root_total.min(u32::MAX as u64) as u32;
    emit.extend_from_slice(&root_total.to_le_bytes());
    emit.extend_from_slice(&emissions.root_floor.raw().to_le_bytes());
    emit.extend_from_slice(&0u32.to_le_bytes()); // reserved
    for (&token, &value) in &emissions.root_prior {
        let token =
            i32::try_from(token).map_err(|_| format!("root prior token {token} exceeds i32"))?;
        emit.extend_from_slice(&token.to_le_bytes());
        emit.extend_from_slice(&value.raw().to_le_bytes());
    }
    let mut emission_start = vec![0u32; node_total];
    let mut emission_len = vec![0u16; node_total];
    let mut emission_list_entries = 0u32;
    for (region_id, list) in emissions.region_lists.iter().enumerate() {
        let node = 1 + region_id;
        emission_start[node] = (emit.len() - 4) as u32; // remainder-relative
        emission_len[node] = u16::try_from(list.len())
            .map_err(|_| "emission list exceeds u16 entries".to_owned())?;
        for &(token, value) in list {
            let token =
                i32::try_from(token).map_err(|_| format!("emission token {token} exceeds i32"))?;
            emit.extend_from_slice(&token.to_le_bytes());
            emit.extend_from_slice(&value.raw().to_le_bytes());
        }
        emission_list_entries += list.len() as u32;
    }
    let max_emission_entries = DEFAULT_MAX_EMISSION_ENTRIES
        .max(emission_len[1..].iter().copied().max().unwrap_or(0) as u32);

    // ROUT: [HALT + padding][(1 + R) × W prototype words][same masks].
    let sig_words = SIG_WORDS as u32;
    let mut rout = Vec::with_capacity(8 + node_total * SIG_WORDS * 8 * 2);
    rout.push(0x00); // HALT
    rout.extend_from_slice(&[0u8; 7]); // program padding to 8-byte alignment
    rout.extend_from_slice(&[0u8; SIG_WORDS * 8]); // root prototype: zeros
    for region in regions {
        let mut words = [0u8; SIG_WORDS * 8];
        words[..SIG_BYTES].copy_from_slice(&region.sig);
        rout.extend_from_slice(&words);
    }
    rout.extend_from_slice(&[0u8; SIG_WORDS * 8]); // root mask: zeros
    for _ in regions {
        let mut words = [0u8; SIG_WORDS * 8];
        words[..SIG_BYTES].fill(0xFF); // all-ones mask (v1)
        rout.extend_from_slice(&words);
    }

    // NODE: the root record is all zeros; regions follow ascending id.
    let mut node_section = Vec::with_capacity(node_total * 30);
    node_section.extend_from_slice(&[0u8; 30]);
    for (index, region) in regions.iter().enumerate() {
        let i = 1 + index;
        node_section.extend_from_slice(&child_start[i].to_le_bytes());
        node_section.extend_from_slice(&child_len[i].to_le_bytes());
        node_section.extend_from_slice(&forward_start[i].to_le_bytes());
        node_section.extend_from_slice(&forward_len[i].to_le_bytes());
        node_section.extend_from_slice(&emission_start[i].to_le_bytes());
        node_section.extend_from_slice(&emission_len[i].to_le_bytes());
        node_section.extend_from_slice(&(1 + (i as u32) * sig_words).to_le_bytes());
        node_section.extend_from_slice(&(1 + (node_count + i as u32) * sig_words).to_le_bytes());
        node_section.extend_from_slice(&region.radius.to_le_bytes());
        node_section.push(region.depth);
        node_section.push(0); // flags
    }

    // EDGE: canonical records followed by the reverse index.
    let mut edge_section = Vec::with_capacity(edges.len() * 20);
    for edge in &edges {
        edge_section.extend_from_slice(&edge.src.to_le_bytes());
        edge_section.extend_from_slice(&edge.dst.to_le_bytes());
        edge_section.extend_from_slice(&edge.score_q.raw().to_le_bytes());
        edge_section.push(edge.kind);
        edge_section.push(0); // flags
        edge_section.extend_from_slice(&0u16.to_le_bytes()); // reserved
    }
    for &id in &reverse {
        edge_section.extend_from_slice(&id.to_le_bytes());
    }

    // EXCT: descriptor + the raw TLS1 carryover.
    let mut exct = Vec::with_capacity(4 + exct_tls1.len());
    exct.extend_from_slice(&[2, 0, 0, 0]);
    exct.extend_from_slice(exct_tls1);

    // HEAD: the fixed 224-byte v0 prefix (convert_r4g1 conventions).
    let (meta, recs) = corpus_cid_material;
    let mut corpus_hasher = blake3::Hasher::new();
    corpus_hasher.update(meta);
    corpus_hasher.update(recs);
    let mut head = Vec::with_capacity(224);
    head.extend_from_slice(blake3::hash(artifact_container).as_bytes()); // teacher_cid
    head.extend_from_slice(&[0u8; 32]); // tokenizer_cid: not carried
    head.extend_from_slice(corpus_hasher.finalize().as_bytes()); // corpus_construction_cid
    head.extend_from_slice(&[0u8; 32]); // corpus_certification_cid: zeroed
    head.extend_from_slice(&[0u8; 20]); // hf_revision: zeroed
    head.extend_from_slice(blake3::hash(COMPILER_VERSION_LABEL).as_bytes());
    head.extend_from_slice(&max_frontier_width.to_le_bytes()); // A
    head.extend_from_slice(&MAX_CANDIDATES.to_le_bytes()); // C
    head.extend_from_slice(&(SIG_WORDS as u16).to_le_bytes()); // W
    head.extend_from_slice(&SHORTLIST_SIZE.to_le_bytes()); // K
    head.extend_from_slice(&max_emission_entries.to_le_bytes()); // E
    head.extend_from_slice(&MAX_PROGRAM_STEPS.to_le_bytes()); // D
    head.extend_from_slice(&node_count.to_le_bytes());
    head.extend_from_slice(&edge_count.to_le_bytes());
    head.push(depth_count);
    head.extend_from_slice(&[0u8; 5]); // fallback policy: unset
    head.extend_from_slice(&[0u8; 2]); // reserved
    head.extend_from_slice(&(SIG_BYTES as u16).to_le_bytes()); // signature_bytes
    head.extend_from_slice(&0u16.to_le_bytes()); // min_runtime_major
    head.extend_from_slice(&0u16.to_le_bytes()); // min_runtime_minor
    head.extend_from_slice(&0u16.to_le_bytes()); // feature_bits_required
    head.extend_from_slice(&vocab_size.to_le_bytes());
    debug_assert_eq!(head.len(), 224);

    let mut builder = uor_r4_graph_format::ArtifactBuilder::new(6);
    builder.add_section(uor_r4_graph_format::SectionId::HEAD, 0, &head);
    builder.add_section(uor_r4_graph_format::SectionId::NODE, 0, &node_section);
    builder.add_section(uor_r4_graph_format::SectionId::EDGE, 0, &edge_section);
    builder.add_section(uor_r4_graph_format::SectionId::ROUT, 0, &rout);
    builder.add_section(uor_r4_graph_format::SectionId::EMIT, 0, &emit);
    builder.add_section(uor_r4_graph_format::SectionId::EXCT, 0, &exct);
    let bytes = builder
        .build()
        .map_err(|error| format!("R4G1 serialization failed: {error}"))?;

    // Fail closed: never emit an artifact the two-stage validator or the
    // integrity CIDs reject.
    let view = uor_r4_graph_format::GraphView::parse(&bytes)
        .map_err(|error| format!("score emitted an invalid R4G1 artifact: {error}"))?;
    view.verify_cids()
        .map_err(|error| format!("score emitted an artifact with bad CIDs: {error}"))?;

    let artifact_bytes = bytes.len();
    Ok((
        bytes,
        ScoredGraphInfo {
            node_count,
            edge_count,
            refinement_edges,
            neighbor_edges,
            forward_edges,
            depth_count,
            max_frontier_width,
            max_emission_entries,
            root_prior_entries: root_entry_count,
            emission_list_entries,
            exct_bytes: exct.len() as u32,
            artifact_bytes,
        },
    ))
}

/// Recover the scoring inputs of a previously emitted cover or scored
/// R4G1 artifact: region parameters and the structural (non-forward)
/// edges. Used by the `--cover` CLI path; byte-identical to the
/// re-induced inputs by construction (deterministic double-run).
pub fn recover_from_artifact(
    r4g1: &[u8],
) -> Result<(Vec<RegionParams>, Vec<StructuralEdge>), String> {
    let view = uor_r4_graph_format::GraphView::parse(r4g1)
        .map_err(|error| format!("invalid cover artifact: {error}"))?;
    view.verify_cids()
        .map_err(|error| format!("cover artifact has bad CIDs: {error}"))?;
    let regions = regions_from_view(&view)?;
    let structural = structural_edges_from_view(&view);
    Ok((regions, structural))
}

/// Convert an induced cover into the scoring region parameters.
pub fn regions_from_cover(cover: &cover::Cover) -> Vec<RegionParams> {
    cover
        .regions
        .iter()
        .map(|region| RegionParams {
            node: cover::region_node_id(region.id),
            depth: region.depth,
            radius: region.radius,
            sig: region.sig,
            parent: region.parent,
        })
        .collect()
}

/// Convert the cover's canonical edges into structural edges (score 0 —
/// the cover carries no log-domain edge scores; E_f is compiled here).
pub fn structural_from_cover(edges: &[cover::CoverEdge]) -> Vec<StructuralEdge> {
    edges
        .iter()
        .map(|e| StructuralEdge {
            src: e.src,
            kind: e.kind,
            dst: e.dst,
            score_q: ScoreQ::ZERO,
        })
        .collect()
}

/// Witten-Bell backoff probability of `next` under the graded store —
/// the `evaluate-report` bits/token semantics, shared so the Gate C
/// baseline and the HF evaluation report compute identical numbers.
pub fn witten_bell_probability(store: &Store, code: &[u8; STAGES], next: u32) -> f64 {
    let mut levels: Vec<(f64, &BTreeMap<u32, u32>, u32)> = Vec::new();
    for (depth, level) in store.iter().enumerate().take(STAGES + 1) {
        let key = code[..depth].to_vec();
        if let Some(distribution) = level.get(&key) {
            let total: u32 = distribution.values().sum();
            let lambda = total as f64 / (total as f64 + distribution.len() as f64);
            levels.push((lambda, distribution, total));
        }
    }
    let mut remaining = 1.0f64;
    let mut probability = 0.0f64;
    for index in (0..levels.len()).rev() {
        let weight = remaining * levels[index].0;
        remaining *= 1.0 - levels[index].0;
        if let Some(&count) = levels[index].1.get(&next) {
            probability += weight * count as f64 / levels[index].2 as f64;
        }
    }
    (probability + remaining / compiler::V as f64).max(1e-30)
}

/// Certifier-side bits/token of one graph-scorer outcome: the candidate
/// scores treated as natural-log weights (ScoreQ carries ln × 2⁻¹⁶),
/// non-candidate tokens held at the baked root smoothing floor (f64,
/// deterministic same-platform; computed max-shifted in the natural-log
/// domain so extreme residuals cannot underflow the accumulator).
fn outcome_bits(scorer: &GraphScorer, candidates: &[(u32, ScoreQ)], next: u32) -> f64 {
    let floor = scorer.root_floor().raw();
    let max_s = candidates
        .iter()
        .map(|&(_, s)| s.raw())
        .max()
        .unwrap_or(floor)
        .max(floor);
    let weight = |s: i32| ((f64::from(s) - f64::from(max_s)) / 65536.0).exp();
    let mut sum = 0f64;
    let mut w_next = None;
    for &(token, score) in candidates {
        let w = weight(score.raw());
        sum += w;
        if token == next {
            w_next = Some(w);
        }
    }
    let w_floor = weight(floor);
    let uncovered = (scorer.vocab() as usize).saturating_sub(candidates.len());
    sum += uncovered as f64 * w_floor;
    let w = w_next.unwrap_or(w_floor).max(1e-300);
    (sum / w).ln() / std::f64::consts::LN_2
}

/// One metric set of the Gate C table.
#[derive(Debug, Clone, Default, Serialize)]
pub struct GateCMetrics {
    pub positions: usize,
    /// P(selected token == recorded teacher argmax).
    pub top1_agreement: f64,
    pub bits_per_token: f64,
}

/// The Gate C outcome: the three number sets plus the witness-replay
/// sample result.
#[derive(Debug, Clone, Default, Serialize)]
pub struct GateCOutcome {
    pub graph_no_exct: GateCMetrics,
    pub graph_with_exct: GateCMetrics,
    pub tla3_baseline: GateCMetrics,
    /// Candidate-set recall is reported separately from selected-token
    /// agreement. A low value means the scorer cannot recover the teacher
    /// token regardless of how its weights are tuned.
    pub candidate_recall: CandidateRecall,
    pub witness_replays: usize,
    pub witness_replay_failures: usize,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct CandidateRecall {
    pub graph_no_exct_top1: f64,
    pub graph_no_exct_top3: f64,
    pub graph_with_exct_top1: f64,
    pub graph_with_exct_top3: f64,
}

/// The Gate C measurement (plan §8 gate C): top-1 teacher-argmax
/// agreement and bits/token on the held-out partition for the graph
/// scorer without EXCT, with EXCT, and the TLA3 store baseline on the
/// same positions. Both graph scorers are rebuilt from the emitted
/// artifact bytes (the artifact is the scoring authority); a bounded
/// sample of witnesses is independently replayed (Theorem 6).
pub fn evaluate_gate_c(
    r4g1: &[u8],
    artifact_container: &[u8],
    artifacts: &compiler::Compiled,
    store: &Store,
    corpus: &Corpus,
    held_out: &[Observation],
    config: &ScoreConfig,
) -> Result<GateCOutcome, String> {
    let scorer_no_exct =
        GraphScorer::from_artifact(r4g1, None, config.root_top_b, config.exct_top_x)?;
    let scorer_with_exct = GraphScorer::from_artifact(
        r4g1,
        Some(artifact_container),
        config.root_top_b,
        config.exct_top_x,
    )?;

    let mut outcome = GateCOutcome::default();
    let mut bits_no_exct = 0f64;
    let mut bits_with_exct = 0f64;
    let mut bits_baseline = 0f64;
    let mut hits_no_exct = 0u64;
    let mut hits_with_exct = 0u64;
    let mut hits_baseline = 0u64;
    let mut candidate_top1_no_exct = 0u64;
    let mut candidate_top3_no_exct = 0u64;
    let mut candidate_top1_with_exct = 0u64;
    let mut candidate_top3_with_exct = 0u64;
    for (index, observation) in held_out.iter().enumerate() {
        let position = observation.position as usize;
        let teacher_argmax = corpus.t_argmax[position];
        let next = corpus.next[position];
        let code = runtime::assign_plain(artifacts, &observation.sig);

        let no_exct = scorer_no_exct.score_candidates(&observation.sig)?;
        let with_exct = scorer_with_exct.score_candidates(&observation.sig)?;
        let baseline = runtime::predict_witness_plain(store, &code);

        let contains = |candidates: &[(u32, ScoreQ)], token: u32| {
            candidates.iter().any(|&(candidate, _)| candidate == token)
        };
        if contains(&no_exct.candidates, teacher_argmax) {
            candidate_top1_no_exct += 1;
        }
        if contains(&with_exct.candidates, teacher_argmax) {
            candidate_top1_with_exct += 1;
        }
        if corpus.top_tokens[position]
            .iter()
            .any(|&token| contains(&no_exct.candidates, token))
        {
            candidate_top3_no_exct += 1;
        }
        if corpus.top_tokens[position]
            .iter()
            .any(|&token| contains(&with_exct.candidates, token))
        {
            candidate_top3_with_exct += 1;
        }

        if no_exct.selected == teacher_argmax {
            hits_no_exct += 1;
        }
        if with_exct.selected == teacher_argmax {
            hits_with_exct += 1;
        }
        if baseline.token == teacher_argmax {
            hits_baseline += 1;
        }
        bits_no_exct += outcome_bits(&scorer_no_exct, &no_exct.candidates, next);
        bits_with_exct += outcome_bits(&scorer_with_exct, &with_exct.candidates, next);
        bits_baseline += -witten_bell_probability(store, &code, next).log2();

        if index < config.witness_sample {
            outcome.witness_replays += 1;
            if verify_witness_replay(
                r4g1,
                Some(artifact_container),
                &with_exct.witness,
                config.root_top_b,
                config.exct_top_x,
            )
            .is_err()
            {
                outcome.witness_replay_failures += 1;
            }
        }
    }
    let n = held_out.len();
    if n == 0 {
        return Err("held-out split is empty; cannot evaluate".to_owned());
    }
    let nf = n as f64;
    outcome.graph_no_exct = GateCMetrics {
        positions: n,
        top1_agreement: hits_no_exct as f64 / nf,
        bits_per_token: bits_no_exct / nf,
    };
    outcome.graph_with_exct = GateCMetrics {
        positions: n,
        top1_agreement: hits_with_exct as f64 / nf,
        bits_per_token: bits_with_exct / nf,
    };
    outcome.tla3_baseline = GateCMetrics {
        positions: n,
        top1_agreement: hits_baseline as f64 / nf,
        bits_per_token: bits_baseline / nf,
    };
    outcome.candidate_recall = CandidateRecall {
        graph_no_exct_top1: candidate_top1_no_exct as f64 / nf,
        graph_no_exct_top3: candidate_top3_no_exct as f64 / nf,
        graph_with_exct_top1: candidate_top1_with_exct as f64 / nf,
        graph_with_exct_top3: candidate_top3_with_exct as f64 / nf,
    };
    Ok(outcome)
}

/// The `score_report.json` document.
#[derive(Debug, Clone, Serialize)]
pub struct ScoreReport {
    pub schema: u32,
    pub inputs: ScoreReportInputs,
    pub config: ScoreReportConfig,
    pub graph: ScoreReportGraph,
    pub gate_c: GateCOutcome,
    pub quantization: ScoreReportQuantization,
    pub determinism: ScoreReportDeterminism,
}

#[derive(Debug, Clone, Serialize)]
pub struct ScoreReportInputs {
    pub artifact_kappa: String,
    pub corpus_kappa: String,
    pub cover_source: String,
    pub graph_kappa: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ScoreReportConfig {
    pub transition_out_degree: usize,
    pub emission_entries: usize,
    pub root_top_b: usize,
    pub exct_top_x: usize,
    pub witness_sample: usize,
    pub top_m: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct ScoreReportGraph {
    pub node_count: u32,
    pub edge_count: u32,
    pub refinement_edges: u32,
    pub neighbor_edges: u32,
    pub forward_edges: u32,
    pub depth_count: u8,
    pub root_prior_entries: u32,
    pub emission_list_entries: u32,
    pub exct_bytes: u32,
    pub artifact_bytes: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct ScoreReportQuantization {
    pub format: String,
    pub smoothing: String,
    pub platform: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ScoreReportDeterminism {
    pub note: String,
}

/// Assemble the report from a finished run.
pub fn build_score_report(
    config: &ScoreConfig,
    inputs: ScoreReportInputs,
    info: &ScoredGraphInfo,
    gate_c: GateCOutcome,
) -> ScoreReport {
    ScoreReport {
        schema: 1,
        inputs,
        config: ScoreReportConfig {
            transition_out_degree: config.transition_out_degree,
            emission_entries: config.emission_entries,
            root_top_b: config.root_top_b,
            exct_top_x: config.exct_top_x,
            witness_sample: config.witness_sample,
            top_m: super::score_runtime::TOP_M,
        },
        graph: ScoreReportGraph {
            node_count: info.node_count,
            edge_count: info.edge_count,
            refinement_edges: info.refinement_edges,
            neighbor_edges: info.neighbor_edges,
            forward_edges: info.forward_edges,
            depth_count: info.depth_count,
            root_prior_entries: info.root_prior_entries,
            emission_list_entries: info.emission_list_entries,
            exct_bytes: info.exct_bytes,
            artifact_bytes: info.artifact_bytes,
        },
        gate_c,
        quantization: ScoreReportQuantization {
            format: "ScoreQ Q16.16 in i32; EMIT storage descriptor {width: i32, shift: 0, \
                     zero_point: 0}; edge weights and residuals via ScoreQ::from_logprob"
                .to_owned(),
            smoothing: "add-one over the compiled vocabulary: P(v|n) = (count_n(v) + 1) / \
                        (total_n + V); evidence = the store's top-3 teacher-weighted counts \
                        over covered binary top-1 members (within calibrated radius; no \
                        backoff-floor assignment); root prior = level-0 store distribution; \
                        smoothing floor baked into the EMIT root header"
                .to_owned(),
            platform: "compiler-side f64 ln quantization is macOS-pinned (libm-sensitive \
                       cross-platform), the same status as the existing κ baseline; the D2 \
                       canonical deterministic compile mode resolves cross-platform byte \
                       equality later. The runtime scoring path is integer-only except the \
                       documented EXCT probe-time quantization (Phase-5 residualized EXCT \
                       removes it)"
                .to_owned(),
        },
        determinism: ScoreReportDeterminism {
            note: "content-addressed observation order; all reductions are B-tree (ordered) \
                   accumulations over counts, so shard/observation order never reaches the \
                   bytes; canonical sorts everywhere; identical inputs produce byte-identical \
                   artifacts and reports"
                .to_owned(),
        },
    }
}
