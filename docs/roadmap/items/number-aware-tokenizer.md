---
id: number-aware-tokenizer
title: Train a number-aware tokenizer and pin its identity
milestone: M1
area: tooling
status: ready
open_to: lab
needs_gpu: false
depends_on: []
---

## Why it matters

It closes the tokenizer dossier's never-tested list and gives the vocabulary comparison a second variant.

## What is missing

The tokenizer dossier lists a number-aware tokenizer as never tested, and none was ever trained. Only d36d3e87 (4,096 pieces) and a 16,384-piece trainer exist.

## Done when

A number-aware vocabulary is trained with the Rust BPE trainer, pinned by identity hash, and included in the tokenizer comparison.

## How to start

Open these first:

- `docs/labs/rust-bpe-trainer-2026-10-19/README.md`
- `docs/mechanisms/tokenizer.md`

## Rules that apply

- **Geometry only** (CHARTER §3.1–§3.3): every mechanism, training signal and fix comes from the project's geometry. No softmax or squashing anywhere, including losses, gradients, sampling and renamed or table forms. No dense matrix multiplication or GEMM, including wrappers and hand loops.
- **Rust first** (§3.11): new code is Rust, and no new Python is added.
- **Records** (§4.1, §5.6): each result is one row in its line's research notebook (`docs/research/notebook/<line>.md`). A negative result closes a configuration, never a mechanism.

## Sources

- [`docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md:13`](../../../docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md)
- [`docs/mechanisms/README.md:95`](../../../docs/mechanisms/README.md)

