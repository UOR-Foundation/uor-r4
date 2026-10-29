---
id: fbs-serving-kernels
title: Split each served FBS layer, then vector kernels exact against the scalar step
milestone: research
area: serving
status: ready
open_to: lab
needs_gpu: false
depends_on: [fbs-cost-profile]
done_class: host
notebook: docs/research/notebook/float-training.md
scope: [crates/uor-r4-integer/src/stack/session.rs, crates/uor-r4-integer/src/stack/simd.rs, crates/uor-r4-integer/src/stack/kernels.rs, crates/uor-r4-integer/src/stack/tests.rs, crates/uor-r4-integer/src/bin/fbs-toks.rs, docs/loop/tasks/fbs-serving-kernels/**, docs/research/notebook/float-training.md, docs/roadmap/items/fbs-serving-kernels.md]
---

## Why it matters

The owner, 28 Oct 2026, in claude's chat: "I don't care what we serve at long as it is cheap". This is the [pipeline map](../../model/pipeline-map.md)'s R3 and its gap 3.

On the M1, a served token is 3.73 ms (265 tok/s). The six layers are 98.6% of it: each layer's recurrence block takes 352–364 µs and its feed-forward block 253–259 µs ([claude-158](../../loop/tasks/fbs-cost-profile/claude-158/REPORT.md)). That is about 57 ns per learnable value, in scalar integer code with no vector instructions.

## What is missing

- **The layer split.** Each block holds three route gathers (`stack_fbs_gather`) and either the recurrence core or the gated product and requantization. Which part dominates is not measured.
- **Vector kernels for the FBS step.** `stack/simd.rs` holds kernels for an older frozen stack (the map, stage 11). None run in `stack_step_fbs`.
- **An exactness check for a faster step.** claude-158's `measure_fbs_step_stages` already runs the served kernels with a clock, and its host test holds the twin equal to `step_native`. A faster step must pass the same equality.

## Done when

1. **The split:** each layer's gathers, core, gated product and requantization timed on the M1, by extending `measure_fbs_step_stages` and `fbs-toks --stages`. The parts sum to the layer within 10%.
2. **A vector kernel for the largest part,** written for D11 (CHARTER §3.6: integer, no multiply instruction; products through its tables), behind a switch.
   - A host test holds every token, kept cell, query and session state equal to the scalar `step_native` over a synthetic trunk and a stored artifact.
   - The zero-multiply audit of `step` still passes.
3. **Speed:** `fbs-toks` decode tokens/s on the M1 with and without the kernel, on claude-126's export, with the same replies.
4. One notebook row and a REPORT. This is host and serving work: no GPU and no sheet (CHARTER §4.5).

## How to start

Read:
- [claude-158's REPORT](../../loop/tasks/fbs-cost-profile/claude-158/REPORT.md);
- `stack_step_fbs`, `stack_step_fbs_timed`, `stack_fbs_gather` and `stack_recurrence_core` in `crates/uor-r4-integer/src/stack/session.rs`;
- `crates/uor-r4-integer/src/stack/simd.rs`.

The artifact is claude-126's seed-1 export (sha256 `314c3909…fdb305`, in claude-158's REPORT). Build `fbs-toks` with `--features serving-wasm`.

## Rules that apply

- **Serving stays integer and multiplier-free** (D11, CHARTER §3.6). `step` and its call graph keep passing the zero-multiply audit.
- **Exact or nothing:** a faster kernel that changes one token, cell or state is a defect, not a trade.
- **No dense matrix multiplication** (§3.3), including in vector form.

## Sources

- The owner's words, 28 Oct 2026, in claude's chat (quoted above).
- [claude-158](../../loop/tasks/fbs-cost-profile/claude-158/REPORT.md) and the [pipeline map](../../model/pipeline-map.md).
