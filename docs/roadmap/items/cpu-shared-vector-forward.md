---
id: cpu-shared-vector-forward
title: Build CPU forward parity for the shared token/vector path
milestone: research
area: training
status: done
open_to: lab
needs_gpu: false
depends_on: [cpu-training-program]
notebook: docs/research/notebook/cpu-training.md
done_class: host
scope: [crates/uor-r4-cpu-train/**, crates/uor-r4-integer/src/stack/session.rs, Cargo.toml, Cargo.lock, docs/loop/tasks/cpu-shared-vector-forward/**, docs/research/notebook/cpu-training.md, docs/roadmap/items/cpu-shared-vector-forward.md]
---

## Why it matters

The owner's long-standing target is direct token-to-vector training in the same representation that serves: token id, shared vector table, geometric trunk, vector point, token. The old joint learner used this shape on CPU but failed on model quality and was later made non-buildable by the dense/softmax rules.

## What is missing

Implement the current serving-aligned forward in the isolated CPU crate: tied 4,096 x 256 token lookup, SpiralCore/FBS route gather and signed-axis action, Hamilton gate, RMS normalization, Hamming read and nearest/writer output. Start with finite parity fixtures; do not add gradients.

## Done when

- Token lookup, routing, transport, normalization and readout match the current integer reference on deterministic fixtures.
- The same token table supplies input, carry and output identities.
- No softmax, squashing or dense matrix multiplication is present.
- The crate reports forward throughput and peak RSS on the Ryzen 9 9950X without claiming a training result.
- One notebook row and a REPORT name every unported operation.

## Cancel if

The current serving artifact lacks enough trace information to establish parity. Build the missing trace or wait for its owning item rather than declaring approximate equality.

## How to start

Read the current FBS serving session and its integer kernels, plus the float forward only as a semantic reference. Select deterministic fixtures whose complete intermediate vectors and final token are available. Write a failing parity test before porting each stage.

## Rules that apply

- One shared token/writer table serves input, carry and output.
- No silent CUDA fallback; the CPU module declares its substrate.
- Finite forward parity is not a training or language result.

## Sources

- `crates/uor-r4-integer/src/stack/session.rs`
- `crates/uor-r4-integer/src/stack/kernels.rs`
- `crates/uor-r4-float-train/src/geometric_stack_native.rs`
- `crates/uor-r4-float-train/src/flock_read_hamming.rs`

## Resolution

Completed across `deepseek-1` through `deepseek-4` and merged through PRs #3513, #3515, #3516 and #3517. The supported snapped, rotated, decay-rate, recurrence-only, no-pointer FBS mode matches serving on complete Q12 query/token/cell outputs, including Hamming copy/type reads, `/7` opening/reply memory and `/8` point-key fallback. Retention-rate, unrotated, raw-gate/transition, pointer and non-recurrence modes remain explicitly refused. Trained FBS artifact/query-panel parity is outside this item's deterministic-forward criteria and remains a later evaluation/pilot prerequisite.
