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

The unit is cancelled if the snap is no longer worth a unit. That happens if either check fails:
- **(a) The code under the profile is still current.** `stack_snap_select` must be unchanged since claude-177's profile (`e0bbacc43`), whose 46% share the item quotes. Check: `git log` of `kernels.rs` since then. Only claude-186 (`98a2f3b52`) touched the file, and none of its changed lines names the snap.
- **(b) The share holds at this base.** Build the base `fbs-toks` at `50d3958cd` and run it on claude-126's seed-1 export (sha256 `314c3909c66b58522a641c3f5b51c25dda4bda5ab2274bcfd0b5ca5038fdb305`). Take a `/usr/bin/sample` of a plain run, as claude-177 did. Cancel if `stack_snap_select` holds under 15% of the thread's samples.

Either result is an instrument reading, not a mechanism result.

**Result (1 Nov): both pass; the unit goes ahead.**
- (a) holds, as above.
- (b) The base `fbs-toks`, built at this ORDER's commit with no code change from `50d3958cd`, has sha256 `ca955e9feaa38db4ecd847352f166284888d8d171c7449701193fc789650c679`. Under the sample, the main thread holds 484 samples, and `stack_snap_select` is the top of stack in 199 of them, 41%. The next entries are `stack_native_writer` at 56 and `stack_snap_rotation` at 35.
- This sampled run decodes at 4,518 tok/s (median of 20). It is not the A/B: a sampler was attached.
- The files are in `local/claude/claude-241/` (`probe-sample.txt`, `probe-run.json`).

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

These are the item's five, as quoted above, met in this unit.

**The change:** a fast path inside `stack_snap_select`, for a query with no zero coordinate and four distinct magnitudes. Every other query keeps the scan.
- Rank the magnitudes. If the sorted order is an even permutation, its golden block is the only golden candidate. If it is odd, the three swaps of neighbouring ranks are the candidates.
- Add the axis root of the largest magnitude and the one compatible half-root.
- Run the same strict comparison over those 3 or 5 candidates, in index order. The scan compares 17.
- The proof is in the code's doc comment and the REPORT. It rests on the rearrangement inequality and Abel summation over the coefficients `(φ, 1, φ − 1, 0)`.

**The second candidate of the item** (one fixed-point product for the exact sign) is not in this unit unless the sort lands and time remains. If it is tried, it is a separate arm with its own A/B.

## Stop and ask if

Each of these is a decision recorded in the REPORT, never a wait.
- **The exhaustive sweep disagrees with the scan** on a query the fast path takes, and the proof does not explain it. Narrow the fast path to the queries the sweep proves, and record why.
- **The serving audit refuses the new code's form** (a NEON vector form, `fmov`, or a symbol outside the audited `stack_` names), and a barrier does not fix it. Revert, and keep the patch.
- **The A/B is not faster.** Revert, and keep the patch in this folder, as claude-185 did.
- **Another lab's build or timing is running during the A/B.** Wait for it to end, then re-run both sides. Never time on a shared machine (24 Oct).
