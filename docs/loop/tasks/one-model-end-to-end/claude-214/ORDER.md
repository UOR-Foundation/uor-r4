# ORDER: one-model-end-to-end, claude-214 (do the copy heads make the reply loop?)

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md). Base: `f67149842`.

## Goal

Casey's GO is one working model end to end ("FIGURE OUT WHAT THE HELL THE BEST MECHANISM IS"). Its replies fail in two ways (claude-211, claude-213):
- **Greedy loops.** On p-c, at seeds 1–3 of (r-drop), its repeat rate is 0.559–0.628 and only 192–202 of 256 replies end.
- **The Flock draws avoid loops only by producing word salad.**

claude-213 found that inside a loop the nearest token is confident: 95.9–99.6% of loop steps admit exactly one candidate. So no draw rule can break a loop. The cause has to be upstream, in what makes the trunk confident.

**The candidate cause is the Hamming read's copy heads** (`flock_read_hamming.rs`).
- **What they do:** at position `t`, a copy head finds an earlier `s` whose previous token is the current token, and brings back the token at `s`, "the token after the last copy of the current token".
- **The loop:** in free generation the read also sees the reply's own earlier tokens. Once a token repeats inside the reply, the head brings back what followed it last time, which closes the loop.

**This unit tests that with a decode-time scope on the read,** changing nothing in training or the checkpoints:
- **`full`:** the read as trained.
- **`copy`:** the copy heads choose only prompt positions; the type heads are as trained.
- **`all`:** every Hamming head chooses only prompt positions.

Each scope is decoded greedy, with the memory on, on (r-drop)'s sealed step-16,384 checkpoints at seeds 1–3. They are compared on DEV and on a new panel, since p-c was opened by claude-213.

## Scope change (a decision, logged on #3407)

This unit adds `crates/uor-r4-float-train/src/flock_read_hamming.rs` to the item's scope, with a fresh decision review.
- **No open PR touches the file.**
- **No other item's scope holds it.** `route-plus-memory`, which held `flock_read*.rs`, is done.
- **The change is additive:** a decode-time limit read from a thread-local that is `None` by default, so training and every existing reading are unchanged.

## Cancel probe (run first)

Result, 30 Oct (instrument; nothing blocks):
- **No open PR touches** `in_space_long.rs`, `train-in-space.rs` or `flock_read_hamming.rs`.
- **The checkpoints** are the ones claude-213 checked against their `SHA256SUMS`, at `results/claude/one-model-end-to-end-2026-10-29/training-460fe46e/s{1,2,3}/r-drop-s{1,2,3}/models/step-016384/`.
- **claude-213 is merged and verified** (`f67149842`).

## Files in scope

- `crates/uor-r4-float-train/src/flock_read_hamming.rs` (the decision above), `crates/uor-r4-float-train/src/in_space_long.rs` and `crates/uor-r4-float-train/src/bin/train-in-space.rs`;
- `docs/roadmap/items/one-model-end-to-end.md` (the scope line);
- `docs/harness/prep/one-model-end-to-end*`;
- `docs/labs/one-model-end-to-end/**` and `docs/loop/tasks/one-model-end-to-end/**`;
- `docs/research/notebook/float-training.md`.

## Done when

1. **The read's decode-time scope exists:** `ReadScope { Full, Copy, All }`, set per decode step by `decode_reply`. `--reply-read copy,all` adds greedy samples under each scope to `--score-rows`.
   - Unit tests pin the cut and that `Full` changes nothing, with negative gates.
   - A CUDA test checks that `Full` decodes exactly as before on a model, and that `All` reads no reply position.
2. **A grade on the laptop** gives every scope's readings against greedy. It applies the decision registered in the sheet, and is unit-tested.
3. **A new held-out panel and its census,** drawn by `heldout-panel` at an unused seed.
4. **A prep sheet, with a fresh FIT and a fresh pre-run review,** written before any decode is read.
5. **The GPU decode at three seeds,** sealed, graded and reported, negatives first and tokens per second first. It includes the loop-step share and a fixed reading of 20 replies per scope.

## Stop and ask if

Stop and decide; never wait.
- **A fault in the instrument, not the mechanism:** a registered file does not reproduce, or `Full` differs from greedy. Repair it; it is never a negative (CHARTER §4.1).
- **Neither scope reduces the loops.** Record a negative for this cause, in this configuration only. The loop-step data then point to the next candidate: the turn memory's reply table, or the trunk itself.
