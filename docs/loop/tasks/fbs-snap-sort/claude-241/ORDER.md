# ORDER: fbs-snap-sort, claude-241

Lab: claude. Item: [fbs-snap-sort](../../../../roadmap/items/fbs-snap-sort.md). Base: `50d3958cd`.

## Goal

The item's done-when reads:

> 1. **Exact.** The new `stack_snap_select` equals `snap_select_full_scan` (the kernels' own test) and the independent oracle (`snap_selection_matches_an_independent_exact_oracle`, `stack/tests.rs`) on every existing test set. It also equals them on an exhaustive sweep of every query with each coordinate in `−8..=8` (83,521 queries, ties included). A negative gate shows the sweep fails with a wrong tie rule.
> 2. **Replies byte-identical** on claude-126's snapped export and on a prompt whose reply varies.
> 3. **The serving audit passes:** the native `--stack` audit and the WASM serving-surface audit, FULL PASS.
> 4. **Faster, measured.** `fbs-toks --parts` against the base binary, alternating, at least two runs each, on this M1 on AC power, with no other build or timing running. It reports decode tok/s against context, the step, and the snap's new share. If it is not faster, the change is reverted and its patch kept in the unit's folder, as claude-185 did.
> 5. **One notebook row, and a REPORT with the negatives first.** The pipeline map's gap 3 records the new speed.

## Cancel probe (run first)

To fill in.

## Files in scope

- `crates/uor-r4-integer/src/stack/kernels.rs`
- `crates/uor-r4-integer/src/stack/tests.rs`
- `crates/uor-r4-integer/src/bin/fbs-toks.rs`
- `scripts/audit_zero_matmul_serving.py`
- `docs/model/pipeline-map.md`
- `docs/roadmap/items/fbs-snap-sort.md`
- `docs/loop/tasks/fbs-snap-sort/**`
- `docs/research/notebook/m4-serving.md`

## Done when

To fill in.

## Stop and ask if

To fill in.
