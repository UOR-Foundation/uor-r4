# REPORT: integer-nearest-readout, claude-251 (Q5's trace instrument, the float side F)

Lab: claude. Item: [integer-nearest-readout](../../../../roadmap/items/integer-nearest-readout.md) (#3462). Order: [ORDER.md](ORDER.md). Rule: claude-247's [decision card](../claude-247/DECISION.md). References #3462.

## Tokens per second

No decode speed is claimed, and no served path changes. R's decode speed is still claude-243's: 1,204 tok/s median decode on R s1.
- On the GPU, F′'s command at seed 1 took 323 s with the trace on and 322 s with it off, side by side on one RTX 5090, in both rounds. Both processes ran at once, so this is an instrument's cost, not a serving reading. The trace covered 308 DEV windows, 36,541 steps.

## What did not work, first

- **Q5 is not read, and nothing here moves done-when 3.** This unit builds F's side of the trace only. T's side with its rounded copy, the replay driver, the near-tie grader and the sheet come next, in that order.
- **Round 1 recorded the codes as the coordinates.** The read copied each head's query and key after its `code`, which makes them ±1 by `p ≥ 0`. So `query_f32` and `key_f32` held ±1, not the values the sign reads, and the near-tie rule could not have read |v| from them.
  - **How it was found.** The fresh review's slice C, at `a571cd3cb`, found that the GPU test's "code against `pack_signs`" check could not fail: the word and the coordinates came from the same host rows. Reading why showed the codes.
  - **The fix (`8dafbe2cc`).** While the trace and its coordinates are on, the read keeps its query and key before the sign, and `trace_head` refuses coordinates that do not sign to their row's word. Nothing the read computes changes.
  - **Round 2 reran the GPU tests and I-F at the fix.** The CUDA suite passed again, I-F's equalities held again, and none of the trace's 37,417,984 recorded coordinates is ±1. A GPU check now fails on the old behaviour (gate m6 below). Round 1's trace, which held the ±1 coordinates, was lost with its pod, and no claim rests on it.
- **The option differs from the ORDER's.** The ORDER wrote `--decision-trace TRACE --trace-positions POSITIONS`. This unit's is `--decision-trace SPLIT=POSITIONS_JSON`, repeatable, and it writes `decision-trace-SPLIT.json` beside the rows in `--out`.
  - **Why:** Q5's set spans DEV and p-w (169 windows over 6 cells), and a position names a window only within its split.
  - It traces in the memory pass only, the pass F′'s rows are read from.
- **The smoke's full trace was lost with its pod.** `uor-pod down --to-store` archives only `uor-r4/{jobs,pods,runs,evals}` under `/workspace`. The job wrote to `/workspace/trace-f`, so `down` reported "Archived 0 MB" and deleted it, along with the 85 MB gzipped trace (sha256 `308c3d73…` uncompressed, in the pod's `SHA256SUMS`).
  - Every claim below rests on files pulled before the pod went down: a summary of every traced window, with its sha256 taken on the pod; the six traced rows files; the sums; the logs.
  - No claim needs the full trace. The next unit reruns F with T beside it.
  - Logged in `local/claude/FRICTION.md`. Lesson: write a pod job under `/workspace/uor-r4/runs/`, or `uor-store put` its folder and read it back before `down`.
  - **Round 2 kept its trace.** Its job ran under `/workspace/uor-r4/runs/claude-251-r2`, and `down --to-store` archived 718 MB and read back 115 files. The gzipped trace is at `vast-archive/vast-58739274/uor-r4/runs/claude-251-r2/on/r-drop-s1/`. The uncompressed copy was removed on the pod first, once `gunzip -c` hashed to its `SHA256SUMS` line, `9e7c9bf3…`.
- **The smoke ran F′'s command with `--samples 1`, not 1,536.** `score_rows` writes every rows file before it decodes a sample, and F′'s decode took about 3.4 h per process. The rows match F′'s sealed rows byte for byte, which shows the change leaves them untouched.
- **The first host test of the recorder could not run,** because CPU tensor compute is refused (D24/D29).
  - The recorder (`FTrace::record`) now reads host slices only.
  - `row_picks` makes the device copies: the traced rows of the points and of the scores, and the table once per trace.
- **R's smoke windows stop at the first reply's end; F's run to the window's last scored offset,** as the ORDER says. On DEV s1, F's 308 windows hold 36,541 steps and R's hold 17,491.
  - R's steps are an exact prefix of F's in all 308 windows (the alignment check below). The grader reads R's range and ignores F's later turns.
- **The margin cut is not recorded,** as the ORDER says. It is `relu(m − d)` and `weights.gt(0)`: the integer bit distance `d` from the recorded codes, against each head's margin `m`, a checkpoint constant.
  - **For I-Complete it is classed as derived:** a kept position is weighted exactly when `d < m`. So one per-head check of the exported threshold against F's `m` covers every step. It is not a sixth kind.
- **Three refusals have no test of their own.** Two are by `--decision-trace`: a split named twice, and a split no panel holds. The third is tracing with a read R does not serve (`unserved_reads`). All three are argument checks in `score_rows`, and the smoke ran none of them.
- **The suites carry failures this unit did not add:**
  - **Clippy with `-D warnings`** fails on 7 lints in the float crate and 52 in the integer crate. All of them are in code this unit does not touch (for example `decode_speed`'s 10 arguments, `reply_parts`'s type, `codec.rs`). None is on a line this unit adds.
  - **`stack_chat_reference`,** one of the integer crate's integration tests, does not compile without `serving-wasm`, because `stack` is gated (`native_geometry.rs`). The lib and bins pass under both feature sets.
- **The disk was under the 30 GiB floor (22 GiB free).** This unit's one build folder is 3.0 GB. No DeepSeek worktree was left in `/private/tmp` to remove.
- **The ladder gave a 2 × 5090 pod at $2.157/h in both rounds,** and one GPU was used. Each pod ran about 16 minutes: about $0.57, then about $0.59.

## What worked

- **F's trace records the card's five kinds in R's order,** at rows 0 to each traced window's last scored offset, in `uor-r4.decision-trace/1` with side `F`. Every float is stored as `to_bits()`, so values round-trip exactly.
  - **Rate buckets:** each layer's 64 lanes, with the score as `rate_select` reads it (`score_f32`) and its entry, `min(⌊s·16⌋, 15)`.
  - **Reads:** the stream read after the layer it follows (layer 2), then the readout read. Each head carries its query and key words, the f32 coordinates their signs read before the sign (`query_f32`, `key_f32`), and its kept set, nearest first by bit distance, ties to the earlier position.
  - **Memory:** each row's lookup, with its key, its stored token and whether it opens the reply, from the turn memory's own tables (`MemoryLookup`).
  - **Nearest:** at each scored offset, the GPU's pick and runner-up from `nearest_scores` (ties to the smaller id). Both scores are in R's units, f64 `−2^12·Σ|q − e|`, and F's f32 query is recorded too, so any token can be scored.
- **R's files are unchanged.** R's trace on claude-249's smoke still hashes `11e6b7d9…` and its rows `74c22378…`. An R step re-serializes byte for byte, and a float step writes no `_q12` or `_q16` field (the snapshot test).
- **On the GPU, at `8dafbe2cc`,** the crate's CUDA library suite passed: 272 tests, 0 failed, 6 ignored, as it had at `928465dad`. Among them, `a_traced_forward_records_the_kernels_choices_in_rs_order` checks a small trunk with both reads:
  - tracing changes no pick;
  - the traced window has its order and lengths;
  - each recorded entry is the kernel's bucket (`rate_select` returns that entry's λ), and the entries span more than one bucket;
  - each code is `pack_signs` of its coordinates, and some coordinate is not ±1, so they are the values before the sign;
  - each kept set, for every head of both reads, is the 16 positions `1 ≤ s < t` nearest its query word by the recorded key words, ties to the earlier. So the recorded words are the ones the kernel chose by;
  - each pick is the kernel's argmax for its recorded query.
- **I-F, the smoke outside Q5's set, passes in both rounds.** It ran F′'s command at seed 1 on the sealed step-16,384 checkpoint, tracing DEV s1's 308 windows of R's smoke. Round 2 ran it at `8dafbe2cc`, and every line below held again.
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
- **`crates/uor-r4-float-train/src/flock_read_hamming.rs`:** `set_hamming_trace_coords` is a second switch. With it on, the read keeps its query and key before the sign, and the head trace records them beside their codes.
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

## Measurements

The I-F smoke ran F′'s command at seed 1 on the sealed step-16,384 checkpoint, with `--samples 1`, on one RTX 5090. Round 2's binary was `train-in-space` at `8dafbe2cc`, sha256 `0345a57c…`; round 1's was at `928465dad`, sha256 `38791d8d…`. Round 2's values:

| What | Compared | Result |
|---|---|---|
| I-F: on = off | `cmp`, the six rows files, trace on against off | 6 equal |
| I-F: = F′ sealed | `cmp` against claude-243's sealed s1 rows | 6 equal |
| I-F: picks | each traced pick against `rows-dev-memory.json`'s `nearest` at its position (the window's position − `reply_start` + its row) | 10,840 of 10,840 |
| I-F: runner-up | each runner-up against its pick | 10,840 of 10,840 differ |
| I-F: opening table | the rebuilt table's hash, both runs | `654fc762…`, F′'s sealed hash |
| Coordinates | every recorded `query_f32` and `key_f32` value, on the pod | 37,417,984 values, none ±1 |
| Alignment with R | position, `reply_start`, ids prefix, nearest and memory steps, against R's smoke trace | 308 of 308 windows |
| R unchanged | claude-249's smoke at `928465dad`; the integer crate is unchanged since | trace `11e6b7d9…`, rows `74c22378…` |
| Trace size | DEV s1, R's 308 windows | 36,541 steps; 621 MB JSON (`9e7c9bf3…`), 208 MB gzipped (`b26b5522…`) |
| Wall time | the two runs side by side on one GPU | traced 323 s, untraced 322 s |
| Pod | 2 × RTX 5090 at $2.157/h, one GPU used | about 16 minutes, about $0.59 |

Round 1's values were the same, except its trace (85 MB gzipped, since its coordinates were ±1).

The evidence is sealed in the HF store:
- **Round 2:** `results/claude/integer-nearest-readout-trace-f-2026-11-02/r2/`, 46 files, read back and checked against the laptop manifest (sha256 `d736669e…`). It holds the pod's sums and logs, the traced and untraced rows, the trace summary (`27c0a80c…`), the coordinate count, m6's output, the positions, the job, the host gates' patches and logs, and where the full trace is archived.
- **The full round-2 trace:** `vast-archive/vast-58739274/uor-r4/runs/claude-251-r2/on/r-drop-s1/decision-trace-dev.json.gz`.
- **Round 1:** `results/claude/integer-nearest-readout-trace-f-2026-11-02/smoke/` (23 files; manifest `a1eb4270…`).

## Checks run at the head

| Gate | Command | Result |
|---|---|---|
| Host tests (float) | `cargo test -p uor-r4-float-train --lib -- a_traced_window_records each_rows_lookup a_head_trace` | 4 passed (the `a_head_trace` filter matches two tests) |
| Snapshot and integer lib | `cargo test -p uor-r4-integer --lib`; with `--features serving-wasm --lib --bins` | 61 passed; 413 passed, 1 ignored, bins pass |
| CUDA type-check | `CUDARC_CUDA_VERSION=12080 NVCC=<stub> UOR_NVCC_ARCH=compute_120 cargo check --offline --locked --features cuda --all-targets -p uor-r4-float-train` | exit 0 |
| Clippy, lines this unit adds | `cargo clippy --no-deps --features cuda --all-targets -- -D warnings` (float); `--features serving-wasm --lib --bins` (integer) | 0 findings on added lines; 7 and 52 in code this unit does not touch |
| CUDA library suite (RTX 5090, `8dafbe2cc`) | `cargo test --release --locked -p uor-r4-float-train --features cuda --lib -- --test-threads=1` | 272 passed, 0 failed, 6 ignored (also at `928465dad`) |
| The CPU crate merged from main | `cargo check -p uor-r4-cpu-train --all-targets`, after the merge | exit 0 |

**Which head each check ran at.**
- The host tests, the CUDA type-check and the float clippy ran at `1e22fc1a1`, this head's code.
- The GPU suite and I-F ran at `8dafbe2cc`. `1e22fc1a1` only clears five lints that `8dafbe2cc` added:
  - two type aliases in `flock_read_hamming.rs`;
  - clippy's shortest literal for the same f64 score in a host test;
  - one tuple statement in the GPU test split into its two calls, in the same order.
- The integer crate is unchanged since the merge of main (`a571cd3cb`). The merge touches `stack/session.rs` beside this unit's three lines, and the integer suite above was rerun after it.

### Negative gates

Each mutation was applied alone and its test run, then the file was restored and checked against the head. Every test failed at an assertion, not a build.
- **Round 1, at `928465dad`:** each mutation was applied by hand and restored with `cmp`.
- **Round 2, at `8dafbe2cc`, after the review:** each mutation was saved as a patch against the head and reversed with `git apply -R` (`local/claude/claude-251/neg2/gates.sh`, its logs beside it). The head was then rebuilt green.

| Round | Piece set aside | Mutation | Test | Result |
|---|---|---|---|---|
| 1 | Kept sets in R's order | drop `kept.sort_by_key` in `kept_in_order` | `a_traced_window_records_the_five_kinds_in_rs_order` | FAILED (exit 101) |
| 1 | Runner-up, ties to the smaller id | `s > b` to `s >= b` in `runner_up` | the same | FAILED |
| 1 | The opening flag | `opening: true` to `false` in `lookups` | `each_rows_lookup_is_the_key_and_entry_the_move_reads` | FAILED |
| 1 | R's bytes kept | drop `score_q16`'s skip when empty | `an_r_step_writes_as_before_and_a_float_step_keeps_its_bits` | FAILED |
| 2 | m1, the entry's clamp | `rate_index` without `.min(RATE_ENTRIES - 1)` | `a_traced_window_records_the_five_kinds_in_rs_order` | FAILED at the entry (`in_space_long.rs:9640`) |
| 2 | m2, the f64 sum | `l1_score_r` summing in f32 | the same | FAILED at the literal score (`:9662`) |
| 2 | m3, kept ties to the earlier | the tie key `Reverse(s)` in `kept_in_order` | the same | FAILED at the tied kept set (`:9656`) |
| 2 | m4, the coordinates, not the codes | `trace_head` writing the codes' bits | `a_head_trace_with_coordinates_keeps_them_bit_for_bit` | FAILED (`flock_read_hamming.rs:1059`) |
| 2 | m5, the sign check | `trace_head` without its word check | the same | FAILED at the refusal (`:1068`) |
| 2 | m6, the read's coordinates before the sign | the read recording its ±1 codes as the coordinates | `a_traced_forward_records_the_kernels_choices_in_rs_order`, on the GPU | FAILED at "the trace records the coordinates, not the codes" (`in_space_long.rs:11566`, exit 101); the source was then restored and checked clean on the pod |

Round 1's coordinate gate (`if !coords` to `if true`) no longer applies to the code; m4 and m5 replace it. m2 was rerun at `1e22fc1a1`, whose literal is written shorter, and failed at the same assertion (`:9662`).

**What the GPU test pins, and what it does not.**
- The entry rule: each recorded entry is checked against the kernel's own `rate_select`.
- The coordinates: m6.
- The kept sets against the recorded words.
- The picks against the kernel's argmax.
- **Not pinned by any test: that `score_f32` is the very tensor the forward's `rate_select` read.** It rests on where the copy sits, `geometric_stack_native.rs`'s rate step, beside the call. I-F's equalities do not pin it, since a misplaced copy changes no row.

## Skills used

None.

## Still open

- **The next unit: T's side,** under the same card. It is T's rounded copy beside `--rebind-fbs-backward`, writing the same schema, with its own smoke against R's 308 windows.
  - **Write the pod job under `/workspace/uor-r4/runs/`,** so `down --to-store` archives it, or seal its folder and read it back before `down`.
- **Then the replay driver and the near-tie grader, then the sheet** with the card's rule verbatim, a fresh FIT and a pre-run review.
- **The reviews' LOW items, not fixed here:**
  - **Overlapping rows.** `TurnMemory::lookups` lets a reply row replace an opening row of the same index, where `apply_readout` refuses the overlap. A successful traced forward cannot reach it, because the forward fails first.
  - **A switch left on after an error.** The decision capture, the coordinates and the rate trace are switched off by plain calls, not a guard. If a traced forward returns an error, they stay on for the thread. In `train-in-space` the error ends the process.
  - **Exact ties in the pick.** The recorded pick is the GPU's own argmax, so it is always the row's served pick. No test checks which id CUDA's argmax returns on an exact tie, while R breaks ties to the smaller id. The near-tie grader treats an exact tie as within G₁₀, so it cannot read one as a fault.
  - **The last scored offset is taken as the last, not the largest.** `assemble` builds the offsets in ascending order. An unsorted window would drop its later rows silently.
  - **The unserved-read refusal comes after setup.** It comes after `--out` is made, so a refused run leaves an empty folder. `ArmGuard` still resets the arms.
- **Carried from claude-249:** the open items of its "Still open", unchanged.

## Plain summary

To find out why the integer model and the trained model sometimes pick different words, claude-247's rule replays each disagreement and records every discrete choice both sides make. claude-249 built the integer model's half. This unit builds the trained model's half: the same record, written from the trained model's own run, with the exact numbers the choices were made from.

Recording changes nothing the trained model outputs. On 308 test windows, its results with the record on match those with it off and those sealed earlier, file for file. Every recorded word matches the word sealed then. The rounded copy that sits between the two models comes next.
