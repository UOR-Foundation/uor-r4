# Derived-state decoding: a useful learned decoder, and the composition limits it exposes

September 21, 2026. Executes the [derived-state decoder brief](deepseek-derived-state-decoder-step-2026-09-21.md) and
the [principal review](geometric-computation-review-2026-09-21.md) on merged `fdc1607f` (PR #1337,
verified equal to the merged head). Isolation worktree `.worktrees/derived-state-decoder`; owner
checkout untouched. New mode `--mode=derived-state-decoder`. [Evidence](../evidence/derived-state-decoder-2026-09-21.json).

## Mechanism

The residual readout scores `F(q1) - F(q0)`. For any finite group and any `F`, right multiplication
permutes the group, so `sum_g (F(g*a) - F(g)) = 0`: that difference cannot supply a frame-independent
boost across a whole action orbit. This run decodes the **relative result** instead:

```text
q1 = (q0 * T_bind[r]) * U_op[observed query role] * V[value]     exact finite products
s  = inverse(q0) * q1 = T_bind[r] * U_op[query] * V[value]       frame cancels
emit = decoder[s]                                                learned token shortlist
```

`U_op` is keyed by the **observed query role token** of the actual causal prefix, never by the
fixture's hidden operation index. The decoder is a bounded learned per-state token shortlist
(<= 16 grounded states, k = 4). **A computed identity is an ordinary emittable state and is distinct
from `NoRead`**, which is the absence of a grounded read or of a learned state and preserves the
frozen local prior. This is a new, separately versioned artifact (`RLDS`/`RLRD`); the old
zero-residual interface and its artifacts are untouched.

## The decoder is useful on the association panel

New populations (seeds `0xC0F20011/12/21`); the preserved PR #1337 regression is unchanged and is
not re-fitted.

| Arm | dev /180 | tune /120 | final /120 |
| --- | ---: | ---: | ---: |
| frozen local prior | 0 | 0 | 0 |
| H4 residual readout (retained regression) | 61 | 41 | 33 |
| derived-state decoder, all factors | **166** | 80 | 91 |
| derived-state decoder, selected-value factor only | 161 | 108 | **104** |
| development-fitted selected-value dictionary | 161 | 108 | 104 |

The value-only decoder **exactly reproduces the strongest arbitrary-label dictionary** on every split
with 8 grounded states. Two things follow. First, the geometric computation can carry the lexical
association once the readout is a learned decoder on the computed state: final accuracy rises from
33/120 to 104/120. Second, the extra binding/operation factors are **nuisance on this task**: they
raise fit accuracy (166 dev) and cost held-out accuracy (91 final), so the factors the data justifies
are the ones to retain. The decoder abstains (`NoRead`) on 14/120 final positions where the state was
never grounded.

## Composition against familiar primitives

Fixture v2 (modulus 10 with a witnessed order-10 element, 8 observed-query operations, 192 development
and 64 held-out cells). A **development-internal probe** (operations 6-7, disjoint from the fit
operations, with every probe output class covered by the fit operations) supplies the generalization
pressure that a raw fit-objective lacks. Selection uses development only.

| Arm | dev /192 | held-out /64 |
| --- | ---: | ---: |
| local / NoRead | 0 | 0 |
| H4 derived state (primary, probe-first) | 103 | 8 |
| matched additive C120 | 132 | 20 |
| payload-only table | 38 | 1 |
| operation x payload table (unseen-cell fallback) | 172 | 8 |
| development constant | 24 | 8 |

Development probe hits move from **0/48 to 27/48** (H4) and 1/48 to 34/48 (C120) when probe hits lead
the objective; held-out cells move from 0/64 to 8/64 (H4). The declared screen is **not met**.
`held_out_states_shared_with_dev` is 13 of 18 distinct held-out states, so the maps do share some
states, but the held-out `(operation, value)` *combinations* stay under-constrained: each value code
is supervised only by the operations whose development mask includes that value, so the even-operation
combinations the fixture withholds are free parameters the fit never sees. The matched **additive**
C120 arm is stronger than H4 here, which is expected because the declared rule is itself additive in
`(op, vi)`; a modulus-10 cyclic task cannot demonstrate unique H4 advantage.

## Interventions and generated output (actual loaded models)

| Intervention | Observed |
| --- | --- |
| change the relevant payload at an identical query | emitted token changes |
| change the observed operation at fixed evidence | emitted token changes (to the expected class token under the fit-first diagnostic: 4088 -> 4094) |
| change an irrelevant distractor | emitted token preserved |
| remove the required source | `read = false`, decoder `NoRead`, local prior stands |
| read disabled / update disabled | decoder `NoRead` in both |
| composed identity (class 0) versus absence | identity is an **Emit** (shortlist token), absence is `NoRead` |

Three loaded three-token completions ran from held-out prompts (` detect-f` in the primary run); the
first emitted token is not the held-out answer. Useful prose remains unqualified.

## What this changes

- The principal's correction is accepted: the earlier "no linear readout can solve it" framing was
  over-claimed. A collision-free feature set is not an infeasibility certificate, and this run
  replaces the claim with a constructive decoder that works.
- The readout is now a **learned lexical emission over computed state**, with explicit validity
  separate from `NoRead`. That is the brief's selected mechanism, and it is useful on the association
  panel.
- Composition's remaining obstacle is now precise: **the fixture's held-out cells are free
  parameters for a per-operation/per-value code family**, not a H4 capacity limit. The next
  discriminating test needs either a structured code family (a one-parameter group family rather than
  eight free codes) or a task whose combination rule is forced by the observed inputs.

## Limitations

Exposed association regression seeds; a constructed, algebra-realisable composition rule; 3-token
generation only; energy `UNAVAILABLE`; whole-path D0-b not claimed. Retained roots
`derived-state-decoder-{1,4,5}` (0 unlisted each), with `-5` primary and `-4` the fit-first diagnostic.
