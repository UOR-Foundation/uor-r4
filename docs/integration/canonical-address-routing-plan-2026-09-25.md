# Canonical Address Routing — an energy-first geometric language model (plan)

**Status:** exploratory. Owner-directed investigation, 2026-09-25. This plan lives on
`codex/canonical-address-routing-20260925` and is **not** proposed for promotion to `main`
until the owner promotes it; it does not yet modify the canonical plan (`project-track.md`).

**Base:** `origin/main` = `413a32fc`.

**Assumed machine:** one M1-class laptop (8 cores, 16 GiB), local-only compute, subject to the
standing resource ledger, storage margins and one-cargo-process discipline.

## 1. Thesis

The objective is not benchmark parity. It is **useful local language at a fraction of the
energy per token**, so that capability stops requiring dense datacenter-style serving.

Four sources of waste in a dense transformer:

1. dense activation — every token reads every parameter;
2. quadratic context — every token re-attends over the whole window;
3. uniform depth — every token receives the full stack regardless of difficulty;
4. float arithmetic — per operation, roughly an order of magnitude (or more) above an
   integer add, shift or table read.

CAR-LM attacks all four with one organising idea: **route each token to the least computation
that preserves quality, and decide that route exactly and cheaply — before computing anything
dense.** Geometry is the control plane that makes routing canonical, exact and drift-free. It
is not proposed here as a substitute carrier for dense computation (see §2).

## 2. Evidence constraints (why the design is shaped this way)

- **No geometric or algebraic parameterisation has beaten an information- and
  compute-matched ordinary control on language quality** in this project's history or in the
  surveyed literature. Reported wins are parameter efficiency (hypercomplex), sample
  efficiency under a genuine symmetry (equivariance), or synthetic tasks. The causal
  ingredient in strong recurrent results is gating / delta-rule updates.
- **Capacity is a counting fact.** Recovering K independent V-ary values from a persistent
  state requires at least K·log₂V bits (pigeonhole). A fixed-width geometric fold cannot
  escape that floor; group structure buys exact closure and zero rounding drift, not
  capacity. (Algebraic statements and their scope: the mathematics review of 2026-09-25.)
- **The served path is dense and latency-bound.** Measured on the retained integer session:
  1,672,960 four-bit code inspections per step (99.7% of the store streamed per token; 85.6%
  of codes nonzero; 62.7% of the work in the dense 4096-row vocabulary projection); a
  recorded full-window read step of 3.695 ms; about 6.5× slower than the matched F32 step;
  roughly 0.57 GB/s against M1 memory bandwidth roughly two orders of magnitude higher —
  i.e. ALU/latency-bound, not bandwidth-bound. D5 (per-token parameter sparsity) is unmet.
- **Binding constraints are objective and interface, not the basis.** Long-range retrieval
  has never been trained or tested; the read path is rank-64; the tuned count reference
  (5.12 nats) beats the served readout (6.68 nats) — a measured interface deficit larger
  than any geometric effect observed to date.
- **Rung-1 transport contrast** (quaternion 2.110368 vs matched ordinary 2.085241 NLL, one
  seed, previously exposed development data) is consistent with the literature; geometric
  transport is treated here as a conditional component, not the thesis.

## 3. Principle: spend energy in proportion to surprise

- The state moves on a bounded manifold (R4-lane / S3 / H4 transport, already present in the
  joint learner).
- **Surprise = exact displacement** of the canonical state after the previous prediction —
  a small integer comparison over quantised coordinates.
- **Compute budget = f(surprise):** cache hit → table read only; small displacement → one
  composed operator; large displacement → full routed program plus sparse retrieval.
- Consequence: energy per token tracks bits of surprise, so predictable spans are nearly
  free — the opposite of uniform per-token cost. This distribution is measurable per token
  and is the programme's central empirical claim.

## 4. Architecture (CAR-LM)

| Layer | Mechanism | Runtime cost |
|---|---|---|
| Canonical state | Bounded state canonically quantised to an address: group cell (2I/C120/H4 tables) × small context code (zeta/dyadic) × tape pointer | table reads + integer compares |
| Router | Address → execution program: 1–K typed operators + n memory rows, by exact lookup; one shared small table; trained with straight-through gradients | 1–2 table reads; no dense gating |
| Operators | Typed group actions (transport, compose, gated write, copy pointer, emission bias); compositions precomputed in closure tables | one compose = one L1-resident table read |
| Sparse readout | Product-key shortlist (≤64 candidates) instead of the dense 4096-row projection | target ≥32× fewer row reads |
| Exact memory | Occurrence tape as authority; geometric addressing proposes candidates; compositional binding (prime / Z[phi] arithmetic) with exact cleanup | O(K) reads |
| Event-driven execution | Unchanged address plus cached emission → emit directly; otherwise run the routed program | 1 compare + 1 read on hits |

**Design contract.** Geometry governs state, addressing, composition and serving arithmetic;
learning fills tables (routing table, operator parameters, sparse readout), not dense
matrices. A small dense residual is permitted only if it pays for itself in quality per
joule.

## 5. Training method (learn offline, route online)

1. **Teacher distillation.** A small dense teacher (trained locally, or the pinned offline
   #1017 reference as comparator) provides targets: emission distillation plus routing
   supervision — which operator reduces teacher KL.
2. **Assignment before gradient gating.** Cluster canonical addresses (Voronoi over group
   cells); assign each cluster the operator program that minimises teacher KL
   (EM / product-quantisation style); then joint fine-tune with straight-through gradients.
   Dense-to-sparse assignment rather than fragile from-scratch discrete routing.
3. **Energy-aware objective.**
   `L = KL(teacher‖student) + λ·E[operators] + μ·E[rows read] + ν·E[displacement] +
   load-balance`.
   The model is rewarded for equal quality at lower cost and for state stability (which
   raises cache hit rate).
4. **Curriculum.** Stage A: MQAR-style multi-query recall plus copy-at-distance with hard
   negatives and the order-2 count prior verified at chance. Stage B: natural text at
   matched tokens. Stage C: composition / state tracking. One graph; nonzero gradient
   checks on all learned paths.
5. **Canonicalisation sweep.** Address coarseness trades cache hit rate against fidelity;
   measured as an explicit Pareto curve, never chosen silently.

## 6. Measurement (the metric is the product)

Instruments:

- **Ops counter** in `crates/uor-r4-integer`: `matrix_work` cells read per token, store size,
  nonzero fraction, cache hits, operator compositions, candidate reads.
- **Wall/RAM**: `/usr/bin/time -l`, fixed request sets, interleaved repeats (n≥5) on a quiet
  machine, load average recorded.
- **Energy**: `powermetrics` on matched F32 and integer runs (J/token, tokens/J); reported
  as UNAVAILABLE until physically measured.
- **Instruction audit**: the mirror of
  `docs/evidence/integer-serving-instruction-audit-2026-09-25.json`, extended to the routing
  and operator paths.

Headline numbers: J/token and tokens/J at matched output quality (median and p95),
parameter-touch fraction, cache hit rate, and the energy–surprisal correlation.

## 7. Gates (cheap first; each falsifiable)

| Gate | Question | Pass condition | Kill criterion | Envelope (provisional) |
|---|---|---|---|---|
| G0 | Can a hand-built CAR match a dense baseline on a small task at a small fraction of the work? | ≤10% ops/token at ≥ baseline quality (synthetic + TinyStories); cache hit ≥50% on predictable spans | cache+compose cannot match an n-gram's quality per op | days; prototype fit ≤4 h M1; ≤6 GiB RAM |
| G1 | Does a learned router distilled from a dense teacher hold quality at low ops? | ≥95% of teacher quality at ≤K operators/token and ≤64 candidate reads; beats an equal-quality, equal-bytes dense integer student on J/token | the dense student is already cheaper at matched quality | ≤1 week; single-seed exploratory, ≥3 seeds to promote |
| G2 | Does geometric routing beat equal-budget LSH / product-key addressing on long-range retrieval? | per-distance-bucket top-K recall win with CI, or match at materially lower cost | geometry ties LSH at equal budget → keep the exact tape; routing becomes a component, not the thesis | evaluation-heavy; reuses existing panels |
| G3 | Is the energy win physical and reproducible? | quality-matched J/token ≥5× better than the float baseline; instruction audit clean; repeats on a quiet machine | unmeasured or unreproducible | measurement only |
| G4 | Does the recipe carry capability at laptop scale? | useful conversation/coding at laptop scale with energy still measured | a quality ceiling forces a dense residual that erases the savings | long pole; earned after G1–G3 |

Every gate records its resource projection, stop margin and cumulative ledger charge before
execution.

## 8. Risks

- **Routing learnability** — load collapse, dead operators. Mitigations: assignment
  initialisation, distillation, load-balance term; G1 kills it early.
- **Quality ceiling** — group operators may be too weak, and a growing dense residual erases
  the savings; the quality/ops Pareto is the instrument.
- **Memory-scatter tax** — many small reads can lose to one dense read on real hardware;
  measure wall time and energy, not op counts alone.
- **Governance drift** — this promotes sparse routing from a conditional option to the
  central architecture; every gate carries cost evidence, and the branch is not merged
  without owner promotion.

## 9. Build order

1. **Instrument first** — ops/energy counters in `crates/uor-r4-integer` (cells read per
   token, cache hits) plus the instruction-audit extension. Nothing is measurable until this
   exists.
2. **G0 prototype** — extend the existing joint learner with canonical address quantisation,
   an address→operator-program table and event-driven caching; train on the Stage-A
   curriculum interleaved with TinyStories; compare against itself dense and against
   count/cache.
3. **G1** — distillation plus assignment training; produce the quality/ops/J Pareto.
4. **G2** — the previously unrun D5 contest (geometric router vs LSH / product-key) on the
   D6 long-range panel.
5. **G3/G4** — measured energy, then scale.

## 10. Boundaries

- Serving arithmetic remains D0-b; per-token parameter sparsity (D5) is promoted from a
  deferred invariant to the organising metric of the design.
- No hidden teacher or provider in serving; no transformer backbone. Sparse routing here is
  geometric (exact address) and cost-evidenced at each gate.
- This document is a branch-local plan. Promotion into `project-track.md` / `DECISIONS.md`
  requires owner direction and a protected PR.

## References

Internal: `docs/integration/current-state.md`; `docs/integration/integer-serving-result-2026-09-25.md`;
`docs/integration/joint-recurrent-result-2026-09-25.md`;
`docs/integration/bounded-admission-result-2026-09-25.md`;
`docs/integration/agent-execution-policy.md`;
`docs/evidence/integer-serving-instruction-audit-2026-09-25.json`.

External (primary sources): Jelassi et al., *Repeat After Me* (2402.01032); Merrill, Petty,
Sabharwal, *The Illusion of State in State-Space Models* (2404.08819); Arora et al.,
*Zoology* (2312.04927); Arora et al., *Simple linear attention...* / Based (2402.18668);
Schlag, Irie, Schmidhuber, *Linear Transformers are Secretly Fast Weight Programmers*
(2102.11174); Yang et al., *Parallelizing Linear Transformers with the Delta Rule*
(2406.06484); Lample et al., *Large Memory Layers with Product Keys* (1907.05242);
Elesedy & Teh, *Provably Strict Generalisation Benefit for Equivariant Models* (2102.10333);
Arjovsky et al., *Unitary Evolution RNNs* (1511.06464); Jing et al., *Gated Orthogonal
Recurrent Units* (1706.02761); Zhang et al., *Beyond Fully-Connected Layers with Quaternions*
(2102.08597); Komatsuzaki et al., *Sparse Upcycling* (2211.15841); Khandelwal et al.,
*Generalization through Memorization: kNN-LMs* (1911.00172); Wei et al., *T-MAC* (2407.00088);
Ma et al., *BitNet b1.58* (2402.17764).
