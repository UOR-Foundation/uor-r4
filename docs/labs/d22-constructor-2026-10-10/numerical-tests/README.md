# D22 numerical repair source checkpoint

**Numerical qualification in progress; no model result.** Owner clarification
and TEST FITNESS admitted this numerical work. The four captured basis checks
are complete. The corrected saved-problem replay is running; no legal assignment
or model-quality conclusion is claimed. The root lab controls model admission.
The existing exposed 128 rows are an owner-authorized diagnostic, not a new
fresh qualification panel.

The adjacent standalone Cargo manifest uses the source-owned microlp vendor
copy and enables only its numerical fixture interface. Twenty-two synthetic test
functions cover small/rescaled pivots, exact-dyadic fallback, exact singularity,
nonsymmetric normal/transpose dense/sparse solves, malformed/nonfinite input,
transactional reset/pivot, corrupt-eta refresh/reselection and preservation of
the artificial phase objective, ambiguous-pivot reconstruction, phase loss, and
fixing a basic variable at an interior value, compensated reduced costs and
transpose refinement, and signed pivot reselection constrained by all reduced-cost
step bounds. The latest corrected source passed
all twenty-two tests and both saved-basis admission tests on the owned pod. These
are numerical fixtures, not model evaluations.

Execution uses an owned per-lab Cargo target and the standalone manifest, never
the registry or shared owner checkout. The regression authenticated the four
captured original CSC bases:

| Capture | SHA256 |
| --- | --- |
| Original basis 1 | `c5c73e1b9f7baf598cdda33e4a458f9208f0c943b4d471ba646eb129c495a794` |
| Original basis 2 | `55d8c2eadbcfc072d86a6310602184603030a318986ebeb2a2d885958f10ae05` |
| Later factor 134 | `337e0917af3a3c397ce70db1d096f24adc07c054de32a5cf6e1ed6c23491e221` |
| Later factor 150 | `4b5abc63cc98cde36c8ff1ba47f1e9388548f81dad3cb26bf15e73bfb2fbd2b6` |

Their source-level qualification entry point is `microlp::repair::qualify`.
There is no automatic file discovery or implicit execution of restored bases.
The sealed `qualification-attempt2` results are: original bases 1 and 2
QUALIFIED (0.247 s and 0.155 s); later factor 134 rejected as exactly singular
(742.157 s); later factor 150 QUALIFIED through the exact fallback (529.776 s).
Each qualified basis passed all 1,760 unit normal/transpose dense/sparse checks.
The later singular rejection is a sound numerical outcome, not LP infeasibility.
The replay gate accepts that explicit later-basis classification only with the
matching exact-singular counters; generic numerical failures remain inadmissible.
Unchanged LU and residual code permit reuse of these basis checks after the
subsequent solver-state corrections.

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

Finite ambiguous pivot arithmetic now proposes a basis exchange on a clone. The
solver rebuilds the proposed basis from original columns and recalculates the
actual RHS and working objective before committing. Primal and dual callers
explicitly certify their required phase. A fix-variable exchange has its own
context, permits the required objective increase and marks an interior fixed
nonbasic variable before checking working dual feasibility. A rejected exchange
or refreshed state leaves the original solver untouched. Free nonbasic variables
require zero working reduced cost within unchanged EPS. Original-objective flags
are preserved during artificial-objective refresh.

The saved input is pinned to SHA256
`0fe91060f10c0d6107be3565f9ae0746c30e742a00d02935398f6764b84b974d`.
Earlier replay attempts 1 and 2 ended without an assignment at a row/column pivot
agreement boundary (row 355, column 465). The arithmetic disagreement was about
9.87e-15 against a 1.15e-16 relative certificate threshold. The threshold was not
lowered. Attempt 3 tests passed and its immutable replay completed after 3,099.539 s
without an assignment. It rejected the proposed fresh basis at the same pivot
boundary, after 248 certified factors and three exact fallbacks. Its source is
superseded because independent review identified missing phase certification.
Attempt 4 compiled and passed 13/14 fixtures, then stopped: the new phase gate
over-rejected a valid zero-cost free nonbasic variable. That real gate defect was
corrected; it was not reclassified as a fixture-only failure.

`replay-attempt5-phase-free` passed fourteen synthetic tests (5.784 s including
compilation), two admission tests (4.014 s) and a release build (5.035 s), with
227,782,656 B peak child RSS. Its saved replay completed after 33.461 s with
`BACKEND_NUMERIC_OR_NO_INCUMBENT`: the recomputed state violated the required
dual phase at iteration 1,307. It returned no assignment. Counters: 63 certified
factors, 118,165 certified solves, two committed refreshes, three rejected
solves and no exact fallback. Report SHA256:
`d2c0c13d89eb72b7118fdb074d6cdd1fa3a3b60106c53e286267ee4bcafe4ef5`;
manifest `adebfe15edbd6a70457b8bf52088abbd603164d324edfc380115f32ba84488b8`.
Exact solver SHA256:
`50abce2c6d33812fffbcac2484ee7c958ad7ec8c7456e4e996ab1d10ee58e7aa`;
standalone source manifest:
`917bf730d0a42133bc46db600596c31e81734332c7d190b757b28799b5791cfb`;
executable:
`c464a9f87675bb62bbe626ded1b839cc422fbc499828e32244c233b6f54f1276`.
The optional progress log reports phase entry, factor boundaries, refreshes and
iteration counts; it does not change solver choices. CPU activity alone is not
convergence evidence.

Subsequent numerical attempts preserve every failure and source:

| Attempt | Focused checks | Saved replay outcome |
| --- | --- | --- |
| 6, every-pivot phase certificate | 15 + 2 PASS | 21.540 s; rejected reduced cost `1.06395941690802742e-10` at an upper bound, above unchanged EPS `1e-10`, iteration 182 |
| 7, compensated costs and transpose refinement | 16 + 2 PASS | 23.177 s; same boundary; residual improved `2.35957e-18` to `4.08420e-19`, while reduced cost changed only about `9.5e-23` |
| 8, signed ratio | 16/17 fixtures passed | Replay NOT_RUN; new fixture failed because its row equilibration doubled the intended negative ratio; production correctly rejected it |
| 9, corrected stored-coefficient fixture | 17 + 2 PASS | 22.043 s; unfiltered signed minimum ratio `-1.76859802026159849e-5` amplified a small wrong-sign cost; full candidate certificate rejected it |
| 10, all-column step interval | 18 + 2 PASS | 22.750 s; no admissible signed zero crossing in the reduced-cost interval, no assignment |
| 11, persistent feasibility-phase recovery | 22 + 2 PASS | RUNNING; no assignment or gate completion claimed |

Attempt 10 uses solver SHA256
`a89c9f9806837c57356b00bc7dc11107670c1b57c603fdd3a197fb39f5f78870`,
source manifest `01deaae355fffb219e85086b2d602e1d9d7730f3722d0f283b5455d2e8716f9b`
and executable `bd6c1d3b00c3dc06c968146949ee8ee8f08cfc6a6870262ac2032ae79785ba99`.
Its tests took 6.179 s and 4.169 s, and release build 4.769 s. It first retains
Harris selection, then refreshes and reselects after a rejected candidate, then
allows one actual signed-ratio fallback. The fallback interval includes every
nonfixed column, including opposite-direction, free and below-eligibility
coefficients, and the departing basic variable's new reduced-cost bound.
The original eligibility threshold and EPS stay unchanged. A negative ratio is
only a tolerance-qualified proposal, not an exact dual-objective progress claim.
Empty intervals and no admissible crossings remain numerical errors. The full
recalculated candidate certificate independently checks every admitted exchange.
Compensated evaluation and at most three original-basis transpose correction
steps run only after a failed dual phase check; a correction must strictly reduce
the compensated residual and pass the original residual certificate. These are
bounded numerical recovery changes, not an exact LP implementation.

All attempts, source snapshots, binaries, logs and sealed basis reports are
retained under the owned pod's
`/workspace/uor-r4/codex/m2-constructor-d22-20261010/numerical` root. Root manages
durable cloud preservation. The standalone copy retains an empty `[workspace]`
manifest stanza; the integrated vendored dependency omits it because Cargo
rejects a nested workspace. Numerical Rust sources match. Initial synthetic
compile/fixture errors and an offline arrayref setup failure are preserved in
separate attempt directories. No model training, panel scoring, legal assignment
or mechanism success is established by these numerical results.

Outstanding: corrected saved-problem completion and source-bound legal-assignment
checks; full cost/resource closeout; model integration and exact exported
candidate checks. A numerical failure remains a numerical blocker and is never
silently skipped as an infeasible branch or counted as a model negative.

The current source adds one internal zero-objective feasibility restart after
certified pivot reselection fails. It clones and refactors the existing original
basis, preserving bounds and interior fixed values. A persistent recovery flag
survives deadline interruption. All restore callers, including `fix_var` and
`add_constraint`, must restore and optimize the original objective and certify
original rows, bounds, primal and dual conditions before clearing the flag and
returning success. Resume through `initial_solve` or `reoptimize` retains the same
obligation. A second restart while this flag is active is rejected. The four new
fixtures exercise direct restoration, both interruption/resume stages, cloning
and interior fixed-variable preservation. Zero-objective simplex convergence is
not guaranteed; a later numerical error remains a blocker, not a model result.

Attempt 11 source solver SHA256 is
`2ae69de143b99529c6af42f09f7b3bb1db4b6bd52ae7cdbe89f5362f9ed80461`;
standalone source manifest
`08e022f8898707b12de10ad5523a5725cdb530142c65f9e57f77b8361072b471`;
executable `0b9bda431d226adf011fd5ce000d1d6aa8e9dc0f3e7142cb9b9e740af6e8353d`.
Its 22 synthetic tests passed in 6.549 s including compilation, the two admission
tests in 4.081 s, and release build in 4.869 s. It was launched at 18:59:55 UTC
on 2026-10-10. These are focused numerical checks; saved replay is still RUNNING
at this source checkpoint. The integrated source manifest below additionally
binds this updated documentation and `UOR-PATCH.md`; it does not rewrite the
immutable standalone execution manifest.
