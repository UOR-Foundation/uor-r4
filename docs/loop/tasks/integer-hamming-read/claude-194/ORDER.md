# ORDER: integer-hamming-read, claude-194 (the copy heads served in the session)

Lab: claude. Item: [integer-hamming-read](../../../../roadmap/items/integer-hamming-read.md). Issue: #3385. Base: `c6e2e19bf`.

## Goal

This is the second half of step 2 of the item's design: claude-189's copy-head kernel (`CopyHeads`, `stack/hamming.rs`) put on the served path of the FBS session. It advances step 2 of the owner's order in PLACE.md: serve the working model's parts in integers.

The session follows the float trunk (`geometric_stack_native.rs:1138–1190`) and the float read (`flock_read_hamming.rs`, `read`) for a read whose type-head weights are all zero:
1. **The stream read.** After the layer whose count is a read's depth `δ` (`1 ≤ δ < layers`), the residual takes `x + w_c ⊙ c`, rounded to Q16 as the float's `q16(read.apply(...))`. `c` is the copy mean at the current row, at `COPY_MEAN_EXP`. A row where no copy head found a candidate adds nothing, as the float's `sums / max(found, 1)` is 0 there.
2. **The readout move.** For a read at depth `δ = layers`, on the final norm's output `n` (after the final gain, before the Q12 query): `n + w_c ⊙ (c − n)` where a copy head found a candidate, and `n` elsewhere. It is not rounded to Q16, as the float's line 1187 is not; it lands at the norm's own exponent.
3. **The tokens the read sees.** The read at row `t` needs `x_0..=x_t`. The session keeps each position's token before its layers run, so the read sees the current one.
4. **The refusals** (the design's rule: refuse what is not served, so a partial port never silently differs from float):
   - a Hamming read on a shape that is not FBS;
   - a read with any non-zero type-head weight (`w_y`, weight row 1): the type heads are step 3;
   - a read at depth 0, or past the layer count.

   claude-188's review asks that the tests match each error, not only `is_err()`.
5. **The timed step** (`stack_step_fbs_timed`) runs the same hooks, so it stays equal to the step.

## Cancel probe (run first)

- **The integer runtime already has a Hamming read on main?** No. At `origin/main` `c6e2e19bf`, `IntegerStackModel::parse` still refuses every `/6` artifact ("this engine does not serve a Hamming read yet (uor-r4.lut-stack/6)", `session.rs:672–678`).
- **The float read withdrawn?** No. `flock_read_hamming.rs` is on main, and claude-146's read is the copy part of PLACE.md's composition table.
- **Classed:** instrument. Nothing cancels the unit.

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
- `docs/harness/prep/integer-hamming-read*`
- `docs/loop/tasks/integer-hamming-read/**`
- `docs/research/notebook/m4-serving.md`
- `docs/roadmap/items/integer-hamming-read.md`

## Done when

All of these hold at the PR head, and the REPORT gives each one's evidence:
1. **Off at weight 0.** A `/6` FBS artifact whose copy weights are 0 serves every step bit for bit as the same trunk exported without a read: the native choice, its kept cells and its Q12 query, over a sequence with repeats, so the copy heads do find candidates.
2. **The hooks against a reference.** With non-zero copy weights, each hook matches a test-side reference that takes the read-free trunk's own integers at the hook and applies the formula above in i128, with its own rounding, at every step of a sequence with repeats:
   - **the stream read** (a read at depth 1 only): the residual after layer 1 equals the read-free residual there plus the rounded `w_c ⊙ c`. A layer before the read cannot see it, so the read-free run gives the read's input exactly. The residual is seen through a `#[cfg(test)]` trace of each layer's output, absent from every non-test build;
   - **the readout move** (a read at the layer count only): the final norm's output equals `n + w_c ⊙ (c − n)` on the read-free `n`, and the Q12 query is that converted;
   - each read changes the query at some step, so the test can fail, and a step where no copy head found a candidate leaves it unchanged.
3. **The refusals,** each matched by its error text: no read on a non-FBS shape, no non-zero type weight, no depth 0 or past the layer count.
4. **The timed step equals the step,** with a read, at every step.
5. **Negative gates:** each named hook, set aside, makes its test fail; restored, all pass.
6. **The serving audit** (`audit_zero_matmul_serving.py --stack` on the release `uor-r4-stack`) passes, and its reachable functions now include the copy read (claude-189's Low: its roots did not reach `CopyHeads::mean`).
7. `cargo test -p uor-r4-integer` shows no new failure against `main`; `uor-harness loop gates --full` is green.
8. One notebook row in `docs/research/notebook/m4-serving.md`. The item stays `ready`: steps 3 to 6 remain.

## Stop and ask if

Stop and decide (never wait) if:
- serving a read needs an artifact format change. Then the format change is its own unit, and this one serves what `/6` already stores;
- the float's rounding or depth semantics turn out not to be what lines 1138–1190 show. Then the float wins, and the ORDER is amended before the code;
- the audit cannot reach the read without editing `scripts/audit_zero_matmul_serving.py`, which is outside this scope. Then the REPORT records the gap, and a separate item takes it.
