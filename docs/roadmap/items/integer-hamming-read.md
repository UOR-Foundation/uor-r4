---
id: integer-hamming-read
title: Serve the Hamming read from the integer artifact on the laptop
milestone: research
area: serving
status: ready
open_to: lab
needs_gpu: true
depends_on: [float-training-module, spiralcore-fbs-offline-float-sheet]
notebook: docs/research/notebook/m4-serving.md
scope: [crates/uor-r4-integer/src/stack/hamming.rs, crates/uor-r4-integer/src/stack/mod.rs, crates/uor-r4-integer/src/stack/session.rs, crates/uor-r4-integer/src/bin/fbs-toks.rs, crates/uor-r4-integer/src/bin/fbs-export.rs, crates/uor-r4-float-train/src/fbs_export.rs, docs/harness/prep/integer-hamming-read*, docs/loop/tasks/integer-hamming-read/**, docs/research/notebook/m4-serving.md, docs/roadmap/items/integer-hamming-read.md]
---

## Why it matters

This is unit U3 of the [pipeline map](../../model/pipeline-map.md)'s order of work. claude-146's induction-keyed Hamming read (`crates/uor-r4-float-train/src/flock_read_hamming.rs`) solves the project's recall probe: far-bucket recall 0.984–0.994 at both seeds, against chance with no read ([claude-146 REPORT](../../loop/tasks/in-space-trunk-to-text/claude-146/REPORT.md)). It exists only in the float trainer. The map's stage 5 (look-back) says: "Serving: none. The integer runtime has no Hamming read."

The read is cheap to serve by construction: per position, 4 heads × 64 sign bits, 32 bytes of code per read; a token costs a popcount pass over the context per head. At a context of 32,768 that is about 2 MiB per token, against 16 GiB for a dense model's cache at Llama-2-7B's shape (the map's stage 5, proof by arithmetic). None of it is measured on the laptop.

It builds on the FBS exporter and session already merged (`fbs_export.rs`, `fbs-export`, `fbs-toks`). Their item, `d11-exporter-float-trunk`, stays open for its export gap, which this unit does not need to close: its parity is the read's own, against the float forward of the same checkpoint.

## What is missing

- The integer runtime (`uor-r4-integer`, the D11 engine) has no Hamming read: no packed sign codes, no popcount match, no hard choice of the nearest earlier positions, no readout move.
- The exporter does not carry the read's parameters (`hamming_read.safetensors`: rotations, margins, weights).
- No decode timing with the read on this M1.

## Done when

1. The integer runtime runs the read as claude-146's REPORT describes it, under D11's rules (CHARTER §3.6: integer only, no multiply instruction; products through its tables): keys and copy queries from the table's points through the key rotation, packed into 64-bit codes; the match by `popcount(q XOR k)`; the 16 nearest earlier positions `1 ≤ s < t`; the weights `max(0, m − d)/m`; the stream read after layer 2 and the move on the readout.
2. **Parity:** on a stored checkpoint with the read (claude-146's language runs, `caseyallard/uor-r4-store:results/claude/hamming-read-2026-10-27/`), the integer forward's picks and decoded ids match the float forward's on DEV within the D11 forward's stated tolerance, and the REPORT gives the gap. The float reference is computed on the GPU (CHARTER §3.5); the integer side runs on the laptop, as serving does.
3. **Speed on this laptop:** `fbs-toks` decode tokens/s against context, with the read and without, on the same artifact.
4. One notebook row; the result on main.

## Cancel if

The integer runtime already has a Hamming read on main, or the float read is withdrawn.

## How to start

Read:
- claude-146's REPORT and `flock_read_hamming.rs` (its module doc gives every step, in order);
- the map's stages 5, 10 and 11;
- [serving and export](../../model/serving-and-export.md), and the FBS exporter (`fbs_export.rs`, `fbs-export`) and session (`stack/session.rs`, `fbs-toks`).

## Rules that apply

- **D11 serving** (CHARTER §3.6): integer only, no multiply instruction, no float in the served path. Popcount and comparisons need no multiply; the weights and the move go through D11's product tables.
- **Geometry only, no softmax or squashing, no dense products** (§3.1–§3.3).
- **Serving the exported integer artifact on a client device is allowed** (§3.5–§3.6); the float reference stays on the GPU.

## Sources

- [The pipeline map](../../model/pipeline-map.md), stages 5, 10 and 11 and unit U3.
- [claude-146 REPORT](../../loop/tasks/in-space-trunk-to-text/claude-146/REPORT.md) and `crates/uor-r4-float-train/src/flock_read_hamming.rs`.
- [m4-serving notebook](../../research/notebook/m4-serving.md).
