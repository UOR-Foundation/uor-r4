# Geometric computation: corrected association regression, composition instrument and the readout diagnosis

September 21, 2026. Executes the [geometric-computation brief](deepseek-geometric-computation-step-2026-09-21.md) on
reviewed `origin/main` at `036e9c4310ca3d8d2440ac367c577a1df7edf58f` (PR #1336, verified equal to the
merged head). Isolation worktree `.worktrees/geometric-computation-step`; owner checkout untouched.

This run **executes the principal's repaired learner end to end for the first time** (the review states
`No new complete model fit ... is claimed by these source repairs`), adds the **served-feature
separability witness** the review required, and builds and runs a **relation-composition instrument**
for the successor capability. All populations are the already-exposed regression seeds; **no fresh
final evaluation is claimed**.

## 1. Corrected lifecycle executes and improves the deployed predictor (regression)

`--mode=contextual-emission`, roots `geometric-computation-1` and (with the new diagnostic)
`geometric-computation-2`, both sealed with 0 unlisted files.

Identical results in both runs, so the corrected path is deterministic:

| Arm | dev /180 | tune /120 | final /120 | dev pairs both /90 | final pairs both /60 |
| --- | ---: | ---: | ---: | ---: | ---: |
| local / NoRead / UpdateDisabled / ReadDisabled / scalar-copy | 0 | 0 | 0 | 0 | 0 |
| development constant | 32 | 13 | 15 | 0 | 0 |
| H4 read-conditioned (**corrected learner**) | **61** | **41** | **33** | **12** | **7** |
| matched cyclic C120 | 54 | 25 | 21 | 8 | 2 |
| categorical selected-value table | 164 | 112 | 108 | 81 | 54 |

The corrected learner is better than the pre-repair report on the same exposed populations
(H4 50/36/25 and 1/60 pairs, C120 42/25/20 and 4/60). The declared screen still **fails**:
H4 7/60 final pairs against the 54/60 table. Reader localisation is 163/180 dev, 111/120 tune,
108/120 final correct exact occurrence. The frozen-row ceiling is still 0/180.

**One committed served objective improves at every stage** (calibrated fixed-point bits):

| Stage | H4 | C120 |
| --- | ---: | ---: |
| initial | 19.8629 | 19.8629 |
| served output, before map search | 10.8529 | 10.2985 |
| served after map search | 9.3995 | 9.2976 |
| post-map refit incumbent (must equal the previous row) | 9.3995 | 9.2976 |
| served final | **7.8697** | **8.2947** |

`post_map_output_refit_executed = true`, the two output hashes differ, serving uses 64 rows with 691
nonzero ternary coefficients, 1080 accepted ternary flips (592 pre-map, 488 post-map) and 7 accepted
map changes. The pre-map to post-map refit contract held on both arms, and independent reload parity
held for every position. `distinct_payload_alias_pairs` is 0 before and after, so the collision-aware
map objective did not buy margin by merging required value distinctions.

Both loaded artifacts are exercised after reload; three loaded three-token continuations ran with
1/3 correct first answers and degenerate text (`info-f`, `-review`, `user-s`). Useful prose remains
unqualified.

## 2. The served feature does **not** alias on the association instrument

The review's unmet diagnostic was: *do source-correct examples with different targets collide in the
actual feature `R(q0*T[r]*V[v]) - R(q0)`?* Measured on the deployed residual:

| Split | source-correct positions | distinct service features | features with conflicting targets | table-on-feature ceiling |
| --- | ---: | ---: | ---: | ---: |
| dev | 163 | 50 | **0** | **163 / 163 (100 %)** |
| final | 108 | 46 | **0** | **108 / 108 (100 %)** |

A per-feature table could recover every source-correct position, while the deployed **linear**
ternary residual recovers 61/180 dev and 33/120 final. The association gap is therefore **not** a
feature-alias or value-code-injectivity failure. It is the **readout family**: the served residual is
the *linear* map `residual[o] = <W[o], d> << shift` over 16 ternary features, and no linear map of
those features reaches the feature table's ceiling.

A separate, smaller limitation is real and measured: `distinct_update_signatures` finds 8 ambiguous
`(q0, relation, selected payload)` signatures covering 21/180 dev positions (6 and 17/120 final).
Those positions demand different targets from an *identical* update input and are reader/input
ambiguity that no readout can remove.

## 3. Relation composition: instrument, obstruction and negative

New mode `--mode=relation-composition`. The answer is a function of the **query operation composed
with the retrieved content** (`class = (op + vi) mod k`), so a payload-only table cannot represent it;
held-out *value* combinations are evaluated after a development fit by all operations, and a fair
two-input table is given the same relation and payload. The declared rule is realisable by the served
algebra: `2I` contains an element of order 10 (witnessed element 74), and `T[rel(op)] = h^op`,
`V[value(vi)] = h^vi` give the served state `q0 * h^class` (recorded focused tests).

**Obstruction (measured).** The served geometric relation interface exposes only **2 distinct directed
relative elements** across all 14 declared role pairs (`mode = Geometric`, descriptor roots are rich —
119, 113, 82, 0, 77, 9, 78, 84, ... — but `inverse(root_q)*root_k` collapses to {80, 85}). An
eight-operation composition is therefore **not representable** through `T[rel]`; the instrument
records `relation_budget_saturating = true` and the full relation-exposure table rather than
asserting a wider budget.

At the reduced two-operation budget (class modulus 7, 4 repeated contexts per cell):

| Arm | dev /48 | held-out /16 |
| --- | ---: | ---: |
| local / NoRead, UpdateDisabled, ReadDisabled | 0 | 0 |
| H4 composition | 15 | **0** |
| matched cyclic C120 | 15 | 0 |
| development constant | 8 | 4 |
| payload-only table (blind to the operation) | 24 | 4 |
| relation+payload two-input table | 48 | 4 |

No held-out cell reached a state produced in development (`held_out_states_already_reached_in_dev = 0`),
so no compositional code was discovered and the declared screen is not met. Here the feature table
ceiling is 42/46 (91 %) while the linear residual reaches 15/48: the same **readout** limitation, on a
task where the relation budget is also insufficient.

The earlier eight-operation construction (`relation-composition-1`, retained) shows the consequence of
the collapse directly: 44 source-correct positions produce only **10 distinct service features**, and
the feature-only ceiling is 12 — exactly the 12 that arm achieved. That arm is feature-limited *because*
its declared operations collapsed at the relation interface, not because of readout capacity.

## 4. Interpretation and next cause

Three separate findings, in the order the principal separated them:

1. **Readout (lexical emission)** is the binding constraint on both instruments: the served linear
   ternary residual falls far below the ceiling of a table on the very same feature (163 to 61 dev;
   42 to 15 dev). The `categorical_selected_value` control's 108/120 is a *table* result. The brief's
   option B — a **learned lexical decoder given the actual selected/derived state, with no target or
   golden source at serving** — is the justified next component, not more geometric width.
2. **Addressing/relation** is the binding constraint for *multi-operation* composition: the served
   relation interface returns two distinct relative elements for fourteen role pairs, so an operation
   cannot be routed into the update. A richer relation/operation signal (an explicit operation operand
   in the update, or a relation learner trained to expose the required distinctions) is required
   before a multi-operation composition can be tested. This is the brief's option C trigger, with a
   concrete witness.
3. **Representation** (feature aliasing) is **not** the association bottleneck. The principal
   correction to the earlier expressivity diagnosis is supported for that instrument.

## Exclusions and scope

Bounded authored instruments, not general language or reasoning. All populations are exposed
regression seeds; the composition held-out values are new *cells* but the instrument, rule and output
bank were authored, and the declared rule was constructed to be realisable by the group composition,
so a pass would be a composition-circuit result, not a language-advantage claim. The
composition-arm negative is at 48 development / 16 held-out positions and is exploratory. Energy
`UNAVAILABLE`; whole-path D0-b not claimed. Sealed roots
`.uor-models/realtext-prior-2026-09-20/{geometric-computation-1,geometric-computation-2,relation-composition-1,relation-composition-5}`
(0 unlisted each); the empty claim directories `relation-composition-{diag,diag2,3,4}` hold no evidence.
Resources and charges: [resource ledger](resource-ledger-2026-09-19.md).
