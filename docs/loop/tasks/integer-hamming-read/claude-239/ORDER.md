# ORDER: integer-hamming-read, claude-239

Lab: claude. Item: [integer-hamming-read](../../../../roadmap/items/integer-hamming-read.md). Base: `fe1619192`.

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

claude-238 (#3478, `fe1619192`) put the read's float forward on its trained path. It then found the last gap on (r-drop) to be a tie rule, not arithmetic. With one tie rule on both sides, d is 6 and 27 of 1,536, inside the bar of 46. With the GPU arg-sort's own order, d is 174 and 243. The item's step 5 names the next step: the trainer and every float reference take the integer session's tie rule by default, with the two tie-switch CUDA tests that claude-235 owed.

**The change, in `flock_read_hamming.rs` and `fbs-snapshot.rs` only.**
- **`TIE` defaults to on.** Ties then break by ascending bit distance, then ascending `s`, on every thread unless `set_hamming_tie_rule(false)` turns them off.
  - The read's codes are ±1 forward, `p + (bits − p).detach()`, so `round(mean|q − k| · width / 2)` removes only the codes' last-bit noise. Only the order among equal distances changes.
  - The selection is detached, so no gradient flows through the choice itself. *Amended after review:* a different kept set still changes the forward and its gradients wherever a tie crosses the 16th place.
  - `tie_keys_exact` already refuses a head whose keys would pass 2^24.
- **The docs** of `set_hamming_tie_rule`, `choose_sorted` and `fbs-snapshot`'s trace arms say the new default. `d11` and `d11-rounded` are now the explicit off arms, the sort's own order, kept so a later trace can still measure the gap.
- **The CPU test** `the_trace_and_the_tie_rule_start_off` becomes "the trace starts off and the tie rule on".
- **CUDA test A (new):** a real `read()` with planted ties and the trace on, at the default. Every traced kept set must equal the 16 nearest by (bits, `s`), computed on the host from the traced codes. It also runs with the rule off, to show that the plant ties across the 16th place and that the sort's own order breaks some of those ties against the rule. Without that, A could pass on a read with no ties to break.
- **CUDA test B** (`the_tie_rule_keeps_the_integer_sessions_order`, existing) gets its first set-aside run. Its codes get last-bit noise, so the rule runs on noisy codes. *Amended after review:* at this noise no key moves by ½, so the test does not pin `.round()`.

## Cancel probe (run first)

- **Does an open PR touch these files?** No open PR changes `flock_read_hamming.rs`, `fbs-snapshot.rs` or `fbs_snapshot.rs` (`gh pr list --state open`, 1 Nov).
- **Is the default already on?** No. Main at this ORDER's base `fe1619192` has `static TIE … Cell::new(false)` at `flock_read_hamming.rs:74`. Only `fbs-snapshot`'s `--hamming-trace` form calls `set_hamming_tie_rule`.

Class: **instrument** (the float reference's tie order). The unit goes ahead.

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

1. **The default is on.** The CPU test says so. The CPU suite and `loop gates --full` are green, and the CUDA code type-checks with the stub nvcc.
2. **Both CUDA tests pass on one pod**, along with the read's and the exporter's other CUDA tests. Each fails with its own lines set aside, shown on the same pod:
   - A fails with the read's `tie` wired to `false`;
   - B fails with `choose_sorted`'s tie branch removed.

   These are finite kernel tests with no data and no bar, so they need no prep sheet (CHARTER §4.5).
3. **The write-up:** a REPORT, a notebook row, and the item's step 5 updated with the next step: done-when 2's parity on claude-146's checkpoints and DEV, with the default on.

## Stop and ask if

This is stop and decide, never stop and wait.
- **A passes with the tie wired off:** the plant has no tie across the 16th place, or the GPU sort breaks those ties as the rule does. Either way that is an instrument fault in the test. Change the plant until the off run disagrees with the rule; do not loosen the check.
- **B's noise moves a code past rounding:** the noise is too large, and that is an instrument fault in the test. Keep it to the last bit.
- **Another test in the crate depends on the old default:** set the rule explicitly in that test, and name it in the REPORT. Do not turn the default back off.
