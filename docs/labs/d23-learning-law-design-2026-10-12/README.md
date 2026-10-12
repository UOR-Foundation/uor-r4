# The D23 replacement learning law: a softmax-free objective, read and output path — design and first rung

References #2028 (claim, [comment 6124719596](https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6124719596)),
[D23](../../integration/DECISIONS.md#d23---no-softmax-anywhere-in-the-active-project),
[D24](../../integration/DECISIONS.md#d24---no-cpu-model-compute-and-no-gpu-on-non-decisive-work),
the [softmax census](../softmax-census-2026-10-12/README.md). Lab: Kimi, session `kimi/s1`, 2026-10-12.
Design only: no training, no pod, no model run. Everything here is a proposal; the run waits for TEST
FITNESS, and the law itself is shared infrastructure for every lab, not a Kimi-only line.

> **One line.** Softmax does three jobs in this project — the training objective, the read's weighting, and
> the served output distribution — and D23/D24 remove all three at once. The replacement is a family, built
> from pieces that are already in the tree or have direct literature backing and need no normalized
> exponential anywhere: **a rank/margin objective** (target-vs-strongest-rival, with the in-tree
> `rank_softplus` as its soft form), **sparsemax** as the only normalized operator where a true probability
> simplex is required (piecewise-linear, no exponential), and **rank-margin credit** for the murmuration
> read's addresses (no teacher of any kind).

## 1. What must be replaced (from the census)

| role | today (banned) | candidate replacement | in-tree / literature anchor |
|---|---|---|---|
| Training objective | cross-entropy over softmax logits (LSE fwd, `exp(v−lse)` bwd) | **rank/margin**: `Σ_v relu(m − z_t + z_v)` or `softplus(m − z_t + max_{v≠t} z_v)` | `rank_softplus` (`cross_state_completion.rs:586-630`); raw-score margin API (`geometric_generate_learning.rs:1421`); the competing-mass hinge that already worked on the read (`read_binding_competing_margin`) |
| Read weighting | causal softmax + NoRead | already served softmax-free: rank / Hamming-rank / learned-rank tables | `rank_weights` (`geometric_stack.rs:15585-15600`), Q31 D11 rank read (`stack/session.rs:227-275`), the export gate (`stack_export.rs:268-321`) |
| Murmuration address learning | softmax surrogate, KL distillation, dense bootstrap (all banned) | **rank-margin on address scores** (true cell out-scores rivals by a margin) + non-softmax straight-through on the hard selection | the read-binding supervision lineage; delta-rule / sparsemax literature (below) |
| Served output distribution | `softmax(z)` head + mixture | greedy argmax over raw scores (already the served decode); where a distribution is needed, **sparsemax** | `sampling.rs:101-148` (already clean); Martins & Astudillo 2016 (arXiv:1602.02068) |
| Evaluation scoring | LSE-based NLL | margin loss + accuracy/recall panels; **no LSE-based NLL in new runs** | the frozen panels' deterministic checks |

Notes on the carve-outs D23 names: `softplus` and `sigmoid` are scalar exponentials, not softmax; the
carve-out does not automatically prohibit them, but the design prefers the pure-hinge form so the question
never arises. `sparsemax`'s α=2 entmax is piecewise-linear with a Euclidean projection — no exponential of
any kind; α=1.5 uses fractional powers, still exp-free, kept as the fallback.

## 2. The objective: rank/margin instead of cross-entropy

For each target position with gold token `t` and logits `z`:

- **Hard form:** `L = mean_v relu(m − z_t + z_v)` over the top few rivals (or only the strongest rival,
  `v* = argmax_{v≠t} z_v`). Gradient is 0/±1 on violated pairs — no normalization anywhere.
- **Soft form:** `L = softplus(m − z_t + z_v*)`, the in-tree `rank_softplus` shape. Smoother early training;
  flagged for owner confirmation because `softplus` is a scalar exponential (allowed by the carve-out, but
  the hard form avoids the question entirely).

The objective trains the *whole* model — recurrence, reads, pointer, head — through the same margin the
answer is judged by, which is also the honest fix for a known measurement problem: this project has never
had an objective that matches what its panels score. Known risks, stated now: (a) margin losses are not
calibrated NLLs, so **BPB/NLL comparisons to historical CE artifacts are invalid by construction** — the
rung reports margin curves and panel accuracy only, and says so; (b) a margin can under-train the long tail
of the vocabulary (only rivals near the boundary get gradient) — the rung watches the unviolated-target
rate; (c) rival selection (strongest-only vs top-k) is a real hyperparameter, swept at k ∈ {1, 8} in the
rung at trivial added cost.

## 3. The murmuration read's addresses, without any teacher

The starling ladder's actual blocker — hard selection gives no gradient to dropped candidates — was what
the banned softmax aids were papering over. The lawful mechanism:

1. **Address credit by margin:** for each query whose true source is known (the bench supplies it, as the
   distillation arms already used), a hinge `relu(m − s(true_cell) + s(rival_cell))` on the address scores
   over the candidate cells. This is exactly the read-binding supervision that worked on the softmax read
   (bound mass 0.73 → 0.90+), re-pointed at addresses instead of values — and it needs no probabilities,
   no teacher and no normalization.
2. **Readout stays the rank table** (already served softmax-free).
3. **The hard selection's gradient:** a non-softmax straight-through — the forward is the hard 600-cell
   keep, the backward passes the *identity* (or a finite-difference slope) to the address maps rather than a
   softmax relaxation. The linear STE is the weakest part of the design and is named as such; if it
   under-trains, the fallback is sparsemax-over-cells as the *selection* operator (still exp-free), which
   gives a real subgradient.

## 4. First rung (pre-registered for TEST FITNESS)

**Question:** does a margin objective train a small geometric stack to attempt the recall task at all —
and does the rank-margin address credit move the murmuration read's hit rate without a teacher?

- **Model:** geometric stack, width 256, pattern `rrarra`, 4 heads, MLP 384, context 384, vocab 4096,
  pointer dim 16 (the smoke-validated 4.4M configuration from the held oracle prep; MLP retained for rung 0
  — the owner's MLP-removal direction is rung 2+ via the E8/product-key line, kept separate so the objective
  question is not confounded).
- **Data:** the project's published memory-dialogue train (`mw-balp`, 7,512,797 tokens) + dev split.
- **Arms (2 seeds each, one pod, GPU only per D24):**
  - **M1:** hard-hinge objective, strongest-rival.
  - **M8:** hard-hinge objective, top-8 rivals.
  - **M1s:** softplus-margin, strongest-rival (the carve-out check).
  - **No CE arm** — re-running softmax for a baseline is itself prohibited (D24); the comparison reference
    is the historical recorded numbers of the preserved CE artifacts at their exact scope, quoted as
    cross-objective readings, never as like-for-like baselines.
- **Readouts:** dev margin loss and unviolated-target rate per arm; the frozen v5 memory panel's
  deterministic checks (40 rows, the same scorer the records use); mqar hit rate with the address credit
  on/off within each arm.
- **Bars (fixed in advance, noise posted):** an arm **works** if (a) the margin loss falls monotonically
  and the unviolated-target rate on dev exceeds the panel's no-model constant rate by ≥ 5×, on both seeds,
  and (b) the address credit lifts the mqar hit rate by ≥ +10 points over the same arm without it, on both
  seeds. Anything short of (a) is a tooling/objective blocker to fix, not a mechanism negative (D22).
- **Cost:** one 1×4090 pod, ≤ $3, leased/released per the pod rules; coordinate on #2037.
- **Stop rule:** three arms × two seeds, one run each; no silent arm additions.

## 5. What this is not

- Not a claim that a margin objective reaches CE-level language quality — that is exactly what the rung
  measures at its scope, with panel accuracy and margin curves instead of incomparable NLLs.
- Not a relaxation of D23/D24: no softmax, no softmax-shaped comparator, no CPU compute, no teacher.
- Not a replacement for the starling/SpiralCore track — it is the piece that track is blocked on.

## Next

1. This design, with the pre-registration, to TEST FITNESS on #2029.
2. Implementation of the three objectives (hard hinge k∈{1,8}, softplus-margin) behind a config flag —
   the in-tree `rank_softplus` and margin APIs are the starting points — with focused gradient/causality
   tests, as the preparation PR.
3. The rung on one pod, then the result record with KEEP/REJECT per arm.
