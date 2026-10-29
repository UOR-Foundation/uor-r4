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
notebook: docs/research/notebook/m4-serving.md
scope: [crates/uor-r4-integer/src/stack/session.rs, crates/uor-r4-integer/src/stack/fbs_simd.rs, crates/uor-r4-integer/src/stack/simd.rs, crates/uor-r4-integer/src/stack/kernels.rs, crates/uor-r4-integer/src/stack/mod.rs, crates/uor-r4-integer/src/stack/tests.rs, crates/uor-r4-integer/src/bin/fbs-toks.rs, scripts/audit_zero_matmul_serving.py, docs/loop/tasks/fbs-serving-kernels/**, docs/research/notebook/m4-serving.md, docs/roadmap/items/fbs-serving-kernels.md]
---

## Why it matters

The owner, 28 Oct 2026, in claude's chat: "I don't care what we serve at long as it is cheap". This is the [pipeline map](../../model/pipeline-map.md)'s R3 and its gap 3.

On the M1, a served token is 3.73 ms (265 tok/s). The six layers are 98.6% of it: each layer's recurrence block takes 352–364 µs and its feed-forward block 253–259 µs ([claude-158](../../loop/tasks/fbs-cost-profile/claude-158/REPORT.md)). That is about 57 ns per learnable value, in scalar integer code with no vector instructions.

## What is missing

- **The layer split.** Each block holds three route gathers (`stack_fbs_gather`) and either the recurrence core or the gated product and requantization. Which part dominates is not measured.
- **Vector kernels for the FBS step.** `stack_step_fbs` is scalar.
  - `stack/simd.rs` holds the WASM-SIMD forms of the dense-map step's inner loop (`stack_gemv_pairs` in `kernels.rs`; Arm T, #1172).
  - The older frozen engine's NEON and AVX2 kernels are in `crates/uor-r4-simd` (the map, stage 11, through `docs/products/desktop-client.md`).
  - Neither runs in the FBS step.
- **What the audit allows.** `scripts/audit_zero_matmul_serving.py --stack` refuses every multiply-class instruction, NEON forms included (`mul`, `mla`, `sdot`, `smull`…), and every float instruction, `fmov` transfers among them. Integer lane adds, absolute differences, loads and stores are not on its list, nor are `umov` and `smov`.
  - The scalar gather passes each value through `black_box` because LLVM's own vectorized forms end in an `fmov`, which fails the audit (`session.rs`, at `stack_fbs_gather` and at the pointer head's maximum).
  - Results kept in memory, or moved to a general register only by `umov`/`smov`, avoid that.
  - The audit's call-graph walk follows every direct `bl`/`b` from `step`, so the kernels `stack_step_fbs` calls are audited today. It does not follow an indirect call (`blr`). So the runtime switch dispatches by a direct branch, not a function pointer, and the kernel goes in the audit's kernel list so its presence is required.
- **An exactness check for a faster step.** claude-158's `measure_fbs_step_stages` already runs the served kernels with a clock, and its host test holds the twin equal to `step_native`. A faster step must pass the same equality.

## Done when

1. **The split:** each layer's gathers, core, gated product and requantization timed on the M1, by extending `measure_fbs_step_stages` and `fbs-toks --stages`. The parts sum to the layer within 10%.
2. **A vector kernel for the largest part,** written for D11 (CHARTER §3.6: integer, no multiply instruction; products through its tables), in `stack/fbs_simd.rs`, behind a runtime switch (no new cargo feature), with the scalar path kept for WASM and other targets.
   - A host test holds every token, kept cell, query and session state equal to the scalar `step_native` over a synthetic trunk and a stored artifact.
   - The kernel is in the audit's kernel list, and `scripts/audit_zero_matmul_serving.py --stack` passes on the release binary.
3. **Speed:** `fbs-toks` decode tokens/s on the M1 with and without the kernel, on claude-126's export, with the same replies.
4. One notebook row and a REPORT. This is host and serving work: no GPU and no sheet (CHARTER §4.5).

## Progress

- **Done-when 1 is met (claude-163).** `fbs-toks --parts` splits each layer into nine parts. Each block's parts lie at 999–1,003 per mille of the block (the [notebook row](../../research/notebook/m4-serving.md) of 28 Oct).
  - **The largest part is the recurrence core,** 56.3% of a token. The feed-forward's gated product is 39.8%. The 36 route gathers, which claude-158 guessed dominated, are 0.5%, counted with the residual adds and the feed-forward's requantization.
  - **Both large parts hold the 120-root icosian snap** (`stack_snap_select` in `kernels.rs`), which scores every root with exact `ℤ[φ]` comparisons. It runs twice per 4-channel block in the core (the lane's transition and the output gate) and once per 4-unit block in the product. The snap's share of each part is not measured yet.
- **An exact sign-compatible snap (claude-164): decode 270 → 749–752 tok/s on the M1, replies byte-identical.**
  - The 120 unit icosians are closed under coordinate sign flips, so only the roots whose signs agree with the query's can win: 17 of 120 when no coordinate is zero.
  - `stack_snap_select` scans those, in index order, from a compile-time table. It equals the full scan on every test query, and the serving audit passes.
  - After it, the core is 54.3% of a 1.33 ms token and the product 36.0%.
  - This is a faster exact kernel for the largest part, not done-when 2's vector kernel, which stays open.
- **The snap's comparison without squares (claude-165): decode 742–744 → 1,169–1,217 tok/s, replies byte-identical.**
  - A profile of claude-164's binary put 33% of its samples in `stack_mul_u128` under the snap's exact sign test of `P + D√5`. Another 25% sat under the Hamilton product.
  - The bounds `2289/1024 < √5 < 1145/512` now decide that sign by shifts and additions. The squares run only between the bounds.
  - The new profile has no `stack_mul_u128` sample from the snap. Its 487 come from `stack_hamilton` (444) and the lanes (43).
- **The Hamilton product by shared tables (claude-166): decode 1,142–1,225 → 1,406–1,410 tok/s, replies byte-identical.**
  - `stack_hamilton` builds 4 tables of multiples (one per `|b_j|`), not 16, and skips a zero `a_i`.
  - The gain is smaller than the profile suggested: the step is 1.16× faster.
  - In the new profile, `stack_hamilton` (34%) and `stack_snap_select` (33%) lead.
- **Tables from four guarded doublings (claude-167): decode 1,384–1,418 → 1,829–1,861 tok/s, replies byte-identical.**
  - The two builders every table product reads now guard only `a`, `2a`, `4a` and `8a`, and sum the rest.
  - The step is 1.31–1.33× faster. The norms, whose squares use the same tables, are 2× faster.
  - In the new profile, `stack_snap_select` leads (42%) and `stack_hamilton` follows (30%).
- **The snap's scores from `|r|` by additions (claude-168): decode 1,878–1,880 → 2,448–2,449 tok/s, replies byte-identical.**
  - Each sign-compatible candidate's score is now a few additions of `|r|`, by its class: axis, half or golden. That replaces 8 `small_mul`s.
  - The step is 1.30–1.33× faster.
  - In the new profile, `stack_hamilton` (38%) leads `stack_snap_select` (25%).
- **The Hamilton product by shifts where it can (claude-169): decode 2,428–2,435 → 3,042–3,044 tok/s, replies byte-identical.**
  - Every caller's snapped quaternion is scaled by a power of two, so its axis, half and `1/2` coordinates are `±2^k`, and their products are shifts.
  - Other coordinates get one table of their own multiples.
  - The step is 1.24–1.26× faster.
  - In the new profile, `stack_snap_select` (32%) leads `stack_hamilton` (21%).
- **What is next:**
  - **Fewer snap candidates.** The 12 golden roots can be cut by the query's magnitude order, still exactly.
  - **Then a vector kernel** for what is left.
  - Each is measured against claude-169's 3,042–3,044 tok/s.

## How to start

Read:
- [claude-158's REPORT](../../loop/tasks/fbs-cost-profile/claude-158/REPORT.md);
- `stack_step_fbs`, `stack_step_fbs_timed`, `stack_fbs_gather` and `stack_recurrence_core` in `crates/uor-r4-integer/src/stack/session.rs`;
- `crates/uor-r4-integer/src/stack/simd.rs` and `crates/uor-r4-simd` (vector kernels already written for this project);
- `scripts/audit_zero_matmul_serving.py` (its forbidden patterns and `STACK_MANDATORY_SYMBOLS`).

The artifact is claude-126's seed-1 export (sha256 `314c3909…fdb305`, in claude-158's REPORT). Build `fbs-toks` with `--features serving-wasm`.

## Rules that apply

- **Serving stays integer and multiplier-free** (D11, CHARTER §3.6). `step` and its call graph keep passing the zero-multiply audit.
- **Exact or nothing:** a faster kernel that changes one token, cell or state is a defect, not a trade.
- **No dense matrix multiplication** (§3.3), including in vector form.

## Sources

- The owner's words, 28 Oct 2026, in claude's chat (quoted above).
- [claude-158](../../loop/tasks/fbs-cost-profile/claude-158/REPORT.md) and the [pipeline map](../../model/pipeline-map.md).
