# Attention artifact hash domains — 17 October 2026

[PM disposition6243260600](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6243260600)
formally accepts the completed [Battery3 preflight](../attention-preflight-2026-10-17/README.md)
without rerun or rescore, and commissions this bounded emission/validation repair
before the paired learning dose. The same stored snapshot has two legitimate
serialization hashes. The earlier export called the stored model-Value hash
`model_snapshot_sha256`, while evaluation used that name for the typed Snapshot
hash. Comparing those two fields directly rejects correct matched artifacts.

## Explicit domains and bindings

| Domain | Exact bytes | Future evidence field |
| --- | --- | --- |
| A | Existing stored model `serde_json::Value` serialization | `model_storage_sha256` |
| B | Supported typed `Snapshot` serialization | `model_snapshot_sha256` |
| C | Canonical typed `DriverCheckpoint` serialization | `driver_checkpoint_sha256` |
| D | Actual raw file bytes | The applicable `FilePin.sha256` |

`DriverCheckpoint.model_sha256` keeps its historical Domain-A meaning and format.
The repair adds explicit A/B metadata to export/evaluation/progress/transfer
records and checks both against the same actual nested checkpoint. Shared
helpers only deserialize/re-encode stored metadata and hash the resulting bytes;
they create no Device, CUDA context, address, geometry, model score or admission.
The existing qualification INIT LOCK observations are persisted with their typed
hashes without introducing any extra control, initialization or forward call.

The checkpoint-bound evaluation path preserves Domain A from the original
stored Value, including explicitly present default fields. It checks Domain B
and epoch against the evaluator's existing single typed Snapshot read before
emitting that A. Restoring typed state may normalize defaults; its normalized
Value hash must not silently replace the original checkpoint's storage hash.
The `eval-corpus` CLI and attention/transfer callers pass the retained exact
checkpoint through this metadata binding. Ordinary Device-only evaluation
continues to report the canonical Value generated from its current snapshot.
No extra snapshot read, initialization or Forward is introduced.

The actual Battery3 checkpoint is the concrete witness: Domain A is
`ea67bedd3b4a3c6c34d3431008fefd451a5b10895306fa6c2ccda4497ab4a7f8`,
Domain B is
`bb160787b43d3bb342e8707d6ff2d3b473f56d6524954335a77920b1e02c00da`.
Its existing `init-b3/report.json` cross-domain validation passed. These are not
corrupted or mismatched storage. Accepted historical files and their hashes stay
unchanged; they are not rewritten to the corrected names.

## Source and execution scope

The change is confined to metadata emission, integrity validation and focused
serialization/refusal fixtures. It changes no CUDA kernel, initialization,
selection, objective, numerical computation, proposal, sole apply transition,
learning law, registered scientific panel, control count, dose or frozen Route
completion/timing source. New-format progress requires its explicit bindings;
it does not silently migrate old progress or checkpoint lineage.

The source identity and future executable identity change even though numerical
behavior does not. Old construction passes, binaries and checkpoints retain
their exact source scope. Exact-head independent source/packet review, applicable
compiler construction and protected delivery are needed before the dose packet
is rebound. This source commission is no model or provider execution release.

## Source-review correction retained

Independent review of the first candidate a2af13cc identified the explicit-
default storage edge before any construction or runtime release. Its report
SHA is `b99f1b53a6c209834367a6aa579ed902fc5c800276e35ae617b4232cf3972ce8`.
The narrow checkpoint-bound metadata repair includes the existing eval-corpus
CLI leaf; it does not tighten legacy checkpoint admission or add an artifact
bridge. The initial candidate and prospective compiler manifest are preserved
as predecessor evidence, not promoted to a final approval or paid attempt.

## Verification and next dependency

The owned `codex/cf1-attention-hash-domains` candidate passes host no-CUDA
all-target checking, formatting and85 metadata library tests (0failed,1ignored).
The first full metadata run had83PASS/1FAIL/1ignored because a pre-existing
storage fixture needs the ignored `local` parent in a fresh worktree. The failed
log is retained; creating that parent, as the accepted compiler recipe already
does, yielded the passing rerun without a source change. New named-field
metadata tests passed in both runs. The initial six-file candidate had84 passing
tests; the exact stored-checkpoint correction adds a third host metadata test
and passes85/0/1 at its final source. Its author verification report SHA is
`7f49ce69c336ade601d0b13e2086209ff9c0514520f322f3eeab7df373fbff0c`. The two real typed-Snapshot serialization
fixtures are CUDA-feature scoped: source authored, execution NOT_RUN. They
construct no Device and test only encoding/default omission/refusals. CUDA
construction, new binary linking, new runtime artifact emission and learning
execution are NOT_RUN. The prospective
Gate10 packet retains the five declared compiler-only commands and accepted
lifecycle, with separate actual release required. No old gate release is reused.

After source review, construction and protected delivery, bind the actual new
source/executable and final artifact contract into the one-seed-first paired
Relation/frozen-zero packet. Kimi retains FIT, protected queue and execution
release. AGY owns its separate consumer and conforms to the explicit domains;
its final evidence synthesis is not a separate learning-start prerequisite.
There is no extra timing run, seed-choice loop or fourth acknowledgement phase.
The registered reply37/232 remains SUSPENDED, learned Relation NOT_RUN and the
recorded line count0/3 is neither reset nor reassigned by this repair. Training,
M2 and StageB HELD; Q4 NOT_EXECUTABLE.
