# REPORT: integer-nearest-readout, claude-253 (Q5's forced replay: forcing on the float side)

Lab: claude. Item: [integer-nearest-readout](../../../../roadmap/items/integer-nearest-readout.md) (#3462). Order: [ORDER.md](ORDER.md). Rule: claude-247's [decision card](../claude-247/DECISION.md). References #3462.

## Tokens per second

No decode speed is claimed, and no served path changes. R's decode speed is still claude-243's: 1,204 tok/s median decode on R s1.

The pod run checks the instrument; it is not a speed result. One RTX 5090 (Vast), at `f6097ced9`, on F's s1 (`r-drop-s1`, step 16,384), each pass a whole `score_rows` run over DEV's 308 traced windows and both panels:

| Pass | Wall | Ran |
|---|---|---|
| Base (traced, nothing forced) | 204 s | alone |
| Self-force (every traced decision forced to itself) | 335 s | at the same time as one-force, on the same GPU |
| One-force (one bucket and one code at one step) | 324 s | at the same time as self-force, on the same GPU |

- Building both force files from the base trace with jq took 84 s.
- The CUDA library suite took 33.7 s at `f6097ced9` and 16.3 s at `9859c419f`.
- The two forced walls are for two passes sharing one GPU, so they measure no cost of forcing.

## What did not work, first

- **Q5 is not read, and nothing here moves done-when 3.** This unit lets the float side run under a forced set. The replay driver, the near-tie grader, the sheet and the GPU run that reads Q5 come next, in that order.
- **The forward's tests are CUDA tests, run on the pod, not host tests.** The ORDER's done-when 2 first said "the CPU forward". The vendored candle refuses tensor compute on the CPU (D24), so the ORDER was corrected before the work was committed:
  - the self-force, the one-bucket and the one-code cases are a CUDA test (`a_forced_forward_takes_each_forced_decision`);
  - the host tests cover the checks, the rows a batch forces, the record of each decision as used, and the trace's exemption.
- **Every float pass is a whole `score_rows` run.** It scores all of DEV and both panels, and its times are below. The card's replay runs single-window passes, so the replay driver needs a float mode that runs only the windows it names. That mode is the next unit's, not built here.
- **The float side has no resume.** Every float pass reads a window from its start. So the card's I-Force clause "a resumed pass equals a full pass" has no float case; it applies to R alone.
- **The one forced code is the head's own key code at that step,** not a code with one bit flipped. jq does no exact 64-bit arithmetic, so the job passes a recorded 64-bit literal through instead of computing one. The job checks that it differs from the query it replaces.
- **Review round 1 changed the code after the pod's main run.** The pod ran `f6097ced9`; the round's head is `9859c419f`.
  - **The step check moved before any forward** (review B). A forced step past its window's last scored offset, but inside the context, passed every check and was refused only after the forward. `forced_within` now refuses it where the forces are validated, together with a position that names no window of its split.
  - **A head wider than 64 coordinates is refused** before `force_codes` shifts a code by 64 or more (review A). The sheet's heads are 64 wide, so this cannot fire there.
  - **The host test forces a key to another code,** taken and not taken (review B).
  - **This unit's own lines were formatted.**
  - None of these lines changes a value the forward computes. The host suite, the CUDA type-check and clippy ran again at `9859c419f`, and so did the CUDA library suite, on the same pod: 277 passed, 0 failed, 6 ignored, one more than at `f6097ced9` (the head-width test). The I-Force run on F's s1 was not repeated.
- **`forced_within`'s call in `score_rows` has no test of its own.** It runs only on the GPU path. The function itself is pinned by the host test and gate G9, and the call is one line beside the other checks.
- **Two refusals inside the forward fire part-way through it:** a forced bucket's row or lane, and a forced code's head. `check_forced` refuses the same before any forward, so these are backstops for an instrument fault.
- **The suites carry failures this unit did not add.**
  - `cargo fmt --check` already fails on the crate's base. Only this unit's lines were formatted, so each file's drift is at or below its base's: `flock_read_hamming.rs` 23 against 26, `in_space_long.rs` 319 against 320, `bin/train-in-space.rs` 18 against 18, `geometric_stack_native.rs` 0.
  - Clippy finds 7 lints in the float crate, all in code this unit does not touch. None is on an added line.
- **The forced passes' times measure no cost of forcing.** The job ran self-force and one-force at the same time on one GPU, so their walls are not comparable to the base's.
- **The seal's first sums file listed itself,** with the sum of a file still being written. It was rebuilt without that line and put again; the store's copy reads back equal to the pod's.
- **No sheet, FIT or pre-run review was written for the pod run.** It checks this unit's own instrument (done-when 3), as claude-251's and claude-252's smokes did, and no claim rests on it.
- **The pod tool's provider adapter had to be rebuilt in this worktree,** as a debug build: `uor-pod` reads `local/pod-provider-target/debug/`, and a release build there is not found. Until then `status` hid Vast's pods from the caps count.

## What worked

- **Forcing on the float forward.** `train-in-space --score-rows … --decision-trace SPLIT=POSITIONS_JSON --decision-force SPLIT=FORCE_JSON` runs the split's traced windows under a `uor-r4.decision-force/1` set, in the pass that traces them (the memory pass), in the same batches as an unforced pass.
  - **A forced bucket** `(step, layer, lane, entry)` sets that lane's score, at its row, to the bucket's centre `(entry + 0.5)/16` before `rate_select` reads it (`geometric_stack_native.rs`, `force_rates`). The host's `rate_index` and the kernel's `rate_index_of` both take `min(⌊16s⌋, 15)`, so both choose `entry`. The centre is exact in bf16. The trace records the score as computed and the entry as used, as R's `BucketTrace::decide` does.
  - **A forced code** `(step, depth, head, role, code)` changes the head's query or key code at its row, after the ±1 codes are made and before the read uses them (`flock_read_hamming.rs`, `force_codes`). Only the coordinates whose bit differs are changed, each to the exact ±1. The rest keep their computed values, last-bit noise included, which the margin weights read. So forcing a side to its own codes is the identity. The read's kept sets are chosen from the forced codes. The trace records the forced codes, and its sign check is waived at a forced row, for that head and role only.
  - **Rows.** A traced window `b` of a batch forces its rows `b · SHEET_CONTEXT + step`, the layout `row_picks` and `record` use. The force is set before the batch's forward and cleared right after it, even if the forward fails.
- **The refusals, before any forward:** another schema; a position not traced, or named twice; a step past the context or past the window's traced steps; a layer, lane or entry outside the model; a depth with no read; a head the read does not serve; a key at step 0; a code with a bit past its head's width; a decision forced twice. After the forward, each forced decision must appear in the trace as forced, or the run is refused as an instrument fault.
- **The host tests and their negative gates.** At `9859c419f` the host suite passes (228 tests), as do the stub-nvcc `cargo check --features cuda --all-targets` and clippy. Each gate sets one piece aside and runs the test that pins it. All eleven fail as expected, each at its own assertion, and the restored tree passes (`local/claude/claude-253/neg/host.sh`):
  - G1, the not-taken refusal; G2, the trace's exemption; G3, the entry as used;
  - G4, `check_forced`'s refusals; G5, the batch rows;
  - G6–G8, the schema, an untraced position and a position named twice;
  - G9, a step past the window (round 1); G10, the key's not-taken check (round 1); G11, the head width (round 1).

- **I-Force on F's s1** (the pod run at `f6097ced9`, DEV seed 1's 308 traced windows):
  - **The base** traces 308 windows and 36,541 steps. Its trace's sha256 is `9e7c9bf312fa…`, the same as claude-251's.
  - **Self-force.** The force file sets every traced decision to itself: 308 windows, 14,031,744 buckets and 582,192 codes. The forced pass's trace and all six rows files equal the base's, byte for byte.
  - **One force,** at position 41, step 49:
    - bucket layer 0, lane 0 is forced to entry 9 (the base took 1);
    - depth 2, head 0's query is forced to the head's own key code at that step.
    - The trace records both as taken, the score as computed, and a base code that differs from the forced one.
    - Steps 0–48 of that window are unchanged, all 49 later steps change, and the other 307 windows are equal.
    - Of the six rows files only `rows-dev-memory.json` differs: the no-memory pass and the panels are not forced.
- **The CUDA gates.** Each set-aside makes `a_forced_forward_takes_each_forced_decision` fail at its not-taken check, and the restored tree passes:
  - n1 sets the bucket force aside: "the bucket forced at step 12, layer 0, lane 0 was not taken";
  - n2 sets the code force aside: "the code forced at step 23, depth 2, head 0 was not taken".
- **Saved** in the store under `results/claude/integer-nearest-readout-force-f-2026-11-02/`:
  - `run/`: the pod's run folder, without its inputs (fetched from the store) or its binary (`binary.sha256` names it). Its own `SHA256SUMS` lists 103 files. The job read both gzipped traces back to their listed sums. The self trace was removed on the pod, and its listed sum, equal to the base's, stands for it.
  - `r1/`: round 1's CUDA suite.
  - `SEAL-SHA256SUMS` (`8b666c70…`) covers the 132 files put. The 54 small files are on the laptop under `local/claude/claude-253/pulled/` and match the seal.
  - The pod was up 30 minutes, about $0.50, and is down.
