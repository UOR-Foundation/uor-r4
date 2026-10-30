# ORDER: one-model-end-to-end, claude-213 (Flock rank inside the nearest token's cell)

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md).

## Goal

Casey, 30 Oct: "do not rule flock out, improve it instead". On P-dev, claude-211's grade shows the two decode rules failing in opposite ways ([grade-registered.json](../../../../labs/one-model-end-to-end/training-2026-10-30/grade-registered.json), [grade-flock-end.json](../../../../labs/one-model-end-to-end/training-2026-10-30/grade-flock-end.json)), at seeds 1–3 of (r-drop):

| Rule | Complete, of 256 | Verbatim TRAIN share `v` (references' `v_ref` 0.566) | Repeated-4-gram rate |
|---|---|---|---|
| Greedy | 186–206 | 0.354–0.484 | 0.563–0.636 (it loops) |
| `FLOCK_END` | 255–256 | 0.004–0.012 (word salad) | 0.0006–0.0020 |

- **`FLOCK_END`'s draw is the cause of the salad.** It draws a harmonic rank over the 16 nearest tokens, so it lands below rank 0 about 70% of the time, including on tokens the model has separated from the nearest.

This unit builds and measures `FLOCK_CELL`, a geometric narrowing of that draw.
- **The candidate set:** the top-16 tokens whose score (negative mean L1 displacement) is within a margin `m` of the nearest's.
  - **Inside `m`,** the model has not separated a token from the nearest, and the harmonic draw chooses among them.
  - **Outside `m`,** it has, and the token is never drawn.
  - **The end of turn:** a nearest `<|eos|>` is always taken, as under `FLOCK_END`.
- **The margin ladder:** `m` ∈ {1.0, 0.5, 0.25, 0.125, 0.062, 0.031}. 1.0 is the cell objective's own margin (`rival_hinge`, `relu(1 + max_r s(r) − s(target))`), the gap training asks a target to keep over its nearest rival. It may admit all 16 (claude-211's train loss is 0.56 against that margin), so the narrower margins trace the path from `FLOCK_END` toward greedy.
- **Selection, then confirmation:**
  - DEV, already opened, selects the margin by a rule registered before any decode is read;
  - a new held-out-phrasing panel, p-c (drawn by `heldout-panel` at seed 2061, a seed no unit has used), confirms it.
  - P-dev and p-f stay as they are: P-dev was opened by the training run, and p-f is bar (f)'s alone.
- **The model:** (r-drop)'s sealed step-16,384 checkpoints, the recipe the training run ships, at all three seeds.

## Cancel probe (run first)

Result, 30 Oct (instrument; nothing blocks):
- **No other unit touches the decode.** No open PR changes `crates/uor-r4-float-train/src/in_space_long.rs` or `src/bin/train-in-space.rs`. The query was `gh pr list --state open` filtered on those paths, and it returned nothing.
- **The checkpoints are in the store.** `results/claude/one-model-end-to-end-2026-10-29/training-460fe46e/s{1,2,3}/r-drop-s{1,2,3}/models/step-016384/` holds `config.json`, `model.safetensors`, `hamming_read.safetensors`, `turn_memory.safetensors` and `turn_memory_reply.safetensors` at each seed.
- **The objective is `cell`.** It is the training sheet's command (`--objective cell`), so `m = 1.0` is the margin the run trained to.

## Files in scope

As the claim recorded:
- `crates/uor-r4-float-train/src/in_space_long.rs` and `crates/uor-r4-float-train/src/bin/train-in-space.rs`;
- `docs/harness/prep/one-model-end-to-end*`;
- `docs/labs/one-model-end-to-end/**` and `docs/loop/tasks/one-model-end-to-end/**`;
- `docs/research/notebook/float-training.md`.

## Done when

1. **`DecodeRule::FlockCell { k, margin_milli }` exists.**
   - `--flock-cell M,...` (margins in thousandths) adds each margin's rule to `--score-rows`' served decodes, with the memory on. Each writes `samples-{split}-flock-cell-{M}-memory.json`, which records every step's admitted count.
   - Unit tests pin the admitted count and the pick, with a negative gate.
   - The CUDA test checks that margin 0 decodes as greedy does on a model.
2. **`--cell-grade 1` grades the stored samples on the laptop.** For each split, rule and seed it gives the complete count, `v` against `v_ref` with its tolerance, the repeat rate and the mean admitted count. Then it applies the registered selection on DEV and the registered decision on p-c. It is unit-tested.
3. **The panel p-c and its census** are in the store and in `docs/labs/one-model-end-to-end/`.
4. **A prep sheet, with a fresh FIT and a fresh pre-run review,** registers the selection rule and the decision table before any decode is read.
5. **The GPU decode of (r-drop) at three seeds** is done, with greedy, `FLOCK_END` and the ladder on DEV and p-c. It is sealed in the store, graded, and reported, negatives first and tokens per second first. The report includes a fixed reading of 20 p-c replies per rule.

## Stop and ask if

Stop and decide; never wait.
- **A fault in the instrument, not the rule** (a reproduced file differs, or the decode errors). Repair it and re-run; it is never a negative (CHARTER §4.1).
- **No margin of the ladder holds on DEV.** Record it as a negative for this configuration, never for the rule (§4.2). The next narrowing comes from the same geometry: the Hamming read's address cell, in place of a score margin.
