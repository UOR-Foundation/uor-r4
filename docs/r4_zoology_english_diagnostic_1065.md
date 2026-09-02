# Retained English construction diagnostic — #1065

## Frozen contract (2026-09-02; before inference)

Issue [#1065](https://github.com/UOR-Foundation/uor-r4/issues/1065) continues
#973 after the [#1063 English curriculum](r4_zoology_english_binding_1063.md).
It asks which behavioral failure profile is present in the retained final
model's construction answers. This is a diagnostic of an existing negative,
not a new capability or geometry qualification.

The sole model is #1063's final update-3,920 artifact, with 303,744 parameters,
two layers, one head, width 64 and an unchanged tied 4,096-token output head.
The prior construction result is exactly `2,396/8,192 = 29.248046875%`, NLL
`1.6081310920417309`. No optimizer, checkpoint/RNG state, development tensor
payload or native geometric frame payload is opened. Published JSON envelopes
include historical development statistics and artifact metadata; binding those
documents does not constitute a new development model decision.

The population is the exact 8,192 supported rows from the retained construction
container: 2,048 groups, each in order base-q0, base-q1, swapped-q0, swapped-q1.
The container also holds the old unknown rows; they are excluded before model
execution. No examples, vocabulary items, labels or worlds are newly generated.
Input length remains 41, selected readout position 40, batch size 256, eight CPU
threads and one inter-op thread. All vocabulary logits and returned causal
attention are included in reproduction. Labels and metadata stay outside the
model invocation.

Before interpreting a diagnostic, reproduce every field of #1063's saved
construction score, including the prediction, selected-logit and attention
digests and both NLL reductions. Any mismatch returns
`UNAVAILABLE_CONSTRUCTION_REPRODUCTION` and suppresses interpretation.

## Categories and denominators

Definitions: relative to each row's actual queried owner/object and its ordered
four facts, classify the predicted token as:

1. the target fact's location;
2. a same-owner, different-object fact's location;
3. a same-object, different-owner fact's location;
4. an unrelated fact's location;
5. a vocabulary location absent from the history;
6. `unknown`;
7. another vocabulary token.

Distinct history locations make these categories mutually exclusive. The q0
question has one fact of each of the first four categories. A q1 question has
one absent attribute-confound category and two unrelated facts. Report
category-eligible rows and eligible fact slots as well as raw counts; an absent
category has unavailable rates. Compare owner/object confounds on q0, where
their opportunities are equal. Also report unknown/out-of-history/other
predictions against all q0 errors, so a concentration within in-history errors
cannot silently stand for the whole failure.

Report all rows and strata by question type (same-owner or same-object question
pair), q0/q1, base/swapped history, and target displayed slot. Display target
exposure, selected-slot counts and correctness together. This separates lexical
role labels from their physical positions; a position association is not by
itself an internal causal explanation.

## Matched responses and next-action rule

There are 4,096 fixed-history question contrasts `(0,1)` and `(2,3)`, and 4,096
fixed-question location exchanges `(0,2)` and `(1,3)`. Every contrast changes
the correct target. Report prediction changes/invariance, both answers correct,
and changed predictions without both correct, overall and by question type.

Definition: when left and right targets are location IDs `a` and `b`, retain
those same two IDs in both logits and compute

```text
delta = [z_left(a) - z_left(b)] - [z_right(a) - z_right(b)]
```

Positive delta denotes a response in the desired direction, not successful
binding. Report its signed distribution, full-vocabulary absolute logit
differences, and each target's margin over the best other vocabulary token.
An unchanged top-1 can coexist with a useful sub-top-1 response.

The following strict-majority rules are descriptive follow-up selectors.
They are not capability thresholds, causal guarantees or authorization for a
new fit. A majority means `2 * numerator > denominator`, with an empty
denominator unavailable. All flags and denominators remain visible.

| Profile | Next focused investigation |
| --- | --- |
| A displayed slot receives most in-history selections, and an attribute confound receives most q0 in-history errors | Joint position-versus-attribute discrimination |
| Displayed-slot majority alone | Position-sensitive selection/readout |
| Same-owner confound majority | Object disambiguation |
| Same-object confound majority | Owner disambiguation |
| Neither concentration, but most matched question pairs keep the same prediction | Question-to-readout sensitivity |
| None of these profiles | Distributed binding errors |

## Binding, resources and named verification

The preparation directly binds the published/local #1063 preparation, fit,
result and replay envelopes, all 199 historical implementation files, the
retained model, construction container, vocabulary and manifest. It adds the
new diagnostic implementation and focused tests. It deliberately does not call
the old whole-data preparation validator, which would open development and
frame payloads. The construction-only loader independently parses supported
labels and verifies canonical group/variant metadata.

Resource ceiling: 300 seconds combined diagnostic plus fresh-process replay,
and 2 GiB maximum RSS per process. The cost basis is #1063's approximately
1.13-second final evaluation; parsing and descriptive reductions add work but
require no fit. All supported rows can contribute to the profile, so this
diagnostic can distinguish follow-up directions across the entire failure.
There is no hours-scale run or hidden admission sweep.

Named checks are the new semantic/classification/pair-margin/action tests,
construction access/binding checks, exact old-score reproduction, independent
fresh-process replay, and review of this implementation and its evidence.
Broad QA remains dormant. Required GitHub queue statuses acknowledge transport
only and are not tests or scientific PASS evidence. Source/model identities
are checked before and after execution. The first canonical quartet supplies
the fixed readable example; examples are not selected after viewing errors.

Prior attention positives #1059 and #1061 remain intact. #1063's construction
and development negatives remain intact. Geometry expansion, new training,
general English understanding, reasoning, chat readiness, softmax removal and
integer/table lowering do not follow from this diagnostic. #973 stays open;
#954 remains blocked.

## Reproduction commands

From `tools/r4-softmax-trainer`, use the locked offline environment. The
following paths identify the original local retained evidence; a relocated
reproduction needs a separately recorded preparation with its new roots.

```bash
uv sync --frozen --offline
.venv/bin/python -m r4_softmax_trainer.zoology_english_diagnostic prepare \
  /Users/casey.allard/uor-r4/.uor-models/research/issue-1065-zoology-english-diagnostic \
  --source-root /Users/casey.allard/uor-r4/.uor-models/research/issue-1063-zoology-english-binding
.venv/bin/python -m r4_softmax_trainer.zoology_english_diagnostic run \
  /Users/casey.allard/uor-r4/.uor-models/research/issue-1065-zoology-english-diagnostic
.venv/bin/python -m r4_softmax_trainer.zoology_english_diagnostic verify \
  /Users/casey.allard/uor-r4/.uor-models/research/issue-1065-zoology-english-diagnostic
```

Each run/verify command is a separate process. Existing preparation, started
markers, results and replay files are immutable; these commands intentionally
refuse to overwrite them. The checked-in JSON evidence accompanies this record.

## Published preparation

The executable and contract were frozen at `58bd1559`. All 11 named focused
checks passed (six direct-binding/access, three analysis, two campaign), and
independent source review found no blocking issue. No fitted inference was
performed by those checks or that review.

The [preparation envelope](r4_zoology_english_diagnostic_1065_preparation.json)
binds 211 implementation files, including all 199 predecessor records:

- Preparation: `blake3:5679f721fa60c16601a4d3a8ca46397055b89d7769dfae1ca099a1e3f3fbe5a9`
- Implementation: `blake3:d22cde07d38be3e3042039a93c6cdc06c5f4c772c7ba5490905889215f24827b`
- Retained model: `blake3:a4eb5ef76c387ca6ebe9f185b1a5ad023c81291ce4cc9000bb5d23248aaef282`

The preparation and this section were published before construction inference.
