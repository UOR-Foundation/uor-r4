# Read-only CUDA token-to-cell export — 17 October 2026

The same-cell reference scorer needs the actual vocabulary-to-cell mapping used
by the model. Existing snapshots contain codebook vectors, and existing DEV
captures contain per-row admitted slots; neither is a declared4096-token map.
[PM ruling6240513498](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6240513498)
approves host analysis of the sealed TRAIN counts and requires this map to be
exported on CUDA through the existing `token_cell`, with no host reimplementation.

**Scope:** source implementation and host metadata checks only. CUDA construction,
the export, the battery and learned attention remain **NOT_RUN**. There is no
compiler or finite execution release for this source yet. Gate8 and its binary
retain their earlier identities and evidence scopes.

## Exact interface and format

```text
discrete-stack attention-export-token-cells CONFIG DRIVER_CHECKPOINT NEW_REPORT_DIR
```

The output directory is exclusive. The command validates the configured attention
lineage and typed DriverCheckpoint, reserves `token_cells.u16` before constructing
CUDA state, restores that exact checkpoint, and checks epoch and full snapshot
equality both before and after export. A failed command keeps available partial
files and a FAILED report; it never overwrites a prior map or claims success from
an empty/partial file.

`token_cells.u16` is exactly8192 bytes:4096 unsigned16-bit words, little-endian,
without a header, in ascending token-ID order0 through4095. Every cell must be
in0..255; ABSENT is refused. The host only copies, checks storage bounds, encodes
bytes, synchronizes the file and verifies readback. It never evaluates root
membership, a token's geometric address, model admission or ranking.

`report.json` uses `uor-r4.attention-token-cells/1` and binds:

- Actual config/checkpoint file paths and SHA256s, canonical config and typed
  checkpoint SHA256, lineage, canonical model snapshot SHA256, and source/kernel/
  geometry/spec/read/initialization/seed metadata.
- Actual output-file SHA256, encoding, count and byte inventory.
- Existing checkpoint epoch/cursor, unchanged snapshot, one model restore, one
  export kernel and4096 `token_cell` evaluations; zero forwards, proposals and
  updates, and no training capability.

The report's epoch may describe an already-learned checkpoint. It records no
new learning: export is read-only. The ordinary typed restore still performs
its existing CUDA initialization/upload/validation internally; this cost and
execution are part of the one restore, not a host reference or an extra training
step. Old-source checkpoints are not bridged to this changed source identity.

## Minimal device change

The new ordinary entry `discrete_export_token_cells` receives const codebook,
geometry and spec pointers, plus writable output and status only. Each token
calls the existing `token_cell(table,g,s,token,2,0,0)`, the identity-candidate map
used by the current writer. Invalid cells raise device status8. No route, gate
or epoch pointer is passed. The kernel and its Rust method call no forward,
proposal or apply routine.

The exact AOT entry inventory gains this one entry. Existing initializer,
`token_cell`, writer, numerical helpers and learning-law bodies are unchanged;
the old CUDA source is recovered byte-for-byte by removing the new entry.
The separate decomposition/reuse source slices remain unchanged. The ordinary
source/AOT/driver identities intentionally change, so a fresh exact-head CUDA
construction transcript and a newly bound binary are required before delivery
and execution. No waiver, lint cap or generic runtime is added.

## Checks and delivery boundary

- Host no-CUDA all-target package check: PASS.
- Two focused host metadata controls: PASS — exact raw encoding/order/length/
  range, and refusal without CUDA instead of input reads or host geometry.
- Formatting and diff checks: PASS.
- Independent exact-source review: pending; AGY review requested at the PR head.
- CUDA all-target check, strict package `--no-deps` Clippy and linked binary:
  **NOT_RUN**, awaiting the separately reviewed/released compiler-only packet.
- Device map correctness, export and immutable runtime receipt: **NOT_RUN**.

## Next dependency and scientific scope

The source-input dependency is `identity → init once → export that checkpoint →
eval/probes/qualification`, unless an existing exact-source checkpoint is
explicitly supplied. Export cannot precede creation of its bound checkpoint.
The PM packet must bind these phases with hash seals without inventing another
manual approval between them. There is no requirement that future GPU output
exist before the lease that produces it.

PM6240513498 assigns the additional DEV admitted-slot capture to the dose/
dual-checkpoint phase. A token map alone does not recreate those row-specific
admissions. Battery count/B3/export observations must remain distinct from full
same-cell dose reports; protected sheet/packet reconciliation remains PM-owned.
The frozen Battery1 draft remains preserved and non-executable until its actual
producer/validator contract is rebound. No full same-cell result follows here.

Line: M1 learned copy-successor link, Unit2c input producer. Headline unchanged;
no scientific result or mechanism negative, and no progress-count reset. This
is the specific missing input for the already-declared decisive learning test,
not a new mechanism or timing series. Training/M2/StageB remain HELD;
Q4 NOT_EXECUTABLE. Source/FIT releases neither compute nor training.
