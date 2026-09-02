# Direct owner-plus-object query encoding at the matched dose — #1069

## Frozen contract (2026-09-02; before fitting)

Issue [#1069](https://github.com/UOR-Foundation/uor-r4/issues/1069) continues
#973 after [#1067's matched readout experiment](r4_zoology_query_readout_1067.md).
The query-object readout improved construction accuracy from 2,396/8,192
(29.2480%) to **3,735/8,192 (45.5933%)**. Object-changing question pairs were
both correct in 447/2,048 cases, while owner-changing pairs reached only
47/2,048. Wrong-owner/same-object choices accounted for 1,439/2,131 (67.5270%)
of q0 in-history errors. Displayed slots 2 and 4 received 74.2686% of in-history
selections despite balanced targets. These identify a remaining behavioral
asymmetry, not its internal cause.

This experiment changes one declared input-encoding operation. At the existing
query-object/readout position 37, use

```text
x37 = E(token37) + P37 + E(token35)
```

The source embedding otherwise returns `E(token_i) + P_i`. Add only the owner
**word** embedding from position 35, with fixed coefficient one; do not add its
position embedding. Apply this operation before the unchanged first-block
embedding dropout and attention trunk. All other positions remain unchanged.
The owner already lies in the causal prefix. This intervention changes direct
access, embedding magnitude and gradient paths together; it does not prove that
owner information was previously absent or isolate an internal mechanism.

The input retains all 41 tokens. Readout remains 37, whose literal next input
token is `?`; the supervised target is a location/unknown label. This is an
explicit joint-query answer interface, not ordinary answer-at-colon generation.

## Matched data, model and learning clock

Copy #1063's construction and vocabulary files byte-for-byte, as in #1067.
All 10,240 input/target/group/variant/type rows and their ordering remain
unchanged. The loader continues to return position 37 instead of the stored
historical position 40. Preserve two attention layers, one head, width 64,
4,096-token tied head, maximum position 120, source normalization/dropout,
and 303,744 parameters. The added connection introduces no parameters,
buffers, modules or random draws. Bind its policy in preparation and model
artifact metadata; the package loader must require that metadata and install
the same operation before inference. Weights alone do not encode this policy.

Use fresh seed-123 initialization, with no prior model/optimizer loading.
Preserve AdamW learning rate `0.00046415888336127773`, weight decay `0.1`,
batch 512, one cosine schedule over 64 blocks of 196 updates, and the original
2,352 supported-phase plus 1,568 mixed-phase updates. The only evaluated model
is the final update-3,920 artifact. Its exact ledger must match #1067:
**2,007,040 presentations = 1,846,452 supported + 160,588 unknown**.
Full forward/dropout tensor shapes and sampler/RNG ordering remain matched.
No intermediate scoring, added updates or checkpoint selection enters fitting.

Before training, freeze 256 new development groups at seed **10692**: 1,024
supported answers and 256 absent-binding answers. Retain the grammar, lexical
map, 64 held-out owner-object pairs, both question types, all five variants,
slot balance and location balance. Exclude every construction world and every
#1063/#1067 development world, including base, location-swapped and absent
variants independently of display order. Also exclude exact input overlaps
and reuse of a world across accepted new groups. Recreate historical
populations with the unchanged frozen generators and verify canonical CIDs;
never open their retained tensor payloads. Prefit preparation audits labels,
exclusions and balance. Fitting/default validation neither opens nor regenerates
development. The new tensor is opened for model scoring only after construction
passes the retained 99% gate.

## Empirical criteria and divergent decisions

The primary bounded improvement concerns owner selection with both answers
correct, not merely a change in the prediction. The fixed construction
thresholds are:

| Criterion | Required candidate count | Retained #1067 |
| --- | ---: | ---: |
| Owner-changing question pairs: both correct | at least 150 / 2,048 | 47 / 2,048 |
| Overall supported answers preserved | at least 3,735 / 8,192 | 3,735 / 8,192 |
| Object-changing question pairs preserved: both correct | at least 447 / 2,048 | 447 / 2,048 |

The owner threshold requires at least 103 additional correct pairs, or
**5.0293 percentage points**. This is a predeclared fixed-population effect
criterion, not statistical significance or full binding. The 2,048 owner pairs
are two related contrasts in each of 1,024 worlds (original and swapped
locations). Report the number of worlds with zero, one and two successful
question pairs; two corresponds to a complete quartet. Do not count these
contrasts as independent experimental units. Report both question types,
location swaps, confound categories, NLL, and per-slot behavior.

1. **All three behavior criteria pass; construction misses:**
   `JOINT_QUERY_PARTIAL_GAIN`. Retain the joint-query artifact as the improved
   construction baseline and use its recorded remaining-error/position profile
   to choose the next separate repair. Fresh development stays unscored.
2. **Either preservation floor misses:** `JOINT_QUERY_PRESERVATION_MISS`.
   Report gains and regressions separately. Reject this residual as the next
   improved baseline; retain #1067 and revise the binding learning recipe.
3. **Preservation passes, owner threshold misses:**
   `JOINT_QUERY_BELOW_DECLARED_OWNER_GAIN`. Preserve any smaller measured gain,
   but retain #1067 as the baseline and revise the binding learning recipe.
4. **Construction reaches at least 8,111/8,192:** score fresh development once
   on the sole final artifact. Full binding requires at least 973/1,024
   supported answers, 231/256 complete groups, 116/128 complete groups for each
   question type, and 244/256 unknown answers. Return
   `JOINT_QUERY_FRESH_TRANSFER_MISS` or `JOINT_QUERY_FRESH_BINDING_PASSED`.
   A miss preserves construction only; a full pass permits a separate unchanged
   R4 inference-preservation step. No R4 execution belongs to this issue.
5. **Interrupted fit:** retain the exact partial checkpoint/work and actual
   numerical/admission/resource limit. An incomplete fit is not a scientific
   negative and never receives a restarted budget.

Construction pass with behavior failure is an integrity inconsistency:
8,111 correct permits only 81 wrong rows, hence at least 2,048 − 81 = 1,967
both-correct pairs in each question type. No legitimate full-construction pass
can fail the lower owner or preservation floors.

The three behavior criteria are separate from full binding. In particular,
`behavior.passed=true` with a construction miss still has overall
`passed=false`. A partial gain never opens fresh development.

## Resources and focused verification

Retain the combined 1,800-second fit/evaluation/replay and 4-GiB RSS ceilings,
eight CPU Apple Accelerate threads, one inter-op thread, 1.25 admission
projection multiplier and 60-second evaluation allowance. The first eight
real updates count toward the 3,920 dose and supply admission. #1067 completed
in 290.31 seconds at 0.7764 GiB peak. One additional owner lookup/vector addition
retains the attention tensor shapes. All 8,192 supported decisions have the
owner, object and answer facts available by position 37; no reachability
restriction excludes most of the measured population.

Named checks cover adapter locality, causal source, gradients, unchanged
state/parameters/RNG and artifact reconstruction; full-trainer AST equivalence
apart from the declared adapter/metadata/identities; copied data and fresh
population semantics/exclusions/access; owner/preservation/development decisions;
world-level dependent-pair accounting; final artifact/state binding; and exact
fresh-process complete-evidence replay. Independent source/preparation/evidence
review is required. Broad workspace, BDD, WASM, fuzz, audit and release QA
remain dormant; protected queue statuses acknowledge transport only.

## Commands

From `tools/r4-softmax-trainer`, in the locked offline environment:

```bash
uv sync --frozen --offline
.venv/bin/python -m r4_softmax_trainer.zoology_joint_query prepare \
  /Users/casey.allard/uor-r4/.uor-models/research/issue-1069-zoology-joint-query \
  --source-root /Users/casey.allard/uor-r4/.uor-models/research/issue-1063-zoology-english-binding
.venv/bin/python -m r4_softmax_trainer.zoology_joint_query fit \
  /Users/casey.allard/uor-r4/.uor-models/research/issue-1069-zoology-joint-query
.venv/bin/python -m r4_softmax_trainer.zoology_joint_query run \
  /Users/casey.allard/uor-r4/.uor-models/research/issue-1069-zoology-joint-query
.venv/bin/python -m r4_softmax_trainer.zoology_joint_query verify \
  /Users/casey.allard/uor-r4/.uor-models/research/issue-1069-zoology-joint-query
```

Preparation/result/replay files are immutable. Resume can only continue the
same bound checkpoint within the original cumulative clock. Evaluation/replay
perform no training and use the explicitly adapted artifact loader. Preserve
#1063/#1065/#1067 evidence and #1059/#1061 attention positives. More geometry,
generation, general English, reasoning, chat readiness, H4 superiority, softmax
removal and integer/table lowering remain deferred or unestablished. #973 stays
open and #954 blocked.


## Published preparation

Source and contract freeze: `b3d8dc60`. All **12 focused methods passed**:
three adapter/state/reconstruction checks, one whole-trainer AST comparison,
three data/exclusion/access checks, and five decision/interface/world-accounting
checks. Independent source review found no blocking issue. These checks used
synthetic inputs; no fitted artifact or historical development payload was read.

The [preparation envelope](r4_zoology_joint_query_1069_preparation.json) binds
**238 implementation files**, the original construction/vocabulary bytes,
the complete retained baseline and work ledger, and the declared owner residual.
Actual preparation audited all 10,240 construction and 1,280 new development
labels. Fresh development has 768 distinct base/swapped/absent worlds, zero
world/input overlap with construction or either historical development
population, all 64 held-out owner-object pairs, and balanced target locations
and relevant slots. Each historical development population has 1,280 rows and
768 worlds; the per-history audit records distinguish #1063 and #1067.
The inherited top-level `historical_development_rows` and
`historical_development_canonical_worlds` fields refer to the original #1063
population, not the union; use the per-history records for the two exclusions.

- Preparation: `blake3:e2709d32436f7979aeda795f2ec735d99932cdd1037e7e17321d72a7009ad7e1`
- Implementation: `blake3:e5436a8c0d8e2e92312721f54164b05269c5a6138a52f54e4574bb3fa7de1e2d`
- Dataset manifest: `blake3:3597f258750242bd1b9482e234b4ed939375d4a32c08c37a76d1a500cfe9e490`
- Dataset tree: `blake3:f85ed814f31be179fbe85d6bc08caedb323c742adba0b0a1660ffff330714119`
- Fresh development: `blake3:cbee180e7e37fe302d9cffd198d7af91ed42ed9851bbbf91e7ee4814d0e0a9b1`
- Unchanged construction: `blake3:d767fafdf544f01db99d9acb317c76df55e9f9d28f99785d2a6ae62b663731a2`

The frozen behavior counts are owner at least 150/2,048, overall at least
3,735/8,192, and object at least 447/2,048. Construction must separately reach
8,111/8,192 before any fresh development model decision. These identities and
criteria are published before the first optimizer update.
