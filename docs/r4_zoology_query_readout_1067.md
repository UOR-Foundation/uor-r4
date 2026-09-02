# Query-object answer readout with the matched English dose — #1067

## Frozen contract (2026-09-02; before fitting)

Issue [#1067](https://github.com/UOR-Foundation/uor-r4/issues/1067) continues
#973 after the [#1065 construction diagnostic](r4_zoology_english_diagnostic_1065.md).
The retained English model reproduced `2,396/8,192 = 29.2480%` construction
accuracy but kept the same answer in `3,974/4,096` question contrasts. Only
`20/4,096` question pairs were both correct: `6/2,048` same-owner pairs and
`14/2,048` same-object pairs. That established a behavioral question/readout
failure profile, not an internal cause.

This experiment asks whether changing the supervised readout placement alone
is sufficient at the same fixed learning dose. The candidate gathers the
answer hidden state at the queried object, position **37**, instead of the
constant colon, position **40**. The owner is at position 35, before possessive
36 and object 37. The literal next input token after the object is `?`; the
supervised target is an explicit location/unknown answer label. A positive
result would qualify this answer-readout interface, not ordinary answer-at-colon
generation or general next-token language modeling.

The full 41-token input and entire model forward computation remain intact.
Positions 38–40 are still materialized but cannot affect the selected causal
position 37. This keeps dropout tensor shapes and the training RNG sequence
matched. The intervention changes the selected position, its direct lexical
input and its available causal prefix together; it does not isolate colon
identity alone.

## Exact matched construction and learning clock

The construction and vocabulary containers are copied byte-for-byte from
#1063. All 10,240 inputs, targets, group IDs, variant IDs and question types
remain identical. Only the loader's returned positions tensor changes from
all 40 to all 37. No original artifact is rewritten.

The fresh seed-123 model retains the released Zoology attention cell: two
layers, one head, width 64, 4,096-token tied vocabulary head, maximum position
120, source normalization/dropout/residual settings and 303,744 parameters.
It loads no earlier model or optimizer checkpoint. The prior trainer is copied
with only descriptive/schema/issue identities changed; a whole-module AST
comparison verifies that the numerical, RNG, sampler, scheduling, admission,
resume and export code is unchanged.

The fixed dose remains batch 512, AdamW learning rate
`0.00046415888336127773`, weight decay `0.1`, one cosine schedule with 64
blocks, each of 196 updates. The first 2,352 updates sample supported
construction; the following 1,568 sample all five variants. Exactly 3,920
updates yield 2,007,040 answer presentations. The final work ledger must match
the retained sampler trajectory: 1,846,452 supported and 160,588 unknown
presentations. No evaluation or additional iterator/model construction is
inserted into fitting. The first eight real updates count toward the dose and
supply the unchanged admission observation.

Every supported answer fact is available before position 37. All relevant
query words are available by that position. Thus all 8,192 supported decisions
can respond to the intervention; there is no route-reachability ceiling that
excludes most of the measured population.

## New development population and access

Before fitting, freeze 256 new groups at seed `10672`: 1,024 supported rows and
256 absent-binding rows. Keep the same lexical map, grammar, held-out
owner-object partition, two question types, five counterfactual variants,
fact-slot balance and answer-location balance. Every new base, location-swapped
and absent-binding world must be disjoint from every historical development
world, every construction world and other accepted new groups. World identity
uses sorted owner/object/location facts, independently of their display order.
Exact input overlap is also checked.

Historical development is regenerated using the unchanged #1063 generator and
seed; its canonical bytes must reproduce the published original CID. Its
retained payload is never opened. Rejection during new-world generation keeps
the preassigned target slots and locations fixed, preserving balance. Build
time independently audits labels, missing bindings, all held-out pairs,
counterfactuals and exclusions without running a model.

Default preparation validation and training loaders read only the manifest,
vocabulary and copied construction payload. They neither load nor regenerate
development. The new development payload is opened for model scoring only if
the final construction score reaches the frozen fit threshold. It measures
fresh combinations within this familiar controlled task, not transfer to an
unseen task or unrestricted English.

## Empirical criteria and divergent actions

The fixed final update-3,920 artifact is the sole evaluated checkpoint. Report
construction accuracy/NLL, all #1065 error categories and matched question/
location-swap responses, including both question types and changes against the
retained baseline. A partial gain remains evidence even when the fit threshold
is missed.

| Criterion | Frozen count |
| --- | ---: |
| Supported construction accuracy at least 99% | 8,111 / 8,192 |
| New supported development accuracy at least 95% | 973 / 1,024 |
| All four answers correct in at least 90% of new groups | 231 / 256 |
| Complete same-owner groups at least 90% | 116 / 128 |
| Complete same-object groups at least 90% | 116 / 128 |
| Correct new missing-binding answers at least 95% | 244 / 256 |

1. **Construction miss:** return `QUERY_OBJECT_READOUT_CONSTRUCTION_MISS`,
   preserve the final model and any partial gains, and leave new development
   unscored. Readout placement is insufficient under this fixed dose. Do not
   add updates or retry; the next design must address joint query binding.
2. **Construction fits, development misses:** return
   `QUERY_OBJECT_READOUT_FRESH_TRANSFER_MISS`. Preserve construction learning
   and address transfer to the new combinations. Report supported, quartet,
   question-type and unknown results separately.
3. **All ordinary criteria pass:** return
   `QUERY_OBJECT_READOUT_FRESH_BINDING_PASSED`; the next separate step is
   unchanged-adapter R4 inference preservation on this artifact.
4. **Admission, numerical or resource interruption:** retain the exact partial
   checkpoint/trajectory and report its actual limit. Never represent an
   incomplete fit as a scientific negative or restart its admission clock.

This experiment performs no R4 adapter or native-frame execution. More
geometry, generation, broader language claims, reasoning, chat readiness,
softmax removal and integer/table lowering do not follow automatically.
The earlier #1059/#1061 attention positives and #1063/#1065 evidence remain
intact; #973 stays open and #954 blocked.

## Resources and named verification

Use the original 1,800-second combined fit/evaluation/replay and 4-GiB RSS cap,
eight CPU Apple Accelerate threads, one inter-op thread, 1.25 admission
projection factor, and 60-second evaluation allowance. #1063 completed fit,
evaluation and replay in 249.37 seconds at 0.486 GiB; retaining the full trunk
retains the work shape. This is one minutes-scale fit, with no hardware sweep,
new seed, extra dose or learning-rate search.

Named checks: copied-data/readout identity and stage-specific data access;
fresh-development semantic/exclusion/balance checks; whole-trainer clock
equivalence; construction-failure suppression of development scoring; both
question-type decision thresholds; preservation of partial gains; final
example serialization; immutable model/source bindings; and exact complete
evidence replay in a different process. Independent review covers source,
evidence and current claims. Broad QA remains dormant. Required queue statuses
are transport acknowledgements, not tests or scientific PASS evidence.

## Commands

From `tools/r4-softmax-trainer`, use the locked offline environment:

```bash
uv sync --frozen --offline
.venv/bin/python -m r4_softmax_trainer.zoology_query_readout prepare \
  /Users/casey.allard/uor-r4/.uor-models/research/issue-1067-zoology-query-readout \
  --source-root /Users/casey.allard/uor-r4/.uor-models/research/issue-1063-zoology-english-binding
.venv/bin/python -m r4_softmax_trainer.zoology_query_readout fit \
  /Users/casey.allard/uor-r4/.uor-models/research/issue-1067-zoology-query-readout
.venv/bin/python -m r4_softmax_trainer.zoology_query_readout run \
  /Users/casey.allard/uor-r4/.uor-models/research/issue-1067-zoology-query-readout
.venv/bin/python -m r4_softmax_trainer.zoology_query_readout verify \
  /Users/casey.allard/uor-r4/.uor-models/research/issue-1067-zoology-query-readout
```

Preparation, results and replay are immutable. A fit may resume only the same
bound checkpoint within the original cumulative budget. Verification performs
no training and compares full final evidence, including prediction/logit/
attention identities, construction diagnostics and any admitted development.

## Published preparation

Executable and contract freeze: `24a8f3cd`. Eight focused checks passed (three
data, one whole-trainer equivalence, four campaign/serialization checks).
Independent source review found no remaining blocking issue. No fitted model
was initialized or executed by those checks or review.

The [preparation envelope](r4_zoology_query_readout_1067_preparation.json) binds
223 implementation files, including the unchanged predecessor closure. The
actual preparation audit verified all 10,240 original construction labels,
all 1,280 fresh development labels, 768 fresh variant worlds, zero historical
or construction world/input overlap, all 64 held-out owner-object pairs,
balanced supported target locations and relevant fact slots. The copied
construction and vocabulary CIDs remain exactly those of #1063.

- Preparation: `blake3:b6df21b50fe67696910e8e01ae2aa590c9e9b6aebf15b090cc11b98edc5a82d3`
- Implementation: `blake3:9d368e194b2fad64aebadaf87ca71b4927134b13c33bb949c0e7cc0ca6e81b48`
- Dataset manifest: `blake3:da7be1fba1bb02d3891f4c08840cdd368828b28a2e0a19119f08ca8aa44a5f28`
- Dataset tree: `blake3:0ce48d3f9266377013543af1eae00fe445b40b90ca29ed73085e732c08725bd5`
- Fresh development: `blake3:5f20f7f5f9e079e73886bd885cb88dc0948effc73a58d2a111f387ae5604b780`
- Unchanged construction: `blake3:d767fafdf544f01db99d9acb317c76df55e9f9d28f99785d2a6ae62b663731a2`

These bindings and the decision contract were published before the first
training update. The fresh development population had zero model decisions
at publication.
