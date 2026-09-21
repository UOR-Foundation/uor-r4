# Shared learned transitions and complete result continuation

September 21, 2026. Executes the [shared-transition brief](deepseek-shared-transition-continuation-step-2026-09-21.md) and
the [principal review](derived-state-decoder-review-2026-09-21.md) on merged `86623138` (PR #1338,
verified equal to the merged head). Isolation worktree `.worktrees/shared-transition`; owner checkout
untouched. New mode `--mode=shared-transition` and module `learner/shared_transition.rs`.
[Evidence](../evidence/shared-transition-2026-09-21.json).

## Accepted corrections

- The claim that the fixture's missing cells are independent free parameters is **withdrawn**. For
  source-correct cells the per-operation binding relation is fixed, so absorbing it into
  `A[o]=T_bind[r(o)]*U[o]` gives `P[o,v]=A[o]*B[v]`, and three rectangle corners determine the fourth.
  The observed graph is connected. Labels pass through a many-to-one decoder, so the latent products
  are not necessarily identified - but the defective optimizer never established non-identifiability.
- The earlier development probe was mis-described: the probe operation tokens were absent from the
  initialized operation domain and fell back to identity while still steering every candidate score.
  That is **supervised outer development fitting through a defective domain**, not held-out-operation
  generalization. This run declares calibration and probe halves explicitly and never claims the
  probe as population evidence.
- The old controls were defective (the operation change moved source and query roles together; the
  identity example checked target class 0 rather than the group identity; the absence parity boolean
  was not a real comparison). All are replaced below.

## The milestone

```text
s0    = E[selected payload]                       learned initial state
s_j   = A[observed primitive j] * s_{j-1}         one shared action per primitive, exact products
out_j = D(s_j)                                    learned lexical decoder
Stop  = P(observed remaining primitives)          learned continuation policy
```

Exact evidence identity, the computed geometric value and the response phase stay distinct. The
predictor **retains its state between emitted tokens**: each state is recomputed from the selected
payload and the observed primitive prefix, and emitted tokens are never re-read as a query (the
three-token loop of the previous milestone did exactly that after its first emission). `Emit`,
`Stop`, `UnknownValue`, `UnknownPrimitive` and `NoGrounding` are separate typed outcomes, so an
unknown operand is not silently an identity. Reversible transport is used for the product only; the
surrounding machine is not forced into a group.

## Task, populations and result

Prefix `[source_role, key, value, primitive.., query_role, key]`, response one label per observed
primitive then `Stop`. The primitive meanings and the label lexicalisations are grounded in
development; the held-out populations are **unseen ordered combinations** (length 4) and **unseen
order reversals**. Actions come from a witnessed non-abelian order-8 subgroup of the served table.

| Arm | dev /288 | unseen length 4 /128 | unseen reversal /64 |
| --- | ---: | ---: | ---: |
| frozen local prior | 0 | 0 | 0 |
| value-only lexical decoder (retained component) | 4 | 0 | 0 |
| exact transition dictionary (lookup, unseen fallback) | 288 | **0** | **0** |
| matched additive C120, same technique | 153 | 24 | 15 |
| **H4 shared transition (primary)** | **186** | **48** | 10 |

Complete responses are multi-token: 556/672 dev tokens, 288/288 dev responses stop, and the learned
remaining-indexed stop policy transfers to the unseen length 4 (124/128 stop correctly there). The
headline structural result is that **shared actions transfer to unseen ordered combinations where an
exact lookup cannot**: 48/128 against 0/128 for the dictionary and 24/128 for the best matched
control.

## Causal controls

| Control | Observed |
| --- | --- |
| selected payload changed, identical query and identical primitives | every step changes to the expected label |
| retained state vs emitted tokens | identical state trajectory; emissions are never re-read |
| **noncommuting witness on the learned codes** | H4 final state 72 forward vs 90 reversed (final tokens 4091 vs 4092); the additive arm's final token is invariant (4090 both ways), as an abelian algebra requires |
| unknown primitive / undefined value | typed `UnknownPrimitive` / `UnknownValue`, not identity |
| absent source | read absent; no response |
| variable lengths 1-4 | all stop, with matching token counts |

The order-reversal *population* (10/64) is confounded: the instruction order changes the
decision-point predecessor, so the read itself can change. The computation-only comparison with a
fixed selected operand is the decisive test and is the one reported above. This coupling between the
observed instruction and source selection is the interface limitation the review predicted; removing
it needs persistent source ownership and a separate control channel, which is **NOT_RUN** here.

## Generated behaviour and independent recount

Four complete responses were generated from held-out prompts; three of the four are exactly correct
four-token responses ending in `Stop`. A recount of complete responses from the saved 1,312 per-step
events, keyed by a unique item id, gives **244**, equal to the arm totals (186 + 48 + 10), so the
headline counts are derived from saved events rather than maintained separately.

## What is not established

Development is 186/288 complete (556/672 tokens), so the learned codes approximate rather than equal
the true action structure; the coordinate search with multi-start and a development probe improves on
a raw fit objective but does not converge. The reversal population does not favour H4. No dependent
second read was executed, so the brief's Read/Compute/Emit/Stop -> dependent-Read step remains open.
This is an authored finite circuit: a circuit result, not language capability or unique geometric
advantage. Energy `UNAVAILABLE`; whole-path D0-b not claimed. Retained roots
`shared-transition-{1,4,5,6}` (0 unlisted each), with `-6` the delivered primary.
