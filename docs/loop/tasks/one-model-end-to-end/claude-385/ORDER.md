# ORDER: one-model-end-to-end, claude-385 (V2(b): which vector read chooses the asked fact)

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407).
Base: `origin/main` at `cb91a1baa879feb9a16e4906aaa64a036411d845` (claude-381, V2(a), #3978). Roadmap step V2(b) of the 14 Nov audit.

## Goal

Run the selection test claude-381 pre-registered ([sheet](../../../../harness/prep/one-model-end-to-end-vector-read-selection-2026-11-14.md)) across the four reads V2(a) put behind flags (`--starling-read lorentz`, `--starling-read l2`, `--prime-route-read 1`, `--state-flock-read 4`) on the synthetic two-class task (B0 and B2, no ask table, no carry). A read passes when the asked value is at least 0.90 at both seeds and falls to R0's level with the read cut (eval only). If none passes, selection stays keyed by token identity: a configuration negative, not a mechanism close.

## Cancel probe

A vector read already trained on `--recall`. At base: none; `run_recall` refuses `config.vector_read`.

## Build first (claude-381's "Still open")

1. `run_recall` accepts a vector read: its optimizer, its credit in the loss, its counts and its save, as `run_long` does.
2. An eval-only cut reading `R^{-v}`: the read's weight set to 0 for the held-out pass and restored, bit for bit.
3. The read's choice at each held-out query's scored row: whether it picked the asked fact's relation position (the only earlier position followed by the asked value).
4. The Starling offset started from the address scale: on the first training forward, the median over rows of the nearest earlier position's distance (so about half the rows read at step 0). At 2.5 both Starling arms read 0 rows (claude-381, item 1).
5. Considered, not built: the 120 × 120 cell-energy table for prime-route. At context 192 the `(16, 192, 192, 32)` phase gap is 75 MB; it matters only past 256.

Credit weight and a stop-gradient on the stream under the address are FIT questions (claude-381, item 2), settled in the sheet before any run.

## Native-space five items (SKILL)

None is met. This is a float diagnostic selection test (D85: float stays reference or diagnostic code). It produces a choice, never a served model or an artifact; the winner's schema, shared operator, replay oracle, in-state update and serialization are V2(c).

## Files in scope

`vector_reads.rs`, `in_space_long.rs` (`run_recall` and the recall reading only; claude-380 edits `run_long`), `bin/train-in-space.rs` (flag gating only), the V2(b) sheet, this folder, the notebook row in `float-training.md`.

## Done when

1. Laptop tests that can fail, each with a negative gate: the recall path refuses nothing it should accept and still refuses `--recall` with the address memory; the cut reading restores the weight; the offset start is the median nearest distance.
2. GPU (wick): the cut leaves R0 unchanged; a vector read on `--recall` trains 64 steps with no NaN; flags-off `--recall` matches the base binary bit for bit.
3. The sheet passes `uor-harness prep-check` with a fresh non-author FIT before any arm.
4. The arms at two seeds, each with its cut reading, against the bar; a completion watcher on every run.
5. REPORT with negatives first, one notebook row, results sealed with `SHA256SUMS`.

## Stop rules

R0 above 0.02 on B0, or a non-finite loss: stop and repair. V2(c) is not run here.
