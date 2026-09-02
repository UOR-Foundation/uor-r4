# Fixed-checkpoint attention continuation (#1057)

- **Status:** `NOT_RUN` — frozen pre-run contract; no result is claimed.
- **Authority:** [issue #1057](https://github.com/UOR-Foundation/uor-r4/issues/1057), under #973.
- **Project value:** test remaining learning in the existing attention cell
  before coherent-R4 integration, without restarting a fit or changing its
  mechanism. The user approved this one bounded continuation.

The [#1055 result](r4_zoology_optimizer_clock_1055.md) improved development
from 12.0117% to 68.2861%, with a rising late curve, but missed its strict
target. It remains an immutable `CLOCK_MATCHED_TRANSFER_MISS`; this continuation
does not retroactively pass that result or its unrun control. #1050,
HELM-D-R4, and #1014 attention positives remain within their existing scope.
#1055 merged through [PR #1056](https://github.com/UOR-Foundation/uor-r4/pull/1056)
at `f2f2396bedeafaa8377eb67232f02e82ca8b45d9` before this successor.

## Frozen source state and continuation

| Inherited evidence | CID |
| --- | --- |
| #1055 result | `blake3:3cb810f09a118cfb70752643f5d9e60d0e42780dc6e47dc4f99224cbd69af0ee` |
| #1055 model artifact | `blake3:2a225b691ffde7b40afd41ac888c5a4449a6b3ba48c2773a2108e1e407d6f8b4` |
| Exact 3,920-update checkpoint | `blake3:e41064d74f31d45b51ecf07590ef1ae12e3f1efddce535e92b8b1b1eabd15386` |

Restore the exact model, AdamW moments/counters, cosine scheduler at block 20
with `T_max=64`, current sampler permutation/cursor, and global Torch RNG.
Preserve the first 20 history entries unchanged. Use a new evidence root and
a small sibling runner bound to the parent checkpoint, parent binding, and
new contract; never rewrite the predecessor or reset from initialization.

Keep the same 8,192 unique training rows and 1,024 development rows, exact
primary/control data, width 64, two layers, one head, V4096/T120/eight K/V,
batch 512, dropout, and optimizer policy. This is more repeated exposure,
not an experiment that increases unique data. Continue the existing cosine
schedule and evaluate after each 196-update block. Strict development top-1
`>99%` stops immediately, with no further optimizer or scheduler step.

| Work | Inherited | Maximum additional | Maximum total |
| --- | ---: | ---: | ---: |
| Source-clock blocks | 20 | 20 | 40 |
| Optimizer updates | 3,920 | 3,920 | 7,840 |
| Training queries | 16,056,320 | 16,056,320 | 32,112,640 |
| Primary-development queries | 163,840 | 163,840 | 327,680 |

Only after a primary pass, score the frozen physical binding-permuted
development input once, without fitting, and require at least a 50-percentage-
point drop. This adds at most 8,192 control queries; report artifact replay
work separately. No new data, LR/width change, parameter grid, new control,
geometry change, or automatic further extension is authorized.

## Minimal implementation and resource boundary

Reuse the unchanged cell, loaders, scorer, sampler, step, checkpoint writer,
and serializer. Do not alter historically CID-bound code or use module-global
monkeypatching. Activate only new continuation-equality, inherited/additional
accounting, restart-budget, strict-pass/control-exclusion, and predecessor-
tamper checks, plus scoped Python format/lint, claim wording, and diff
integrity. Existing #1055 checks remain valid; do not rerun old QA, C0, source
reproduction, or a CPU timing matrix.

Reuse eight intra-op Apple Accelerate CPU threads, one inter-op thread, and
one training process. The measured same-shape #1055 primary took
`1,078.926785 s`; multiplying by the declared 1.25 safety factor projects
`1,348.658482 s`, within the `1,800 s` additional-execution ceiling. Peak RSS
must remain at most `8 GiB`. CUDA and MPS are forbidden.

Keep inherited, additional, and total time/work separate. Checkpoint at least
every 16 updates and each block boundary, persisting sampler/RNG state and
accumulated additional elapsed time. Recovery must neither renew the budget
nor regenerate an unconsumed permutation. Emit flushed progress and measured
ETA. A resource/interruption stop before a complete decision is incomplete,
not a model failure.

## Decision and pre-run ledger

The metric is development top-1, currently `5,594/8,192 = 68.2861328125%`
with NLL `2.6761019229888916`, against strict `>99%`. All targets have causal
binding evidence, giving a structural ceiling of 100%, not a model guarantee.

- Primary pass plus binding drop `≥50pp`: qualify the continued artifact for
  a separately scoped inference-only coherent-R4 integration.
- Miss by total update 7,840: retain the complete curve/artifact, do not infer
  a capacity ceiling or extend/sweep again, and recommend a distinct project
  action; qualified #1050 R4 integration remains available.
- Primary pass but insufficient binding drop: a nonassociative-shortcut
  result, with no geometric promotion.
- Invalid inherited state: stop before fitting as invalid continuation
  mechanics. A resource/interruption stop remains incomplete.

Continuation binding, new focused checks, execution, conditional control,
artifact/state/result CIDs, exact work/read ledger, and fresh-process replay
are `NOT_RUN`. Completed evidence will be appended. No English generation,
reasoning, exact lowering, product promotion, or release runs in this issue.
Completion requires current-summary updates, this append-only record,
protected PR delivery, and child/#973 outcome comments. Queue acknowledgements
are transport only, not test evidence.
