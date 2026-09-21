# Grounded reusable computation in an owned, resumable dependent session

September 21, 2026. Executes the [grounded dependent-session brief](deepseek-grounded-dependent-session-step-2026-09-21.md) and
the [shared-transition review](shared-transition-review-2026-09-21.md) on merged `85df1739` (PR
#1339, verified equal to the merged head). Isolation worktree `.worktrees/grounded-dependent-session`;
owner checkout untouched. New module `learner/grounded_session.rs` and mode `--mode=grounded-session`.
[Evidence](../evidence/grounded-session-2026-09-21.json).

## Accepted corrections

- The reversal failure is **computational/learning failure in that population**, not reader
  confounding: every development and reversal example selects the correct source. The earlier
  "confounded by the reader" attribution is withdrawn.
- The development probe was value-index parity with every program in both halves, i.e. supervised
  value-disjoint fitting, not sequence-disjoint validation. Not reused as evidence here.
- The whole-program cache was not a competent transition baseline. The competent ordinary comparator
  is the shared recurrent table, and it is now the control.
- The `remaining == 0` stop table is **explicit program completion**, not learned semantic
  continuation.

## Part A - constructive factorization of the observed transition graph

The declared development labels give one permutation of the typed outcomes per observed primitive.
Those permutations are bijective, distinct and closed (order spectrum 1 x 1, 1 x 2, 6 x 4 -> Q8). An
exhaustive search finds an isomorphism into a **witnessed** quaternion subgroup of the served table;
outcome coordinates come from a reference-state orbit and initial coordinates from
`inverse(A[p]) * Z[first outcome]`, checked across every observed primitive. The reference outcome
fixes the gauge.

```text
Z[next outcome] = A[observed primitive] * Z[current outcome]        holds by construction
E[selected payload] = inverse(A[p]) * Z[first outcome of payload under p]
```

1,440 per-arm events were saved from the scoring pass. Every one of the **672 declared development
transitions is reproduced exactly**, and the factorization matches the competent shared finite-state
comparator on all three panels while carrying far fewer bytes:

| Arm | dev /288 | unseen length 4 /128 | unseen reversal /64 | artifact bytes |
| --- | ---: | ---: | ---: | ---: |
| fitted shared recurrence (previous milestone) | 248 | - | - | 370 |
| competent finite-state control | 288 | **124** | **64** | 1668 |
| **grounded factorization (this run)** | **288** | **124** | **64** | **102** |

The review's diagnosis is confirmed: the earlier learner had not recovered structure already present
in its supervision, and a constructive geometric factorization recovers all of it. This is a
compression/sharing result on supplied intermediate supervision, not language capability.

## Part B - an owned, resumable session frame

`SessionFrame` carries an exact evidence lease (sequence, absolute position, payload), the retained
computed state and outcome, response phase, emissions and a typed terminal status. `read`,
`apply_primitive`, `emit` and `stop` are separate typed operations; the frame serializes (`RLSF` v1).
Tests and the run show a paused frame resumes to an identical result (4/4 chains) and that advancing
one frame does not change an unrelated one. A lease is checked against the live ring, so a different
sequence or an overwritten payload is explicitly stale rather than silently reused.

## Part C - a genuinely dependent read

Evidence is a set of `(role, key, value)` records. The first key is the request key; the second hop's
keys are the **typed outcome labels**, so the first computed result forms the second query. The query
is issued as `(role, key)` with no declared previous context, so candidate admission is driven by the
key alone.

| Measurement | Result |
| --- | ---: |
| first hop correct | 4 / 4 |
| second occurrence selected correctly | 4 / 4 |
| complete answer correct | 4 / 4 |
| paused/resumed chains identical | 4 / 4 |

Causal controls:

| Control | Observed |
| --- | --- |
| changed first source | first outcome, **second query key**, selected second occurrence and final answer all change together |
| added irrelevant record | first outcome, second selection and answer all preserved |
| **required record removed** | `read = false`, terminal `NoRead`, no answer |
| reader disabled | `read = false`, terminal `NoRead` |
| unknown primitive / payload | typed `UnknownPrimitive` / `UnknownValue` |

The final answer is a typed outcome label - a disjoint alphabet from every record payload - so it is
**absent from all source payloads**: the answer is derived, not copied.

## Disagreements and deviations

- I did **not** adopt the brief's illustrative `U[op] = h^(hidden index)` or any fixture-rule
  seeding. The isomorphism is found from declared labels and the gauge is fixed by a reference
  outcome, which is representation normalisation rather than a semantic label.
- The brief's non-invertible control operators are kept separate: only the ordered product is a group
  operation; read, commit, emit, exhaustion and the unknown/absent outcomes are typed transitions.
- A useful deviation: rather than treat the finite table as a rival, it is retained as the
  **equal-supervision control**, and the factorization is evaluated against it. Where the table wins I
  say so; here the factorization matches it with 102 bytes against 1668, on the same supervision.

## Not established

Supplied record layout and a supplied primitive program; the instruction span is declared structure,
not a learned parser. Completion is explicit program exhaustion, not learned continuation. The second
hop is exact-match key retrieval over typed outcome labels, not semantic query formation. No prose,
no executed Rust task, no whole-path energy measurement. Energy `UNAVAILABLE`; whole-path D0-b not
claimed. Retained roots `grounded-session-{1,2,3,4}` (0 unlisted each), with `-4` delivered.
