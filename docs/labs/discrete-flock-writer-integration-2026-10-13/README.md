# Discrete flock and shared vector writer integration

Date: 2026-10-13. Owning milestone M4 #2032, shared M1 learning interface #2029.
Status: SOURCE IN PROGRESS. Training and M2 HELD. No flock CUDA qualification has run.

Claude's corrected design was approved at `7e3d1995fa5f8775182c17d65c51e517bcbc80a6`
and merged as #2378 at `5e85f696e3d23f9a6028bcd6ecf411a28111a373`, byte-identical.
This integration uses the shared integer codebook, SpiralCore frame/payload transport,
ordered FBS binder plans and geometric writer introduced by #2379. It does not inherit
#2379's numerical qualification for its new read or gate paths.

## Source ownership and interface

Claude owns `src/discrete.cu` and `src/flock_qualification.rs`; Codex owns the Rust
host, CLI, artifact and build bindings. All files are in `crates/uor-r4-discrete`.
The source is developed in separately claimed worktrees; the owner checkout is read-only.

The read configuration is artifact-bound: `ReadSpec::Route` or `Flock { kept }`,
with flock K in 1..8. Existing `Device::new` selects Route. `new_with_read` selects
explicitly. Snapshot schema2 includes the read configuration and the full i32 gate
inventory `[blocks, stages]`. Missing fields or schema1 do not silently migrate.
Mode and K occupy device spec words8/9; gates follow routes in the six affected
kernel interfaces. The existing seven PTX entry names remain the current contract.

Gate kind3 uses exactly three active candidates: ordinal0 is the current trit;
ordinals1/2 are the other values in `[0,-1,+1]` order, skipping current. The physical
loss buffer and launch retain sixteen slots/lanes, with unused thirteen set to MAX.
The chooser examines only three active gate candidates. Route/VectorBlock retain
sixteen active candidates. There is no host scoring, transport, model fallback or
floating-point model arithmetic. Route parameters in Flock mode and Gate parameters
in Route mode refuse. The CLI still exposes no apply command; training admission
continues to return TrainingHeld.

`Proposal::candidate_losses()` is readback of completed CUDA observations for
qualification. It does not compute candidates or select a winner on the host.

## Verification and limits

Host metadata-only library tests: 4/4 PASS. Scoped Rust formatting and diff checks
PASS. The new test exposed Serde accepting extra fields on an internally tagged
unit Route variant; strict deserialization through an empty-struct wire variant
now rejects them. This is metadata validation, not model execution.

Composed CUDA build, new exact flock controls and artifact round-trip: NOT_RUN.
No applied update, corpus learning, learned-language result, throughput or full E8
coverage is established by this work in progress. The geometric atlas remains the
sixteen-root anchor orbit, not all240 E8 roots. Exact fixture expectations will be
frozen before prospective finite TEST FITNESS and numerical execution.

## D2 static cost review supporting Kimi's pending learning sheet

This is a source-loop bound for the pre-flock Route kernel, SHA256
`9d3368a4c526686c53826cfa0d6dc0e2b164bc2cff384b3f8ea520af488e503c`.
It is not measured GPU instruction count, time, throughput or a spend projection.
Let V=vocabulary, B=blocks, C=context, m=batch rows, K=writer kept cells,
S=1+floor(log3 C), O=occupied cells and k=min(K,O). Flock neighbour K is a separate
quantity and must not be substituted for writer K.

For a fully eligible sixteen-move proposal, current source evaluates sixteen
baselines plus sixteen candidate paths:32m read/writer calls. A single baseline
and fifteen nonidentity candidates would need only16m distinct evaluations.
This redundancy is an implementation cost to account for, not a geometric negative.

Each writer performs at most:

| Work | Source-loop ceiling |
|---|---:|
| token-cell lookups | (k+1)V+1 |
| root-ID coordinate checks within those lookups | 256((k+1)V+1) |
| address-distance calls | kO-k(k-1)/2+1 |
| full token-distance calls | V+1 |
| token-distance coordinate comparisons | 8B(V+1) |

The writer scans identities for occupied cells and again for each kept cell,
recomputing first/last root IDs. Source-loop ceilings ignore early exits. Full
Route/VectorBlock sweep inventory is B(S+V) proposals on the declared fixed batch.
Initial carry touches8BmC physical elements; accepted updates invalidate carried
epochs, so the eventual driver must include recarry work in its dose accounting.

Illustrative dimensions ONLY (not a registered run):V4096,C192,m64,writer K8.
S=5; B3 yields12303 proposals and25196544 read/writer calls per full sweep;
B4 yields16404 proposals and33595392 read/writer calls. One proposal has a ceiling
of19327877120 root-ID coordinate checks. These bounds cannot be converted to dollars
or tokens/s without an admitted instrument and measured execution.

Flock read and gate costs await the completed kernel. Route parameters are inactive
in Flock mode; its sweep inventory is vector blocks plus gates. A gate proposal
with three duplicated baseline/candidate evaluations would have6m writer calls,
conditional on the final implementation. Prospective finite qualification must
pin source, shape, measurement method and scope before any timing run. Training
remains held regardless of a finite qualification result.

## Learning-sheet review

Kimi's #2391 successor sheet is PENDING, not a training release. Its current arm
explicitly excludes flock/Gate. Codex's review is
[#2391 comment6156858784](https://github.com/UOR-Foundation/uor-r4/pull/2391#issuecomment-6156858784).
D1 requires actual seeded initialization, corpus/mask/BOS segmentation, exact scored
coverage, GPU count reference and top8 evaluation, parameter schedule and qualified
update boundaries. D2 requires a concrete cost model; static bounds above do not
supply measured throughput. Pin full representation dimensions and evaluator
eligibility before assigning a model negative. Missing rivals can produce zero
hinge by law; those rows cannot count as successful learning. The owner training
release, fresh-transfer identity and R14 suspended reply-quality headline remain
separate constraints.
