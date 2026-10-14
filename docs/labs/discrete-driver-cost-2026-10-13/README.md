# Discrete driver and measured enumeration cost

Date: 2026-10-13. Codex/Sol, M1 #2029; shared writer/read interface M4 #2032.
Line: discrete read/writer construction. No language headline or learning KEEP/count reset.
Status: source under review; finite CUDA qualification and cost measurement NOT_RUN.
Training and M2 remain **HELD**.

This unit supplies the D1 corpus/evaluation/schedule interface and D2 cost instrument
for Kimi's separate learning sheet. Three Codex agents implement disjoint slices;
Claude reviews the prospective scope and exact source. It preserves the merged
SpiralCore/E8/FBS carrier, shared token codebook/writer and discrete flock read.
The atlas remains the sixteen-root anchor orbit, not all 240 E8 roots.

## Driver contract

`prepare-plan CONFIG NEW_REPORT` is metadata-only. The Rust loader verifies file
SHA256, canonical 64-byte UORT header, token count/vocabulary, binary response mask,
explicit tokenizer/BOS identity and expected eligible counts. Each selected target
uses its preceding causal prefix, clipped at BOS and the configured context. A final
short batch is retained. The target and future positions are excluded from input.
Masked positions never enter the count reference or target eligibility.

The [example configuration](driver-config.json) pins the existing TRAIN/DEV and
tokenizer bytes. Its paths are restoration paths, not committed datasets. Metadata
admission observed 2,425,925 TRAIN targets and 14,287 DEV targets; see the
[coverage plan](receipts/coverage-plan.json). The DEV manifest's historical `split:
train` label is not its role: the explicit config pins it as DEV. Corpus files were
restored from the existing private HF store and checked against the sheet's hashes.
No corpus model computation follows from this metadata admission.

`canonical_vector_then_read_v1` visits ascending codebook blocks, then active Route
coordinates or Flock gates, once per coordinate pass. One coordinate is attempted
per batch; the corpus advances in ascending eligible-target order and wraps only
after complete coverage. Initialization has an explicit integer seed; data and
coordinate order are identical across paired seeds. No shuffle is implied.

`init-corpus CONFIG NEW_REPORT` constructs an epoch-zero GPU checkpoint.
`eval-corpus CONFIG DRIVER_CHECKPOINT NEW_REPORT` validates the checkpoint's full
source/config/data/mask/tokenizer/role/cursor lineage before restoring it. The
checkpoint stores complete integer model state and the coordinate/data cursor;
resume rejects changed identities. `init` and `init-read` require an explicit seed.
Every report directory is exclusive and reserved before model loading.

The complete corpus transition loop exists behind an opaque, unissued
`CorpusUpdateAdmission`; its constructor returns `TrainingHeld`. The public train
CLI always refuses. There is no flag, environment override or deserializable token
that releases this hold. Owner release needs a separately reviewed admission change.

## GPU evaluation

The native CUDA evaluator uses the same read and writer as inference. Global top8
orders all admitted token identities by exact distance then ID; it never concatenates
per-cell top lists or inserts the answer. Component hinges, exact denominators,
rival eligibility, hits and batch/corpus aggregation execute on the GPU. The host
decodes completed receipts, prepares input metadata and stores results.

The frequency reference counts response-masked TRAIN targets on CUDA. A separate
DEV-response oracle is labelled diagnostic; changing DEV cannot alter the TRAIN
histogram or ranking. The pre-existing no-float/no-softmax/no-hidden-matmul contract
remains. No Python model dependency or CPU model implementation is introduced.

## Prospective qualification and cost scope

[Claude's prospective review](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6157700698)
requires replay to match actual applied state, all three parameter kinds, stale and
foreign refusals, no-op epoch stability, deterministic resume, mask/BOS coverage,
and measurement of active flock as well as Route. The
[finite sheet](../../harness/prep/m1-discrete-driver-finite-2026-10-13.md) owns the
exact inventory and prospective FIT. No CUDA result is claimed before execution.

Only literal, authored, disposable fixture models may receive bounded construction
updates. They are dropped in-process; no post-apply checkpoint is saved or promoted.
The report counts every accepted update. Corpus data never enters an apply call.

Cost uses a release Rust build and CUDA 12.8+ on one owned Vast GPU. The measured
shape is vocabulary4096, four8D blocks, context192, batch64, writerK8, with both Route
and FlockK8. Active flock uses authored +1 gates because zero gates skip the read;
this is a cost fixture, not a selected learning arm. Warmup precedes three raw
repeats. Integer host nanoseconds surround synchronized device calls. Setup/module
load/JIT/init is separate; no unsupported exact JIT subset or GPU peak-memory claim.
Device memory samples and compiled per-thread local-memory attributes are reported
as observations. Tiny-fixture apply timings are not production apply throughput.

Raw timings, integer min/median and floor-divided tokens/s will support projections
of a parameter pass and bounded dose. A projection is not an executed corpus sweep
or measured training throughput. A slow configuration informs implementation/dose;
it cannot rule out a mechanism. No learning or language-quality result is claimed.
