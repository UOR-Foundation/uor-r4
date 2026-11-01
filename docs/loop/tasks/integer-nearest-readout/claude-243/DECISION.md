# Decision card: what end-to-end parity is read on (integer-nearest-readout, claude-243, 1 Nov 2026)

Lab: claude. ORDER: [claude-243](ORDER.md), done-when 1. The item's done-when 3 names the reference. This card changes both sides of the bar, which is a change to a bar's scope (CHARTER §5.1, §5.2).

## The question

**On which integer artifact, and against which float greedy replies, is the end-to-end parity bar read?**

Done-when 3 says: "Integer greedy of (r-drop) step 16,384, memory on, against claude-220's stored float greedy on DEV and p-w at seeds 1–3." It does not name the export mode. claude-232 read it on the default export, which snaps.

## What is measured

**The tie order, on the float side:**
- **claude-220's stored replies** (F) were decoded at `eb762e602`.
  - There, the Hamming read keeps its 16 nearest by one GPU `arg_sort` (`flock_read_hamming.rs:714`), whose own doc says "Ties may come out in another order" (`:689`).
  - The integer read keeps ascending distance, then ascending `s`.
- **The float read has kept ties the integer way by default since claude-239** (`cef022733`, `TIE` starts on). `train-in-space` has no switch, so at main its decode runs with the rule on.
- **The tie order alone moves next-token choices far past the bar:**
  - on (r-drop) step 16,384's fixed sequences, the D11 readouts with the two orders differ in top-1 at 174 (s1) and 227 (s3) of 1,536 positions (claude-238);
  - on claude-146's checkpoints on DEV, they differ in `next` at 590, 815 and 578 of 5,795 positions (claude-240).
- **With the same tie rule on both sides,** the integer read meets parity with its float forward on DEV at all three of claude-146's seeds: d of 47, 72 and 49 of 5,795 (claude-240, row P).

**The snap, on the integer side:**
- `fbs-export` snaps the trunk's gates and transition to the 120 icosians by default. claude-232's artifacts (S) are snapped; the trained float forward is not.
- claude-233's diagnostic, not registered: the snapped query sits a median 46% (in L1) from the trained one.
- `fbs-export --raw-gates --raw-transition` writes them unsnapped (`fbs-export.rs:9`, `:109–118`), and the engine normalizes them at serving (claude-181).

**claude-232's MISS against F:** first-token disagreement of 19, 25 and 26 of 310 on DEV, and 291, 425 and 1,138 of 1,202 on p-w, against bars of 9 and 36. It was read on S.

So against F, an exact port of the read cannot meet the bar, and S carries a known gap of its own. A MISS of S against F does not tell an integer fault from the tie order or from the snap. That makes it an instrument problem, not a result about the model (CHARTER §4.2, L-007).

## The options

- **A. Keep S against F.** The bar stays unreachable by construction, and every miss stays unclassified.
- **B. Read the bar on R against F′.** Two changes, one on each side:
  - **The reference: F becomes F′.** F′ is the same decode as claude-220's arm G (greedy, memory on): the same checkpoints, prompts, panels and flags, run again at this unit's head on the GPU, with the tie rule on. F stays as a reading.
    - F′ is "the trained model as the float side now serves it". Its read keeps ties the way the integer read does.
    - It is repeatable only if GPU greedy is. The sheet's I-Repeat checks that, and a failure stops the run as row 0.
  - **The integer side: S becomes R.** R is the same snapshot exported with `--raw-gates --raw-transition`, so its trunk is the trained one, not the snapped one.
    - S against F′ is read alongside, with no bar. It is the control that isolates the snap.
- **C. Use the D11 forward as the reference** (`fbs-snapshot`'s readouts), as claude-227 and claude-240 did.
  - That checks the integer port against its own float twin, not against the trained model's decode.
  - Done-when 3 asks whether the integer artifact decodes as the trained model does. C does not answer that.

## Constraints

- The bar's value, its grader and its six cells stay claude-232's: first-token disagreement ≤ ⌊0.03 n⌋, so 9 on DEV and 36 on p-w.
- F′'s inputs are claude-220's, all sealed: the checkpoints, the prompts and the panels. Only the build moves. R's inputs are claude-227's sealed full snapshots and claude-232's configs and prompt files.
- The float side stays on the GPU (CHARTER §3.5). The integer decodes stay on the laptop under the owner's 31 Oct allowance.
- **"Done" applies to the raw artifact.** If R passes, done-when 3 is met for `fbs-export --raw-gates --raw-transition`, and the item records that recipe as the served artifact's.
  - The default export still snaps, and point-key-integer's artifacts use it. Making raw the default, or the snap faithful, is a follow-up the item records; this unit does not change the default.
  - If S also passes, the snap is shown harmless here, and either export is the served one.
- **The bar's margin without the snap.** It was registered against claude-227's snapped residual (0.33% of sequence positions, against a D11 forward that snaps too). The raw export's rounding residual against the trained forward is not measured. The 3% bar is kept, not re-derived; the sheet says so, and a narrow miss is located before it is read.
- **F′ also differs from F by the code since `eb762e602`** on the three decode files: `c3d5cb684` (claude-220's own merge, after its decode build), `18c4c3da7` (claude-224), `62e3cfed4` (claude-235), `ebe0930ba` (claude-237), `fe1619192` (claude-238) and `cef022733` (claude-239).
  - Read from the code, not tested: on arm G's path, the only change is the tie rule. The `in_space_long.rs` changes are in the served arm, the opening table and the graders. claude-238's `as_trained` keeps the read's rotations unsnapped, which `in_space_long.rs` (itself `HamiltonActive`) already did.
  - F′ against F is read with this confound named. It decides nothing, and the REPORT does not credit the tie rule alone with it.

## Recommendation: B

The bar's purpose is that the served integer artifact decodes as the trained model does.
- The trained model's decode now keeps ties by the integer rule, so F′ is that decode; F is not.
- The trained model's trunk is not snapped, so R is its integer port; S carries a second, known change on top.

What moves, and what does not:
- the bar value, the grader, the checkpoints and the prompts are unchanged;
- the reference becomes one new GPU decode, with a repeatability check;
- the integer side becomes one new laptop export, with S read beside it as the snap's control.

The item's done-when 3 text is amended to name R and F′, with S and F kept as readings. The amendment cites this card.

## The decision review

- **Round 1, at `30f882edc`: REVISE.** A fresh-context reviewer read the card, the ORDER and the code at `eb762e602` and origin/main `abbfb0001`.
  - **It confirmed:**
    - the tie lines at `eb762e602`, and `TIE` on at main, with no caller in `train-in-space`;
    - that ±1 codes make the rounded bit counts change only the order of equal distances;
    - claude-232's counts against the item's.
  - **It did not check** claude-238's and claude-240's numbers.
  - **Major 1:** the bar moves on both sides, S to R as well as F to F′, and the card named only one. Applied above: option B names both changes, the constraints say "done" applies to the raw artifact and name S against F′ as the snap's control, and the margin without the snap is stated.
  - **Minor 2:** I-Repeat covered one cell of six. The ORDER now repeats seed 3, both its DEV cell and its p-w cell; s3 p-w is the worst cell.
  - **Minor 3:** the confound's commits were loose. They are listed above.
  - **Minor 4:** "the same flags" had no check. The ORDER adds I-Flags.
  - **Minor 5:** F′ against F cannot separate the tie rule from code drift. Stated above.
  - On B itself: an integer fault still shows against F′; the bar value and grader are unchanged; nothing in §3 or the caps is touched. B is the right call over A and C.
- **Round 2:** to be filled in.

## What a reversal costs

- **Reverting to A** costs only the text: S's and F's replies stay sealed, and the grader runs on any pair.
- **Reverting the integer side to S** costs the text too: S against F′ is read in this unit.
- **Reverting to C** needs a GPU readout run under its own sheet.
- F′'s run costs about one RTX 5090 for four hours; the sheet gives the estimate.
