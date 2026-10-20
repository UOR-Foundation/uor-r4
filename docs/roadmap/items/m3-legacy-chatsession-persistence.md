---
id: m3-legacy-chatsession-persistence
title: Retire or complete legacy width-576 ChatSession persistence
milestone: M3
area: tooling
status: ready
open_to: anyone
needs_gpu: false
depends_on: []
---

## Why it matters

Clears a tracked defect so session persistence has one tested path.

## What is missing

The legacy ChatSession (uor-r4.integer-session/1) does not serialize its three width-576 value stores; restored_dialogue576_session_continues_identically in capability_api_tests.rs is ignored. If reactivated, serialize the stores; otherwise retire the ignored test in favour of the IntegerStackSession reload gate (#1476).

## Done when

Either the three value stores serialize and the test passes, or the ignored test is retired in favour of the IntegerStackSession reload gate.

## How to start

Open these first:

- `crates/uor-r4-integer/tests/capability_api_tests.rs`
- `crates/uor-r4-integer/src/session_codec.rs`

## Rules that apply

- **Geometry only** (CHARTER §3.1–§3.3): every mechanism, training signal and fix comes from the project's geometry. No softmax or squashing anywhere, including losses, gradients, sampling and renamed or table forms. No dense matrix multiplication or GEMM, including wrappers and hand loops.
- **Rust first** (§3.11): new code is Rust, and no new Python is added.
- **Records** (§4.1, §5.6): each result is one row in its line's research notebook (`docs/research/notebook/<line>.md`). A negative result closes a configuration, never a mechanism.

## Sources

- issue#2031 (folds #1476)

