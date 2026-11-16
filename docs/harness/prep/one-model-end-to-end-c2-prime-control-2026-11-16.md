# C2′ control: the fact asked in the previous slot (2026-11-16)

Status: **host count PASS; GPU arm NOT RUN.** Owner approved the control replacement on
2026-11-16 (`docs/loop/tasks/one-model-end-to-end/codex-18/DECISION.md`).

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

- **Determined control.** A read trained on the previous slot's asked fact must pick it at the
  A-ext bar (0.90 per bucket); the salience read is 0.004, so the two are separated.
- **`chose_other` unchanged.** The C2′ other-fact half keeps its +0.10 over C0; the rule read is
  exact on `K=1` (35/35, 12/12, 42/42, 8/8).
- **No bar moves before the GPU arm.** The host count only shows the target is determined and the
  salience read does not follow it.

## GPU arm (pending, claude-398)

Train the native read on the previous-slot target, both ext states, seeds 1 and 2, at that state's
A-ext shift; the A-ext asked read is the salience baseline. Needs a sheet addendum and a non-author
FIT (CHARTER §4.5); no compute until then. Stop rule: if the host count's separation (1.000 against
0.004) does not survive the trainer's fit, report it and do not run the panel.

## Sources

- `docs/loop/tasks/one-model-end-to-end/codex-18/DECISION.md` (approved 2026-11-16).
- `docs/loop/tasks/one-model-end-to-end/claude-397/REPORT.md` (the instrument fault).
- `docs/harness/prep/one-model-end-to-end-native-read-train-2026-11-15.md` (the arm structure).
