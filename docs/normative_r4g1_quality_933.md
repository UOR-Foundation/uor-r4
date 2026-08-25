# Normative R4G1 serving and deployed-quality reconciliation (#933)

- **Parent / stage:** #821, S0 truth and inference closure.
- **Status:** implementation and evidence pending; RF-31 **NOT ESTABLISHED** at
  normative deployed-serving scope.
- **Normative authority:** ADR-0001. `R4G1Runtime` is the sole production
  candidate/token selector; `R4Engine` / `GraphScorer` may resolve tokenless D4
  policy and serve as an explicitly named reference/certifier only.
- **Execution scope:** normative runtime, production decode adapters, offline
  compiler/certifier report construction, and strict production admission.
- **Evidence policy:** append-only. This record preserves historical results and
  records the correction and eventual #933 outcome separately.

## Opening truth correction

The 2026-08-24 call-graph and artifact audit established four distinct facts:

1. **#821:** the 2026-08-20 S0 promotion remains historical, but its strict
   normative serving premise was not established. #821 is reopened for this
   foundational child.
2. **#833:** source/corpus provenance, #755 ordering, deterministic compiler
   outputs, and package attestation remain established. The `r4 ask` canary used
   `load_accepting_quality`, and Gate C scored certifier rows; neither proves
   strict production admission or the exact `R4G1Runtime` served token.
3. **Empirical Criterion. Status: Empirical.** **#908 execution scope:
   reference/off-serving.** 21,424 / 72,130 = **29.702%** and the paired
   **+28.45‰ [25.57, 31.32]** effect remain valid teacher-free evidence for
   the exact `R4Engine` skip-mix harness.
4. **#910:** SKMX/PSIB compiler emission and the `R4Engine` reroute landed, but
   `R4G1Runtime` did not consume the sections; local chat, API/server, sampled,
   greedy, beam, and WASM surfaces did not share one normative candidate owner.
   RF-31 registration therefore overclaimed `normative-runtime` /
   `deployed-serving` reachability.

**Empirical Criterion. Status: Empirical. Execution scope:
reference/off-serving.** The current canonical `smollm2-360m-broad-clean`
graph predates #910 and has no active SKMX/PSIB lane. Its historical certifier
rows—Rule 1+2 **24.393%** and same-position TLA **28.121%**—are comparators,
not a normative serving verdict.

## Required implementation and falsifiers

**Definition.** The #933 normative production selector is one adapter around
`R4G1Runtime::predict_served_candidates`: D4 may permit or decline but cannot
provide a token. The runtime must consume bounded SKMX/PSIB with fixed-point,
saturating, deterministic ordering; preserve exact absent-section candidate,
token, status, and witness identity; and attribute a planted lane promotion.
Promotion witnesses separately record whether nonzero SKMX primary entries,
PSIB fallback entries, or both contributed to the selected candidate.
Greedy, pinned-seed sampled, and beam decode must consume the same ranked list.

**Definition.** A #933 deployed-quality report is a versioned record that binds
the exact graph, teacher artifact, corpus, partition manifest, tokenizer,
compiler/configuration,
active-section set, decode mode/seed, selector identity, execution scope,
population/sample status, paired counts, comparator definition, witness replay,
negative controls, and the exact internal sections-absent identity census.
Production requires one zero-mismatch internal control check per evaluated
position; external parity rows cannot substitute for that census. Production
admission rejects missing, legacy,
wrong-selector, sampled-as-census, unknown-schema, or identity-mismatched reports
with typed errors. Explicit research loading remains available with a warning
and is never reported as production admission.

Binding falsifiers: absent sections change any normative output; a planted SKMX
or PSIB partner cannot reach the winner; a content token outside the newest
eight-token compiler window affects bounded work; sampled decode uses a
base/reference candidate list; D4
substitutes a token; any production surface diverges; corrupt sections or report
identity mismatches do not fail closed; a label-shuffled lane has a positive
promotion-class effect; or a required canonical fixture is missing. Missing
evidence is UNAVAILABLE, never PASS.

## Mechanical surface reachability

The wrapper call graph is covered by executable behavior tests, separately
from the canonical cross-surface evidence artifact:

- local CLI sampled decode composes D4 and `R4G1Runtime` at every emitted step;
  greedy beam replays the same adapter for every bounded hypothesis and commits
  D4 only along the winning path before any token reaches chat history;
- socket-level tests drive `/api/chat`, `/v1/chat/completions`,
  `/v1/responses`, `/api/r4g1/predict`, and `/api/r4g1/generate`, including
  both greedy and default sampled request policies, and require the shared
  D4 predict/serve counters to advance on a context with runtime candidates;
- the public graph response functions use one function body on native and
  WASM (`wasm_bindgen` is a target attribute), so native behavior tests execute
  the exact exported production facade. A legacy lane-absent install fails
  closed there and remains available only through the explicitly named,
  warning-bearing research replay function.

The beam planted negative is stronger than a first-step source inspection: its
initial context has runtime candidates, every one-token hypothesis is rejected
by D4, and the test requires a typed zero-token abstention. The HTTP tests use a
small injected fixture to test routing mechanics, not canonical bundle quality;
they deliberately do not create parity rows or authorize production. The
content-bound canonical parity artifact and deployed-quality census remain the
separate decision evidence described below.

The version-4 canonical parity artifact has eight rows in four exact-input
cohorts: stateless direct/state greedy, stateless direct/state sampled,
session-bound direct/CLI beam-first, and session-bound direct/CLI sampled. Each
mechanically executed adapter exports its actual fixed-capacity ranked
shortlist; the artifact derives separate authoritative and observed CIDs and
requires candidate equality as well as disposition/token equality. A planted
alternate graph adds only a lower-ranked candidate while retaining the winner,
and must be recorded as a mismatch. Deterministic mismatch rows are written
before the command returns `STOP`, with a content-bound terminal artifact. Its
ranked-candidate CIDs include the fixed SKMX/PSIB contribution flags, so source
attribution cannot drift while token/score tuples remain unchanged. A negative
run therefore remains inspectable and production admission still rejects it.

The version-2 normative witness-replay artifact carries the same SKMX/PSIB
provenance on its selected runtime candidate and independently replays those
flags beside token, score, source, and lane attribution. Version-1 witness
bytes cannot inherit this provenance credit and are rejected by current
production packaging.

Production serving construction requires the opaque
`VerifiedProductionEnvelope` capability returned by the complete schema-2
verifier. Engine graph, signature artifact, tokenizer, score report, and
deployed-quality bytes are re-hashed against that capability at construction.
The named production policy loader returns only an opaque token-free D4 policy;
the public facade never returns the independently token-selecting reference
`R4Engine`. The root signature-only diagnostic is correspondingly named
`predict_signature_status_for_research`; production-looking window and
generation methods always compose with `R4G1Runtime`.

## Teacher-free decision contract

**Empirical Criterion. Status: Empirical.** The declared population is the
canonical 72,130-position held-out partition and the deterministic cheap
instrument is a label-free, story-round-robin midpoint ordering whose first
6,000 positions are nested within its first 18,000. The measured quantities are
paired top-1 counts against the corpus-recorded labels, their declared 95%
intervals, cross-surface mismatch count, negative-control effect, and
witness-replay failure count. These measurements authorize exactly one of the
outcomes below; they are not a structural guarantee.

The first instrument is the deterministic 6,000-position nested prefix. A
typed `INCONCLUSIVE:` extends that exact order to at most 18,000 non-census
positions. A full 72,130-position census runs only if the sample can change the
decision and all reachability, cross-surface, binding, absent-section, and
planted-negative checks have teeth. No teacher forward pass, observation run,
source download, or hours-class parity run is authorized.

The executable command enforces this sequencing rather than relying on operator
memory: requesting full mode runs the content-bound 6,000-position sample
first. `PROCEED:` reaches the census; `STOP:` returns before census launch; and
`INCONCLUSIVE:` runs the single predeclared extension. After that extension,
an overlapping interval can reach only the census, and only when its
reachability ceiling remains sufficient. Raw cross-surface and witness
artifacts replay the exact tokenizer and score-report bytes; missing or foreign
bytes are planted negative controls. Canonical runs use the resource wrapper
to record host identity, workers, elapsed time, peak RSS, storage, output sizes,
and exit state. A create-once five-second stream also records process-tree
CPU/RSS/process count and host/disk headroom while the evaluator reports
counters, rate, and ETA; opaque phases say `UNAVAILABLE` rather than fabricating
completion. A sample-derived full-population projection beyond the configured
budget or one hour also stops before launch and requires a revised contract.
Before any preflight that can be hosted by a valid real bundle directory, the
command syncs a create-once, non-semantic invocation journal. Its terminal row
records completion, failure, or best-effort interruption; a `started` row with
no terminal row is unresolved rather than `PASS`. The journal is excluded from
generation identity, admission, and release archives.

- **RATIFY:** cross-surface mismatches = 0; paired TLA lower bound ≥ 0;
  paired sections-present minus sections-absent lower bound ≥ +20‰; shuffled
  control not positive; absent identity exact; witness failures = 0; strict
  admission accepts only the correctly bound full census.
- **LIMIT / RETIRE:** reachable normative lane fails a frozen quality gate;
  retain the evidence and do not activate RF-31.
- **NOT ESTABLISHED:** reachability or binding falsifier fires; stop before the
  census when the decision is already forced.
- **UNAVAILABLE:** a required canonical identity or fixture is absent; do not
  substitute another artifact, partition, scorer, or sample.

## Pending evidence ledger

The final append must record graph/report/manifest/result CIDs; full paired
counts and intervals; sample/census status; every falsifier; deterministic
double-build identity; strict empty-root admission; cross-surface results; wall
time, peak RSS, storage, workers, throughput, counters, and progress evidence;
and the exact RATIFY/LIMIT/RETIRE/NOT-ESTABLISHED/UNAVAILABLE outcome. Until that
append exists, this document is a correction and execution contract, not a
positive quality claim.

## Parked implementation checkpoint — 2026-08-24

**Claim status: no empirical verdict.** Work was deliberately parked before a
canonical graph build or deployed-quality evaluation. This checkpoint is not a
RATIFY, LIMIT, RETIRE, NOT ESTABLISHED outcome, or UNAVAILABLE outcome; RF-31's
empirical deployed-quality claim remains not established and production
admission remains fail-closed.

Implemented on `issue-933-normative-r4g1-quality` from base
`9aeb5427c7fd3a965b0e4a0228f137889dbafa9b`:

- the exact bounded SKMX/PSIB `R4G1Runtime` serving lane and production-surface
  unification, with the independently selecting legacy engine retained only as
  an explicitly research/off-serving surface;
- schema-2 production envelopes, content-bound deployed-quality reports,
  internal absent-section identity census, negative controls, witness replay,
  cross-surface parity evidence, and strict packaging/admission checks;
- deterministic worker partitioning and ordered reductions for graph fitting
  and deployed-quality evaluation, including worker-invariance tests;
- create-once invocation-terminal evidence, exact phase counters, five-second
  liveness heartbeats, truthful `UNAVAILABLE` fields for opaque phases, and
  five-second child-process-tree CPU/RSS/process-count plus host/disk-headroom
  sampling; and
- current claim-boundary, lifecycle, configuration, release, roadmap,
  conformance-model, BDD, native, HTTP, and WASM-facing documentation/tests.

Verification completed before parking included focused graph-runtime, API
deployed-quality, release-packaging, BDD, graph-compiler progress, Gate C
progress, an all-target workspace compile, claim-wording, and conformance-model
checks during implementation. The final post-instrumentation workspace test ladder, clippy ladder,
no-std/WASM ladder, deterministic rebuild, κ reproduction, and merge-queue
verification have **not** been completed. The checked-in branch must therefore
be treated as an implementation checkpoint, not a ready pull request.

No canonical graph/report/manifest/result CID exists yet. The required next
owner sequence is:

1. review the complete checkpoint diff and run every final repository gate;
2. correct any failure without weakening claim or negative-control boundaries;
3. build the release binary from the checkpoint revision;
4. make two fresh immutable staging copies, emit the canonical graph and both
   controls at different worker counts, and require byte-identical outputs;
5. run two independently created 6,000-position samples at different worker
   counts and require byte-identical reports;
6. launch a full census only after the sample returns typed `PROCEED` and its
   projected runtime is at most one hour; and
7. append exact positive, negative, unavailable, or stopped evidence here
   before packaging, admission, PR, or RF-31 claim promotion.

No source download, teacher forward pass, new observation run, parity marathon,
full census, package admission, pull request, or merge-queue operation was
started by this checkpoint.

## Active continuation and pre-measurement amendment — 2026-08-25

**Claim status: no new empirical verdict.** #934 is now merged and its canonical
audit is the binding diagnosis for this continuation. Active ownership resumed
on #933 after the protected `main` revision
`c29fa0e003aaa2176d7b4024508540d883490141`; the parked checkpoint was reconciled
with that revision before any canonical build or evaluation began.

The audit fixes the decision target at a 2,689-hit deficit: reference Rule 1+2
records 17,595 / 72,130 while same-position TLA records 20,284 / 72,130. It also
shows two unresolved populations—26,983 positions where the recorded teacher
argmax is absent from the reference candidate set and 27,552 where it is present
but not selected—without their joint intersection with the 3,015 TLA-only
positions. The normative evaluator must therefore emit, post hoc, a deterministic
joint table over resolution status, normative/TLA correctness cell, target
presence and rank, SKMX/PSIB contribution, and toward/away transition. Recorded
targets diagnose the fixed selector after prediction; they never participate in
candidate construction, ranking, or decode.

The evaluator must also distinguish a selector-semantics deficit from lane
quality. The portable runtime deliberately does not scan the legacy EXCT storage
descriptor, while the reference Rule 2 row gives supported exact-context
entries precedence. Because ExactContext supplies 16,933 of the reference row's
17,595 hits, the bounded replay records whether missing TLA-only evidence is
absent from the normative shortlist rather than assuming SKMX/PSIB is the sole
lever. Any later EXCT/TLA lowering must be a bounded indexed candidate source
owned by `R4G1Runtime`; reintroducing a raw descriptor scan or an independent
production scorer is not authorized.

Four immutable arms separate quality credit:

1. the current canonical sections-absent graph;
2. a newly emitted sections-absent graph;
3. the same-generation SKMX/PSIB sections-present graph; and
4. the same-generation TRAIN-label-rotated control.

The first diagnostic arm is retained at
`graph/score_canonical_base.r4g1`. It is reported only to identify drift from
the previous canonical sections-absent generation; it is excluded from
`QualityMeasurements`, the RF-31/TLA gates, and production admission.

The first comparison identifies compiler/artifact/selector drift, the second
isolates RF-31 lane movement, and the fourth retains the conditioning-specificity
null. Distinct-worker builds and evaluations must produce identical ordered
bytes and CIDs before any result is interpreted.

**Empirical Criterion. Status: Proposed. Execution scope: teacher-free
normative-runtime screening.** A content-bound ordering distributes selected
positions across held-out stories without reading held-out labels. Stage one
evaluates 6,000 positions for reachability, binding, absent identity, null teeth,
cross-surface agreement, witness replay, and statistical non-futility. A
structural falsifier or an upper interval bound that cannot clear its frozen
floor returns `STOP`. Lower interval bounds clearing both the TLA zero floor and
the RF-31 +20 permille floor return `PROCEED`. Otherwise the same immutable order
extends to 18,000 positions and returns a typed `INCONCLUSIVE` rather than
mislabeling an underpowered interval as a quality failure. After the extension,
an overlapping interval may authorize only the full census—and only when the
reachable ceiling remains sufficient and the measured projection is at most 60
minutes. A sample never authorizes production admission.

The full-census gates are unchanged: paired TLA lower bound at least zero,
paired sections-present lower bound at least +20 permille, non-positive shuffled
effect, exact absent-section identity, and zero binding, surface, or witness
failures. No teacher forward, observation run, source download, parity marathon,
or hours-class process is authorized. Every phase must retain durable progress
and terminal evidence; missing or interrupted evidence remains `UNAVAILABLE` or
`NOT_RUN`, never `PASS`.
