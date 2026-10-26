# REPORT: d11-exporter-float-trunk, claude-130

Lab: claude. Order: [ORDER.md](ORDER.md). Prep sheet: [d11-exporter-float-trunk-d11-train-2026-10-26](../../../../harness/prep/d11-exporter-float-trunk-d11-train-2026-10-26.md) (FIT round 1; D67 pre-run review GO). Decision review: DECISION_REF.

References #2820.

## What did not work, first

- **`G-D11-ExportGap` FAILs on both seeds, on its top-1 half, in opposite directions.** The trunk was trained in the D11 mode, and its artifact was scored against its own plain-float forward:

  | Seed | F top-1 (plain float) | A top-1 (artifact) | b | c | Net b − c | F_I | Top-1 half |
  |---|---|---|---|---|---|---|---|
  | 1 | 3,941 | 3,663 | 719 | 441 | +278 | 69 | outside |
  | 2 | 917 | 1,249 | 258 | 590 | −332 | 59 | outside |

  - Both counts are over the 14,287 `ft/dev` targets. b is F hits with A misses, stored as `a_hit_b_miss_c`; c is the reverse, stored as `a_miss_b_hit_b`.
  - **On seed 2 the artifact beats its own float forward (c ≫ b).** Under the sheet's pre-registered reading, there the float model, not the artifact, is off: the weights adapted to the D11 forward.
  - **On seed 1 the artifact loses 278 net rows.**
  - **What it decides,** under the sheet's FAIL row: D11-mode training by this recipe (the bridges, Stage 4b's 1,024 steps and data) does not bring the gap inside the band. That closes this configuration, never the mechanism. The next step is the decision review's, below.
- **D11-mode training is unstable across seeds in plain float.** Its top-1 is 3,941 and 917, against the plain-trained trunk's 1,988 and 2,643. Its own two-seed loss band, |L_F(1) − L_F(2)|, is 0.0445, about Stage 4b's whole band of 0.0457.
- **The item's done-when is not met,** so #2820 stays open.

## What changed

Size against `origin/main`: float-train, in `hamilton_sheet.rs`, `geometric_stack_native.rs`, `spiralcore_fbs_sheet.rs` and the sheet binary. Docs: the ORDER, the prep sheet, this REPORT and the notebook row.

- **`with_d11_training`** runs a training step's loss and backward in the D11 mode with geometric straight-through bridges:
  - **the gates** train through `Hamilton_IcosianHard`'s S³ tangent bridge;
  - **the transition snap** passes the unit normalization's gradient (`d11_transition_unit`), the same bridge;
  - **the Q12 query** passes its gradient straight through (`q12_query`).

  The flag is off outside it, so the evaluation's D11 forward is unchanged.
- **`run_fbs_arm_with(…, d11_train)`** trains an arm that way.
- **`d11-train-probe`** gives 32 D11-mode steps and the projected wall time.
- **`d11-train-export`** trains the export arm on both seeds, checks each saved model, and writes its snapshot and float predictions. Its receipt is in the shape claude-129's scorer reads.
- **Tests.**
  - The host test `d11_training_sets_and_restores_its_bridge` pins the flag.
  - The CUDA test `gpu_d11_bridges_pass_the_tangent_and_straight_gradients` pins both bridges:
    - their forwards equal the snaps;
    - the transition's gradient is the normalization's;
    - the Q12 gradient is the identity;
    - in evaluation, the snap passes no gradient.

## Measurements

**The loss half is inside the band on both seeds** (Δ_noise,L 0.0457):

| Seed | L_F | L_A | L_A − L_F |
|---|---|---|---|
| 1 | 1.8836242760872335 | 1.8859133148256773 | +0.0023 |
| 2 | 1.928099705005298 | 1.923851524814812 | −0.0042 |

**Every instrument check was green.**
- **`I-Bridges`:** the float-train library passed on the RTX 5090, 55 passed and 5 ignored. That includes the bridge test, the flag test, and claude-125's and claude-126's GPU D11 tests.
- **`I-Probe`:**
  - the D11-mode training loss fell from 2.8008 to 2.6160 over 32 steps;
  - the DEV float loss fell from 2.7939 to 2.6153;
  - a step took 0.137 s;
  - the full run projected to 377 s.
- **`I-Save-Load`:** both seeds' saved models reproduce their own float panel and loss.
- **`I-Verify-All`:** G = H = R at all 82,176 window positions on each seed. The evaluator's PTX `013d19fd…` is claude-128's qualified one.
- **`I-Inputs`:** the windows, each seed's exported artifact and the five data and receipt pins all match.
- **`I-DEV-Denominator`:** 14,287 rows.
- **`I-Finite`:** every step exited 0, and all 92 saved files verify against `SHA256SUMS`.

**Readings, with no bar:**
- **The artifact against the model's export-mode panel.** The artifact's top-1 (3,663 and 1,249) is within a few dozen rows of the same weights' export-mode panel (3,638 and 1,237). So after D11 training, what is left of the gap lies between the plain-float forward and the D11-like forwards of the same weights, not between the D11 forward and the artifact.
- **Against claude-129's direct export of the plain-trained trunk.** That artifact scored 1,907 and 2,294. The D11-trained artifact scores 3,663 on seed 1, almost twice as many rows, and 1,249 on seed 2.
- **The step-0 query credit** is in the probe receipt.
- **Training wall time:** 1,024 steps per seed took about 3 minutes, and both seeds with their evaluations about 6 minutes.

**Evidence:** the HF store at `results/claude/d11-fbs-d11-train-2026-10-26/`, each folder with a `SHA256SUMS`:
- `d11t/` holds every step's stdout, stderr and exit code, the probe, the training receipt, the saved models and snapshots, both artifacts, the windows, both evaluator runs, both gap receipts and `job.log`;
- `d11t-bin/` holds the three binaries run.

## Checks run at the head

See the exact-head review record on the PR.

## Skills used

- **holding-the-goal.** The bar stayed Stage 4b's, and so did the comparator: F, this candidate's own float forward. The pre-registered reading of the gap's direction was applied as written. Seed 2's c ≫ b is reported as the float model being off, not as an export success.
- **writing-meaningful-tests.** The bridge test pins both the forward and the gradient of each bridge, together with the no-gradient case in evaluation. It ran on the GPU before any training.

## Still open

- **The next step is the decision review's** (the sheet's FAIL row). Its options:
  - **A comparator question.** For a candidate trained in the D11 mode, CHARTER §3.7's float candidate is still its plain-float forward, so the gap measures how far the weights have moved away from plain float.
  - **Seed instability** of D11-mode training: top-1 of 3,941 against 917.
  - **A longer or annealed D11 schedule.**
  - **Seed 1's writer-address collapse** (claude-127).
- **The item's "What is missing" text** is outside this ORDER's scope.

## Plain summary

The model was trained so that it already works the way the shipped integer file computes, and then exported and scored again.
- **The file is much closer to the model's own way of working.** Its accuracy matches the model's integer-like mode within a few dozen answers.
- **The comparison still fails.** The pre-registered test compares the file against the model's ordinary float mode, and training this way pulls that float mode away. In one run the float mode got 278 more answers right than the file; in the other it got 332 fewer.
- **The two training runs also disagreed with each other a lot.** So the next step is a decision about which of these to fix first.
