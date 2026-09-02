# English supplied-context binding with the qualified attention cell — #1063

## Frozen contract (2026-09-02; before fitting)

Issue [#1063](https://github.com/UOR-Foundation/uor-r4/issues/1063) is a child of
#973 and follows the [exact-data R4 preservation result](r4_zoology_exact_coherent_inference_1061.md).
The preceding checkpoints remain immutable. Their vocabulary IDs represent
MQAR symbols; this curriculum initializes a fresh seed-123 copy of the same
Zoology source cell with an explicit English lexical map. It does not remap or
continue either earlier checkpoint.

The architecture remains two layers, one attention head, width 64, vocabulary
4,096 and positional capacity 120, with the source dropout, normalization,
residuals, double initialization and tied full-vocabulary head. Exactly 52 IDs
represent the declared words, punctuation, BOS and `unknown`; all 4,096 IDs
remain eligible outputs. An unused ID counts as a wrong answer. There is no
teacher, external provider, pretrained tokenizer, role-tagged model input,
candidate filtering or answer parser in model execution. The serialization
parser is solely a preparation instrument.

Each 41-token input has four facts such as `mara put the key in the drawer.`
and a question such as `where is mara's key? answer:`. The model predicts one
location word or `unknown` at position 40. The label is never appended to its
input. This is a controlled English-form binding task, not unrestricted English
understanding or open-ended generation.

Each world contains `(A,x), (A,y), (B,x), (C,z)` with three distinct owners,
three distinct objects and four distinct locations. Same-owner and same-object
distractors force the central question to use both lexical attributes. A
four-row group asks two questions of a history and of the same history with
their locations exchanged. Half of groups contrast questions sharing an owner;
half contrast questions sharing an object. Question words, fact positions and
location targets are explicitly balanced.

The fifth row exchanges only the object words in logical facts zero and three:
`(A,z), (A,y), (B,x), (C,x)`. The original `(A,x)` question now has no answer
fact, while its owner and object still appear elsewhere. The token multiset,
length, fact count, sentence order and locations remain identical. Its target
is `unknown`. Removing a whole sentence would supply an unintended shortcut.

Construction uses 2,048 groups: 8,192 supported rows and 2,048 unknown rows.
Development uses 256 groups: 1,024 supported rows and 256 unknown rows. Seeds
are 10631 and 10632. Owner-object pairs satisfying
`(owner_index + object_index) % 4 == 0` are excluded from every construction
fact and question, including absent-binding variants, and reserved for
development. All 192 construction and 64 development pairs occur, with zero
pair or canonical-world overlap. All lexical items occur in construction.
Each location has exactly 1,024 construction targets and 128 development
targets. Within each question type, both relevant-fact positions are balanced.

## Fixed learning clock and resource decision

One fresh ordinary-model fit uses batch 512, AdamW learning rate
`0.00046415888336127773`, weight decay `0.1` and the source defaults. It runs
2,352 supported-only updates, then 1,568 mixed updates, with one cosine schedule
stepped every 196 updates and `T_max=64` such blocks. The complete fixed dose is
3,920 updates and 2,007,040 answer presentations: 1,204,224 in the first phase
and 802,816 in the second. Actual supported/unknown counts in the partial final
mixed traversal are recorded. Neither development scores nor checkpoint
selection alter this dose.

The first eight actual updates supply one time/memory admission observation;
their optimizer, sampler and RNG states are retained and their updates count
toward the dose. Remaining time is projected from their mean step duration
with a 1.25 safety factor plus 60 seconds for final evaluation and replay.
The whole campaign has a 1,800-second and 4-GiB ceiling, with one CPU process,
eight intra-op threads and one inter-op thread. Checkpoints preserve cumulative
time and state every 16 updates and at phase/block boundaries. An ambiguous
interrupted admission update stops unavailable rather than repeating timing.

The prior cost basis is #1057: 3,920 updates at sequence 120, vocabulary 4,096
and eight queries took 1,096.9868 seconds. The new sequence is 41 with one
query; the prior conservative fit/evaluation/replay estimate is approximately
1,413.5 seconds. This estimate is superseded by the actual eight-update
admission observation. No hardware matrix, learning-rate sweep, new seed,
post-reveal retry or extra dose is authorized.

Reachability is explicit: every supported input contains its unique answer
fact, all queries follow their evidence, and the two histories require
different answers. Every absent-binding input lacks exactly the queried pair.
The preparation instrument independently parses the decoded input IDs to
verify all labels, split exclusions, pairings, vocabulary coverage and balance
before any model update.

## Empirical criteria and divergent actions

The fixed final artifact is the sole scored checkpoint. Report construction,
supported development, complete counterfactual groups and unknown behavior
separately:

| Criterion | Required count |
| --- | ---: |
| Construction supported accuracy at least 99% | 8,111 / 8,192 |
| Development supported accuracy at least 95% | 973 / 1,024 |
| All four answers correct in at least 90% of groups | 231 / 256 |
| Complete same-owner groups at least 90% | 116 / 128 |
| Complete same-object groups at least 90% | 116 / 128 |
| Development unknown accuracy at least 95% | 244 / 256 |

Only if all ordinary-language criteria pass does this campaign install the
unchanged #1059/#1061 inference-only R4 adapter on the final artifact. Require
identical top-1 IDs for all 1,280 development answers, maximum selected
full-vocabulary logit difference at most `0.005`, attention difference at most
`1e-5`, NLL difference at most `1e-5`, unchanged learned/tied weights and zero
future contributions. The native 8,192-token map and 120-frame bundle remain
unchanged; no new geometry is introduced.

After preserved inference, the inconsistent source-frame transport control
must use equal causal support and work. A supported-accuracy loss of at least
50 percentage points is the separate strong-sensitivity criterion. Its result
does not erase any established plain/R4 preservation. Fresh-process replay
must reproduce final-artifact behavior, logits and audits exactly without
retraining or reading optimizer/checkpoint RNG state.

A positive recommends a separately frozen broader language/context
application. A construction miss retains the artifact and redirects to
learning this lexical recipe. A construction fit with development miss
redirects to compositional transfer. A missing-binding miss isolates that
behavior. A language pass with R4 disagreement isolates integration without
retraining. Resource interruption retains the available checkpoint/artifact
and reports unavailable; it does not authorize a forced continuation.

This task does not establish general English understanding, reasoning,
factual correctness, chat or release readiness, H4 superiority, softmax
removal, recurrence, integer/table lowering or completion of #973/#954.

## Activated implementation checks

Three focused data checks cover the serialized oracle and split, bag-preserved
absence and lexical round trip, and construction-only training access with
corrupt-label rejection. Five focused decision checks cover unknown masking,
question ignoring, matched causal-control work, carried resource budgets and
post-load artifact mutation. Independent review covers the fixed optimizer
clock, sampler/checkpoint continuity, label boundary, immutable lineage and
conditional claims. All eight checks passed before fitting. Broad workspace,
BDD, no_std, native/wasm, kappa, legacy campaign and release QA remain dormant.
The protected queue supplies transport acknowledgements only.

## Reproduction entry points

From `tools/r4-softmax-trainer`, the locked Python environment runs:

```text
python -m r4_softmax_trainer.zoology_english_binding prepare ROOT --frames-root FRAMES
python -m r4_softmax_trainer.zoology_english_binding fit ROOT
python -m r4_softmax_trainer.zoology_english_binding run ROOT
python -m r4_softmax_trainer.zoology_english_binding verify ROOT
```

`prepare` requires a new evidence root. `fit` resumes only the same frozen
trajectory and returns immutable completed results. `run` scores only a
complete fixed-dose fit. `verify` must execute in a fresh process and carries
the fitting and evaluation time into its budget. New execution is not an
authorization to overwrite or tune this frozen campaign.

The implementation, exact dataset, predecessor evidence and native frames are
bound in the preparation envelope. The following append-only entries will
record the preparation identity, actual dose/outcome and protected delivery.
