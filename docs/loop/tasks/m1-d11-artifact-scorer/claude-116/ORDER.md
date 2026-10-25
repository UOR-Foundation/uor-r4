# ORDER: m1-d11-artifact-scorer, claude-116

Lab: claude. Item: [m1-d11-artifact-scorer](../../../../roadmap/items/m1-d11-artifact-scorer.md). Base: `ffe0dcf8b`.

## Goal

The item's done-when reads:

> All three are built as a detached instrument in `docs/labs/m1-d11-artifact-scorer/instrument/`: its own `[workspace]`, with path dependencies on `uor-r4-discrete` and `uor-r4-integer` and no edit to either. That keeps `forbid(unsafe_code)` in the serving crate (§3.13) and takes no crate lock.
> - **`export`** turns a `DriverCheckpoint` JSON into a `UORLUT01` file. The file is byte-identical to `export_d11` on the same snapshot, and `IntegerDiscreteModel::from_bytes` accepts it.
> - **`score`** runs a CUDA, clean-room port of `IntegerDiscreteModel::evaluate_prefix` over the registered DEV panel. The rows come from `uor_r4_discrete`'s own corpus windows, so they are the same rows `eval-corpus` scores. It writes per-row prediction, top-8 and admission, and integer top-1, top-8 and admitted totals.
>   - The kernel is built ahead of time by nvcc, with no NVRTC, no float and no squashing (§3.2–§3.4).
>   - A finite test shows the CUDA results bit-identical to `evaluate_prefix` on fixtures, and fails when the port is set aside.
>   - A GPU qualification on one RTX 5090 shows parity on a real exported checkpoint. The A8 FrozenZero `continuation-05248.json`, inside `results/antigravity/a8-full-dose-rerun-2026-10-25/a8-full-dose-rerun-20261025.tar.gz` (store, with its `SHA256SUMS`), qualifies the instrument only, and is never step 6's reported measurement.
> - **`toks`** times `IntegerDiscreteModel` serving on the laptop by the step-6 protocol, and records the hardware.
> - **A note on #2839** maps each tool line in the step-6 sheet to the instrument command that replaces it. Those lines are `:32` (the exporter "merged in #2998"), `:81` (`discrete-stack export-d11`) and `:82` (the `eval-corpus`/`IntegerDiscreteModel` comparison). This item does not edit that sheet.

## Cancel probe (run first)

To fill in.

## Files in scope

- `docs/labs/m1-d11-artifact-scorer/**`
- `docs/harness/prep/m1-d11-artifact-scorer*`
- `docs/loop/tasks/m1-d11-artifact-scorer/**`
- `docs/research/notebook/m4-serving.md`

## Done when

To fill in.

## Stop and ask if

To fill in.
