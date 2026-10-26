# Pre-registration: longer dual-forward training (`d11-exporter-float-trunk`, kimi-46), 2026-10-26

- **Item:** #2820; ORDER `docs/loop/tasks/d11-exporter-float-trunk/kimi-46/ORDER.md`.
- **Prior decision:** claude-136's bound proves exporter fidelity cannot flip any of the six existing verdicts. The remaining pivot is one decisive longer run, not more residual inspection. claude-131's DEV losses were still falling at step 1,024.
- **Single change:** **2,048 steps per seed instead of 1,024**, with the same per-step learning rates (`0.002` base, `20` decay multiplier), weight decay `0.1`, clipping, data, seeds, dual 1:1 loss, bridges, exporter and GPU evaluator. Evaluation points are `0, 256, 512, 1,024, 2,048`. Nothing else changes.
- **Status:** `TEST FITNESS: PENDING` until a fresh non-author review. No compute before FIT and D67.

## 1. Question

Does doubling only the dose of the existing `Fbs_1Atom_Active` dual-forward recipe bring the exported artifact inside Stage 4b's export-gap band on both seeds, without sacrificing claude-129's artifact top-1 floors?

## 2. Arms and artifacts

- **F:** this run's plain-float model, trained from step 0 with `TrainForward::Dual` for 2,048 steps on seeds 1 and 2, scored on 14,287 `ft/dev` rows.
- **A:** each saved model's UORLUT01 artifact, scored through `fbs-export`, `d11-fbs-eval run --verify` and the existing artifact-gap scorer.
- **D:** each saved model's D11 forward, retained so `net(D,A)` can bound how much of any remaining gap exporter fidelity could explain.
- Each seed's model, snapshot, predictions, artifact, evaluator output and gap receipt are sealed with its source, kernel, geometry, data and sheet identities.

## 3. Bar and outcomes

`b` is F-hit/A-miss and `c` the reverse on 14,287 rows; `F_I = max(24, ceil(2*sqrt(b+c)))`. `Δ_noise,L = 0.045679637781173676`, read from claude-126's `d11-gap.json` (SHA-256 `88b6a5f988fe144671365a258a100e5cf3935f98e917040b38f90df2637a1e8a`).

| Outcome | Condition | Decision |
|---|---|---|
| **Unflagged PASS** | Both seeds have `|b-c| <= F_I` and `|L_A-L_F| <= Δ_noise,L`, all instruments green, and artifact top-1 is at least 1,907 (seed 1) and 2,294 (seed 2). | Measure product decode/context tokens/s, run zero-multiply audits and integer FBS serving tests, then close #2820. |
| **Flagged PASS** | The gap passes but either artifact is below its claude-129 floor. | D68 review; do not close #2820 automatically. |
| **Residual unresolved** | A seed is outside F_I by no more than `|net(D,A)|`. | Run the continuous localisation before any stop; exporter fidelity could explain the remaining gap. |
| **FAIL** | Any other bar failure, including a mixed-seed result. | Close only this 2,048-step dual configuration; record a lab decision to stop this line. The mechanism stays open and #2820 is not closed. |
| **Instrument stop** | Failed bridges/probe, non-finite loss, identity mismatch, failed save/load/verify or nondeterminism. | Repair the instrument; do not count a model result. |

Report plain/export-mode top-1 and admitted rates, training trajectories, `net(D,A)`, full-precision two-seed loss band against `0.0230`, and plain-float top-1 spread against `655`. A mixed seed is never averaged into a pass.

## 4. Required instrument checks

1. **I-Bridges:** `cargo test --release --locked -p uor-r4-float-train --features cuda --lib` passes, including dual gradient sum, tangent bridge, Q12 clamp and all prior D11 tests.
2. **I-Probe:** `--mode d11-train-probe --train-forward dual --steps 2048` has finite losses, falling training/DEV loss and `under_one_hour_per_seed=true`.
3. **I-Save-Load:** each reloaded model reproduces its plain predictions and loss exactly at step 2,048.
4. **I-Verify-All:** `d11-fbs-eval run --verify` gives G = H = R at all 82,176 positions per seed.
5. **I-Inputs:** all data/artifact/window/model/source/sheet pins match; each gap receipt names this sheet.
6. **I-DEV-Denominator:** 14,287 rows over 428 windows.
7. **I-Finite:** every command exits 0; stdout/stderr/exit codes and SHA256SUMS are saved and read back.

## 5. Data and source

Use `w2-inputs/2026-10-25/driver-data/` with the existing pins:

- train tokens `3dee858f84b750f507ceb3dc3f393711741950954735efaaf61bf704c017a4c8`; train mask `b5e7352d67d6883a833155a5fd397a03f6d4f1b2649e3d4ec3e2ea7d95b949e3`.
- DEV tokens `0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b`; DEV mask `a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078`.
- tokenizer `d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89`.
- No `fresh_v2` evaluation and no base checkpoint. Models train from step 0 at the pushed head. The new `--steps` binding defaults to 1,024 and only the longer-dose sheet changes for Dual above 1,024.

## 6. Steps and minimum power

1. Lease one RTX 5090 inside the fleet cap and build/copy `spiralcore-fbs-sheet`, `fbs-export`, and `d11-fbs-eval` with SHA-256 pins.
2. Fetch/check section 5 inputs, run I-Bridges and I-Probe, then stop on any failure.
3. Run `spiralcore-fbs-sheet --mode d11-train-export --train-forward dual --steps 2048 --data-root <driver-data> --save-dir save --stage4b d11-gap.json --out train.json`.
4. For each seed, export with `fbs-export`, verify with `d11-fbs-eval`, and score with `--mode d11-artifact-gap` using the saved model/predictions, verified evaluator output, artifact/window hashes and this run's `train.json`.
5. Save every output to `results/kimi/d11-fbs-longer-dual-2026-10-26/` with SHA256SUMS, read back, then release/down the pod.
6. Only for an unflagged PASS, run laptop `fbs-toks` decode/context, both zero-multiply audits and integer FBS serving tests.

**Power:** two independent training seeds, 14,287 DEV rows, 2,048 steps per seed (1,572,864 tokens), with paired discordance and no seed averaging.

## 7. Cost and stop

- Estimated one-pod time about 1.5 hours: 10 minutes builds, 12 minutes dual training, 20 minutes verified evaluation per seed, plus probes/archival. At `$1.11/h`, about `$1.70`; lease 2.5 hours and record complete cost. Healthy runs are renewed, not cancelled for estimates.
- Stop before full training on failed I-Bridges or I-Probe. Stop and preserve evidence on identity mismatch, non-finite loss, failed verification or host fallback. Classify instrument versus configuration before any conclusion.

## 8. Review

TEST FITNESS: PENDING. A fresh non-author FIT and a fresh D67 pre-run review are required before dispatch.
