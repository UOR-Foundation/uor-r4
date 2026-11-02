# ORDER: wire training through the served graph, deepseek-4

Lab: deepseek. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md). Base: `2f8f665c5`.

## Goal

Make `train-in-space --train-as-served 1` run the complete tested float
surrogate for the current supported FBS path: Q16 residual values, dynamic i16
MLP activations, D11 gate/transition snaps and the Q12 scored query. The wrapper
must cover both the training forward and its backward, while parameter export
rounding continues to bind the optimizer step and saved checkpoint.

This unit wires the graph and adds refusal/parity controls. It does not claim a
training result or replace integer serving.

## Cancel probe

Check whether `run_long` already wraps `batch_loss` with
`with_served_surrogate_training`, whether the MLP applies
`activation_quantize16` in the hidden path, and whether `batch_loss` applies
`q12_query`. Existing primitive tests are not integration evidence.

## Scope

- `crates/uor-r4-float-train/src/hamilton_sheet.rs`
- `crates/uor-r4-float-train/src/geometric_stack_native.rs`
- `crates/uor-r4-float-train/src/in_space_long.rs`
- `docs/loop/tasks/one-model-end-to-end/deepseek-4/**`
- `docs/research/notebook/float-training.md`
- `docs/roadmap/items/one-model-end-to-end.md`

## Required design

- In one scoped closure, enable Q16 residual, dynamic i16 activation, D11 snaps
  and Q12 query bridges for both the loss forward and `backward`.
- Keep `ServedValues` parameter rounding and restoration outside that closure;
  gradients update float masters, while saved values remain export-rounded.
- In served mode, checkpoint evaluation and generation use the same D11/Q16/i16
  graph and Q12 query path rather than the plain float gate path.
- Refuse `turn_memory_dropout` and `reply_read_train` in served mode until their
  train-only row selection has a defined served counterpart.
- Restore every thread-local flag after success, error and panic.

## Done when

- The real `run_long` training closure uses `with_served_surrogate_training`
  when `train_as_served` is on and the ordinary gate mode when it is off.
- The geometric MLP applies the dynamic i16 quantizer in that mode, and scored
  queries use the Q12 bridge.
- Exact checkpoint evaluation, panel loss and nearest decoding use the same
  mode, so checkpoint selection is not scored through a different graph.
- CPU tests verify boundary activation, refusal cases and state restoration.
- A CUDA-gated finite test verifies the integrated graph produces finite loss
  and gradients, with values restored before the optimizer. It may be
  `NOT_RUN` locally under D24/D29 and must be reported as such.
- Focused tests, the full float library suite and changed-file formatting pass;
  pre-existing crate-wide Clippy debt is recorded rather than rewritten.
