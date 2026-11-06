---
id: fbs-query-cache
title: Cache teacher-forced FBS queries for CPU readout experiments
milestone: research
area: training
status: ready
open_to: lab
needs_gpu: false
depends_on: [one-model-end-to-end]
notebook: docs/research/notebook/float-training.md
done_class: host
scope: [crates/uor-r4-float-train/**, docs/harness/prep/fbs-query-cache*, docs/labs/fbs-query-cache/**, docs/loop/tasks/fbs-query-cache/**, docs/research/notebook/float-training.md, docs/roadmap/items/fbs-query-cache.md]
---

## Why it matters

The active FBS training loop can produce millions of exact teacher-forced query
points, but every readout comparison currently requires another GPU forward.
LexLean-R4 demonstrates a useful separation: extract the target-side trajectory
once, then fit and compare output readouts independently.

This item turns that idea into a checked UOR-R4 instrument. It does not train the
trunk and does not change the optimizer. The first cache is produced by the
existing captured scorer path; later centroid, shuffled, address-margin and
rank-aware output experiments can run on the workstation CPU against the same
fixed queries and panel.

## Cancel if

The current scorer cannot expose query rows without changing the forward or
holding the full TRAIN cache in memory, or a proposed cache loses enough
identity to make the frozen original readout unreproducible.

## What is missing

The cache/evaluator instrument is being built in `deepseek-1`. It becomes useful
research evidence only after one sealed current-FBS checkpoint is captured on
the selected TRAIN and held-out panels, the original decisions reproduce, and a
pre-registered centroid arm beats both the original and the equal-count shuffled
control. No GPU run is part of this build unit.

Address-margin and rank-aware readouts are later candidates against the same
cache. They are not implemented or authorized by this item. Training the trunk
or changing writer admission remains outside it.

## Done when

- `--score-rows` optionally writes a fresh cache while its existing outputs and
  decisions stay byte-identical.
- Every split records exact checkpoint, tokenizer, corpus, table and row
  identities; malformed, reordered, overwritten or mixed caches are refused.
- A CPU evaluator verifies the cache, reproduces the original nearest decisions
  exactly and reports top-1/top-8 for a centroid table and a deterministic
  matched control on the same panel. The development default fits a token only
  after eight observations.
- The manifest is sealed and records the checkpoint inventory and the run
  `config.json` used to determine query space.
- The cache preserves the active memory and served-query choice, and does not
  silently patch the shared input table.
- Focused tests, formatting, Clippy and ratchets pass.

## How to start

Read `in_space_long::row_picks`, `score_rows`, the checkpoint loader and
`train-in-space --score-rows`. Extend the existing query callback only; do not
modify `step_gradient`, `run_long`, the optimizer or decoded replies.

## Rules that apply

- **Geometry only** (CHARTER §3.1–§3.3): the cache records the active FBS query
  and the table it read. It does not replace the nearest readout with a dense or
  transformer path.
- **Serving stays integer** (§3.6): the CPU centroid evaluator is an offline
  development instrument. It does not change exported artifacts or serving
  defaults.
- **Claims** (§4.6): a centroid, rank or margin result is reported against the
  frozen original and a matched control on the same panel. A code check is not a
  capability claim.
- **No hidden retuning:** observation threshold, splits, context and memory mode
  are recorded before the panel result is opened.

## Sources

- [Issue #3801](https://github.com/UOR-Foundation/uor-r4/issues/3801)
- [one-model-end-to-end](one-model-end-to-end.md)
- LexLean-R4 `gpu/train_uorlut_v8.rs`'s teacher-forced mean-row fit, used as a
  design lead only.
