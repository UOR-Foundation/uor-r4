# ORDER: integer-hamming-read, claude-238

Lab: claude. Item: [integer-hamming-read](../../../../roadmap/items/integer-hamming-read.md). Base: `ebe0930ba`.

## Goal

The item's done-when reads:

> 1. The integer runtime runs the whole read as claude-146's REPORT describes it, all four heads of both reads, under D11's rules (CHARTER §3.6: integer only, no float or transcendental instruction; since D80 and D81 the hardware integer multiply and divide are allowed once [`integer-mul-div-serving`](integer-mul-div-serving.md)'s audits allow them, and until then products go through D11's tables):
>    - **copy heads:** keys and queries from the table's points through the key rotation. These depend only on the token, so their codes can be built once per vocabulary entry;
>    - **type heads:** queries from the residual through the query rotation, and payloads from the residual at `s` through the value rotation. These rotate the state at run time, so each product goes through D11's tables;
>    - the codes packed into 64-bit words; the match by `popcount(q XOR k)`; the 16 nearest earlier positions `1 ≤ s < t`; the weights `max(0, m − d)/m`; the stream read after layer 2 and the move on the readout, each product through the tables.
> 2. **Parity,** against the float forward of the same stored checkpoint (claude-146's language runs, `caseyallard/uor-r4-store:results/claude/hamming-read-2026-10-27/`), on DEV. No D11 tolerance exists for this read yet, so the unit's prep sheet pre-registers the bar for two numbers, and the REPORT gives both:
>    - the share of (position, head, read) kept sets equal to float's. One sign flip in an integer rotation changes a code, so this is not assumed to be exact;
>    - top-1 agreement between the integer and float decoded ids.
>    
>    The float reference is computed on the GPU (CHARTER §3.5); the integer side runs on the laptop, as serving does.
> 3. **Speed on this laptop:** `fbs-toks` decode tokens/s against context, with the read and without, on the same artifact.
> 4. One notebook row; the result on main.

## This unit

claude-237 (#3474) found the stage: in D11 mode, the CUDA gate hook snaps the read's rotations to icosians. Training (`HamiltonActive`) and the artifact (`lut-stack/5`'s raw gates) do not snap them. Its decision review passed (A): keep the read out of the hook in the float forward. The owner log is #3385 6554327155. This unit makes that fix, re-traces on the GPU, and measures the parity against a reference that computes the read as trained.

**The fix, in `flock_read_hamming.rs` only.** The read's gate calls are every `rotate` (key, query, value, step) and the step path's direct `hamilton_gate`. They run inside `hamilton_sheet::with_d11_snaps(D11Snaps { gates: false, transition: is_snapping_transition() }, …)`. That helper saves and restores the whole hook state: arm, call index, export flag, transition snap and telemetry flag. claude-237's item list left out the telemetry flag.
- **Inside the guard,** the arm is `HamiltonActive`, the training path's pass-through, and export is off, so `FbsReadoutMode::Export` stops snapping the read too.
- **After the guard,** the trunk's later gates snap as before, and their call numbering is unchanged.
- **Outside D11,** nothing changes. Only `in_space_long.rs` (under `HamiltonActive`) and `fbs-snapshot.rs` install a read, so the control arms, telemetry capture and D11-bridge training never meet one.

## Cancel probe (run first)

- **Does an open PR touch these files?** No open PR changes `flock_read_hamming.rs`, `hamilton_sheet.rs` or `geometric_stack_native*.rs` (`gh pr list`, 1 Nov).
- **Has the gap closed on main?** No. Main is at this ORDER's base `ebe0930ba`, claude-237's merge, and the item's step 5 names this fix as next.

Class: **instrument** (the reference's arithmetic). The unit goes ahead.

## Files in scope

- `crates/uor-r4-integer/src/stack/hamming.rs`
- `crates/uor-r4-integer/src/stack/mod.rs`
- `crates/uor-r4-integer/src/stack/session.rs`
- `crates/uor-r4-integer/src/stack/format.rs`
- `crates/uor-r4-integer/src/fbs_snapshot.rs`
- `crates/uor-r4-integer/src/stack/synthetic.rs`
- `crates/uor-r4-integer/src/stack/tests.rs`
- `crates/uor-r4-integer/src/bin/fbs-toks.rs`
- `crates/uor-r4-integer/src/bin/fbs-export.rs`
- `crates/uor-r4-float-train/src/fbs_export.rs`
- `crates/uor-r4-float-train/src/bin/fbs-snapshot.rs`
- `crates/uor-r4-float-train/src/flock_read_hamming.rs`
- `docs/harness/prep/integer-hamming-read*`
- `docs/loop/tasks/integer-hamming-read/**`
- `docs/research/notebook/m4-serving.md`
- `docs/roadmap/items/integer-hamming-read.md`

## Done when

1. **The guard is in,** with CUDA tests that fail with it set aside (shown on the pod):
   - (T1) a read with non-identity rotations computes the same output bit for bit under `with_d11_mode`, under `with_d11_export_gates` and under `with_gate_mode(HamiltonActive, …)`;
   - (T2) under `with_d11_mode`, a gate after the read is still snapped, and the snap capture holds the trunk's gates only.

   The CPU suite and `loop gates --full` are green. The CUDA code type-checks with the stub nvcc.
2. **A prep sheet** with its FIT and a pre-run review, then one pod. It runs:
   - the CUDA tests, T1 and T2 with the guard set aside as well;
   - claude-235's d11 and d11-rounded arms at s1 and s3, and their `fbs-compare` against claude-233's files;
   - a trunk-only readout, which must equal claude-233's d of 5 and 11.
3. **On the laptop:**
   - `fbs-toks --copy-codes` on the new traces: the float trace must equal (a) at every code (an instrument check);
   - `--hamming-trace-compare` against claude-235's integer traces;
   - each seed's d against the bar of 46, with the kept-set share and top-1 against the bar the sheet registers (the item's done-when 2, on (r-drop)).
4. **The write-up:** a REPORT, a notebook row, and the item's step 5 updated.

## Stop and ask if

This is stop and decide, never stop and wait.
- **T1 cannot hold bit for bit:** decide by a decision review whether the pass-through differs from training's path. Do not loosen the test.
- **The trunk-only d moves:** the guard leaks into the trunk. That is an instrument fault, and the run counts nothing.
- **The new float trace differs from (a):** a second stage remains. Classify it, then trace it; do not call it parity.
