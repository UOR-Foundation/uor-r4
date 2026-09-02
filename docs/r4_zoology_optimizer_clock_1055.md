# Exact-byte attention transfer: optimizer-clock correction (#1055)

- **Status:** `NOT_RUN` — frozen pre-run contract.
- **Authority:** [issue #1055](https://github.com/UOR-Foundation/uor-r4/issues/1055), under #973.
- **Project value:** give the already-working copied attention cell a
  source-comparable optimizer clock on our exact data before coherent-R4
  transport/replacement. This is not a new attention mechanism.

The [#1053 transfer](r4_zoology_exact_transfer_1053.md) exhausted its cosine
schedule after 1,024 updates; the [#1050 positive](r4_zoology_release_reproduction_1050.md)
needed 3,920. #1050, HELM-D-R4 ordinary attention, and #1014 load-bearing
attention remain established within their existing claim boundaries.

## Frozen single arm

Bind #1053 result
`blake3:e2d1deb55a4612015ba924a94051beacd517f3c062c714c4972ba954f57621a1`,
its preparation, exact primary/control tensors, and source/dependency closure.
Read no fitted predecessor weights. Reuse unchanged credited model, exact data
adapters, serializer, scorer, and source-cell C0 evidence; do not rerun its
source reproduction or overfit campaign.

| Component | Contract |
| --- | --- |
| Initialization/cell | fresh seed 123; width 64; two layers; one head |
| Data/model shape | unchanged #1053: 8,192 train and 1,024 development rows; V4096/T120; eight K/V pairs |
| Batch/optimizer | 512; AdamW weight decay 0.1; unchanged source dropout |
| Initial learning rate | `0.00046415888336127773` |
| Optimizer clock | at most 20 blocks × 196 updates = 3,920 updates |
| Scheduler | cosine `T_max=64`; advance once after a failed block, not per data pass |
| Development | existing split after each 196-update block; strict top-1 `>0.99` stops |
| Maximum primary work | 16,056,320 training queries; 163,840 development queries |

Cycle complete shuffled permutations of the unchanged 8,192 training rows.
Retain the current permutation and cursor across block boundaries and resume;
never discard its unconsumed rows. Training/development shuffles and dropout
share the global Torch RNG. A passing block permits no further optimizer or
scheduler step. A saved pass finalizes immediately on restart.

This matches optimizer opportunities and evaluation/scheduler cadence, not
source ordering, exposure, or arithmetic. #1050 ends each 100,000-row source
epoch with a 160-row batch; every #1055 batch has 512 rows. Eight rather than
four queries per row also increases query work. Equal updates are not equal
query dose, data coverage, or wall time.

Only after a primary pass, score the frozen physical binding-permuted
development input once without fitting; require a top-1 drop of at least
50 percentage points. That conditional evaluation adds 8,192 queries. No
other model controls, probes, sweeps, or same-issue tuning are authorized.

## Minimal implementation and CPU admission

Add only the clocked trainer, lifecycle/implementation bindings, and focused
schedule/order/cycling/RNG/resume/control-boundary checks. Bind `pyproject.toml`
and `uv.lock`; preserve historically CID-bound #1050/#1053 code. Activate only
those focused checks, Python format/baseline lint, claim wording, and diff
integrity. Broad Rust/workspace/BDD/WASM/release QA stays dormant.

Measure stable one-, four-, and eight-intra-op-thread CPU plans on the fixed
batch shape and select the fastest eligible plan. Require safety-adjusted
full-run projection `≤1,800 s` and measured peak RSS `≤8 GiB`. Reuse passed
#1053 deterministic mechanics; do not add a cross-backend campaign. Use one
training process, no multiprocessing of dependent updates, and no CUDA/MPS.
Emit flushed progress, completed/remaining updates, and measured ETA.
Checkpoint at least every 16 updates and every block boundary, including the
current permutation/cursor and global RNG. A resource stop is incomplete,
not model failure.

## Decision

Metric: exact-data development top-1, currently `12.01171875%`, target strict
`>99%`. All 8,192 targets have admitted causal K/V evidence: the structural
ceiling is 100%. The instrument is focused clock/resume checks plus measured
CPU admission; no broader scientific claim is inferred from passing it.

| Outcome | Terminal and next action |
| --- | --- |
| Primary `>99%` and binding drop `≥50pp` | `CLOCK_MATCHED_TRANSFER_PASSES`; scope separate coherent-R4 transport/replacement |
| Primary miss by 3,920 updates | `CLOCK_MATCHED_TRANSFER_MISS`; preserve curve/model/positives and recommend a distinct next action, not another LR/width/epoch sweep |
| Primary passes, binding drop misses | `NONASSOCIATIVE_SHORTCUT` |
| No eligible CPU plan | `NOT_RUN_PREFLIGHT` |
| Invalid clock/lifecycle mechanics | `INVALID_CLOCK_MECHANICS` |
| Resource/interruption without a complete decision | `INCOMPLETE`, never attention failure |

## Pre-run ledger and scope boundary

Input/dependency/preparation bindings, focused clock tests, CPU admission,
primary/control execution, artifact/state/result CIDs, and fresh-process replay
are `NOT_RUN`. Completed evidence and exact work/read ledgers will be appended.
The binding control is conditional; its prepared input is not a model result.
Role/geometry/future-value model inputs, teacher/provider/fitted-predecessor
weights, and hidden/sealed payload reads must remain zero.

No R4 code, new corpus, generation, reasoning, W8/lowering, product, or release
work runs here. Scope ends with one terminal result, concise current-summary
updates, this append-only record, protected PR delivery, and child/#973 closure
comments. Queue acknowledgements are transport only, not test evidence.

## Resource-only pause after preflight (2026-09-02)

The implementation is frozen at `47ebdabc`. The 15 focused clock/lifecycle
tests passed in `0.982 s`; code formatting, scoped baseline Python lint, claim
wording, and diff integrity also passed. The inherited source-cell C0 was
reused without a new reproduction or overfit campaign.

Preparation CID is
`blake3:935363a19038bb9573cda29c29179f98b4b2a80f4d2e0ac9b64b46ae399f5916`;
implementation/dependency CID is
`blake3:bf45a18a2b4ed1aee607220f8d0331c32254bb43e94c2e1d7bece70d33634de3`.
The `46.456571 s` CPU preflight produced CID
`blake3:a0f88c9328505f56860a3e2cfdb947be83fb5d5877d9acac2e9ab3f4ff92ffd5`.
It measured two eight-batch training units per Apple Accelerate/OpenMP plan:

| Threads | Training units (s) | Development units (s) | Stable | Peak RSS (bytes) | Safety-adjusted full projection (s) |
| ---: | --- | --- | :---: | ---: | ---: |
| 1 | 3.376828 / 3.319727 | 0.467915 / 0.457117 | yes | 1,162,067,968 | 2,080.005000 |
| 4 | 2.378479 / 5.017547 | 0.320396 / 0.518029 | no | 1,197,031,424 | 3,086.198205 |
| 8 | 6.401626 / 10.149635 | 0.703424 / 0.637655 | no | 958,251,008 | 6,234.236962 |

All three plans fit memory but exceeded the frozen `1,800 s` projection
ceiling; four and eight threads also failed timing stability. Their repeated
loss sequences were deterministic within each plan. No plan was admitted
(`selected=null`, `passed=false`): this is `NOT_RUN_PREFLIGHT`, not an
attention or model negative.

Primary optimizer updates, primary development evaluations, and binding-control
evaluations are all **zero**. Disposable timing work is not a primary fit.
No fitted #1055 artifact exists, so artifact replay is unavailable. The issue
remains open and execution pending. The next proposed action is to resolve
the active Studio/browser workload and obtain direction for timing-only
remeasurement; no new attempt or model change is recorded here. The existing
#1050, HELM-D-R4, and #1014 attention evidence remains unchanged.
