# R4 learned candidate-leaf associative-readout prompt-capacity freeze (#973)

- **Issue:** [#973](https://github.com/UOR-Foundation/uor-r4/issues/973)
- **Authoritative public freeze:**
  [issue comment 5498390609](https://github.com/UOR-Foundation/uor-r4/issues/973#issuecomment-5498390609)
- **Live base:** `92aadcf01623792a03b9d7bb03c4c0df96afbda4`
- **Branch:** `issue-973-learned-associative-readout`
- **Mechanism policy:** `R4LearnedCandidateLeafAssociativeReadoutV1`
- **Campaign policy:** `R4LearnedAssociativeReadoutPromptCapacityV1`
- **Status:** `FROZEN_ARCHITECTURE / POPULATIONS_NOT_CREATED / NOT_RUN`
- **Qualified predecessor:** `R4RetainedLanguagePathV1`, artifact CID
  `blake3:d1417b325e7a545057cd38e9f1a723933a3682801877433d20e98774a5e9172d`
- **Outcome:** `NOT_RUN`

## Decision and boundary

The parameter-free retained-readout ladder ended at
`LAYERWISE_NORMALIZED_RETAINED_READOUT_PROMPT_CAPACITY_PARTIAL`. This record
freezes the one authorized learned successor before implementation, V4
population creation, optimization, or outcome access. It adds candidate-relative
learned query capacity over the qualified retained value field while preserving
the accepted V1 recurrence and language path.

This document is a public architecture, data-boundary, and decision freeze. It
contains no V4 population contents, trained candidate artifact, reveal, score,
result, or verification claim. A commitment, successful mechanics probe, or
create-once seal will not by itself be a model result.

## Exact frozen mechanism

**Definition.** For layer `l`, token step `t`, and exact-H4 relative address
`g`, `V[l,t,g]` is V1's strict-prior, post-transport, post-decay value field
immediately before the current write. `lambda(c)` is candidate token `c`'s
existing canonical exact-H4 leaf, `h[t]` is the unchanged final V1 hidden
state, `E` is the unchanged tied embedding/head, `N` is the existing final
RMSNorm, and `R` is a parameter-free RMS normalization that maps an empty zero
field exactly to zero.

**Definition.** The geometric arm learns one independent candidate-query table
`qG[l,c] in R^48 = (R^4)^12` and scores

```text
zG[t,c] = <E[c], N(h[t])>
         + (1 / (2*sqrt(48))) * sum_l <qG[l,c], R(V[l,t,lambda(c)])>
```

V1's separate learned key/value projections, exact-H4 recentering, four decay
timescales, delta writes, occupied-slot softmax, read-before-write order,
residual/MLP path, tied base head, and persistent state remain unchanged. The
new readout adds no direct key-energy term and is not defined as
QK-softmax-value attention.

**Definition.** The matched address-blind arm learns its own unshared query
table `qP[l,c]`, with byte-identical shape and initialization, and uses the same
score form. It replaces candidate-leaf selection with the occupied-address mean
of V1's same strict-prior transported value field before `R`. This is the
equal-parameter learned non-geometric-readout control over the same geometric
backbone.

Both query tables have shape `[2,4096,12,4]`, initialize to exact zero, and are
optimized independently. Each effective arm therefore reproduces qualified V1
logits before learning while retaining live first-step gradients. Each arm adds
exactly `393,216` learned f32 values: `645,376` total parameters per effective
arm, of which only the `393,216` new values are trainable. The predecessor's
`252,160` parameters are immutable. Recurrent state remains exactly `23,040`
f32 values / `92,160` bytes plus `240` validity bits.

A shared-feature implementation may train both heads in one process, but their
parameters, losses, optimizer slots, checkpoints, and artifacts remain
disjoint. Exact-leaf gather and occupied pooling both execute in every training
and evaluation call so the compared head work is fixed. A head-off intervention
must be byte-identical to qualified V1. Full state-off must zero both V1's
retained contribution and the added associative score.

## Geometry-destroying intervention

**Definition.** The fixed-leaf control cyclically deranges the sorted set of
actually used token leaves and applies the same derangement to every candidate
read. It does not map candidates into structurally unused slots. It reuses the
trained geometric query table and otherwise executes identical scoring work.
This isolates candidate-to-exact-H4-address binding from the mere presence of a
learned vocabulary table.

A row of twelve R4 vectors plus exact-H4 candidate-address selection is a
geometric representation under the repository's qualified convention. This
freeze does not define intrinsic Spin(4) transport, a learned orthogonal 4x4
action, or H4 superiority.

## Frozen fit and reachability

Both learned heads receive the exact predecessor training slice and BLAKE3
order: `43,680` nonoverlapping 121-token windows, `5,241,600` causal decisions,
batch `16`, seed `9738`, and exactly `2,730` AdamW steps. AdamW remains betas
`0.9/0.95`, epsilon `1e-8`, weight decay `0.1`, gradient clip `1.0`, a 100-step
linear warmup to `3e-4`, then cosine decay to `3e-5`. The two independent
cross-entropies use the same frozen-base features.

There is one trajectory. No sweep, alternate seed, table/rank change,
continuation, or scientific retry is authorized.

The new seam is structurally reachable on `5,197,920/5,241,600` training
decisions and `245,854/247,920` fresh-language decisions: 119 of 120 positions
in every window. All `8,192/8,192` prompt target tokens occur after a complete
48-token prompt and are reachable. These counts establish reachability only,
not predictive value or attention quality.

## Independently frozen data boundary

`R4RetainedPromptSwapContrastV4` preserves V3's selection law of `256` pairs,
`512` directions, and `8,192` target tokens. Selection must begin strictly after
revealed V3 source ordinal `324,230` and exclude the exact 1,536-story V1+V2+V3
union. That exclusion witness is
`blake3:e8d02abcf9ab326545afa80c5191285ec37110cf73f0d389cd6a2f75fcd5c121`;
the V3 population CID is
`blake3:165be397b73041afd39aa65ae796400ea539399f8586729ad19a168c4daa9e93`.

The fresh-language slice is fixed by source coordinates before materialization:

- token range `[156,032,138, 156,282,124)`;
- `249,986` tokens, `2,066` windows, and `247,920` decisions;
- slice CID
  `blake3:77dfa0744c140e5affe9be233244e616c940dbff469f786deadeb87768e3c752`;
- capacity stories `764,050..765,247`;
- source stories `848,493..849,802`;
- 1,198-story witness
  `blake3:c112790145657c771cf72d63d8e1f055b3b2d772f1cd3485c7cacb74dbb1e4a0`;
  and
- canonical index CID
  `blake3:0032889e32b38801476223c5bed7e401d77b61afbbd6cf9afddaceee18e2136e`.

V4 prompts and fresh heldout must be created and sealed together in one
create-once mode-`000` evaluation directory. A mechanics/performance probe may
read training batches only. Qualified V1 and both final learned-head artifact
CIDs must be fixed before one immutable reveal marker opens either evaluation
population. After reveal, optimization is permanently closed. The only allowed
recovery is result finalization from an already-completed identical checkpoint;
it may not construct an optimizer, fit, or change an artifact.

At this freeze, the V4 population, population commitment, preparation, probe,
run contract, learned artifacts, reveal, result, and independent verification
are all `NOT_CREATED` or `NOT_RUN` as applicable.

## Binding empirical criteria

**Empirical Criterion.** Each learned arm earns `ASSOCIATIVE_CAPACITY_PASS` only
if every condition below passes on the one V4 reveal:

- prompt gain `G >= ln(2)/16 = 0.04332169878499658`;
- arm-minus-V1 prompt gain
  `>= ln(1.5)/16 = 0.025341569256760274`;
- at least `308/512` own-prompt directional wins;
- own-prompt NLL no worse than V1;
- fresh-language NLL no more than `0.05` behind V1 and top-1 no more than
  `1.0` percentage point behind V1;
- full state-off costs at least `0.10` NLL and `2,480` correct fresh-language
  decisions, while its prompt contrast collapses within `1e-7`; and
- head-off/V1 identity, strict causality, stationary/direct parity, finite
  active gradients, artifact replay, fixed work, and zero forbidden reads all
  pass.

**Empirical Criterion.** `GEOMETRY_ATTRIBUTED` is a separate, stronger verdict.
It additionally requires geometric-minus-pooled prompt gain and
geometric-minus-fixed-leaf-deranged gain each to be at least
`ln(1.5)/16 = 0.025341569256760274`, at least `308/512` paired directional
improvements against each, and geometric own-prompt NLL no worse than either
control. A generic learned-head gain without both separations is not geometric
advantage.

Both criteria are **Unproven** and `NOT_RUN`.

## Predeclared outcome actions

- If the geometric arm passes capacity and geometry attribution, preserve it
  and authorize one separately frozen, disjoint autonomous
  subject/scene-retention smoke.
- If one or both learned arms pass capacity without geometric separation,
  retain the learned-readout result, report geometry attribution unestablished,
  and authorize one smoke only from the passing arm with the lowest frozen
  fresh-language NLL.
- If prompt capacity passes but fresh-language nonregression fails, stop before
  generation and freeze a joint language/capacity objective next.
- If neither head passes capacity, reject this exact learned query-to-value
  readout without table, rank, scalar tuning, or retry, then revisit the
  retained value representation/binding law.
- If mechanics are invalid, repair mechanics only and make no model claim.
- If compute is unavailable, preserve or resume only the identical pre-reveal
  trajectory, with no architecture or threshold change.

## Compute contract

A training-only probe will compare Apple Accelerate CPU with four threads, CPU
with eight threads, eligible two-worker CPU execution, and sequential MPS using
one warmup plus five measured steps. Eligibility requires numerical agreement
with CPU4 within `5e-5`, memory below `80%`, and a 1.25x wall projection. The
measured-fastest eligible plan is binding. Final scoring and fresh-process
verification remain canonical Apple CPU. CUDA and external GPU execution are
forbidden.

Runs projected beyond 15 minutes must emit durable progress, ETA, per-head
checkpoints, and exact resume instructions. The whole-process hard ceiling is
two hours.

## Nonclaims

A future positive result could establish only bounded learned associative
prompt capacity over qualified V1. Geometry attribution would require the
separate frozen control criteria above. No result from this rung, by itself,
establishes coherent generation, reasoning, correctness, intrinsic Spin/H4
superiority, exact/table/`no_std` lowering, browser readiness, release
readiness, #973 closure, or #954 unblocking.

A future negative would falsify only this exact frozen query-table/readout law.
At the present freeze, every empirical outcome remains `NOT_RUN`.
