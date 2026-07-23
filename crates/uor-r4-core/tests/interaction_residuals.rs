//! Issue #69 — interaction residuals evaluation (release-only fixture
//! workload): decide whether any high-co-activation region pairs justify
//! explicit overlap nodes with interaction residuals, judged on held-out
//! gain vs bytes. Rejection with numbers attached is a valid outcome.
//!
//! # Design (as specified in the issue)
//!
//! 1. **Candidate pairs**: the top-K co-activated same-depth region pairs
//!    from the induced cover's E_o builder — the same top-M co-activation
//!    counting as [`cover::build_edges`], floored at the existing E_o
//!    threshold [`cover::coactivation_min`], ranked by count descending
//!    then `(a, b)` ascending; K = min(16, available).
//! 2. **Interaction residual** per pair (a, b):
//!    `ΔI(pair, v) = ln P(v|a∧b) − ln P(v|a) − ln P(v|b) + ln P(v|parent(a,b))`
//!    with the emit path's add-one smoothing over the compiled vocabulary.
//!    - P(v|a∧b): the store's top-3 teacher-weighted evidence over the
//!      train observations whose top-M memberships at the pair's depth
//!      include BOTH a and b (the shared construction corpus).
//!    - P(v|a), P(v|b): the exact distributions the deployed ΔE tables
//!      are compiled from (covered binary top-1 members, teacher-weighted
//!      — `score::compile_emissions` evidence), so ΔI corrects the sum the
//!      deployed scorer actually composes.
//!    - P(v|parent(a,b)): the deepest common ancestor in the refinement
//!      tree (region id); the level-0 store root distribution when the
//!      pair shares no region ancestor (all depth-1 and cross-subtree
//!      pairs).
//!
//!    The shipped form is bounded: the top-E (64) tokens of the pair's
//!    joint-observed types by ΔI descending, token ascending — the
//!    `compile_emissions` top-E selection rule — stored ascending token.
//! 3. **Measurement**: a scorer variant adds ΔI exactly once when both a
//!    and b are in the active cloud (canonical contribution id
//!    `Interaction{a, b, token}`, Theorem 10). Under Rule 1 the overlap
//!    node's list also joins the candidate set (unlisted tokens enter at
//!    the root prior plus ΔI — the overlap node is not on the refinement
//!    chain); under Rule 2 (EXCT precedence) the exact-context candidate
//!    set is kept and listed candidates are rescored only (graph
//!    candidate generation stays skipped). Deployed scorer configuration:
//!    predicted-cloud (ΔT) emissions OFF (the #66 ablation default).
//!    Compared on the fixture corpus's held-out Gate C set: Rule 1
//!    (chain-telescoped, no EXCT) top-1 agreement AND bits/token with and
//!    without each interaction; Rule 1+2 (with EXCT) bits/token and
//!    agreement with and without. Deterministic double-run.
//! 4. **Decision rule**: an overlap node justifies its bytes only if it
//!    improves held-out bits/token by ≥ 0.05 (declared threshold) on
//!    either rule without losing agreement.
//!
//! # Byte cost model (per shipped overlap node)
//!
//! NODE record (30 B) + two E_o membership edges (2 × 20 B) + their
//! reverse-index entries (2 × 4 B) + the bounded EMIT list (8 B ×
//! entries). The overlap node needs no routing prototype: membership is
//! derived from the pair's activation, not Hamming routing, so ROUT is
//! unchanged.
//!
//! Run with
//! `cargo test -p uor-r4-core --release --offline --test interaction_residuals -- --ignored --nocapture`.

use std::collections::{BTreeMap, BTreeSet};

use uor_r4_core::transformerless::compiler::{self, Corpus, STAGES};
use uor_r4_core::transformerless::cover::{self, CoverConfig, Observation};
use uor_r4_core::transformerless::runtime;
use uor_r4_core::transformerless::score::{self, ScoreConfig};
use uor_r4_core::transformerless::score_runtime::{self, GraphScorer, RegionParams};
use uor_r4_graph_format::ScoreQ;

/// Top-K co-activated pairs evaluated (issue #69 design step 1).
const TOP_K_PAIRS: usize = 16;
/// Bounded interaction residual list per overlap node (mirrors
/// `ScoreConfig::emission_entries`).
const INTERACTION_LIST_MAX: usize = 64;
/// Declared decision threshold: minimum held-out bits/token improvement.
const BITS_GAIN_THRESHOLD: f64 = 0.05;
/// Byte accounting of one shipped overlap node (module docs).
const NODE_RECORD_BYTES: usize = 30;
const EDGE_RECORD_BYTES: usize = 20;
const REVERSE_ENTRY_BYTES: usize = 4;
const EMIT_ENTRY_BYTES: usize = 8;

/// ln of an add-one-smoothed probability — the emit path's
/// `score::Smoothing::AddOne` arm (compiler-side f64 → f32; macOS-pinned
/// like every compiler ln quantization).
fn add_one_ln(count: u64, total: u64, vocab: u32) -> f32 {
    ((count as f64 + 1.0) / (total as f64 + f64::from(vocab))).ln() as f32
}

/// Per train observation: the sorted top-M membership region ids per
/// depth — one shared membership computation for the co-activation
/// counting and the joint evidence (the `cover::build_edges` semantics:
/// `ReferenceClassifier::binary_memberships`, sorted and deduplicated).
fn train_memberships(
    reference: &cover::ReferenceClassifier,
    train: &[Observation],
    max_depth: usize,
) -> Vec<Vec<Vec<u32>>> {
    train
        .iter()
        .map(|observation| {
            (1..=max_depth)
                .map(|depth| {
                    let mut memberships = reference.binary_memberships(depth, &observation.sig);
                    memberships.sort_unstable();
                    memberships.dedup();
                    memberships
                })
                .collect()
        })
        .collect()
}

/// Co-activation counts per same-depth region pair — the
/// `cover::build_edges` E_o counting kept as an inspectable table
/// (issue #69 design step 1).
fn coactivation_table(memberships: &[Vec<Vec<u32>>]) -> BTreeMap<(u32, u32), u64> {
    let mut coactive: BTreeMap<(u32, u32), u64> = BTreeMap::new();
    for per_depth in memberships {
        for at_depth in per_depth {
            for (a_idx, &a) in at_depth.iter().enumerate() {
                for &b in &at_depth[a_idx + 1..] {
                    *coactive.entry((a, b)).or_insert(0) += 1;
                }
            }
        }
    }
    coactive
}

/// Per-region teacher-weighted next-token evidence under the emit path's
/// covered binary top-1 membership — the exact distributions the
/// deployed ΔE tables are compiled from (`score::compile_emissions`).
fn region_evidence(
    corpus: &Corpus,
    regions: &[RegionParams],
    train: &[Observation],
    max_depth: usize,
) -> Vec<BTreeMap<u32, u64>> {
    let pop = runtime::derive_popcount_table();
    let mut k = runtime::OpKernel::default();
    let mut evidence: Vec<BTreeMap<u32, u64>> = vec![BTreeMap::new(); regions.len()];
    for observation in train {
        let i = observation.position as usize;
        for depth in 1..=max_depth {
            let Some((top1, _)) =
                score_runtime::binary_top1_covered(&mut k, &pop, regions, depth, &observation.sig)
            else {
                continue;
            };
            let dist = &mut evidence[top1 as usize];
            for k_idx in 0..corpus.top_tokens[i].len() {
                let token = corpus.top_tokens[i][k_idx];
                let weight = corpus.top_weights[i][k_idx];
                if weight > 0 {
                    *dist.entry(token).or_insert(0) += u64::from(weight);
                }
            }
        }
    }
    evidence
}

/// Teacher-weighted next-token evidence of the pair's joint activation:
/// train observations whose sorted top-M memberships at the pair's depth
/// include both regions (the shared construction corpus).
fn joint_evidence(
    corpus: &Corpus,
    train: &[Observation],
    memberships: &[Vec<Vec<u32>>],
    depth: usize,
    a: u32,
    b: u32,
) -> (u64, BTreeMap<u32, u64>) {
    let mut observations = 0u64;
    let mut dist: BTreeMap<u32, u64> = BTreeMap::new();
    for (obs_idx, observation) in train.iter().enumerate() {
        let at_depth = &memberships[obs_idx][depth - 1];
        let joint = at_depth.binary_search(&a).is_ok() && at_depth.binary_search(&b).is_ok();
        if !joint {
            continue;
        }
        observations += 1;
        let i = observation.position as usize;
        for k_idx in 0..corpus.top_tokens[i].len() {
            let token = corpus.top_tokens[i][k_idx];
            let weight = corpus.top_weights[i][k_idx];
            if weight > 0 {
                *dist.entry(token).or_insert(0) += u64::from(weight);
            }
        }
    }
    (observations, dist)
}

/// Deepest common ancestor of two regions in the refinement tree
/// (region id), or `None` when the pair shares no region ancestor (the
/// parent is then the root distribution). Walking b's ancestor chain
/// upward, the first node also in a's ancestor set is the deepest common
/// node (common ancestors form a root-side chain prefix).
fn deepest_common_ancestor(regions: &[RegionParams], a: u32, b: u32) -> Option<u32> {
    let cap = regions.len();
    let mut ancestors: BTreeSet<u32> = BTreeSet::new();
    let mut current = Some(a);
    let mut steps = 0usize;
    while let Some(id) = current {
        if !ancestors.insert(id) || steps > cap {
            break;
        }
        current = regions.get(id as usize).and_then(|r| r.parent);
        steps += 1;
    }
    let mut current = Some(b);
    let mut steps = 0usize;
    while let Some(id) = current {
        if ancestors.contains(&id) {
            return Some(id);
        }
        if steps > cap {
            break;
        }
        current = regions.get(id as usize).and_then(|r| r.parent);
        steps += 1;
    }
    None
}

/// One candidate pair's compiled interaction residual table plus the
/// evaluation metadata the report needs.
#[derive(Debug, Clone)]
struct InteractionResidual {
    a: u32,
    b: u32,
    depth: usize,
    /// Train co-activation count (the E_o ranking signal; identical to
    /// the joint observation count by construction — asserted).
    coactivation: u64,
    /// Train observations whose top-M memberships include both regions.
    joint_observations: u64,
    /// Teacher-weighted evidence total of the joint distribution.
    joint_total: u64,
    /// Observed token types of the joint distribution.
    joint_types: usize,
    /// Deepest common ancestor region id, or `None` for the root.
    parent: Option<u32>,
    /// Bounded top-E ΔI list (selection mirrors `compile_emissions`:
    /// ΔI desc, token asc; storage ascending token).
    entries: Vec<(u32, ScoreQ)>,
    /// ΔI norms over the joint-observed types (nats, f32 domain).
    max_nats: f32,
    min_nats: f32,
    mean_abs_nats: f64,
    /// Token with the largest ΔI (report color).
    max_token: u32,
    /// Shipped byte cost of the overlap node (module docs).
    overlap_bytes: usize,
}

/// Compile one candidate pair's interaction residual table (issue #69
/// design step 2; the evidence model documented in the module header).
#[allow(clippy::too_many_arguments)]
fn compile_interaction(
    regions: &[RegionParams],
    evidence: &[BTreeMap<u32, u64>],
    root_dist: &BTreeMap<u32, u64>,
    joint_dist: &BTreeMap<u32, u64>,
    joint_observations: u64,
    depth: usize,
    a: u32,
    b: u32,
    coactivation: u64,
    vocab: u32,
) -> InteractionResidual {
    let parent = deepest_common_ancestor(regions, a, b);
    let (parent_dist, parent_total): (&BTreeMap<u32, u64>, u64) = match parent {
        Some(p) => {
            let dist = &evidence[p as usize];
            (dist, dist.values().sum())
        }
        None => (root_dist, root_dist.values().sum()),
    };
    let dist_a = &evidence[a as usize];
    let total_a: u64 = dist_a.values().sum();
    let dist_b = &evidence[b as usize];
    let total_b: u64 = dist_b.values().sum();
    let joint_total: u64 = joint_dist.values().sum();

    // ΔI(pair, v) = ln P(v|a∧b) − ln P(v|a) − ln P(v|b) + ln P(v|parent)
    // over the joint-observed types, f32 like the emit path's residuals.
    let mut residuals: Vec<(u32, f32)> = Vec::with_capacity(joint_dist.len());
    for (&token, &count_ab) in joint_dist {
        let lp_ab = add_one_ln(count_ab, joint_total, vocab);
        let lp_a = add_one_ln(dist_a.get(&token).copied().unwrap_or(0), total_a, vocab);
        let lp_b = add_one_ln(dist_b.get(&token).copied().unwrap_or(0), total_b, vocab);
        let lp_p = add_one_ln(
            parent_dist.get(&token).copied().unwrap_or(0),
            parent_total,
            vocab,
        );
        residuals.push((token, lp_ab - lp_a - lp_b + lp_p));
    }
    let max_nats = residuals
        .iter()
        .map(|&(_, d)| d)
        .fold(f32::NEG_INFINITY, f32::max);
    let min_nats = residuals
        .iter()
        .map(|&(_, d)| d)
        .fold(f32::INFINITY, f32::min);
    let mean_abs_nats = if residuals.is_empty() {
        0.0
    } else {
        residuals
            .iter()
            .map(|&(_, d)| f64::from(d.abs()))
            .sum::<f64>()
            / residuals.len() as f64
    };
    let max_token = residuals
        .iter()
        .max_by(|x, y| x.1.total_cmp(&y.1))
        .map(|&(t, _)| t)
        .unwrap_or(0);

    // Top-E selection mirrors compile_emissions (ΔI desc, token asc;
    // ascending-token storage), quantized like the emit path.
    let mut entries: Vec<(u32, ScoreQ)> = residuals
        .iter()
        .map(|&(token, delta)| (token, ScoreQ::from_logprob(delta)))
        .collect();
    entries.sort_by(|x, y| y.1.cmp(&x.1).then_with(|| x.0.cmp(&y.0)));
    entries.truncate(INTERACTION_LIST_MAX);
    entries.sort_by_key(|&(token, _)| token);
    let overlap_bytes = NODE_RECORD_BYTES
        + 2 * (EDGE_RECORD_BYTES + REVERSE_ENTRY_BYTES)
        + EMIT_ENTRY_BYTES * entries.len();

    InteractionResidual {
        a,
        b,
        depth,
        coactivation,
        joint_observations,
        joint_total,
        joint_types: joint_dist.len(),
        parent,
        entries,
        max_nats,
        min_nats,
        mean_abs_nats,
        max_token,
        overlap_bytes,
    }
}

/// Canonical argmax over an ascending-token candidate list: highest
/// score, ties to the lowest token (strict `>` keeps the first).
fn canonical_argmax(candidates: &[(u32, ScoreQ)]) -> u32 {
    let mut best = candidates[0];
    for &candidate in &candidates[1..] {
        if candidate.1 > best.1 {
            best = candidate;
        }
    }
    best.0
}

/// Certifier-side bits/token of one scored candidate set — the
/// `score::evaluate_gate_c` outcome_bits semantics (ScoreQ candidates as
/// natural-log weights, max-shifted; non-candidate tokens held at the
/// baked root smoothing floor).
fn outcome_bits(root_floor: ScoreQ, vocab: u32, candidates: &[(u32, ScoreQ)], next: u32) -> f64 {
    let floor = root_floor.raw();
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
    let uncovered = (vocab as usize).saturating_sub(candidates.len());
    sum += uncovered as f64 * w_floor;
    let w = w_next.unwrap_or(w_floor).max(1e-300);
    (sum / w).ln() / std::f64::consts::LN_2
}

/// Rule 1 variant: the overlap node's ΔI list joins the candidate set
/// (unlisted tokens enter at the root prior plus ΔI — the overlap node
/// is not on the refinement chain) and each listed candidate receives ΔI
/// exactly once (canonical id `Interaction{a, b, token}`, Theorem 10).
fn apply_interaction_rule1(
    base: &[(u32, ScoreQ)],
    entries: &[(u32, ScoreQ)],
    root_prior: &BTreeMap<u32, ScoreQ>,
    root_floor: ScoreQ,
) -> Vec<(u32, ScoreQ)> {
    let mut map: BTreeMap<u32, ScoreQ> = base.iter().copied().collect();
    for &(token, delta) in entries {
        let entry = map
            .entry(token)
            .or_insert_with(|| root_prior.get(&token).copied().unwrap_or(root_floor));
        *entry = entry.saturating_add(delta);
    }
    map.into_iter().collect()
}

/// Rule 1+2 variant: EXCT precedence keeps its exact-context candidate
/// set; listed candidates receive ΔI exactly once, and no new candidates
/// are generated (graph candidate generation stays skipped under Rule 2).
fn apply_interaction_rule12(
    base: &[(u32, ScoreQ)],
    entries: &[(u32, ScoreQ)],
) -> Vec<(u32, ScoreQ)> {
    let mut map: BTreeMap<u32, ScoreQ> = base.iter().copied().collect();
    for &(token, delta) in entries {
        if let Some(entry) = map.get_mut(&token) {
            *entry = entry.saturating_add(delta);
        }
    }
    map.into_iter().collect()
}

/// Baseline (interaction-free) aggregates over the held-out set.
#[derive(Debug, Clone, Default, PartialEq)]
struct BaselineAcc {
    r1_hits: u64,
    r1_bits: f64,
    r12_hits: u64,
    r12_bits: f64,
}

/// Per-pair aggregates, gated positions only (ungated positions are
/// identical to the baseline by construction and cancel in the deltas).
#[derive(Debug, Clone, Default, PartialEq)]
struct PairAcc {
    gated: u64,
    r1_dbits: f64,
    r1_dhits: i64,
    r12_dbits: f64,
    r12_dhits: i64,
}

/// One measurement pass over the held-out set: baseline Rule 1 / Rule
/// 1+2 outcomes once per position, then each pair's variant applied when
/// both regions are in the active cloud (the scorer's membership
/// semantics — `score_runtime::binary_memberships`).
#[allow(clippy::too_many_arguments)]
fn measurement_pass(
    scorer_rule1: &GraphScorer,
    scorer_rule12: &GraphScorer,
    regions: &[RegionParams],
    max_depth: usize,
    corpus: &Corpus,
    held_out: &[Observation],
    interactions: &[InteractionResidual],
    root_prior: &BTreeMap<u32, ScoreQ>,
    root_floor: ScoreQ,
    vocab: u32,
) -> (BaselineAcc, Vec<PairAcc>) {
    let pop = runtime::derive_popcount_table();
    let mut baseline = BaselineAcc::default();
    let mut pairs = vec![PairAcc::default(); interactions.len()];
    for observation in held_out {
        let position = observation.position as usize;
        let teacher_argmax = corpus.t_argmax[position];
        let next = corpus.next[position];
        let rule1 = scorer_rule1
            .score_candidates(&observation.sig, &[])
            .expect("rule 1 scores");
        let rule12 = scorer_rule12
            .score_candidates(&observation.sig, &[])
            .expect("rule 1+2 scores");
        let r1_sel = canonical_argmax(&rule1.candidates);
        let r12_sel = canonical_argmax(&rule12.candidates);
        let r1_bits = outcome_bits(root_floor, vocab, &rule1.candidates, next);
        let r12_bits = outcome_bits(root_floor, vocab, &rule12.candidates, next);
        baseline.r1_hits += u64::from(r1_sel == teacher_argmax);
        baseline.r1_bits += r1_bits;
        baseline.r12_hits += u64::from(r12_sel == teacher_argmax);
        baseline.r12_bits += r12_bits;

        // Active cloud per depth (the deployed scorer's semantics).
        let mut active: Vec<BTreeSet<u32>> = Vec::with_capacity(max_depth);
        for depth in 1..=max_depth {
            let mut k = runtime::OpKernel::default();
            active.push(
                score_runtime::binary_memberships(&mut k, &pop, regions, depth, &observation.sig)
                    .into_iter()
                    .map(|(region, _)| region)
                    .collect(),
            );
        }
        for (idx, interaction) in interactions.iter().enumerate() {
            let members = &active[interaction.depth - 1];
            if !(members.contains(&interaction.a) && members.contains(&interaction.b)) {
                continue;
            }
            let acc = &mut pairs[idx];
            acc.gated += 1;
            let v1 = apply_interaction_rule1(
                &rule1.candidates,
                &interaction.entries,
                root_prior,
                root_floor,
            );
            acc.r1_dbits += outcome_bits(root_floor, vocab, &v1, next) - r1_bits;
            acc.r1_dhits += i64::from(canonical_argmax(&v1) == teacher_argmax)
                - i64::from(r1_sel == teacher_argmax);
            let v12 = apply_interaction_rule12(&rule12.candidates, &interaction.entries);
            acc.r12_dbits += outcome_bits(root_floor, vocab, &v12, next) - r12_bits;
            acc.r12_dhits += i64::from(canonical_argmax(&v12) == teacher_argmax)
                - i64::from(r12_sel == teacher_argmax);
        }
    }
    (baseline, pairs)
}

/// The full issue-#69 evaluation on the pinned fixture corpus.
#[test]
#[ignore = "release-only fixture workload"]
fn fixture_interaction_residuals_evaluation() {
    // ---- Fixture pipeline (mirrors tests/score.rs fixture_corpus_end_to_end).
    let dir = env!("CARGO_MANIFEST_DIR");
    let artifact_container =
        std::fs::read(format!("{dir}/tests/fixtures/tless_artifacts.bin")).expect("fixture TLA5");
    let artifacts = compiler::parse_artifacts(&artifact_container).expect("fixture parses");
    let meta_bytes = std::fs::read(format!("{dir}/tests/fixtures/c_meta.bin")).expect("meta");
    let recs_bytes = std::fs::read(format!("{dir}/tests/fixtures/c_recs.bin")).expect("recs");
    let corpus = compiler::load_corpus_from(
        &format!("{dir}/tests/fixtures/c_meta.bin"),
        &format!("{dir}/tests/fixtures/c_recs.bin"),
    )
    .expect("fixture corpus loads");
    let artifact_kappa = format!("blake3:{}", blake3::hash(&artifact_container).to_hex());
    let corpus_kappa = {
        let mut h = blake3::Hasher::new();
        h.update(&meta_bytes);
        h.update(&recs_bytes);
        format!("blake3:{}", h.finalize().to_hex())
    };

    let config = ScoreConfig::default();
    let (train_pos, held_out_pos) = cover::split_positions(&corpus);
    let train = cover::build_observations(&artifacts, &corpus, &train_pos);
    let held_out = cover::build_observations(&artifacts, &corpus, &held_out_pos);
    let induced = cover::induce_cover(
        &train,
        &CoverConfig::default(),
        &artifact_kappa,
        &corpus_kappa,
    )
    .expect("fixture induction");
    let reference = cover::ReferenceClassifier::freeze(&induced.cover);
    let cover_edges = cover::build_edges(&induced.cover, &reference, &train);
    let regions = score::regions_from_cover(&induced.cover);
    let structural = score::structural_from_cover(&cover_edges);
    let max_depth = regions.iter().map(|r| r.depth as usize).max().unwrap_or(1);
    let (store, _) = runtime::build_store(&artifacts, &corpus);
    let tls1 = runtime::store_bytes(&store);
    let transitions = score::compile_transitions(
        &corpus,
        &regions,
        &train,
        max_depth,
        config.transition_out_degree,
    );
    let vocab = (artifacts.token_codes.len() / STAGES) as u32;
    let emissions =
        score::compile_emissions(&corpus, &store, &regions, &train, max_depth, vocab, &config);
    let (bytes, info) = score::emit_scored_r4g1(
        &artifact_container,
        (&meta_bytes, &recs_bytes),
        vocab,
        &score::ScoredGraphSections {
            regions: &regions,
            structural: &structural,
            transitions: &transitions,
            emissions: &emissions,
            exct_tls1: &tls1,
            exct_top_x: config.exct_top_x,
        },
    )
    .expect("fixture emit");
    // Pipeline anchor: the build reproduces the pinned fixture artifact.
    assert_eq!(
        format!("blake3:{}", blake3::hash(&bytes).to_hex()),
        "blake3:de04eec8be0ce001c1493acee1b28f83976a74f85519855e1f23e8676d713704",
        "harness pipeline reproduces the canonical fixture scored artifact"
    );

    // ---- Step 1: candidate pairs (E_o co-activation counting).
    let memberships = train_memberships(&reference, &train, max_depth);
    let coactive = coactivation_table(&memberships);
    let floor = cover::coactivation_min(train.len());
    let mut pool: Vec<((u32, u32), u64)> = coactive
        .iter()
        .filter(|&(_, &count)| count >= floor)
        .map(|(&pair, &count)| (pair, count))
        .collect();
    pool.sort_by(|x, y| y.1.cmp(&x.1).then_with(|| x.0.cmp(&y.0)));
    let available = pool.len();
    pool.truncate(TOP_K_PAIRS);

    // Sanity: every wired E_o neighbor edge passes the floor in the table.
    for edge in &cover_edges {
        if edge.kind == cover::EDGE_KIND_NEIGHBOR {
            let pair = (edge.src - 1, edge.dst - 1);
            assert!(
                coactive.get(&pair).copied().unwrap_or(0) >= floor,
                "wired E_o edge {pair:?} below the co-activation floor"
            );
        }
    }

    // ---- Step 2: interaction residual tables per candidate pair.
    let evidence = region_evidence(&corpus, &regions, &train, max_depth);
    let root_dist: BTreeMap<u32, u64> = store
        .first()
        .and_then(|level| level.get(&[][..]))
        .map(|dist| dist.iter().map(|(&t, &c)| (t, u64::from(c))).collect())
        .unwrap_or_default();
    let mut interactions = Vec::with_capacity(pool.len());
    for &((a, b), count) in &pool {
        let depth = regions[a as usize].depth as usize;
        assert_eq!(
            depth, regions[b as usize].depth as usize,
            "E_o pairs are same-depth by construction"
        );
        let (joint_observations, joint_dist) =
            joint_evidence(&corpus, &train, &memberships, depth, a, b);
        assert_eq!(
            joint_observations, count,
            "joint support equals the co-activation count by construction"
        );
        interactions.push(compile_interaction(
            &regions,
            &evidence,
            &root_dist,
            &joint_dist,
            joint_observations,
            depth,
            a,
            b,
            count,
            vocab,
        ));
    }

    // ---- Step 3: the measurement (deployed scorer configuration:
    // predicted-cloud ΔT emissions off, the #66 ablation default).
    let scorer_rule1 =
        GraphScorer::from_artifact(&bytes, None, config.root_top_b, config.exct_top_x)
            .expect("rule 1 scorer");
    let scorer_rule12 = GraphScorer::from_artifact(
        &bytes,
        Some(&artifact_container),
        config.root_top_b,
        config.exct_top_x,
    )
    .expect("rule 1+2 scorer");
    let root_floor = scorer_rule1.root_floor();
    let measure = |interactions: &[InteractionResidual]| {
        measurement_pass(
            &scorer_rule1,
            &scorer_rule12,
            &regions,
            max_depth,
            &corpus,
            &held_out,
            interactions,
            &emissions.root_prior,
            root_floor,
            vocab,
        )
    };
    let (baseline, pair_acc) = measure(&interactions);
    // Deterministic double-run: bit-identical accumulators.
    let (baseline2, pair_acc2) = measure(&interactions);
    assert_eq!(baseline, baseline2, "double-run baseline must be identical");
    assert_eq!(
        pair_acc, pair_acc2,
        "double-run pair deltas must be identical"
    );

    // ---- Report.
    let n = held_out.len() as f64;
    let r1_agree = baseline.r1_hits as f64 / n;
    let r1_bits = baseline.r1_bits / n;
    let r12_agree = baseline.r12_hits as f64 / n;
    let r12_bits = baseline.r12_bits / n;
    println!(
        "issue #69 interaction residuals evaluation — fixture corpus\n\
         train {} held-out {} | regions {} (max depth {}) | vocab {} | scored artifact {} B\n\
         E_o floor (coactivation_min) = {} | floor-passing pairs {} | evaluated K = {}\n\
         baseline (deployed config, ΔT off):\n\
         \x20 Rule 1  (no EXCT):  agree {:.4}%  bits/token {:.6}\n\
         \x20 Rule 1+2 (EXCT):    agree {:.4}%  bits/token {:.6}\n\
         anchor (`evaluate_gate_c`, F-on columns): Rule 1+2 31.71% / 9.8612 bits, \
         TLA3 baseline 31.71% / 11.8781 bits, ExactContext 30036/30036",
        train.len(),
        held_out.len(),
        regions.len(),
        max_depth,
        vocab,
        info.artifact_bytes,
        floor,
        available,
        interactions.len(),
        100.0 * r1_agree,
        r1_bits,
        100.0 * r12_agree,
        r12_bits,
    );
    println!(
        "\n{:<14} {:>5} {:>7} {:>7} {:>6} {:>6} {:>5} {:>6} | {:>9} | {:>10} {:>9} | {:>10} {:>9} | decision",
        "pair(r,n)", "depth", "coact", "parent", "types", "E", "bytes", "maxΔI", "gated", "r1 Δbits", "r1 Δagr", "r12 Δbits", "r12 Δagr"
    );
    let mut any_cleared = false;
    for (interaction, acc) in interactions.iter().zip(&pair_acc) {
        let r1_gain = -acc.r1_dbits / n;
        let r12_gain = -acc.r12_dbits / n;
        let r1_dagr = acc.r1_dhits as f64 / n;
        let r12_dagr = acc.r12_dhits as f64 / n;
        let parent = interaction
            .parent
            .map(|p| format!("r{p}"))
            .unwrap_or_else(|| "root".to_owned());
        let clears = (r1_gain >= BITS_GAIN_THRESHOLD && acc.r1_dhits >= 0)
            || (r12_gain >= BITS_GAIN_THRESHOLD && acc.r12_dhits >= 0);
        any_cleared |= clears;
        println!(
            "r{:>2}/r{:>2} n{}/{} {:>5} {:>7} {:>7} {:>6} {:>6} {:>5} {:>6.3} | {:>9} | {:>10.6} {:>8.4}% | {:>10.6} {:>8.4}% | {}",
            interaction.a,
            interaction.b,
            interaction.a + 1,
            interaction.b + 1,
            interaction.depth,
            interaction.coactivation,
            parent,
            interaction.joint_types,
            interaction.entries.len(),
            interaction.overlap_bytes,
            interaction.max_nats,
            acc.gated,
            r1_gain,
            100.0 * r1_dagr,
            r12_gain,
            100.0 * r12_dagr,
            if clears { "CLEARS" } else { "no" },
        );
    }
    println!("\nper-pair ΔI detail (nats over the joint-observed types):");
    for interaction in &interactions {
        println!(
            "  r{:>2}/r{:>2}: max {:>8.4} (token {}) min {:>9.4} mean|ΔI| {:>7.4} | joint obs {} weighted total {}",
            interaction.a,
            interaction.b,
            interaction.max_nats,
            interaction.max_token,
            interaction.min_nats,
            interaction.mean_abs_nats,
            interaction.joint_observations,
            interaction.joint_total,
        );
    }
    println!(
        "\nTHRESHOLD: {:.2} bits/token improvement on either rule without losing agreement",
        BITS_GAIN_THRESHOLD
    );
    if any_cleared {
        println!("DECISION: at least one candidate CLEARS the threshold — implementation follows.");
    } else {
        println!(
            "DECISION: no candidate clears the threshold — evaluated, not justified \
             (overlap nodes with interaction residuals rejected on this distribution)."
        );
    }
    // The test's hard gate: the measurement is the deliverable; keep the
    // aggregates in-range so a broken harness cannot print a clean table.
    assert!((0.0..=1.0).contains(&r1_agree));
    assert!((0.0..=1.0).contains(&r12_agree));
    assert!(r1_bits.is_finite() && r1_bits > 0.0);
    assert!(r12_bits.is_finite() && r12_bits > 0.0);
}
