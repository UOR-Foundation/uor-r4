# Geometric product head and shared native vector writer

References #2029, #2030 and #2028. Owner requests Kimi coordination, completion
of the head and common native vector writer, no prohibited probability/dense
path, training held, preservation and protected merge. Implementation pending
qualification/delivery. M1memory20/40 reply37/232, M2saved437/512 opened0/128
unchanged; fresh NOT_RUN. Unified integration count8/3 carried, no reset.

## Current decision and mechanism

Kimi ratifies the split at #2029 comment6138381873: Kimi owns recurrence gates
and rate-select CUDA plus the rung-0 sheet; Codex owns OP3a/shared writer and
finite device qualification. Head/writer integration only; no recurrence edits.

`head=spiralcore_product` plus `projection=spiralcore_fbs` binds one learned
`embedding.weight` parameter table to token input and output, and exposes the
same Tensor to the compiler/exact-store/emitter vector channel. Whole8D blocks,
not a forced384D representation. First/last8D query blocks admit product cells
in the same exact16-root SpiralCore atlas; at width8 these read the same block.
This is not an independently learned H4 companion or a claim about all240 roots.

Cell = token identity modulo256, collision slot = quotient. These are explicit
address assignments, not a token/hash semantic metric. Fixed atlas ports are
routing geometry; token vectors are directly learned. At most16 cells ×32
identities: at most512 token-vector comparisons; default8 cells (128 forV4096).
Expose actual cell/token IDs and validity masks. Raw score = negative mean
absolute coordinate displacement over the complete vector. Units: average
coordinate displacement, fixed scale1; margin1 uses those units. Exact ties
choose smallest cell/token identity, padding cannot emit. A compatibility
rows×V score allocation exists for callers, but adds no vector comparison or
parameter scan. Omitted tokens carry a sentinel, never objective credit.

Native margin explicitly computes the target vector's raw score, even when
omitted, versus declared admitted non-target rivals. It separately trains an
address margin versus admitted non-target cells. This is auxiliary credit for
hard admission, not its derivative or a measured descent guarantee. No target
is injected into inference candidates. The old probability producer/selection/
exponential table pool and probability loss refuse; no silent artifact conversion.
Removed source will be indexed as a restoration patch. Exact store record,
commit and occurrence/version stay separate from geometry; missing/evicted
statuses stay typed. This channel is vector infrastructure, not an adopted
learned intent compiler or a newly qualified full conversation model.

## Qualification and outcome

Pending production CUDA build and finite controls: identity/ties/padding,
raw-distance adjoints, omitted-target and address credit, same parameter across
both adapters, bound source/artifact identity and CPU/numerical refusal. No
optimizer, full trunk, training, language panel or CPU model reference.

## Resources and delivery

Owned worktree; shared harness claim and live heartbeat. Claim/card #2029
comments6138232476/6138327458. Owned podvast-55678610, RTX5090 $0.536/h,
Ubuntu24.04.3, CUDA12.8.93, Rust1.97.1. Setup controller stopped before the old
bootstrap parity suite; manual build-only setup retained. Training held.
Projected<=2h pod,4 build jobs,8GiB RSS,30GiB target+1GiB receipts; local
37GiB free at admission, preserve30GiB+128MiB floor. Cumulative pod ledger is
not reset. Old model-time path absent, not treated as a zero balance.
