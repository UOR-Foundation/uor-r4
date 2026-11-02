# ORDER: integer-nearest-readout, claude-253 (Q5's forced replay: forcing on the float side)

Lab: claude. Item: [integer-nearest-readout](../../../../roadmap/items/integer-nearest-readout.md) (#3462). Base: `27e103bc5`. Rule: claude-247's [decision card](../claude-247/DECISION.md).

## Goal

Let the float forward run under a forced set Φ, as R's already does (claude-249, `fbs-toks --forced --force`). Q5's trace has three sides: F (claude-251), T (claude-252) and R (claude-249). The card's forced replay compares them in two steps:
- **The structure step** forces T to F's decisions.
- **The arithmetic step** forces R to T's decisions.

So the structure step needs forcing on a float side, and nothing yet provides it. Casey, 1 Nov: "Finish Q5 first" (#3462, `6564833770`). The replay driver, the near-tie grader, the sheet and the GPU run that reads Q5 are later units.

**What a forced set does (the card, `uor-r4.decision-force/1`, R's semantics):**
- **A forced bucket** `(step, layer, lane, entry)` replaces the entry that the lane's rate score chooses at that step. The score is recorded as computed, and the entry as used.
- **A forced code** `(step, depth, head, role, code)` replaces the head's query code at that step, or its key code (the code of `x_{step−1}`, which every later row reads for candidate `step`). The read at that depth then chooses its kept sets from the forced codes.
- **Not forced:** kept sets, memory lookups and the nearest. A difference in any of them is "not accounted for" (the card).

**The float side:**
- **The option:** `train-in-space --score-rows --decision-force SPLIT=FORCE_JSON`, beside `--decision-trace SPLIT=POSITIONS_JSON` for the same split. It forces the windows the file names, in the pass that traces them (the memory pass), in the same batches as an unforced pass.
- **Every forced position is a traced position.** The trace then records each forced decision as used.
- **R's refusals, checked before any forward:**
  - a step past the context;
  - a layer, lane or entry outside the model;
  - a key forced at step 0;
  - a depth with no read;
  - a head the read does not serve;
  - a decision forced twice;
  - a file of another schema;
  - a position that is not traced.
- **One refusal R has no need of:** a code with a bit set past the head's width.

## Cancel probe (run first)

- **Is there a point on the float forward where each forced kind can be applied?** Found, at the base:
  - **Buckets:** the clamped score `score` in `geometric_stack_native.rs:1476`. `RATE_TRACE` records it and `rate_select` reads it (`:1486`).
    - Overwriting a forced lane's score with `(entry + 0.5)/16` makes rate selection choose `entry`. The host's `rate_index` (`:2009`) and the kernel's `rate_index_of` (`cuda_stack.cu:254`) both take `min(⌊16·s⌋, 15)`.
    - `(k + 0.5)/16` is already the bucket centre the init uses (`:811`), and it is exact in bf16.
  - **Codes:** the ±1 `query` and `key` tensors in `flock_read_hamming.rs:672`, after `code()` and before the per-head loop (`:708`). Both `choose_sorted` and the distances read them.
- **Does a window's step map to one tensor row?** Row `b · SHEET_CONTEXT + step`, as `FTrace::record` reads it (`in_space_long.rs`, `let row = b * time + t`). The key at row `t` is `DecisionHead.key` at step `t` (`decision_read`).
- **Does the trace refuse a forced code as it stands?** Yes. `trace_head` refuses coordinates that do not sign to their code. A forced code must therefore be exempt at its own row only, and its coordinates are recorded as computed.
- **Does R record the computed score and the entry used?** Yes: `BucketTrace::decide` (`session.rs:1794`). The float trace must do the same. Today it derives the entry from the score (`rate_index(s)`), which would hide a force.
- **No resume on the float side.** Every float pass reads a whole window from its start, so the card's I-Force clause "a resumed pass equals a full pass" has no float case.
- **Classed:** instrument.

## Files in scope

The item's scope. This unit edits:
- `geometric_stack_native.rs`: the bucket force, and the entry used, in `RateScores`;
- `flock_read_hamming.rs`: the code force, and the trace's exemption at forced rows;
- `in_space_long.rs` and `bin/train-in-space.rs`: the option, the forced set's checks, and its rows per batch;
- this folder and the item.

## Done when

1. **Forcing applies on the float forward.**
   - Each forced bucket takes its entry, and each forced code its code, at its own row, read and head. The trace records them as used.
   - The forced set is set before the forward it applies to and cleared after it.
2. **I-Force, tests.** The vendored candle refuses tensor compute on the CPU (D24), so the forward's tests are CUDA tests, run on the pod; the host tests cover the rest.
   - **CUDA, on the sheet's model:**
     - **Self-force:** forcing a side to every one of its own buckets and codes leaves its trace and its rows byte-identical.
     - **One forced bucket**, set to another entry: it takes that entry and its score stays as computed. The forward downstream of it changes.
     - **One forced query code and one forced key code**, each set to another code: each takes its code. The kept sets that read it are chosen from it.
   - **Host:** each refusal above; each forced decision recorded as used, and a forced decision not taken refused; the rows a batch forces; the trace's sign check exempt at a forced row only.

   Each has a negative gate: the force's own lines set aside, and the test then fails. The crate passes:
   - its host suite;
   - a stub-nvcc `cargo check --features cuda --all-targets`;
   - clippy, with no finding on added lines.
3. **I-Force on the GPU, on a pod, outside Q5's set** (an instrument check, as claude-252's I-Twin was; no sheet):
   - on F's s1 checkpoint, DEV s1's 308 windows: the self-force run gives a trace and rows byte-identical to the unforced run;
   - a set of one forced bucket and one forced code takes its values.
   - The times are recorded. Results are saved off the pod before `down`.
4. **A fresh exact-head review passes,** in slices of at most 600 lines. The REPORT puts the negatives first. No notebook row: this is an instrument with no result.

## Stop and decide (never wait) if

- **A forced kind has no point on the CUDA path that matches the host's.** Decide whether to apply it in the kernel, and record the decision.
- **Self-forcing is not byte-identical on the GPU.** Classify it first (instrument). Find the term that moves before anything is read. Batch composition is the same by construction, so a difference there is a bug in the force.
