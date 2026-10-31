---
id: integer-long-context
title: Serve prompts and conversations past the 192-token window
milestone: research
area: serving
status: ready
open_to: lab
needs_gpu: true
depends_on: [integer-hamming-read]
notebook: docs/research/notebook/m4-serving.md
scope: [crates/uor-r4-integer/src/stack/session.rs, crates/uor-r4-integer/src/stack/mod.rs, crates/uor-r4-integer/src/stack/tests.rs, crates/uor-r4-integer/src/stack/synthetic.rs, crates/uor-r4-integer/src/bin/fbs-toks.rs, docs/model/pipeline-map.md, docs/harness/prep/integer-long-context*, docs/roadmap/items/integer-long-context.md, docs/loop/tasks/integer-long-context/**, docs/research/notebook/m4-serving.md]
---

## Why it matters

The pipeline map's gap 6: "The serving session cannot slide its 192-token window ... Long context, the geometry's main claim, cannot be served yet." A conversation of a few turns passes 192 tokens, so a served model can hold no real conversation until this is solved.

**What is true now:**
- **The integer session stops.** Every served step returns `StackError::ContextFull { context }` once the session holds its `context` tokens (`stack/session.rs`). `fbs-toks` times a 128-token prefill and 64 decode steps, which is all 192.
- **Training sees 192-token blocks.** It cuts each document's targets into back-to-back chunks of 192 from its `<|bos|>`, and a row of a full chunk sees from that chunk's start (`hamilton_sheet.rs:1351–1360`, as `turn-memory-integer` records it).
- **The float greedy decode slides** over the last 192 tokens (`in_space_long.rs:1433`), and reads no reply table once the prompt has slid out (`turn-memory-integer`).
- So the trained model, its float decode and its integer session follow three different rules past 192.

The map's "the integer runtime has no look-back" is out of date: the Hamming read's copy and type heads are served (claude-194, claude-195). Their parity with the float read is [`integer-hamming-read`](integer-hamming-read.md)'s, which this item depends on.

## What is missing

**A rule for what comes after 192, chosen by measurement.** Each candidate changes cost or the model:
- **(a) Slide.** Re-run the last 192 tokens for each new token, as the float decode does. Exact against it, but it costs a 192-token prefill per token.
- **(b) Blocks, as training cuts them.** Reset at each 192 from `<|bos|>`. Exact against training, but the reply loses everything before its block.
- **(c) Carry the state.** The FBS trunk is a recurrence, so its state can run on. The Hamming read looks back over a bounded set of positions. The model has never been trained or scored this way, so the rule needs a training arm that sees longer windows.

The trunk's recurrence makes (c) the geometry's own answer, which is why long context is "the geometry's main claim". The item measures before choosing.

## Done when

1. **The decision,** recorded by a decision review on this item's issue (`[claude] owner log:`): which rule, and why, read from the measurements in 2.
2. **The measurements,** under a prep sheet with a fresh FIT and a pre-run review: on one trained checkpoint, the float model's held-out loss and greedy reply agreement past 192 tokens under each rule, on multi-turn prompts whose context passes 192. A rule needing a new training arm gets that arm.
3. **The integer session serves the chosen rule,** with a test against the float forward's rule past 192, and a negative gate. A session within 192 tokens decodes byte for byte as at the base.
4. **Tokens per second against context** out to at least 2,048 tokens, with the resident memory.
5. **One notebook row, and a REPORT with the negatives first.** The pipeline map's gap 6 records the result.

## How to start

- **Read:** `stack/session.rs` where `ContextFull` is returned; `in_space_long.rs` near line 1433 (the slide); `hamilton_sheet.rs:1351–1360` (training's blocks); [`turn-memory-integer`](turn-memory-integer.md)'s notes on the window; the pipeline map's stages 5 and 8.
- **Before claiming:** `integer-hamming-read`, `turn-memory-integer`, `point-key-integer` and `integer-nearest-readout` all edit `stack/session.rs`. Check that no open order holds it.

## Rules that apply

- **GPU-only research compute** (CHARTER §3.5): the float measurements run on the GPU. Integer decodes on the laptop are product serving (§3.6).
- **Serving stays integer** (§3.6, as amended by D80 and D81).
- **Lead with tokens per second** against context.
- **A negative closes a configuration, never a mechanism** (§4.1).

## Sources

- `docs/model/pipeline-map.md`, gap 6 and stages 5 and 8.
- [`turn-memory-integer`](turn-memory-integer.md), its notes on training's blocks and the float decode's slide.
- `crates/uor-r4-integer/src/stack/session.rs`, `StackError::ContextFull`.
