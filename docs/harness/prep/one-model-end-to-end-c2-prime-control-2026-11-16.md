# C2′ control: the fact asked in the previous slot (2026-11-16)

Status: **host count PASS; GPU arm RUN (one state per seed).** Owner approved the control
replacement on 2026-11-16 (`docs/loop/tasks/one-model-end-to-end/codex-18/DECISION.md`). The gate read
fails the determined control (0.142 / 0.139); the rule read passes the host count (1.000).

## Question

Bar 4b's `chose_target` half used the **asked fact**, the salient fact, so any rule that finds fact
rows raised it (claude-397). Replace it with a target the stream determines: the fact asked in the
**previous slot**, whose relation is in the stream as that slot's `Q(K)`.

## Method (host, no GPU)

`crates/uor-r4-float-train/src/bin/recall-rule-read.rs`: for every query after the first, match the
previous slot's query relation against the window's `K v` facts, and count the scan's hit. The
**salience baseline** is the saved A-ext asked pick (the native read's first pick) scoring on the
same previous fact.

## Result

| Split | Seed | Previous fact found by the rule | A-ext asked pick on it |
|---|---|---|---|
| held | 1 | **768/768** | 3/768 (0.004) |
| held | 2 | **768/768** | no picks saved |
| TRAIN | 1 | **6144/6144** | no picks saved |
| TRAIN | 2 | **6144/6144** | no picks saved |

The current other-fact rule stays exact on every identifiable row in the same runs (35/35, 12/12,
42/42, 8/8 held; 308/308, 81/81, 296/296, 66/66 TRAIN).

## Bars

- **Determined control.** A read trained on the previous slot's asked fact must pick it at >= 0.90
  of the **768 rows that have a previous fact** (slots 1-3, 3 per window; >= 692/768 held). Per
  bucket, the bar is 0.90 of the **nonzero-target rows** (seed 1: 341/378 and 351/390): slot 0 has
  no previous fact and stays in the trainer's `n`, so 0.90 of `n` is unreachable. The salience read
  is 0.004.
- **`chose_other` in this arm is the asked fact.** The trainer's `chose_other` column reads
  `--eval-other .ask`, so it is the salience drift, not the C2′ other-fact half. The other-fact
  half is measured separately by the rule read (exact on `K=1`: 35/35, 12/12, 42/42, 8/8).
- **No bar moves before the GPU arm.** The host count only shows the target is determined and the
  salience read does not follow it.

## GPU arm (run 2026-11-16)

Trained the native read on the previous-slot target: ext-s1 at shift 8, ext-s2 at shift 4, identity
start, 512 steps, batch 64; the held dumps were regenerated at the TRAIN `e_x` (24) so the trainer's
state check accepts the pair. Held results, step 512:

| State | `chose_target` (previous) | per valid bucket | `chose_other` (asked, salience) |
|---|---|---|---|
| ext-s1 | 109/768 (0.142) | 43/378, 66/390 | 66/512, 65/512 |
| ext-s2 | 107/768 (0.139) | 52/387, 55/381 | 55/512, 60/512 |

The bar is 0.90 of the 768 rows with a previous fact (per bucket: 378/390 seed 1, 387/381 seed 2); the gate read is 0.142/0.139. The rule read
finds the previous fact **768/768**, and the A-ext asked read 3/768. So the determined control
separates the rule from the gate, and the gate fails it: another gate fit is not the mechanism.

## Served decode (rule picks injected)

`recall-rule-read --picks-out` writes the rule's 1-based pick per row; `recall-payload-probe
--native-picks` injects it as the hard copy's first pick. On ext-s1, scale 16:

| Reading | Rule picks | A-ext baseline |
|---|---|---|
| `R_oth` (decoded token = the other fact) | **0.406 / 0.436** | 0.002 |
| `R_ask` (decoded token = the asked fact) | **0** | 0.984 / 0.969 |

So the rule read passes the **served answer** as well as the pick class: the model answers the other
fact at the all-rows ceiling and the asked fact at zero. The injection is the probe's hard copy (the
D85 diagnostic), not the integer session; the served integer read is the next unit.

## Verdict

The approved control **passes for the rule read** (host 1.000; served answer `R_oth` 0.406/0.436,
`R_ask` 0) and **fails for the learned gate** (0.142/0.139). The C2′ `chose_target` half is now a
determined target that the rule follows and the gate does not. The `chose_other` column here is the
asked (salience) count, not the C2′ other-fact half; that half is measured separately by the rule
(exact on `K=1`).

## Sources

- `docs/loop/tasks/one-model-end-to-end/codex-18/DECISION.md` (approved 2026-11-16).
- `docs/loop/tasks/one-model-end-to-end/claude-397/REPORT.md` (the instrument fault).
- `docs/harness/prep/one-model-end-to-end-native-read-train-2026-11-15.md` (the arm structure).
