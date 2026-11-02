# REPORT: independent CPU Route read, deepseek-1

Lab: deepseek. Item: [cpu-discrete-training-path](../../../../roadmap/items/cpu-discrete-training-path.md) (#3501). Base: `f7f8773f8`.

## What did not work

- **This is a first slice, not #3501 completion.** It parses and evaluates Route reads, but Flock, Relation, proposals, BVG-K, apply and resume remain disabled.
- **No release Route benchmark is included yet.**
- **No corpus or training run occurred.**

## What changed

- Added `CpuDiscreteRoute`, an independent reader for four-table `uor-r4.lut-discrete/1` Route artifacts.
- Validates artifact schema, Route-only initialization, shape, codebook orbit roots, map lengths and action bounds.
- Validates all four required tables, kinds, lengths, duplicate/missing names, zero Route gates, expected stage counts and checked section bounds.
- Reimplements the ternary binder plan and Route traversal using the exact finite SpiralCore composition table.
- A serialized two-block, two-stage artifact with nonzero Route actions matches `IntegerDiscreteModel` exactly on one- and multi-token prefixes.
- Explicitly refuses non-Route artifacts; Flock and Relation cannot enter this path.

## Measurements

- Focused tests: 3 passed.
- Serialized nonzero-action Route reads match `IntegerDiscreteModel` on four prefixes.
- Identity Route output is deterministic; empty and out-of-vocabulary prefixes are refused.
- Malformed artifact, non-orbit root, structure and bounds controls pass.

## Checks run at the head

- Full tests, Clippy, formatting, ratchets, PR checks and loop gates are recorded after they run.

## Skills used

No repository skill was invoked. The executed Route semantics in `discrete_d11.rs` and the finite SpiralCore table were read directly.

## Still open

- Serialized Route parity against `IntegerDiscreteModel`.
- Flock and Relation reads.
- Counterfactual scoring, BVG-K, apply and checkpoint resume.

## Plain summary

The CPU crate now has a guarded independent Route-read skeleton, but #3501 remains open and no training capability is claimed.
