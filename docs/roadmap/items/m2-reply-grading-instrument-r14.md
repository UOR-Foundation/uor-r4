---
id: m2-reply-grading-instrument-r14
title: Build the instrument that R14 says reply grading lacks
milestone: M2
area: evaluation
status: ready
open_to: anyone
needs_gpu: false
depends_on: []
---

## Why it matters

M2 and Stage 5 ask for a first simple conversation graded by fixed deterministic checks ([where we are going](../../start/where-were-going.md)). Rule R14 voids every reply target that rests on an instrument that fails on evidence, so the 116/232 reply bar is BLOCKED and is not a closing condition. Without a reply-grading instrument that passes R14, a conversation bar cannot be scored, and no reply number can bind anything.

## What is missing

The `#2029` body says the 116/232 reply bars are BLOCKED under R14 with "no instrument" (v5-reply stood at 37/232). R14 (`docs/harness/pm-rules.md`, section R14) voids a target when its instrument is vacuous (a constant answer clears half the panel), confounded (a copy control ties the model) or noisy beyond the bar. It names what must exist before any reply target: a v2 check table with frozen positive controls, per-category best-constant pairing and abstention-as-discrimination scoring. Even then, a target is set from measured mechanism behaviour, not picked. The qwen-graded 116/232 target is banned under D28 (neural grader), and the 88-row deterministic re-score is vacuous: a constant canned reply scores 40/88.

A v2 check table (`crates/uor-r4-training/fixtures/reply-panel-checks-v2.tsv`, 97 lines) and a grader (`crates/uor-r4-training/src/bin/chat-grade.rs`) already exist in the tree. The sources do not say which of the three R14 requirements they already meet, so scoping that is the first job. The bar is not yet registered.

## Done when

A fixed deterministic grading instrument for replies exists, with a recorded scope note on what R14 required and how this instrument meets it. The note shows the three R14 requirements (frozen positive controls, per-category best-constant pairing, abstention-as-discrimination scoring) each met on evidence, and a constant-reply control that no longer clears the panel.

## How to start

1. Read R14 in `docs/harness/pm-rules.md` (and R12, which it cites), `docs/integration/project-track.md` (the BLOCKED paragraph) and the `v5-reply` entry in `docs/harness/panels/v5-reply.json`. Also check `docs/harness/panels/m2-reply512.json` for the M2 panel definition. `docs/integration/DECISIONS.md` D39 now only points into the charter; the owner's M1 closing bar is in CHARTER §4.8.
2. Open the existing instrument: `crates/uor-r4-training/src/bin/chat-grade.rs`, `crates/uor-r4-training/fixtures/reply-panel-checks-v2.tsv` and `data/panels/README.md`. List which of the three R14 requirements each already meets, using the audit findings quoted in `v5-reply.json`.
3. Post the first deliverable on `#2029`: a scope note with that list. It also names the frozen positive controls and the constant and copy controls that will be run, and says how the panel and checks are frozen and hashed (add them to `data/panels/MANIFEST.sha256` if they change).
4. Build the missing parts in Rust inside `chat-grade.rs` or a sibling module under `crates/uor-r4-training/src/`: finite tests first (`~/.cargo/bin/cargo test -p uor-r4-training`), each control as a test that must fail without the new rule. Grading must stay string checks and exact counts: no neural grader (D28), no softmax, no matmul.
5. Run the instrument on CPU against the existing replies and controls, and record the constant-reply and copy-control results. Ask a non-author to review the scope note and the instrument before any bar is proposed. Registering a bar is a decision card to the PM or owner, not part of this item.
6. Open a PR that says `References #2029` (it closes this item only if the owner accepts the instrument). Record the result as one row in the closest research notebook under `docs/research/notebook/` (check `docs/research/notebook/README.md`; `harness-governance.md` is the likely home for instrument rows).

## Rules that apply

- **Geometry only** (CHARTER §3.1–§3.3): every mechanism, training signal and fix comes from the project's geometry. No softmax or squashing anywhere, including losses, gradients, sampling and renamed or table forms. No dense matrix multiplication or GEMM, including wrappers and hand loops.
- **Rust first** (§3.11): new code is Rust, and no new Python is added.
- **Records** (§4.1, §5.6): each result is one row in its line's research notebook (`docs/research/notebook/<line>.md`). A negative result closes a configuration, never a mechanism.

## Sources

- [`docs/start/where-were-going.md:52`](../../../docs/start/where-were-going.md)
- issue#2029
- [`docs/harness/pm-rules.md`](../../../docs/harness/pm-rules.md) (R14)
- [`docs/harness/panels/v5-reply.json`](../../../docs/harness/panels/v5-reply.json)
- [`crates/uor-r4-training/src/bin/chat-grade.rs`](../../../crates/uor-r4-training/src/bin/chat-grade.rs)
