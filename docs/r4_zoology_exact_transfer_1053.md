# Source-positive Zoology cell on exact #1045 bytes (#1053)

- **Pre-run status:** `NOT_RUN`.
- **Authority:** [issue #1053](https://github.com/UOR-Foundation/uor-r4/issues/1053),
  under #973; follows the [positive #1050 reproduction](r4_zoology_release_reproduction_1050.md).
- **Question:** can a freshly initialized copied causal-softmax cell learn the
  exact open #1045 assignment-disjoint population with the #1050-positive
  training semantics?
- **Claim boundary:** this is an exact-byte transfer decision. Ordinary causal
  softmax attention already works at the released #1050 configuration. A miss
  here does not revoke that result or falsify attention.

## Bound inputs and one-arm contract

The source-positive reference result is
`blake3:bd16d012c01262ffb8c5197e4cf316c6fee1d722cf0700a0048386180a8122e0`.
Only its evidence identity is inherited: this run must not open its fitted
weights. The copied architecture retains the credited
[HazyResearch/Zoology ICLR24 source](https://github.com/HazyResearch/zoology/tree/de4e258784224e09909c257ff3ea040f089ed660)
and the CPU/query-only-projection adaptations documented in #1050.

The [#1045 record](r4_role_tagged_associative_curriculum_1045.md) binds the open
8,192 construction rows and 1,024 assignment-disjoint development rows. The
recorded split identity is
`blake3:d36937f974e5e96dc697b219db8a7eb448dff7192abdf88bf6b21000f58b1f48`.
The #1053 preparation must bind and validate the exact serialized population
before training; it must not generate substitute rows or reopen sealed data.
Role bytes are validated for provenance but are not model inputs.

The only primary arm is frozen as follows:

| Component | Frozen value |
| --- | --- |
| Initialization | independent seed `123`; no fitted predecessor weights |
| Cell | width 64, two blocks, one head; copied causal softmax |
| Required data-shape changes | vocabulary 4,096; context 120; eight K/V pairs |
| Learning rate | `0.00046415888336127773` (`0x1.e6b4b396428e5p-12`) |
| Optimizer | AdamW, weight decay `0.1` |
| Batch | `512` |
| Schedule | cosine, at most 64 epochs |
| DataLoader semantics | train and development shuffled; shared Torch RNG; `num_workers=0` |
| Epoch order | train, then development, then strict `accuracy > 0.99`; scheduler step only after a miss |
| Maximum training work | `8,192 × 8 × 64 = 4,194,304` query presentations |
| Maximum primary development work | `1,024 × 8 × 64 = 524,288` query presentations |

Development is open and evaluated each epoch for the declared stop decision;
it is not a sealed terminal population. There is no learning-rate sweep,
second arm, checkpoint selection by a different metric, or same-issue tuning.

## Required lifecycle repairs and focused admission

Before admission, the new implementation/dependency commitment must include
`pyproject.toml` and `uv.lock`. A restored checkpoint that already ends in a
strict passing epoch must finalize that pass without another optimizer step.
Focused tests must prove both behaviors. The #1050 implementation and evidence
remain immutable; these repairs apply only to this successor.

Admission requires the named source-cell, exact-loader, causal-prefix,
query-only loss/gradient projection, shared DataLoader RNG, dependency-CID,
and passing-checkpoint-finalization checks. A failed mechanics check returns
`INVALID_TRANSFER_MECHANICS` and stops this issue; any repair campaign requires
a separately scoped issue.

Measure one-, four-, and eight-intra-op-thread CPU plans with one training
process and zero DataLoader subprocesses. Select the measured-fastest stable
eligible plan, recording its backend, timing, RSS, and thread settings. CUDA
and MPS are forbidden. A plan is eligible only when its safety-adjusted
primary projection is at most `900 s` and peak RSS is at most `8 GiB`.
No eligible plan means `NOT_RUN_PREFLIGHT`, not a model failure. CPU work and
memory are measured, not inferred from thread count.

The `900 s` limit governs the safety-adjusted admission projection, not a
strict physical cutoff. An additional safeguard checks cumulative elapsed
time before starting each epoch; an epoch already in progress may finish
beyond that point. An early stop is `INCOMPLETE_EPOCH_BOUNDARY_BUDGET`, not a
transfer miss, and does not authorize the binding control.

## Ordered decision and falsifier

The metric to move is #1045 development top-1, previously `7,137/8,192 =
87.121582%`, to strict `>99%`. Every one of the 8,192 development query targets
is directly scoreable from its admitted causal K/V binding; the structural
ceiling is therefore 100%. The sole primary run stops on its first strict pass
or after epoch 64, subject to the epoch-boundary resource safeguard above.

Only after a primary pass, evaluate the existing exact data-level
binding-permuted development control once, using the same fitted artifact and
no retraining. This control adds exactly 8,192 query evaluations. It must
reduce top-1 by at least 50 percentage points relative to the passing primary
evaluation.

| Outcome | Terminal and resulting action |
| --- | --- |
| Primary `>99%`, binding-permuted drop `≥50pp` | `STOCK_CELL_PASSES_EXACT_BYTES`; scope a separate coherent-R4 K/V transport/replacement parity issue |
| Primary never `>99%` by epoch 64 | `STOCK_CELL_TRANSFER_MISS`; stop without R4 changes; record a serialization/population-transfer miss |
| Primary passes, control drop `<50pp` | `NONASSOCIATIVE_SHORTCUT`; reject the score as binding evidence |
| C0/lifecycle mechanics fail | `INVALID_TRANSFER_MECHANICS`; do not train or repair within this campaign |
| No CPU plan fits the frozen limits | `NOT_RUN_PREFLIGHT`; do not train |

No outcome authorizes English, generation, reasoning, R4 geometry, W8/modulo-256
softmax, integer/table lowering, product work, width/head changes, additional
controls, or broad QA in this issue. The intended later CPU-native geometric
runtime is unchanged; this offline control retains ordinary floating-point
matrix operations and softmax.

## Pre-run evidence ledger

This ledger records the state before execution; completed evidence is appended
below rather than silently replacing this contract.

| Evidence | Pre-run state |
| --- | --- |
| Exact input/population identities and read audit | `NOT_RUN` |
| Implementation/dependency and preparation CIDs | `NOT_RUN` |
| Focused loader/model/lifecycle checks | `NOT_RUN` |
| 1/4/8-thread CPU preflight and CID | `NOT_RUN` |
| Single primary arm, artifact/state/result CIDs | `NOT_RUN` |
| Binding-permuted control and CID | `NOT_RUN_PRIMARY_REQUIRED` |
| Fresh-process artifact/result verification | `NOT_RUN` |
| Python format/baseline lint, claim wording, Rust format, diff integrity | `NOT_RUN` |
| Role/H4/R4/W8/teacher/provider/sealed/future-value model reads | required zero; `NOT_RUN` |
| English, generation, reasoning, geometry, lowering, product work | `NOT_RUN_OUT_OF_SCOPE` |
| Broad workspace/release QA | `NOT_RUN_OUT_OF_SCOPE` |

Only the checks explicitly activated by #1053 are evidence for this issue.
The protected PR/queue's compatibility acknowledgements are transport only:
they perform no verification and must not be reported as test passes.
