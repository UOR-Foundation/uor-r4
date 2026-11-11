# ORDER: one-model-end-to-end, deepseek-276 (E8 first-test sheet)

Lab: deepseek. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md)
(#3407). Base: `d792c77f2` (E8 draft merged with current `main`).

## Goal

Prepare the E8 address memory's first registered test on T0's memorize set:
the no-memory baseline already recorded, and four address arms at two seeds.
This unit writes the pre-registration only. It runs no GPU, changes no model
code, and does not touch Claude's stage-3 branch or pod.

## Done when

1. The sheet names T0's exact corpus, transform model, recipe, seeds, arms,
   steps, scoring commands and sealed store paths.
2. The eight CPU/GPU preflight tests for the E8 tensor path and the decode-cache
   parity are named, with an expected 18-test total.
3. T0's recorded negative is the no-memory baseline; the four arms use identical
   commands except `--address-memory`.
4. The sheet fixes bars, a top-down outcome map, cost/time caps, instrument
   checks and what each E8/control combination means.
5. `cargo test -p uor-r4-harness --lib -- the_real_sheets` and the sheet's
   TEST FITNESS line are present.

## Cancel if

- T0's input hashes, trainer pin or scoring commands cannot be reproduced;
- the E8 branch requires a production change before a test can be registered;
- the registered test would require more than four pods or $8/h.

## Files in scope

- `docs/loop/tasks/one-model-end-to-end/deepseek-276/**`
- `docs/harness/prep/one-model-end-to-end-e8-address-2026-11-11.md`

Size budget: 700 changed lines.

## Stop and report

Stop after the sheet, its shape checks and the host/preflight evidence are
recorded. Do not launch a pod. A non-author FIT review and owner run approval
remain separate.
