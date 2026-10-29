# ORDER: integer-hamming-read, claude-195 (the type heads served)

Lab: claude. Item: [integer-hamming-read](../../../../roadmap/items/integer-hamming-read.md). Issue: #3385. Base: `9bf364855`.

## Goal

Step 3 of the item's design: the Hamming read's type heads, served in the FBS session beside claude-194's copy heads, so that a `/6` read with non-zero type weights is served rather than refused. It advances step 2 of PLACE.md's order (serve the working model's parts in integers). claude-146's checkpoints and claude-194's run train `--hamming-read 4`, two copy heads and two type heads, so parity (step 5) needs them.

The float read (`flock_read_hamming.rs`, `read`), for heads `h ≥ heads / 2`, at row `t`, with `p` the hook's input (the residual before the stream read; the final norm's output before the readout move):
1. **Query:** the code of `p_t` through `query_rotation`, on head `h`'s coordinates.
2. **Keys:** the code of `x_{s−1}`'s table point through `key_rotation`, as for the copy heads, on head `h`'s coordinates. They depend only on the token, so they are built once per vocabulary entry.
3. **Choice and weights:** as the copy heads, with the type heads' own margins.
4. **Payload:** the full-width `p_s` through `value_rotation`. The session keeps it for every position, per read.
5. **Mean:** each head's `Σ w_s · v_s / max(1, #{w_s > 0})`, then the mean over the type heads that found a candidate: `y`.
6. **Combine:** the stream adds `w_y ⊙ y`, `q16(x + w_c ⊙ c + w_y ⊙ y)`; the readout adds it unconditionally, `n + [matched] w_c ⊙ (c − n) + w_y ⊙ y`.

In integers:
- the rotations are `stack_rotation(raw, 2^31, GATE_EPSILON)` and `stack_hamilton`, as the copy heads;
- a code bit is 1 at or above 0;
- the payload is rounded half away from zero back to `p`'s exponent;
- the popcount is `popcount_swar`;
- each hook's sum is rounded once (claude-194's rule), the stream to Q16 exactly. The readout's type term is first rounded 43 bits below `n`'s grid, so that the sum stays inside an i128 for any `n`. That wording was made exact in review round 1.

**What is not served stays refused:**
- a session restore when a read has type heads. The payload cache is not in the saved session, and the byte codec (`session_codec.rs`) is outside the item's scope, so a restore would silently differ;
- a cache too large for `MAX_READ_CACHE_BYTES`.

## Cancel probe (run first)

- **The type heads already served on main?** No. At `origin/main` `9bf364855`, `ServedHamming::new` refuses any non-zero type-head weight (claude-194).
- **The float read withdrawn?** No.
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

All at the PR head, each with its evidence in the REPORT:
1. **The kernel against an f64 reference.** The type heads' codes, kept sets and means match a test-side f64 reference of the float read's steps, on synthetic reads over several seeds:
   - every vocabulary key code and every runtime query code equal;
   - every kept set equal with its distances;
   - every type mean within one step of the hook input's grid: `2^-16` in real units at the stream's Q16.

     Amended before any code ran. The first bound, `2^-20` of the input's scale, cannot hold: each payload is rounded half away from zero back to the input's grid, so a mean can be off by up to half a step, `2^-17`.
2. **The hooks against a reference** (claude-194's method). At every row of a sequence with repeats, each hook equals a test-side i128 combine over the read-free trunk's own integers at the hook:
   - the stream read, at depth 1, with both kinds of head;
   - the readout move, at the layer count, with both kinds of head;
   - type weights alone (copy weights 0) change the queries at some rows, so the test can fail.
3. **Weight 0 still serves as the read-free trunk,** with type heads present and both weights 0.
4. **The refusals by their text:**
   - a restore with type heads, through `restore_state`;
   - a payload cache past `MAX_READ_CACHE_BYTES`, through its check. No legal small synthetic shape reaches 4 GiB, so the check is a function of its own, tested at the limit and one past it, and called at load.

   claude-194's refusal of non-zero type weights is gone, and its test changes to serving.
5. **The timed steps equal the step,** with both kinds of head.
6. **Negative gates:** each named piece set aside fails its test: the payload cache write, the value rotation, the type mean in each hook, `popcount_swar`, and the restore refusal.
7. **Both disassembly audits pass** with the type heads reached: `audit_zero_matmul_serving.py --stack` and `m5_full_path_cost_controls`.
8. `cargo test -p uor-r4-integer` shows no new failure against `main`, and `loop gates --full` is green.
9. One notebook row. The item stays `ready`: steps 4 to 6 remain.

## Stop and ask if

Stop and decide (never wait) if:
- the type heads need a format change: then it is its own unit;
- the payload cache cannot be bounded inside the existing session limits;
- a bound in done-when 1 fails for a reason that is the float's, not the port's. Then the bound is amended in this ORDER with the reason, before any claim.
