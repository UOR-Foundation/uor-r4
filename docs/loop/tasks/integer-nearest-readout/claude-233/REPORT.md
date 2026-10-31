# REPORT: integer-nearest-readout, claude-233 (locate the parity gap: trunk and Hamming read on the GPU): row R

Lab: claude. Item: [integer-nearest-readout](../../../../roadmap/items/integer-nearest-readout.md) (#3462). Order: [ORDER.md](ORDER.md), written by hand, since `loop claim` refuses on #3436's false glob overlap. Sheet: [integer-nearest-readout-locate-2026-10-31](../../../../harness/prep/integer-nearest-readout-locate-2026-10-31.md).

## Tokens per second

No new measurement: this run compares queries and tokens, not speed. claude-231 timed the nearest head at 1,365 tok/s decode on (r-drop) s1.

## What did not work, first

- **The integer Hamming read does not compute what the float read computes.**
  - With the read on, the integer token disagrees with the float D11 token on 694 (s1) and 823 (s3) of 1,536 sequence positions. The bar is 46.
  - The trunk alone disagrees on only 5 and 11.
  - The query moves past 1e-3 on 1,526 and 1,528 positions, against 435 and 513 for the trunk alone. Its largest gap is 8.04 and 8.19, against 3.89 and 2.83.
  - Each read arm exceeds its own trunk: 689 > 2√699 ≈ 52.9, and 812 > 2√834 ≈ 57.8.
  - The sheet's row R applies. This form of the integer read is negative; the read as a mechanism is not closed (§4.1).
- **The integer read is only partly right.** An integer read that did nothing would agree on 402–412 positions at s1 and 408–430 at s3. It agrees on 842 and 713, so it carries part of the float read's effect, not all of it.
- **A second gap, found after the reading (a diagnostic, not registered): the served export is snapped, and the snapped D11 forward sits far from the trained forward.**
  - `fbs-export` snaps the Hamilton gates and the transition to the 120 icosians unless it is given `--raw-gates --raw-transition`. claude-232 used neither flag, nor did this run.
  - For the trunk alone at s1, the D11 query differs from the plain (trained) query by a median 46% of its L1 norm, ranging from 18% to 83%.
  - The export gates alone account for a median 28%, and the snapped tables with the Q12 query a further 40%.
  - The writer's token differs between the two on 1,336 of 1,536 positions.
- **So claude-227's parity holds the artifact against its own D11 forward, not against the trained forward.**
  - Its record's "within the exporter's known residual" understates that residual on this model.
  - That record is outside this unit's scope, so it is unchanged here (Still open).
- **Whether the snapped gap moves the nearest token, the one the trained model decodes, is not measured.** The writer's pick is a fragile probe, and no tool here computes the nearest token on these readouts.
- **The synthetic sequences cannot show a fault seen only on real prompts.** This run located the read on fixed sequences. The size of each gap's share in claude-232's miss is still unknown.
- **The pre-run review's expectation did not hold.** It expected the read to be barely exercised, since no token repeats inside these sequences. In fact the float read moves 1,129 and 1,117 of 1,536 sequence tokens. The sheet's `R-Exercised` reading measured that, so the read is measured on the sequences, and it is not measured on the single tokens (0 of 4,096).

## What worked

- **The trunk alone passes at both seeds:** 5 and 11 of 1,536, and 73 and 104 of 4,096, inside 46 and 122. At s3, the seed where claude-232 collapsed, the trunk agrees with its own D11 forward.
- **Every instrument held:**
  - `I-Data`;
  - `I-Export-Tests`, 12 of 12;
  - `I-Reading-Carried` on every arm: `retain_rates` once, the read on the h snapshots only, the 23-byte writer copies, and no nearest head in any artifact;
  - `I-Artifact-Identity`;
  - `I-Finite`, with all four comparisons present.
- **I-Repro is exact.** t1's counts are claude-227's (5 and 73). Its writer snapshot, artifact and D11 readout are byte-identical to claude-227's stored ones. So this instrument is claude-227's, at a later head.
- **The fault caught before the FIT.** The run's objective is `cell`, so since claude-231 its snapshots read nearest, while the D11 readout's token is the writer's. Comparing them would have read as a false row 0. The writer copy fixed it, and its three checks held.
- **The reviews changed the run before it spent anything:**
  - FIT round 1 found row B unreachable, and a failed variant hidden behind EXIT 0;
  - round 2 was FIT;
  - the pre-run review added `R-Exercised` and the watcher's seal wait.
- **Cost:** one RTX 5090 for about 16 minutes, about $0.30. The pod was deleted, and 0 pods are running.
- **The sealed run:** `results/claude/integer-nearest-readout-locate-2026-10-31/`, listed back:
  - `out/`: 80 files under `SHA256SUMS`, checked on the pod;
  - `job.sh`;
  - `reading/`: `read.sh`, `diag.sh`, their outputs and the watcher, under their own sums.

## What changed

- `docs/harness/prep/integer-nearest-readout-locate-2026-10-31.md`: the sheet, its two FIT rounds and the pre-run review.
- `docs/labs/integer-nearest-readout/README.md`: the lab record's claude-233 section.
- `docs/research/notebook/m4-serving.md`: one row.

The item's status stays `ready`, so its file is unchanged. No code changed.

## Measurements

- The four arms' d and query gaps, above.
- `R-Exercised` at both seeds.
- The diagnostics: plain against D11, and plain against export against D11, on t1's sequences; and the plain-against-D11 token difference on all four arms.

## Checks run at the head

- The sheet's §4 checks, each read from the run's files.
- `read.sh` and `diag.sh` check every file they read against the run's `SHA256SUMS` before reading it.
- `the_real_sheets_pass_the_shape_rule` passes locally.

## Skills used

None.

## Still open

- **Next for this item: the unsnapped re-export, then claude-232's parity again.**
  - Re-export claude-227's full (r-drop) snapshots with `--raw-gates --raw-transition --run-config`, the `lut-stack/5` forms of claude-181 and claude-182.
  - Rerun claude-232's laptop parity against the same stored float replies, under a new sheet.
  - First check that the unsnapped session serves the nearest head, the read and the memory together.
- **integer-hamming-read's step 5 (#3385): trace the read's hooks, integer against float,** on a stored h snapshot: which heads and which positions diverge. Its float side should be the trained or unsnapped forward, so the two gaps are not confused.
- **claude-227's record** (`docs/labs/integer-retention-rates/README.md`): its "computes what the trained trunk computes" should say "its own D11 forward". That needs a unit with that file in scope.
- **The item stays `ready`.** point-key-integer's rerun waits on it.

References #3462.

## Plain summary

We checked two parts of the converted model separately against the trained model, on a graphics card.

The main body of the model is computed correctly in whole numbers at both settings we tried: it matches the converter's own decimal version. The part that looks back at earlier words does not: with it switched on, the converted model's choice differs about half the time.

We also found a second, separate cause. The converter rounds some of the model's rotations to a fixed set of shapes, and that moves the model's internal reading a long way from the trained one. An earlier unit built a converter option that skips this rounding, and it closed the same kind of gap on other models.

So the next step is to convert without the rounding and check again against the trained model's answers. Then trace the look-back part.
