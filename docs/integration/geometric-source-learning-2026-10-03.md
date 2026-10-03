# Geometric source learning and output support — October 3

The geometric occurrence consumer now reuses its immutable prepared q4 context
within one batch/update, instead of reconstructing all 8,963,136 coefficients
and expanded tables for every answer token. The prepared object must be dropped
before optimizer mutation. Protocol-2 labels use the actual leading content
space and include EOS. The Rust driver measures complete shortest/longest B8
answers, retains hard-payload changes, independently reloads checkpoints, and
uses its own generated prefixes during fit evaluation.

## Measured cost and the representation blocker

At source `8afe768054cfcc409b8f25f17e88f1a9563166e9`, the zero-update cost witness
runs 32 and 58 teacher-forced response tokens in two B8 batches. Context,
potential and NoRead all receive finite nonzero credit. Packed native payloads
remain unchanged and independent source/native reload preserves traces.
Driver time is 23.954s; process real time is 24.98s and monitored wrapper elapsed
is 25.760s. Maximum RSS is 590,348,288B; peak memory footprint is 942,736,584B,
a different OS measure. This is development cost with a frozen floating parent,
not optimized serving throughput or fitted language.

The all20 source-boundary audit, helper source `6033186b` with linked production
library `8afe7680`, executes 114 frozen-parent inferences and zero updates.
All20 canonical first target tokens are absent from the raw source token pool.
For example, raw `singer` is `[85,300,284]`, whereas the literal-role reply starts
with `[1130,284]` representing ` sing` and `er`. The raw source bytes are correct;
the response boundary changes BPE segmentation.

For an unsupported token y the current hybrid has
`P(y) = beta * P_parent(y)`. Every higher-probability parent token stays higher:
a source competitor also receives nonnegative copy mass. Setting beta to zero
leaves y at zero. At these fixed teacher-forced prefixes, changing occurrence
weights or NoRead cannot make y win. Unsupported-target CE changes NoRead mass,
not value-specific copy support. This statement concerns the canonical token
sequence, not every alternative tokenization, accepted rendering or sample.

A source-only view `encode(" " + exact_source_bytes)` covers all20 first labels.
However, the period is unsupported and outranked at all20 canonical value-complete
prefixes: parent top is ` is` in19 and ` sounds` in1. EOS is parent rank1 after
the period, which does not prove stopping under the copy mixture. Across114
labels, raw source lacks60 targets and has40 ordering-blocked positions;
the lexical view lacks40 and leaves20 ordering-blocked positions. A64-update
hybrid fit is therefore NOT_ADMITTED; no optimizer update was run.

## Label-free lexical repair

`geometric_source_emission_view` compiles exactly one literal ASCII space plus
the original source bytes with the bound byte-BPE tokenizer. It never trims,
normalizes or appends answer punctuation/EOS. Original token IDs and bytes remain
unchanged. Each derived token records half-open rendered/source byte intervals,
original overlapping-token intervals, and whether it includes the separator.
Merged/split tokens and partial UTF8 pieces are supported without replacing bytes.
The derived reader frame preserves supplied record/commit and typed metadata,
and rejects different original token IDs. View offsets are explicitly distinct
from original occurrence offsets.

The numeric reader remains the same q4 H4 context/potential/NoRead mechanism.
Artifact schema /2 binds the lexical policy and tokenizer; /1 remains the raw
source format. Reader entry points reject crossing the two representations.
The adapter and trace wrapper allocate outside the numerical kernel. No
allocation-free or complete native serving claim follows from this repair.

Validation of the changed source-view path is pending. The old cost and audit
are retained independently; they are not reported as tests of the repaired path.

## Next geometric output mechanism

The next bounded candidate removes the frozen parent from source realization:
learn geometric scores for Copy occurrences, EmitPeriod and Stop, sum head Q24
logits before one integer normalization, then sum all action masses yielding the
same token. Punctuation/EOS identities come from tokenizer/protocol, never answer
labels. All actions remain available at every structurally legal prefix; no
scripted copy cursor, forced period or forced stop is adopted.

The integer pool alone is arithmetic scaffolding. Learned action projections,
ordinary complete-answer credit, artifact reload and own-prefix generation must
still be implemented and measured. Ordering/progress collisions, repeated-token
mass and ambiguous same-token action provenance remain explicit risks. This
scope is bounded selected-source realization; free prose, NotFound replies,
coding/reasoning and general conversation are unfinished.

Transformer superiority and an ordinary-reader win remain later attribution
questions, not first-fit gates. Preserve accepted parents and historical negative
rows. Do not repeat an unchanged hybrid dose or retire geometry from this support
failure. Keep `stack_grounded_session.rs` untouched pending #1552 coordination.

## Retained evidence

Local roots under `~/uor-r4-local/workspace/research/geometric-source-learning-20261003`:
`cost-1` and `boundary-audit-1`, each exclusively claimed, sealed and verified by
the Rust driver. Source-view validation will use a new independent root.

Cost executable SHA256:
`8609afc6aa0e29b6d1e96b72825f034581645c34f0983f8dcaeb4b38bac48791`.
Boundary executable SHA256:
`52e76ab55b2396570278062da86a0deec3e840b3f8ce4bbe592c1b459cd791b0`.
Audit helper and linked library source identities are deliberately separate.
All20 cases are exposed development, not a final held-out evaluation.
