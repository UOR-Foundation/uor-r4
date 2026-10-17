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

## Verification and next dependency

The owned `codex/cf1-attention-hash-domains` candidate passes host no-CUDA
all-target checking, formatting and84 metadata library tests (0failed,1ignored).
The first full metadata run had83PASS/1FAIL/1ignored because a pre-existing
storage fixture needs the ignored `local` parent in a fresh worktree. The failed
log is retained; creating that parent, as the accepted compiler recipe already
does, yielded the passing rerun without a source change. New named-field
metadata tests passed in both runs. The two real typed-Snapshot serialization
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
