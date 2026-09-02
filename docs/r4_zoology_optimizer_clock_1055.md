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
