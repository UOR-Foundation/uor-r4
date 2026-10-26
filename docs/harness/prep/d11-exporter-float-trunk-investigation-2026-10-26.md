# Prep sheet: the residual investigation, a Q16 control and the drift readings (d11-exporter-float-trunk, claude-133), 2026-10-26

- Item: [d11-exporter-float-trunk](../../roadmap/items/d11-exporter-float-trunk.md) (#2820). ORDER: [claude-133](../../loop/tasks/d11-exporter-float-trunk/claude-133/ORDER.md).
- This is the instrument investigation that [claude-132's residual sheet](d11-exporter-float-trunk-residual-2026-10-26.md) pre-registered for its indeterminate row, after claude-132 classified four of six models as indeterminate.
- Run: one job on one leased RTX 5090. It runs `spiralcore-fbs-sheet --mode d11-residual` at this unit's head over claude-132's six saved models and their artifacts' stored, verified runs. No training, and no artifact re-run.

## 1. Question

**Is the residual between the D11 forward and the artifact the integer session's designed arithmetic, or an exporter departure?**
- On four models, the D11 panel (D) and the artifact (A) differ on 3.4% to 4.9% of tokens, above claude-132's 3% faithful bound. The head is identical to A on every row, and D against A passes the paired test.
- D is a float32 forward, and the integer session keeps its residual stream on a 2^-16 grid (`RESIDUAL_EXP` = −16).
- If A tracks the D11 forward computed on that grid, the residual is the design's own precision, not a departure.

## 2. Arms

These are claude-132's five panels (F, E, D, H, A), on the 14,287-row `ft/dev` panel, plus one new one:
- **D16, the Q16 control:** D, the D11 forward with the tables rounded as exported, but with the residual stream rounded to 2^-16 after the embedding and after each sublayer add (`with_q16_residual`), where the integer session holds it. The final norm, the gates, the snaps and the Q12 query are as in D.

**The drift readings,** for A against D, for A against D16, and for D16 against D, all in Q12 codes:
- the median, mean, 99th-percentile and largest absolute difference;
- the three columns holding the most per-row maxima;
- the largest per-channel mean signed difference;
- the entries saturated at the i16 ends on each side;
- the per-row L1 scale Σ|A| / Σ|reference|, at its 5th percentile, median and 95th percentile.

## 3. Bar and decision table

The bar applies per model. Every instrument check of section 4 must be green first. `classification_q16` is claude-132's rule (`classify_residual`) with D16 as the reference: d_H = 0, D16 differing from A on at most 3% of rows, and the paired test.

**A departure signature, on any model, is any of these for A against D16:**
- the median row L1 scale outside [0.98, 1.02];
- the largest per-channel mean signed difference above 8 codes in size;
- A saturated on more than 1% of entries above the reference's count;
- `classification_q16` = departure.

| Outcome | It means | It decides |
|---|---|---|
| **Q16 arithmetic.** On each of claude-132's four indeterminate models, `classification_q16` = faithful, and no model shows a departure signature. | A computes the D11 forward at the integer session's designed residual precision. D's float32 residual explains the rest of the D-against-A residual. | The residual sheet's indeterminate row is settled as faithful, so all six models are faithful against the integer session's own precision. The residual sheet's "faithful on all six" decision follows: the three FAILs stand as mechanism results of their configurations, and the item parks behind the owner's reading of §3.7 (#2028). |
| **A departure.** Any model shows a departure signature. | A computes something measurably different from its D11 forward, even at Q16 precision. | An exporter or evaluator defect: an instrument repair item opens, located by the readings. After the repair, the affected artifacts are re-exported and re-scored under the unchanged bar. |
| **Other arithmetic, still indeterminate.** Neither of the above. | The Q16 residual does not explain the residual, and no departure signature shows. | The remaining residual lies in the integer session's other roundings: the norm at 2^-24, the products, the state exponents, or snap ties. The next control emulates those roundings. It is never a mechanism result, and the item does not park. |

**Readings, with no bar:**
- per model, D16 against A, against D and against H, each with b, c, net and the number of rows that differ;
- the top-1 counts of all six panels;
- the drift summaries (claude-132's) for D and D16 against A;
- the Q12 i32-oracle check of D16's first chunk.

## 4. Instrument checks (each must be green before the classification counts)

These are claude-132's checks, unchanged:
1. **`I-GPU-Tests`.** The float-train CUDA library passes, including `gpu_the_q16_control_holds_the_residual_on_its_grid_and_is_off_by_default` and the earlier D11 and head tests.
2. **`I-Fetch`.** Every fetched file verifies against its store folder's `SHA256SUMS`, and the DEV data and the six models against their pins.
3. **`I-Save-Load`, `I-Inputs`, `I-Verify-All`, `I-DEV-Denominator`, `I-D-Determinism` and `I-Head`,** as in claude-132.
4. **`I-Head-Q16`.** The head on D16's own queries writes exactly D16's predictions.
5. **`I-D-Stored`, plain-c126 only.** D equals Stage 4b's stored D11 panel.
6. **`I-D-Reproduces`.** D's predictions are claude-132's, row for row: its D-against-A rows-differ count and its D top-1 equal the receipts in `results/claude/d11-fbs-residual-2026-10-26/resid/`. This shows that the Q16 hook, off, leaves D unchanged.
7. **`I-Finite`.** Every step exits 0, and each step's stdout, stderr and exit code are saved. The log's last line is written before `SHA256SUMS`.

## 5. Base artifact and data split

**The six models and their artifacts' stored runs,** with the same pins as claude-132's sheet, §5 ([residual sheet](d11-exporter-float-trunk-residual-2026-10-26.md)):
- **plain-c126:**
  - models in `results/claude/d11-fbs-export-gap-2026-10-26/d11gap/save/seed-{1,2}/`;
  - runs in `results/claude/d11-fbs-artifact-gap-2026-10-26/gap/`;
  - artifacts `314c3909…` and `d5b43596…`;
  - models `63ae5513…` and `37aae396…`.
- **d11-c130:**
  - models and runs in `results/claude/d11-fbs-d11-train-2026-10-26/d11t/`;
  - artifacts `4255d39b…` and `b466501e…`;
  - models `55f5d6dc…` and `f3c1b206…`.
- **dual-c131:**
  - models and runs in `results/claude/d11-fbs-dual-train-2026-10-26/dual/`;
  - artifacts `0ae7dcd4…` and `8394a4d3…`;
  - models `6861db0f…` and `a78ce0b9…`.

The windows are `083c1be628f871ff1502e7d05f84cc3f55dda2ac461614a62df9167bcd08893e`. The full 64-hex pins are in the residual sheet and in the job's `inputs.sha256`.

**DEV data:** `w2-inputs/2026-10-25/driver-data/ft/dev/`:
- `tokens.u16` `0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b`;
- `response_mask.u8` `a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078`.

No training data is read.

## 6. Steps and minimum test power

1. `uor-pod up --count 1 --gpu 5090 --ref <pushed head>`.
2. Build `spiralcore-fbs-sheet` with nvcc, and copy it with its sha256.
3. `I-GPU-Tests`.
4. Fetch and check the inputs exactly as claude-132's job did (`I-Fetch`). Also fetch claude-132's six residual receipts, for `I-D-Reproduces`.
5. Run `--mode d11-residual` on each of the six models, with claude-132's arguments, writing `residual-<label>-seed-<s>.json`.
6. Check `I-D-Reproduces` against claude-132's receipts. It is a job step: `jq` compares the D top-1 and the D-against-A rows-differ count.
7. Save the run folder to the store at `results/claude/d11-fbs-investigation-2026-10-26/`, with a `SHA256SUMS`, from the laptop, confirmed with `uor-store ls` before `down`.

**Minimum test power:**
- six models, across 3 configurations and 2 seeds;
- 14,287 frozen DEV targets each, so 7.2 million query entries per panel pair;
- every A row read from a verified run, so P = 0.

## 7. Cost and stop rule

**Cost:**
- one RTX 5090 on Vast, about $0.56 to $1.11/h, under a 1.5 h lease;
- claude-132's job took 7 minutes, and the added D16 panel adds about a sixth;
- in all, about 20 minutes, under $1.

**Stop rule:**
- Stop and save on a failed `I-GPU-Tests` or `I-Fetch`, a non-finite loss, or any nonzero exit.
- A failed per-model instrument check leaves that model unclassified, and the others still count.
- Classify every failure as instrument or mechanism first.
- Never cancel a healthy run because the lease ran short; renew it.
- Save everything off the pod before `down`.

## 8. Non-author TEST FITNESS review

To be filled in by the fresh reviewer.
