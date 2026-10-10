# D22 numerical repair source checkpoint

**UNCOMPILED / UNQUALIFIED.** Source preparation only. No fixture execution,
actual-basis qualification, solver replay, model training or native scoring is
admitted by this checkpoint. The owner clarification / TEST FITNESS gate remains
pending. The root lab controls build and compute admission.

The adjacent standalone Cargo manifest uses the source-owned microlp vendor
copy and enables only its numerical fixture interface. Nine synthetic test
functions cover small/rescaled pivots, exact-dyadic fallback, exact singularity,
nonsymmetric normal/transpose dense/sparse solves, malformed/nonfinite input,
transactional reset/pivot, corrupt-eta refresh/reselection and preservation of
the artificial phase objective. All tests are NOT_RUN.

After admission, use an owned per-lab Cargo target and the standalone manifest;
never build in the registry or shared owner checkout. Actual regression must
also restore and authenticate the four captured original CSC bases:

| Capture | SHA256 |
| --- | --- |
| Original basis 1 | `c5c73e1b9f7baf598cdda33e4a458f9208f0c943b4d471ba646eb129c495a794` |
| Original basis 2 | `55d8c2eadbcfc072d86a6310602184603030a318986ebeb2a2d885958f10ae05` |
| Later factor 134 | `337e0917af3a3c397ce70db1d096f24adc07c054de32a5cf6e1ed6c23491e221` |
| Later factor 150 | `4b5abc63cc98cde36c8ff1ba47f1e9388548f81dad3cb26bf15e73bfb2fbd2b6` |

Their source-level qualification entry point is `microlp::repair::qualify`.
There is no automatic file discovery or implicit execution of restored bases.
A later admitted runner must verify the exact files and record each outcome;
none is executed or certified in this checkpoint.

The fast LU keeps the retained scale-aware threshold, followed by all-unit
normal/transpose dense/sparse original-basis residual checks. Failure invokes
exact rational LU on the *stored f64 dyadic matrix*, with exact zero tests and
partial pivoting. Rounded factors are still residual-qualified. Limits are
512 rows and 16,384 bits per rational numerator/denominator; these bound memory
and return a numerical resource error, never an infeasibility certificate.
The fallback does not implement an exact LP or guarantee a usable floating solve
for an ill-conditioned exact-nonsingular basis.

Every eta solve is checked against original columns of the current basis. A
failed selection/pivot gets one old-basis refresh and complete reselection;
repeat failure remains an internal numerical error. Pivot uses a cloned whole
solver and commits only after updated primal/current-objective solves pass.
Reset builds new factors and scratch before discarding old factors/etas. The
working artificial objective is preserved across a numerical refresh; global
simplex EPS, destinations, constraints and branch budget are unchanged.

Outstanding: actual Rust type checking and synthetic tests; fixtures against
restored bases; actual saved-problem replay; timing/RAM of all-unit checks,
per-pivot solver cloning/recalculation and rational growth; independent source
review; model integration and exact exported candidate checks. A persistent
ambiguous pivot is rejected after refresh, not skipped as an infeasible branch.
Further pivot selection changes may still be necessary; no assignment or
mechanism success is claimed.
