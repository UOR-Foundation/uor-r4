---
id: m1-w2-code-utilization
title: Run the W2 code-utilization diagnostic on the stored capture
milestone: M1
area: evaluation
status: done
open_to: lab
needs_gpu: false
depends_on: [m1-w2-capacity-capture]
scope: [docs/harness/prep/m1-w2-code-utilization*, docs/labs/m1-w2-code-utilization/**, docs/loop/tasks/m1-w2-code-utilization/**]
notebook: docs/research/notebook/m1-discrete-attention.md
done_class: host
---

## Why it matters

W2 classified the seed-1 Q4 read code as H_mixed, the third row of its registered sheet's decision table (`docs/harness/prep/m1-read-code-capacity-2026-10-17.md` §4.7). The sheet's next step for that row is "Code-utilization/capacity diagnostic; do not change the learning law yet". The M1 bar needs the exported model to beat the T2 successor table (6,300 of 14,287 on DEV). The trained model's prediction is a function of its 16-bit read code alone. So where the code loses target information decides the next M1 spend:
- in admission: the candidate is the full-vector address key, `m1-address-key-full-vector` (#2834). Its item says the 5-value address key leaves "which cells open" largely to "arbitrary numbering", and the open cells are the writer's admitted cells;
- in the code itself: a change to what the read encodes;
- in too little TRAIN data: the full-TRAIN capture that W2 skipped.

The Stage T item (`m1-bvg-k-stage-t-training`, #2836) is "only warranted if W2 blames the learning law (H_law)". This diagnostic supplies the evidence for that call; the call itself is a separate decision review.

## What is missing

**Outcome: complete.** The stored-capture diagnostic reproduced every W2 number exactly and answered all three registered questions: QA A1a, QB B2 and QC C1. Code-count admission scored 9,064 against the writer's 1,920 and the fixed frequent-cell list's 5,935; the sets are not load-matched, so this is not a load-controlled model result. The data-scaling projection was 3,552, below the H_law line, so the full-TRAIN G1 capture was declined. The code × query table scored 7,262 against the bigram's 6,300, showing information beyond the last token. These are host diagnostics on one checkpoint and DEV's 14,287 rows, not model scores ([record](../../labs/m1-w2-code-utilization/README.md)).

The original pre-run text below described four unmeasured questions. They are now measured. The notebook-row write remains tracked separately as #3189; the sealed lab record and claude-95 REPORT hold the result.

## Done when

**Original acceptance text, retained as history:**
- The registered sheet `docs/harness/prep/m1-w2-code-utilization-2026-10-25.md` (with its non-author FIT) runs once on the stored W2 capture.
- Each of its questions records one band, and the bands go in as one row in `docs/research/notebook/m1-discrete-attention.md`.
- The full-TRAIN GPU capture is either registered under its own pre-run review or declined, as the sheet's data-scaling question decides.
- An instrument fault is recorded instead if any recomputed W2 number differs from the sealed record.

**Observed outcome:** all three bands are recorded in the sealed record; G1 was declined and no precedence-0 fault occurred. The notebook-row tooling gap is #3189, not a reason to rerun the diagnostic.

## How to start

**Historical pre-run procedure; the diagnostic is complete.**

Open these first:

- `docs/labs/m1-w2-capacity-capture/README.md` (the numbers, the artifacts and how to reproduce them)
- `docs/harness/prep/m1-read-code-capacity-2026-10-17.md` (§4.3 per-row fields, §4.7 rows 3–4)
- `crates/uor-r4-discrete/src/w2_capacity.rs` and `crates/uor-r4-discrete/src/bin/w2-assemble.rs` (the host tables, tie rules and floors to reuse, read-only)
- `docs/labs/m2-fresh-heldout-panel/instrument/Cargo.toml` (the standalone-instrument pattern: a detached `[workspace]` and path dependencies on workspace crates)
- `docs/roadmap/items/m1-address-key-full-vector.md` and `docs/roadmap/items/m1-bvg-k-stage-t-training.md` (what the outcomes feed)

## Rules that apply

- **Geometry only** (CHARTER §3.1–§3.3): no softmax or squashing anywhere; no dense matrix multiplication. Count tables are lawful diagnostic comparators (§4.4).
- **Exact integers (this item's choice):** every quantity is an exact integer or an exact integer ratio. The W2 sheet's "code–target mutual information" (§4.7 row 4) is measured in exact forms such as leave-one-out top-1 and collision agreement. Information in bits would need logarithms and float types. CHARTER §3.7 keeps float out of the discrete crate, and its banned-presence gate counts float types.
- **Host post-processing only** (§3.5, the W2 sheet's D24 classification): counting and scoring stored GPU observations on the host is allowed; any model forward, init or apply is GPU work and needs its own sheet and pre-run review.
- **Pre-registration first** (§4.5): the sheet and its non-author FIT come before the run, because the result routes M1 spend.
- **Rust first** (§3.11): the host tool is Rust; no new Python.
- **A standalone instrument, never a crate edit:**
  - **Where it lives:** the tool is the crate `docs/labs/m1-w2-code-utilization/instrument/`, with its own `Cargo.lock`.
  - **Its dependencies:** a detached `[workspace]` and a read-only path dependency on `uor-r4-discrete`, built without the `cuda` feature.
  - **The crate is not edited:** if the tool seems to need a change in `uor-r4-discrete`, that change is a separate item, not part of this one. This keeps the item off the crate scope that other M1 items hold, so `loop claim` can serialize them.
  - **What it reuses, and what it reimplements:**
    - Reused, as public API: `w2_capacity`'s `decode_read_vector`, `unpack_code`, `build_tables`, `predict`, `score_dev` and `exact_mcnemar_rejects_005`.
    - Reimplemented in the instrument, because it is private in the crate: the tie rule (`rank`), the floor helpers, the keyed-count internals, and the new leave-one-out, joint-key and candidate-load counts.
    - The guard against divergence is the sheet's precedence 0: every W2 number (T 3,414, G 6,300, U 1,226, W 149, A_W 1,920 and the rest) must be recomputed exactly before any band is read.
  - **Drift risk:** the instrument sits outside the workspace, so the workspace build and CI never compile it. A later change to `w2_capacity` (Stage T's scope covers the crate) could break it unnoticed. So the instrument pins the API at the `uor-r4-discrete` commit it builds against, and its record carries that commit; precedence 0 catches any change in behaviour. This is acceptable for a tool that runs once, for minutes.
  - **Dependencies:** the implementing unit's ORDER carries a `deps:` line for the instrument's new `Cargo.lock` (`docs/loop/README.md`, "Scope, without a line cap"), as in `docs/labs/m2-fresh-heldout-panel/instrument/`.
  - **Float:** the exact-integer rule applies to the instrument too.
- **Records** (§4.1, §5.6): one notebook row; a negative closes this configuration (seed-1 Q4, epoch 407, model-matched TRAIN), never a mechanism.

## Sources

- [Completed code-utilization record](../../labs/m1-w2-code-utilization/README.md) and [claude-95 REPORT](../../loop/tasks/m1-w2-code-utilization/claude-95/REPORT.md)
- [W2 capacity-capture record](../../labs/m1-w2-capacity-capture/README.md)
- [Registered W2 sheet](../../../docs/harness/prep/m1-read-code-capacity-2026-10-17.md) and [code-utilization sheet](../../../docs/harness/prep/m1-w2-code-utilization-2026-10-25.md)
- [Stage T item](m1-bvg-k-stage-t-training.md) for the subsequent diagnostic-stop outcome

## Cancel if (historical pre-run guard)

The stored W2 capture cannot be authenticated against its pins (run-b.tar `6aa0a766…`, train-stream `70a58767…`, query-sidecar `6d5d4f5d…`), or a later W2 rerun on another checkpoint supersedes the seed-1 Q4 capture before this runs.
