# REPORT: exact BVG-K joint decision and batch apply, deepseek-11

Lab: deepseek. Item: [cpu-discrete-training-path](../../../../roadmap/items/cpu-discrete-training-path.md) (#3501). Base: `f1405cc10`.

## What did not work

- **This is in-memory BVG-K decision/apply.** Cursor persistence and restart/replay remain disabled.
- **No corpus, training run or language result occurred.**

## What changed

- The ORDER reserves exact BVG-K member selection, joint decision and atomic batch apply.

## Measurements

- Pending implementation and verification.

## Checks run at the head

- Pending implementation and verification.

## Skills used

No repository skill was invoked. The executed BVG-K semantics are being read directly from `bvgk_source.rs`.

## Still open

- Deterministic cursor, checkpoint and restart/resume.

## Plain summary

This branch is the isolated BVG-K decision/apply slice for #3501.
