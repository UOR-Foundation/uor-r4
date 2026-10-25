# ORDER: r-secant-surrogate-descent, antigravity-77

Lab: antigravity. Item: [r-secant-surrogate-descent](../../../../roadmap/items/r-secant-surrogate-descent.md). Base: `e1e3a3fbd7d79f92f4d6c898382e5108cbf4bb41`.

## Goal

Execute the registered two-seed 256-step secant surrogate descent check for `RecGates::Geometric` (`rate_select_bwd`) and `QuaternionWeights::SpiralcoreFbs` (`spiralcore_fbs_bwd`) in `crates/uor-r4-float-train` on a leased RTX 5090 pod (`#2886`), meeting the item's done-when:

> A recorded descent or no-descent result for each of the recurrence gates and SpiralCore/FBS, with instrument versus mechanism classified.

## Cancel probe (run first)

Ran `uor-harness check` and inspected `docs/harness/prep/r-secant-surrogate-descent-2026-10-25.md`, `crates/uor-r4-float-train/src/native_vector_writer.rs:394`, and `w2-inputs/2026-10-25/SHA256SUMS` in `caseyallard/uor-r4-store`. Identified three pre-run instrument/sheet discrepancies from drafting the sheet before `#2823` was built:
1. `uor-harness check` reports `error prep.sheet_shape [docs/harness/prep/r-secant-surrogate-descent-2026-10-25.md]: missing block(s): Stop rule` because Section 6 used `Abort criteria` instead of `Stop rule`.
2. `NativeVectorWriter::margin_rows` (`native_vector_writer.rs:394`) enforces `1..=8` rivals (`kept_cells = 8`), rejecting `rivals: 64`.
3. The sealed training corpus in `caseyallard/uor-r4-store` is `w2-inputs/2026-10-25/driver-data/mw-balp/train/tokens.u16` (`3dee858f84b750f507ceb3dc3f393711741950954735efaaf61bf704c017a4c8`) and `response_mask.u8` (`b5e7352d67d6883a833155a5fd397a03f6d4f1b2649e3d4ec3e2ea7d95b949e3`), tokenized with `tokenizer.json` (`d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89`).
4. In `StackModel` (`width = 256`, `vocab_size = 4096`), `embedding.weight` has `1,048,576` parameters that directly enter `NativeVectorWriter::margin_rows`, so parameter updates in Part A and Part B must isolate the recurrence-gate rate parameters (`Part A`) and `spiralcore_fbs` query tensors (`Part B`), and $\Delta_{\text{noise}}$ must be computed on the paired per-seed loss drop $\Delta L^{(a,s)} = L_0^{(a,s)} - L_{256}^{(a,s)}$ from identical seed-$s$ initialization $L_0^{(a,s)}$.

Classification: **`instrument`**. Reconcile the prep sheet, re-run fresh-context `TEST FITNESS: FIT`, D68 decision review, and D67 pre-run review before spinning up the RTX 5090 pod.

## Files in scope

- `crates/uor-r4-float-train/**`
- `docs/mechanisms/recurrence-gates.md`
- `docs/mechanisms/spiralcore-fbs.md`
- `docs/harness/prep/r-secant-surrogate-descent*`
- `docs/labs/r-secant-surrogate-descent/**`
- `docs/loop/tasks/r-secant-surrogate-descent/**`

## Done when

- `docs/harness/prep/r-secant-surrogate-descent-2026-10-25.md` passes `uor-harness check`, carries fresh-context `TEST FITNESS: FIT`, D68 decision review, and D67 pre-run review posted on `#2886`.
- `crates/uor-r4-float-train` builds and runs `secant-surrogate-descent` on a leased RTX 5090 pod across all 6 arms (`RecSecant_Active`, `RecZeroCredit_Null`, `RecRandomRefit_Null`, `FbsSecant_Active`, `FbsZeroCredit_Null`, `FbsRandomRefit_Null`) $\times$ 2 seeds (`1, 2`) $\times$ 256 steps (plus a second independent nvcc build verifying byte-identical replication), with `I-Finite-CUDA`, `I-Boundary-Crossings`, and `I-Keep-Plateau` recorded and artifacts uploaded to `caseyallard/uor-r4-store` under `results/antigravity/r-secant-surrogate-descent-2026-10-25/`.
- `docs/labs/r-secant-surrogate-descent/README.md`, `docs/labs/r-secant-surrogate-descent/receipt.json`, `docs/mechanisms/recurrence-gates.md`, `docs/mechanisms/spiralcore-fbs.md`, and `docs/research/notebook/float-training.md` record the results, and `docs/roadmap/items/r-secant-surrogate-descent.md` is set to `status: done` (`Closes #2886`).

## Stop and ask if

- Never wait: if `I-Boundary-Crossings < 1.0%` at default learning rate, run the registered step-0 boundary-crossing learning-rate calibration pass before scoring `G-Rec-Descent` and `G-Fbs-Descent`; if `I-Keep-Plateau` triggers, apply the analytic $\Delta\kappa_i$ instrument fix before calling a mechanism verdict.
