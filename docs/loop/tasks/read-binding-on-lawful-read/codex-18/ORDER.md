# ORDER: read-binding-on-lawful-read, codex-18

Lab: codex. Item: [read-binding-on-lawful-read](../../../../roadmap/items/read-binding-on-lawful-read.md).
Owner direction, 2026-11-16: revisit the held mechanism; harness ceremony is not the priority.

## Why this unit exists

The item was held until `m1-bvg-k-stage-t-training` (`#2836`) and
`m1-trunk-assembly-and-training` (`#2844`); both are now `done`. The held design's finite ladder starts
with an exact host reference, and that reference exposes a defect in the design's own rank key:
`R = (rel << 3) - distance` assumes a distance in `{0,2,4,6,8}`, but `positioned_root_cost` sums an
eight-coordinate L1 and the distance is unbounded. The scalar key ties a pair the read orders.

## Method

- `crates/uor-r4-discrete/src/read_binding.rs` (new, live): the read's exact comparator, the
  four-word sidecar span, and the rank-gap hinge `max(0, m_bind - Δrank) + W_miss * miss`, with
  `m_bind = 2` and `W_miss = 4`.
- One test records the `rel 1, distance 8` tie that breaks the scalar key; the rest cover the
  margin, the miss, the absent span and the permuted sidecar.
- `docs/labs/read-binding-on-lawful-read/README.md` gains the correction; the pre-registered
  `m_bind`/`W_miss` are unchanged.

## Files in scope

- `crates/uor-r4-discrete/src/read_binding.rs` (new)
- `crates/uor-r4-discrete/src/lib.rs` (module registration)
- `docs/labs/read-binding-on-lawful-read/README.md`
- `docs/loop/tasks/read-binding-on-lawful-read/codex-18/**`

Out of scope: wiring the sidecar into the CUDA counterfactual, the 4-arm GPU run, any float or
softmax, and the 1B run's files.

## Done when

1. The exact comparator and the rank-gap hinge are live and tested.
2. The scalar key's boundary tie is recorded in a test.
3. The lab README carries the correction and the unchanged `m_bind`/`W_miss`.
4. The discrete library suite passes except the pre-existing
   `apply_calls_are_confined...` failure.

## Cancel

Stop if the rank gap cannot be computed from the read's own keys without a float or a multiplier, or
if the correction changes `m_bind`/`W_miss`.

STOP and report when done.
