# Learned soft-role and compound-binding preservation through R4 — #1079

## Frozen contract (2026-09-02; before learned-model inference)

Issue [#1079](https://github.com/UOR-Foundation/uor-r4/issues/1079) continues
#973 after [#1077](r4_zoology_language_interface_1077.md), merged in #1078 at
`6ebcf4bf48ed82addd0a38c7fe368b989815f772`. The retained reader has 141,571
parameters; the frozen #1073 binding core has 286,976, for 428,547 total.
The two models, ordinary contextual role scores, softmax, vocabulary, dataset,
normalizations, projections and tied full 4,096-token head remain unchanged.
There is no fitting, new data, geometry export/expansion or generation.

Bind all four exact public/local #1077 envelopes, the reader artifact and state,
its complete 295-file implementation closure, the inherited #1073 core lineage,
and the unchanged native frame bundle. The reader artifact CID is
`blake3:c11d21817bff818fa242f653279e9e0c12d21641ff63df3a5f7a6680bcc732a7`;
its state CID is
`blake3:7c659422df2e65a0ce24c08738dc9f08dca99775de1702251097a0fc6483404e`.
The native bundle tree is
`blake3:94762441a43b03f596a66131ec34af15bba3afbc2bbc5d28ab7dfdabd9b6d68c`.
All 120 registered matrices, the 8,192-token map, multiplication table and native
prefix witnesses are retained. Coverage is measured; registration does not mean
that all 120 frames are reached.

## Two coordinate stages

The new reference is #1077's **learned ordinary soft execution**. Its logits can
differ from the earlier hard-field oracle (previous maximum 0.06234240531921387).
Do not compare this adapter against the old oracle or rerun its canonical
forwards. Reuse the historical scorer to reproduce every learned-path record,
role decision, group/syntax summary, work count and qualification exactly.

Frame assignment folds each row's actual valid input tokens continuously
through the five supplied clauses, from native identity, including punctuation
and the final question colon. No BOS, separator or clause reset is inserted.
Token frames are cumulative prefixes after each token; clause frames are their
last valid token frames. Padding is never folded or transported. New lexical
aliases 52–56 retain the existing token-ID map; this does not assert that native
leaves encode the aliases' word meanings. Query frame is the complete question
clause end, fact frames are the four fact clause ends, and null is identity.

**Definition — token-to-clause role pooling.** For frozen embedding E(x_i),
true token frame F_i, clause frame F_c and unchanged learned role weight a_cri:

```text
e_local_i = transpose(F_i) E(x_i)
T_ci = transpose(F_c) F_i
r_cr = f32(F_c sum_over_valid_tokens(a_cri T_ci e_local_i))
```

Treat each 64-wide vector as sixteen four-lane blocks. Encoding, transport,
weighted pooling and decoding use native f64. Cast each completed role vector
to f32 before the original normalization and Q/K/V projection. All grammar,
entity and distractor tokens participate with their learned soft weights.
Compute all three roles for all five clauses: 15 vectors per row, although only
14 are supervised and the question's location vector is unused downstream.
The learned Conv/GELU/role-score/softmax computation stays in ordinary f32.

The second stage reuses the qualified #1075 compound gauge helper unchanged:
encode projected Q and all four fact K/V plus null; transport into the question
frame; cast the completed f64 dot to f32 before division by eight; use f32
softmax; aggregate all five values in f64 and decode to f32 before the original
output projection, normalization and tied head. Both full soft mixtures remain.

## Population and empirical criteria

Use exactly the six already-observed #1077 views: two construction views of
10,240 rows each and four development views of 1,280 rows each. Development
views 0/1 use seen syntax combinations and 2/3 use combinations withheld during
the earlier reader fit. Their outcomes are already known here; this is
preservation, not another fresh or sealed generalization result.

All six ordinary views must exactly reproduce the retained learned-path fields
before any R4 forward. Then each coherent view, separately in all/supported/
unknown strata, must have identical predicted token IDs, maximum full-vocabulary
logit difference <=0.005, binding-attention and all-computed-role-vector maximum
differences <=1e-5, and mean NLL difference <=1e-5 nats. Role attention must be
bit-identical. All role decisions, role accuracies, counterfactual groups,
same-bag syntax pairs and complete support/work counts must be preserved.

Learned state, eval/no-grad mode, parameter counts, tying, source/data/frame/
implementation identities are checked before and after. No gold roles,
answer targets, equality matcher, token filter or future token enters the model.

## Conditional controls and distinct decisions

Only after every primary criterion passes, execute both fixed controls over
all six views, with true local source encoding unchanged:

1. **Token source frame permutation:** in each clause, connection at position i
   uses the frame at (i+1) modulo its valid length. Token embeddings stay encoded
   in their true frames. Clause destination, role weights and the compound
   gauge stage remain coherent.
2. **Fact source frame permutation:** role pooling remains coherent; substitute
   fact connection source indices [1,2,3,0], with null identity fixed, reusing
   #1075's control. Coherent role vectors must remain bit-identical.

Both controls keep the same support and transformation work as coherent R4.
Reader attention must remain exact. Count actual changed matrices and require
agreement with preflight; merely cycling source positions is insufficient.
At least 50 percentage points of supported accuracy loss in **each view of each
control** establishes strong transport sensitivity. Unknown effects and changed
binding weights are descriptive. A fixed null frame does not fix its softmax
weight. No hypothesis about reassigned answer IDs is imposed.

- Reference mismatch: stop before R4/control scoring and resolve reproduction.
- Preservation miss: retain #1077 ordinary execution; localize only this adapter
  mismatch under a separately declared follow-up, with no tuning/retry here.
- Preservation with weak or invalid control: retain preservation, limit
  attribution, and separately diagnose control sensitivity/integrity.
- Preservation and both strong valid controls: retain the two-stage path and
  separately freeze a small causal output-state prototype. It would emit a short
  controlled answer clause token by token, including termination, with the
  location coming through binding rather than a formatter inserting an answer.
  Supplied clauses and the fixed lexicon initially remain. No successor runs here.
- Resource stop: preserve incomplete evidence and consumed clock; no replacement
  run or renewed budget.

This is an empirical coordinate-preservation claim, not a geometry advantage,
softmax replacement, integer/table lowering, general English, unrestricted
parsing, free generation or deployed product-readiness claim. #973 stays open;
#954 remains blocked.

## Reachability, structural gate and resource budget

Every primary row traverses both new coordinate stages. Per complete arm:

| Work | Count |
|---|---:|
| Full-vocabulary answer decisions | 25,600 |
| Supported / unknown answers | 20,480 / 5,120 |
| Valid input tokens | 1,561,600 |
| Admitted / materialized role scores | 4,684,800 / 4,992,000 |
| Computed / supervised role vectors | 384,000 / 358,400 |
| Binding pairs / included nulls | 128,000 / 25,600 |
| Token blocks encoded and transported, each | 24,985,600 |
| Weighted role-value block contributions | 74,956,800 |
| Role-output blocks decoded | 6,144,000 |
| Binding query/output blocks, each | 409,600 |
| Binding K/V encoding/transport blocks, each | 2,048,000 |

Seen views contain 61 valid tokens per row; development views 2 and 3 contain
57 and 65 respectively. Padding transformations and future reads remain zero.
The structural gate loads observed inputs and native frames without either
learned model. For each control/view, it counts supported rows containing an
actual changed source matrix. Their fraction is an upper bound on possible
accuracy loss and must reach 0.5 before launching. It predicts no accuracy loss.

Use CPU/Apple Accelerate with four intra-op threads, one inter-op thread, one
process and batch 256, matching #1077 exactly. Shared run-plus-replay cap:
900 seconds and 4 GiB peak RSS. Four complete arms plus replay admit at most
204,800 learned forward rows. The earlier evaluation's 12.174 seconds anchors
ordinary cost; added f64 transport remains unmeasured until this sole campaign.
Retained ordinary full-head tensors occupy 419,430,400 bytes (400 MiB), with
98,304,000 bytes of role vectors; only one current R4/control view is retained.
Checks run between batches and stages; the clock includes binding validation.

## Execution ledger

Source, preparation and outcome entries will be appended here as they occur.
Only the focused adapter, provenance and decision checks named in this contract
are active. Broad workspace/BDD/wasm/fuzz/audit/teacher QA remains dormant;
protected queue acknowledgements are transport status, not test evidence.

### Source freeze and prepared admission

Source frozen at `91aecda179209041decacef9488d5e8ec2681299`. Twelve focused checks passed in
0.089 seconds; Ruff and claim wording passed. Independent source review found
no remaining blocker. No learned-model forward preceded this preparation.

The [exact preparation](r4_zoology_language_r4_1079_preparation.json) has CID
`blake3:d9c8ad8448365b2039276fdeda6b70da53ef63fde24e02dd1dd8dea437b546a4`.
Its 307-file implementation closure has CID
`blake3:a94432fb7e764247521a9fbd42ee8f99472bb6f2e800e895f380806b0bbf1462`.
All six views reach 24 of the 120 registered native frames. Both controls change
an actual source matrix in every supported row: all twelve reachability ceilings
are 100%, above the frozen 50% criterion. This is opportunity, not a model result.

| View | Changed token-source matrices | Changed fact-source matrices |
|---|---:|---:|
| construction 0 | 471,638 | 39,322 |
| construction 1 | 461,246 | 39,905 |
| development 0 | 59,105 | 5,112 |
| development 1 | 57,605 | 5,120 |
| development 2 | 57,925 | 5,120 |
| development 3 | 63,045 | 4,866 |
