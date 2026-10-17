# Pre-registration — D34 successor-table control

## 6.1 Header

- **Date / owner:** 2026-10-17 / Kimi PM (kimi/pm1), drafted by DeepSeek design seat
- **Authority:** [D34](../../integration/DECISIONS.md:1642-1660); [finite-relational-head](../../mechanisms/finite-relational-head.md); [M4 sheet](m4-finite-relational-head-2026-10-15.md); [relation sub-panel audit](../../labs/relation-subpanel-audit-2026-10-15/README.md).
- **Line:** starling read / exact addressed memory; M4 count **3/3** unchanged; M1 count **0/3** unchanged. No mechanism negative, KEEP or count reset.
- **Delivery:** docs only; all new observations `NOT_RUN`. The Relation arm is blocked on an artifact that does not exist.
- **Scope:** diagnostic, not a capability claim. Training/M2/Stage B **HELD**. These are not D-Release-class decisions; no owner hold changes.

## 6.2 Question and smallest falsifiable hypothesis

**Question.** Does the geometric Relation read beat a **model-free successor table** on the rows it should own — the 3,847-row copy-successor sub-panel (26.9266% of DEV) — at the same admission and the same training data?

**H_geom.** On the 435 `P_succ` rows where the Relation read admits the target cell (`B3`), its top-1 exceeds the strongest **context-free** TRAIN successor table by at least the paired noise floor.

**H_table.** The table matches or beats the Relation read on those rows; the geometry adds nothing beyond a token→successor lookup at this configuration.

**H_null.** Neither beats the global unigram; the sub-panel is not informative at this scale.

**Disconfirming observations.**
- H_geom is disconfirmed if the table ≥ Relation on the 435 matched rows (within the paired floor).
- H_table is disconfirmed if Relation ≥ table + floor.
- H_null is disconfirmed if either beats 1,226-class unigram accuracy on the sub-panel.

## 6.3 The saturation exposure (the most important design correction)

The proposed context-local "most-recent-successor table" is **tautological** on `P_succ`:
- `P_succ` is defined as: target = `tokens[j*+1]` where `j*` is the most recent earlier occurrence of the query token in the window (`relation-subpanel-audit-2026-10-15/README.md:24-25`).
- The context-local rule applies exactly that definition. It scores **3,847/3,847 = 100%** on `P_succ` by construction.
- The Relation read's own `SequentialSuccessor` edge also selects the most recent prior matching node (`discrete.cu:199-200`), so the panel is definitionally aligned with the mechanism.

Therefore:
- **T1 (context-local rule) is a panel-definition check, not a comparator.** It must be reported as 3,847/3,847 (or a fault if not), and it must never be used to decide that "the geometry adds nothing".
- The decisive comparator is **T2**, a **context-free** table built from TRAIN: for each query token `x`, the most frequent successor `y` in the same 1,049,856 TRAIN targets the Relation arm consumed. T2 does not see the DEV window. It is an ordinary, model-free, information- and cost-matched comparator.
- The informative rows are the **1,335 conflict rows** (`P_any_succ \ P_succ`, where the target is the successor of some earlier occurrence but not the most recent) and the **10,440 complement rows**, not the definitionally aligned `P_succ` rows. Report all three; the decision uses the 435 matched `P_succ` rows plus the conflict/complement rows as secondary.

## 6.4 Controls and matching

1. **T1 context-local most-recent rule** — tautological check; expected 3,847/3,847. Not a comparator.
2. **T2 context-free most-frequent-successor table** — primary ordinary comparator; built from the exact 1,049,856 TRAIN targets; 4,096 entries; tie by lowest token id; unigram fallback for unseen query tokens.
3. **T3 last-seen-successor table** — secondary deterministic table: for each query token, the successor of its last TRAIN occurrence.
4. **T4 query-token bigram** — identical to T2 if T2 is the most-frequent successor; report the same table under both names to avoid double counting.
5. **T5 global unigram** — 1,226/14,287; the "neither beats the marginal" check.
6. **Admission-matched comparison** — restrict the table's prediction to the Relation read's admitted cells per row (from `Forward::admitted_cells`, `device.rs:307-308`); compare on the 435 rows where the target cell is admitted. This is the only matched denominator.
7. **Data-matched** — the same 1,049,856 TRAIN targets the Relation arm consumed, not the full 2,425,925.
8. **Cost-matched** — host table (seconds) vs one zero-apply Relation forward (minutes).

## 6.5 Design and dose

**Host-only now ($0 GPU).**
1. Build T1/T2/T3/T5 from the pinned TRAIN (`3dee858f…`, `b5e7352d…`); build the DEV partition from `0919e90f…` / `a838bb1a…`; verify 3,847 / 5,182 / 10,440 / 1,335 and buckets `[50, 2872, 900, 25]`.
2. Score T1/T2/T3/T5 on all 14,287 DEV rows and on `P_succ`, `P_comp`, the conflict rows, and the four distance buckets. Emit exact integer counts and per-row records.
3. Report T1 = 3,847/3,847 as the panel-definition check.

**Relation arm (blocked).**
- No frozen Relation checkpoint exists. `ReadSpec::Relation` is implemented (`discrete.cu:78-267`; `device.rs:341-356`; `lib.rs:76-79`, `116-127`, `140-153`; `relation_qualification.rs:1-120`), but the M2 trained-in discriminator is `HELD` behind G1–G5 (`m4-finite-relational-head-2026-10-15.md:8`, `86-100`).
- When a frozen Relation checkpoint at cursor 16,404 exists, run one zero-apply `Device::forward` over the 224 DEV batches with `ReadSpec::Relation { kept: 8, sweeps: 1 }`; capture `predictions` and `admitted_cells` per row; join by ordinal; compare on the 435 matched `P_succ` rows and the 3,847 full sub-panel. Cost 5–15 GPU-minutes, <$0.25.
- If the Relation artifact does not exist, the arm is **`UNAVAILABLE`**, not a negative.

**Optional interim Route arm (executable now).**
- The sealed Q4 Route checkpoint (`4477eb5c…`) already has 29/435 conditional successor top-1 and 20 strict wins (`relation-subpanel-audit-2026-10-15/README.md:61-64`).
- A zero-apply `Device::forward` over the 224 DEV batches gives the per-row admitted cells and predictions; the same table comparison on the 435 Route-admitted `P_succ` rows is executable now. Label it a **Route-configuration** result, not a Relation result. It answers "does the current trained geometric read beat a context-free table on the rows it admits?", not "does the Relation mechanism?".

## 6.6 Bars and uncertainty

- **435-row matched set.** Paired discordance `c − b`; null SE `sqrt(b+c)`; bar = `max(2·sqrt(b+c), 20 rows)` (the 20-row floor is the full-panel top-1 floor; on 435 rows it is conservative). Report the exact 2×2 table.
- **3,847-row full sub-panel.** 39-row floor (`m4-finite-relational-head-2026-10-15.md:30-31`); report end-to-end counts (29/3,847 for Route; the Relation's number when available) and the table's admission-restricted upper bound.
- **Conflict rows (1,335).** Report exact counts; no pre-registered bar above the paired 2 SE, because this is a diagnostic split.
- **Complement (10,440).** Report exact counts; the 63-row top-8 floor applies if a top-8 claim is made.
- **Wilson intervals** for every rate.

## 6.7 Decision rule

| Observation | Interpretation at this configuration | Next decision |
| --- | --- | --- |
| Relation > T2 + floor on the 435 matched rows | H_geom: the geometry adds information beyond a context-free successor lookup | Proceed to the M2 B1/B2/B3 bars; no capability claim yet |
| T2 ≥ Relation on the 435 rows | H_table: the geometry adds nothing on the rows it should own | Close the `(Relation, kept=8, sweeps=1, C192/B4/V4096, P_succ)` configuration (D22); report the table as the strong baseline; owner decides whether to change the read or the panel |
| Both ≤ unigram on the sub-panel | H_null: the sub-panel is not informative at this scale | Diagnose the panel/table; do not infer a mechanism negative |
| T1 ≠ 3,847/3,847 | Panel-definition/instrument fault | Diagnose; do not use the panel |
| Relation arm unavailable | Commissioned comparison cannot execute | Lead resolves: authorize M2, accept the Route interim, or defer |
| Hash/partition/epoch fault | INSTRUMENT FAULT | Diagnose/repair; not a negative |

**What changes the decision.** The single number is the paired difference on the 435 matched `P_succ` rows. If the table wins, the configuration is closed and the mechanism family stays open (D22). If the Relation wins, the M2 discriminator proceeds. If the artifact is missing, no decision can fire — that is the current state.

## 6.8 Instrument validity

1. **Partition exactness.** 14,287 eligible; 3,847 `P_succ`; 5,182 `P_any_succ`; 10,440 `P_comp`; 1,335 conflict; buckets `[50, 2872, 900, 25]`; zero at `d=1`; `ordered_row_binding_sha256 = e4ef44fb…`.
2. **T1 identity.** T1 must score 3,847/3,847 on `P_succ`; any deviation is a partition/instrument fault.
3. **Table conservation.** T2/T3 counts sum to the 1,049,856 matched TRAIN targets; no token ≥ 4,096; deterministic tie/fallback.
4. **Relation forward parity.** If the Relation arm runs, its per-row predictions/admitted cells must match the sealed M2 evaluation receipt; snapshot/epoch unchanged; `updates_applied: 0`.
5. **Admission match.** The table's admitted-restricted prediction must use the same 8 cells as the Relation read on each row; no host recomputation of admission (D24).
6. **D23/D25/D32 ratchets** on any new source; the Relation construction controls C1–C8 remain the finite qualification (`relation_qualification.rs:1-120`).

**Which negatives only trigger diagnosis.** A T1 deviation, a partition/hash mismatch, a table-conservation failure, a Relation forward parity failure, or an epoch/apply fault is an instrument fault and triggers repair, not a mechanism negative. A table ≥ Relation result closes a **configuration**, not the relational attention family; the owner decides on stopping the mechanism.

## 6.9 Loaded artifacts, lineage and resource projection

| Item | Value |
| --- | --- |
| Base | `origin/main = d325c3037b5a6cebdf9401636f9ac0cf91a259fe` |
| DEV tokens / mask | `0919e90f…` / `a838bb1a…` |
| TRAIN tokens / mask | `3dee858f…` / `b5e7352d…` |
| Tokenizer | `d36d3e87…` |
| Partition | 3,847 / 5,182 / 10,440 / 1,335; buckets `[50, 2872, 900, 25]` |
| Q4 Route checkpoint | `4477eb5c…`; evaluation `4fff1d3c…`; Route `P_succ` 29/435 conditional, 20 strict |
| Relation implementation | `discrete.cu:78-267`, `:166-218`, `:270-273`; `device.rs:341-356`, `1231-1246`, `1512-1514`; `lib.rs:76-79`, `116-127`, `140-153`; `relation_qualification.rs:1-120` |
| Stale brief | `docs/mechanisms/finite-relational-head.md:183` — "zero code exists" is false |
| D34 | `docs/integration/DECISIONS.md:1642-1660` |

**Resource projection.** Host table: **$0 GPU**, seconds, <10 MiB records. Relation arm (when released): one zero-apply forward, 224 batches, 5–15 GPU-minutes, <$0.25, 1-hour lease at most. Optional Route interim: the same forward on the sealed Q4 checkpoint, <$0.25. No training; no new spending class.

**Stop conditions.** First partition/hash/T1/epoch fault stops the arm and preserves the partial record. The Relation arm stops on parity, identity, apply, resource or disk-floor faults. No on-lease patch or extra sample.

**Record schema.** `uor-r4.successor-table-control/1`, with `applied_updates: 0`, `model_compute_executed: false` (host table) / `true` (Relation/Route forward), per-row predictions, admitted cells, sub-panel labels, and exact aggregates.

## 6.10 D22, claim discipline, FIT and assumptions

**D22.** A table ≥ Relation result closes the `(Relation, kept=8, sweeps=1, C192/B4/V4096, P_succ)` configuration, never the relational attention mechanism family. The owner decides on stopping the mechanism.

**Claim discipline.** No capability claim. The copy-successor sub-panel is a diagnostic panel; a win is not a general language or reasoning result, and the panel is definitionally aligned with the rule.

**FIT / execution release.** Design FIT only; no compute. The host table is non-model work; the Relation/Route forward needs a separate PM release and, for the Relation arm, the M2 artifact and its own gates. Training remains HELD.

**Assumptions the lead must resolve.**
1. **The Relation checkpoint does not exist.** The lead must either authorize the M2 run (a separate, larger release), accept the Route interim as the executable substitute, or defer this pre-reg. Without one of these, the commissioned comparison cannot execute.
2. Confirm that the **context-free** table (T2) is the intended comparator, not the context-local rule (T1), which is saturated by the panel definition.
3. Confirm the 435-row admission-matched set as the primary denominator, with the 3,847-row end-to-end count as secondary.
4. Confirm the 1,049,856-target data match for the table.
5. Confirm that a model-free table is an acceptable ordinary comparator under D34, even though it is not a proposed serving mechanism.
