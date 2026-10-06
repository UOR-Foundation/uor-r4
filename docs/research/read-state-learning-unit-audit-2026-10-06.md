# Read-state bridge: coefficient units and learning boundaries

Scope: independent read-only source review of numerical code at
`a5e132b3203d019c7501db7225480567cd20cd9a`, following the completed CUDA
admission. This is source reasoning, not another numerical run or a language
result. No inconsistency was found that justifies changing the matched fit.

## Coefficient units

Continuous masters retain intermediate values; export uses the nearest quarter,
with native integer coefficient `n = round(4m)`. Projection constrains finite
masters to `[-1.75, 1.75]`; it does not round away accumulated updates.

| Operator | Native score contribution | Derivative with respect to quarter-valued master |
| --- | --- | --- |
| Bridge bias/relative field | `n/16` | `1/4` |
| Generate bias/unary/pair selected field | `n/16` times selected feature | `1/4` times feature |
| Potential scalar field | `n/4` | `1` |
| Potential unary/pair field | `n/4` times normalized geometric feature | Geometric feature |

The bridge's native shift is 20 in Q24, and its learning wrapper applies the
corresponding `0.25` to quarter-valued masters. Potential scalar shift 22 and
its normalized unary/bilinear arithmetic produce the larger `n/4` scale;
the coefficient adjoint correctly uses nat-valued masters directly. The
factor-four difference is an implemented operator scale, not a missing gradient
conversion. Copy additionally sums lanes and heads, so numerical correctness
does not imply balanced capacity against Generate.

Source: [native bridge](../../crates/uor-r4-core/src/native_geometric/learner/geometric_read_state_bridge.rs),
[learning bridge](../../crates/uor-r4-training/src/geometric_read_state_bridge.rs),
[native potential](../../crates/uor-r4-integer/src/geometric_potential_q4.rs),
[potential coefficient graph](../../crates/uor-r4-training/src/geometric_potential_q4.rs).

## Common token loss and intentional surrogate limits

The vocabulary marginal converts all Q24 atoms to nats, clips Generate and Copy
scores to `[-8,8]`, and sums probability over every gold-token occurrence alias.
Its forward probability is the canonical native probability; its differential
comes from the F32 softmax graph. Relative to the ordinary softmax alias-loss
gradient, this gives the same positive row factor
`P_soft(gold aliases) / P_native(gold aliases)` to every Generate and Copy atom.
There is no asymmetric factor-four loss conversion.

Emission clipping removes score credit strictly outside the clip interval.
The separate read selector uses unclipped raw Copy scores in nats. Its
distribution can therefore be sharper than the emission distribution, and
selector credit is invariant to a common shift. These properties can matter
when diagnosing Copy saturation. They are not newly discovered implementation
defects. Bridge and selector credit remain local first-order surrogates; neither
connected gradients nor correct units prove improvement of finite native action
choices or the exact native objective.

Source: [vocabulary marginal loss](../../crates/uor-r4-training/src/geometric_generate_learning.rs),
[selector contrasts](../../crates/uor-r4-training/src/geometric_bank_generate.rs),
[quarter STE](../../crates/uor-r4-training/src/geometric_context.rs),
[CUDA quantizer](../../crates/uor-r4-training/src/native_geometric_cuda_kernels.rs).

## Conditional frame structure

For a fixed selected occurrence, the bridge has the form `F(q,k) = q h(q^-1 k)`.
For a common signed-H4 left-frame change `g`, the group identities give
`(gq)^-1(gk) = q^-1 k` and therefore `F(gq,gk) = g F(q,k)`. Per-lane relative
tables see the unchanged relative element, so their scores and chosen action
are unchanged. Identity-first ties preserve this conditional property.

This algebraic conclusion assumes the retained exact H4 group operations and
fixed selected occurrence. It does not establish equivariance of the full
selector, context encoder or Generate decoder with fixed token prototypes,
and does not establish predictive advantage. It explains the bridge's reuse
across common frames rather than introducing an arbitrary absolute-state map.

## Next decision

Proceed with the prepared matched joint fit from the feasible initial parent.
Keep the existing rates and objective; inspect exported coefficient changes,
observed discrete actions, source choices and actual replies. A future negative
must distinguish native discretization, read ranking, transport utility and
emission competition rather than treating source-unit speculation as measured
mechanism failure. Work card: #820 issuecomment-6024955480.

Inspect the final trained checkpoint as well as the selected checkpoint, so
baseline selection cannot hide the trained candidate's changes. Join saved
rows by case identity and retain regressions. Entry correctness, complete
own-prefix replies, EOS and first-error phase precede aggregate likelihood.
Separate Copy-covered from Generate-only target positions using the actual
native candidate inventory after forward. Group opposite questions by their
authenticated physical bank; repeated questions are not independent banks.

Nonidentity actions alone can be a learned constant rotation. If actual native
crossings make attribution consequential, use a fixed-checkpoint mediator
contrast: factual bridge, identity bridge, and learned bridge with the highest
raw-scoring occurrence from a different physical Source (earliest ties).
Choose the alternate without consulting labels. Hold query state, Copy atoms
and scores, aliases and Generate parameters fixed; score labels only afterward.
This contrasts use of the source-state channel with decoder adaptation or
uniform Copy suppression. The key is an accumulated contextual state, so key
sensitivity does not establish exclusive encoding of the selected record.
This conditional diagnostic is not a contextual fact-swap or chat result.

If useful source-channel consumption is established, the next substantive
grounding test changes literal facts while preserving question, source roles,
order and public token budget, with independently authored expected answers.
If output remains blocked by Copy scores above Generate's attainable bound,
retain the bridge result and address read-versus-emission competition; do not
repeat the same fit or retire geometry from that negative.
