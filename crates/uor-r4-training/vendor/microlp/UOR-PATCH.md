# UOR-R4 D22 numerical repair

This directory retains `microlp` 0.6.0 and its Apache-2.0 license. The original
registry archive checksum is
`a8f19803f918039a07f3c32b23716824d8f073543417bdac5b1b8619817e3674`.
Original package metadata is retained in `Cargo.toml.orig` and
`.cargo_vcs_info.json`. The workspace patches this exact dependency for offline
constructor training; native serving crates do not depend on it.

The UOR modifications are in `src/repair.rs`, `src/lu.rs`, `src/solver.rs` and
their fixture interface in `src/lib.rs`. They add scale-aware factorization,
original-basis normal/transpose residual checks, bounded exact dyadic refactoring,
and transactional replacement of factors and solver state. An unusable basis is
rejected without treating that numerical result as LP infeasibility. A previous
basis may be refreshed before pivot selection is recomputed.

Exact arithmetic uses Rust `num-bigint`, `num-rational` and `num-traits`.
The fallback is bounded to 512 rows and 16,384-bit rationals; crossing either
limit is a numerical resource failure. Rounded factors must still pass the
residual qualification. Exact singularity is distinct from a floating-point
threshold failure.

The `numerical-fixtures` feature exposes the scoped retained-basis tests. The
standalone harness lives in
`docs/labs/d22-constructor-2026-10-10/numerical-tests/`. Source manifests and sealed
attempts identify which bytes were actually executed. A captured singular basis
must be rejected; it cannot be required to factor. Synthetic qualification and
historical constructor replay do not establish current-model performance.

The dated lab result owns the execution outcome and model comparison. This
source note is not an acceptance or capability claim.
