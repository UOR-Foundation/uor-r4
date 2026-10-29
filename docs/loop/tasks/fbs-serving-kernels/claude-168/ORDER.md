# ORDER: fbs-serving-kernels, claude-168 (the snap's scores from |r| by additions)

Lab: claude. Item: [fbs-serving-kernels](../../../../roadmap/items/fbs-serving-kernels.md). Base: `73b5d61f0`.

## Goal

The owner, 28 Oct 2026, in claude's chat: "I don't care what we serve at long as it is cheap".

claude-167 brought decode on the M1 to 1,829–1,861 tok/s. In its profile (956 samples on the thread), `stack_snap_select` holds 400 (42%). Each candidate's score `(A, B)` is built from 8 `small_mul`s, each a `match` over the coefficient, then summed.

**For a sign-compatible root, every product `r_c · coefficient` is `|r_c|` times a fixed small number,** by the definition of compatibility (claude-164). So a root's score comes from `s = |r|` by a few additions:
- **an axis root** on axis `i` (indices 0–7, `i = j >> 1`): `A = 2 s_i`, `B = 0`;
- **a half-root** (indices 8–23): `A = s_0 + s_1 + s_2 + s_3`, `B = 0`;
- **a golden root** (indices 24–119, block `(j − 24) >> 3`). The block's even permutation puts `φ/2` on axis `p`, `1/2` on `q` and `(φ−1)/2` on `t`, so `A = s_q − s_t` and `B = s_p + s_t`.

The candidates are still scanned in index order, with the same strict comparison, so the lowest-index tie rule is unchanged. Each block's `(p, q, t)` is read at compile time from `H4_ROOT_COEFFICIENTS`.

This is another step toward done-when 2 and 3. The item stays `ready`.

## Cancel probe (run first)

Run at `73b5d61f0`:
- **No open PR** touches `stack/kernels.rs`.
- **The root order** in `h4_classifier.rs` (`historical_root_coefficients`) is: 8 axis roots (two per axis), then 16 half-roots, then 12 permutations × 8 sign patterns. The new tests check that order against the table itself.

Both held. **Not cancelled** (instrument: none needed).

## Files in scope

- `crates/uor-r4-integer/src/stack/kernels.rs`
- `crates/uor-r4-integer/src/stack/tests.rs`
- `docs/loop/tasks/fbs-serving-kernels/**`
- `docs/research/notebook/m4-serving.md`
- `docs/roadmap/items/fbs-serving-kernels.md` (the Progress note)

## Done when

1. **Exactness.**
   - For every root and 10,000 random sign-compatible queries each, the structured `(A, B)` equals the `small_mul` sums.
   - `stack_snap_select` still equals the full 120-root scan (the existing grid, random and near-tie sets).
   - A test checks the index layout and every golden block's `(p, q, t)` against the table.
2. **Nothing else moves.** The whole `uor-r4-integer` library suite passes. `python3 scripts/audit_zero_matmul_serving.py <release uor-r4-stack> --stack` passes.
3. **Speed, on this M1, on claude-126's seed-1 export.** The new `fbs-toks --parts` runs alternately with claude-167's binary (sha256 `215b7adb…c0bd`), two runs each. Replies are identical to claude-163's to claude-167's. Measured: decode tok/s, the parts and a `sample` profile, with the total read from the thread line.
4. One row in `docs/research/notebook/m4-serving.md` and a REPORT.

## Stop and decide if

- **Any score or index differs.** Then the layout assumption is wrong for this table, so stop and find why.
- **The audit refuses the new code.** Then rewrite the read in the existing kernels' style.

No step waits on Casey.
