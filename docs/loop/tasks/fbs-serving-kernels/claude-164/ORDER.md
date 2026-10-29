# ORDER: fbs-serving-kernels, claude-164 (an exact sign-compatible icosian snap)

Lab: claude. Item: [fbs-serving-kernels](../../../../roadmap/items/fbs-serving-kernels.md). Base: `d0f79c441`.

## Goal

The owner, 28 Oct 2026, in claude's chat: "I don't care what we serve at long as it is cheap".

claude-163 split a served token on the M1. Two parts take 96% of it:
- the recurrence core, 56.3%;
- the feed-forward's gated product, 39.8%.

Both run the 120-root icosian snap (`stack_snap_select`, `kernels.rs`) several times per layer: twice per 4-channel block in the core, once per 4-unit block in the product. It scores all 120 roots with exact `ℤ[φ]` comparisons and keeps the lowest index on a tie.

**This unit makes the snap score only the roots that can win, with the same result.**
- **The argument:** the 120 unit icosians are closed under flipping the sign of any coordinate. Take a root whose nonzero coordinate `j` has the opposite sign to the query's nonzero `r_j`. Flipping that coordinate gives another root whose score is higher by `2 |r_j ρ_j| > 0`, so the first root is never a maximizer.
- **The candidates:** the sign-compatible roots therefore hold every maximizer. Scanning them in index order with the same strict comparison returns the same root as the full scan, the lowest-index tie included.
- **How many:** for a query with no zero coordinate, 17 of the 120 are compatible. They are one axis root per coordinate (4), one half-root (1), and one golden root per even permutation (12). A zero coordinate frees that sign, and the zero query keeps all 120.
- **The table:** each of the 81 sign patterns maps to its compatible roots in index order, built at compile time. At run time the snap reads it by the query's signs, with no new arithmetic.

This is a step toward the item's done-when 2 and 3 (a faster exact kernel for the largest part, and its tokens/s), not their completion. The done-when names a vector kernel; the vector kernel and the switch stay open. The item stays `ready`.

## Cancel probe (run first)

Run at `d0f79c441`:
- **No open PR** touches `stack/kernels.rs`, `stack/session.rs` or `h4_classifier.rs`.
- **`stack_snap_select` still scans all 120 roots** (`for (index, root) in H4_ROOT_COEFFICIENTS.iter().enumerate()`).
- **The roots are closed under sign flips.** `h4_classifier.rs` builds the 8 axis roots with both signs, the 16 half-roots with every sign, and the 96 golden roots as 12 even permutations with independent signs on their three nonzero coordinates. A test in this unit checks closure over the table itself.

All three held. **Not cancelled** (instrument: none needed).

## Files in scope

- `crates/uor-r4-integer/src/stack/kernels.rs`
- `crates/uor-r4-integer/src/stack/tests.rs`
- `docs/loop/tasks/fbs-serving-kernels/**`
- `docs/research/notebook/m4-serving.md`
- `docs/roadmap/items/fbs-serving-kernels.md` (the Progress note)

## Done when

1. **Exactness.** `stack_snap_select` returns the full 120-root scan's index on:
   - every query in `[-6, 6]^4` (28,561, zeros and ties included);
   - 200,000 pseudo-random queries at magnitudes from 1 to `2^30`;
   - queries placed on and between roots, scaled.

   The full scan is kept as the test's reference.
2. **The table.** A test checks the closure of the root set under sign flips. It checks the sign rule for each root coordinate against the exact `ℤ[φ]` sign (`snap_difference_positive`). It checks that each pattern's list is exactly its compatible roots, in index order.
3. **Nothing else moves.** The whole `uor-r4-integer` library test suite passes (`--features serving-wasm`, `--no-fail-fast`). `python3 scripts/audit_zero_matmul_serving.py <release uor-r4-stack> --stack` passes.
4. **Speed, on this M1, on claude-126's seed-1 export:**
   - The new `fbs-toks --parts` runs alternately with claude-163's sealed binary (sha256 `9c1f1778…6f9a`), two runs each.
   - Every run's reply is identical to claude-163's.
   - Measured: decode tok/s, and the core and product parts, before and after.
5. One row in `docs/research/notebook/m4-serving.md` and a REPORT.

## Stop and decide if

- **Any query in the exactness test returns a different index.** The argument above is then wrong for this table, so the unit stops and finds why. It never ships an approximate snap.
- **The audit fails** on an instruction the table read brings in, such as an `fmov` from a packed load. Then the read is rewritten in the style of the existing kernels (`black_box`, explicit loops).

No step waits on Casey.
