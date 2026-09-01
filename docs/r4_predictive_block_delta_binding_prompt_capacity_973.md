# #973 predictive R4 block-delta binding and prompt capacity

- **Freeze date:** 2026-09-01
- **Issue:** #973
- **Programme root:** #820
- **Mechanism:** `R4PredictiveBlockDeltaBindingV1`
- **Campaign:** `R4PredictiveBlockDeltaPromptCapacityV1`
- **Evidence status:** `FREEZE_CORRECTED_BEFORE_IMPLEMENTATION_OR_V5_POPULATION_CREATION`
- **Current result:** `NOT_RUN`
- **Generation:** `NOT_AUTHORIZED`

## Result first

The next #973 experiment changes the retained **write and binding law**. It
does not add another readout over the failed 120-slot value field.

The completed learned-associative experiment showed that the qualified V1
field can help language modeling when pooled, but does not place enough
prompt-specific information at the exact candidate address. The geometric
head gained only `0.0063767854` nats/direction and the pooled head gained
`0.0102632346`, below the frozen `0.0433216988` absolute and `0.0253415693`
incremental capacity floors. A fixed candidate-leaf read also has a structural
reachability problem: 4,096 tokens collapse onto 35 directly used H4 leaves,
while V1 writes only the identity slot after transport.

A read-only census of the immutable V1 schedule makes that defect concrete:

| Population | True target leaf occupied |
| --- | ---: |
| Construction train, all positions | `1,885,115 / 5,241,600 = 35.9645%` |
| Construction train, positions 47 through 62 | `35.9130%` |
| Validation, all positions | `89,069 / 247,920 = 35.9265%` |
| Validation, positions 47 through 62 | `35.8906%` |
| First post-prompt position, train / validation | `32.3375% / 30.8809%` |

The earlier “reachable” census meant that some state existed. It did not mean
the true candidate's leaf held a value. A fixed-leaf readout therefore had an
approximately 36% structural opportunity ceiling before value quality was
even considered. The new dense block memory makes every written association
query-readable; its reachability ceiling is 100% by construction.

This freeze replaces that address-dependent association with a bounded
key-to-value matrix memory. The key is the previous causal context; the value
is the token that is subsequently observed. The value and candidate scorer
remain anchored to their immutable token H4 leaves so the geometry cannot
cancel into a pure change of basis.

No V5 prompt or fresh-language target is created or inspected until the code,
mechanics checks, and revealed-data expressivity gate below pass unchanged.

## Frozen architecture

The complete qualified `R4RetainedLanguagePathV1` artifact remains immutable.
It supplies the causal hidden state `h_t`, its ordinary tied output embedding,
and the existing V1 logits. Four new bias-free `48 x 48` maps `Wq`, `Wk`,
`Wv`, and `We`, plus four-bank `rho`, `eta`, and `alpha` logits, are the only
trainable values.

| Item | Frozen value |
| --- | ---: |
| Qualified V1 parameters | `252,160`, immutable |
| New trainable parameters | `4 * 48 * 48 + 3 * 4 = 9,228` |
| R4 blocks | `48 / 4 = 12` |
| Retention banks | `4` |
| Matrix state | `4 * 12 * 4 * 4 = 768` f32 values |
| Pending previous-context key | `12 * 4 = 48` f32 values |
| Total new recurrent state | `816` f32 values / `3,264` bytes |
| Additional bounded metadata | one H4 frame index and one key-valid bit |

Let `F_t` be the cumulative canonical H4 frame after observing token `x_t`,
and let `L(c)` be candidate `c`'s immutable canonical token-leaf frame. A
registered H4 frame matrix decodes local coordinates to model coordinates, so

```text
T(a -> b) = F_b^T F_a
```

maps coordinates from frame `a` into frame `b`. The compiler-side f32 table is
derived from, and content-bound to, Rust's canonical exact-H4 root registry;
no independently invented or learned coordinates are allowed.

For each bank and each of the 12 independent R4 blocks, observing `x_t`
executes this strict causal schedule:

```text
P_t       = T(F_(t-1) -> F_t)
S_bar     = P_t S_(t-1) P_t^-1
k_bar     = P_t k_(t-1)
v_t       = T(L(x_t) -> F_t) block(Wv E[x_t])
S_t       = rho * S_bar
            + eta * (v_t - (rho * S_bar) k_bar) outer k_bar
q_t       = block(F_t^T Wq N(h_t))
r_t       = sum_bank softmax(alpha)_bank * S_t q_t
e_(t,c)   = block(T(L(c) -> F_t) We E[c])
z_(t,c)   = zV1_(t,c) + dot(e_(t,c), r_t) / sqrt(48)
k_t       = block(F_t^T Wk N(h_t))
```

Every R4 key block is normalized with the frozen denominator
`max(||k||_2, 1e-6)` before the delta update. `rho=sigmoid(rho_logit)` and
`eta=sigmoid(eta_logit)`. The read and logits for `x_t`'s prediction occur
before `x_(t+1)` is observed. The update shown above binds the already stored
`k_(t-1)` to the now-observed `x_t`; it never writes an unobserved target.
At BOS, the key-valid bit is false, so the complete update work executes but
its write delta is multiplied by zero. After each prediction, `k_t` is retained
for the next observation.

This is a compiler-side floating-point learning experiment. It makes no claim
yet about integer/table lowering or the multiplication-free deployed kernel.

## Why the leaf anchors are load-bearing

Transporting every q/k/v/readout vector coherently by the same cumulative
frame and then back-transporting the result would make exact H4 and identity
transport algebraically equivalent. Such a test could never attribute an
effect to geometry.

Here the observed value is anchored at `L(x_t)` and every candidate scorer at
its own `L(c)` before both are expressed in the current route frame. The plain
arm removes all of those transports while retaining identical parameter,
state, contraction, and optimizer budgets. The fixed transport-permuted
intervention breaks the canonical connection while preserving the same
matrices and work.

## Frozen arms and interventions

1. **Geometric:** the complete law above with the canonical H4 frame table.
2. **Plain:** an independently fitted equal-parameter arm in which every
   `P_t`, `F_t`, and `L(c)` matrix is identity; all matrix contractions still
   execute.
3. **Transport-permuted:** evaluate the fitted geometric arm with the frozen
   identity-fixing, non-homomorphic H4 connection permutation while preserving
   canonical token value and candidate anchors.
4. **No-delta-overwrite:** evaluate the fitted geometric arm with the matched
   additive/Hebbian update `rho * S_bar + eta * v outer k`. Both `v` and the
   full residual `v - (rho * S_bar) k` are computed before the frozen selector,
   so this removes only key-specific replacement while preserving a live write,
   transported decay, parameters, and work.
5. **State-off:** execute the full geometric arm and multiply only its final
   logit addition by zero. This must reproduce qualified V1 logits exactly.
6. **Order-shuffled:** rebuild state under one pre-bound within-sequence order
   permutation for a causal order-sensitivity audit only.
7. **External comparators:** immutable qualified V1 and the completed frozen
   pooled learned-readout artifact are rescored, never refitted.

The geometric and plain trainable tensors are byte-identical at
initialization, see the same construction windows in the same order, and use
the same optimizer schedule. CUDA is forbidden.

## Data boundary

The new heads may train on the already bound #989/#953 construction view:
`43,680` windows, context `120`, batch `16`, exactly `2,730` optimizer steps
and `5,241,600` token presentations in the existing deterministic order. The
qualified V1 artifact and all of its parameters remain frozen.

Revealed V4 data may be used only by the disposable cheap gate below. It is
not terminal evidence and no weights from that gate may enter the final fit.

If and only if the cheap gate passes, preparation creates and jointly seals:

- a V5 set of `256` matched prompt pairs / `512` directions / `8,192` scored
  tokens, selected strictly after V4 source ordinal `409,546` and excluding
  the exact `2,048`-story CID union of V1 through V4; and
- a story-disjoint fresh-language slice of `249,986` tokens / `2,066` windows
  from token offsets `[156,282,226, 156,532,212)`, beginning at capacity story
  `765,248` / source story `849,803` and ending at capacity story `766,489` /
  source story `851,190`, with the complete `1,242`-story CID set bound.

Both populations are selected by frozen coordinates and content identities,
stored mode `000`, and committed before either fitted artifact CID is known.

## Cheap hard gate before V5 creation

The implementation first has to pass focused mechanics checks:

- canonical frame-table identity, orthogonality, closure binding, and
  independent reload;
- read-before-write causality and a counterfactual unobserved-target mutation;
- nonzero finite gradients for every one of the `9,228` trainable values;
- deterministic replay and equal geometric/plain work ledgers;
- exact state-off reproduction of V1 logits;
- nontrivial transport equivariance and a destructive-permutation effect; and
- an observability check that two prior contexts followed by different
  observed tokens create different, query-readable matrix bindings.

It then performs one disposable overfit on the first 32 already revealed V4
pairs (`64` directions / `1,024` targets), for at most `256` updates. Admission
requires all of:

```text
own-minus-foreign prompt gain >= ln(2) / 16 = 0.0433216988
own-prompt wins              >= 52 / 64
gain lost without delta overwrite >= ln(1.5) / 16 = 0.0253415693
gain lost under state-off     >= ln(1.5) / 16 = 0.0253415693
```

The weights are destroyed after scoring. A miss terminates this version as
`PREDICTIVE_BINDING_NOT_OBSERVABLE`; V5 is not created, no full fit starts,
and the next action is to inspect the key/value placement or causal schedule.
A pass freezes the implementation CID and authorizes exactly one V5 campaign.
Thresholds and code do not change within the version.

## Terminal decision contract

The terminal geometric arm has prompt-conditioning capacity only if all of
these hold on unopened V5:

```text
own-minus-foreign prompt gain >= 0.0433216988
gain over qualified V1        >= 0.0253415693
gain over frozen pooled head  >= 0.0253415693
own-prompt wins               >= 308 / 512
own-prompt NLL                <= both comparator own-prompt NLLs
```

Fresh-language retention additionally requires NLL no more than `0.05` above
the better immutable comparator and top-1 no more than `1.0` percentage point
below it. State must be load-bearing, all causal/mechanics/work/replay checks
must pass, and post-reveal optimizer steps must equal zero.

Geometry is attributed separately only if the geometric arm beats both the
independently fitted plain arm and the transport-permuted intervention by at
least `0.0253415693` gain, at least `308/512` paired improvements, and no worse
own-prompt NLL. Key-specific delta overwrite is attributed under the same
rules against the live additive no-delta-overwrite intervention.

- **Capacity positive, geometry positive:** preserve this cell and freeze one
  bounded autonomous generation rung before any lowering work.
- **Capacity positive, geometry negative:** preserve predictive delta memory
  as an attention-capable non-geometric control; do not claim geometric
  attention, and isolate only the leaf/connection term next.
- **Capacity negative:** reject this write/binding law without generation,
  corpus expansion, another readout, or exact-runtime lowering.
- **Any integrity failure:** terminal `INVALID`; repair the harness, not the
  model, and do not interpret scores.

## Compute contract

The focused disposable gate is budgeted at five minutes. Before the terminal
fit, a construction-only timing probe compares CPU `4`, CPU `8`, and two
independent CPU workers with `4` threads each under Apple Accelerate and
ordered deterministic reduction. The measured fastest eligible plan is used.
The expected full fit is `20-35` minutes; hard wall time is `3,600` seconds
before scoring and independent verification. A timeout is `UNAVAILABLE`, not
a scientific miss. No broad workspace suite or unrelated campaign runs in the
experiment loop.

## Claim boundary

This freeze does not establish prompt capacity, geometry attribution,
attention, coherent generation, reasoning, integer/table lowering,
multiplication-free runtime legality, product readiness, or release readiness.
It defines one falsifiable attempt to put the missing causal context-to-token
association into bounded R4 matrix state while keeping the qualified language
model and strongest prior comparator fixed.

## V1 execution result — 2026-09-01

V1 executed once on CPU with eight PyTorch intra-op and eight inter-op threads.
All `256` updates and all scoring completed in `64.6513` seconds. The exact
create-once result is
`blake3:004abd0ab27e63065c4961863123c8e086ff1b88ea12162de558a0bdaac8dac8`.
The disposable `9,228` fitted values were destroyed and no fitted artifact was
written. V5 remained uncreated and uninspected.

All mechanics passed: causal-prefix, unobserved-future, state-off, replay, and
forbidden-read deltas were zero; every trainable value received a finite
nonzero gradient; the qualified V1 artifact was unchanged; and the largest
all-frame connection/covariance error was below `1.8e-7` against the frozen
`2e-5` ceiling.

| Arm | gain, nats/token | own wins | own NLL | foreign NLL |
|---|---:|---:|---:|---:|
| Full predictive delta | `1.1097723` | `64/64` | `2.6302566` | `3.7400288` |
| State-off qualified V1 | `0.0048169` | `38/64` | `3.5984988` | `3.6033157` |
| Same fitted values, additive intervention | `1.1499870` | `38/64` | `18.4722066` | `19.6221937` |

The frozen V1 verdict is `PREDICTIVE_BINDING_NOT_OBSERVABLE` because the sole
`delta_over_additive` gate missed: full-delta gain minus additive gain was
`-0.0402148`, below `0.0253416`. That verdict is not revised. It also must not
be misreported as absence of native predictive-binding capacity: the native
arm exceeded the absolute gain floor by `25.62x`, won every direction,
improved own NLL over state-off by `0.9682423`, and was state-load-bearing.

The additive comparison exposed a decision-metric pathology. Its slightly
larger relative contrast was the difference between two catastrophically bad
likelihoods: additive own NLL was `15.8419501` worse than full delta and
`14.8737078` worse than state-off. Relative contrast alone therefore cannot
decide whether the ablation remains a usable next-token model. V1 remains a
procedural miss, while its native-capacity observation motivates the versioned
control correction below.

## V2 matched-control correction freeze — 2026-09-01

V2 does not alter `R4PredictiveBlockDeltaBindingV1`, reinterpret V1, create V5,
or inspect any V5 coordinate. It changes only the disposable control decision
and uses the non-overlapping already revealed V4 pair indices `32..63`
(`32` pairs / `64` directions / `1,024` targets). Their ordered identities are
bound before fitting.

Two models start from byte-identical binding values and the same immutable V1:

1. full predictive delta, fitted with the native update; and
2. additive/Hebbian no-overwrite, fitted independently with its own live
   additive update.

Each receives exactly the same optimizer, batch order, eight directions per
batch, and at most `256` updates. Both receive complete gradient, replay,
causality, unchanged-base, and equal-work checks. Both fitted value sets are
destroyed after scoring. The complete V2 gate is CPU8-only, forbids CUDA, and
has one `300`-second wall.

Native capacity is admitted only if all integrity checks pass and:

```text
full gain                         >= ln(2) / 16 = 0.0433216988
full own-vs-cross wins            >= 52 / 64
full gain - state-off gain        >= ln(1.5) / 16 = 0.0253415693
full own NLL                      <= state-off own NLL + 0.05
```

V5 authorization depends only on that native-capacity decision. Delta
attribution is reported separately. The independently fitted additive arm is
language-valid only if its own NLL is no more than `0.05` above state-off. If
valid, full delta has prompt-specific superiority only when its own NLL is no
worse and its gain exceeds additive gain by at least `0.0253415693`. If the
additive arm is not language-valid, V2 records
`ADDITIVE_CONTROL_NO_STABLE_CAPACITY`: delta overwrite is load-bearing for
stability on the disposable slice, but prompt-specific delta superiority
remains unclaimed.

- **Native capacity miss:** reject this unchanged binding mechanism and keep
  V5 closed.
- **Native capacity pass:** authorize exactly one frozen V5 terminal campaign;
  carry the additive verdict as a separate attribution result.
- **Integrity failure or timeout:** `INVALID`/`UNAVAILABLE`; repair or report
  the harness and do not interpret model scores.

This V2 correction is frozen before implementation and before opening its
disjoint V4 slice. It is still only a revealed-data expressivity gate. Even a
pass would not establish held-out prompt capacity, geometry attribution,
general attention, autonomous generation, or reasoning.
