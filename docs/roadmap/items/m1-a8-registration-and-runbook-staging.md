---
id: m1-a8-registration-and-runbook-staging
title: Register A8 finite qualification and stage the dose runbook
milestone: M1
area: tooling
status: ready
open_to: anyone
needs_gpu: false
depends_on: []
---

## Why it matters

Nothing in A8 can run until the finite qualification is registered and the owner has a staged release command to type. This is host and paperwork work, with no GPU. It sits ahead of the A8 rerun in the discrete-attention line; see [where we are](../../start/where-we-are.md).

## What is missing

[`NOW.md`](../../integration/NOW.md) names two items as current work: staging the audited-dose runbook and executor, and a separate prospective finite-qualification registration. Registration must keep the immutable scientific FIT sheet identity. PR #2785 tracks sheet restoration and the separate qualification binding. The admission controller and finite qualification producer (#2782) and the host consumer verifier (#2783) are already on main. Real CUDA qualification, restoration and timing remain NOT_RUN. The A8 qualification and dose keep the sheet-bound source `fd409fcdf`, not later main.

## Done when

#2785 merged; finite-qualification sheet registered with its own binding and the original FIT sheet identity unchanged; runbook and executor staged; the exact owner-release command staged for Casey to type.

## How to start

1. Read `docs/integration/NOW.md` lines 18-34, then the three lab pages: `docs/labs/admission-anchor-controller-2026-10-19/README.md`, `docs/labs/address-cells-consumer-verifier-2026-10-19/README.md` and `docs/labs/a8-training-configs-2026-10-19/README.md`. Check PR #2785 with `gh pr view 2785` and note what is still open.
2. Post a one-line intent note on the issue. First deliverable: a short plan that lists the sheet identity (file and hash) that must not change, and the new binding the finite-qualification sheet needs.
3. Draft the finite-qualification registration as a one-page sheet with its own binding. It must pin source `fd409fcdf` and state the bounded-update and restoration controls. The bar is not yet registered here; take it from the sheet-bound A8 FIT and say where it came from.
4. Check that the original FIT sheet identity is unchanged: run `~/.cargo/bin/cargo run -q -p uor-r4-harness -- check` (or `uor-harness check`) and compare the sheet hash before and after.
5. Stage the audited-dose runbook and executor as files in the A8 lab folder. The finite stage needs its own FIT, costed owner approval and PM dispatch; full dose is a separate release.
6. Stage the exact owner-release command in the runbook for Casey to type. Do not type it. The owner types one release per pre-registered sheet (CHARTER section 5.1).
7. Open a PR that says `Closes #<this issue>` (or `References` if #2785 is still open), and add one row to `docs/research/notebook/m1-discrete-attention.md`.


## Rules that apply

- **Geometry only** (CHARTER §3.1–§3.3): every mechanism, training signal and fix comes from the project's geometry. No softmax or squashing anywhere, including losses, gradients, sampling and renamed or table forms. No dense matrix multiplication or GEMM, including wrappers and hand loops.
- **Rust first** (§3.11): new code is Rust, and no new Python is added.
- **Records** (§4.1, §5.6): each result is one row in its line's research notebook (`docs/research/notebook/<line>.md`). A negative result closes a configuration, never a mechanism.

## Sources

- [`docs/integration/NOW.md:18-34`](../../../docs/integration/NOW.md)

