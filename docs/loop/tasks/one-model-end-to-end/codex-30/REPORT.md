# REPORT: one-model-end-to-end, codex-30 (the k-term Cl(0,6) reader, slice 1)

Lab: codex. Order: [ORDER.md](ORDER.md).

## What was built

The **reader** accepts a map whose table holds `out_blocks * k` words and serves the sum of `k`
gathers. Nothing else changed.

- `crates/uor-r4-integer/src/stack/session.rs`:
  - `FbsMap` and `FbsMapImage` gain `terms`; `atoms`/`codes` hold `out_blocks * terms`.
  - the route loader derives `terms = table_words / out_blocks` (refused unless 1..=4 and unless
    it divides exactly), so **a one-term map parses today's table byte for byte** and needs no
    record field or schema change;
  - `stack_fbs_gather` accumulates the terms into the block, saturating at the residual exponent. A
    one-term map writes the same words it always did.
- `crates/uor-r4-cpu-train/src/fbs.rs`: the CPU port mirrors the terms (its gather sums, its layer
  validation divides by `terms`), so it cannot silently serve term 0 of a k-term map.

## The fixture

`a_two_term_route_is_the_sum_of_its_two_gathers_and_one_term_is_unchanged`: one term reproduces the
historical gather lane for lane; a two-term route equals the sum of its two one-term gathers; the
term count is the table length. **Integer library 701 passed** (700 before, 1 new); native-vector 49.

## What did not work, first

- **Two `uor-r4-cpu-train` tests fail — and they fail without this change too.** The stash check
  reproduces `ask_heads_at_both_hooks_match_serving_without_carry` at `fbs.rs:2296` with the change
  set aside, so it is a pre-existing CPU-port/serving parity red in the ask-head/carry path, not
  this unit's. Recorded, not fixed here.

## Still open (slice 2 and 3)

1. **The writer**: `pack_fbs_routes` still validates `atoms.len() == out_blocks` and writes one
   `u32` a block; the snapshot's `FbsRoutes` carries one atom/state a block. The writer must emit
   `terms` words a block for a k-term map.
2. **The trainer**: `FbsRoutes { atoms, states }` becomes `[u16; k]`/`[u8; k]`, the secant surrogate
   credits each of the 2k tables, and term 0 starts from the k = 1 solution.
3. **The arm**: k = 2 and 4 against k = 1 on the fresh panel, value-present per stratum with the
   exact sentence beside it.
