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

## Slice 2: the writer, the fitter, and the measurement that settles post-hoc k

### What was built

- **`FbsRoutes` carries `terms` and `counts`** (`crates/uor-r4-integer/src/fbs_snapshot.rs`). `terms`
  is one by default and `counts` is empty by default, both skipped on serialize when empty, so a
  historical snapshot's JSON and artifact are unchanged.
- **`pack_fbs_routes`** (`stack/format.rs`) writes `out_blocks * terms` words and records each
  block's own term count as `count - 1` in bits 22..23 of the block's first word. Bits 22..31 were
  unused (`atom` is 16 bits, `state` 6), so a one-term route is byte for byte the old word.
- **The reader** (`stack/session.rs`) reads the count from that word and serves only those terms,
  and `FbsMapImage`/the CPU port (`crates/uor-r4-cpu-train/src/fbs.rs`) carry and honour it. A
  map's extra terms are padding for a block that does not use them, **not a perturbation**.
- **`fit_routes_k`** (`crates/uor-r4-float-train/src/fbs_export.rs`) fits `k` signed Cl(0,6)
  gathers to a projection's dense map, greedily by signed correlation, and stops adding terms when
  the block's residual is gone (`<= 1e-6` of its norm), recording the count.
- **`read_projection_dense` + `route_term_report` + the `route-terms-probe` bin**: the projection's
  effective dense map read through the model's own CUDA projection on the unit basis, fitted at
  each `k`.

### The measurement

On arm r's master (`/tmp/opencode/armr-copy-s2/models/step-016384`, 6 layers, 264 projections):

| k | residual (max) | blocks reproduced exactly |
|---|---|---|
| 1 | 0.000000000 | **1584 / 1584** |
| 2 | 0.000000000 | 1584 / 1584 (every block serves 1 term) |
| 4 | 0.000000000 | 1584 / 1584 |

**The trained trunk's projection is already exactly one signed gather, in every block.** The
secant surrogate does not leave off-gather mass: the one-atom family is not a lossy compression of
what this model learned, it is what it learned. So:

- **Post-hoc k buys nothing.** Exporting this master at `terms = 2` is a provable no-op (and now a
  safe one, where before the greedy fit added a spurious ±1 to all 1,584 blocks).
- The earlier "rank-1 covers 1.7 % of a random dense projection" figure is about a *random* map.
  Trained maps adapt to the family; that number cannot be used to argue headroom.
- A k-term gain, if any, must come from **training with the extra terms in the forward**, so the
  optimizer can spend them. That is a new arm, not an export flag.

### What the arm needs

`Fbs_2Atom_Mix` (`spiralcore_fbs_arms.cu`, arm 2) is already the sum of two routed gathers, but it
scales by `1/sqrt(2)` (`sc_arm_scale`), which the `±1` route format cannot carry. The arm is
therefore **arm 3**: the same kernel with `paths = 2` and `scale = 1`. In the kernel that is one
line (`sc_arm_scale`) and one (`sc_arm_paths`); around it, a fifth registered arm (enum, `ALL`,
the receipt identity, the two-path query widening) and the runner's selection. Then the same
16,384-step recipe, exported with `terms = 2` and read on the fresh panel.

### Checks

- Integer library **704 passed** (701 + 3: the summed-gather fixture, the block-count fixture, the
  count-encoding fixture); native-vector 49; float-train 625 passed with 2 pre-existing
  decode-cache failures (both fail with these changes stashed); cpu-train 509 with the same 2
  pre-existing ask/carry parity failures.
