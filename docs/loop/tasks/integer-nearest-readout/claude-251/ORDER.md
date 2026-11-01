# ORDER: integer-nearest-readout, claude-251 (Q5's trace instrument: F's side)

Lab: claude. Item: [integer-nearest-readout](../../../../roadmap/items/integer-nearest-readout.md) (#3462). Base: `739a0ca6e`. Rule: claude-247's [decision card](../claude-247/DECISION.md).

## Goal

Build F's side of Q5's three-way trace. R's side merged in claude-249 (#3499), and the files this needs entered the scope in claude-250 (#3512). Casey, 1 Nov: "Finish Q5 first" (#3462, `6564833770`).

F is F′ rerun: `train-in-space --score-rows` with claude-243's arguments for the seed ([sheet](../../../../harness/prep/integer-nearest-readout-tie-parity-2026-11-01.md)), on the GPU, with the trace on. F is a step's reference side only, so it is never forced. This unit records; T's rounded copy and the float side's forcing come next.

**Where F records: inside `score_rows`' own forward,** the computation that wrote F′'s sealed rows. A panel prompt is its window's `input_ids[..=scored_offsets[0]]`, and its `position` is the window's first scored position (`split_prompts`, `in_space_long.rs`). So window row t is R's step t, and F's windows join R's by `position`, with `ids` and `reply_start` as R writes them.

**What F records,** in `uor-r4.decision-trace/1` with side `F`, at rows 0 to the last scored offset of each traced window, in the memory pass:
- **Codes:** each read's query and key per head, as R's word and as the f32 coordinates the sign reads. The near-tie rule reads |v| on a reference side.
- **Kept sets:** the existing Hamming trace's real picks (`flock_read_hamming.rs` `trace_head`), sorted as R sorts them: nearest first, ties to the earlier position.
- **The turn memory's lookups:** key, stored token, and whether the row opens the reply (`flock_read_memory.rs` `stored`, `stored_reply`).
- **Rate buckets:** each layer's lanes, with the score as `rate_select` reads it (f32) and the entry `min(⌊s·16⌋, 15)` as the kernel computes it (`cuda_stack.cu`), recorded at `geometric_stack_native.rs`'s `rate_select` call.
- **The nearest,** at each scored offset: the GPU's top two by `nearest_scores` (−mean L1, f32), the pick equal to its argmax. Both scores are recomputed on the host in R's units, f64 −2^12·Σ|q − e|, from F's f32 query, which is recorded so any token's score can be computed.

**Schema:** F's floats are stored as `to_bits()` in new fields with `#[serde(default, skip_serializing_if = …)]`, so `Eq` holds and values round-trip exactly. R-only numbers become optional the same way. R's files must serialize byte for byte as before, and older files must still parse. `HammingTraceHead` gains the same optional coordinates, written only behind a second switch.

**The margin cut** (`relu(m − d)`, `weights.gt(0)`) is a function of the recorded codes and each head's margin parameter. The REPORT classes it for I-Complete; it is not a sixth record.

## Cancel probe (run first)

- Does `score_rows` still decode by `nearest_scores(...).argmax(1)`, and does `split_prompts` still cut prompts from window offset 0?
- Does `params()` still require F32 CUDA (`geometric_stack_native.rs`)? Then F's path is plain f32, and there is no host fallback.
- Are claude-243's F′ rows, the seed's checkpoint and the DEV split reachable from a pod (HF store)?
- **Classed:** instrument.

## Files in scope

The item's scope. This unit edits `fbs_snapshot.rs`, `bin/fbs-toks.rs` (the R-only scores become optional), `flock_read_hamming.rs`, `flock_read_memory.rs`, `geometric_stack_native.rs`, `in_space_long.rs`, `bin/train-in-space.rs`, this folder and the item.

## Done when

1. **The trace.** `train-in-space --score-rows … --decision-trace TRACE --trace-positions POSITIONS` writes side F's trace for the named windows only, in R's order. The order is: layer by layer, the buckets in lane order and each read at its depth; then the readout read, the memory and the nearest.
2. **It changes nothing.** With the trace on, the rows files are byte-identical to the trace-off run's.
   - R's trace on claude-249's smoke still hashes `11e6b7d9…`.
   - An R trace written before this unit parses and re-serializes byte for byte.
3. **Tests.** Host tests on the laptop for the pure pieces: the entry, the f64 score, the top-two order and the kept order. CUDA finite tests on a pod, since the float path has no host fallback (CHARTER §3.5), cover:
   - the trace's order and lengths on a small trunk;
   - a recorded entry against the kernel's λ;
   - a recorded code against `pack_signs`;
   - the pick against `argmax`.

   Each piece's negative gate fails with that piece set aside. The crate also passes a stub-nvcc `cargo check --features cuda`.
4. **I-F, the smoke, outside Q5's set.** On DEV s1, at the seed's checkpoint, F's traced picks equal F′'s sealed `nearest` at every traced scored position. Times are recorded. No row of the set is run.
5. **A fresh exact-head review passes,** in slices of at most 600 lines. The REPORT puts the negatives first. No notebook row: the unit measures no result.

## Stop and decide (never wait) if

- F′'s sealed rows cannot be reproduced with the trace off. That is an I-F instrument fault, repaired first;
- a kind cannot be read as the decision reads it without changing the served computation.
