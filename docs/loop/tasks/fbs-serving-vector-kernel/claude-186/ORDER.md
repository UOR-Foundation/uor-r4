# ORDER: fbs-serving-vector-kernel, claude-186

Lab: claude. Item: [fbs-serving-vector-kernel](../../../../roadmap/items/fbs-serving-vector-kernel.md). Base: `c5805400a`.

## Goal

The item's done-when 2 and 3 (restated by claude-185):

> 2. **A faster exact change** to the snap site, in either form, or to the Hamilton product. Either form means the snapped selection or the unsnapped `stack_rotation` (claude-185). The change can be fewer golden candidates, a faster exact square root or quotient, or a vector kernel the audit allows.
> 3. **Speed:** `fbs-toks --parts` against the previous unit's sealed binary, run alternately, on this M1.

**This unit: `stack_isqrt` starts at the value's top bit.**
- It now walks its probe bit down from `2^126`, two bits at a time, until the bit is no larger than the value. That is the largest power of four at most the value.
- One leading-zero count gives the same bit, `1 << ((127 − lz) & !1)`, and zero stays zero.
- That is exact by construction. It serves the unsnapped rotation, and every other caller too: both RMS norms, the lift and the session's complement.

## Cancel probe (run first)

Run at `c5805400a`:
- **The loop is there:** `stack_isqrt` (`crates/uor-r4-integer/src/stack/kernels.rs:408`) still walks down from `2^126`.
- **It has six callers:**
  - the rotation (`session.rs:3466`);
  - the complement (`session.rs:3407`);
  - the norms (`kernels.rs:593`, `:631`);
  - the lift (`:955`);
  - `:987`.
- **No open PR touches `stack/kernels.rs`.**
- **A reply that can differ on `raw_both`:** dual-c131's default-prompt reply is one token repeated (claude-185). The prompt `id_i = (613 i) mod 4000 + 50`, for i = 1..128, gives a 65-token reply with 29 distinct tokens on the base binary. That prompt is the reply check on that artifact.

**Go. Class: mechanism.**

## Files in scope

- `crates/uor-r4-integer/src/stack/kernels.rs`
- `docs/roadmap/items/fbs-serving-vector-kernel.md` (the state note)
- `docs/research/notebook/m4-serving.md`
- `docs/loop/tasks/fbs-serving-vector-kernel/**`

## Done when

1. **`stack_isqrt` starts at the top bit,** and a host test holds it equal to the walking form:
   - every value below `2^20`;
   - every power of two and its neighbours up to `2^127`;
   - `u128::MAX`;
   - a million seeded values at every bit length.

   A negative gate, starting one power of four too low, turns that test red.
2. **Every reply is byte-identical** against the sealed base (`f7842898…e2a3`): claude-126's default reply, and dual-c131's `raw_both` reply under the varying prompt.
3. **The audit:** `audit_zero_matmul_serving.py --stack` is FULL PASS on the release binary.
4. **Speed:** `fbs-toks --parts` against the base, alternating, two runs each, on both artifacts.
5. **One notebook row,** a REPORT, and the item's state note.

## Stop and ask if

Each of these is a stop-and-decide, never a wait:
- **any value, reply or state differs:** a defect. Revert.
- **it is not faster:** a measured negative, so revert, as in claude-185.
