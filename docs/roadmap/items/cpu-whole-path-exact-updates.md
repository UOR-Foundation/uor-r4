---
id: cpu-whole-path-exact-updates
title: Resolve the whole-path update premise and retain transaction/replay evidence
milestone: research
area: training
status: done
open_to: lab
needs_gpu: false
depends_on: [cpu-training-program, cpu-discrete-writer-core]
notebook: docs/research/notebook/cpu-training.md
done_class: host
scope: [crates/uor-r4-cpu-train/**, Cargo.toml, Cargo.lock, crates/uor-r4-harness/src/ratchet.rs, scripts/no-softmax-baseline.json, scripts/no-matmul-baseline.json, scripts/no-discrete-float-baseline.json, docs/loop/tasks/cpu-whole-path-exact-updates/**, docs/research/notebook/cpu-training.md, docs/roadmap/items/cpu-whole-path-exact-updates.md]
---

## Why it matters

The historical whole-path code records native decisions and stages exact store transactions, but its own contract and retained records say that it has no optimizer or run loop. It was initially registered as an exact-update trainer; the CPU repair needed to establish whether that premise was real before porting it.

## What is missing

Apply the cancel probe to the retained source. Separate its useful transaction, record-selection and replay mechanics from the rejected sampled M2 objective. Do not relabel replay as parameter training.

## Done when

- The source contract, batch API and retained recovery record establish whether an exact parameter-update implementation exists.
- The result separates transactional replay from sampled/objective training and names the banned dependencies that prevent direct reuse.
- The rejected M2 configuration remains a historical negative, with transaction/replay ideas retained only as future design input.
- One notebook row and REPORT record the definitive disposition.

## Cancel if

The retained source cannot be separated from a banned model objective or a dependency on `uor-r4-training`. Preserve the negative and use the current discrete/shared-vector paths instead of reviving the old model.

## Disposition

**Rejected as a CPU port target, 2 November 2026.** The retained implementation explicitly performs no optimizer, training loop or parameter update. Its decision tape also depends on normalized-exponential sampling, dense tensor products and Candle, so the cancel condition holds. The independently useful concepts are exact-store staging/rollback, bounded route-record ordering, journaled replay and fail-closed checkpoint identity. They may inform a future transaction primitive, but they do not constitute a trainer and are not imported here.

## How to start

Read `crates/uor-r4-training/src/whole_path_learning.rs`, its device companion and the archived M2 context/prototype records. Specify the bounded exact-update subset before writing code, then build fixtures from the first reproducible trace.

## Rules that apply

- Geometry only; preserve the no-squashing and no-dense-matmul bans.
- No compiled dependency on `uor-r4-training` or its historical model code.
- Exact parity is a component result, not evidence that the old whole-path model becomes useful.

## Sources

- `crates/uor-r4-training/src/whole_path_learning.rs`
- `crates/uor-r4-training/src/whole_path_device.rs`
- The archived legal-construction and M2 context/prototype records
