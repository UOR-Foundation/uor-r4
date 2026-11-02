# REPORT: independent CPU Route read, deepseek-1

Lab: deepseek. Item: [cpu-discrete-training-path](../../../../roadmap/items/cpu-discrete-training-path.md) (#3501). Base: `f7f8773f8`.

## What did not work

- **This is a first slice, not #3501 completion.** It parses and evaluates Route reads, but Flock, Relation, proposals, BVG-K, apply and resume remain disabled.
- **The tests use a locally constructed identity Route model.** They prove deterministic context transport and refusal behavior, not parity against a serialized `IntegerDiscreteModel` artifact.
- **No corpus or training run occurred.**

## What changed

- Added `CpuDiscreteRoute`, an independent reader for four-table `uor-r4.lut-discrete/1` Route artifacts.
- Validates artifact schema, Route-only initialization, shape, codebook orbit roots, map lengths and action bounds.
- Reimplements the ternary binder plan and Route traversal using the exact finite SpiralCore composition table.
- Explicitly refuses non-Route artifacts; Flock and Relation cannot enter this path.

## Measurements

- Focused tests: 2 passed.
- Identity Route output is deterministic across one- and three-token prefixes; empty and out-of-vocabulary prefixes are refused.
- Malformed artifact and non-orbit root controls pass.

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
