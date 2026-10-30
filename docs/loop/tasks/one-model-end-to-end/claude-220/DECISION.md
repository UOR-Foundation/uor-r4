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
| B | A: with exactly equal points merged, each keeping its target counts | at most about A's size; the ratio is unknown, possibly 1 | The same names: commonest-at-least-distance over counts is the same rule | The merge ratio, counted from A |
| C | Quantized points (i8, or after the Hadamard rotation) | about 55 MB as i8 | Different distances, so different names | Its own bar on a fresh panel |
| D | The sign code (Hamming) | 6.8 MB | Measured too thin to read on p-v | A larger panel |

## Constraints

- **CHARTER §3 is met by all four.** L1 is a sum of absolute differences, not a dense matmul. Nothing squashes. Model compute stays on the GPU.
- **D32's integer artifact cannot hold A's floats.** No item yet covers the point table's integer export: `turn-memory-integer` holds only the memory's two exact-id tables. That export becomes a new roadmap item or a recorded amendment, as C or D. D is the form `integer-hamming-read`'s popcount machinery already fits.
- **The goal is one model end to end** (token → vector → vector → integer → token). This unit makes the float served model whole with its key. The integer step comes after it.

## Recommendation: A, then B and C in later units with their own bars

- **A keeps this unit's free-generation numbers resting on claude-219's registered result.** Changing the table and the serving path at once would leave a difference unattributable.
- **The cost is acceptable for the float served model.**
  - 218 MB per checkpoint is large for a phone, but not for the GPU research host.
  - The search runs once per reply. The batched instrument's 6.8 ms is 0.6% of a 64-token reply's decode time; the served single-query search is measured in this unit, not assumed.
- **Next:**
  - **B is a lossless size cut:** it needs only a count, and changes no name.
  - **C or D is the integer artifact's form,** and needs its own bar, since its names differ.

## The decision review: APPROVE (a fresh non-author reviewer, 30 Oct, D68)

It checked:
- the sizes;
- that B changes no name (an exact-equality merge keeps every count at the least distance);
- that no option breaks CHARTER §3 or D32;
- that PLAN.md points no other way.

Its conditions, each taken into this unit:
- **(a)** The REPORT calls A's results a float research evaluation (CHARTER §3.5, §3.7), and its tok/s the GPU float decode's, not the served speed. It names the integer export as owed.
- **(b)** C's bar, when it comes, is measured with the integer trunk's own points for both the query and the table, never A's float points quantized after the fact.
- **(c)** The served search is measured per reply, as a single query, inside tok/s.
- **(d)** The wording above is corrected: B is at most A's size; the integer form is C or D; the integer export needs its own item.
- **(e)** The REPORT gives residency: 218 MB resident per checkpoint, 654 MB in the store for three seeds, and the bytes scanned per keyed opening.

## What a reversal costs

Little. The table is built from the checkpoint and TRAIN by a deterministic pass (claude-218's `opening_points`), so any form can be rebuilt from A at any time.
