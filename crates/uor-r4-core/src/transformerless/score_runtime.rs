//! The Phase-4 reference scorer: the witness-replayable, integer-only
//! scoring model of the graph-compiler plan (§5 Phase 4, §6 runtime
//! contract, glossary "Scoring model"):
//!
//! ```text
//! S(v) = B(v) + Σ_{n∈A} ΔE(n,v) + Σ_{m∈F} ΔT(m,v) + ΔX(X,v)
//! ```
//!
//! over a validated R4G1 artifact produced by [`super::score`]. Every
//! contribution is a [`ScoreQ`] (Q16.16 log-domain integer); the
//! accumulation core — candidate union, contribution application, the
//! canonical argmax — uses ScoreQ saturating add/sub, integer compares,
//! and table reads only: no float, no multiply, no divide, no modulo.
//! The single exception is delimited by `BEGIN/END COMPILER-SIDE FLOAT`
//! markers and documented at [`quantize_exct`]; the machine-checked
//! source scan (`tests/score.rs`) asserts the rest of this file carries
//! no `f32`/`f64` and no `*` `/` `%` value arithmetic.
//!
//! # Scoring semantics (normative)
//!
//! - **Active cloud A**: the top-`TOP_M` masked-Hamming memberships at
//!   each depth 1..=max_depth — exactly the
//!   [`super::cover::ReferenceClassifier::binary_memberships`] semantics
//!   (top-M by distance, ties to the lower region id, within-radius
//!   filter, nearest-region fallback), recomputed here over the region
//!   parameters recovered from the artifact's NODE/ROUT sections so the
//!   artifact alone is the scoring authority.
//! - **Predicted cloud F**: the union of E_f edge targets of the active
//!   regions. Each predicted region m keeps its single best incoming
//!   edge (highest `score_q`, ties to the lowest canonical edge id).
//! - **ΔT(m,v)** := w(n_m→m) + ΔE(m,v). The edge-weight part is
//!   token-independent, so it is algebraically folded into the scalar
//!   `transition_offset = Σ_m w(n_m→m)` added to every candidate once
//!   (addition commutes; the witness records the applied edges and the
//!   offset). The emission part applies per candidate as usual.
//! - **Candidates**: the union of the emission lists of A ∪ F, the root
//!   prior's top-`root_top_b` tokens (precomputed at construction), and
//!   the EXCT probe's top-`exct_top_x` tokens when EXCT evidence is
//!   wired. Every candidate receives exactly one root-prior contribution
//!   (the stored B(v), or the baked smoothing floor for tokens absent
//!   from the root block).
//! - **Selection**: the canonical tie-break — highest score, then the
//!   lowest token id (matches `runtime::predict_witness_plain`'s rule).
//!
//! # Theorem 10 by construction
//!
//! Contribution IDs are canonical: `RootPrior(token)`, `Emission(node,
//! token)`, `ExactContext(token)`, and one applied-edge id per predicted
//! region. A node's emission list is applied at most once per context
//! even when the node is simultaneously active (ΔE) and predicted (ΔT)
//! — A ∪ F is deduplicated by node before any emission is read — so no
//! emission entry can enter S(v) twice. The induced cover has no
//! explicit overlap nodes, so there are no interaction residuals this
//! phase: the root-plus-residual decomposition attaches every
//! contribution to exactly one node. The witness verifier independently
//! rejects duplicate contribution IDs (belt-and-braces, Theorem 10).
//!
//! # Witness and independent replay (Theorem 6)
//!
//! [`GraphScorer::score_candidates`] emits a bounded [`ScoreWitness`]:
//! graph CID, input code, active regions + margins, predicted cloud,
//! applied transition edges + folded offset, the EXCT probe record (when
//! used), the selected token with its full contribution list and score,
//! the candidate count, and the op census. [`verify_witness_replay`]
//! rebuilds a fresh scorer **from the validated R4G1 bytes** (plus the
//! teacher TLA container when EXCT evidence is present — checked against
//! HEAD `teacher_cid`, so the class-code derivation chains to pinned
//! content), recomputes the entire prediction without any compiler
//! state, and requires bit-exact equality of every witness field;
//! duplicate contribution IDs are rejected out of hand.
//!
//! # Op census
//!
//! Hamming membership runs through [`OpKernel`] (xor + popcount table +
//! add per byte), and emission/root/EXCT application, the candidate
//! scan, and the argmax are counted as adds / compares / table reads /
//! candidate scans. Bounded lookup structures (the root-prior map and
//! per-node emission maps) are built once at scorer construction — the
//! reference form of the deployed runtime's fixed tables; per-token
//! scoring itself performs no allocation-dependent work beyond the
//! candidate accumulation map (the deployed Phase-5 runtime replaces it
//! with fixed-capacity arrays).

use std::collections::{BTreeMap, BTreeSet};

use uor_r4_graph_format::{GraphView, ScoreQ, SectionId};

use super::compiler::{self, Compiled, SIG_BYTES, STAGES};
use super::runtime::{self, OpKernel, Store};

/// Edge kind of refinement (parent/child) edges (cover/transitions).
pub const EDGE_KIND_REFINEMENT: u8 = 0;
/// Edge kind of lateral neighbor (co-activation) edges.
pub const EDGE_KIND_NEIGHBOR: u8 = 1;
/// Edge kind of forward transition edges (E_f).
pub const EDGE_KIND_FORWARD: u8 = 2;

/// Bounded multi-membership per depth (matches `cover::TOP_M` and the
/// runtime's top-M).
pub const TOP_M: usize = 3;

/// Byte length of one EMIT entry: `(i32 token, i32 score_q)`.
pub const EMIT_ENTRY_BYTES: usize = 8;
/// Byte length of the EMIT root-prior block header:
/// `(u32 entry_count, u32 total_count, i32 floor_score_q, u32 reserved)`.
pub const EMIT_ROOT_HEADER_BYTES: usize = 16;

/// The region parameters the scoring path reads from the artifact:
/// node identity, multiresolution depth, calibrated radius, and the
/// packed sign-bit prototype. The f32 prototypes stay compiler-side;
/// membership here is the shipped binary (masked-Hamming) semantics.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegionParams {
    /// Artifact node id (region id = node − 1; the root is node 0).
    pub node: u32,
    /// Multiresolution depth (1..=max_depth).
    pub depth: u8,
    /// Calibrated acceptance radius (masked-Hamming bound).
    pub radius: u16,
    /// Binarized prototype signature.
    pub sig: [u8; SIG_BYTES],
    /// Parent region id (`None` at depth 1 — the parent is the root).
    pub parent: Option<u32>,
}

impl RegionParams {
    /// Region id of this node (index into the region vector).
    pub fn region_id(&self) -> u32 {
        self.node - 1
    }
}

/// Region parameters recovered from a validated artifact view: one entry
/// per region (node 1..=node_count−1), ascending node id. The parent of
/// a region is the source of its refinement edge (the root maps to
/// `None`); a region without a refinement parent maps to `None` as well
/// (every emitted region has exactly one).
pub fn regions_from_view(view: &GraphView) -> Result<Vec<RegionParams>, String> {
    let head = view.head().ok_or("artifact carries no HEAD section")?;
    let rout = view
        .section(SectionId::ROUT)
        .ok_or("artifact carries no ROUT section")?;
    let node_count = head.node_count();
    if node_count == 0 {
        return Err("HEAD declares zero nodes".to_owned());
    }
    // Parents from refinement edges (dst node -> src node).
    let mut parent_of: BTreeMap<u32, u32> = BTreeMap::new();
    for edge in view.edges() {
        if edge.kind == EDGE_KIND_REFINEMENT {
            parent_of.insert(edge.dst.0, edge.src.0);
        }
    }
    let signature_bytes = head.signature_bytes() as usize;
    let mut regions = Vec::with_capacity((node_count - 1) as usize);
    let mut node_index = 1u32;
    while node_index < node_count {
        let node = view
            .node(node_index)
            .ok_or("node record missing within declared count")?;
        let start = (node.prototype_word_start as usize) << 3;
        let window = rout
            .get(start..start + signature_bytes)
            .ok_or("prototype window outside ROUT")?;
        let mut sig = [0u8; SIG_BYTES];
        sig.copy_from_slice(window);
        let parent =
            parent_of
                .get(&node_index)
                .and_then(|&src| if src == 0 { None } else { Some(src - 1) });
        regions.push(RegionParams {
            node: node_index,
            depth: node.depth.0,
            radius: node.radius.0,
            sig,
            parent,
        });
        node_index += 1;
    }
    Ok(regions)
}

/// One structural edge recovered from the canonical array (E_r/E_o with
/// their stored scores — E_f is recompiled, not carried over).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StructuralEdge {
    pub src: u32,
    pub kind: u8,
    pub dst: u32,
    pub score_q: ScoreQ,
}

/// The non-forward edges of a validated artifact view, in canonical
/// edge-id order.
pub fn structural_edges_from_view(view: &GraphView) -> Vec<StructuralEdge> {
    view.edges()
        .filter(|edge| edge.kind != EDGE_KIND_FORWARD)
        .map(|edge| StructuralEdge {
            src: edge.src.0,
            kind: edge.kind,
            dst: edge.dst.0,
            score_q: edge.score_q,
        })
        .collect()
}

/// Kernel-counted Hamming distance between two signatures.
fn hamming_counted(
    k: &mut OpKernel,
    pop: &[u8; 256],
    a: &[u8; SIG_BYTES],
    b: &[u8; SIG_BYTES],
) -> u32 {
    let mut dist = 0u32;
    for (&x, &y) in a.iter().zip(b.iter()) {
        let xored = k.xor(x, y);
        let ones = k.table_u8(pop, xored);
        dist = k.add(dist as i64, ones as i64) as u32;
    }
    dist
}

/// The shipped binary membership at `depth` — the
/// `ReferenceClassifier::binary_memberships` semantics over recovered
/// region parameters: scan regions in ascending node-id order keeping
/// the top-[`TOP_M`] by masked-Hamming distance (strict `<` insertion,
/// so ties go to the lower id), filter to those within their calibrated
/// radius, and fall back to the nearest region when nothing is in
/// range. Returns `(region_id, distance)` pairs in distance order.
pub fn binary_memberships(
    k: &mut OpKernel,
    pop: &[u8; 256],
    regions: &[RegionParams],
    depth: usize,
    sig: &[u8; SIG_BYTES],
) -> Vec<(u32, u32)> {
    let mut top: Vec<(u32, u32)> = Vec::with_capacity(TOP_M);
    for region in regions {
        if region.depth as usize != depth {
            continue;
        }
        let dist = hamming_counted(k, pop, sig, &region.sig);
        let mut inserted = false;
        for (idx, &(_, d0)) in top.iter().enumerate() {
            k.compares += 1;
            if dist < d0 {
                top.insert(idx, (region.region_id(), dist));
                inserted = true;
                break;
            }
        }
        if !inserted && top.len() < TOP_M {
            top.push((region.region_id(), dist));
        }
        if inserted && top.len() > TOP_M {
            top.pop();
        }
    }
    if top.is_empty() {
        return Vec::new();
    }
    let within: Vec<(u32, u32)> = top
        .iter()
        .filter(|&&(id, dist)| {
            k.compares += 1;
            dist <= u32::from(regions[id as usize].radius)
        })
        .map(|&(id, dist)| (id, dist))
        .collect();
    if within.is_empty() {
        // Nearest-region fallback (the backoff floor).
        vec![top[0]]
    } else {
        within
    }
}

/// Covered binary top-1 membership at `depth`: the nearest region when
/// — and only when — the context signature lies within its calibrated
/// radius. This is the *evidence* membership (region content), distinct
/// from the routing membership above: the nearest-region fallback is a
/// routing behavior, not region content. Without this rule, the backoff
/// floor would assign every observation to some region at every depth
/// and deep regions' distributions would collect the whole corpus.
pub fn binary_top1_covered(
    k: &mut OpKernel,
    pop: &[u8; 256],
    regions: &[RegionParams],
    depth: usize,
    sig: &[u8; SIG_BYTES],
) -> Option<(u32, u32)> {
    let (region, dist) = binary_memberships(k, pop, regions, depth, sig)
        .into_iter()
        .next()?;
    k.compares += 1;
    if dist <= u32::from(regions[region as usize].radius) {
        Some((region, dist))
    } else {
        None
    }
}

// BEGIN COMPILER-SIDE FLOAT QUANTIZATION ---------------------------------
// The one floating-point site of the reference scoring path (macOS-pinned
// libm, exactly the status of the existing κ baseline; the D2 canonical
// deterministic compile mode resolves cross-platform byte equality later).
// The deployed integer runtime never quantizes at probe time: residualized
// EXCT tables (the Phase-5 EXCT migration) bake these values at compile
// time. This helper is kept out of every accumulation function so the
// S(v) integer core stays machine-checkable by source scan.

/// ΔX probe-time quantization: `ScoreQ::from_logprob(ln P(v|X))` with
/// add-one smoothing over the compiled vocabulary,
/// `P(v|X) = (count + 1) / (total + V)`. The residual form used by the
/// scorer subtracts the stored root prior from this value with integer
/// saturating subtraction.
fn quantize_exct(count: u32, total: u32, vocab: u32) -> ScoreQ {
    let p = (f64::from(count) + 1.0) / (f64::from(total) + f64::from(vocab));
    ScoreQ::from_logprob(p.ln() as f32)
}

// END COMPILER-SIDE FLOAT QUANTIZATION -----------------------------------

/// Canonical identity of one score contribution (Theorem 10): no id may
/// appear twice in one candidate's contribution list. The transition
/// edge-weight contributions are folded into the scalar offset and
/// witnessed once per applied edge (see module docs).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ContributionId {
    /// Root prior B(v) — exactly one per candidate.
    RootPrior { token: u32 },
    /// Emission residual ΔE(node, v) — at most one per node per context.
    Emission { node: u32, token: u32 },
    /// Exact-context residual ΔX(X, v) — at most one per context.
    ExactContext { token: u32 },
}

/// One contribution to a candidate's score: a canonical id and the
/// ScoreQ value applied.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Contribution {
    pub id: ContributionId,
    pub value: ScoreQ,
}

/// One active region of the witness: region identity, depth, masked-
/// Hamming distance to the context signature, and the membership margin
/// (`radius − distance`; negative only on the nearest-region fallback).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActiveRegion {
    pub region: u32,
    pub depth: u8,
    pub distance: u16,
    pub margin: i16,
}

/// One applied transition edge of the witness: the best incoming E_f
/// edge of one predicted region.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EdgeUse {
    pub edge_id: u32,
    pub src: u32,
    pub dst: u32,
    pub score_q: ScoreQ,
}

/// The EXCT probe record of the witness: the deepest populated graded
/// prefix, its evidence total, and the number of probe tokens admitted
/// to the candidate set.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExctProbe {
    pub level: u8,
    pub key: Vec<u8>,
    pub total: u32,
    pub admitted: u32,
}

/// The bounded, replayable record of one prediction (glossary
/// "Witness"; plan §24, Theorem 6). Everything the independent verifier
/// needs is recomputed from the validated artifact bytes; the witness
/// carries only claims.
#[derive(Debug, Clone, PartialEq)]
pub struct ScoreWitness {
    /// `artifact_cid` of the scored R4G1 container.
    pub graph_cid: [u8; 32],
    /// H(x): the context's sign-bit signature.
    pub input_sig: [u8; SIG_BYTES],
    /// Active cloud A (ascending region id) with margins.
    pub active: Vec<ActiveRegion>,
    /// Predicted cloud F (ascending node id).
    pub predicted: Vec<u32>,
    /// Applied transition edges (ascending edge id).
    pub edges_applied: Vec<EdgeUse>,
    /// Σ of the applied edge weights (the folded ΔT edge part).
    pub transition_offset: ScoreQ,
    /// The EXCT probe, when exact-context evidence was consulted.
    pub exct: Option<ExctProbe>,
    /// The selected token and its score (canonical tie-break).
    pub selected: u32,
    pub selected_score: ScoreQ,
    /// The selected token's full contribution list, canonical order.
    pub selected_contributions: Vec<Contribution>,
    /// Size of the scored candidate set.
    pub candidate_count: u32,
    /// The op census of this prediction.
    pub census: OpKernel,
}

/// The outcome of [`GraphScorer::score_candidates`]: the selection plus
/// every candidate's final score (ascending token order) — the certifier
/// reads the distribution for bits/token; the witness stands alone.
#[derive(Debug, Clone)]
pub struct ScoreOutcome {
    pub selected: u32,
    pub selected_score: ScoreQ,
    /// Every candidate `(token, score)` in ascending token order.
    pub candidates: Vec<(u32, ScoreQ)>,
    pub witness: ScoreWitness,
}

/// The reference scorer: validated R4G1 bytes (+ the teacher TLA
/// container when EXCT evidence is wired) parsed once into bounded
/// lookup structures. Construction fails closed — invalid bytes or CIDs
/// never yield a scorer.
pub struct GraphScorer {
    graph_cid: [u8; 32],
    regions: Vec<RegionParams>,
    max_depth: usize,
    /// Forward edges grouped by source node, ascending edge id within.
    forward_by_src: BTreeMap<u32, Vec<EdgeUse>>,
    root_prior: BTreeMap<u32, ScoreQ>,
    root_floor: ScoreQ,
    root_top: Vec<u32>,
    /// Per-region emission maps (index = region id), token → ΔE.
    emissions: Vec<BTreeMap<u32, ScoreQ>>,
    store: Option<Store>,
    artifacts: Option<Compiled>,
    vocab: u32,
    exct_top_x: usize,
    pop: [u8; 256],
}

impl GraphScorer {
    /// Build a scorer from validated R4G1 bytes. `teacher_container` is
    /// the TLA artifact the EXCT class codes derive from; when the
    /// artifact carries an EXCT section, supplying it enables
    /// exact-context evidence (its blake3 must equal HEAD `teacher_cid`
    /// — fail closed, so the evidence chains to the pinned teacher),
    /// and a `None` teacher runs the same artifact without EXCT.
    pub fn from_artifact(
        r4g1: &[u8],
        teacher_container: Option<&[u8]>,
        root_top_b: usize,
        exct_top_x: usize,
    ) -> Result<Self, String> {
        let view = GraphView::parse(r4g1).map_err(|e| format!("invalid R4G1: {e}"))?;
        view.verify_cids().map_err(|e| format!("bad CIDs: {e}"))?;
        let head = view.head().ok_or("artifact carries no HEAD section")?;
        let graph_cid = view.header().artifact_cid.0;
        let regions = regions_from_view(&view)?;
        let max_depth = regions.iter().map(|r| r.depth as usize).max().unwrap_or(0);

        let mut forward_by_src: BTreeMap<u32, Vec<EdgeUse>> = BTreeMap::new();
        for (edge_id, edge) in view.edges().enumerate() {
            if edge.kind == EDGE_KIND_FORWARD {
                forward_by_src.entry(edge.src.0).or_default().push(EdgeUse {
                    edge_id: edge_id as u32,
                    src: edge.src.0,
                    dst: edge.dst.0,
                    score_q: edge.score_q,
                });
            }
        }

        // EMIT: descriptor (stage-2 validated), the root-prior block,
        // and per-node emission lists wired by the packed ranges.
        let emit = view
            .section(SectionId::EMIT)
            .ok_or("artifact carries no EMIT section")?;
        let remainder = emit
            .get(uor_r4_graph_format::STORAGE_DESCRIPTOR_LEN..)
            .ok_or("EMIT section shorter than its descriptor")?;
        let header = remainder
            .get(..EMIT_ROOT_HEADER_BYTES)
            .ok_or("EMIT remainder shorter than the root header")?;
        let root_entry_count = u32::from_le_bytes(header[0..4].try_into().expect("4 bytes"));
        let root_floor = ScoreQ::from_raw(i32::from_le_bytes(
            header[8..12].try_into().expect("4 bytes"),
        ));
        let root_entry_bytes = (root_entry_count as usize) << 3;
        let root_block = remainder
            .get(EMIT_ROOT_HEADER_BYTES..EMIT_ROOT_HEADER_BYTES + root_entry_bytes)
            .ok_or("EMIT root prior block truncated")?;
        let mut root_prior = BTreeMap::new();
        for entry in root_block.chunks_exact(EMIT_ENTRY_BYTES) {
            let token = i32::from_le_bytes(entry[0..4].try_into().expect("4 bytes"));
            let value = i32::from_le_bytes(entry[4..8].try_into().expect("4 bytes"));
            let token = u32::try_from(token).map_err(|_| "EMIT root token is negative")?;
            root_prior.insert(token, ScoreQ::from_raw(value));
        }
        // Top-B root candidates by (score desc, token asc) — context-
        // independent, precomputed once (module docs).
        let mut ranked: Vec<(u32, ScoreQ)> = root_prior.iter().map(|(&t, &s)| (t, s)).collect();
        ranked.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        let root_top: Vec<u32> = ranked
            .into_iter()
            .take(root_top_b)
            .map(|(t, _)| t)
            .collect();

        let node_count = head.node_count();
        let mut emissions = vec![BTreeMap::new(); regions.len()];
        let mut node_index = 1u32;
        while node_index < node_count {
            let node = view
                .node(node_index)
                .ok_or("node record missing within declared count")?;
            let start = node.emission_start as usize;
            let byte_len = (node.emission_len as usize) << 3;
            let list = remainder
                .get(start..start + byte_len)
                .ok_or("node emission list outside the EMIT remainder")?;
            let map = &mut emissions[(node_index - 1) as usize];
            for entry in list.chunks_exact(EMIT_ENTRY_BYTES) {
                let token = i32::from_le_bytes(entry[0..4].try_into().expect("4 bytes"));
                let value = i32::from_le_bytes(entry[4..8].try_into().expect("4 bytes"));
                let token = u32::try_from(token).map_err(|_| "EMIT emission token is negative")?;
                map.insert(token, ScoreQ::from_raw(value));
            }
            node_index += 1;
        }

        // EXCT: the TLS1 carryover (converter convention). When the
        // artifact carries EXCT, consulting it requires the teacher
        // container for class-code derivation; a `None` teacher means
        // exact-context evidence is simply not consulted (Gate C mode
        // (a) runs the same artifact without EXCT).
        let exct = view.section(SectionId::EXCT);
        let (store, artifacts) = match (exct, teacher_container) {
            (Some(bytes), Some(teacher)) => {
                if blake3::hash(teacher).as_bytes() != &head.teacher_cid().0 {
                    return Err("teacher container does not match HEAD teacher_cid".to_owned());
                }
                let parsed = compiler::parse_artifacts(teacher)
                    .ok_or("teacher container is not a TLA artifact container")?;
                let body = bytes
                    .get(uor_r4_graph_format::STORAGE_DESCRIPTOR_LEN..)
                    .ok_or("EXCT section shorter than its descriptor")?;
                let store = runtime::parse_store(body)
                    .or_else(|| runtime::parse_store_legacy_u16(body))
                    .ok_or("EXCT remainder is not a TLS1 store (either era)")?;
                (Some(store), Some(parsed))
            }
            _ => (None, None),
        };

        Ok(GraphScorer {
            graph_cid,
            regions,
            max_depth,
            forward_by_src,
            root_prior,
            root_floor,
            root_top,
            emissions,
            store,
            artifacts,
            vocab: head.vocab_size(),
            exct_top_x,
            pop: runtime::derive_popcount_table(),
        })
    }

    /// True when exact-context evidence is wired.
    pub fn has_exct(&self) -> bool {
        self.store.is_some()
    }

    /// The compiled vocabulary size (HEAD).
    pub fn vocab(&self) -> u32 {
        self.vocab
    }

    /// The root-prior smoothing floor (baked into EMIT).
    pub fn root_floor(&self) -> ScoreQ {
        self.root_floor
    }

    /// The stored root prior B(v), or the smoothing floor for tokens
    /// absent from the root block.
    fn root_score(&self, token: u32) -> ScoreQ {
        self.root_prior
            .get(&token)
            .copied()
            .unwrap_or(self.root_floor)
    }

    /// Score one context signature: compute the active cloud A, the
    /// predicted cloud F, the bounded candidate set, and S(v) for every
    /// candidate by integer ScoreQ accumulation; select by the canonical
    /// tie-break; emit the bounded witness. The arithmetic of this
    /// function is integer-only (module docs; the EXCT probe's ΔX
    /// quantization is the delimited compiler-side exception).
    pub fn score_candidates(&self, sig: &[u8; SIG_BYTES]) -> Result<ScoreOutcome, String> {
        let mut k = OpKernel::default();

        // Active cloud A: top-M memberships at each depth.
        let mut active: Vec<ActiveRegion> = Vec::new();
        for depth in 1..=self.max_depth {
            for (region, dist) in binary_memberships(&mut k, &self.pop, &self.regions, depth, sig) {
                let radius = u32::from(self.regions[region as usize].radius);
                let margin = radius as i64 - dist as i64;
                active.push(ActiveRegion {
                    region,
                    depth: depth as u8,
                    distance: dist as u16,
                    margin: margin.clamp(i16::MIN as i64, i16::MAX as i64) as i16,
                });
            }
        }
        active.sort_by_key(|a| a.region);

        // Predicted cloud F: union of E_f targets of the active regions,
        // keeping the single best incoming edge per predicted region
        // (highest score_q, ties to the lowest canonical edge id).
        let active_nodes: BTreeSet<u32> = active.iter().map(|a| a.region + 1).collect();
        let mut best_edge: BTreeMap<u32, EdgeUse> = BTreeMap::new();
        for &node in &active_nodes {
            if let Some(edges) = self.forward_by_src.get(&node) {
                for edge in edges {
                    k.table_reads += 1;
                    let better = match best_edge.get(&edge.dst) {
                        None => true,
                        Some(current) => {
                            k.compares += 1;
                            (edge.score_q, std::cmp::Reverse(edge.edge_id))
                                > (current.score_q, std::cmp::Reverse(current.edge_id))
                        }
                    };
                    if better {
                        best_edge.insert(edge.dst, *edge);
                    }
                }
            }
        }
        let predicted: Vec<u32> = best_edge.keys().copied().collect();
        let edges_applied: Vec<EdgeUse> = best_edge.values().copied().collect();
        let mut transition_offset = ScoreQ::ZERO;
        for edge in &edges_applied {
            transition_offset = transition_offset.saturating_add(edge.score_q);
            k.adds += 1;
        }

        // Candidate accumulation: every node of A ∪ F applies its
        // emission list exactly once (Theorem 10 by construction).
        let mut candidates: BTreeMap<u32, (ScoreQ, Vec<Contribution>)> = BTreeMap::new();
        let mut contributing: BTreeSet<u32> = active_nodes.clone();
        for &node in &predicted {
            contributing.insert(node);
        }
        for &node in &contributing {
            let emissions = &self.emissions[(node - 1) as usize];
            for (&token, &value) in emissions {
                k.candidate_scans += 1;
                k.table_reads += 1;
                let entry = candidates
                    .entry(token)
                    .or_insert_with(|| (ScoreQ::ZERO, Vec::new()));
                if entry.1.is_empty() {
                    entry.1.push(Contribution {
                        id: ContributionId::RootPrior { token },
                        value: self.root_score(token),
                    });
                }
                entry.0 = entry.0.saturating_add(value);
                k.adds += 1;
                entry.1.push(Contribution {
                    id: ContributionId::Emission { node, token },
                    value,
                });
            }
        }
        // Root prior top-B tokens join the candidate set.
        for &token in &self.root_top {
            k.table_reads += 1;
            let entry = candidates
                .entry(token)
                .or_insert_with(|| (ScoreQ::ZERO, Vec::new()));
            if entry.1.is_empty() {
                entry.1.push(Contribution {
                    id: ContributionId::RootPrior { token },
                    value: self.root_score(token),
                });
            }
        }

        // EXCT: the existing prefix probe (deepest populated graded
        // prefix) over the exact-context store; ΔX(X,v) = quantized
        // probe log-prob minus the stored root prior (integer sub).
        let mut exct_probe: Option<ExctProbe> = None;
        if let (Some(store), Some(art)) = (&self.store, &self.artifacts) {
            let code = runtime::assign_plain(art, sig);
            for level in (0..=STAGES).rev() {
                if let Some(dist) = store[level].get(&code[..level]) {
                    let total: u32 = dist.values().sum();
                    let mut ranked: Vec<(u32, u32)> = dist.iter().map(|(&t, &c)| (t, c)).collect();
                    ranked.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
                    let mut admitted = 0u32;
                    for &(token, count) in ranked.iter().take(self.exct_top_x) {
                        k.table_reads += 1;
                        let value = quantize_exct(count, total, self.vocab)
                            .saturating_sub(self.root_score(token));
                        k.adds += 1;
                        let entry = candidates
                            .entry(token)
                            .or_insert_with(|| (ScoreQ::ZERO, Vec::new()));
                        if entry.1.is_empty() {
                            entry.1.push(Contribution {
                                id: ContributionId::RootPrior { token },
                                value: self.root_score(token),
                            });
                        }
                        entry.0 = entry.0.saturating_add(value);
                        k.adds += 1;
                        entry.1.push(Contribution {
                            id: ContributionId::ExactContext { token },
                            value,
                        });
                        admitted += 1;
                    }
                    exct_probe = Some(ExctProbe {
                        level: level as u8,
                        key: code[..level].to_vec(),
                        total,
                        admitted,
                    });
                    break;
                }
            }
        }

        // Final per-candidate scores: apply the baked root prior and the
        // folded token-independent transition edge weight (module docs:
        // ΔT(m,v) = w_m + ΔE(m,v)); canonicalize contribution order.
        let mut ranked_candidates: Vec<(u32, ScoreQ, Vec<Contribution>)> =
            Vec::with_capacity(candidates.len());
        for (token, (residual, mut contributions)) in candidates {
            let with_offset = residual.saturating_add(transition_offset);
            k.adds += 1;
            let base = self.root_score(token);
            let score = base.saturating_add(with_offset);
            k.adds += 1;
            contributions.sort_by_key(|c| c.id);
            ranked_candidates.push((token, score, contributions));
        }
        if ranked_candidates.is_empty() {
            return Err("scoring produced an empty candidate set".to_owned());
        }

        // Canonical argmax: highest score, then the lowest token id
        // (ascending-token iteration with a strict `>` keep).
        let mut best_index = 0usize;
        for (index, &(_, score, _)) in ranked_candidates.iter().enumerate() {
            k.candidate_scans += 1;
            k.compares += 1;
            if score > ranked_candidates[best_index].1 {
                best_index = index;
            }
        }
        let (selected, selected_score, _) = ranked_candidates[best_index];
        let selected_contributions = ranked_candidates[best_index].2.clone();
        let candidate_count = ranked_candidates.len() as u32;
        let candidates_out: Vec<(u32, ScoreQ)> = ranked_candidates
            .into_iter()
            .map(|(token, score, _)| (token, score))
            .collect();

        let witness = ScoreWitness {
            graph_cid: self.graph_cid,
            input_sig: *sig,
            active,
            predicted,
            edges_applied,
            transition_offset,
            exct: exct_probe,
            selected,
            selected_score,
            selected_contributions,
            candidate_count,
            census: k,
        };
        Ok(ScoreOutcome {
            selected,
            selected_score,
            candidates: candidates_out,
            witness,
        })
    }
}

/// Rejection reasons of the independent witness-replay verifier
/// (Theorems 6 and 10). Each variant names exactly one inconsistency
/// between the witness's claims and the recomputation from the
/// validated artifact bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReplayError {
    /// The artifact bytes failed two-stage validation or CID integrity.
    Artifact(String),
    /// The witness's graph CID does not match the artifact's
    /// `artifact_cid`.
    GraphCidMismatch,
    /// EXCT evidence was claimed but no teacher container was provided.
    TeacherMissing,
    /// The teacher container does not match HEAD `teacher_cid`.
    TeacherCidMismatch,
    /// The recomputed active cloud (regions, distances, margins)
    /// differs.
    ActiveMismatch,
    /// The recomputed predicted cloud differs.
    PredictedMismatch,
    /// The recomputed applied-edge set or folded offset differs.
    EdgesMismatch,
    /// The recomputed EXCT probe differs.
    ExctMismatch,
    /// Theorem 10: a contribution id appears twice in the witness.
    DuplicateContributionId,
    /// Theorem 10: an applied edge id appears twice in the witness.
    DuplicateEdgeId,
    /// The selected token's contribution list differs from the
    /// recomputation (ids, order, or values).
    ContributionsMismatch,
    /// The selected token or its score differs from the recomputation.
    SelectedMismatch,
    /// The recorded candidate count differs from the recomputation.
    CandidateCountMismatch,
    /// The recorded op census differs from the recomputation.
    CensusMismatch,
}

impl std::fmt::Display for ReplayError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReplayError::Artifact(e) => write!(f, "artifact validation failed: {e}"),
            ReplayError::GraphCidMismatch => write!(f, "graph CID mismatch"),
            ReplayError::TeacherMissing => write!(f, "EXCT claimed but no teacher container"),
            ReplayError::TeacherCidMismatch => write!(f, "teacher CID mismatch"),
            ReplayError::ActiveMismatch => write!(f, "active cloud mismatch"),
            ReplayError::PredictedMismatch => write!(f, "predicted cloud mismatch"),
            ReplayError::EdgesMismatch => write!(f, "applied edges mismatch"),
            ReplayError::ExctMismatch => write!(f, "EXCT probe mismatch"),
            ReplayError::DuplicateContributionId => {
                write!(f, "duplicate contribution id (Theorem 10)")
            }
            ReplayError::DuplicateEdgeId => write!(f, "duplicate applied edge id (Theorem 10)"),
            ReplayError::ContributionsMismatch => write!(f, "contribution list mismatch"),
            ReplayError::SelectedMismatch => write!(f, "selected token or score mismatch"),
            ReplayError::CandidateCountMismatch => write!(f, "candidate count mismatch"),
            ReplayError::CensusMismatch => write!(f, "op census mismatch"),
        }
    }
}

impl std::error::Error for ReplayError {}

/// Independent witness replay (Theorem 6): rebuild a scorer from the
/// validated R4G1 bytes — with the teacher container when the witness
/// claims EXCT evidence — recompute the prediction of
/// `witness.input_sig` without any compiler state, and require bit-exact
/// equality of every witness field. Duplicate contribution ids are
/// rejected before any comparison (Theorem 10).
pub fn verify_witness_replay(
    r4g1: &[u8],
    teacher_container: Option<&[u8]>,
    witness: &ScoreWitness,
    root_top_b: usize,
    exct_top_x: usize,
) -> Result<(), ReplayError> {
    // Theorem 10: reject duplicate contribution ids out of hand.
    let mut seen: BTreeSet<ContributionId> = BTreeSet::new();
    for contribution in &witness.selected_contributions {
        if !seen.insert(contribution.id) {
            return Err(ReplayError::DuplicateContributionId);
        }
    }
    let mut seen_edges: BTreeSet<u32> = BTreeSet::new();
    for edge in &witness.edges_applied {
        if !seen_edges.insert(edge.edge_id) {
            return Err(ReplayError::DuplicateEdgeId);
        }
    }

    if witness.exct.is_some() && teacher_container.is_none() {
        return Err(ReplayError::TeacherMissing);
    }
    let scorer = GraphScorer::from_artifact(r4g1, teacher_container, root_top_b, exct_top_x)
        .map_err(|e| {
            if e.contains("teacher_cid") {
                ReplayError::TeacherCidMismatch
            } else {
                ReplayError::Artifact(e)
            }
        })?;
    if scorer.graph_cid != witness.graph_cid {
        return Err(ReplayError::GraphCidMismatch);
    }
    let outcome = scorer
        .score_candidates(&witness.input_sig)
        .map_err(ReplayError::Artifact)?;
    let recomputed = &outcome.witness;

    if recomputed.active != witness.active {
        return Err(ReplayError::ActiveMismatch);
    }
    if recomputed.predicted != witness.predicted {
        return Err(ReplayError::PredictedMismatch);
    }
    if recomputed.edges_applied != witness.edges_applied
        || recomputed.transition_offset != witness.transition_offset
    {
        return Err(ReplayError::EdgesMismatch);
    }
    if recomputed.exct != witness.exct {
        return Err(ReplayError::ExctMismatch);
    }
    if recomputed.selected_contributions != witness.selected_contributions {
        return Err(ReplayError::ContributionsMismatch);
    }
    if recomputed.selected != witness.selected
        || recomputed.selected_score != witness.selected_score
    {
        return Err(ReplayError::SelectedMismatch);
    }
    if recomputed.candidate_count != witness.candidate_count {
        return Err(ReplayError::CandidateCountMismatch);
    }
    if recomputed.census != witness.census {
        return Err(ReplayError::CensusMismatch);
    }
    Ok(())
}
