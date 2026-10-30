# ORDER: one-model-end-to-end, claude-215 (do the copy heads make the reply loop?)

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md). Base: `df3c352c4`.

## Goal

Casey's GO is one working model end to end ("FIGURE OUT WHAT THE HELL THE BEST MECHANISM IS"). Its replies fail in two ways (claude-211, claude-213):
- **Greedy loops.** On p-c, at seeds 1–3 of (r-drop), its repeat rate is 0.559–0.628 and only 192–202 of 256 replies end.
- **The Flock draws avoid loops only by producing word salad.**

claude-213 found that inside a loop the nearest token is confident: 95.9–99.6% of loop steps admit exactly one candidate, so no draw rule can break it.

**The candidate cause is the Hamming read's copy heads** (`flock_read_hamming.rs`).
- **What they do:** at position `t`, a copy head finds an earlier `s` whose previous token is the current token, and brings back the token at `s`.
- **The loop:** in free generation the read also sees the reply's own earlier tokens. So once a token repeats inside the reply, the head brings back what followed it last time.

**The test** is a decode-time scope on the read. Training and the checkpoints are unchanged.
- **`full`:** the read as trained.
- **`copy`:** the copy heads choose only prompt positions; the type heads are as trained.
- **`all`:** every Hamming head chooses only prompt positions.

Each scope is decoded greedy, with the memory on, on (r-drop)'s sealed step-16,384 checkpoints at seeds 1–3. They are compared on DEV and on the new panel p-r (`heldout-panel` seed 2071; p-c was opened by claude-213).

**Bound by claude-214's scope decision** (#3407 6518346519, decision review PASS):
- `choose_sorted` takes an explicit limit;
- the default path is unchanged byte for byte;
- the cut is `s < prompt.len() − start` in window coordinates, and `P ≤ 1` reads nothing;
- the limit is set only around `decode_reply`'s forward and cleared before any `?`;
- a negative gate covers the cut.

## Cancel probe (run first)

Result, 30 Oct (instrument; nothing blocks):
- **The scope:** `flock_read_hamming.rs` is in the item's scope on main (claude-214, `df3c352c4`).
- **Open PRs:** none touches `in_space_long.rs`, `train-in-space.rs` or `flock_read_hamming.rs`.
- **The checkpoints:** the ones claude-213 checked against their `SHA256SUMS`, at `results/claude/one-model-end-to-end-2026-10-29/training-460fe46e/s{1,2,3}/r-drop-s{1,2,3}/models/step-016384/`.
- **The panel p-r** is drawn, byte-identical on a second draw. It is stored at `inputs/claude/heldout-panel-p-r-2026-10-30/` with a `SHA256SUMS` and its census:
  - 0.000 of 1,453 prompts are in TRAIN, exactly or as templates at K = 1,024;
  - 0.632 of its replies are verbatim TRAIN replies.

## Files in scope

As the item's scope: `crates/uor-r4-float-train/src/{flock_read_hamming.rs,in_space_long.rs,bin/train-in-space.rs}`, `docs/harness/prep/one-model-end-to-end*`, `docs/labs/one-model-end-to-end/**`, `docs/loop/tasks/one-model-end-to-end/**` and `docs/research/notebook/float-training.md`.

## Done when

1. **The read's decode-time scope exists:** `ReadScope { Full, Copy, All }`, set per decode step by `decode_reply`. `--reply-read copy,all` adds greedy samples under each scope to `--score-rows`.
   - Unit tests pin the cut and the per-head limit, with negative gates.
   - A CUDA test checks that a set cut keeps every pick before it, and that `Full` decodes exactly as before on a model.
2. **A laptop grade** (`--read-grade 1`) gives every scope's readings against greedy and applies the sheet's registered decision. It is unit-tested.
3. **The panel p-r and its census** are in the store and in `docs/labs/one-model-end-to-end/`.
4. **A prep sheet, with a fresh FIT and a fresh pre-run review,** written before any decode is read.
5. **The GPU decode at three seeds,** sealed, graded and reported, negatives first and tokens per second first. It includes a fixed reading of 20 p-r replies per scope.

## Stop and ask if

Stop and decide; never wait.
- **A fault in the instrument, not the mechanism:** a registered file does not reproduce. Repair it; it is never a negative (CHARTER §4.1).
- **Neither scope reduces the loops.** Record a negative for this cause, in this configuration only. The next candidates are the turn memory's reply table, or the trunk itself.
