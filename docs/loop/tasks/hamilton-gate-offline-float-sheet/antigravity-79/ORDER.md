# ORDER: hamilton-gate-offline-float-sheet, antigravity-79

Lab: antigravity. Item: [hamilton-gate-offline-float-sheet](../../../../roadmap/items/hamilton-gate-offline-float-sheet.md). Base: `bf92c9228`.

## Goal

The item's done-when reads:

> An offline-float (D37) sheet reports a language fit on the integer DEV metrics and an export gap, and states whether the gate suppressed signal.

## Cancel probe (run first)

Inspected `docs/roadmap/items/hamilton-gate-offline-float-sheet.md`, `docs/mechanisms/hamilton-gate.md`, `docs/labs/repo-renormalization-2026-10-19/audit/mechanisms/hamilton-gate.json`, `docs/labs/r-hamilton-gate-suppression/README.md`, `docs/labs/r-secant-surrogate-descent/README.md`, `crates/uor-r4-float-train/src/geometric_stack_native.rs`, `crates/uor-r4-float-train/src/native_vector_writer.rs`, and `crates/uor-r4-integer/src/h4_classifier.rs`.

- **Cancel probe result (`instrument`):** Both prerequisites (`float-training-module` `#2823` and `r-secant-surrogate-descent` `#2886`) are `done` on `main`, and the pinned `mw-balp/train` and `ft/dev` (`14,287` eligible response targets across `400` documents) inputs exist in `caseyallard/uor-r4-store:w2-inputs/2026-10-25/`. However, no pre-registration sheet `docs/harness/prep/hamilton-gate-offline-float-sheet*` and no runner in `crates/uor-r4-float-train` exist yet to train and evaluate `Gate::Hamilton` against matched gate controls, measure the 120-element icosian-snap (`H4_ROOT_COEFFICIENTS`) + integer Q12 fixed-point `NativeVectorWriter` export gap on the `14,287` DEV rows, and measure empirical gate signal suppression (`|g|_2`, `rho(g,u) = ||out||_2 / ||u||_2`, backward scale `1/r`, and downstream projection norm ratio `||P_down(q (x) u)||_2 / ||P_down(u)||_2`). Proceed to author the prep sheet and runner, obtain fresh-context `TEST FITNESS: FIT`, D68, and D67 pre-run reviews, execute the two-seed GPU run on a leased RTX 5090 pod, and record the results.

## Files in scope

- `crates/uor-r4-float-train/**`
- `crates/uor-r4-integer/**`
- `docs/mechanisms/hamilton-gate.md`
- `docs/harness/prep/hamilton-gate-offline-float-sheet*`
- `docs/labs/hamilton-gate-offline-float-sheet/**`
- `docs/loop/tasks/hamilton-gate-offline-float-sheet/**`
- `docs/research/notebook/float-training.md`
- `docs/roadmap/items/hamilton-gate-offline-float-sheet.md`
- `docs/roadmap/README.md`

## Done when

1. One-page pre-registration sheet `docs/harness/prep/hamilton-gate-offline-float-sheet-2026-10-25.md` is authored and passes fresh-context `TEST FITNESS: FIT`, D68 decision review, and D67 pre-run review (`CHARTER.md` §4.5, §5.1).
2. `crates/uor-r4-float-train` implements the offline-float Hamilton gate sheet (`src/hamilton_sheet.rs` and `src/bin/hamilton-gate-sheet.rs`) with:
   - `Hamilton_Active` (`Gate::Hamilton`, $q = g/\sqrt{\|g\|_2^2 + 10^{-12}}$, $\text{out} = q \otimes u$),
   - `Hamilton_IcosianHard` (hard forward 120-element icosian snap $q_{\text{snap}} \in \mathcal{I}_{120}$ with straight-through unit-quaternion adjoint),
   - `IdentityGate_Control` ($q \equiv (1, 0, 0, 0)$, $\text{out} = u$), and
   - `RandomGate_Null` (deterministic norm-preserving random unit-quaternion gate rotation),
   trained across seeds `1` and `2` on `mw-balp/train` and evaluated on the `14,287`-row `ft/dev` split in both continuous `f32` and D11-compatible integer export (120-icosian snap + Q12 `i32` L1 `NativeVectorWriter`), plus the `mw-balp/train` response-target unigram count table (`1,226 / 14,287` top-1, `4,060 / 14,287` top-8).
3. Two independent `nvcc` builds execute the probe ladder and full 2-seed run on a leased `uor-pod` RTX 5090, artifacts are archived to `caseyallard/uor-r4-store:results/antigravity/hamilton-gate-offline-float-sheet-2026-10-25/`, the pod is torn down, and `docs/labs/hamilton-gate-offline-float-sheet/{README.md,receipt.json}`, `docs/mechanisms/hamilton-gate.md`, `docs/research/notebook/float-training.md`, `docs/roadmap/items/hamilton-gate-offline-float-sheet.md` (`status: done`), and `REPORT.md` are committed.

## Stop and ask if

- Any instrument check (`I-Finite-CUDA`, `I-DEV-Denominator`, `I-Icosian-Closure`) fails and cannot be repaired inside `crates/uor-r4-float-train`.
- Any edit outside declared scope or any `CHARTER.md` §3 / GPU cap change is required.
