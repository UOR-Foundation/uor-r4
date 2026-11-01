---
id: fbs-snap-sort
title: An exact icosian snap by sorting, in place of the 17-candidate scan
milestone: research
area: serving
status: ready
open_to: lab
needs_gpu: false
depends_on: [fbs-serving-vector-kernel]
notebook: docs/research/notebook/m4-serving.md
scope: [crates/uor-r4-integer/src/stack/kernels.rs, crates/uor-r4-integer/src/stack/tests.rs, crates/uor-r4-integer/src/bin/fbs-toks.rs, scripts/audit_zero_matmul_serving.py, docs/model/pipeline-map.md, docs/roadmap/items/fbs-snap-sort.md, docs/loop/tasks/fbs-snap-sort/**, docs/research/notebook/m4-serving.md]
---

## Why it matters

The owner, 31 Oct 2026, in claude's chat: "why is the icosian snap 42% of our time? what is slowing it down? we should improve that process". Earlier, 28 Oct: "I don't care what we serve at long as it is cheap".

**The snap is the largest single kernel in a served token.** claude-177's profile, after the hardware multiply and divide, has `stack_snap_select` at 333 of 725 samples on the thread: 46% of a 207 µs step on claude-126's snapped export (width 256, 6 layers). The 42% was claude-167's reading.

**Its share, unit by unit** (each row's new profile, in the order the units ran). claude-168 made the snap itself cheaper, and its share fell. Since then the kernels around it got faster, and its share rose:

| Unit | Change | Snap's share |
|---|---|---|
| claude-167 | the multiple tables by doublings | 42% |
| claude-168 | the snap's scores by additions | 25% |
| claude-169 | the Hamilton product by shifts | 32% |
| claude-177 | the hardware multiply and divide | 46% |

**How often it runs** (claude-163): two snaps per 4-channel block in each layer's recurrence core (the lane's transition and the output gate), and one per 4-unit block in its gated product.

[`fbs-serving-vector-kernel`](fbs-serving-vector-kernel.md) left "fewer golden candidates" open and "named for a follow-up item". This is that item.

## What is missing

**What one snap does now** (`stack_snap_select`, `crates/uor-r4-integer/src/stack/kernels.rs`):
- It finds the root of the 120 unit icosians with the largest exact dot product with the raw quaternion, and keeps the lowest index on an exact tie.
- It scores only the roots that agree in sign with the query on every coordinate where both are nonzero (`SNAP_CANDIDATES`, claude-164). For a query with no zero coordinate that is 17: four axis roots, one half-root and 12 golden roots.
- Each score is a pair `(A, B)` meaning `A + Bφ`, made of additions of `|r|` (`snap_compatible_score`, claude-168).

**Where its time likely goes** (read from the code, not split by a profile):
1. **17 comparisons in series.** Each candidate is compared with the best so far, so each step waits on the one before.
2. **Each comparison branches on the data.** The exact sign of `da + db·φ` (`snap_difference_positive`) takes up to five branches: `db == 0`, `db`'s sign, `P`'s sign, and the two bounds of `below_d_sqrt5`. Inside a 0.04% band it takes two `u128` squares.
3. **Code from the multiply ban.** The bounds' products by 2289 and 1145 are shifts and additions behind `black_box`, built when the audit refused every multiply (claude-165). D80 and D81 allow the scalar multiply now. Other `black_box`es keep the lanes out of NEON registers, whose `fmov` the audit refuses. They stay unless the audit allows another form.

**Candidate exact changes, best first:**
- **Sort instead of scan.** Every golden root's compatible score is `φ·s_p + s_q + (φ − 1)·s_t` for its block's even permutation `[p, q, t]` of the axes, with `s = |r|`. By the rearrangement inequality, the best of all 24 orders puts the largest `s` on `φ`, the next on `1` and the next on `φ − 1`. Only the 12 even permutations are roots. So if the sorted order is odd, the best root is likely one swap of neighbouring ranks away: a swap of ranks `i` and `j` loses `(c_i − c_j)(s_i − s_j)`, and a wider swap or a 4-cycle loses at least as much. The unit proves this or keeps every candidate it cannot rule out. The family winners are then:
  - the axis root of the largest `s`, scoring `2 s_max`;
  - the one compatible half-root, scoring `s_0 + s_1 + s_2 + s_3`;
  - at most three golden roots.

  That is five compare-and-swaps and two or three exact comparisons in place of 17. Ties are the hard part: when magnitudes are equal, several roots score the same, and the lowest index must still win. A query with a zero coordinate may keep the scan.
- **One fixed-point product for the exact sign.** For these ranges (`|da| ≤ 2^35`, `|db| ≤ 2^34`), the norm `da² + da·db − db²` is a nonzero integer whenever `da + db·φ ≠ 0`, and `|da − db/φ| < 2^36`. So `|da + db·φ| > 2^-36`, and the sign of `da·2^70 + db·round(φ·2^70)` in `i128` is the exact sign. That is a few scalar multiplies (`round(φ·2^70)` has 71 bits) and no branch. Check the bound in the unit before relying on it.
- **Fewer barriers,** keeping only those the serving audit still needs.

**Not this item:** a NEON kernel (the audit refuses vector forms and `fmov`). And not the unsnapped contract: `lut-stack/5` runs no snap but is about 30 µs a token slower, a 235 µs step against 205 µs, in its core and its gated product (claude-185).

## Done when

1. **Exact.** The new `stack_snap_select` equals `snap_select_full_scan` (the kernels' own test) and the independent oracle (`snap_selection_matches_an_independent_exact_oracle`, `stack/tests.rs`) on every existing test set. It also equals them on an exhaustive sweep of every query with each coordinate in `−8..=8` (83,521 queries, ties included). A negative gate shows the sweep fails with a wrong tie rule.
2. **Replies byte-identical** on claude-126's snapped export and on a prompt whose reply varies.
3. **The serving audit passes:** the native `--stack` audit and the WASM serving-surface audit, FULL PASS.
4. **Faster, measured.** `fbs-toks --parts` against the base binary, alternating, at least two runs each, on this M1 on AC power, with no other build or timing running. It reports decode tok/s against context, the step, and the snap's new share. If it is not faster, the change is reverted and its patch kept in the unit's folder, as claude-185 did.
5. **One notebook row, and a REPORT with the negatives first.** The pipeline map's gap 3 records the new speed.

## How to start

- **Read:** `kernels.rs` from `snap_add_square_terms` to `stack_snap_rotation`, and the snap tests in its test module (`snap_select_full_scan`, `snap_difference_positive_by_squares`). The root table's order is `H4_ROOT_COEFFICIENTS`: axis roots `0..8`, half-roots `8..24`, golden blocks `24 + 8k`.
- **Before claiming:** `integer-hamming-read`'s open order (claude-235) holds `stack/tests.rs` and `m4-serving.md`. Claim after it merges, or keep the new tests in `kernels.rs`'s own module. `docs-status-refresh`'s scope covers `docs/model/*.md`, the pipeline map included: check that no open order of it holds the map.
- **Timing:** no other lab's build or timing may run on the laptop during the A/B. A shared machine gave a false reading on 24 Oct.

## Rules that apply

- **Serving stays integer** (CHARTER §3.6, as amended by D80 and D81): no float or transcendental instruction in the step.
- **Exact or nothing:** the snap's choice is the model's arithmetic. A faster snap that differs on one tie changes the model.
- **A negative closes a configuration, never a mechanism** (§4.1).

## Sources

- `docs/research/notebook/m4-serving.md`: the rows of claude-163, claude-164, claude-165, claude-167, claude-168, claude-169, claude-177 and claude-185.
- `crates/uor-r4-integer/src/stack/kernels.rs`: `stack_snap_select`, `snap_difference_positive`, `below_d_sqrt5`, `SNAP_CANDIDATES` and `GOLDEN_AXES`.
- [`fbs-serving-vector-kernel`](fbs-serving-vector-kernel.md): its open candidates.
