# REPORT: one-model-end-to-end, deepseek-28

Lab: deepseek. Item:
[one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md)
(#3407). Order: [ORDER.md](./ORDER.md). Laptop preparation only so far; no model,
pod or spend.

## What did not work, first

- The 1,024-step run reached outcome row 6, not the positive row 3: EOS top-1 and
  complete accepted replies remained zero.
- The final 1,024-step checkpoint regressed on free EOS stops and continuation.
- The extension therefore tests dose rather than claiming the recipe already
  works.

## What changed

- Adds the 2,048-step pre-registration and active execution ORDER.
- Parameterizes the reviewed launcher and job so the same code supports the
  legacy 1,024-step unit and this one registered extension.
- Requires a distinct run/store label and reviewed sheet/report paths.

## Measurements

No model or GPU step has run. Empirical calibration from the completed 1,024
run projects about 24 minutes of training and 45-50 minutes of pod wall time,
for roughly $0.45-$0.55 at the prior $0.563/hour offer.

## Checks run at the head

- `gate.sh`: 221 checks passed.
- `test-mutants.sh`: 84 of 84 semantic mutations caught.
- `bash -n`, ShellCheck at warning severity and `git diff --check`: passed.
- `uor-harness check --only prep`: the 2,048-step sheet has every required
  block and no shape error.
- The job rejects an unregistered 1,025-step dose, and the launcher rejects a
  2,048 dose paired with the 1,024 run label before repository or network work.
- Fresh-context FIT: **FIT GO**, no blockers, at
  `a628f4e9ee3f418e5e527a7d8ff34d3ad0bc7d5f`, runtime
  `0964a28196410a44a7d6af28194731a1003d0bb211d890b7a00fbaf2cf7153a3`,
  reviewer session `01a1b8dd-6013-7da2-9d34-53490ddef0e5`.

Not run: pod creation, remote build, training, export, panel scoring, store write
or seal readback.

## Skills used

No system-provided Codex skill applied. The repository's loop, compute and
geometry rules were used directly.

## Still open

- Fresh D67 review of the exact parameterized runtime.
- Execute and evaluate the 2,048-step dose; no automatic second extension.

## Plain summary

This unit prepares the one registered 2,048-step extension from row 6.
