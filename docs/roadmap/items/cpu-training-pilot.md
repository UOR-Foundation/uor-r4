---
id: cpu-training-pilot
title: Run and grade the bounded CPU training pilot
milestone: research
area: training
status: ready
open_to: lab
needs_gpu: false
depends_on: [cpu-discrete-training-path, cpu-shared-vector-train, cpu-quantization-exact-memory, cpu-whole-path-exact-updates]
notebook: docs/research/notebook/cpu-training.md
done_class: host
scope: [crates/uor-r4-cpu-train/**, Cargo.toml, Cargo.lock, docs/harness/prep/cpu-training-pilot*, docs/loop/tasks/cpu-training-pilot/**, docs/research/notebook/cpu-training.md, docs/roadmap/items/cpu-training-pilot.md]
---

## Why it matters

Component speed does not show whether CPU training can improve a model at useful scale. The workstation must be measured under a bounded, pre-registered pilot with the same evaluator as the GPU path.

## What is missing

Write a pre-registration with a non-author FIT and pre-run review; run at least two seeds or 200 fresh rows; record context-tokens/s, target visits/s, step time, peak RSS, wall time and resume determinism; grade both complete replies and fresh phrasing against headline-preserving controls.

## Done when

- The pre-registration exists before compute and names pass, fail and stop outcomes.
- The pilot runs on the named Ryzen 9 9950X and records its source, binary, hardware, OS and checksums.
- Training loss is never the only outcome; admission, top-1/top-8, exact export, complete replies and fresh phrasing are reported.
- Every historical training family has a final supported, superseded or rejected disposition linked to evidence.
- The REPORT states explicitly whether CPU training is useful for development, ready for a named qualification path, or rejected for the tested configurations.

## Cancel if

Any dependency still fails parity or deterministic resume. A fast but unverified trainer cannot enter the pilot.

## How to start

First collect the child REPORTs and rerun their parity/resume checks at the pilot head. Write the pre-registration with explicit pass, fail and stop outcomes and the machine identity. Import only the selected bounded corpus slice after the sheet is FIT.

## Rules that apply

- Pre-register before CPU research compute.
- Name the Ryzen 9 9950X and exact source/binary identity.
- Report capability and cost together; no loss-only success.

## Sources

- `docs/research/cpu-training-program.md`
- The six preceding CPU component items and their REPORTs
- The current evaluator and panel selected by the pre-registration
