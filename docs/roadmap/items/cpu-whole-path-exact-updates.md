---
id: cpu-whole-path-exact-updates
title: Port whole-path exact-update and recovery parity to CPU
milestone: research
area: training
status: ready
open_to: lab
needs_gpu: false
depends_on: [cpu-training-program, cpu-discrete-writer-core]
notebook: docs/research/notebook/cpu-training.md
done_class: host
scope: [crates/uor-r4-cpu-train/**, Cargo.toml, Cargo.lock, crates/uor-r4-harness/src/ratchet.rs, scripts/no-softmax-baseline.json, scripts/no-matmul-baseline.json, scripts/no-discrete-float-baseline.json, docs/loop/tasks/cpu-whole-path-exact-updates/**, docs/research/notebook/cpu-training.md, docs/roadmap/items/cpu-whole-path-exact-updates.md]
---

## Why it matters

The historical whole-path code records native decisions and applies bounded exact updates, but it is inactive evidence rather than a current trainer. Leaving it out would omit one of the project's CPU learning mechanisms from the requested all-family repair.

## What is missing

Port the decision-record ordering, legal-update candidates, acceptance checks and recovery traces into the isolated CPU crate without importing the old M2 objective or linking `uor-r4-training`. Reuse ideas only after checking each operation against the current geometry and scan rules.

## Done when

- Deterministic fixtures reproduce exact update choices and rejection reasons for accepted, tied and illegal moves.
- Apply, rollback and recovery traces match the reference semantics without host mutation during comparison.
- The new crate is an independently scanned module with no dependency edge to `uor-r4-training`.
- A negative or blocked historical arm remains labelled as such; this item does not revive the M2 context/prototype objective.
- One notebook row and REPORT distinguish exact-reference parity from training capability.

## Cancel if

The retained source cannot be separated from a banned model objective or a dependency on `uor-r4-training`. Preserve the negative and use the current discrete/shared-vector paths instead of reviving the old model.

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
