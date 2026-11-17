# ORDER: one-model-end-to-end, codex-47 (the step's cost, in the order to fix it)

Lab: codex. Nothing here changes the model's maths, the served path or a bar. Every unit is
"same outputs, less work", verified before it is believed.

The starting point is measured: **0.74 s/step = 5,500 tokens/s** at 30 M values (512 width, 12
layers, 8,192 vocab, 512 context), i.e. **≈ 17 h per 332 M-token epoch**. Three of the four largest
costs are the same shape: a full-width intermediate materialised to compare identities.

## U0 — Measure first (no code)

`train-in-space --profile 16` at the current shape, into `profile.json`. Deliver the stage table.
Everything below is aimed by that table; if a stage is not where this order expects it, the order
changes rather than the table.

## U1 — The objective's whole-table scan (largest expected win)

**Now:** `nearest_loss_on` calls `nearest_scores(query, table)` — `rows × vocabulary × width` =
**17.2 G distance elements, ≈ 69 GB of intermediate per forward** — then 8 rounds of `best_positions`.
The bounded fused pass (`top8_positions::nearest_positions`, `--fast-rivals 1`) exists but the
**cell objective never takes it** (`if FAST_RIVALS && !cell`).

**Fix:** allow the fused pass for the cell objective, or (if it cannot serve the cell hinge) shortlist
with the writer's own admission — 8 cells × 32 slots = 256 candidates of 8,192 — and exact-score
those. The second is the geometric route: the identity shortlist is the writer's own choice rule.

**Gate:** on a fixed 4,096-row sample, the rivals and the loss equal the full scan (exact), or the
shortlist's miss rate is reported and the objective is redefined on the shortlist — no silent change
of the bar. Then tokens/s.

## U2 — The route per output, not per row

**Now:** `spiralcore_fbs_fwd`/`_bwd` compute `sc_fbs_route` (and the sibling counterfactuals) once per
`(row, output)` item. The route depends only on the output block's query and the tables, so it is
recomputed `rows` (4,096) times, per projection, per layer, in **both** directions.

**Fix:** a `routes_fwd` kernel over `outputs` (64 threads a projection) writing `(atom, state)` per
block; the gather reads it. Backward: precompute the per-output sibling set; the per-row kernel only
pairs it with `dL/dy`.

**Gate:** bit-identical forward outputs and gradients on a fixed batch (the route is unchanged, only
its recomputation is removed). Then tokens/s.

## U3 — The writer's candidate inventory

**Now:** `kept_cells (8) × slots (32) × width (512)` per row = **537 M elements per step**, scored as
`broadcast_sub → abs → mean` over the full width.

**Fix:** the candidates are signed gathers of the table — 8 coordinates a block. Score on the block
identity (or stream the inventory in a fused kernel) instead of materialising 512 lanes a candidate.

**Gate:** identical picks and scores on a fixed batch. Then tokens/s.

## U4 — The per-stage quantisers

**Now:** every projection output and residual add passes `q16`/`to_exp_i32`, ~2 M values × 72
projections a step, as a separate pass.

**Fix:** fuse the conversion into the gather kernel.

**Gate:** bit-identical stage outputs. Then tokens/s.

## U5 — The optimiser pass

**Now:** ~30 M parameters of AdamW state read and written each step.

**Fix:** fused/foreach updates, or a lower-precision moment state if the native-space rule allows it.

**Gate:** identical updates for one step. Then tokens/s.

## Order and rationale

| unit | expected share | risk | depends on |
|---|---|---|---|
| U0 | — | none | — |
| **U1** | largest (the 17.2 G-element scan) | medium: the rivals must not move | U0 |
| **U2** | large (×4,096 redundancy) | low: bit-identical | U0 |
| U3 | medium | medium: the shortlist changes the score | U1 |
| U4 | medium (traffic tax on every layer) | low: bit-identical | U2 |
| U5 | small | low | — |

U1 and U2 are independent and are the two that matter; do them first and in that order. U3 follows
U1 because both are the same shortlist question. U4 rides on U2's kernel rewrite. U5 last.

## What this unlocks

At 3× the step becomes ~0.25 s (≈ 16,500 tokens/s) and one epoch **≈ 5.7 h**; at 5×, ~4 h. That is
what makes a three-epoch language run affordable, and it is measured at every unit rather than
assumed.

## Out of scope

Any dense or transformer comparison (D28); changing the objective's definition without saying so;
the read's write scale and the other-fact ceiling (claude's line).
