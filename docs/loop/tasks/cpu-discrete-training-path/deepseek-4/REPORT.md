# REPORT: independent CPU Relation read, deepseek-4

Lab: deepseek. Item: [cpu-discrete-training-path](../../../../roadmap/items/cpu-discrete-training-path.md) (#3501). Base: `4706fb2ee`.

## What did not work

- **This unit is in progress.** No Relation parity, refusal control or benchmark result is claimed until its command runs at the implementation head.

## What changed

- The ORDER reserves a Relation-only read slice. Proposals, apply, BVG-K and checkpoint resume remain disabled.

## Measurements

- Pending implementation and verification.

## Checks run at the head

- Pending implementation and verification.

## Skills used

No repository skill was invoked. The executed Relation semantics and finite SpiralCore tables are being read directly from the repository.

## Still open

- Relation parsing, multi-sweep relation score parity and refusal controls.
- Counterfactual scoring, BVG-K, apply and checkpoint resume.

## Plain summary

This branch is the isolated Relation-read slice for #3501. It does not yet claim a working read.
