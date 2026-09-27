# Read-side localization: result

September 27, 2026 (run 2). References #973 under #820. OpenCode/DeepSeek lab.
**Status: executed; predeclared outcome MIXED, with a post-hoc refined signature
(READ_RANKING) at the stated scope.** Read-only; no training, no weight change.

Plan and frozen thresholds: [`read-localization-plan-2026-09-27.md`](read-localization-plan-2026-09-27.md).
Instrument: `crates/uor-r4-training/examples/joint-read-localize.rs` +
`crates/uor-r4-training/src/read_localize.rs` (registration in `lib.rs`; no other shared
change). Evidence: [`docs/evidence/read-localization-2026-09-27.json`](../evidence/read-localization-2026-09-27.json).

## What was executed

Frozen parent `fit-quaternion-6/checkpoint-final` (step 15,672), read mode Enabled,
deterministic greedy with the frozen stop rule, the 32-row source panel under five
declared conditions: A baseline; B entity-final; C matched non-entity final; D
near-query entity mention (generation target unchanged); D′ near-query matched
non-entity mention. Decision-0 read evidence: the full read mass row over causally
available occurrences with token identity, NoRead mass, copy gate, effective copy mass,
entity span/mass/share/rank, and the vocabulary/copy mixture.

**Guardrails:** baseline parity `PARITY_EXACT` on all 32 rows (prompt ids, generated
ids, stop, text, both verdict flags, and decision-0 read fields against the retained
parent `story-probes.json`); masses normalize in every evaluated row; all reads causal;
all spans resolved and unambiguous; mutations checked against their intended decoded
text; one D/D′ row declared length-mismatched (`15|edited`). Full run 1.29 s in-process
(smoke 0.2 s) — evaluation-scale.

## Result by class (complete answers per condition)

| Class | Rows | A | B | C | D | D′ |
|---|---|---|---|---|---|---|
| Distractor copy (`clouds.`) | 5 | 0/5 | 0/5 | 0/5 | **5/5** | 0/5 |
| Correct noun + extra phrase | 5 | 0/5 | 0/5 | 0/5 | 3/5 | 4/5 |
| Morphology (`pears.`) | 1 | 0/1 | 0/1 | 0/1 | 0/1 | 0/1 |

## Predeclared outcome: MIXED

The frozen READ_ACCESS_LIMITED clause requiring the entity share < 5% with rank > 5 in
at least four distractor rows **failed** (0 of 5: the entity was present at rank 2–3
with 16–32% share). Its third clause also failed: condition D raised the entity
share/rank in 5 of 5, but the matched control D′ did so in 5 of 5, so the entity-share
metric is non-discriminative. STATE_EMISSION failed because D changed the verdict in
5 of 5. Per the frozen rule the outcome is **MIXED**; the rule is not relaxed.

## Post-hoc refined signature (not the predeclared verdict)

In all five distractor rows the emitted token equals the **read top-1 token**
(` clouds`), while the correct entity is a readable event at rank 2–3 (share
0.16–0.32). Adding a near-query **entity** mention (D) moves the entity to rank 1–2
(share 0.39–0.64) and completes 5/5; adding a matched **non-entity** mention (D′) makes
the model emit the inserted noun (` carrot`) in 5/5. The emission is therefore a faithful
readout of the read ranking — the distractor failures are a **read-ranking** failure
(the learned age/recency prior prefers the later `clouds` occurrence over the correct
entity 50–70 tokens back), not entity absence and not an emission-stage defect.

Two further observations bound this: the extra-phrase class is **not** entity-specific
(D 3/5 vs D′ 4/5), consistent with the earlier termination-weighting INERT result; and
the single morphology row emits the correct singular noun at rank 1 yet fails the
completion, so it is a separate emission/morphology issue.

## Decision and next step

Instrument retained; nothing promoted. The decision-relevant finding is the read-ranking
signature for the distractor class, with the caveat that D makes the entity the most
recent noun, so the flip demonstrates recency/ranking sensitivity but not that a
long-range ranking repair is learnable. Recommended successor (bounded,
evaluation-scale): an **oracle read re-rank intervention** — clamp the decision-0 read
mass onto the entity occurrence and check whether the distractor rows flip — which would
causally confirm the read ranking as the sole bottleneck before any mechanism change.

## Limits

Exposed 32-row development panel, one parent, one deterministic policy; no fresh
holdout, no training, no weight or serving change. The predeclared rule returned MIXED;
the READ_RANKING signature is a post-hoc refinement at this exact scope. It does not
qualify general language or a mechanism family.

## Cost

Instrument implementation, focused checks, smoke and full run are charged once with the
milestone at delivery; the run itself is 1.29 s of model time. A prospective ledger
extension (+12,000,000 ms; limit 756,000,000 ms) was recorded before compute under the
standing owner authorization.
