# Pre-registration — Stage-A K-sweep re-score

## 5.1 Header

- **Date / owner:** 2026-10-17 / Kimi PM (kimi/pm1), drafted by DeepSeek design seat
- **Authority:** [Stage-A sweep record](../../labs/rung0-stage-a-sweep-2026-10-15/README.md); [Stage-A instrument](../../labs/stage-a-admission-instrument-2026-10-15/README.md); [admission sheet](m1-discrete-rung0-admission-2026-10-15.md); [M4 sheet](m4-finite-relational-head-2026-10-15.md).
- **Line:** M1 qualification; count **0/3**, suspended headline **37/232**, unchanged. No mechanism negative, KEEP or count reset.
- **Delivery:** docs only; all new observations `NOT_RUN`. This sheet is a re-analysis design, not a training release.
- **Scope:** diagnostic. Training/M2/Stage B **HELD**; no K>8-trained lineage is run. FIT releases no hold.

## 5.2 Question and smallest falsifiable hypothesis

**Question.** At the tested scope — the four preserved seed-1 quarter checkpoints, `C192 / B4 / V4096`, `ReadSpec::Route`, K=8..16, frozen 14,287-row DEV panel — does the sealed Stage-A 36-point table reproduce, and what does the trained ceiling at K=9..16 consist of? The aggregate ceiling is already known (2,400 at K=16/Q4; 3,242/3,206/2,509 at the earlier quarters); the open questions are composition and decision relevance.

**H1 (reproduction).** The sealed 36-point record reproduces exactly, including the four K=8 stored-value controls (`ALL_FOUR_EXACT`), and a host re-analysis of its per-row records can be used without a new GPU run.

**H2 (admission-only widening).** Widening K from 8 to 16 adds target-admitted rows (1,920 → 2,400) but does **not** improve conditional top-1 within admitted rows by ≥ the noise floor. The K-widening trade-off is admission-only; a K>8-trained lineage would not automatically convert the extra admission into hits.

**H3 (sub-panel divergence).** The 3,847-row copy-successor sub-panel and the 10,440-row complement respond differently to K; the Stage-B K choice should be panel-weighted, not a single full-panel count.

**Disconfirming observations.**
- H1 is disconfirmed by any K=8 control mismatch or any of the 36 points not reproducing; that is an instrument fault (D31), not a finding.
- H2 is disconfirmed if conditional top-1 within admitted rows rises with K by ≥ the paired 2-SE floor.
- H3 is disconfirmed if the sub-panel deltas are within the 39-row floor.

## 5.3 Observable signature, controls and design

**Observable signature.** Per (checkpoint, K) cell: exact integer counts of `top1_correct`, `top8_correct`, `strict_wins`, `admitted_strict_wins`, `target_admitted_rows`, plus per-row joins to `P_succ`, `P_comp`, `P_any_succ`, the four distance buckets, and the admitted/conditional split.

**Strongest simple control.** The four K=8 stored-value cells. They must reproduce the sealed quarter counts exactly: 96/1090/177/41/2201; 94/967/178/42/2131; 96/953/214/44/1954; 149/747/105/65/1920 (`rung0-stage-a-sweep-2026-10-15/README.md:50-64`). If they do not, the re-analysis is invalid.

**Primary design (host-only, $0 GPU).**
1. Fetch the sealed archive `icloud:UOR-R4/results/kimi/stage-a-sweep-1-20261015.tar` (MD5 `b274beb321ce9232bd3690659fee7890`, 33,802,240 bytes); verify the manifest and all 38 blake3 hashes.
2. Parse `sweep-1/report.json` and the 36 `evaluation-*.json` records (33,627,092 derived bytes). Each record is row-lossless (`row_columns`/`rows`); join by row ordinal.
3. Re-derive the 36-point table; verify it against `rung0-stage-a-sweep-2026-10-15/README.md:72-109`.
4. Join to the DEV partition by ordinal; verify `ordered_row_binding_sha256 = e4ef44fb…` and the `P_succ`/`P_comp` counts (3,847 / 10,440 / 1,335).
5. Produce the per-K, per-checkpoint, per-sub-panel table with exact integer counts, Wilson intervals, and paired K-vs-K+1 discordance.
6. Verify per-row monotone target admission across K (the sealed record already did this; re-check).

**Fallback design (GPU, only if the sealed record is missing or fails verification).** Re-run the 36 evaluations with the reviewed Stage-A instrument (executable `712cd0cc…`, source `719f38d9…`, compiled model `e1836149…`, kernel `931a511b…`, geometry `46ddbb76…`) on the same four checkpoints, same 14,287 DEV rows, K=8..16, 224 batches each, zero-apply. <15 GPU-minutes, <$0.25. This is a **record-recovery** run, not a new ceiling measurement.

**Dose.** 36 host re-analyses, or 36 GPU evaluations in the fallback (4 checkpoints × 9 K values). No training; no K>8-trained lineage; no checkpoint mutation.

## 5.4 Bars and decision rule

**Bars.** 2-SE floors: 20 rows (top-1), 63 rows (top-8), 87 rows (B3) on the full panel; 39 rows on the 3,847-row `P_succ` sub-panel. Paired discordance `c − b` with null SE `sqrt(b+c)`. Wilson 95% intervals for every cell. The G-A gate 4,894 is reported as the reference, not re-litigated.

| Observation | Interpretation | Next decision |
| --- | --- | --- |
| All 36 points reproduce; four K=8 controls exact | The sealed record is valid; no new GPU information from a re-run | Use the host re-score for the Stage-B K choice; no GPU re-run |
| Sealed record unavailable; the 36-evaluation fallback reproduces the table | The ceiling is confirmed; the record is repaired | Same |
| K=16/Q4 ceiling < 4,894 (confirmed at 2,400) | Eval-time widening is closed at this scope | A K>8-trained lineage (Stage B) is the untested successor; separate pre-reg and owner release |
| Conditional top-1 within admitted rows flat or falling with K | Widening K buys admission but not prediction | Stage-B K choice should not be a simple K widening; consider the address/selection policy |
| Conditional top-1 rises with K by ≥ the paired floor | Widening K buys prediction too | Stage-B K choice can use the smallest K that maximizes the panel-weighted objective |
| `P_succ` and `P_comp` diverge by ≥ 39 rows | The sub-panel composition matters | Report both; weight the Stage-B decision by the intended panel |
| Any K=8 control mismatch, hash/row-binding mismatch | INSTRUMENT FAULT | Diagnose/repair (D31); not a finding |
| Sealed record missing and the GPU fallback unavailable | UNAVAILABLE | Do not infer a ceiling; no Stage-B release |

**What changes the decision.** The number that matters is not the ceiling (already 2,400 vs 4,894) but the **conditional top-1 slope with K**. If it is flat, the Stage-B design must change the admission/selection policy, not just K. If it rises, a wider-K Stage-B lineage is justified.

## 5.5 Instrument validity, lineage, resource projection, stop conditions

**Instrument validity.** The K=8 controls are the qualification; any mismatch is an instrument fault (`stage-a-admission-instrument-2026-10-15/README.md:20-38`). The archive hashes and the 38-file manifest must verify before analysis. The per-row monotone-admission check must pass. The DEV row binding must match `e4ef44fb…`. The Q4 evaluation receipt must match `4fff1d3c…`.

**Loaded artifacts.** The four checkpoints (`a5a22dce…`, `dde889aa…`, `102beed4…`, `4477eb5c…`); the 36 evaluation records; the DEV panel (`0919e90f…`, `a838bb1a…`). No generation; teacher-forced scoring only.

**Lineage.** The sealed archive, the 36-point table, the Stage-A instrument identity, the Q4 receipt, the DEV binding. The archive's 38 blake3 hashes and the cloud MD5 round-trip are the preservation record.

**Resource projection.** Host-only re-analysis: **$0 GPU**, seconds to minutes of host work, <50 MiB derived records. GPU fallback: <15 GPU-minutes, <$0.25, one 1-hour lease at most. No training; no new spending class.

**Stop conditions.** First hash/manifest/row-binding/K=8-control mismatch stops the analysis and preserves the partial record for PM disposition. The GPU fallback stops on identity, parity, epoch/apply, resource or disk-floor faults. No on-lease patch or extra sample.

**Record schema.** `uor-r4.stage-a-rescore/1`, with `applied_updates: 0`, `model_compute_executed: false` (host) / `true` (fallback), per-row records and exact aggregates, source/checkpoint/evaluation hashes, and the `P_succ`/`P_comp` partition binding.

## 5.6 D22, claim discipline, FIT and assumptions

**D22.** The G-A failure already closed **eval-time widening of the K=8-trained codebook** — a configuration, not a mechanism. A K>8-trained lineage remains an untested configuration. This re-score cannot reopen the closure and cannot close the mechanism.

**Claim discipline.** No capability claim. The re-score is a composition analysis of a sealed diagnostic, not fresh transfer and not a Stage-B result.

**FIT / execution release.** Design FIT only; no compute. The host re-analysis is non-model work; the GPU fallback needs a separate PM release. Training remains HELD.

**Assumptions the lead must resolve.**
1. Whether the iCloud archive is accessible and current; if not, whether a GPU fallback is authorized.
2. Whether the sealed per-row records contain the fields needed for the sub-panel join (the README says row-lossless; I did not open the archive).
3. Whether the PM wants the host re-analysis only, or a GPU replication as a record-integrity control.
4. Whether the Stage-B K choice should be panel-weighted by `P_succ` or by the full panel.
