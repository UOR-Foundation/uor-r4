---
id: m3-policy-precedence-and-sieve-tests
title: Pin OpPolicy precedence and store-miss sieve coupling with tests
milestone: M3
area: tooling
status: ready
open_to: anyone
needs_gpu: false
depends_on: []
---

## Why it matters

Removes ambiguity in memory behaviour before M3 qualification.

## What is missing

No explicit unit tests assert the documented precedence of OpPolicy::UnlessQuery versus OpPolicy::TableStatements when the table and op-model disagree (relation_compiler.rs). In stack_grounded_session.rs, a query that resolves to a relation id and misses the store returns StoreRead::Absent ('Memory: none.') without consulting the conversation log sieve, which runs only on MemoryEffect::Unresolved; the intended coupling is undocumented and untested (#1738).

## Done when

Unit tests for both precedences and a documented, tested store-miss and sieve behaviour.

## How to start

Open these first:

- `relation_compiler.rs`
- `stack_grounded_session.rs`

## Rules that apply

- **Geometry only** (CHARTER §3.1–§3.3): every mechanism, training signal and fix comes from the project's geometry. No softmax or squashing anywhere, including losses, gradients, sampling and renamed or table forms. No dense matrix multiplication or GEMM, including wrappers and hand loops.
- **Rust first** (§3.11): new code is Rust, and no new Python is added.
- **Records** (§4.1, §5.6): each result is one row in its line's research notebook (`docs/research/notebook/<line>.md`). A negative result closes a configuration, never a mechanism.

## Sources

- issue#2031 (folds #1738)

