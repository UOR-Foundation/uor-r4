---
id: fbs-serving-vector-kernel
title: A NEON kernel and fewer snap candidates for the served FBS step
milestone: research
area: serving
status: done
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

**Done on 29 Oct 2026 (claude-186).** Done-when 2 and 3 are met: `stack_isqrt` now starts at the value's top bit, in place of walking down from `2^126`.
- **Exact:** it equals the walking form on 2,048,961 values.
- **Replies:** byte-identical on claude-126's export, and on dual-c131's `raw_both` under a prompt whose reply varies (29 distinct tokens of 65).
- **Faster on the unsnapped contract:** `raw_both` decode rose from 4,176 / 4,195 to 4,491 / 4,504 tok/s, and the step fell from 238.9 / 238.5 to 222.6 / 221.9 µs, alternating against the sealed base. Snapped is unchanged, within noise.
- **The serving audit** is FULL PASS.

The candidates below stay open for a later item.

**State on 29 Oct 2026 (claude-185).** Done-when 1 is met: the map's gap 3 and R3 record exact structured kernels and today's speeds.

Two changes have moved this item since it was written:
- **The hardware multiply and divide** (claude-177, D80 and D81) took the snapped serve to 4,829 / 4,827 tok/s.
- **The unsnapped contract** (`lut-stack/5`, claude-181) carries the artifacts that pass the export gap (claude-182). It runs no icosian snap. Each gate and transition site normalises its quaternion in `stack_rotation`, with an integer square root and four `u128` divides by the norm.

**claude-185's profile** (`fbs-toks --parts`, this M1, AC power):

| Artifact | Decode tok/s | Step | `recurrence.core` per layer | `mlp.product` per layer |
|---|---|---|---|---|
| claude-126's snapped export | 4,832 | 205 µs | about 15.2 µs | about 9.0 µs |
| dual-c131's `raw_both` | 4,236 | 235 µs | about 18.0 µs | about 11.4 µs |

The unsnapped contract's extra 30 µs is all in the two parts that run the rotation.

**claude-185's change** was one reciprocal of the norm in place of the four divides. It was exact (1,384,160 cases, replies identical) but 5% slower, so it is reverted, and its patch is in that unit's folder. A likely reason, not measured: the old divides' numerators often fit in 64 bits, which is a library call's fast path. The reciprocal's `2^78 / norm` never does.

**What remains, each a candidate faster exact change:**
- **The rotation's integer square root** (`stack_isqrt`). It is a digit-by-digit `u128` loop, and it first walks down from `2^126` to the value's top bit. Its share of the rotation is not split yet.
- **Fewer golden candidates, for the snapped contract.** For a query with no zero coordinate, the snap scores 17 sign-compatible roots (claude-164), 12 of them golden. The query's magnitude order may rule some golden roots out exactly, but the lowest-index tie rule must hold.
- **A NEON kernel.** No vector code runs in the FBS step.
  - Since claude-175 the audit allows the scalar integer multiply and divide. Which vector forms it allows has not been checked.
  - It refuses every `fmov`. claude-168 met two from packed lanes, so a kernel must keep its lanes in memory, or leave them by `umov`/`smov`.
  - A kernel outside `step`'s direct-branch call graph must be in the audit's kernel list.

## Done when

1. **The map.** Its gap 3 and R3 record that R3 was met with exact structured kernels (claude-164 to claude-169, about 5.2 ns a value). The per-value claim is restated as an open question.
2. **A faster exact change** to the snap site, in either form, or to the Hamilton product. Either form means the snapped selection or the unsnapped `stack_rotation` (claude-185). The change can be fewer golden candidates, a faster exact square root or quotient, or a vector kernel the audit allows.
   - A host test holds the changed kernel equal to the old one.
   - Every reply is byte-identical, on claude-126's export and on dual-c131's `raw_both` artifact (claude-182).
     - dual-c131's reply under `fbs-toks`'s default prompt is one token repeated (claude-185), so it cannot show a difference.
     - On that artifact the check also compares something that can differ: the session's final state, the per-step query codes, or a prompt whose reply varies.
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

**The baseline binary** is the head's `fbs-toks` at `c95dfaea6`, with claude-177's kernels. It is sealed at `caseyallard/uor-r4-store:results/claude/fbs-step-profile-2026-10-29/base/fbs-toks`, sha256 `f78428981f9575c9b688f7d7e2ce21e624299913e5afb91d0ad260236ef1e2a3`. claude-169's binary, `8a7a83c76ef83e4302a0fbfb211061c113795fbdb35761bc2e8822123207805a`, predates the hardware multiply.

**The artifacts:**
- claude-126's seed-1 export, at `caseyallard/uor-r4-store:results/claude/d11-fbs-export-gap-2026-10-26/d11gap/save/seed-1/artifact.uorlut`, sha256 `314c3909…fdb305`;
- dual-c131's seed-1 `raw_both`, at `results/claude/d11-unsnapped-gap-2026-10-29/art/dual-c131-seed-1-raw_both.uorlut`, sha256 `a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd`.

## Rules that apply

- **Serving stays integer** (CHARTER §3.6, as amended by D80 and D81): no float or transcendental instruction. The hardware integer multiply and divide are allowed once [`integer-mul-div-serving`](integer-mul-div-serving.md)'s audits allow them; until then `step` and its call graph keep passing the zero-multiply audit.
- **Exact or nothing:** a faster kernel that changes one token, cell or state is a defect, not a trade.
- **Geometry only, and no dense matrix multiplication** (§3.1, §3.3), including in vector form.

## Sources

- The owner's words, 28 Oct 2026, in claude's chat (quoted above).
- [`fbs-serving-kernels`](fbs-serving-kernels.md), with claude-163 to claude-170's REPORTs.
- The [pipeline map](../../model/pipeline-map.md), gap 3 and R3.
