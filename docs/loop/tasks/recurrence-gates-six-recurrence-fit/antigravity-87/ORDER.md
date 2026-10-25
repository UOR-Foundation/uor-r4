# ORDER: recurrence-gates-six-recurrence-fit, antigravity-87

Lab: antigravity. Item: [recurrence-gates-six-recurrence-fit](../../../../roadmap/items/recurrence-gates-six-recurrence-fit.md). Base: `4c3e13817`.

## Goal

The item's done-when reads:

> The six-recurrence fit runs after surrogate descent is shown and reports integer DEV metrics against a control.

In this unit (`antigravity-87`), following the `TEST FITNESS: FIT` and D68 `APPROVE` pre-registration sheet [`docs/harness/prep/recurrence-gates-six-recurrence-fit-2026-10-25.md`](../../../harness/prep/recurrence-gates-six-recurrence-fit-2026-10-25.md) and the completed `r-secant-surrogate-descent` (`#2886`, `G-Rec-Descent = PASS` at `1b5a6abe3`):
1. Build the standalone Rust/CUDA six-recurrence fit and D11 integer recurrence export instrument under `docs/labs/recurrence-gates-six-recurrence-fit/instrument/` (depending on `crates/uor-r4-float-train` with zero softmax/squashing/matmul under D37), implementing:
   - the 4 geometric arms (`Rec_Geometric_S3_Active`, `Rec_Geometric_U1_AbelianControl`, `Rec_Geometric_IdentityTransport_Control`, `Rec_FrozenRates_Null`) and the GPU train-split response count-table baseline (`TrainSplit_CountTable_Ref`, `1,226 / 14,287` top-1 on `ft/dev`);
   - the 4 instrument checks (`I-Standalone-CUDA-Parity`, `I-DEV-Denominator`, `I-Rate-Mobility-And-Plateau`, `I-Export-Exactness`);
   - the D11 integer recurrence exporter (16-entry dyadic bit-shift `>> (i + 1)` for $\lambda = 2^{-(i+1)}$, Q12 table lookup for $\kappa = \sqrt{1 - \lambda^2}$, 120-icosian discrete rotation carry + Hamilton gate snap, and Q12 `i32` `NativeVectorWriter` with zero float/multiply instructions) and pure-integer CPU oracle verification.
2. Run a fresh-context D67 pre-run review (`CHARTER.md` §5.1) and post the review on `#2890`.
3. Execute the pre-registered ladder (`--mode synthetic-probe`, `--mode throughput-probe`, and `--mode full` across seeds `1` and `2`, replicated across two clean `nvcc` builds A & B) on a leased RTX 5090 pod via `scripts/pod/uor-pod`, upload the sealed artifacts (`receipt.json`, `receipt-build-b.json`, `SHA256SUMS`, and exported D11 tables) to `caseyallard/uor-r4-store:results/antigravity/recurrence-gates-six-recurrence-fit-2026-10-25/`, and tear down the pod.
4. Record the complete evidence in `docs/labs/recurrence-gates-six-recurrence-fit/README.md` and `docs/labs/recurrence-gates-six-recurrence-fit/receipt.json`, update `docs/mechanisms/recurrence-gates.md` and `docs/research/notebook/float-training.md` (leading with synchronized CUDA `tokens/s`), mark `docs/roadmap/items/recurrence-gates-six-recurrence-fit.md` `status: done` (`Closes #2890`), and write `REPORT.md`.

## Cancel probe (run first)

Check on base `4c3e13817`:
1. Whether `docs/labs/recurrence-gates-six-recurrence-fit/` already exists or whether the six-recurrence fit and D11 integer DEV metrics have already been executed on `main`.
2. Whether the prerequisite `r-secant-surrogate-descent` (`#2886`) is `done` on `main` (`G-Rec-Descent = PASS`) and the pre-registration sheet `docs/harness/prep/recurrence-gates-six-recurrence-fit-2026-10-25.md` carries `TEST FITNESS: FIT`.

**Cancel probe result (executed on base `4c3e13817`, classed: instrument/pre-flight check — proceed):**
- `ls -la docs/labs/recurrence-gates-six-recurrence-fit/` returns `No such file or directory`: no six-recurrence fit instrument, GPU execution, or D11 integer DEV metrics exist on `main`.
- `r-secant-surrogate-descent` (`#2886`) is `status: done` on `main` (`G-Rec-Descent = PASS`), and `docs/harness/prep/recurrence-gates-six-recurrence-fit-2026-10-25.md` is present with `TEST FITNESS: FIT` and D68 `DECISION REVIEW: APPROVE` (`antigravity-82`). `crates/uor-r4-float-train/src/geometric_stack_native.rs` (`675a11e5…`) and `crates/uor-r4-float-train/src/geometric_stack_native_cuda.rs` (`3a279df7…`) updated on `main` when `#3232` merged; we refresh those two source SHA-256 pins in `docs/harness/prep/recurrence-gates-six-recurrence-fit-2026-10-25.md` before the D67 pre-run review.

## Files in scope

- `docs/mechanisms/recurrence-gates.md`
- `docs/harness/prep/recurrence-gates-six-recurrence-fit*`
- `docs/labs/recurrence-gates-six-recurrence-fit/**`
- `docs/loop/tasks/recurrence-gates-six-recurrence-fit/**`
- `docs/research/notebook/float-training.md`
- `docs/roadmap/items/recurrence-gates-six-recurrence-fit.md`
- `docs/roadmap/README.md`

## Done when

- Standalone instrument crate `docs/labs/recurrence-gates-six-recurrence-fit/instrument/` builds cleanly and passes host unit tests (`--no-fail-fast`) and `uor-harness loop gates --full`.
- Fresh-context D67 pre-run review (`PRE-RUN REVIEW: PASS`) is posted on `#2890`.
- Probe (a), Probe (b), and Probe (c) (`4` geometric arms $\times$ `2` seeds $\times$ `1,024` steps + `TrainSplit_CountTable_Ref`, replicated across two clean `nvcc` builds A & B) complete on a leased RTX 5090 pod via `scripts/pod/uor-pod`, with sealed artifacts archived to `caseyallard/uor-r4-store:results/antigravity/recurrence-gates-six-recurrence-fit-2026-10-25/` and the pod torn down.
- `docs/labs/recurrence-gates-six-recurrence-fit/README.md`, `docs/labs/recurrence-gates-six-recurrence-fit/receipt.json`, `docs/mechanisms/recurrence-gates.md`, `docs/research/notebook/float-training.md`, `docs/roadmap/items/recurrence-gates-six-recurrence-fit.md` (`status: done`), `docs/roadmap/README.md`, and `docs/loop/tasks/recurrence-gates-six-recurrence-fit/antigravity-87/REPORT.md` (`Closes #2890`) are committed and pass `uor-harness loop report --check` and `uor-harness loop pr-check --local`.

## Stop and ask if

- Never stop and wait for user input (`CHARTER.md` §5.1). Stop the GPU run immediately if `I-Standalone-CUDA-Parity`, `I-DEV-Denominator`, or `I-Export-Exactness` fails, or if `I-Rate-Mobility-And-Plateau` remains out of bounds after one learning-rate calibration pass, classify instrument vs. mechanism (`CHARTER.md` §4.2), and record the result.
