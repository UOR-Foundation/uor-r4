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
scope: [crates/uor-r4-integer/src/stack/hamming.rs, crates/uor-r4-integer/src/stack/mod.rs, crates/uor-r4-integer/src/stack/session.rs, crates/uor-r4-integer/src/stack/format.rs, crates/uor-r4-integer/src/fbs_snapshot.rs, crates/uor-r4-integer/src/stack/synthetic.rs, crates/uor-r4-integer/src/stack/tests.rs, crates/uor-r4-integer/src/bin/fbs-toks.rs, crates/uor-r4-integer/src/bin/fbs-export.rs, crates/uor-r4-float-train/src/fbs_export.rs, crates/uor-r4-float-train/src/bin/fbs-snapshot.rs, docs/harness/prep/integer-hamming-read*, docs/loop/tasks/integer-hamming-read/**, docs/research/notebook/m4-serving.md, docs/roadmap/items/integer-hamming-read.md]
---

## Why it matters

This is unit U3 of the [pipeline map](../../model/pipeline-map.md)'s order of work. claude-146's induction-keyed Hamming read (`crates/uor-r4-float-train/src/flock_read_hamming.rs`) solves the project's recall probe: far-bucket recall 0.984–0.994 at both seeds, against chance with no read ([claude-146 REPORT](../../loop/tasks/in-space-trunk-to-text/claude-146/REPORT.md)). It exists only in the float trainer. The map's stage 5 (look-back) says: "Serving: none. The integer runtime has no Hamming read."

The read is cheap to serve by construction: per position, 4 heads × 64 sign bits, 32 bytes of code per read; a token costs a popcount pass over the context per head. At a context of 32,768 that is about 2 MiB per token, against 16 GiB for a dense model's cache at Llama-2-7B's shape (the map's stage 5, proof by arithmetic). None of it is measured on the laptop.

It builds on the FBS exporter and session already merged (`fbs_export.rs`, `fbs-export`, `fbs-toks`). Their item, `d11-exporter-float-trunk`, stays open for its export gap, which this unit does not need to close: its parity is the read's own, against the float forward of the same checkpoint.

## What is missing

- The integer runtime (`uor-r4-integer`, the D11 engine) has no Hamming read: no packed sign codes, no popcount match, no hard choice of the nearest earlier positions, no readout move.
- The exporter does not carry the read's parameters (`hamming_read.safetensors`: rotations, margins, weights).
- No decode timing with the read on this M1.

**The design the port follows** (claude-187, from `flock_read_hamming.rs` and the integer format; a decision review passed):

- **What a read stores.** There are two reads, at depths `HAMMING_DEPTHS = [2, 6]`: after layer 2 in the stream, and on the readout. Each, in `hamming_read.safetensors` as `d{depth}.{name}`, has:
  - `query_rotation`, `key_rotation` and `value_rotation`, each `(width / 4, 4)`: one quaternion per lane;
  - `margin`, `(heads)`, in bits;
  - `weight`, `(2, width)`: row 0 is `w_c`, the copy heads', and row 1 is `w_y`, the type heads'.

  There is also a `heads` scalar. claude-146 trained `heads = 4` at width 256: 2 copy heads and 2 type heads, each on its own 64 coordinates, one `u64` word. An optional `d{depth}.step_rotation` is claude-151's type spin. claude-146's checkpoints do not have it, and the port refuses it until the type spin is ported.
- **The artifact.**
  - `FbsTrunkSnapshot` gains an optional `hamming`. It is `serde(default, skip_serializing_if = "Option::is_none")`, so every existing `snapshot.json` still parses.
  - `StackShape` gains an optional `hamming` that is likewise skipped when absent, so a read-free export stays `lut-stack/4` or `/5`, byte for byte.
  - A shape with a read is `uor-r4.lut-stack/6`. The raw flags are kept beside it.
  - The sections are `hamming.d{depth}.{name}`, each `i32` at an exponent declared in the shape.
- **Rotations are stored raw, and normalized at serving.**
  - The float read rotates through `hamilton_gate`, which normalizes `g / |g|`.
  - The integer read stores each quaternion as exported, at a fixed exponent. It normalizes it by `stack_rotation(g, 1 << 31, GATE_EPSILON)`, the exact integer path `lut-stack/5`'s raw gates use (claude-181). Then it rotates by `stack_hamilton`.
  - So no second normalization routine is needed.
- **Codes.** A coordinate's bit is 1 at or above 0, 64 coordinates to a word in coordinate order, as `pack_signs`. Copy-head keys and queries depend only on the token: the shared table's point is the `writer.table` row, which is also the input embedding. So their codes are built once per vocabulary entry, at load.
- **Choice: the tie rule.**
  - **Recorded as a decision.** The float `choose_sorted` sorts the masked distances, `arg_sort_last_dim(true)`, and keeps the first `kept`. Its own doc says "Ties may come out in another order".
  - **The integer read** keeps ascending distance, then ascending position `s`: deterministic, and the order of any stable sort.
  - **Parity:** it is measured on kept *sets*. A tie's order inside the set does not matter, and a tie across the 16th place is counted, not assumed away.
  - **Candidates:** `1 ≤ s < t` (`first = 1`; position 0 has no previous token). Only real candidates are kept: the float masks the others to `FAR` and drops them by `real`.
- **Rounding at the hooks.** In the float trunk, the stream read's output is rounded to Q16: `x = q16(read.apply(&x, ids, &table)?)?` (`geometric_stack_native.rs:1148`). The readout read's output is not rounded (`:1187`). Both reads get `table = embedding.weight`, the tensor `fbs_export.rs` writes as `embed`, which is `writer.table`. The integer combine reproduces both.
- **A zero quaternion.** At `g = 0`, `hamilton_gate`'s `1e-12` and `GATE_EPSILON = 0` both give 0. The round-trip test includes one.
- **The session refuses what it does not serve.** Until a unit serves a part, an artifact that needs it is refused with a clear error: a read at all, then non-zero type-head weights `w_y`, then a `step_rotation`. So a partial port never silently differs from float.
- **The order of units:**
  1. the artifact, with a round-trip and read-free bytes unchanged;
  2. the copy heads served, with synthetic controls against an exact reference;
  3. the type heads;
  4. the exporter reading `hamming_read.safetensors`: `fbs-snapshot` (`crates/uor-r4-float-train/src/bin/fbs-snapshot.rs`, the only path from a checkpoint to a snapshot) loads it with `HammingReads::load` and carries it, and `check_checkpoint_reads` stops refusing it. The float readout read sits at `HAMMING_READOUT_DEPTH` (6), so the export maps it to the trunk's layer count, or refuses a trunk of another depth (claude-199);
  5. the float parity on claude-146's checkpoints (GPU, under a prep sheet: the trunk runs only on CUDA);
  6. the speed.

## Done when

1. The integer runtime runs the whole read as claude-146's REPORT describes it, all four heads of both reads, under D11's rules (CHARTER §3.6: integer only, no float or transcendental instruction; since D80 and D81 the hardware integer multiply and divide are allowed once [`integer-mul-div-serving`](integer-mul-div-serving.md)'s audits allow them, and until then products go through D11's tables):
   - **copy heads:** keys and queries from the table's points through the key rotation. These depend only on the token, so their codes can be built once per vocabulary entry;
   - **type heads:** queries from the residual through the query rotation, and payloads from the residual at `s` through the value rotation. These rotate the state at run time, so each product goes through D11's tables;
   - the codes packed into 64-bit words; the match by `popcount(q XOR k)`; the 16 nearest earlier positions `1 ≤ s < t`; the weights `max(0, m − d)/m`; the stream read after layer 2 and the move on the readout, each product through the tables.
2. **Parity,** against the float forward of the same stored checkpoint (claude-146's language runs, `caseyallard/uor-r4-store:results/claude/hamming-read-2026-10-27/`), on DEV. No D11 tolerance exists for this read yet, so the unit's prep sheet pre-registers the bar for two numbers, and the REPORT gives both:
   - the share of (position, head, read) kept sets equal to float's. One sign flip in an integer rotation changes a code, so this is not assumed to be exact;
   - top-1 agreement between the integer and float decoded ids.
   
   The float reference is computed on the GPU (CHARTER §3.5); the integer side runs on the laptop, as serving does.
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

- **D11 serving** (CHARTER §3.6, as amended by D80 and D81): integer only, no float or transcendental instruction in the served path. The hardware integer multiply and divide are allowed once [`integer-mul-div-serving`](integer-mul-div-serving.md)'s audits allow them. Popcount and comparisons need no multiply; until then the weights and the move go through D11's product tables.
- **Geometry only, no softmax or squashing, no dense products** (§3.1–§3.3).
- **Serving the exported integer artifact on a client device is allowed** (§3.5–§3.6); the float reference stays on the GPU.

## Sources

- [The pipeline map](../../model/pipeline-map.md), stages 5, 10 and 11 and unit U3.
- [claude-146 REPORT](../../loop/tasks/in-space-trunk-to-text/claude-146/REPORT.md) and `crates/uor-r4-float-train/src/flock_read_hamming.rs`.
- [m4-serving notebook](../../research/notebook/m4-serving.md).
