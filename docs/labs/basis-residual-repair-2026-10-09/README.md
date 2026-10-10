# Saved-basis residual-qualified numerical repair — October 9

## Question and fixed policy

Can a generic, scale-aware LU singularity rule cross the two captured numerical
failures from [#2132](../legal-basis-diagnosis-2026-10-09/README.md) while retaining
accurate solves against the original matrices? This is an offline constructor
boundary, not a serving operator or a new learned representation.

[Prospective claim](https://github.com/UOR-Foundation/uor-r4/issues/2030#issuecomment-6093025651).
Base main: d4628eff9362ce3c8cef7de92245b66e80ab2c7a.

For dimension n and machine epsilon u, use the fixed threshold
tau_j = 32*n*u*max(original basis column maxabs, computed residual column maxabs).
Retain the backend's 0.1 pivot preference and reject zero/nonfinite or insufficient
pivots. Global simplex EPS, all coefficients, 1,920 coordinates, 380 guards,
legal destination domains and the 4,096 branch-node bound remain unchanged.
The constants are a prospective numerical policy, not a theorem or exact-rank test.

Before accepting each fresh factorization, check every unit right-hand side in
normal and transpose orientation against the original CSC basis. Require finite
arithmetic and normwise backward error

    ||b - B*x||inf / (||B||inf*||x||inf + ||b||inf) <= 128*n*u

with the corresponding transpose norm for B^T. A zero denominator passes only
with zero residual. No iterative refinement or threshold sweep is included.
Small backward error does not establish a condition number or small forward error.
Later eta updates are outside this fresh-factor guarantee.

## Artifact and execution scope

Original saved input SHA256:
0fe91060f10c0d6107be3565f9ae0746c30e742a00d02935398f6764b84b974d.
It retains original f32 master, objective and 380 protected derivative rows from
#2101; the harness reconstructs the same ordered f64 unit rows. Zero new gradients.

Captured 440 by 440 basis SHA256 values:
- c5c73e1b9f7baf598cdda33e4a458f9208f0c943b4d471ba646eb129c495a794
- 55d8c2eadbcfc072d86a6310602184603030a318986ebeb2a2d885958f10ae05

All 235 manifest-listed payload files were authenticated after cloud-store's MD5
round-trip of codex-legal-basis-diagnosis-20261009.tar
(MD5 32dcdd9a0f2aa1092430490d754f49e2, 13,264,384 bytes).

Production constructor source is included unchanged, SHA256
f0242836d08dbadbb5659cd8f158d79c8de9420c9de2fb213de9ef149510c24b.
The isolated local backend is microlp 0.6.0 plus the corrected #2132 observer and
this explicitly identified numerical patch. An unchanged nested constructor
receipt's microlp=0.6.0 field names its base version, not unmodified upstream.
No Cargo registry or production serving dependency is modified.

First qualify both captured bases. Only after both pass and source review passes
may one unchanged saved constructor replay run. Any screened offer must still
pass the native transaction, export/reload and immediate actual-artifact checks.
A numerical rejection is not a model negative; bounded no-offer is not global
infeasibility. Native qualification has not been performed merely because a
standalone saved constructor returned an assignment.

## Resources and delivery

Projection: 60 minutes preparation, build, review, conditional execution and
protected delivery; two CPU threads, at most 6 GiB RAM, 4 GiB temporary and
512 MiB retained evidence. Disk floor: 30 GiB plus 128 MiB margin. No pods or paid
compute. This saved CPU-only numerical/native work follows the checked-in
October 9 compute clarification. The shared ledger was charged through
2026-10-10T02:54:19.500371+00:00 and extended 60 minutes under standing same-class
owner authorization before execution, preserving previous charges.

## Result

PENDING execution and review. Accepted 8/512 complete development replies and
the separate conditional 9/15 artifact are unchanged.
