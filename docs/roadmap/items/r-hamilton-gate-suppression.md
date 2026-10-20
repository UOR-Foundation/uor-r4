---
id: r-hamilton-gate-suppression
title: Settle whether a norm-preserving Hamilton gate can suppress a signal
milestone: research
area: mechanism
status: ready
open_to: anyone
needs_gpu: false
depends_on: []
---

## Why it matters

The Hamilton gate is the planned feed-forward gate in the Stage 3 trunk. It replaces the sigmoid gate that the rules now ban. A sigmoid gate can shut a channel, and a unit rotation cannot, so this question decides whether the gate can do that job at all. See [the Hamilton gate page](../../mechanisms/hamilton-gate.md).

## What is missing

The claim that a norm-preserving Hamilton gate can suppress a signal is a **hypothesis**, "a reading of the dossier only" (`docs/research/open-questions.md`). The mechanism page says: "This is a reading of the formula; the records do not discuss it." Nothing in the sources shows it was checked.

What the page already derives from the formula (**proof**): with `r = sqrt(|g|^2 + 1e-12)` and `q = g/r`, the output length is `|q||u|`, so it equals `|u|` unless the gate block is near zero. The factor is 0.71 at `|g| = 1e-6`, and a zero gate block gives zero output. What is not settled:

- A definition of "suppress". The bar is not yet registered.
- Whether a near-zero gate block counts as a usable off state, given the backward factor `1/r` of about 10^6 there (**proof**, from the formula; no real run has checked it).
- Whether steering the direction of a block, without changing its length, counts as suppression for a later layer.

## Done when

A proof or finite test that settles whether such a gate can suppress a signal, labelled **proof** or **measurement** (see `docs/formal_vocabulary.md`). The definition of "suppress" is registered with it, and the result is one row in a research notebook.

## How to start

1. Read the "How it works" and "What has never been tested" sections of `docs/mechanisms/hamilton-gate.md`, then `docs/formal_vocabulary.md` and `docs/geometry/quaternions-and-rotation.md`. In `crates/uor-r4-training/src/geometric_stack.rs`, read `hamilton_product` (line 18699), the `HamiltonGate` op with its CPU forward (line 18711) and `hamilton_gate` (line 18828).
2. First deliverable, posted on the issue: a design note that defines "suppress" before any test. It must pick one definition and say why. Candidates: (a) the per-block ratio `|out|/|u|` over all gate blocks; (b) whether a downstream read can lose all contribution from a block. The note must also say what result would count as a yes or a no, and what the `1e-12` guard and the `1/r` hazard change.
3. Do the proof on paper under that definition: the exact range of `|out|/|u|` over all `g`, and the smallest `|g|` for which the ratio falls below the threshold you chose. Label it **proof**. Check the result against the f32 arithmetic of `hamilton_gate`, since rounding matters near `|g| = 1e-6`.
4. Add a finite test, host only, with no matmul and no softmax, in the `mod tests` block of `geometric_stack.rs` (a local `hamilton` helper already sits near line 35248). Sweep `|g|` over decades from 1e-9 to 1e3 with several fixed gate and value directions. Assert that the measured ratio matches the formula within f32 tolerance, and print the smallest ratio reached. This is a host check, not a training run. Run `~/.cargo/bin/cargo test -p uor-r4-training --lib <your test name>`. For the sibling parity tests, see `test_hamilton_gate_parity` in `crates/uor-r4-training/tests/cuda_stack_ops_parity.rs`.
5. State what follows from the result. If the gate cannot suppress except at a near-zero block, say so as a **proof** and note that a negative closes a configuration, never a mechanism (D22). The training sheet in the roadmap item `hamilton-gate-offline-float-sheet` would build on this answer.
6. Open a PR that says `Closes #<this issue>`. Update the claim row at `docs/research/open-questions.md:97` and the matching bullet in `docs/mechanisms/hamilton-gate.md`. Record the result as one row in the research notebook. No notebook page covers the feed-forward gate yet (`docs/research/notebook/` has data-tokenizer, harness-governance, m1-discrete-attention, m1-prime-zeta, m3-durable-memory, m4-serving), so ask a maintainer on the issue which page to use or to open one.

## Rules that apply

- **Geometry only** (CHARTER §3.1–§3.3): every mechanism, training signal and fix comes from the project's geometry. No softmax or squashing anywhere, including losses, gradients, sampling and renamed or table forms. No dense matrix multiplication or GEMM, including wrappers and hand loops.
- **Rust first** (§3.11): new code is Rust, and no new Python is added.
- **Records** (§4.1, §5.6): each result is one row in its line's research notebook (`docs/research/notebook/<line>.md`). A negative result closes a configuration, never a mechanism.

## Sources

- [`docs/research/open-questions.md:97`](../../../docs/research/open-questions.md)
- [`docs/mechanisms/hamilton-gate.md`](../../../docs/mechanisms/hamilton-gate.md)
