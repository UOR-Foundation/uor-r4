---
id: m1-w2-capacity-capture
title: Run the W2 read-code capacity capture
milestone: M1
area: evaluation
status: done
open_to: lab
needs_gpu: true
depends_on: []
scope: [crates/uor-r4-discrete/**, docs/harness/prep/m1-w2-capacity-capture*, docs/labs/m1-w2-capacity-capture/**, docs/loop/tasks/m1-w2-capacity-capture/**]
---

## Why it matters

Routes all later spend: H_rep means fix the read and address metric and do not fund float training; H_law means run BVG-K (W3) and, if it fails, W4.

## What is missing

**Outcome: complete.** W2's checkpoint-restricted evaluation is `T = 3,414`, `G = 6,300`, `W = 149`: H_mixed. Its read-code table is informative beyond the writer but not beyond the bigram. The follow-up code-utilization diagnostic is also complete: QA A1a, QB B2 and QC C1; code-count admission 9,064 against the writer's 1,920; code × query 7,262 against the bigram's 6,300; and the registered data-scaling projection 3,552 below the H_law line, so G1 was declined ([diagnostic record](../../labs/m1-w2-code-utilization/README.md)). The separate Stage T follow-up reached `T_restr(A_W) = 2,140` and `G_restr(A_W) = 3,084` on `T16_fz`, both above unigram 1,226, and identifies representation/read-code and within-cell ranking as separate constraints. Cost is recorded in the Stage T report, not the old $0.91 estimate.

## Done when

**Original acceptance text, retained as history:** the registered W2 sheet runs once with all arms; H_rep or H_law is recorded as one notebook row, and the admission-matched next-word table number (needed for the Stage 2 bar) is produced.

**Observed outcome:** the run completed and recorded H_mixed instead of either anticipated classification. This label preserves the original acceptance text rather than inventing a new bar; the follow-up outcomes above are dated evidence, not a rewritten acceptance rule.

## How to start

Open these first:

- `docs/integration/DECISIONS.md`
- `PLAN.md` and rendered `STATE.md`
- `docs/labs/course-correction-2026-10-18/audits/d37-reactivation-architect.md`
- `docs/labs/k16-outcome-call-2026-10-19/README.md`
- `docs/research/open-questions.md`

## Rules that apply

- **Geometry only** (CHARTER §3.1–§3.3): every mechanism, training signal and fix comes from the project's geometry. No softmax or squashing anywhere, including losses, gradients, sampling and renamed or table forms. No dense matrix multiplication or GEMM, including wrappers and hand loops.
- **GPU only** (§3.5): training, evaluation and qualification run on a GPU, never a CPU. Outside contributors may run on their own GPU and must attach the hardware specs to the PR. If the hardware is not equivalent to the project baseline (an RTX 5090), a maintainer re-runs it after the PR, under the same FIT (owner, 20 Oct 2026).
- **Pre-registration first** (§4.5): a GPU training or evaluation run, and any run a milestone claim rests on, needs a one-page sheet with a non-author TEST FITNESS: FIT review before it runs.
- **Runs on a pre-run review** (D67, §5.1): a GPU or training run goes ahead when the lab's own fresh pre-run review passes, inside the GPU caps. A registered sheet still needs its non-author FIT. Preparation that runs no compute (code, finite CPU tests, the pre-registration sheet) can proceed now.
- **Rust first** (§3.11): new code is Rust, and no new Python is added.
- **Records** (§4.1, §5.6): each result is one row in its line's research notebook (`docs/research/notebook/<line>.md`). A negative result closes a configuration, never a mechanism.

## Sources

- [W2 capacity-capture record](../../labs/m1-w2-capacity-capture/README.md)
- [Completed code-utilization diagnostic](../../labs/m1-w2-code-utilization/README.md)
- [Stage T follow-up record](../../labs/m1-bvg-k-stage-t-training/README.md)
- [Historical `NOW.md` snapshot, line 65](../../../archive/integration/NOW-2026-10-22.md) and [19 October pre-run audit](../../../docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md)

