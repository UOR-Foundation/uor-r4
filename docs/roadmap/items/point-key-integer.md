---
id: point-key-integer
title: Serve the point key's opening in the integer artifact on the laptop
milestone: research
area: serving
status: ready
open_to: lab
needs_gpu: true
depends_on: [one-model-end-to-end, turn-memory-integer]
notebook: docs/research/notebook/float-training.md
scope: [crates/uor-r4-integer/src/stack/session.rs, crates/uor-r4-integer/src/stack/format.rs, crates/uor-r4-integer/src/stack/memory.rs, crates/uor-r4-integer/src/stack/mod.rs, crates/uor-r4-integer/src/stack/point_key.rs, crates/uor-r4-integer/src/fbs_snapshot.rs, crates/uor-r4-float-train/src/fbs_export.rs, crates/uor-r4-float-train/src/bin/fbs-snapshot.rs, docs/roadmap/items/point-key-integer.md, docs/harness/prep/point-key-integer*, docs/labs/point-key-integer/**, docs/loop/tasks/point-key-integer/**, docs/research/notebook/float-training.md]
---

## Why it matters

The project's first hurdle is one working model end to end: token → vector → vector → integer → token (`PLAN.md`). The point key is the float model's working opening, and the integer artifact does not serve it yet.

- **claude-219 found the point key.** At a reply's opening where the turn memory's exact first-8 key misses, the trunk's opening point is named by its L1-nearest TRAIN openings.
- **claude-220 serves it inside the float model's decode** ([REPORT](../../loop/tasks/one-model-end-to-end/claude-220/REPORT.md), merged as `c3d5cb684`, #3443). Its registered outcome is row 2: the served key carries the reply at all three seeds on the fresh 1,200-conversation panel p-w.
  - Templates against greedy: (50, 9), (44, 4), (47, 5), against bands of 15.4, 13.9 and 14.4.
  - Every served token is the measured key's name, with a distance drift of 0.0.
  - The negatives: the gain is recall of TRAIN replies, and it closes about 18% of the gap to a forced right first token.

The integer artifact cannot hold the float table, since no floating point is allowed in discrete-model computation (D32). claude-220's decision review set two conditions for its integer form:
- **(b)** its bar is measured with the integer trunk's own points, for both the query and the table, never with float points quantized after the fact;
- **(d)** the export needs its own item. This is that item.

## What is missing

- **A table built by the integer engine.**
  - It holds TRAIN's opening points at the writer query's representation (i16 Q12: 212,968 × 256 × 2 B, about 109 MB) and their targets.
  - It is taken with the memory off, as an export step.
  - claude-220's float `opening-table.safetensors` is never quantized into it.
- **The served search in the integer session.**
  - **Where:** at `stack_step_fbs`'s first reply step, and its timed twin. Where the turn memory's opening read misses its exact key, the memory moves nothing, so `b.query_q12` is the trunk's own point. The search overrides the native token there.
  - **How:** the `-L1` scan of `stack_whole_table_nearest`, plus a second pass for the commonest target among all rows at the least distance, ties to the smaller id. That is float's `l1_named` rule.
  - **The miss record** (`moved`) is kept in every build, not only under test.
- **The artifact:** `uor-r4.lut-stack/8`, or a new optional record that older engines refuse, since `StackTurnMemory` denies unknown fields.
  - The new section is `memory.point.{points,targets}`.
  - The restore refusal covers its state.
- **Optionally, the sign code** (6.8 MB, the Hamming form that `integer-hamming-read`'s popcount fits). It is reported only, since claude-219 read it too thin to tell.

## Done when

1. **The integer table and search are built and unit-tested.**
   - The search names a missed opening by the commonest target at the least L1 distance, ties to the smaller id.
   - An opening the exact key fires on is left to the memory.
   - The table round-trips through the artifact byte for byte.
2. **Parity, under a prep sheet with a fresh FIT and a pre-run review:**
   - **Names:** on p-w, p-v and DEV, the integer served names against claude-220's float served names (its `samples-{split}-served-memory.json`). They are counted at every opening the exact key misses, and each difference is classed as a rounding near-tie or a fault.
   - **Replies:** the integer model's replies against the float served model's, on the same prompts.
3. **Tokens per second first:** the integer decode against context on the laptop, with the table search counted, and its resident memory. Serving the exported artifact on a client is the product (CHARTER §3.6).
4. **Each result is one notebook row, and each REPORT gives the negatives first.**

## How to start

- **Read:**
  - claude-220's REPORT and DECISION.md (conditions (b) and (d));
  - `turn-memory-integer`'s item, and DeepSeek's integer turn memory (#3446, the `/7` turn memory in the integer session);
  - `crates/uor-r4-integer/src/stack/{memory.rs,session.rs,format.rs}` at the plug-in point.
- **Before claiming:** check that neither `integer-hamming-read` nor `fbs-serving-kernels` holds a claim on `session.rs`, and that no Studio worktree plans edits to `session.rs` or `wasm_serve.rs`.
- **Settle in the sheet:**
  - where the table is built (an integer export pass over TRAIN's openings) and what it costs;
  - whether the parity evaluation runs on the laptop (integer serving) or must run on the GPU (CHARTER §3.5), against the earlier integer units' practice.

## Rules that apply

- **Serving stays integer** (CHARTER §3.6, as amended by D80 and D81): no float or transcendental instruction. D32 applies to the table and its search: integer only, with adds, subtracts, absolute values and compares.
- **Geometry only, no squashing** (§3.1–§3.2).
- **Deterministic exact or membership checks only,** with no neural graders (§3.10, §4.4).
- **A negative closes a configuration, never a mechanism** (§4.1). An integer rounding difference is classed before it is called anything.

## Sources

- [claude-220's REPORT](../../loop/tasks/one-model-end-to-end/claude-220/REPORT.md), [DECISION.md](../../loop/tasks/one-model-end-to-end/claude-220/DECISION.md) and its [lab page](../../labs/one-model-end-to-end/served-key-2026-10-31/README.md).
- [claude-219's REPORT](../../loop/tasks/one-model-end-to-end/claude-219/REPORT.md): the point key named in free generation.
- [turn-memory-integer](turn-memory-integer.md): the integer turn memory this builds on.
