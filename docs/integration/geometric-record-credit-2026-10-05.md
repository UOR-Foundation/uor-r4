# Geometric record credit and the attention-to-emission boundary — October 5

## Implemented intervention

The supported current-bank reader now has an opt-in offline record-ranking loss
alongside ordinary full-answer/EOS token supervision. Both current records reach
the unchanged native bank/cue/prefix/SourceEnd inference path. Labels enter only
after the complete native read; no supplied selected record, gold admission mask,
runtime role label or provider response is introduced.

The auxiliary objective groups Copy candidates by exact source segment and
record/commit identity, using the maximum native joint Copy Q24 score per record.
Duplicate aliases therefore do not accumulate record probability merely by being
duplicated. Different record lengths can still affect the maximum's distribution;
this is not a demonstrated length-neutral ranking rule. The existing root-score
surrogate supplies offline credit. Native checkpoint selection remains the
baseline-inclusive earliest strict minimum of full-answer/EOS token-alias CE.
Record labels supervise payload positions only; Period/EOS have no record target.

Only token/self/neighbor observation roots learn. Recurrence, categories, cue,
prefix and endpoint tables remain frozen. No serving operator changes. The
combined objective has finite nonzero gradients in all three admitted root
families; this does not isolate useful record-only gradient contribution.

## Fixed conditions and completed outcomes

Numerical source: `dea30c2a659c6db72cbfcb6e25eed0358610817f`; release executable
SHA256: `49d2e8f7fc9995cbed0b389eec50e4a6cfd53528eb54274f7c7f9cd47eeea132`.
The [supported reader study](geometric-supported-reader-transfer-2026-10-05.md)
owns panel identities, answerability, exposure and the token-only controls.
Development128 and diagnostic32 are unchanged. The diagnostic was already exposed
before this intervention; it is not a new held-out transfer test.

All three orders share one learned initialization. Each completes 64 balanced B8
updates, four visits per episode, learning rate0.003, with exported and independently
reloaded native checkpoints0/16/32/48/64. The episode-order seeds do not constitute
three independent initializations or chat lineages. All select checkpoint64 by
the fixed development criterion; frozen parameter bits remain equal.

| Arm/order | Development CE | First source /128 | Both queries /64 | Complete /128 | Diagnostic source /32 | Both queries /16 | Complete /32 |
|---|---:|---:|---:|---:|---:|---:|---:|
| Shared parent | 1.138694423 | 57 | 13 | 10 | 17 | 6 | 1 |
| Token-only061 | 1.050793672 | 76 | 23 | 12 | 15 | 5 | 2 |
| Record061 | 1.045332684 | 80 | 24 | 11 | 15 | 4 | 1 |
| Token-only062 | 1.044903199 | 73 | 18 | 11 | 21 | 8 | 3 |
| Record062 | 1.061998721 | 79 | 21 | 11 | 17 | 3 | 2 |
| Token-only063 | 1.063494227 | 78 | 19 | 6 | 15 | 4 | 1 |
| Record063 | 1.059553592 | 75 | 22 | 10 | 16 | 5 | 3 |

Development source selection improves in every record-credit order against the
shared parent. Against matched token-only orders it changes by +4,+6,-3; complete
answers change by -1,0,+4. On the exposed diagnostic, matched source changes are
0,-4,+1 and complete-answer changes are -1,-1,+2. There is no dominance claim,
reliable transfer result or general conversation qualification. Preserve all six
candidates rather than selecting an order by diagnostic performance.

Saved case-by-case comparisons cover every new candidate against the parent and
all three token-only candidates, plus the other new candidates. For matched
development orders, source gains/losses are18/14,23/17,19/22 and complete-answer
gains/losses are6/7,7/7,6/2. Aggregate gains hide losses on previously successful
rows. Initial factual selection is distinct from token-alias provenance and
complete generated response behavior.

## Discriminator: fixed-source terminal competition

Saved-generation arithmetic/provenance audits verify exact record/commit/event
bindings, actual generated prefixes, Copy ordinals and native earliest-maximum
selection, all-token alias conservation, target payload/suffix and answer membership.
They are saved-JSON checks, separate from the fitter's native export/reload checks.

On development128, parent→record061 has381 matched-prefix/same-source steps and
token-only061→record061 has524. Every such step retains identical terminal raw
scores. Source inspection agrees: observation roots change Copy observations;
the frozen latent transitions and selected-source endpoint supply terminal state.
At fixed prefix/source, a missing direct root-to-terminal derivative is not the
cause. Root changes can still change Copy competition, factual-source selection,
or subsequent generated prefixes.

At target Period/EOS steps with fixed prefix/source, token-only061→record061
regresses target probability at31 steps and improves it at11 despite identical
terminal raw scores. Development first errors classified as terminal/length
competition number71,76,65 in the new orders. These descriptive categories do
not establish independent causes and can include wrong-source cases.

In reached correct-prefix/correct-source reads, the full eight-lane angular and
relative-root signatures have no payload-versus-terminal collision. All reached
true Period boundaries have response latent states equal to the selected endpoint;
none of these payload reads does. This demonstrates available phase information
in those tuples, not additive per-lane Q4 expressivity, coverage of unreached states,
or a reason to immediately add latent coordinates.

## Next causal intervention

Retain all reader artifacts. Use record061 checkpoint64 as the prospective frozen
reader for endpoint calibration: it has the lowest development CE among these
record-credit candidates; diagnostic scores do not select it. It is not declared
the best complete-response model or superior to all token-only controls.

Reuse the existing SourceEnd loss to fit both Period and Stop tables while freezing
all source parameters, cue/prefix operators and Copy scoring. The old learner's
zero initialization and legacy panel checks require an explicit opt-in natural-panel
warm-start path. Initialize shadows from this selected reader's actual native Q4
payloads with freshly bound metadata; preserve historical fractional optimizer
state as history, not a purported resume. Require byte-identical payload roundtrip
and complete native baseline fidelity before updates. No historical metadata
transplant, new serving operator or root-derivative repair is proposed.

Complete-answer improvement with fixed initial source selection supports endpoint
calibration. Relaxed-loss improvement without native Q4 improvement directs the
next investigation to discretization or bounded additive expressivity. Improved
correct-prefix terminals without own-prefix completion isolates a trajectory
problem. None of these outcomes qualifies raw-history compiler integration or chat.

## Cost, artifacts and delivery scope

Runpod Linux x86_64,48 CPU threads, no GPU. Completed fit worker times are
798.967,773.071,712.951 seconds (2,284.989 seconds total); maximum sampled process
tree RSS is about912MB. Zero-update admission takes151.370 seconds; the executed
record-credit scoped test/build worker takes449.331 seconds. These are measured
components, not a claim of total preparation/review/delivery duration. The whole
work card precharges110 minutes cumulatively, including a recorded20-minute
extension; its complete resource envelope includes8GiB RAM,2GiB new storage,
10GiB owned pod storage and the free-space stop margin. No new paid resource.

Four record-ranking controls, the actual-trace terminal-credit fixture and all27
observation-example tests pass at numerical headdea30c2a; the release example
builds. Final integrated-head delivery checks must be recorded on the protected PR.
Linux cannot run Metal-only tests; they are unavailable and not counted. No laptop
energy, complete D11 serving, learned raw-history compiler or general prose claim.

Full checkpoints, reports, receipts, configurations and saved audits are retained
under `/root/codex/results/geometric-supported-reader-transfer-20261005/` and the
owned `/workspace/codex-uor-r4-20261004/` archive. Small reports and audit sources
are returned to `~/uor-r4-local/workspace/research/geometric-supported-reader-transfer-20261005`.
Archive/off-host verification receipts and current delivery status are published on
[#820](https://github.com/UOR-Foundation/uor-r4/issues/820) and
[#1552](https://github.com/UOR-Foundation/uor-r4/issues/1552).
