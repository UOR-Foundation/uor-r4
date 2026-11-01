# Decision card: the float reference for end-to-end parity (integer-nearest-readout, claude-243, 1 Nov 2026)

Lab: claude. ORDER: [claude-243](ORDER.md), done-when 1. The item's done-when 3 names the reference. This card changes the bar's reference, which is a change to a bar's scope (CHARTER §5.1, §5.2).

## The question

**Against which float greedy replies is the integer artifact's end-to-end parity bar read?**

Done-when 3 says: "Integer greedy of (r-drop) step 16,384, memory on, against claude-220's stored float greedy on DEV and p-w at seeds 1–3."

## What is measured

- **claude-220's stored replies** (F) were decoded at `eb762e602`.
  - There, the Hamming read keeps its 16 nearest by one GPU `arg_sort` (`flock_read_hamming.rs:714`), whose own doc says "Ties may come out in another order" (`:689`).
  - The integer read keeps ascending distance, then ascending `s`.
- **The float read has kept ties the integer way by default since claude-239** (`cef022733`, `TIE` starts on). `train-in-space` has no switch, so at main its decode runs with the rule on.
- **The tie order alone moves next-token choices far past the bar:**
  - on (r-drop) step 16,384's fixed sequences, the D11 readouts with the two orders differ in top-1 at 174 (s1) and 227 (s3) of 1,536 positions (claude-238);
  - on claude-146's checkpoints on DEV, they differ in `next` at 590, 815 and 578 of 5,795 positions (claude-240).
- **With the same tie rule on both sides,** the integer read meets parity with its float forward on DEV at all three of claude-146's seeds: d of 47, 72 and 49 of 5,795 (claude-240, row P).
- **claude-232's MISS against F:** first-token disagreement of 19, 25 and 26 of 310 on DEV, and 291, 425 and 1,138 of 1,202 on p-w, against bars of 9 and 36.

So against F, an exact port of the read cannot meet the bar. A MISS against F does not tell an integer fault from the tie order. That makes it an instrument problem, not a result about the model (CHARTER §4.2, L-007).

## The options

- **A. Keep F as the reference.** The bar stays unreachable by construction, and every miss stays unclassified.
- **B. Replace F with F′.** F′ is the same decode as claude-220's arm G (greedy, memory on): the same checkpoints, prompts, panels and flags, run again at this unit's head on the GPU, with the tie rule on. F stays as a reading.
  - F′ is "the trained model as the float side now serves it". Its read keeps ties the way the integer read does.
  - It is repeatable only if GPU greedy is. The sheet's I-Repeat checks that, and a failure stops the run as row 0.
- **C. Use the D11 forward as the reference** (`fbs-snapshot`'s readouts), as claude-227 and claude-240 did.
  - That checks the integer port against its own float twin, not against the trained model's decode.
  - Done-when 3 asks whether the integer artifact decodes as the trained model does. C does not answer that.

## Constraints

- The bar's value, its grader and its six cells stay claude-232's: first-token disagreement ≤ ⌊0.03 n⌋, so 9 on DEV and 36 on p-w.
- F′'s inputs are claude-220's, all sealed: the checkpoints, the prompts and the panels. Only the build moves.
- The float side stays on the GPU (CHARTER §3.5). The integer decodes stay on the laptop under the owner's 31 Oct allowance.
- F′ also differs from F by the decode-path changes since `eb762e602` (claude-224 to 240, in `in_space_long.rs`, `train-in-space.rs` and `flock_read_hamming.rs`). F′ against F is read, with that confound named. It does not decide anything.

## Recommendation: B

The bar's purpose is that the served integer artifact decodes as the trained model does. The trained model's decode now keeps ties by the integer rule, so F′ is that decode; F is not.

The change is narrow:
- the same bar value, grader, checkpoints and prompts;
- one new GPU decode, with a repeatability check.

The item's done-when 3 text is amended to name F′, with F kept as a reading. The amendment cites this card.

## The decision review

To be filled in after the fresh decision review.

## What a reversal costs

- **Reverting to A** costs only the text: F's replies stay sealed, and the grader runs on any pair.
- **Reverting to C** needs a GPU readout run under its own sheet.
- F′'s run costs about one RTX 5090 for under an hour; the sheet gives the estimate.
