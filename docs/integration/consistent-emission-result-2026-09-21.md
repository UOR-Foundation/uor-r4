# Consistent geometric emitter: corrected contract, decisive comparator, no geometric transfer on this task

September 21, 2026. Executed from reviewed parent `f07568f8` (PR #1335). [Contract](consistent-emission-design-2026-09-21.md);
[principal review](contextual-emission-review-2026-09-21.md); [prompt](deepseek-consistent-emission-step-2026-09-21.md).
Delivered root `.uor-models/realtext-prior-2026-09-20/consistent-emission-2` (sealed, verified, 0
unlisted, 6 files incl. `rows.jsonl`). Diagnostic `consistent-emission-1` is retained unchanged.

## Decision

The learner is now numerically and causally consistent and preserves the required payload distinctions,
but it **does not solve the context-required task**, and the required control shows why: a plain
**eight-entry categorical selected-value table**, learned from development and using only the observed
selected payload, scores **108/120** on the untouched final draw with **54/60** pairs fully correct,
while the geometric emitter scores **25/120** and **1/60**. The declared screen (≥ 50 % paired, beating
the controls) is **not met**, and the geometry adds **no transfer** beyond finite value association on
this instrument.

## Corrected contract actually executed

Calibrated fixed-point units (`f_bits = 10`) in both the float objective and the served evaluation
(old "15561 bits" → **19.86 → 10.11** bits); residual **shift fixed before fitting**; **ternary
straight-through** training against the deployed map; **one target-free reader** (`read_step`) at
extraction and serving with intended vs selected occurrence/value recorded separately; **injective
value codes** plus a **collision-aware map objective**; **independent artifact reload before
evaluation** with fail-closed full-predictor parity; fresh development/tune and one **untouched** final
population (`0xC0F00011/12/21`; the old seeds are exposed regression data).

## Results on identical rows

| Arm | dev /180 | tune /120 | final /120 | dev pairs both /90 | final pairs both /60 |
| --- | ---: | ---: | ---: | ---: | ---: |
| local / NoRead, UpdateDisabled, ReadDisabled | 0 | 0 | 0 | 0 | 0 |
| scalar-copy parent | 0 | — | 0 | 0 | 0 |
| development-fitted constant | 32 | 13 | 15 | 0 | 0 |
| **H4 read-conditioned** | **50** | **36** | **25** | **5** | **1** |
| matched cyclic C120 | 42 | 25 | 20 | 6 | 4 |
| **categorical selected-value (headline control)** | **164** | **112** | **108** | **81** | **54** |

Reader localisation: correct exact occurrence **163/180** dev, **111/120** tune, **108/120** final
(~90 %); correct payload value within one. Frozen-row ceiling **0/180**. Collisions from the selected
payload: 36 → 36 dev (the residual collisions come from the ~10 % wrong-source positions, not from
value-code merging, which the injective initialization and collision-aware objective prevent).

## Diagnosis

| Candidate | Evidence | Verdict |
| --- | --- | --- |
| Source selection | ~90 % correct occurrence on every split | secondary, not the main gap |
| Value-code merging | injective codes forced; collisions unchanged by the search | repaired |
| Units / shift / reader / loading | calibrated bits, shared shift, one reader, loaded parity | repaired |
| Emitter expressivity on this task | table 108/120 vs H4 25/120 with the *same* observed inputs | **the concrete failure** |

The task's required output is an **arbitrary eight-value association** applied to the selected payload.
A direct table over the observed payload captures it; routing the same value through a ternary state
embedding, a ternary transport and a ternary shared residual does not, at this width. This is a
representational/optimisation limitation of the geometric path **on this task**, not the withdrawn
width diagnosis, and not evidence about geometry in general: it says nothing about tasks whose required
output is a *composition* rather than a per-value lookup.

## Retained and next

Retained: the corrected numerical/causal contract, ternary straight-through training, the
collision-aware objective and value-distinction preservation, the loaded-artifact evaluation and
generation path, both algebra artifacts (`h4_emission`, `cyclic_c120_emission`), the development
constant and categorical comparators, the per-position `rows.jsonl`, and all failed criteria.

**Successor (evidence-supported):** test the geometric emitter on a task whose required output is a
**composition** of the retrieved evidence — e.g. the output class determined by the *relation between*
the query role and the retrieved role, or a typed derived token absent from all payloads — because an
arbitrary per-value association is already solved by a table and therefore cannot discriminate geometry.
Width expansion remains unjustified. Generated continuations and text integration are NOT_RUN.

Resources and charges: [resource ledger](resource-ledger-2026-09-19.md). Energy UNAVAILABLE;
whole-path D0-b not claimed.
