# ORDER: fbs-serving-vector-kernel, claude-185

Lab: claude. Item: [fbs-serving-vector-kernel](../../../../roadmap/items/fbs-serving-vector-kernel.md). Base: `c95dfaea6`.

## Goal

The item's done-when 1 and 2 read:

> 1. **The map.** Its gap 3 and R3 record that R3 was met with exact structured kernels (claude-164 to claude-169, about 5.2 ns a value). The per-value claim is restated as an open question.
> 2. **A faster exact change** to the snap or the Hamilton product: fewer golden candidates, or a vector kernel the audit allows.

**The item was written before claude-177 and claude-182,** which changed what serving computes:
- the stack uses the hardware multiply and divide (claude-177, 4,829 tok/s);
- the artifacts that pass the export gap are unsnapped (`lut-stack/5`, claude-182). They run no icosian snap; each gate and transition site normalises its quaternion with an integer square root and four `u128` divides by the same norm (`stack_rotation`).

**This unit:**
1. **The map's gap 3 and R3,** as done-when 1 asks, with the current speeds.
2. **A faster exact `stack_rotation`, the unsnapped form of the snap site.**
   - One reciprocal of the shared norm, `m = ⌊2^78 / norm⌋`, then per coordinate `q = (|v| m) >> 32` and one remainder correction. That is one divide in place of four.
   - `q` is the quotient or one less, because its error is below `|v| / 2^32 < 1`.
   - **Decision (CHARTER §5.1):** reading done-when 2's "the snap" to include its unsnapped form, and moving the item's baseline from claude-169's binary to this head's. The item text is updated under a fresh decision review.

## Cancel probe (run first)

Run at `c95dfaea6`, `fbs-toks --parts` on this M1 on AC power. The binary is `f7842898…e2a3`.

| Artifact | Decode tok/s | Step | `recurrence.core` per layer | `mlp.product` per layer |
|---|---|---|---|---|
| claude-126 s1, snapped (`314c3909…`) | 4,832 | 205 µs | about 15.2 µs | about 9.0 µs |
| dual-c131 s1, `raw_both` (`a9df5263…`) | 4,236 | 235 µs | about 17.9 µs | about 11.4 µs |

- **The unsnapped artifact is about 30 µs per token slower,** all of it in the two parts that run the rotation.
- **No vector code runs in the step** (the item's text still holds).
- **No open PR touches** `stack/session.rs` or `stack/kernels.rs`.

**Go. Class: mechanism.**

## Files in scope

- `crates/uor-r4-integer/src/stack/session.rs`
- `crates/uor-r4-integer/src/stack/tests.rs`
- `docs/model/pipeline-map.md`
- `docs/roadmap/items/fbs-serving-vector-kernel.md`
- `docs/research/notebook/m4-serving.md`
- `docs/loop/tasks/fbs-serving-vector-kernel/**`

## Done when

1. **The map's gap 3 and R3** say that R3 was met by exact structured kernels, with today's speeds. The per-value claim is an open question.
2. **`stack_rotation` uses one divide,** and a host test holds it equal to the four-divide form:
   - every edge case: zero, one non-zero coordinate, `i32::MIN` and `i32::MAX`, tiny norms;
   - both epsilons, several `lambda`;
   - at least a million seeded quaternions.

   A negative gate (dropping the correction) turns that test red.
3. **Every reply is byte-identical** on claude-126's export and on dual-c131's `raw_both` artifact, against the base binary.
4. **The audit:** `audit_zero_matmul_serving.py --stack` is FULL PASS on the release binary.
5. **Speed:** `fbs-toks --parts` against the base binary, run alternately, on both artifacts.
6. **The item text** is restated under a fresh decision review. One notebook row and a REPORT.

## Stop and ask if

Each of these is a stop-and-decide, never a wait:
- **any reply or state differs:** a defect, not a trade. Revert the change.
- **the change is not faster:** report it as a measured negative for this configuration, and keep the four divides.
