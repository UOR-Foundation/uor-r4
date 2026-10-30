# Decision card: the point key's served table (one-model-end-to-end, claude-220, 30 Oct 2026)

Lab: claude. ORDER: [claude-220](ORDER.md), done-when 1. It is owed by [claude-219's REPORT](../claude-219/REPORT.md): "Its decision review weighs the point key's table (218 MB as f32 for TRAIN's openings) against the gain."

## The question

**In what form does the one model serve the point key's table first?**

The table is TRAIN's 212,968 opening points under the trunk (256 f32 each) and their targets. The served key names an opening by the commonest target among the table's L1-nearest points, ties to the smaller id.

## What is measured (claude-219, merged as `e44f68e65`)

- **The point key carries the reply in free generation at all three seeds** on the fresh panel p-v: templates (16, 3), (9, 1) and (14, 2) against greedy, each beyond its band. Exact (12, 3), (9, 1) and (14, 2).
- **That is the exact f32 table, read by `l1_named`.**
- **The cheap keys are unread here.** The sign key (32 bytes an opening) and the Hadamard sign key were too thin to read on p-v: template b + c at most 7.
- **Cost:**
  - the search takes about 6.8 ms an opening in the two-pass instrument, once per reply, against about 18 ms a token for 64 tokens of decode (55–57 tok/s);
  - the table is 218 MB as f32 for each checkpoint, plus 0.85 MB of u32 targets.

## The options

| | Form | Size | Behaviour against the measured key | What it needs first |
|---|---|---|---|---|
| A | The exact f32 table, as measured | 218 MB | The same names (bit-for-bit points; the parity instrument checks it) | Nothing |
| B | A: with identical points merged, each keeping its target counts | below 218 MB, by an unknown ratio | The same names: commonest-at-least-distance over counts is the same rule | The merge ratio, counted from A |
| C | Quantized points (i8, or after the Hadamard rotation) | about 55 MB as i8 | Different distances, so different names | Its own bar on a fresh panel |
| D | The sign code (Hamming) | 6.8 MB | Measured too thin to read on p-v | A larger panel |

## Constraints

- **CHARTER §3 is met by all four.** L1 is a sum of absolute differences, not a dense matmul. Nothing squashes. Model compute stays on the GPU.
- **D32's integer artifact cannot hold A's floats.** The integer export of the table belongs with `turn-memory-integer`'s remaining units (session keys and moves), as C or an integer form of B.
- **The goal is one model end to end** (token → vector → vector → integer → token). This unit makes the float served model whole with its key. The integer step comes after it.

## Recommendation: A, then B and C in later units with their own bars

- **A keeps this unit's free-generation numbers resting on claude-219's registered result.** Changing the table and the serving path at once would leave a difference unattributable.
- **The cost is acceptable for the float served model.**
  - 218 MB beside a 52 MB checkpoint is large for a phone, but not for the GPU research host.
  - The search, once per reply, is under 1% of a 64-token reply's decode time.
- **Next:**
  - **B is a lossless size cut:** it needs only a count, and changes no name.
  - **C is the integer artifact's form,** and needs its own bar, since its names differ.

## What a reversal costs

Little. The table is built from the checkpoint and TRAIN by a deterministic pass (claude-218's `opening_points`), so any form can be rebuilt from A at any time.
