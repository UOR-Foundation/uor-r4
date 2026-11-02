# REPORT: exact single-coordinate proposal apply, deepseek-9

Lab: deepseek. Item: [cpu-discrete-training-path](../../../../roadmap/items/cpu-discrete-training-path.md) (#3501). Base: `4fdec6487`.

## What did not work

- **This is single-coordinate atomic-free apply.** BVG-K joint application and deterministic restart/resume remain disabled.
- **No corpus, training run or language result occurred.**

## What changed

- The ORDER reserves exact epoch, mode, inventory and strict-improvement apply.

## Measurements

- Pending implementation and verification.

## Checks run at the head

- Pending implementation and verification.

## Skills used

No repository skill was invoked. The executed apply semantics are being read directly from `discrete.cu`.

## Still open

- BVG-K joint application.
- Deterministic cursor, checkpoint and restart/resume.

## Plain summary

This branch is the isolated single-coordinate apply slice for #3501.
