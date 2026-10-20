---
id: m3-qualify-memory
title: Qualify durable memory on the selected native model
milestone: M3
area: evaluation
status: ready
open_to: lab
needs_gpu: true
depends_on: [m2-train-grounded-replies, m3-memory-turn-discriminator-gate, m3-policy-precedence-and-sieve-tests]
---

## Why it matters

M3 closes the durable-conversation-memory claim.

## What is missing

m3-memory is 0/1 qualified. The evaluator and session exist (stack_grounded_session, typed-script multi-turn evaluator, IntegerStackSession, UR4SESS1) but have never run on a trained M1 or M2 model. Status is paused.

## Done when

Provenance, contradiction and correction, abstention on unstated or retracted facts, explicit reset and process restart pass on the selected artifact; saved and restored sessions continue identically token for token (IntegerStackSession plus UR4SESS1).

## How to start

Open these first:

- `crates/uor-r4-integer/src/session_codec.rs`
- `stack_grounded_session.rs`

## Rules that apply

- **Geometry only** (CHARTER §3.1–§3.3): every mechanism, training signal and fix comes from the project's geometry. No softmax or squashing anywhere, including losses, gradients, sampling and renamed or table forms. No dense matrix multiplication or GEMM, including wrappers and hand loops.
- **GPU only** (§3.5): training, evaluation and qualification run on a GPU, never a CPU. Outside contributors may run on their own GPU and must attach the hardware specs to the PR. If the hardware is not equivalent to the project baseline (an RTX 5090), a maintainer re-runs it after the PR, under the same FIT (owner, 20 Oct 2026).
- **Pre-registration first** (§4.5): a GPU training or evaluation run, and any run a milestone claim rests on, needs a one-page sheet with a non-author TEST FITNESS: FIT review before it runs.
- **Rust first** (§3.11): new code is Rust, and no new Python is added.
- **Records** (§4.1, §5.6): each result is one row in its line's research notebook (`docs/research/notebook/<line>.md`). A negative result closes a configuration, never a mechanism.

## Sources

- [`ROADMAP.md:16`](../../../ROADMAP.md)
- [`STATUS.md:18`](../../../STATUS.md)
- issue#2031

