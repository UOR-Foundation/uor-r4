# Prep sheet: snap-flip localisation (d11-exporter-float-trunk, claude-134), 2026-10-26

- Item: [d11-exporter-float-trunk](../../roadmap/items/d11-exporter-float-trunk.md) (#2820). ORDER: [claude-134](../../loop/tasks/d11-exporter-float-trunk/claude-134/ORDER.md).
- This is the next control named by [claude-133](../../loop/tasks/d11-exporter-float-trunk/claude-133/REPORT.md), whose investigation ended "other arithmetic, still indeterminate".
- Run: one job on one leased RTX 5090:
  - requalify the traced evaluator;
  - trace claude-132's six artifacts on 32 DEV windows;
  - compare with each saved model's D11 snap choices.

No training.

## 1. Question

**Where do the D11 forward (D) and the artifact (A) first part, and do their token differences follow icosian snaps that flip on near-ties?**
- claude-132 left four of six models indeterminate: D and A differ on 3.4% to 4.9% of tokens.
- claude-133 found no gross signature beyond the faithful models' envelope.
- The untested hypothesis: a snap whose two best roots nearly tie flips between the float and integer forwards, and the recurrence carries the flip to later positions.
- If the token differences follow near-tie snap flips, the residual is the D11 forward's own sensitivity at its discrete choices, which the integer path computes as designed. If they follow snaps the float forward chose decisively, or no snap at all, something upstream of the snap differs.

## 2. Arms

There is one model per (configuration, seed), as in claude-132: plain-c126, d11-c130 and dual-c131, two seeds each. All are compared on **the first 32 DEV windows**: 6,144 positions and about 1,060 scored rows.
- **D's snap choices** come from `with_snap_capture` over the D11 forward on those windows. That gives, per position and layer, the root chosen at:
  - the transition, one per lane (64);
  - the output gate, one per lane (64);
  - the feed-forward gate, one per 4 MLP channels (96).

  With each choice comes its relative margin, (best − second) / best.
- **A's snap choices** come from `d11-fbs-eval trace`, the traced kernel, at the same sites in the same layout.
- **The comparison** (`--mode d11-snap-trace`) gives:
  - the disagreements per layer and site;
  - zero blocks, counted apart: the float forward snaps them to the identity, the integer path to root 0;
  - per window, the first position with a disagreement;
  - for each row where D and A differ, whether a snap disagreement precedes it in its window (at or before its position) and whether one sits at its position;
  - D's relative margins at every disagreement.

## 3. Bar and decision table

The bar is per model. Every instrument check of section 4 must be green first. Let R be the rows where D and A differ in the 32 windows, P the rows of R preceded by a snap disagreement, and m the median of D's relative margin at the disagreements.

| Outcome | It means | It decides |
|---|---|---|
| **Snap-flip arithmetic.** On each of the four indeterminate models: P ≥ 90% of R, and m < 1e-3. | The token differences follow snap flips where D's two best roots nearly tie. The integer path, computing the same forward at integer precision, falls on the other side. | The residual is the D11 forward's sensitivity at its discrete choices. The four indeterminate models are settled as faithful, so the residual check's "faithful on all six" follows: the exporter is cleared, the three FAILs stand as mechanism results, and the item parks behind the owner's reading of §3.7 (#2028). |
| **Localise upstream.** On any indeterminate model, m ≥ 1e-2: D chose decisively where A chose differently. | A's input to the snap differs from D's by more than rounding. | An instrument localisation of the pre-snap values on that model: the gate and transition raws, layer by layer. It is not a ruling. A departure is called only when that localisation finds the defect. |
| **Not snap-carried.** On any indeterminate model, P < 50% of R. | Most token differences come with no snap disagreement before them. | The residual is in the continuous arithmetic: the final norm, the Q12 query, or the writer's kept cells. The next control compares A's and D's queries at those rows. It is never a mechanism result. |
| **Indeterminate.** Anything else. | — | The readings are reported. The next step is a decision review, and the item does not park. |

**Readings, with no bar:**
- the same readings on the two faithful models, as the comparison;
- the disagreements per layer and site, and the zero-block count;
- the first-disagreement positions;
- the margin counts below 1e-6, 1e-4 and 1e-2.

## 4. Instrument checks (each must be green before the bar counts)

1. **`I-GPU-Tests`.** The float-train CUDA library passes, including `gpu_the_snap_capture_records_each_site_and_changes_nothing`. The instrument's host parity tests pass, including the trace test.
2. **`I-Qualify`.** The rebuilt evaluator, with its two kernels, passes:
   - `d11-fbs-eval qualify`;
   - `d11-fbs-eval run --verify` on all 428 DEV windows of plain-c126 seed 1, with G = H = R at all 82,176 positions, as claude-128 qualified it.

   Its new PTX and cubin sha256s are recorded.
3. **`I-Trace`.** For each artifact, the traced run's outputs and trace equal the host twin's (G = H), and the trace fits the trunk's shape.
4. **`I-Old-Run`.** For each artifact, the traced build's tokens on the 32 windows equal the stored, qualified run's: claude-129's, claude-130's or claude-131's.
5. **`I-Inputs`.** Each artifact's, the windows' and each model's sha256 equal their pins (section 5).
6. **`I-Fetch` and `I-Finite`,** as in claude-132: every fetched file against its folder's `SHA256SUMS`, every step exiting 0, and the log written before `SHA256SUMS`.

## 5. Base artifact and data split

- **The six models, their pins and their stored runs** are claude-132's, from the [residual sheet](d11-exporter-float-trunk-residual-2026-10-26.md) §5.
- **The six artifacts:**
  - `results/claude/d11-fbs-export-gap-2026-10-26/d11gap/save/seed-{1,2}/artifact.uorlut` (plain-c126);
  - `results/claude/d11-fbs-d11-train-2026-10-26/d11t/artifact-seed-{1,2}.uorlut` (d11-c130);
  - `results/claude/d11-fbs-dual-train-2026-10-26/dual/artifact-seed-{1,2}.uorlut` (dual-c131).

  Their sha256s are the residual sheet's artifact pins.
- **The windows:** the first 32 sequences of `windows.json` (sha256 `083c1be628f871ff1502e7d05f84cc3f55dda2ac461614a62df9167bcd08893e`), as the job cuts them. Their file's sha256 is recorded, and it is the `--windows-sha256` of the comparison.
- **DEV data:** `w2-inputs/2026-10-25/driver-data/ft/dev/`, pinned as in claude-132. No training data is read.

## 6. Steps and minimum test power

1. `uor-pod up --count 1 --gpu 5090 --ref <pushed head>`.
2. Build `spiralcore-fbs-sheet` (float-train) and `d11-fbs-eval` (the instrument) with nvcc, and copy both with their sha256s.
3. `I-GPU-Tests`: the float-train CUDA library and the instrument's tests.
4. Fetch and check the inputs (`I-Fetch`, `I-Inputs`).
5. `I-Qualify`: `d11-fbs-eval qualify`, then `run --verify` on plain-c126 seed 1's artifact over all 428 windows.
6. Cut the 32 windows. For each artifact, run `d11-fbs-eval trace`, then `spiralcore-fbs-sheet --mode d11-snap-trace … --windows-count 32 --stored-run <its run-seed-s.json>`.
7. Save the run folder to the store at `results/claude/d11-fbs-snap-trace-2026-10-26/`, with a `SHA256SUMS`, from the laptop, confirmed before `down`.

**Minimum test power:**
- six models;
- 32 windows each, 6,144 positions, and 224 snap sites per layer per position;
- about 1,060 scored rows per model, about 20 to 50 of them in R on the indeterminate models.

## 7. Cost and stop rule

**Cost:**
- one RTX 5090, about $1/h, under a 2 h lease;
- builds about 3 minutes;
- the qualification's `--verify` about 20 minutes (claude-131 and claude-132);
- six traces and comparisons a few minutes each;
- in all, about 45 minutes, about $1.

**Stop rule:**
- Stop and save on a failed `I-GPU-Tests` or `I-Qualify`, or any nonzero exit.
- A failed per-model check leaves that model unclassified.
- Classify every failure as instrument or mechanism first.
- Never cancel a healthy run because the lease ran short; renew it.
- Save everything off the pod before `down`.

## 8. Non-author TEST FITNESS review

To be filled in by the fresh reviewer.
