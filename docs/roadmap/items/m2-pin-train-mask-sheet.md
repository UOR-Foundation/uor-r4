---
id: m2-pin-train-mask-sheet
title: Pin the M2 TRAIN-mask, SHA, CLI and source sheet
milestone: M2
area: tooling
status: ready
open_to: anyone
needs_gpu: false
depends_on: []
---

## Why it matters

M2 is the first simple conversation: at least 256 of 512 saved questions and at least 40% of fresh ones, graded by fixed deterministic checks ([where we are going](../../start/where-were-going.md), Stage 5). Before any GPU training or fresh-transfer evaluation, the charter requires a one-page pre-registration sheet with a non-author FIT (CHARTER §4.5). Today the masked TRAIN count reference is built and tested, but its physical sheet is still PENDING. A sheet that is ready for FIT shortens the path once the owner decides on the M2 hold. See also [the transformer-to-geometry map](../../start/transformer-vs-geometry.md).

## What is missing

The 18 October next step is mechanical pinning of the masked TRAIN reference into a sheet. The reference is frozen at source `2fd6358bbc52d2a202a872ad0954ebb12894606f`, which passes 11 finite controls (9 metadata, 2 CUDA). Protected PR #2328 merged at `9e904a83448c451f738d5c1d45224dfdf73eb84e`, and an independent Claude CONFIRM FIT is recorded at `#2029` comment 6146716704. An independent data-only check counts 2,425,925 eligible TRAIN targets.

The record says the physical sheet verdict stays PENDING until the sheet adds four pins: the mechanical TRAIN-mask path and checksum, the expected count, the CLI, and the new source. M2 training and rental stay separately HELD. A FIT releases nothing.

## Done when

A sheet pins the TRAIN mask (path and checksum), the SHA, the CLI and the source, with a non-author FIT, ready for an owner hold-release decision. The sheet itself runs nothing: no training or optimizer update is authorised by it.

## How to start

1. Read the record `archive/labs/count-reference-2026-10-13/README.md` (the item's earlier path `docs/labs/m2-masked-train-reference-2026-10-13/` no longer exists), then the Amendment 5 registration `docs/harness/prep/m1-rank-margin-rung0-2026-10-13.md` and `docs/harness/milestones.json` for the exact wording of the pending pins.
2. Open the code at the pinned source: `git show 2fd6358bbc52d2a202a872ad0954ebb12894606f --stat`, `crates/uor-r4-training/examples/geometric-stack.rs` (the CLI) and `crates/uor-r4-core/src/native_geometric/learner`. Find where the TRAIN mask is read and where the expected count is checked.
3. Reproduce the pins on a clean checkout, CPU only: build with `~/.cargo/bin/cargo build -p uor-r4-training --features native-rung0 --example geometric-stack`, hash the TRAIN mask file, and confirm that the eligible-target count is 2,425,925. Note that the example may need CUDA for its two CUDA controls; the 9 metadata controls are host checks.
4. Post a draft one-page sheet on `#2030` as the first deliverable. It pins the mask path and checksum, the expected count, the exact CLI line, and the source SHA, and it states the hold status (training and M2 stay HELD) and the bars unchanged from Amendment 5.
5. Ask for a non-author FIT (a fresh-context reviewer counts under CHARTER §4.5) at the exact head of the sheet.
6. Open a PR that says `References #2030` (the sheet is ready for FIT, but the owner's hold release is not part of this item). Record the result as one row in the closest research notebook under `docs/research/notebook/` (see `docs/research/notebook/README.md`; no M2 page exists yet, so propose the line with the row).

## Rules that apply

- **Geometry only** (CHARTER §3.1–§3.3): every mechanism, training signal and fix comes from the project's geometry. No softmax or squashing anywhere, including losses, gradients, sampling and renamed or table forms. No dense matrix multiplication or GEMM, including wrappers and hand loops.
- **Rust first** (§3.11): new code is Rust, and no new Python is added.
- **Records** (§4.1, §5.6): each result is one row in its line's research notebook (`docs/research/notebook/<line>.md`). A negative result closes a configuration, never a mechanism.

## Sources

- issue#2030
- [`archive/labs/count-reference-2026-10-13/README.md`](../../../archive/labs/count-reference-2026-10-13/README.md)
- [`docs/harness/prep/m1-rank-margin-rung0-2026-10-13.md`](../../../docs/harness/prep/m1-rank-margin-rung0-2026-10-13.md)
- [`docs/harness/milestones.json`](../../../docs/harness/milestones.json)
