---
id: m1-bvg-k-block-apply-c2
title: Draft the BVG-K Stage T pre-registration sheet (C2 has passed)
milestone: M1
area: training
status: done
open_to: lab
needs_gpu: false
depends_on: [m1-a8-full-dose-rerun, m1-w2-code-utilization]
scope: [docs/harness/prep/m1-bvg-k-block-apply-c2*, docs/harness/prep/m1-bvgk-stage-t*, docs/harness/prep/m1-bvgk-staget*, docs/labs/m1-bvg-k-block-apply-c2/**, docs/loop/tasks/m1-bvg-k-block-apply-c2/**, docs/research/notebook/m1-discrete-attention.md]
---

## Why it matters

BVG-K is the faster discrete learning law that Stage 2 tries first if W2 blames the learning law. It scores 16 coordinates in one GPU launch and commits a block of moves only when the joint result is strictly better. Stage C2 is the last finite qualification before a training run (Stage T) can be registered. See [the discrete learning page](../../mechanisms/discrete-learning.md).

## What is missing

**Outcome: complete.** Stage C2 (block propose, verify and apply) qualified with V0 source identity, RC re-carry parity, the UT/JR/DC/V3 block sweep, RF refusals and C1R parity. All 3,419 updates were on synthetic fixture devices, and `Device::apply` was never called. The receipt is `docs/labs/injective-preflight-and-c2-qualify-2026-10-19/evidence/bvgk-c2-qualify-20261019/report/bvgk-qualification.json`, SHA-256 `2fff942fa2b802e180c9bd87336692730d6beaf22f2c0bac8867afb0dea345f3`. Stage C1 also passed: 5,130/5,130 exact K-coordinate scoring and K=16 at 16.25x throughput.

The Stage T sheet, FIT, D67/D68 reviews and two-seed run are now complete. Its five Step 0 controls covered the C2 gaps, and its actual outcome was a diagnostic stop with the Stage 2 writer bar unmet, not a Stage 2 pass ([Stage T record](../../labs/m1-bvg-k-stage-t-training/README.md)). No further pre-registration work remains on this item.

## Done when

**Original acceptance text, retained as history:** Stage C2 finite controls PASS and are recorded, and a one-page Stage T pre-registration sheet is posted complete enough for non-author FIT. The Stage 2 writer bar remains quoted in the Stage T item; it was not rewritten here.

**Observed outcome:** C2 passed and the Stage T sheet/reviews/run are complete. Stage T's diagnostic-stop result is separate from C2's finite-control pass.

## How to start

**Historical pre-registration procedure; both C2 and the Stage T sheet/run are complete.**

1. Read `docs/labs/injective-preflight-and-c2-qualify-2026-10-19/README.md`, the C2 sheet `docs/harness/prep/m1-bvgk-stagec2-block-qualification-2026-10-18.md` (use its layout), the row 7 entry in `docs/integration/NOW.md`, and `docs/mechanisms/discrete-learning.md`. Also read the K16 outcome record `docs/labs/k16-outcome-call-2026-10-19/README.md` (no KEEP) and the W2 entry in NOW.md, because NOW.md row 7 holds Stage T for the K16 outcome, and row 8 runs the float-credit sheet only if W2 blames the learning law (H_law). W2 is done: H_mixed (#3177, `docs/labs/m1-w2-capacity-capture/README.md`), so the sheet waits on the code-utilization diagnostic and on a base that passes Q1 (the A8 full dose); see the owner log on #2835.
2. Skim the code the sheet will name: `crates/uor-r4-discrete/src/bvgk_block.rs`, `bvgk_device.rs`, `bvgk_qualification.rs` and `bvgk_source.rs`. Run the host checks with `~/.cargo/bin/cargo check -p uor-r4-discrete`. If the crate does not build without nvcc on your machine, say so on the issue; do not start a GPU run.
3. First deliverable, posted on the issue: a short design note for Stage T. It states the geometric construction (K coordinates scored per launch, a block committed only if `l_joint < l_best`), why it uses no softmax or squashing and no matmul (integer hinge loss, table moves, adds), and the finite controls that already pass. It also says how the three uncovered items above are covered: as extra finite controls before the run, or as checks inside it.
4. Draft the Stage T sheet in `docs/harness/prep/` (for example `m1-bvgk-stage-t-training-<date>.md`). It needs: the data and panel with hashes; the two seeds; the arms; the comparators (unigram, then the admission-matched next-word table); the numeric stop rules and bar; the cost cap; the lease and GPU count; the pre-run review it will run under (D67); and "what this leaves uncovered". Leave any number the sources do not give as "the bar is not yet registered", and settle it in the lab's own decision review (D68). No owner release is needed: the run goes ahead on its FIT and the lab's pre-run review (D67).
5. Request a non-author FIT review on the owning issue. Do not launch anything.
6. Open a PR with the sheet that says `Closes #<this issue>` (or `References` if the sheet needs another round), and add one row to `docs/research/notebook/m1-discrete-attention.md` (the M1 notebook page) that records C2 PASS (the receipt SHA-256 above) and the sheet as the next step.

## Rules that apply

- **Geometry only** (CHARTER §3.1–§3.3): every mechanism, training signal and fix comes from the project's geometry. No softmax or squashing anywhere, including losses, gradients, sampling and renamed or table forms. No dense matrix multiplication or GEMM, including wrappers and hand loops.
- **GPU only** (§3.5): training, evaluation and qualification run on a GPU, never a CPU. Outside contributors may run on their own GPU and must attach the hardware specs to the PR. If the hardware is not equivalent to the project baseline (an RTX 5090), a maintainer re-runs it after the PR, under the same FIT (owner, 20 Oct 2026).
- **Pre-registration first** (§4.5): a GPU training or evaluation run, and any run a milestone claim rests on, needs a one-page sheet with a non-author TEST FITNESS: FIT review before it runs.
- **Rust first** (§3.11): new code is Rust, and no new Python is added.
- **Records** (§4.1, §5.6): each result is one row in its line's research notebook (`docs/research/notebook/<line>.md`). A negative result closes a configuration, never a mechanism.

## Sources

- [C2 qualification record](../../../docs/labs/injective-preflight-and-c2-qualify-2026-10-19/README.md) and [C2 sheet](../../../docs/harness/prep/m1-bvgk-stagec2-block-qualification-2026-10-18.md)
- [Stage T record](../../labs/m1-bvg-k-stage-t-training/README.md) for the completed sheet, reviews, Step 0 controls and two-seed diagnostic-stop outcome
- [Current project direction](../../start/where-were-going.md)
- [Historical `NOW.md` snapshot](../../../archive/integration/NOW-2026-10-22.md), retained as pre-registration history
