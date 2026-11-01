---
id: cpu-shared-vector-train
title: Train the CPU shared-vector path with a lawful rank margin
milestone: research
area: training
status: ready
open_to: lab
needs_gpu: false
depends_on: [cpu-shared-vector-forward]
notebook: docs/research/notebook/cpu-training.md
done_class: host
scope: [crates/uor-r4-cpu-train/**, Cargo.toml, Cargo.lock, crates/uor-r4-harness/src/ratchet.rs, scripts/no-softmax-baseline.json, scripts/no-matmul-baseline.json, scripts/no-discrete-float-baseline.json, docs/loop/tasks/cpu-shared-vector-train/**, docs/research/notebook/cpu-training.md, docs/roadmap/items/cpu-shared-vector-train.md]
---

## Why it matters

This is the CPU form of direct token-to-vector training. It matters only if the forward is already parity-checked and the trainer preserves admission rather than optimizing a disconnected score.

## What is missing

Add rank-margin gradients for the shared token table and lawful route/gate/normalization parameters, deterministic update order, clipping and checkpoint/resume. Evaluate the exact integer artifact immediately after checkpoints and preserve complete-reply and fresh-phrasing controls.

## Done when

- Rank-margin learning is finite, geometric and free of squash/matmul code.
- Two deterministic resumes reproduce the same checkpoint bytes and next-step losses.
- Admission, top-1/top-8 and exact-export readings are logged alongside training loss.
- Training and evaluation use the same context length and token representation.
- A bounded pilot has a pre-registered bar, controls, seed count and stop rule before it runs.

## Cancel if

Forward parity misses, the trainer cannot preserve admission, or the only apparent gain disappears in exact integer evaluation. Record the configuration negative; do not switch to dense or softmax arithmetic.

## How to start

Use the finite forward from `cpu-shared-vector-forward` as the only model graph. Add one parameter group at a time, beginning with the shared token table under the existing rank-margin objective. Save the pre-registration before any corpus-scale step.

## Rules that apply

- Rank margin only; no implicit vocabulary normalization.
- No dense matrix multiplication, including hand loops and wrappers.
- Loss is reported beside admission, exact export and complete-reply outcomes.
- Reuse only independently reimplemented lawful patterns; the new crate has no dependency edge to `uor-r4-training` and receives its own scanned allowlist and ratchet entry.

## Sources

- `crates/uor-r4-training/src/joint_optimizer.rs` and `joint_campaign.rs` for reusable checkpoint/cursor patterns
- `crates/uor-r4-float-train/src/in_space_long.rs` for the current serving-aligned training objective
- `docs/model/pipeline-map.md`, stages 6-8
