# REPORT: integer-nearest-readout, claude-251 (Q5's trace instrument, the float side F)

Lab: claude. Item: [integer-nearest-readout](../../../../roadmap/items/integer-nearest-readout.md) (#3462). Order: [ORDER.md](ORDER.md). Rule: claude-247's [decision card](../claude-247/DECISION.md). References #3462.

## Tokens per second

No decode speed is claimed, and no served path changes. R's decode speed is still claude-243's: 1,204 tok/s median decode on R s1.
- On the GPU, F′'s command at seed 1 took 323 s with the trace on and 322 s with it off, side by side on one RTX 5090. Both processes ran at once, so this is an instrument's cost, not a serving reading. The trace covered 308 DEV windows, 36,541 steps.

## What did not work, first

- **Q5 is not read, and nothing here moves done-when 3.** This unit builds F's side of the trace only. T's side with its rounded copy, the replay driver, the near-tie grader and the sheet come next, in that order.
- **The option differs from the ORDER's.** The ORDER wrote `--decision-trace TRACE --trace-positions POSITIONS`. This unit's is `--decision-trace SPLIT=POSITIONS_JSON`, repeatable, and it writes `decision-trace-SPLIT.json` beside the rows in `--out`.
  - **Why:** Q5's set spans DEV and p-w (169 windows over 6 cells), and a position names a window only within its split.
  - It traces in the memory pass only, the pass F′'s rows are read from.
- **The smoke's full trace was lost with its pod.** `uor-pod down --to-store` archives only `uor-r4/{jobs,pods,runs,evals}` under `/workspace`. The job wrote to `/workspace/trace-f`, so `down` reported "Archived 0 MB" and deleted it, along with the 85 MB gzipped trace (sha256 `308c3d73…` uncompressed, in the pod's `SHA256SUMS`).
  - Every claim below rests on files pulled before the pod went down: a summary of every traced window, with its sha256 taken on the pod; the six traced rows files; the sums; the logs.
  - No claim needs the full trace. The next unit reruns F with T beside it.
  - Logged in `local/claude/FRICTION.md`. Lesson: write a pod job under `/workspace/uor-r4/runs/`, or `uor-store put` its folder and read it back before `down`.
- **The smoke ran F′'s command with `--samples 1`, not 1,536.** `score_rows` writes every rows file before it decodes a sample, and F′'s decode took about 3.4 h per process. The rows match F′'s sealed rows byte for byte, which shows the change leaves them untouched.
- **The first host test of the recorder could not run,** because CPU tensor compute is refused (D24/D29).
  - The recorder (`FTrace::record`) now reads host slices only.
  - `row_picks` makes the device copies: the traced rows of the points and of the scores, and the table once per trace.
- **R's smoke windows stop at the first reply's end; F's run to the window's last scored offset,** as the ORDER says. On DEV s1, F's 308 windows hold 36,541 steps and R's hold 17,491.
  - R's steps are an exact prefix of F's in all 308 windows (the alignment check below). The grader reads R's range and ignores F's later turns.
- **The margin cut is not recorded,** as the ORDER says. It is `relu(m − d)` and `weights.gt(0)`: the integer bit distance `d` from the recorded codes, against each head's margin `m`, a checkpoint constant.
  - **For I-Complete it is classed as derived:** a kept position is weighted exactly when `d < m`. So one per-head check of the exported threshold against F's `m` covers every step. It is not a sixth kind.
- **Two refusals have no test of their own:** a split named twice, and a split no panel holds, both by `--decision-trace`. They are argument checks in `score_rows`, and the smoke ran neither.
- **The suites carry failures this unit did not add:**
  - **Clippy with `-D warnings`** fails on 7 lints in the float crate and 52 in the integer crate. All of them are in code this unit does not touch (for example `decode_speed`'s 10 arguments, `reply_parts`'s type, `codec.rs`). None is on a line this unit adds.
  - **`stack_chat_reference`,** one of the integer crate's integration tests, does not compile without `serving-wasm`, because `stack` is gated (`native_geometry.rs`). The lib and bins pass under both feature sets.
- **The disk was under the 30 GiB floor (22 GiB free).** This unit's one build folder is 3.0 GB. No DeepSeek worktree was left in `/private/tmp` to remove.
- **The ladder gave a 2 × 5090 pod at $2.157/h,** and one GPU was used. The pod ran about 16 minutes, about $0.57.

## What worked

- **F's trace records the card's five kinds in R's order,** at rows 0 to each traced window's last scored offset, in `uor-r4.decision-trace/1` with side `F`. Every float is stored as `to_bits()`, so values round-trip exactly.
  - **Rate buckets:** each layer's 64 lanes, with the score as `rate_select` reads it (`score_f32`) and its entry, `min(⌊s·16⌋, 15)`.
  - **Reads:** the stream read after the layer it follows (layer 2), then the readout read. Each head carries its query and key words, the f32 coordinates their signs read (`query_f32`, `key_f32`), and its kept set, nearest first by bit distance, ties to the earlier position.
  - **Memory:** each row's lookup, with its key, its stored token and whether it opens the reply, from the turn memory's own tables (`MemoryLookup`).
  - **Nearest:** at each scored offset, the GPU's pick and runner-up from `nearest_scores` (ties to the smaller id). Both scores are in R's units, f64 `−2^12·Σ|q − e|`, and F's f32 query is recorded too, so any token can be scored.
- **R's files are unchanged.** R's trace on claude-249's smoke still hashes `11e6b7d9…` and its rows `74c22378…`. An R step re-serializes byte for byte, and a float step writes no `_q12` or `_q16` field (the snapshot test).
- **On the GPU, at `928465dad`,** the crate's CUDA library suite passed: 272 tests, 0 failed, 6 ignored. Among them, `a_traced_forward_records_the_kernels_choices_in_rs_order` checks a small trunk with both reads:
  - tracing changes no pick;
  - the traced window has its order and lengths;
  - each recorded entry is the kernel's bucket (`rate_select` returns that entry's λ);
  - each code is `pack_signs` of its coordinates;
  - each kept set is in R's order;
  - each pick is the kernel's argmax for its recorded query.
- **I-F, the smoke outside Q5's set, passes.** It ran F′'s command at seed 1 on the sealed step-16,384 checkpoint, tracing DEV s1's 308 windows of R's smoke.
  - **The trace changes nothing.** The six rows files are equal byte for byte with the trace on and off, and equal to F′'s sealed rows (claude-243).
  - **Every pick equals F′'s sealed nearest:** 10,840 of 10,840 traced scored rows. Each runner-up differs from its pick.
  - Both runs rebuilt the opening table to F′'s sealed hash, `654fc762…`.
  - No row of the set (4763 or 10757) was traced, so no trace value Q5 reads has been seen.
- **F's windows join R's exactly** (I-Align's shape), in all 308 windows: same position and `reply_start`, R's ids a prefix of F's, and the same steps carrying a nearest and a memory lookup across R's range.
- **Tracing refuses the reads R does not serve:** the Flock read, the content read, the opening read and the prompt register.

## What changed

- **`crates/uor-r4-integer/src/fbs_snapshot.rs`:**
  - `HammingTraceHead` gains `query_f32` and `key_f32`.
  - `DecisionLayer` gains `score_f32`; `score_q16` is written only where it is set.
  - `DecisionHead` gains its coordinates.
  - `DecisionNearest`'s Q12 scores and query become optional, beside `score_f64`, `runner_score_f64` and `query_f32`.
  - Every new field defaults and is skipped when empty, so R's files are unchanged.
- **`crates/uor-r4-integer/src/stack/hamming.rs`, `stack/session.rs`, `bin/fbs-toks.rs`:** R builds the same records with the new fields empty. R's values do not change. The ORDER's file list was amended to name the first two.
- **`crates/uor-r4-float-train/src/flock_read_hamming.rs`:** `set_hamming_trace_coords` is a second switch. With it on, the head trace keeps each code's coordinates.
- **`crates/uor-r4-float-train/src/flock_read_memory.rs`:** `TurnMemory::lookups` returns each row's key, stored token and opening flag, by the move's own rule. The opening keys are shared with `stored` (`opening_keys`).
- **`crates/uor-r4-float-train/src/geometric_stack_native.rs`:** `set_rate_trace` and `take_rate_trace` copy each layer's rate scores before `rate_select`, only while a trace is on.
- **`crates/uor-r4-float-train/src/in_space_long.rs`:**
  - `FTrace` names a split's windows by position and refuses a position named twice or naming no window.
  - `row_picks` fills it in its own forward.
  - `score_rows` takes the traces, refuses an unserved read, and writes one file per split.
- **`crates/uor-r4-float-train/src/bin/train-in-space.rs`:** `--decision-trace SPLIT=POSITIONS_JSON`, for `--score-rows` only.
- **Tests:**
  - three host tests: `a_traced_window_records_the_five_kinds_in_rs_order`, `each_rows_lookup_is_the_key_and_entry_the_move_reads`, `a_head_trace_with_coordinates_keeps_them_bit_for_bit`;
  - one GPU test: `a_traced_forward_records_the_kernels_choices_in_rs_order`;
  - one snapshot test: `an_r_step_writes_as_before_and_a_float_step_keeps_its_bits`.
- **The item's "What is missing"** marks F's side built and names T's side next.

## Checks

| Gate | Command | Result |
|---|---|---|
| Host tests (float) | `cargo test -p uor-r4-float-train --lib -- a_traced_window_records each_rows_lookup a_head_trace` | 4 passed |
| Snapshot and integer lib | `cargo test -p uor-r4-integer --lib`; with `--features serving-wasm --lib --bins` | 61 passed; 413 passed, 1 ignored, bins pass |
| CUDA type-check | `CUDARC_CUDA_VERSION=12080 NVCC=<stub> UOR_NVCC_ARCH=compute_120 cargo check --offline --locked --features cuda --all-targets -p uor-r4-float-train` | exit 0 |
| Clippy, lines this unit adds | `cargo clippy --no-deps --features cuda --all-targets -- -D warnings` (float); `--features serving-wasm --lib --bins` (integer) | 0 findings on added lines; 7 and 52 in code this unit does not touch |
| CUDA library suite (RTX 5090, `928465dad`) | `cargo test --release --locked -p uor-r4-float-train --features cuda --lib -- --test-threads=1` | 272 passed, 0 failed, 6 ignored |
| I-F: on = off | `cmp`, the six rows files, trace on against off | 6 equal |
| I-F: = F′ sealed | `cmp` against claude-243's sealed s1 rows, themselves checked against its `SHA256SUMS` | 6 equal |
| I-F: picks | each traced nearest against `rows-dev-memory.json`'s `nearest` at its position | 10,840 of 10,840 |
| Alignment with R | position, `reply_start`, ids prefix, nearest and memory steps, against R's smoke trace | 308 of 308 |
| R unchanged | claude-249's smoke at this head | trace `11e6b7d9…`, rows `74c22378…` |

The evidence is sealed at `results/claude/integer-nearest-readout-trace-f-2026-11-02/smoke/` (23 files; the laptop manifest's sha256 is `a1eb4270…`). It holds the pod's sums and logs, the traced rows, the trace summary (`27c0a80c…`), the positions (`960f4340…`) and the job.

## Negative gates

Each mutation was applied alone, its test run, and the file restored and checked with `cmp`. Every test failed at an assertion, not a build.

| Piece set aside | Mutation | Test | Result |
|---|---|---|---|
| Kept sets in R's order | drop `kept.sort_by_key` in `kept_in_order` | `a_traced_window_records_the_five_kinds_in_rs_order` | FAILED (exit 101) |
| Runner-up, ties to the smaller id | `s > b` to `s >= b` in `runner_up` | the same | FAILED |
| The opening flag | `opening: true` to `false` in `lookups` | `each_rows_lookup_is_the_key_and_entry_the_move_reads` | FAILED |
| Coordinates behind their switch | `if !coords` to `if true` in `trace_head` | `a_head_trace_with_coordinates_keeps_them_bit_for_bit` | FAILED |
| R's bytes kept | drop `score_q16`'s skip when empty | `an_r_step_writes_as_before_and_a_float_step_keeps_its_bits` | FAILED |

The GPU test's checks run on the device only. Its pieces (the rate copy, the coordinates, the device copies of `row_picks`) are pinned there and by I-F's equalities.

## Next

T's side, under the same card: T's rounded copy beside `--rebind-fbs-backward`, writing the same schema, with its own smoke against R's 308 windows. Then the replay driver and the near-tie grader, then the sheet with a fresh FIT.
