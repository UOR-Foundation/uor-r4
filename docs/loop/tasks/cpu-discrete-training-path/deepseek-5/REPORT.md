# REPORT: exact discrete proposal chooser, deepseek-5

Lab: deepseek. Item: [cpu-discrete-training-path](../../../../roadmap/items/cpu-discrete-training-path.md) (#3501). Base: `d65054c4d`.

## What did not work

- **This unit is in progress.** No chooser parity, refusal control or benchmark result is claimed until its command runs at the implementation head.

## What changed

- The ORDER reserves the read-only 16-lane chooser. Model mutation, virtual writer scoring, BVG-K apply and resume remain disabled.

## Measurements

- Pending implementation and verification.

## Checks run at the head

- Pending implementation and verification.

## Skills used

No repository skill was invoked. The executed chooser semantics are being read directly from `discrete.cu`.

## Still open

- Exact chooser parity and refusal controls.
- Virtual writer scoring, apply, BVG-K and deterministic resume.

## Plain summary

This branch is the isolated proposal-chooser slice for #3501. It does not yet claim a working chooser.
