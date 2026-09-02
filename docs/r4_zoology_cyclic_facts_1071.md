# Cyclic fact-order learning at the matched dose — #1071

## Frozen contract (2026-09-02; before fitting or model scoring)

Issue [#1071](https://github.com/UOR-Foundation/uor-r4/issues/1071) continues
#973 after [#1069](r4_zoology_joint_query_1069.md). The owner residual raised
aggregate construction accuracy to 50.2686%, but failed object-pair preservation
and reached 99.6582% at target slot 4 versus 29.4922–36.0352% elsewhere.
The retained reference is the plain [#1067 query-object readout](r4_zoology_query_readout_1067.md).
This experiment teaches the same bindings across positions before changing
geometry. One fresh fit is authorized; no extra dose or retuning follows.

## Intervention, exact dose and reachability

After the unchanged sampler returns each batch, reshape input tokens 1–32
into four eight-token fact blocks and **right-rotate** the fact axis by:

```text
supported phase: floor(completed_updates / 16) modulo 4
mixed phase:     floor((completed_updates - 2352) / 20) modulo 4
```

`completed_updates` is the zero-based count before the next update. The offset
resets to zero at update 2,352. Each full supported traversal has 16 batches;
each mixed traversal has 20. All variants of a world share the same offset
within a traversal. Clone-and-roll also executes at offset zero, so the first
eight real updates measure the same transform path used later. Rotation uses
no random draws and changes no labels or metadata. Original construction and
vocabulary bytes stay identical. BOS token 0, query suffix 33–40, all 41 input
tokens and supervised position 37 stay intact. Each world has distinct
owner-object keys; order changes preserve answer and absence semantics.

Use the plain source model: two attention layers, one head, width 64, tied
4,096-token head, maximum position 120 and 303,744 parameters. No owner residual.
Preserve seed 123, fresh initialization, source normalization/dropout, AdamW
learning rate 0.00046415888336127773, weight decay 0.1, batch 512, and the
original cosine clock. The sole evaluated candidate is update 3,920:
2,352 supported plus 1,568 mixed updates, **2,007,040 presentations =
1,846,452 supported + 160,588 unknown**. Sampler/RNG and dropout tensor shapes
remain matched; the sampled token values are rotated. No prior learned state
or optimizer is loaded into fitting.

Expected total presentations by offset are **507,904 / 507,904 / 501,760 /
489,472**. Supported-phase updates are 592 / 592 / 592 / 576; mixed-phase
updates are 400 / 400 / 388 / 380. The mixed phase ends after 78 full traversals
plus eight batches at offset 2. Derive supported/unknown exposure from the
actual existing counters, including the partial tail, rather than assuming a
20% unknown fraction. The derived ledger adds no checkpoint state.

The readout is an explicit supervised answer at the query object; its literal
next input token is `?`. This does not establish ordinary generated answers.
All 8,192 supported construction decisions have the four facts available in
the causal prefix. Canonical paired headroom is 2,001 owner and 1,601 object
pairs, above the declared 205-pair gain. No geometric expansion is needed to
reach this population.

## Matched comparisons and divergent decisions

After the fit, score the retained #1067 model and candidate on the same 8,192
supported rows at offsets 0, 1, 2 and 3. Offset zero **is** canonical order,
not a fifth population. Require the reference's entire canonical score and
diagnostic to reproduce the published #1067 evidence exactly before interpreting
the comparison. Retained reference weights are hashed during preparation and
loaded only for final scoring/replay; default fitting validation does not open
them. The checked plain-model loader binds artifact and learned-state identities.

In **every rotation**, adoption requires:

- For each question type, both-correct pairs at least
  `min(same-rotation reference + 205, 2048)`. The untruncated gain is
  10.009765625 percentage points. Flag ceiling-limited cases explicitly.
- Each of four target slots at least **1,024/2,048 = 50%** correct.
- Overall supported accuracy and each individual question-family accuracy
  preserved against that matching reference. Any paired, individual-family or
  overall regression is reported as a tradeoff and keeps #1067 as reference.

Report per-order NLL, paired question/location-swap behavior, confounds, slots
and fixed first-quartet examples. Two question pairs per world and four cyclic
views are correlated observations. Report worlds with zero/one/two successful
question pairs per order, and world counts with zero through four completely
correct rotations. No independence or significance claim follows. Four cyclic
orders cover only four of the 24 possible fact permutations.

1. Behavior passes but any construction view is below 8,111/8,192:
   `CYCLIC_FACTS_PARTIAL_GAIN`. Retain the improved augmentation recipe and
   address its recorded remaining errors separately. Development stays unscored.
2. Gain/slot criteria miss without regression:
   `CYCLIC_FACTS_BELOW_DECLARED_GAIN_OR_SLOT_FLOOR`. Preserve partial progress,
   retain #1067, and revise the binding learning recipe.
3. Any family/overall regression: `CYCLIC_FACTS_PRESERVATION_MISS`.
   Preserve measured tradeoffs and retain #1067.
4. **Both behavior and all four construction >=8,111/8,192 pass**: open fresh
   development for the final candidate in all four cyclic orders. Require per
   order 973/1,024 supported, 231/256 complete groups, 116/128 complete groups
   for each question type, and 244/256 unknown answers. Return
   `CYCLIC_FACTS_FRESH_TRANSFER_MISS` or `CYCLIC_FACTS_FRESH_BINDING_PASSED`.
   A transfer miss preserves construction; all pass permits a separate unchanged
   R4 inference-preservation step.
5. Incomplete/resource/integrity outcomes are explicit and do not become
   scientific negatives. Retain the exact partial checkpoint and cumulative
   clock; no restarted budget or replacement fit.

A 99% construction pass can legitimately miss the relative behavior criterion
against an unusually strong rotated reference. In that case return
`NOT_RUN_BEHAVIOR_MISS` for development; it is not an integrity contradiction.
A construction miss uses `NOT_RUN_CONSTRUCTION_MISS`. Both cases have zero
fresh-development model decisions.

## Fresh population and access

Before fitting, freeze seed **10712**, 256 fresh groups with all five variants:
1,024 supported and 256 unknown answers, original grammar/lexical map,
64 held-out owner-object pairs and balanced question types, slots and locations.
Exclude construction and **#1063/#1067/#1069** development base,
location-swapped and absent-binding canonical worlds, regardless of display
order, plus exact input overlaps. Reject world reuse across new groups.

Recreate historical populations using their unchanged generators and verify
canonical tensor CIDs against published records; never open retained historical
development payloads. Preparation audits labels, exclusions and balance.
Fitting/default validation neither opens nor regenerates development. Record
per-history and union counts explicitly. Fresh transfer, if admitted, still
concerns familiar grammar and held-out combinations rather than general English.

## Resources, source review and commands

Keep the combined **1,800-second / 4-GiB** ceiling over fit, eight construction
views, conditional development and exact fresh-process replay. Use eight Apple
Accelerate CPU threads and one inter-op thread. Preserve the first-eight-real-
update admission rule, 1.25 projection multiplier and 60-second evaluation
allowance. The prior 277-second fit and 1.8-second single view motivate a
sub-six-minute estimate; the cumulative cap remains binding.

Focused verification covers fact/causal semantics, RNG, phase boundaries,
actual rotation exposure, whole-trainer AST equivalence except declared changes,
data copying/exclusions/access, matched decisions, both development gates,
correlated-world accounting, canonical reference reproduction, immutable model
and state binding, and exact fresh-process complete-evidence replay. Independent
source, preparation and evidence reviews are required. Broad workspace, BDD,
WASM, fuzz, audit and release QA remain dormant; queue statuses acknowledge
transport only.

From `tools/r4-softmax-trainer` in the locked environment:

```bash
uv sync --frozen --offline
.venv/bin/python -m r4_softmax_trainer.zoology_cyclic_facts prepare \
  /Users/casey.allard/uor-r4/.uor-models/research/issue-1071-zoology-cyclic-facts \
  --source-root /Users/casey.allard/uor-r4/.uor-models/research/issue-1063-zoology-english-binding \
  --reference-root /Users/casey.allard/uor-r4/.uor-models/research/issue-1067-zoology-query-readout
.venv/bin/python -m r4_softmax_trainer.zoology_cyclic_facts fit \
  /Users/casey.allard/uor-r4/.uor-models/research/issue-1071-zoology-cyclic-facts
.venv/bin/python -m r4_softmax_trainer.zoology_cyclic_facts run \
  /Users/casey.allard/uor-r4/.uor-models/research/issue-1071-zoology-cyclic-facts
.venv/bin/python -m r4_softmax_trainer.zoology_cyclic_facts verify \
  /Users/casey.allard/uor-r4/.uor-models/research/issue-1071-zoology-cyclic-facts
```

Preparation/result/replay files are immutable. Publish preparation before the
first update. Append final evidence, reconcile six current mirrors and native
trackers, deliver through the protected PR path, then close only #1071.
Historical records remain intact. #1059 retains 11,900/12,000 = 99.1667%
preservation; #1061 retains ordinary/coherent R4 both at 8,071/8,192 = 98.5229%,
all predictions identical and an 86.2061-point transport-control loss.
More geometry, native-frame work, generation, general English, H4 superiority,
softmax removal, reasoning and chat readiness remain deferred or unestablished.
#973 stays open and #954 blocked.


## Published preparation

Source freeze: `98ea9564`. All **15 focused methods passed** in 8.451 seconds:
three rotation/ledger checks, one whole-trainer AST check, three data/exclusion/
access checks, two reference-binding checks, and six comparison/reveal/world-
accounting checks. Ruff and claim wording passed. Independent source reviews
found no blockers. No fitted model had been scored or trained.

The [preparation envelope](r4_zoology_cyclic_facts_1071_preparation.json) binds
254 implementation files, original construction/vocabulary bytes and the plain
#1067 comparison artifact. Preparation hashes that artifact without loading model
tensors. Its complete published baseline and exact dose are retained.

The audited new population has 1,280 rows and 768 distinct base/swapped/absent
worlds, 64 held-out owner-object pairs, and balanced slots/locations/question
types. Each of the three historical populations has 1,280 rows and 768 worlds;
their union contains 3,840 inputs and 2,304 worlds. Including construction,
exclusion covers 14,080 distinct inputs and 8,448 canonical worlds. Every
new-development overlap count is zero. The default validation path does not
open or regenerate development.

- Preparation: `blake3:92ffef681fe6bd4cfc6532bd811496b6e9c4d052b978c172d3a5ca9bcef8ef05`
- Implementation: `blake3:4693fca24e67aa4bab43ddfc71ee144cb3771c7fca32abea5cb55cdb4356b761`
- Dataset manifest: `blake3:6eeb4831d9558dbbb0ac1235b314fb47e1412a561aad429d2af56acd81dc0a54`
- Dataset tree: `blake3:4856c4365443f562803880a14013569c8ce7c944eda4a174d4b1d8f50f97da1c`
- Fresh development: `blake3:31dca3ff6327c75bd5208758dcd748661002c8bb109346d8c2d4a3b414c02ed9`
- Retained reference model: `blake3:9386849d191b038803ae30b267f2fbf654cb077f48d5f3ddc92137c60875dd98`

These identities, the capped 205-pair gains, each-slot 50% floor, family/overall
preservation and both development-admission gates are published before the
first optimizer update.
