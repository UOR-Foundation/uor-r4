# REPORT: m1-address-as-read, kimi-40 (Stage 0 copy and parity)

Lab: kimi. Order: [ORDER.md](ORDER.md). Decision: [DECISION.md](DECISION.md). Sheet: [`m1-address-as-read-stage0-2026-10-26.md`](../../../harness/prep/m1-address-as-read-stage0-2026-10-26.md).
References #2833; this unit does not close the item or claim an oracle gain.

## What did not work, first

- The old 54/80 oracle plan was unfalsifiable: panel-v2 contains answers in model-visible dialogue on 70/80 rows, its 19/40 baseline belongs to another model/panel, and no lawful BPB evaluator exists for this stream. The D68 review rejected that plan and the absolute bar was removed, not re-run.
- FIT initially returned NOT_FIT for an unspecified manifest, a host-reference parity path, missing identity/schema/cost details, and a stop path that lost aggregate deltas. The final sheet uses two GPU paths only, a sealed original receipt, a concrete 40-row manifest and aggregate-before-stop reporting. Fresh review then returned FIT.
- Three dispatch setup faults preceded scoring: nested store paths hid `SHA256SUMS`; an explicit-target build was needed for the known `cdylib` collision; and the staged config needed its nested input path normalized. No model result was read from those failed dispatches.
- **Copy gate failed:** exact-hint free-running recovered **0/40** exact values. Exact versus no-hint has `b = c = 0`; exact versus random-hint also has `b = c = 0`. Neither primary comparison meets `d > 24 && d^2 > 4(b+c)`. Panel-v3, BPB and training did not follow.

## What changed

- Added `address_stage0` manifest generation and a GPU-only runner behind `stage0-lab`: pinned artifact/config/receipt validation, corpus-leak scan, two D11 parity passes, sealed `final-dev.json` comparison, teacher-forced and free-running copy, identity and aggregate-preserving stop reports.
- Added the 40-row manifest (`ea360118…`), protocol and pure parity tests, and the root workspace exclude required by the nested D11 instrument.
- Recorded the scoped result in `docs/mechanisms/exact-memory.md`, the lab README and `docs/research/notebook/m3-durable-memory.md`.
- Size at the REPORT commit: 16 files, 8,198 additions and 6 deletions. Code and unit tests 2,743 lines, documentation 201, generated manifest 5,212, dependency/workspace metadata 42. No numeric estimate was set; most additions are the generated manifest and the GPU runner.

## Measurements

**GPU, one RTX 5090 (`vast-57806658`) at `97354145ecaf7ea70908ef25922ea3c57dec55fe`:**

- Parity: **14,287/14,287** rows have status 0 and agree with the sealed original GPU receipt on target position/token, prediction, top-8 and admitted count. The receipt has no ordinal field; row order was used only to join corresponding records. Repeat records are byte-identical (`7f373b3b…`). Aggregate top-1 `455`, top-8 `1,484`, admitted `5,009`; all position-strip deltas are `0`. `parity-report.json` PASS.
- Copy: exact free-running **0/40 exact matches**, no-hint **0/40**, random-hint **0/40**. Normalized emissions were **12 / 0 / 9** respectively; these are not exact matches. Both primary comparisons have `b = c = 0`, hence `COPY_GATE_FAIL`. Random-hint planted-wrong held on all 9 normalized rows.
- Teacher-forced (descriptive only): exact `3/275` token top-1 and `15/275` top-8; no-hint `4/275` and `17/275`; random-hint `5/275` and `18/275`. No row has all value tokens top-1 or top-8. These are not generation capability claims.
- Config provenance: original config `43a316bc67bb45fe5b50bd2ceee57f6b919164c02fe45ee9e677f8819fea5d15`; normalized config `e6f85ca9ddaf2d1e471d6e15d385f4bce6411ef7514c412f32ac44776d838569`. The staged copy changed only the nested input path with a `sed` path replacement so it named the pod's staged input location.
- Driver provenance: the measured run's identity receipt did not include the actual NVIDIA driver version, so it is **UNVERIFIED** for this run. The repaired runner now queries `cuDriverGetVersion`; no driver version is inferred for the old receipt.
- Cost: pod uptime `1,163 s` at `$0.563/h` = **$0.182** (not an inferred energy measurement). One pod, released and deleted after upload. Host preparation/review cost was lab time only.
- Artifacts: `results/kimi/m1-address-stage0-2026-10-26/results/stage0-result.tar`, local tar SHA-256 `2be4f5ef8880e6c7a53505bb6974ee06f063b49108f98b6bc35ce3f73e4c4a07`. Binary `6aa0709ddbf2b78275f95e9a8005328c37b4951fb69814098eb74abaffc7e530`; cubin `17dd4184d1e49dbb07b7f1d7bb236859c38b715edd968255dd41e61196a10465`; PTX `e8ee1c1292a46f9a5eb24779f163e771fdce94dc10644f9ce89050f5dec333be`.

## Checks run at the head

- `CARGO_TARGET_DIR=local/target ~/.cargo/bin/cargo test -p uor-r4-object-store --no-fail-fast`: **83 passed, 0 failed, 1 ignored** after the repair.
- `CARGO_TARGET_DIR=local/target ~/.cargo/bin/cargo test -p uor-r4-object-store --features stage0-lab --no-fail-fast`: **112 passed, 0 failed, 1 ignored** after the repair.
- `CARGO_TARGET_DIR=local/target ~/.cargo/bin/cargo check -p uor-r4-object-store` and the same check with `--features stage0-lab`: PASS after the repair.
- `NVCC=/Users/casey.allard/uor-r4/local/fake-nvcc/nvcc UOR_NVCC_ARCH=compute_120 CARGO_TARGET_DIR=local/target ~/.cargo/bin/cargo check -p uor-r4-object-store --features cuda`: PASS.
- `uor-harness loop gates --full`: see final head.
- `uor-harness loop report --check --order docs/loop/tasks/m1-address-as-read/kimi-40/ORDER.md`: see final head.
- `uor-harness loop pr-check --local`: see final head.
- D68 review: REJECT old plan; Stage 0 accepted as the required next measurement.
- FIT: FIT at `97354145e` (agent-32). D67: PASS at `97354145e` (agent-34), with full-HEAD, pod-build, input and cost corrections applied.

## Skills used

- `uor-research-architect`: whole-project mechanism/claim boundaries and instrument-first review.
- `superpowers:systematic-debugging`, `test-driven-development`: failing protocol tests before implementation and each dispatch fault classified before any result.
- `scope-guard`, `solution-integrity`, `holding-the-goal`: no larger panel or training after the copy gate failed.

## Still open

- This artifact/copy layout cannot support a hidden-fact oracle measurement. Reassess the model/read instrument before any v3; no current mechanism claim follows.
- `needs_gpu: false` in the roadmap frontmatter disagrees with GPU model evaluation and remains a separate metadata follow-up.
- The address consumer's 6 finite controls remain valid at store scope. Level 3 trunk behavior is not in the D11 and was not measured here.

## Plain summary

The integer model reproduces the original GPU result exactly on every development row. It still cannot copy a short supplied value exactly once in 40 tries. That means a bigger memory-oracle panel would not measure anything useful on this artifact, so I stopped instead of calling it a memory result.
