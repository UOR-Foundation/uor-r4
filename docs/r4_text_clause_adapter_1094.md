# Frozen raw-text adapter comparison — #1094

**2026-09-03 — preparation and independent source review in progress. Model
execution and withheld evaluation are `NOT_RUN`.** This record appends to the
accepted [#1085 specification](integration/clause-segmentation-1085.md); it does
not change its syntax, populations, thresholds, resource caps or outcome actions.
The active issue is [#1094](https://github.com/UOR-Foundation/uor-r4/issues/1094),
based on `3e894820c520f3b7803a48c6a2eeeb5b7d7021c5` in an isolated worktree.

## Decision and unchanged model boundary

**Empirical Criterion.** Determine whether the sole deterministic raw-text
adapter recovers independently annotated clauses and preserves the accepted
R4 reader/core computation. Positive evidence admits only bounded raw-text entry.
It cannot establish new semantic-world transfer, general language, reasoning,
coding, geometry advantage or the final source-free integer/table kernel.

Keep the #1077 reader, #1073 core, #1079 coherent R4 path, all soft mixtures,
full vocabulary, known query forms and four-fact context. No new fit or model
parameter is permitted. Preserve #1079 `LANGUAGE_R4_PRESERVED_CONTROL_WEAK`,
including its token-control miss in three of six views, and #1082's descriptive
exposure/displacement result. The adapter is not a causal explanation for that
miss. #973's higher-context terminal remains unmet and #954 remains blocked.

The implementation lives in
[text_clause_adapter](../tools/r4-softmax-trainer/src/r4_softmax_trainer/text_clause_adapter/adapter.py).
It receives an exact request schema plus original bytes, retains punctuation and
ordered spans, and emits only IDs/lengths to the unchanged reader. A boolean
grammar recognizer cannot forward captures, roles, semantic worlds or answers.
Typed refusals perform zero model forwards. Model output is the actual full-head
token, decoded through the core vocabulary; reader aliases are not output words.

## Independent preparation and commitments

The production adapter author and independent curator are separate agents.
The curator never imports the adapter or historical rendering/parsing/decoding
helpers and never loads a model. It verifies the original #1073 construction
semantics and #1077 construction distractor assignments, then authors text and
annotations independently. Text preparation is independent; semantic worlds are
historically observed. Five related rows and all their surface variants remain
in one partition and are not counted as independent statistical trials.

The policy bytes were fixed before population preparation at SHA256
`91cce30a0b78c48130595369d3ea2a47c4de89cab5db1d4219d1874198cf52d0`.
The independent selection receipt fixes canonical group serialization, four
authoring/sixteen withheld groups, their hashes, exact four surface-profile
rules, refusal families and boundary-control choices. It was written exclusively,
made read-only and [published before text generation](https://github.com/UOR-Foundation/uor-r4/issues/1094#issuecomment-5520856972)
at SHA256 `892e3239773e8a14e72ee650dc12c98ee4e1a5b432b69365a60cef8b15c9b5fa`.

| Partition | Valid rows | Refusals | Boundary controls |
|---|---:|---:|---:|
| Authoring | 320 | 16 | 0 |
| Withheld | 1280 | 64 | 16 |

The curator's population receipt is SHA256
`ad5bf0fdecb66b0de9e28c98941cf0fb2c6f737c7e1be3cbf48570822c65ba30`.
The [public curation record](r4_text_clause_adapter_1094_curation.json) preserves
the complete selection/population metadata and both preparation attempts,
including their original receipt identities and exact serialization.
Corpus, selection and policy total 3,397,265 bytes. Successful independent
preparation took 0.271631 seconds with 79,626,240 bytes peak RSS and zero model
forwards/optimizer updates. Its resources are reported separately from the
frozen comparison budget, as specified in #1085.

An initial launch used system Python 3.9.6 and stopped on unsupported
`zip(strict=...)` before any payload file write or withheld-text generation.
The exact unchanged curator source and selection then succeeded under the
existing Python 3.12.14 environment. Both immutable attempt receipts remain:
`0bccffabe574f8d79a9fef80b487acccb7e77a12e1b4a0f1395235972fbf4330`
and `ceb9582ad6cf63e8001cb8dd39505c130f68b47c0e1543e3dc90b3c3584255bd`.
This is an authoring-environment failure followed by execution of the same
frozen preparation, not a parser/model result or population revision.

Withheld raw/reference files remain under a mode-000 curator directory until
the complete source and independent review freeze. Authoring files alone may
guide parser preflight. No revealed withheld failure can be repaired and rerun
on these inputs.

## Execution and evidence accounting

The compact model binding carries only verified artifact, codec, frame, source
and runtime identities. Historical fit/preparation reports and their answer
arrays do not enter inference. Startup verifies all identities before request
admission or model construction. The two artifacts are loaded once per worker;
every parameter stays frozen and model state is compared before/after execution.

Reuse Python 3.12.14, Torch 2.7.1 and CPU Apple Accelerate with four intra-op
threads, one inter-op thread, one model worker at a time, deterministic
algorithms and batch size 128. The coordinator runs the oracle and adapter
workers sequentially. It sends only supplied IDs/lengths to O and raw requests
to A; row metadata, reference roles and targets stay in the coordinator.

The macOS file-access profile makes corpus, reference and historical result
paths unavailable to model workers while allowing the exact model assets,
source package and existing runtime. A harmless sentinel inside the denied
curator root must raise `PermissionError` before any model is constructed.
This is the declared oracle-access control, not a general sandbox-security
claim about hostile native code.

The empirical criteria remain 100% acceptance/input/span fidelity in all sixteen
form/profile cells; byte-identical full reader attention, all fifteen computed
role vectors, binding attention, full 4096-head logits and decisions; agreement
of the fourteen consumed diagnostic roles; exact typed refusals with zero
forwards; and independent fresh-process replay. Reference errors remain in the
denominators. Gold roles never select model roles or change model outputs.

The hard caps are 120 seconds preparation integrity, 120 seconds execution and
120 seconds replay, at most 360 seconds cumulative, 3 GiB peak RSS and 128 MiB
new corpus/results. The upper bound is 6400 logical row forwards, not an
obligation to execute invalid rows. Complete tensors are compared while
streaming; retained evidence uses domain-tagged digests, exact decision rows,
counts, model/work identities and bounded mismatch details. Only one temporary
oracle stream is needed for sequential comparison; it is discarded after its
bytes are compared and their receipts retained. No model downloads, fitting,
new geometry, accelerator search or broad QA is activated.

The named decision branches remain those of #1085: `CLAUSE_ADAPTER_PRESERVED`,
`CLAUSE_ADAPTER_PREFLIGHT_MISS`, `CLAUSE_ADAPTER_MISS`, the explicit
provenance/reference unavailable terminals, or `INCOMPLETE_RESOURCE`. A failure
retains its evidence and does not silently relax a threshold or renew a budget.

## Source and integration review

The original [NEMESIS/W33/UOR source audit](integration/clause-segmentation-1085-sources.md)
continues to support typed mapping, raw-byte identity and ordered provenance.
It supplies no parsing or learned-capability proof. The user-requested
[afflom ecosystem follow-up](integration/afflom-ecosystem-followup.md) inspects
Prism, both Atlas sources, LexLean, lean4-prod, GNAF and both matmul repositories.
Their concrete integration opportunities belong to #1083/#1087/#1089; the frozen
comparison's arithmetic and reader are unchanged.

The [independent review](r4_text_clause_adapter_1094_review.md) must bind final
source identities before withheld release. Only the declared preparation,
comparison/refusal/replay, claim-wording and evidence-integrity checks are
active. Protected-queue acknowledgements are transport, not scientific tests.
Storage review preserves the original mixed checkout, unique model/corpus and
diagnostic evidence, sealed files and user material.
