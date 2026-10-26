# Pre-registration: longer dual-forward training (`d11-exporter-float-trunk`, kimi-46), 2026-10-26

- **Item:** #2820; ORDER `docs/loop/tasks/d11-exporter-float-trunk/kimi-46/ORDER.md`.
- **Prior decision:** claude-136's bound proves exporter fidelity cannot flip any of the six existing verdicts. The remaining pivot is one decisive longer run, not more residual inspection. claude-131's DEV losses were still falling at step 1,024.
- **Intervention:** **2,048 steps per seed instead of 1,024**, with the same per-step learning rates (`0.002` base, `20` decay multiplier), weight decay `0.1`, clipping, corpus, dual 1:1 loss, bridges, exporter and GPU evaluator. Evaluation points are exactly `0, 256, 512, 1,024, 2,048`. The first 1,024 batches are claude-131's deterministic document-order batches; the next 1,024 are new windows from the same corpus, so this is a dose-plus-coverage intervention, not a pure optimizer-step isolation.
- **Status:** `TEST FITNESS: PENDING` until a fresh non-author review. No compute before FIT and D67.

## 1. Question

Does the 2,048-step `Fbs_1Atom_Active` dual-forward recipe bring the exported artifact inside Stage 4b's export-gap band on both seeds, without sacrificing claude-129's artifact top-1 floors? This tests the configuration only; it does not show that dual training beats a 2,048-step plain arm, closes M1, or supplies sealed-fresh acceptance. #2820 may close only at its DEV export-gap contract.

## 2. Base artifact

- **F:** this run's plain-float model, trained from step 0 with `TrainForward::Dual` for 2,048 steps on seeds 1 and 2, scored on 14,287 `ft/dev` rows.
- **A:** each saved model's UORLUT01 artifact, scored through `fbs-export`, `d11-fbs-eval run --verify` and the existing artifact-gap scorer.
- **D:** each saved model's D11 forward, with D predictions and `loss_d11` captured. D is the perfect-exporter counterfactual: D passes only if `|net(F,D)| <= F_I(F,D)` and `|L_D-L_F| <= Δ_noise,L`.
- Each seed's model, snapshot, F/D/A predictions, artifact, evaluator output and gap/residual receipts are sealed with source, binary, kernel, geometry, data and sheet identities.

## 3. Bar and outcomes

`b` is F-hit/A-miss and `c` the reverse on 14,287 rows; `F_I = max(24, ceil(2*sqrt(b+c)))`. `Δ_noise,L = 0.045679637781173676`, read from claude-126's `d11-gap.json` (SHA-256 `88b6a5f988fe144671365a258a100e5cf3935f98e917040b38f90df2637a1e8a`). Apply this precedence per seed; never average mixed seeds.

1. **Instrument stop** if any instrument, batch/prefix, save/load, verification or finiteness check fails. Repair it; no model result.
2. **Unflagged PASS** if both A seeds pass top-1 and loss, all instruments are green, and artifact top-1 is at least 1,907 (seed 1) and 2,294 (seed 2). Measure product decode/context tokens/s, zero-multiply audits and integer FBS serving tests, then close #2820 at DEV scope only.
3. **Flagged PASS** if both A seeds pass but either artifact is below its floor. D68 review; do not close #2820 automatically.
4. **Exporter unresolved** if at least one A seed fails and every failing A seed has D passing top-1 and loss, including mixed pass/unresolved. Do not call FAIL; open a separately FIT'd localisation/repair unit with its own cost.
5. **FAIL** if any seed has both A and D failing, including mixed pass/clean-fail. Close only this 2,048-step dual configuration and record a lab stop decision; the mechanism stays open and #2820 is not closed.

At claude-131's discordance, a pass requires roughly 79% (seed 1) and 70% (seed 2) reductions in absolute net gap. A FAIL therefore rejects this configuration, not every longer-training benefit. Report plain/export-mode top-1 and admitted rates, trajectories, `net(D,A)`, full-precision two-seed loss band against `0.0230`, plain-float top-1 spread against `655`, and paired b/c/F_I with a 95% paired interval as a reading.

## 4. Required instrument checks

1. **I-Bridges:** `cargo test --release --locked -p uor-r4-float-train --features cuda --lib` passes, including dual gradient sum, tangent bridge, Q12 clamp and all prior D11 tests.
2. **I-Probe:** `--mode d11-train-probe --train-forward dual --steps 2048` has finite losses, falling training/DEV loss and `under_one_hour_per_seed=true`; its receipt distinguishes 32 probe steps from the 2,048 target and projected evaluation points.
3. **I-Batch-Preflight:** the real loader returns exactly 2,048 batches (8,192 usable windows), total windows, train tokens, scored targets, peak host RSS, and exact projected eval points `[0,256,512,1024,2048]`. `train.json` repeats steps, batch count and eval points.
4. **I-Prefix:** at step 1,024, both seeds reproduce claude-131's stored top-1/export-mode/admitted counts exactly and full-precision losses within `1e-6`. A mismatch is source/data/device drift, not a dose result; stop and diagnose before interpreting step 2,048.
5. **I-Save-Load:** each reloaded model reproduces its plain predictions and loss exactly at step 2,048.
6. **I-Verify-All:** `d11-fbs-eval run --verify` gives G = H = R at all 82,176 positions per seed.
7. **I-D:** D is deterministic, D predictions and loss are saved, and the F-vs-D counterfactual is scored with paired F_I and `Δ_noise,L`.
8. **I-Inputs:** source commit, all three binary SHAs, four data pins, `d11-gap.json`, `train.json`, model/config/snapshot/artifact/windows/evaluator/gap/residual SHAs, exact commands and exit codes are recorded and verified. `d11-artifact-gap` checks `--model-sha256` and `--steps 2048`; every residual receipt binds the same inputs.
9. **I-Negative:** one deliberately wrong artifact or windows hash yields `i_inputs=false` and `gap_inside_band="not counted"`.
10. **I-DEV-Denominator:** 14,287 rows over 428 windows; do not conflate 1,572,864 training positions with DEV rows.
11. **I-Finite:** every command exits 0; stdout/stderr/exit codes and SHA256SUMS are saved and read back.

## 5. Data split

Use `w2-inputs/2026-10-25/driver-data/` with the existing pins:

- train tokens `3dee858f84b750f507ceb3dc3f393711741950954735efaaf61bf704c017a4c8`; train mask `b5e7352d67d6883a833155a5fd397a03f6d4f1b2649e3d4ec3e2ea7d95b949e3`.
- DEV tokens `0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b`; DEV mask `a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078`.
- tokenizer `d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89`.
- No `fresh_v2` evaluation and no base checkpoint. Models train from step 0 at the pushed head. `--steps` accepts only 1,024 or 2,048 and is validated before opening CUDA; other longer doses are refused.

The two seeds are **initialization seeds on a common fixed batch stream and common DEV panel**, not independent data-order replicates. Each seed's paired row statistics are reported separately; no cross-seed confidence interval is inferred.

## 6. Steps and minimum power

1. Lease one RTX 5090 inside the fleet cap and build/copy `spiralcore-fbs-sheet`, `fbs-export`, and `d11-fbs-eval` with SHA-256 pins. Record the source commit and binary hashes in a run manifest.
2. Fetch/check section 5 inputs, run I-Bridges, I-Probe and I-Batch-Preflight, then stop on any failure.
3. Run `spiralcore-fbs-sheet --mode d11-train-export --train-forward dual --steps 2048 --data-root <driver-data> --save-dir save --stage4b d11-gap.json --out train.json` and apply I-Prefix to its step-1,024 evaluations against claude-131's sealed receipt.
4. Generate DEV windows. For each seed, export with `fbs-export`, verify with `d11-fbs-eval`, and run both:
   - `--mode d11-artifact-gap --steps 2048 --model-sha256 <model> --stage4b train.json --preds-artifact-out preds-artifact-seed-<s>.json` with model/evaluator/artifact/window/prediction pins;
   - `--mode d11-residual --steps 2048 --model-sha256 <model> --stage4b train.json --preds-d11-out preds-d11-seed-<s>.json` with the same inputs and label `longer-dual`.
5. Save every model, snapshot, F/D/A prediction, artifact, evaluator/gap/residual receipt, manifest, command, exit code and SHA256SUMS to `results/kimi/d11-fbs-longer-dual-2026-10-26/`, read back, then release/down the pod.
6. Only for an unflagged PASS, run laptop `fbs-toks` decode/context, both zero-multiply audits and integer FBS serving tests.

**Power:** two initialization seeds, 14,287 DEV rows, 2,048 steps per seed (1,572,864 training positions), paired discordance and no seed averaging. The decision is the large registered gap reduction above, not a precise modest-effect estimate. No 2,048-step plain arm is run, so no dual-versus-plain causal comparison is licensed.

## 7. Cost and stop

- Base run: about 1.75 hours—10 minutes builds, 12 minutes dual training, 20 minutes verified evaluation per seed, residual scoring and archival. At `$1.11/h`, about `$1.95`; lease 2.5 hours and record complete cost. Healthy runs are renewed, not cancelled for estimates.
- Exporter-unresolved localisation is a separate GPU unit with its own FIT and cost, not included in this lease or authorized here.
- Stop before full training on failed I-Bridges/I-Probe/I-Batch. Stop after training on prefix, identity, verification or finiteness failure and preserve evidence. Classify instrument versus configuration before any conclusion.

## 8. Review

TEST FITNESS: PENDING. A fresh non-author FIT and a fresh D67 pre-run review are required before dispatch.
