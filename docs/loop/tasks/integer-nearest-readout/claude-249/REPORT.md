# REPORT: integer-nearest-readout, claude-249 (Q5's trace instrument, the integer side R)

Lab: claude. Item: [integer-nearest-readout](../../../../roadmap/items/integer-nearest-readout.md) (#3462). Order: [ORDER.md](ORDER.md). Rule: claude-247's [decision card](../claude-247/DECISION.md). References #3462.

## Tokens per second

No new decode speed is claimed, and the served path is unchanged (see the checks). R's decode speed is still claude-243's: 1,204 tok/s median decode on R s1, on this M1.
- A teacher-forced window on R took 46.7 ms with the trace on and 45.8 ms with it off (308 DEV s1 windows, prefill included, one run each). The trace adds about 2% per window.
- So one pass over Q5's 169 windows is seconds on R, not the 0.16 s per pass times replays that the ORDER planned around. That is an instrument's rate, not a serving reading.

## What did not work, first

- **Q5 is not read, and nothing here moves done-when 3.** This unit builds R's side of the trace only. F's and T's sides, T's rounded copy, the replay driver, the near-tie grader and the sheet come next, in that order.
- **The first refusal of a snapped artifact missed the gates.** It checked the session's `snapped`, which is the transition's icosian snap (`transport_snap`) alone. A Hamilton gate snapped to the 120 icosians is a nearest-icosian choice, a decision the card does not list. The check now refuses unless both `raw_gates` and `raw_transition` hold. The fix came before the first commit, and `the_decision_trace_refuses_what_is_off_rs_path` pins all three snapped exports.
- **The timed step ignored the trace.** Slice 2's review found it: `measure_fbs_step_stages` and `measure_fbs_step_parts` step through `stack_step_fbs_timed`, which neither forces nor records. A traced session stepped that way would apply no forcing, push no step, and leave every later step out of line with its window. The timed step now refuses a traced session (gate H).
- **The WASM call-graph audit fails both step roots, at the base and at this head alike.** `scripts/audit_zero_matmul_wasm.py --serving --call-graph --tap` reports `step` and `IntegerStackSession::step` with unclassified instructions on both sides, and no float on either (the table under the checks). The trace adds 17 multiplies, 2 divides and about 57 unclassified instructions to each root's call graph. The native stack audit, the crate's serving gate, passes in full. This unit does not fix the WASM audit's findings, which predate it.
- **Two refusals have no test of their own:**
  - **A point key.** The synthetic point-key stack exports snapped, so the snap check refuses it first.
  - **A recurrence that is not geometric.** An FBS artifact cannot have one: the shape's own validation refuses it before a session exists.
  
  Both checks are in `set_decision_trace`; neither can fail on any artifact this crate can build.
- **The nearest is recorded only from the prompt's last step on.** Before it, the pick feeds nothing, because the next id is the prompt's. The schema says so (`nearest: None` at the prompt's earlier steps). The grader must treat those steps as having no nearest decision.
- **The disk was under the 30 GiB floor (23 GiB free) all unit.** Nothing was removed: the older `local/claude/` run folders may hold evidence, and they would not reach the floor. The smoke run's files take 69 MB.

## What worked

- **R's trace records the card's five kinds in the forward's order,** at every step of every window:
  - each layer's 64 rate buckets with the score `s_q16` each was chosen from;
  - the stream read after the layer it follows, then the readout read: each head's query code, key code and kept set;
  - the turn memory's lookup: its key, what it stored and whether it was the opening;
  - from the prompt's last step on, the nearest: the pick, the runner-up, both Q12 scores, and R's Q12 query, so a grader can score any token.
- **The trace and the forcing change nothing they do not force.** A traced session, and one forced to every decision it recorded, serve what an untraced one serves at every step. The forced one records the same trace (I-R's test, and I-Force's second part).
- **A forced decision takes its forced value and changes what follows,** for a rate bucket and for a query or key code of either kind of head. Each of the seven places that apply the forcing, set aside alone, fails its test (the negative gates below).
- **Rounding is not a decision, as the card reads it.** The card counts `stack_quantize16`'s choice of exponent, the clamps, the saturation and the rounding as arithmetic ("Everything else in the step is arithmetic"). So the trace records none of them; they are what the arithmetic step (T, R) compares.
- **On R s1, over DEV s1's 308 rows outside Q5's set, the forced rows equal claude-244's sealed `forced-dev-s1.json` rows exactly:** 308 of 308, with the trace on and with it off. Both runs write the same file. No row of the set (4763 and 10757) was run, so no trace value Q5 reads has been seen.
- **`--force` on R s1 reproduces its own decisions:** forcing three rows (positions 141, 304 and 482) to all 65,664 of their recorded buckets and 2,712 of their codes writes the same rows and the same trace.

## What changed

- **`crates/uor-r4-integer/src/fbs_snapshot.rs`:** the two shared schemas, beside `HammingTrace`, so the float side's unit writes the same records.
  - `uor-r4.decision-trace/1`: `DecisionTrace`, `DecisionWindow`, `DecisionStep`, `DecisionLayer`, `DecisionRead`, `DecisionHead`, `DecisionMemory` and `DecisionNearest`.
  - `uor-r4.decision-force/1`: `DecisionForce`, `DecisionForceWindow`, `ForcedBucket`, `ForcedCode` and `CodeRole`.
- **`crates/uor-r4-integer/src/stack/hamming.rs`:** the copy and type heads choose with forced codes (`choose_forced`).
  - A forced query at step t replaces row t's query. A forced key at step s replaces candidate s's key at every later row.
  - `ServedHamming::forcing` builds each read's forced codes. It refuses a depth no read runs at, a head the read does not serve, a step past the context, a key at step 0, and a code forced twice.
  - The served means go through the same `mean_by` with the unforced `choose`, so the untraced path is the one it was.
- **`crates/uor-r4-integer/src/stack/session.rs`:**
  - `set_decision_trace(&DecisionForceWindow)` and `take_decision_trace()`.
  - `stack_lanes` sends each geometric lane's bucket through `BucketTrace::decide` only while a traced step runs. The lanes then run on one thread, as the snap trace's do.
  - Refused: a model that is not FBS, a recurrence that is not geometric, a snapped gate or transition, a gate other than Hamilton's, a head other than the nearest, a point key, a session that has stepped, and a forced bucket outside the model's steps, layers, lanes or 16 entries, or one forced twice.
  - The timed step (`stack_step_fbs_timed`) refuses a traced session.
- **`crates/uor-r4-integer/src/bin/fbs-toks.rs`:** `--forced` takes `--decision-trace TRACE_JSON` and `--force FORCE_JSON`, each at most once.
  - The trace (side `R`) fills each reply offset's nearest from `whole_table_nearest_two`.
  - `--force` runs only the rows it names, by position, and refuses another schema, a position named twice, and one that names no prompt.
  - The rows keep claude-244's format. The printed line adds `per_row_us` and the trace's sha256, even without either option. Under `--force` its `rows` counts only the rows run.
- **Tests:** three in `stack/tests.rs` and two in `fbs-toks.rs`, below.
- **The item's "What is missing"** names the next three units.

## Measurements

| R s1 (`a7r-s1.uorlut`), DEV s1 outside Q5's set | Trace off | Trace on |
|---|---|---|
| Rows run (positions 4763 and 10757 left out) | 308 | 308 |
| Offsets read | 4,382 | 4,382 |
| Steps traced | — | 17,491 |
| Rows equal to the sealed `forced-dev-s1.json` | 308 of 308 | 308 of 308 |
| Rows file sha256 | `74c223782daa96a2b92dd9416103a533035756d2e2dc647d85384c80003c7ceb` | the same |
| Time per window (prefill included) | 45.8 ms | 46.7 ms |
| Trace file | — | 68,157,721 bytes, sha256 `11e6b7d9347fed0cbb390ba2e7b04b1bfe7031e946db87c05fd842d2421cca7d` |

- **R's shape, as the trace records it:** 6 layers, 64 lanes, 4 heads (2 copy, 2 type), at most 16 kept, a stream read at depth 2 and the readout read at depth 6.
- **Forced to its own decisions (positions 141, 304 and 482):** 65,664 buckets and 2,712 codes forced. The rows (sha256 `814979ee48557f9b28013b810aada3d6d8003a5880a0dd9903ab0ec84aab1c7f`) and the trace equal the unforced ones.
- **Where the files are:** `local/claude/claude-249/smoke/` in this unit's worktree. They are instrument checks, not a result, so they are not sealed and there is no notebook row.

## Checks run at the head

All at e5608dc84, clean tree, on this M1 (log: `local/claude/claude-249/gates-final.log` in the worktree).

- **The crate's tests:** `cargo test -p uor-r4-integer --features fbs-synthetic,serving-wasm --lib --bins --no-fail-fast`. The library has 412 passed and 1 ignored. Every binary's tests pass: fbs-toks 11, uor-chat 3, fbs-compare 1.
- **The served path's audit:** `scripts/audit_zero_matmul_serving.py --stack` on the release `uor-r4-stack` is a FULL PASS. It finds 0 floats, and 52 / 1 scalar multiplies and divides in the symbol ranges, 131 / 15 in the call graph, all allowed (D80, D81). It checked 129 call-graph functions, with 0 violations.
- **The WASM library builds** (`--target wasm32-unknown-unknown --features serving-wasm --lib`).
- **The WASM call-graph audit** (`scripts/audit_zero_matmul_wasm.py --serving --call-graph --tap`): five roots pass. Both step roots fail at the base (381ae4d98, built from a `git archive` of it) and at this head alike:

  | Root | Side | mul | div | float | unclassified | instructions | functions |
  |---|---|---|---|---|---|---|---|
  | `step` | base | 29 | 14 | 0 | 340 | 40,268 | 95 |
  | `step` | head | 46 | 16 | 0 | 397 | 44,733 | 97 |
  | `IntegerStackSession::step` | base | 29 | 14 | 0 | 334 | 39,175 | 82 |
  | `IntegerStackSession::step` | head | 46 | 16 | 0 | 391 | 43,640 | 84 |

- **Formatting:** `cargo fmt -p uor-r4-integer --check` passes.
- **Clippy** (`-p uor-r4-integer --all-targets --features serving-wasm -- -D warnings`) reports 68 findings in 21 files. Blame puts none of them in this unit's commits; the nearest are tests.rs and session.rs:736, from earlier units.
- **`uor-harness loop gates --full`:** 11 of 11 green.
- **The negative gates.** Each place below was set aside alone, against the tree it was taken from, and its test failed; each was then restored and the tests passed:
  - A: `BucketTrace::decide`'s forced entry. Failed at tests.rs:6142.
  - B, C, D: the forced query and the two forced keys in the copy and type heads' `choose_forced`. Each failed at tests.rs:6173, the kept-set check.
  - E: `type_mean`'s forced query. Failed at 6177, the served-change check.
  - F: the forced arm of the served copy read in `stream` and `readout`. Failed at 6177.
  - G: `record`'s forced query. Failed at 6168, the forced-code check.
  - H: the timed step's refusal of a traced session. Failed at 6210.
- **The three reviews.** Each slice had a fresh reviewer, and each returned APPROVE with LOW items only:
  - slice 1: `hamming.rs` and `tests.rs`;
  - slice 2: `session.rs` and `fbs_snapshot.rs`;
  - slice 3: `fbs-toks.rs`, this REPORT and the item.

  Slice 2's first item is fixed (H). The LOW items not fixed are under Still open.

## Skills used

None.

## Still open

- **The next unit: F's and T's sides of the trace.** They write the same `uor-r4.decision-trace/1` from the float trunk (`flock_read_hamming.rs`, `geometric_stack_native.rs`, `cuda_stack.cu`, `in_space_long.rs`, all in the item's scope).
  - T is F's path on parameters rounded as the export rounds them, a copy built beside `--rebind-fbs-backward`.
  - F and T also record their code coordinates, since they are the reference sides; R records each code's word only.
- **Then the replay driver and the near-tie grader:**
  - **The driver** forces the first differing code or bucket in forward order, step by step, structure (F, T) and arithmetic (T, R).
  - **The grader** judges each step's pick by G₁₀ on the reference side. A differing kept set or memory lookup fails the step.
- **Then the sheet,** with the card's rule verbatim, a fresh FIT and a pre-run review. Its Row 0 is I-F, I-R, I-Twin, I-Rerun, I-Force, I-Align, I-Complete, I-Set and I-Inputs.
- **Untested refusals:** a raw point-key artifact, if one is ever built, should get its own test.
- **The reviews' LOW items, not fixed here:**
  - The key test checks that a forced key changes a kept set and what R serves. It does not check each later row on its own.
  - No test forces a code at a depth inside the model that no read runs at. The refusal is in `ServedHamming::forcing`.
  - `BucketTrace::decide` is on the served step's call graph. It is not marked inline, and it grows a `Vec`. The native stack audit passes with it. If a later audit objects, write into preallocated slots, as `SnapTrace::record` does.
  - `fbs-toks` writes `--out` before the trace. If the trace's path already exists, or is `--out`'s, the command fails after writing the rows, and a retry then fails on `--out`. Checking both paths up front, before the run, closes it.
  - `--force` without `--decision-trace` still builds the trace and drops it. That costs time only.
- **Carried from earlier units:**
  - making raw the default, or the snap faithful;
  - a cheaper exact nearest, which is 65% of a decode step;
  - reading F′'s own top two;
  - the cut windows' openings;
  - claude-240's code minors and claude-241's Lows.

## Plain summary

To check why the integer model and the trained model sometimes pick different words, the rule fixed in claude-247 replays each disagreement while forcing one side's discrete choices to match the other's. This unit builds the integer model's half of that tool. It records every such choice the integer model makes at every step, and it can force any of them to a given value.

Recording changes nothing the model outputs, and forcing a choice to the value it already had changes nothing either. On the real artifact, 308 test rows came out identical to the sealed results from before. The trained model's half and the comparison come next.
