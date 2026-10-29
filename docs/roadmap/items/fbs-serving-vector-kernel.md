---
id: fbs-serving-vector-kernel
title: A NEON kernel and fewer snap candidates for the served FBS step
milestone: research
area: serving
status: ready
open_to: lab
needs_gpu: false
depends_on: [fbs-serving-kernels]
done_class: host
notebook: docs/research/notebook/m4-serving.md
scope: [crates/uor-r4-integer/src/stack/kernels.rs, crates/uor-r4-integer/src/stack/session.rs, crates/uor-r4-integer/src/stack/fbs_simd.rs, crates/uor-r4-integer/src/stack/simd.rs, crates/uor-r4-integer/src/stack/mod.rs, crates/uor-r4-integer/src/stack/tests.rs, crates/uor-r4-integer/src/bin/fbs-toks.rs, scripts/audit_zero_matmul_serving.py, docs/model/pipeline-map.md, docs/loop/tasks/fbs-serving-vector-kernel/**, docs/research/notebook/m4-serving.md, docs/roadmap/items/fbs-serving-vector-kernel.md]
---

## Why it matters

The owner, 28 Oct 2026, in claude's chat: "I don't care what we serve at long as it is cheap".

[`fbs-serving-kernels`](fbs-serving-kernels.md) is done (claude-163 to claude-170). Exact structured kernels took a served token on the M1 from 270 to 3,042–3,044 tok/s on claude-126's export, with every reply byte-identical. That is about 5.2 ns per learnable value over a whole token (0.33 ms), against claude-158's 57 ns over the six layers alone (3.68 ms).

Its decision review (claude-170) left two follow-ups that need a record of their own:
- **a NEON kernel** the serving audit allows;
- **fewer golden candidates** in the icosian snap.

In [claude-169](../../loop/tasks/fbs-serving-kernels/claude-169/REPORT.md)'s profile (940 samples on the thread), `stack_snap_select` holds 32% and `stack_hamilton` 21%.

## What is missing

- **The map is stale.** Its gap 3 and R3 ([`docs/model/pipeline-map.md`](../../model/pipeline-map.md)) still name NEON kernels as the change. R3's "pass unlocks" claims a per-value rate near the hardware's, which is not shown.
- **Fewer golden candidates.** For a query with no zero coordinate, the snap scores 17 sign-compatible roots (claude-164), 12 of them golden. The query's magnitude order may rule some golden roots out exactly, but the lowest-index tie rule must hold.
- **A NEON kernel.** No vector code runs in the FBS step. The audit refuses every multiply-class instruction and every `fmov`. claude-168 met two `fmov`s from packed lanes, so a kernel must keep its lanes in memory, or leave them by `umov`/`smov`. A kernel outside `step`'s direct-branch call graph must be in the audit's kernel list.

## Done when

1. **The map.** Its gap 3 and R3 record that R3 was met with exact structured kernels (claude-164 to claude-169, about 5.2 ns a value). The per-value claim is restated as an open question.
2. **A faster exact change** to the snap or the Hamilton product: fewer golden candidates, or a vector kernel the audit allows.
   - A host test holds the changed kernel equal to the old one.
   - On claude-126's export, every reply is byte-identical.
   - `scripts/audit_zero_matmul_serving.py --stack` passes on the release binary.
3. **Speed:** `fbs-toks --parts` against the previous unit's sealed binary, run alternately, on this M1.
4. One notebook row and a REPORT for each unit. This is host and serving work: no GPU and no sheet (CHARTER §4.5).

## How to start

Read:
- [claude-164](../../loop/tasks/fbs-serving-kernels/claude-164/REPORT.md)'s exactness argument (the sign-compatible roots);
- [claude-168](../../loop/tasks/fbs-serving-kernels/claude-168/REPORT.md)'s scores and the two `fmov`s;
- [claude-169](../../loop/tasks/fbs-serving-kernels/claude-169/REPORT.md)'s profile;
- `stack_snap_select`, `snap_compatible_score` and `stack_hamilton` in `crates/uor-r4-integer/src/stack/kernels.rs`;
- the audit's forbidden patterns in `scripts/audit_zero_matmul_serving.py`.

The baseline binary is claude-169's, sha256 `8a7a83c76ef83e4302a0fbfb211061c113795fbdb35761bc2e8822123207805a`. The artifact is claude-126's seed-1 export at `caseyallard/uor-r4-store:results/claude/d11-fbs-export-gap-2026-10-26/d11gap/save/seed-1/artifact.uorlut`, sha256 `314c3909…fdb305`.

## Rules that apply

- **Serving stays integer and multiplier-free** (D11, CHARTER §3.6). `step` and its call graph keep passing the zero-multiply audit.
- **Exact or nothing:** a faster kernel that changes one token, cell or state is a defect, not a trade.
- **Geometry only, and no dense matrix multiplication** (§3.1, §3.3), including in vector form.

## Sources

- The owner's words, 28 Oct 2026, in claude's chat (quoted above).
- [`fbs-serving-kernels`](fbs-serving-kernels.md), with claude-163 to claude-170's REPORTs.
- The [pipeline map](../../model/pipeline-map.md), gap 3 and R3.
