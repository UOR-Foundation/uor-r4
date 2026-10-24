# REPORT: m1-position-mechanism, kimi-33

Lab: kimi. Order: [ORDER.md](ORDER.md).

References #2843

## What did not work, first

- **Actual CUDA/PTX and GPU behavior are NOT_RUN.** This Mac has no `nvcc`. `cargo check -p uor-r4-discrete --features cuda` first refused the missing explicit architecture and then failed to find `nvcc` with `UOR_NVCC_ARCH=compute_120`. An earlier progress note incorrectly said PTX admission had run; only default host builds and the build-script self-test had run. The ORDER's PTX and finite-device requirements remain open, so this PR stays a draft and no merge is requested.
- Independent DeepSeek architecture review found real defects in the first implementation: an invalid `self` argument in `from_snapshot`, a duplicate position argument that made validation fail open, a wrong `root_cost` API call, a missing CUDA-only `DriverConfig` field, accidental formatting outside scope, ambiguous payload/span anchors, and a gate-mutated query alias. These were repaired with focused controls; a later review found only the driver pin, which was updated after the final source/test text change.
- Existing internal BFS aggregation is unchanged. The new action is only the final block-`b` transport anchored at `last[node]`, ranked against an immutable copy of the block-`b` query. The pre-registration must pin that definition before any experiment; this PR does not run one.
- The position table is frozen configuration, not a learned parameter. `Parameter` has no position variant and choose/apply never writes `position_steps`, so the registered learned `H_pos`/`C_match` experiment is not runnable yet. Identity initialization collapses all three position modes to identity.
- Old binary snapshots are invalidated by the kernel/source identity changes even with `position: None`. Legacy JSON still omits `position`; D11 export explicitly refuses positioned snapshots because serving has no position read.
- Accidental formatting in five out-of-scope files and four unrelated in-crate files was reverted. The shared model-ledger path `.uor-models/native-joint-learning-2026-09-04/model-time.json` was unavailable; local free space was about 32 GiB before the final checks.

## What changed

Optional `PositionConfig { mode, steps }` now flows through `DriverConfig`, `Device::Binding` and `Snapshot`, with constructor and deserialization validation, checkpoint mismatch refusal, and default-omitted JSON. `s[14]` selects legacy BFS (`0`), prefix holonomy (`1`), matched endpoint control (`2`) or identity (`3`). The table reaches ordinary and generated CUDA entries and launchers. Nonzero modes use payload-block roots at `last[node]` and an immutable query; legacy ranking and BFS payload behavior remain on the original frame path. Validation covers every position word. D11 export refuses positioned snapshots. Driver and compiled source identities include the position module.

Sixteen `crates/uor-r4-discrete` source files change, plus this ORDER/REPORT. There are no dependency, corpus, roadmap-status, hold, bar or generated-document changes. The ORDER gave no numeric size estimate, so a ratio is not applicable.

## Measurements

- `cargo test -p uor-r4-discrete`: **237 lib passed, 1 ignored; 19 binary passed; 20 integration passed; 0 doc-test failures**.
- Position-focused controls: **22 passed**. These include constructor/deserialization validation, optional snapshot and driver round trips, mismatch refusal, raw-root legacy ranking, immutable payload query, last-position span anchors, one-position-argument snapshot ABI, and D11 export refusal.
- Red-green checks: missing config/snapshot types failed before implementation; positioned D11 export succeeded before the refusal was added; the raw-ranking, query-alias and launch-ABI controls failed on the reviewed defects; removing table revalidation allowed a full 64-entry table with candidate `16` before the guard was restored.
- `driver_source_sha256`: `0fc786d38fbebe7075702c53f706b5a93287daf2a4a35db24501937b768d1706`, verified by all three binary targets that carry the identity test.
- GPU training/evaluation, position learning, order-effect measurements, mechanism verdicts and owner releases: **NOT_RUN**.

## Checks run at the head

- `git diff --check`: pass.
- `uor-harness loop gates --quick`: 3/3 GREEN.
- `uor-harness loop gates --full`: 11/11 GREEN.
- `cargo test -p uor-r4-discrete`: all targets above pass.
- Independent DeepSeek architecture reviews: earlier blockers addressed; final review found no remaining compile/ABI/semantic defect beyond the then-stale driver pin, now corrected.
- Actual CUDA Rust type-check, `nvcc` compilation, PTX admission, ptxas/resource report, legacy device bit-identity, positioned device equivalence and malformed-snapshot device refusal: **NOT_RUN** (no local `nvcc` or GPU).
- `uor-harness loop report --check` and `uor-harness loop pr-check --local`: run after committing this REPORT; results must pass before any later review request.

## Skills used

`uor-research-architect`, `superpowers:using-superpowers`, `superpowers:brainstorming`, `superpowers:test-driven-development`, `superpowers:systematic-debugging`, and `superpowers:verification-before-completion`.

## Still open

The ORDER is not complete until actual PTX admission and finite device controls pass. A narrowly scoped owner-approved pod is required for those checks. The pre-registration needs the explicit final-transport definition before an experiment; position learning/proposal application and D11 position export are separate later work. #2843 remains `ready`; no GPU run, item completion, hold release or capability claim is made here.

## Plain summary

This change teaches the existing discrete GPU read how to accept an optional table of geometric position steps and use it for prefix, endpoint-only, or identity transport. If no position configuration is supplied, the old frame ranking and BFS transport remain selected. The change also preserves table data in checkpoints and rejects malformed tables and unsupported D11 export. Host tests and loop gates pass, and independent review caught several integration mistakes that were fixed. The new kernel has not yet been compiled with NVIDIA's compiler or run on a GPU, and the table cannot learn yet, so this is draft instrument wiring—not evidence that position helps language or that the registered experiment can run.
