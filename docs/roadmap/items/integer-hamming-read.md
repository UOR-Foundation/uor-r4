---
id: integer-hamming-read
title: Serve the Hamming read from the integer artifact on the laptop
milestone: research
area: serving
status: done
open_to: lab
needs_gpu: true
depends_on: [float-training-module, spiralcore-fbs-offline-float-sheet]
notebook: docs/research/notebook/m4-serving.md
scope: [crates/uor-r4-integer/src/stack/hamming.rs, crates/uor-r4-integer/src/stack/mod.rs, crates/uor-r4-integer/src/stack/session.rs, crates/uor-r4-integer/src/stack/format.rs, crates/uor-r4-integer/src/fbs_snapshot.rs, crates/uor-r4-integer/src/stack/synthetic.rs, crates/uor-r4-integer/src/stack/tests.rs, crates/uor-r4-integer/src/bin/fbs-toks.rs, crates/uor-r4-integer/src/bin/fbs-export.rs, crates/uor-r4-float-train/src/fbs_export.rs, crates/uor-r4-float-train/src/bin/fbs-snapshot.rs, crates/uor-r4-float-train/src/flock_read_hamming.rs, docs/harness/prep/integer-hamming-read*, docs/loop/tasks/integer-hamming-read/**, docs/research/notebook/m4-serving.md, docs/roadmap/items/integer-hamming-read.md]
---

## Why it matters

This is unit U3 of the [pipeline map](../../model/pipeline-map.md)'s order of work. claude-146's induction-keyed Hamming read (`crates/uor-r4-float-train/src/flock_read_hamming.rs`) solves the project's recall probe: far-bucket recall 0.984–0.994 at both seeds, against chance with no read ([claude-146 REPORT](../../loop/tasks/in-space-trunk-to-text/claude-146/REPORT.md)). It exists only in the float trainer. The map's stage 5 (look-back) says: "Serving: none. The integer runtime has no Hamming read."

The read is cheap to serve by construction: per position, 4 heads × 64 sign bits, 32 bytes of code per read; a token costs a popcount pass over the context per head. At a context of 32,768 that is about 2 MiB per token, against 16 GiB for a dense model's cache at Llama-2-7B's shape (the map's stage 5, proof by arithmetic). claude-242 timed it on the laptop (step 6 below): at 32,768 the read is 97% of decode time.

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
  4. the exporter reading `hamming_read.safetensors`: `fbs-snapshot` (`crates/uor-r4-float-train/src/bin/fbs-snapshot.rs`, the only path from a checkpoint to a snapshot) loads it with `HammingReads::load` and carries it, and `check_checkpoint_reads` stops refusing it. The conversion is a `pub fn` in `fbs_export.rs`, which can read `HammingRead`'s `pub(crate)` fields (the binary, a separate crate, cannot). The float readout read always runs on the readout, at `HAMMING_READOUT_DEPTH` (6), and the integer engine takes the read whose depth is the layer count as the readout; so the export refuses a trunk whose layer count is not `HAMMING_READOUT_DEPTH` (a 2-layer trunk would otherwise serve the stream read on the readout) (claude-199);
  5. the float parity on claude-146's checkpoints (GPU, under a prep sheet: the trunk runs only on CUDA). It starts with a trace, because claude-233 (#3465) found the gap first. On (r-drop) step 16,384, the trunk alone agrees with its float D11 forward at s1 and s3 (5 and 11 of 1,536 positions differ); with the read it does not (694 and 823). Casey chose this step next (31 Oct 2026, in chat): "Trace attention first". claude-234 recorded the method:
     - **The float side, on the GPU.** A capture inside `flock_read_hamming.rs`'s `apply` and `apply_readout` writes, per (position, head, read), the query code, the key code, the kept positions with their distances, and the distance at the 16th place. Only real candidates are written: at `t ≤ 16` there are fewer than 16, and the float drops its masked entries by `real`. The capture is off by default, so training is unchanged. It runs in the D11 readout (`fbs_float_readout_on`) on claude-233's sequences.
     - **The integer side, on the laptop.** The same record from `stack/hamming.rs`, served from the artifact exported from the same checkpoint.
     - **The timing, on the same laptop pass.** `fbs-toks` decode tokens/s against context, with the read and without, on the same artifact: done-when 3, step 6's measurement, taken early. Casey's call (31 Oct 2026, in chat): "Yes, fold it in". The timing does not depend on the parity. It gives the read's long-context cost, the number that decides whether a cheaper read is worth building. The two candidates are untested:
       - the Flock tree read, whose cost of about log n steps is a hypothesis, never measured in integer form ([flock read](../../mechanisms/flock-read.md));
       - a fixed-size hypervector bundle, whose mechanism is PARKED ([vector-symbolic binding](../../mechanisms/vsa.md)).
     - **The comparison.** The share of kept sets equal to float's, and top-1, each split by head kind.
       - **Which sets are judged.** Only one kind of code depends on the residual: the type heads' queries. Copy heads' queries and keys, and type heads' keys, are built from tokens' points (claude-189, claude-195). So:
         - every copy-head set, at both reads, is judged at every position;
         - every stream-read type-head set is judged too. Its query comes from the trunk below the read, which the read does not feed (`geometric_stack_native.rs`, after layer 2);
         - a readout type-head set is judged only where every stream-read set agrees at its position and before it, since the readout's residual carries the stream read's output. The other readout type-head sets are counted apart, as downstream.
       - **The cause of a judged set that differs:**
         - **A tie across the 16th place,** decided by the tie rule: the same query code, and the same key code at every position either side keeps. Each member that is in only one of the two sets must then sit at the 16th-place distance; if one does not, that is an instrument fault.
         - **A sign flip:** anything else. A copy-head code or a type-head key depends only on the table and the read's rotations, so its flip comes from rounding, never from the trunk. A type-head query can flip from either; the parameters arm below separates the read's own rounding.
       - An instrument fault is repaired before any number is read.
     - **Two arms beside the plain D11 forward,** one per lead:
       - **The tie rule.** The float `choose_sorted` keeps the first 16 of a GPU arg-sort, whose order among ties is not specified. The integer `nearest` keeps ascending distance, then ascending `s`. The arm gives the float the integer's rule, behind the same off-by-default switch. It sorts on the key `h·T + s`, where `h` is the distance in bits and `T` the sequence length. The float's distance is a mean L1 between ±1 codes, `2h` over the head's width. The key is an integer far below 2^24, so it is exact in f32.
       - **The read's parameters.** The D11 readout loads `hamming_read.safetensors` at full f32 (`fbs-snapshot.rs`), and `round_tables_as_exported` rounds only the trunk's tables. The artifact holds the rotations and weights at 2^-24 and the margins at 2^-16. The arm rounds the read's parameters as exported.
     - A lead that closes the gap is fixed in the port or the exporter. Then the parity is measured against the bar the sheet registers, on claude-146's checkpoints as done-when 2 says, and on (r-drop), which the one-model path ships;
     - **The trace's result (claude-235, row F):** neither lead closes or moves the gap (s1 d 694 / 696, s3 823 / 816, the plain and tie arms; the rounded arms change no code). The two sides already part at the first read, in the copy heads, whose codes are by design (not tested here) a function of the token alone: 89–94% of their query codes differ at depth 2, and the two sides' codes sit 2.2–3.2 bits of 64 apart on average (4.6–5.2 at depth 6). Each side recorded the codes it chose with (the own-codes check, review round 1). The product's form, the activation storage and the table's grid match by reading the code only. **Next:** a laptop host check of every vocabulary entry's copy codes, at both depths, three ways (the CPU float control `hamilton_gate_fwd_cpu` with `pack_signs` on the checkpoint's rounded key rotation and Q12 table; the integer `vocab_codes` on the artifact; the GPU trace). It names the stage where they part, and the fix follows in this scope. Timing taken early (done-when 3): decode on the M1 is 3,812–4,109 tok/s with the read and 4,467–4,733 without, at contexts 96–192;
     - **The host check's result (claude-237): the stage is the GPU's D11 forward, which snaps the read's rotations.** `fbs-toks ARTIFACT --copy-codes` compares every vocabulary entry's copy codes at both depths:
       - the CPU float control (a) equals the integer session's codes on 8,192 of 8,192 codes, with 0 differing bits;
       - the integer trace equals its session's codes;
       - the GPU trace equals none of them, but it does equal (a′), the control with the key rotation snapped lane by lane to the nearest of the 120 icosians: 3,072 of 3,072 query codes and 3,056 of 3,056 key codes, at both depths, at both seeds, in both arms.

       In D11 mode the CUDA gate hook (`hamilton_sheet::try_hamilton_gate_fwd_cuda`, under `HamiltonIcosianHard`) snaps every Hamilton gate, and the read's `rotate` is one. The key rotation's snap is measured; the query, value and step rotations go through the same `rotate`, so the code snaps them too. The checkpoints' trainer, `train-in-space` (`in_space_long.rs`), runs under `HamiltonActive`, which passes the gates through, and the artifact serves the rotations unsnapped by this item's design. So the float reference is the side that differs, and claude-233's and claude-235's parity numbers measured that snap.
     - **Next (decided in claude-237's decision review):**
       - keep the read's rotations out of the D11 gate hook in `flock_read_hamming.rs`, so the float D11 forward computes the read as training did, with a CUDA test that fails without it. The form saves and restores the whole hook state (arm, call index, export flag, transition snap), as `with_d11_snaps` does, or routes `rotate` around the hook. A scoped `with_gate_mode` does not do: it leaves the arm at `HamiltonActive` and resets the call index, so the trunk's later layers would run unsnapped;
       - with the fix in, the trunk-only figures (5 and 11 of 1,536 positions differing, claude-233) stay as they are;
       - the Export readout mode (`with_d11_export_gates`) snaps every gate whatever the arm, so the fix keeps the read out of that path too, or the next ORDER states that Export is not a parity reference;
       - then re-trace on the GPU under a prep sheet, its FIT and its pre-run review. `fbs-toks --copy-codes` on the new trace must find the float codes equal to (a);
       - then read the kept-set share and top-1 against the registered bar (done-when 2).

       The same pod runs claude-237's CUDA-only snap test and claude-235's two tie-switch tests;
     - **The re-trace's result (claude-238, row T): with one tie rule on both sides, the read is at parity on (r-drop).** The guard (`as_trained`) is in, and T1 and T2, on reads with the type heads' step arm on, fail with it set aside and with only the step arm outside it. The trunk alone is claude-233's byte for byte. Every float copy code now equals the CPU control (a), with 0 differing bits.
       - **d11-tie meets both bars at both seeds:**
         - d is 6 and 27 of 1,536 (bar 46);
         - the copy-head sets past t = 17 are all equal;
         - the first read's type heads are at 0.997;
         - the second read's type heads are at 0.993 and 0.983, on 1,570 and 1,494 judged sets.
       - **d11, the rule off (the GPU sort's own order, which the checkpoint trained under; not shown to reproduce its training forward exactly), does not:** d is 174 and 243. With equal codes, 1,448–1,567 of the 3,072 sets in each of the three fully judged cells tie across the 16th place, and the GPU arg-sort's unspecified order breaks them differently: d11 and d11-tie differ in top-1 at 174 and 227 of 1,536 positions. The rounded arms change no code. claude-235's two tie-switch tests were never written, so they did not run.
       - **Next:**
         - the float read takes the integer session's tie rule by default (`set_hamming_tie_rule`, ascending bits then ascending `s`), so the trainer and `fbs-snapshot`'s readouts and traces take it unless an arm turns it off, with the two tie-switch CUDA tests;
         - then the same bars on claude-146's checkpoints and on DEV (done-when 2).

         Decode on the M1 under load: 3,068–4,085 tok/s with the read, 4,306–4,857 without, at contexts 96–192;
     - **The tie rule is on by default (claude-239).** The float read now breaks ties by ascending bits, then ascending `s`, on every thread, unless `set_hamming_tie_rule(false)` turns it off. `fbs-snapshot`'s `d11` arm is now that explicit off arm, the GPU sort's own order.
       - **The tests:**
         - CUDA test A runs a real `read()` with planted ties, checked against the host's nearest 16 by (bits, `s`). It fails with the default off and with the read's wiring off.
         - Test B, now on codes with last-bit noise, fails with `choose_sorted`'s tie branch removed.
         - Restored, the 44 `flock_read_hamming::` and `fbs_export::` tests pass with `--features cuda`. A's off run keeps a different set in 307 of 512 rows.
       - **Checkpoints trained before this unit** trained with the rule off (claude-146's `choose_sorted` is one arg-sort). The `d11` arm turns the rule off; it is not shown to reproduce their training forward exactly.
       - **Next:** the bars on claude-146's checkpoints and on DEV, with the default on (done-when 2). That needs a prep sheet, its FIT and a pre-run review.
     - **Parity on claude-146's checkpoints, on DEV (claude-240, row P): done-when 2 holds.** The sample is DEV's 54 sampled windows (every 8th of 428), each cut after its last scored position: 5,795 positions. The bars are the [prep sheet](../../harness/prep/integer-hamming-read-dev-2026-11-01.md)'s.
       - **`d11-tie`, the float read's default, meets both bars at all three seeds:**
         - d is 47, 72 and 49 (bar 173), so top-1 agrees at 98.8–99.2%;
         - the copy-head sets past t = 17 are all equal;
         - the first read's type heads are at 0.997–0.998;
         - the second read's are at 0.987–0.991, on 7,260–7,858 judged sets (floor 965).
         Both comparisons count 0 ties and 0 faults. `I-Copy` is clean on every arm.
       - **`d11`, the GPU sort's own order, does not:** d is 612, 843 and 597, the barred cells are at 0.47–0.55 with thousands of ties, and the fourth cell is unmeasured.
       - **The bridge,** `d11-tie` on the fixed sequences, gives d of 28, 19 and 21 (claude-238's bar 46).
       - **The checkpoints load through an evaluation copy.** claude-179 changed `spiralcore_fbs.cu`'s backward after claude-146 trained, so `StackModel::load` refused the saved plan fingerprints (attempt 1, row 0). `fbs-snapshot --rebind-fbs-backward` rewrites only those six fingerprints, and refuses unless the forward's text is unchanged. The copy is held to claude-146's sealed inspection (`fbs-snapshot --inspect as-trained`), with claude-146's own build at `df1d7892` as the control. Both equal the sealed line in all 22 fields at every seed.
       - **Decode on the M1, on battery:** 3,715–4,092 tok/s with the read, 4,508–4,852 without, at contexts 96–192 (s1's artifact). The sweep stops at the artifact's context.
       - **Next:** the read's cost at long contexts (done-when 3), on a synthetic artifact with a longer context (`stack/synthetic.rs`), since the cost does not depend on trained weights.
  6. **the speed (claude-242): done-when 3 holds, so the item is done.**
     - **The sweep.** `fbs-toks --context-sweep --tokens` ran on claude-240's s1 artifact, re-exported at a context of 32,768, over DEV's first 32,704 ids, on this M1 on AC power. Load averages fell from 3.6 to 2.8 across the run.
       - With the read, decode is 4,647 tok/s at 192, 3,345 at 576, 1,851 at 2,112, 659 at 8,256 and 177 at 32,768.
       - Without it, decode is 5,462 to 6,093 at every length.
       - The read's share of decode is 24% at 192, 66% at 2,112 and 97% at 32,768.
     - **The cost is linear in the context:** about 167 ns per earlier position per decode token, or about 20 ns per candidate for each of the 8 head-reads. `nearest` sorts every candidate to keep 16. That is read from the code; how the time splits was not profiled.
     - **The checks:**
       - I-Same: the short re-export is the stored artifact byte for byte.
       - I-Reply: the long export differs only in its header's context, and gives the same replies.
     - **Next:** choose the 16 by counting the distances, which run 0 to 64. That gives the same set with no sort ([REPORT](../../loop/tasks/integer-hamming-read/claude-242/REPORT.md), Still open).

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

Another unit already meets this item's done-when on main (the read served at parity with its float forward, and its speed measured), or the float read is withdrawn. The read existing on main is not a cancel. Its copy-head kernel came in with claude-189. claude-194 and 195 serve its copy and type heads, and claude-200's export carries its parameters. Its parity was met by claude-240 and its speed measured by claude-242, which close it.

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
