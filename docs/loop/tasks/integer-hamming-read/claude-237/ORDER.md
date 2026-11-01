# ORDER: integer-hamming-read, claude-237

Lab: claude. Item: [integer-hamming-read](../../../../roadmap/items/integer-hamming-read.md). Base: `62e3cfed4`.

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

The item's step 5 names the next check (claude-235, row F). Two facts frame it:
- **The two sides' copy-head codes already differ at the first read.** On by design, those codes depend only on the token. Yet 89–94% of d2's copy query codes differ between the float D11 forward and the integer session, and the two sides' codes sit 2.2–3.2 bits apart on average.
- **The trace ruled some causes out.** The rounded reads change no code. Each side's kept sets are the nearest by its own codes.

This unit builds the laptop host check that names the stage where the codes part, and runs it. It compares every vocabulary entry's copy-head code, at both depths, three ways:
- **(a) the CPU float control:** `hamilton_gate_fwd_cpu`, then `pack_signs`, on the artifact's own key rotation and Q12 writer table;
- **(b) the integer codes the session serves:** `CopyHeads` on the same artifact;
- **(c) the sealed traces:** the GPU float trace's codes, and the integer trace's, at each traced position, matched by its token.

(a) against (b) isolates the arithmetic, since the inputs are identical. (a) against (c) isolates the GPU forward's inputs. (b) against the integer trace checks the record.

## Cancel probe (run first)

- **Does any open PR already touch the copy codes?** No: no open PR changes `stack/hamming.rs`, `flock_read_hamming.rs` or `kernel_controls.rs`.
- **Has the gap closed on main?** No. Main is at this ORDER's base `62e3cfed4`. The item's step 5 still names this check as next and unclaimed.

Class: **instrument** (the probe asks whether the check is still needed). It is, so the unit goes ahead.

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

1. **`fbs-toks ARTIFACT --copy-codes FLOAT_TRACE INTEGER_TRACE OUT_JSON`** is a CPU-only host check, run with no GPU, that compares the three ways at every read depth.
   - **Where it lives.** It lives in the integer crate's serving build, not in `fbs-snapshot`. Under D29 the float trainer depends on the integer crate with no features, so it reaches only the native modules. The served `CopyHeads` sit behind `serving-wasm`. (Set 1 Nov, after the first builder found this.)
     - Part (a) is an f32 port of `hamilton_gate_fwd_cpu` and `pack_signs` in the native `fbs_snapshot`.
     - A float-train test pins that port bit for bit to the trainer's own.
   - It reports, per depth:
     - the codes that agree and the bits that differ, for (a) against (b);
     - how far from 0 the float value sits at each differing bit;
     - each trace's agreement with (a) and (b), positions matched by token;
     - each trace's own consistency. A copy code depends only on the token, so the check counts the tokens that carry more than one code across a trace's positions;
     - each trace's agreement with (a′), which is (a) with the key rotation first snapped lane by lane to the nearest of the 120 icosians. That is the snap the GPU forward's D11 mode applies to every Hamilton gate. (Added 1 Nov, after the first run found that (a′) reproduces the float trace. A float-train test pins the 120 roots, and a CUDA-only test pins the snap to the GPU's own.)
   - It refuses a trace whose ids or depths do not fit the artifact.
   - A test plants a known difference and must find it. With the comparison set aside, the test must fail.
2. **It runs on claude-233's h1 and h3 artifacts** against claude-235's sealed d11 and d11-rounded traces. Its outputs and their sums are kept in the unit's folder.
3. **The REPORT names the stage where the codes part,** with the evidence:
   - the inputs the GPU forward reads;
   - the float arithmetic;
   - the integer arithmetic;
   - the record.

   If the fix is small and inside this scope, it follows here, and its effect on the integer trace is measured against the sealed float traces on the laptop. Otherwise the next step is written down.
4. **One notebook row, the REPORT with negatives first, and the item's step updated.**

## Stop and ask if

Each of these is a stop and decide, never a wait:
- **The stage is the float trainer or the GPU forward's own inputs.** Then the fix changes the float model or the export, outside this scope. The lab decides by a decision review and writes the next step. A GPU re-run needs a prep sheet and its FIT.
- **The check needs a GPU or a file not on the laptop.** Then it stops, and says which.
- **(a), (b) and (c) all agree.** Then the trace's 89–94% is an instrument fault in the record or the comparison. That is classed instrument and repaired first.
