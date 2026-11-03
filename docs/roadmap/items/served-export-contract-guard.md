---
id: served-export-contract-guard
title: Refuse an integer export whose snapped contract differs from its served-training config
milestone: M4
area: serving
status: ready
open_to: lab
needs_gpu: false
depends_on: []
notebook: docs/research/notebook/m4-serving.md
done_class: host
scope: [crates/uor-r4-integer/src/bin/fbs-export.rs, docs/harness/prep/served-export-contract-guard*, docs/loop/tasks/served-export-contract-guard/**, docs/research/notebook/m4-serving.md, docs/roadmap/items/served-export-contract-guard.md]
---

## Why it matters

The unsnapped served-training contract is now the one used by the one-model
end-to-end line, but `fbs-export` still defaults to snapping both Hamilton
gates and the recurrence transition. A caller can therefore pass a run's
`config.json` to `--run-config`, omit `--raw-gates --raw-transition`, and
receive a valid artifact that serves a different geometric contract from the
one the checkpoint was trained on.

The exporter already uses `--run-config` to bind the readout and retention
settings. The served contract belongs in the same fail-closed check.

## What is missing

`fbs-export --run-config CONFIG SNAPSHOT OUT` does not inspect
`train_as_served`. In the current trainer that flag means the unsnapped served
bridge (`SERVED_SNAPS`), so a true value must require both raw export flags.
Runs without the flag keep their existing behaviour: the caller may choose
either contract for comparison.

## Done when

- A run config with `train_as_served: true` is refused unless both
  `--raw-gates` and `--raw-transition` are present.
- A missing, false or non-boolean value is handled explicitly; malformed and
  mismatched values fail closed without creating the output artifact.
- Existing exports without `train_as_served: true` remain byte-identical for
  the same snapshot and flags.
- Focused tests cover all four raw-flag combinations, legacy configs and
  malformed boolean values.
- The exporter documentation, a notebook row and a REPORT record the guard.

## How to start

Read the `--run-config` path in
`crates/uor-r4-integer/src/bin/fbs-export.rs`, `LongConfig.train_as_served`
in the float trainer, and `SERVED_SNAPS` in `hamilton_sheet.rs`. Add the
strict parser as a small pure function, test it directly, then wire it before
the artifact is built.

## Cancel if

The run configuration does not identify the served contract, or a
`train_as_served` run can legitimately be exported snapped under a declared
second contract. Then this item is rejected and the contract must be made
explicit in the snapshot format instead.

## Rules that apply

- Serving remains deterministic integer (`CHARTER` §3.6).
- A mismatched export is an instrument failure, not a model negative (§4.2).
- Builds and finite host tests need no pre-registration (§4.5).

## Sources

- [claude-265 REPORT](../../loop/tasks/one-model-end-to-end/claude-265/REPORT.md)
  records the missing guard and the gap it creates.
- `crates/uor-r4-float-train/src/hamilton_sheet.rs`: `SERVED_SNAPS`.
- `crates/uor-r4-float-train/src/in_space_long.rs`: `train_as_served`.
