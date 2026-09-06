# First learned H4 routing block — #1139

This record follows the owner-directed geo-transformer plan delivered in #1146.
It concerns one optional residual block in the existing native predictor. It
does not replace the entire predictor or qualify a geo-transformer LLM.

## Mechanism and retained information

Each of two independent channels learns query, key and value codes for tokens
in the existing artifact vocabulary. Codes identify full signed elements of the
120-element binary icosahedral group. The channel composes the last two query
codes in their observed order. It examines at most eight recent source keys,
selects one source, gathers that source's value code and applies a learned
query-conditioned group action. All group products read the existing exact H4
product table. Sparse output rows contribute quantized residual token scores
and at most eight candidate postings per channel to `Model::predict`.

Angular selection ranks the signed real component of `query * inverse(key)`.
Host construction orders the exact `Z[phi]` coefficients and emits a finite
rank table. Serving reads that table; it computes no cosine, floating-point
distance, multiplication or dense projection. Ties prefer the most recent
source. The full selected key/value and transported output retain orientation;
the scalar angular comparison alone does not distinguish every orientation.
The exact-code comparator selects code equality, with the same code capacity,
context limit, transport and readout. It is a selector comparison, not a wholly
non-geometric model. The fixed-placement arm learns actions and emission rows
while retaining the initialized token codes.

Token identity remains the parent's lexical/byte codec and prime-addressed
vocabulary. The new codebook does not replace canonical identity with a semantic
hash. Query/key/value placement is learned from next-token loss. The fixed zeta
phases, ordered base features, paired-H4/icosian witnesses, exact relation store,
typed arithmetic and committed copy path remain in their existing roles; this
block does not establish a new learned zeta or paired-H4 scoring advantage.
Committed exact copying still bypasses ordinary prediction work.

The eight-token window retains exact source token IDs until selection. Each
channel then discards all unselected payloads. Identical token codes, angular
ties and finite group products can collide. The two-token query is compressed
to 120 states, and each selected/transported output is also one of 120 states.
The readout sums independent channel scores, without a joint relation between
the channels. Source offset is diagnostic; it is not an extra learned feature.
The block does not encode older context, source revisions, or a persistent
derived-value chain. Existing bounded memory remains separate.

## Learning and comparison

Rust fitting uses hard coordinate proposals evaluated by the actual serving
selector and transport. For each channel, it adjusts query/key/value codes and
query-conditioned actions, refitting route-conditional token counts after each
proposal. The accepted objective is smoothed conditional next-token NLL for
that channel. It is not end-to-end optimization of the parent's complete
residual score or a soft attention surrogate. Quantized sparse emission scores
are compiled from the final counts. No dense training library is needed here.

The bounded example uses 24 authored raw-text documents, equally split between
prose and Rust, and eight distinct development documents. It samples up to
2,048 fit positions across complete documents, adjusts the 24 most frequent
context token codes, and makes two passes with three proposals per coordinate.
Each fit has a 15-second safety ceiling. Three arms share these limits and the
initialization seed; a stopped fit must not be described as a completed matched
search. Development text and generated prompts are OPEN, synthetic and small.
They are not broad natural-language or sealed capability qualification.

The unchanged parent and three fitted arms receive the same next-token and
24-token generation checks. The angular artifact also runs with the block
disabled, source selection forced to the most recent token, and the learned
action replaced by identity. Selection-disabled still calculates all source
ranks. Source hashes, fit reports, exact outputs, route examples and complete
native work counters are retained. Artifact reload and session restoration are
checked separately. No teacher or provider authors serving output.

The routing block is fitted last and binds the exact parent artifact CID.
Loading validates inherited response components against that unchanged parent,
including their original nested identity checks, then validates the complete
new artifact. Replacing or training a nested component under an already fitted
routing block is not supported by this version. The first execution exposed a
missing distinction between the new outer artifact and those older frozen
parents: its reload failed after the first arm. Those diagnostic outputs and
the failed 4.124-second command remain preserved. The correction adds explicit
parent binding without changing learned route/scoring equations or weakening
nested provenance checks. Final results must come from the corrected executable.

## Cost boundary

For vocabulary size V and up to W=8 sources, the new codebooks contain 6V u16
entries, two 120-entry action tables and one 120-entry angular-rank table.
Emission storage depends on observed token/root pairs; output postings are
bounded by 2 * 120 * 8 token IDs. Existing shared H4 tables are reused. There
is no online training, new persistent per-token cache, or vocabulary-wide scan.

Each active prediction builds two ordered queries, examines at most 16 keys,
gathers exactly two value codes and executes two query-conditioned actions.
Each angular source requires an inverse, group-product and rank lookup. Each
selected payload needs two further group products. Candidate scoring adds two
bounded sparse-row searches per candidate; admission adds at most 16 postings.
Thus selection/transport is O(W), and residual readout is O(A log V) for the
number A of candidate evaluations during finite posting admission. A is not
the final shortlist size: a dropped candidate can be offered and scored again.
The existing feature, memory, write and
decoding work remains and must be included in whole-path timing.

`RoutingWork` reports code/table reads, comparisons, examined sources, payload
gathers, executed actions, emission queries and logical operand bytes. These
are algorithmic counters, not a complete CPU instruction or physical DRAM
traffic census. The report retains the surrounding native counters as well.
Host tokenization, artifact loading/validation and sparse-table construction
remain outside the allocation-free prediction kernel. Existing startup geometry
validation still uses floating point; whole-lifecycle integer-only execution
is not claimed.

The complete pre-execution projection is retained in
`.uor-models/native-typed-value-2026-09-05/learned-routing-projection.json`:
950 seconds engineering and 105 seconds model work, within cycle ceilings of
1,200 and 120 seconds. The cumulative ledger starts at 1,363.884/1,800 seconds.
Projected new storage is 288 MiB with approximately 307 MiB before the existing
tighter stop. The 4 GiB sampled child-RSS target and one model process remain.
Necessary storage increases are preauthorized; no material is deleted.

## Executed result and decision

**Keep this block as development code and artifacts. Retain parent `067adbf0`
for the accepted bounded behavior.** The complete comparison executes from the
corrected release binary. All three fits complete their configured passes;
angular and equality each evaluate 2,304 proposals, fixed placement 1,440.
Actual sampled fit positions are 1,890 from 24 documents. Angular accepts
32 query-code, 25 key-code, 28 value-code and 198 action changes across the two
channels. Its per-channel fit NLL changes from 4.756/4.740 to 4.465/4.402.
Only the 24 most frequent context token codes are eligible for adjustment in
this first fit; the vocabulary contains 611 token IDs.

| Arm | Prose next-token correct | Rust next-token correct | Total | Target continuations |
|---|---:|---:|---:|---:|
| Unchanged parent | 6/435 | 39/300 | 45/735 | 0/8 |
| Learned angular | 103/435 | 79/300 | 182/735 | 0/8 |
| Learned exact-code selection | 127/435 | 96/300 | 223/735 | 0/8 |
| Fixed placement, fitted actions/readout | 103/435 | 74/300 | 177/735 | 0/8 |
| Angular, source selection disabled | 43/435 | 42/300 | 85/735 | 0/8 |
| Angular, learned action disabled | 66/435 | 53/300 | 119/735 | 0/8 |

Disabling the whole new block reproduces the parent's 45/735 result and eight
generated byte sequences. The new representation/selection/action combination
helps this token-prediction population; the learned angular placement increment
over fixed placement is only five decisions. Exact-code selection performs
better under the matched placement-search budget. No angular superiority,
general semantic codebook or generation qualification follows. Prose contains
383 byte-fallback tokens and only 48 lexical tokens; Rust contains 176 and 120,
respectively, plus four EOS targets per family. These are substantially
byte-level results on authored synthetic text.

Both fitted selectors preserve only **38/62** previously correct responses.
The old parent reproduces all **62/62 complete Generation objects**, including
work and state, on the changed executable. On 28 earlier longer-context relation
cases, both candidates retain all 28 exact write sequences and all 28 restored
relation states. Angular returns 16/28 exact answers, equality 28/28. The
angular failures include correct `Unknown` prefixes followed by unwanted text.
This is output/termination regression despite correct retained facts.

The actual CLI matches angular's probe output for the Rust continuation:
` Unknown.\nss  (r (leftl a (r (l`. Disabling the existing response-entry
head removes the forced prefix but yields ` a)rt: i32] n) )c ))l3e i32eo i32`,
still incoherent. The parent under that same control produces
`\n    let shifted =);\n}\n`. Inspection and the trace explain the prefix:
the older entry head adds its positive margin above the base candidate's score.
Separately, residual language scores disrupt the old stopping behavior. The
block's per-channel conditional objective does not optimize either full-path
competition or final decoded continuation.

**Next within #1139:** train response dispatch and EOS together with the routed
predictor against final output, including ordinary prose/Rust continuations and
the preserved memory cases. Keep the selector comparison. Establish a useful
continuation and preserved termination before increasing context, adding more
channels or introducing another abstraction stage. The immediate problem is
not missing cache capacity. #1139 stays open; #1140 remains subsequent. No
larger language or frontier claim is admitted by this checkpoint.

## Measured costs, artifacts and verification

The angular artifact is `09f9991cf769e3008d78a487c5115502f06cbca681577a81278401714a8edc2b`
(10,807,631 bytes, 79,348 bytes above parent). Equality is
`c6d3739db83e60f4e0f5d1ed25e261fb205fbca784fbf2bdfc3981aaebea85ad`
(10,803,043 bytes); fixed placement is
`f7702f024ad505b2e1c08931907e94d49942aff4f17ec50d5e009087183e15e5`
(10,812,953 bytes). Each has the same 8,052 bytes of u16 token/action/rank
entries before container metadata. Angular adds 1,698 sparse emission scores
and 1,268 postings; equality has 1,538/1,224, fixed placement 1,872/1,407.
Prediction adds fixed inline decision/counter state and a 32-byte stack window;
the focused allocation census measures zero heap allocations and zero bytes
through repeated observation, selection and transformation.

The 735 angular development positions examine 11,312 source keys, gather
1,470 payloads and execute 1,470 actions. Residual scoring performs 284,066
emission queries across 142,033 candidate evaluations, with 1,321,154 new
comparisons and 7,756,832 logical operand bytes. The surrounding base still
performs 3,364,474 feature-score lookups. Selected transport is a small part of
this complete path; scoring candidates repeatedly remains substantial work.

Single-pass next-token timing (encoding, observation and full prediction) is
87.881 ms parent, 102.005 ms angular, 101.193 ms equality and 101.079 ms fixed
placement. These are diagnostics, not repeated speed measurements. Parent file
read/deserialization/validation is 2,497.554 ms; new in-memory artifact
deserialization/validation is 2,830.333–2,845.592 ms. These startup boundaries
differ by file reading and are stated separately. Fits take 531, 470 and
471 ms. The full comparison, including three fitted artifacts and their reloads,
takes 13.546 seconds under the monitor. Its peak sampled child RSS is
834,306,048 bytes; this includes fitting and validation clones, not a minimal
single-session serving footprint.

Four focused learned-routing unit tests, the new allocation test and the
integer-kernel source scan pass. They cover hard source choice, query action,
code bounds, nested parent identity refusal, fitting-data overlap refusal,
counter aggregation/saturation, disabled-parent preservation, artifact reload,
retained tail and session checkpoint replay. The architecture-policy check
passes. The changed release CLI and both examples compile and execute. New
generated Rust is not compiled or executed as semantic evidence; it has already
failed the target-continuation decision. Existing checked Rust evidence keeps
its original scope.

The [compact evidence](evidence/native_geometric_learned_routing_1139.json)
binds the complete report, exact outputs and controls. Local receipts remain
under `.uor-models/native-typed-value-2026-09-05/`: the initial failed
`learned-routing-comparison/`, corrected `learned-routing-corrected-comparison/`,
three `learned-routing-preserve-*` directories, two relation reports, three CLI
logs and command ledgers. The correction preserves every learned parameter and
configuration from the first angular fit, adding only parent provenance.

Model work including the failed attempt and focused executions is
**46.768/120 seconds** for this cycle. The cumulative ledger is
**1,410.652/1,800 seconds**, leaving **389.348 seconds**. Engineering through
the corrected build and architecture check is 862.744/1,200 seconds; final
formatting/claim checks and delivery receipts are recorded separately. Storage
after evaluation is 5,852,442,624 bytes against the unchanged 6,459,228,160 cap,
with 251,379,712 bytes before the tighter existing stop. No storage increase,
deletion, external model or paid compute was needed. Samples are not exact
transient peaks or unique-extent measurements.
