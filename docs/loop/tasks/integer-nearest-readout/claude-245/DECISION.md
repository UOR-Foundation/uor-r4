# Decision card: the parity bar, re-derived from the measured residual (integer-nearest-readout, claude-245, 1 Nov 2026)

Lab: claude. ORDER: [claude-245](ORDER.md). This card changes the bar of the item's done-when 3, which is a change to a bar (CHARTER §5.1, §5.2). claude-244's sheet names it as its next unit: the near-tie residual row, with pooled r₊ ≤ 3%, "re-derives the first-token bar from this measured per-position residual".

## The question

**What must R, the raw integer export, show against F′, the trained model's own decode, for done-when 3 to be met?**

Done-when 3's bar is first-token disagreement ≤ ⌊0.03 n⌋ in each cell: 9 of 310 on DEV and 36 of 1,202 on p-w. It was "registered in the sheet against the exporter's known rounding residual (claude-227: 0.33% of sequence positions)", measured on the snapped export against a D11 forward that snaps too. claude-243's decision card moved both sides to R against F′, and kept the 3% without re-deriving it.

## What is measured

- **claude-243, free decode, R against F′** ([REPORT](../claude-243/REPORT.md)). First-token disagreement is 4, 4 and 5 of 310 on DEV (it meets the bar), and 64, 47 and 78 of 1,202 on p-w (a MISS at every seed).
- **claude-244, teacher forcing, R against F′'s sealed rows** ([REPORT](../claude-244/REPORT.md)). The disagreements sit where R's own top two are nearly tied. Those are the positions where the trained model is itself close to indifferent.
  - Each cell's κ, the share of disagreements at R's lowest-margin decile corrected for chance, is 0.856, 0.907 and 0.838 on p-w, and 0.972, 0.903 and 0.919 on DEV. The sheet registered κ ≥ 4/9, clear of chance at 3 SE, as the near-tie residual row; all six cells meet it.
  - The median margin at a p-w disagreement is 1,568 to 1,778 (Q12 units of −L1). The median over all positions is 128,152 to 183,087.
  - No pick in any cell was an exact tie.
- **The same artifact flips more picks on the panel with more near ties.**
  - At every position, R disagrees with F′ on 1.57% of DEV (195 of 12,441) and 2.98% of p-w (1,246 of 41,793).
  - The 10th-percentile margin G₁₀ is 22,526 to 30,228 on DEV and 7,760 to 10,112 on p-w, so p-w's is 2.5 to 3.1 times smaller at each seed.
  - A fixed share of disagreements therefore reads the panel's tie density as well as the port.
- **The opening is worse than a later position, though not 2×.** On pooled p-w, r₀ is 132 of 3,201 (4.12%) and r₊ is 1,114 of 38,592 (2.89%). That is 1.43×, which by the REPORT's counts is 3.4 SE apart. On DEV, r₀ is 1.29% and r₊ is 1.59%.
  - claude-244 did not read the margin by offset, so whether the opening has more near ties or an opening-specific cause is not known.
- **p-w seed 3** has r₊ = 397 of 12,864 (3.09%), the only cell over 3%. It also has the smallest G₁₀ of the three p-w seeds (7,760).
- **151 of the 1,246 pooled p-w disagreements (12.1%) sit at clear margins,** above their cell's G₁₀, up to g = 125,797. DEV has 11 of 195. A rounding residual at their own position does not explain them; they are untraced.
- **The cut windows** (rows whose prompt is cut, so it does not start at `<|bos|>`) were left out of claude-244's counts. claude-243's bar counted their first tokens. Their openings disagree on 23, 13 and 21 of 135 p-w rows per seed (10% to 17%), against 41, 34 and 57 of 1,067 for the counted rows (3.2% to 5.3%).
- **The rounding residual's size is not measured.** F′'s sealed rows carry its `nearest` and its `targets`, but no scores. So claude-244's reading is one-sided: R's margin is known at every position, F′'s is not.
- **Unread:** no record counts how often R's pick, or F′'s, equals the reference token. The ORDER's cancel probe (b) checks this. The one such count is claude-205's, of the float model alone, at an older build.

## The options

- **A. Keep ⌊0.03 n⌋ on first tokens.** The p-w MISS stands. The item closes only if the residual falls below the bar, through a finer fixed point or a located fault.
  - The same artifact meets A on DEV and misses it on p-w. The measured cause, the panel's tie density, is not a fault, so A grades the panel as much as the port.
  - Its basis is gone: claude-227's 0.33% was the snapped export against a snapped forward. R against F′ flips 2.89% of later p-w positions, about 9 times that.
- **B. Keep a rate bar and re-derive its number from the residual.** For example: in each cell, the opening disagrees no more than its own later positions allow, d₀ ≤ n₀·r₊ + 3·√(n₀·r₊(1 − r₊)), under an absolute ceiling on r₊.
  - A rate cannot tell openings that sit nearer a tie from an opening fault, and claude-244 did not read the margin by offset.
  - A rate test relative to r₊ passes a uniform fault, so it needs an absolute ceiling, and that ceiling has no measured basis.
  - r₊ comes from the same rows the bar would grade.
  - For the record: read on claude-244's counts, B's opening test fails pooled p-w (132 against a bound of 121) and seed 3 (57 against 50), and passes seeds 1 and 2. That is not the reason it is set aside; the reasons above are.
- **C. Margin-aware identity, against a measured residual ε.** No disagreement may sit where the margin exceeds 2ε.
  - That needs F′'s top-two scores at every position, which a GPU run must write. The writer, `in_space_long.rs`, is outside this item's scope.
  - A sign flip in a Hamming code would put a large change in a single score, so ε would be set by the worst position, not a typical one.
  - Kept as a lead: it bounds the residual from both sides.
- **D. Quality parity, under the rounding signature.** R must pick the reference token as often as F′ does, and its differences must sit at near ties. Defined exactly in the recommendation.

## Recommendation: D

**The bar's purpose:** the served integer artifact must serve the trained model. Every finite-precision port flips some picks where the trained model's top two are nearly equal. Those flips change which near-equal token comes out; they cost no quality, and their number follows the panel. A fault shows instead as picks lost where the trained model was right, or as differences at clear margins. D tests for exactly those, on a quantity no one has read.

**The bar, under teacher forcing at every position of the rows claude-244 counted.** It is read per cell (split × seed) and pooled per panel, on R against F′, with t the reference token at each position:
- a = positions where R's pick equals t and F′'s nearest does not;
- b = positions where F′'s nearest equals t and R's pick does not.
- **Q1, no detectable loss:** b − a ≤ 3·√(a + b), compared in integers: b ≤ a, or (b − a)² ≤ 9(a + b).
- **Q2, a bounded loss:** b − a ≤ ⌊n / 200⌋, at most half a point of next-token accuracy over the n positions read.
- **Q3, the opening:** Q1 again, on offset 0 alone. claude-244 found the opening 3.4 SE above the later positions; Q3 asks whether that costs anything.
- **Q4, the rounding signature:** κ ≥ 4/9, clear of chance at 3 SE. This is claude-244's registered row, already met in all six cells. It is carried, not re-read; the next unit's sheet cites claude-244's sealed grade.
- **The cut windows** are graded as a stratum of their own, pooled per panel (a cell holds too few for power), on Q1 over all their positions and Q3 at their openings. They are not pooled into the counted rows, since their openings disagree 2.9 to 4.4 times as often.
- **Pass:** Q1, Q2 and Q3 hold in every cell, in both pooled panels and in the cut-window stratum, and Q4 stands.

**What the bar says about the two named cases:**
- **p-w seed 3** gets no allowance. Its r₊ of 3.09% is no longer a criterion, because a rate reads tie density. Like every cell, it must pass Q1 to Q3 on its own, and Q4 holds for it at κ = 0.838.
- **The 151 clear-margin disagreements** count in a and b like every other. If they cost R picks that F′ gets right, Q1 on pooled p-w catches it at about that size: with the near ties balanced, b − a ≈ 151, against a bound of about 3·√1,246 ≈ 106. A pass does not explain them, and their trace stays on the item's list whatever the bar reads.

**The next unit's decision rows:**
- **PASS:** done-when 3 is met for R. The item names R's recipe (`fbs-export --raw-gates --raw-transition`) as the served artifact's, and closes once its other done-whens are checked at that head.
- **LOSS** (any of Q1 to Q3 fails, with every instrument check held): R loses picks that F′ gets right beyond what rounding does. That is a port fault to locate. The item stays `ready`, and the trace starts with the stratum that failed.
- **Row 0:** an instrument check fails, such as alignment, coverage or a target that is not the reference token. Nothing is read, and the instrument is repaired.

## Constraints

- **The bar is fixed before it is read.** The next unit's sheet registers Q1 to Q4 and the stratum rule verbatim, with a fresh FIT. Changing them needs a new card.
  - This unit counts no pick against the target. Nobody knows whether D passes: it can fail.
- **It costs nothing to read.** The inputs are claude-244's sealed forced decodes and claude-243's sealed F′ rows, both read back from the store. The grade is a count on the laptop: grading stored outputs, CHARTER §3.5.
- **What stays recorded:**
  - claude-243's free-decode first-token and whole-reply agreement stay reported as readings;
  - claude-243's MISS on the old bar is recorded as it is, not relabelled;
  - the old bar's numbers stay in the item's history.
- **"Done" changes meaning, and the item says so.** It now means "R picks the reference as often as the trained model does, and differs from it only where the trained model is nearly indifferent". It no longer means "R's first tokens equal F′'s on 97% of rows".
- **Power and false fails.**
  - Q1 detects a net loss of about 3·√(a + b) positions. Since a + b is at most the cell's disagreements, that is at most about 60 in a p-w cell, 20 to 27 in a DEV cell, and 106 of 41,793 (0.25%) on pooled p-w. Q2's half point is 69 in a p-w cell and 20 in a DEV cell, so it is the tighter test on DEV.
  - With no loss, each one-sided 3-SE test fails by chance with probability about 0.13%. There are 20 of them: Q1 and Q3 in six cells, two pooled panels and two cut-window strata. By a union bound, the chance that any fails is under 3%.
- **The residual risk.** A fault confined to near ties that costs no accuracy would pass, for example a bias that breaks near ties toward frequent tokens. It changes outputs only where the trained model is close to indifferent, and the item records it as a known blind spot. C, if it is ever run, would close it.
- **Nothing in CHARTER §3 or the caps is touched.**

## The decision review

To be recorded: round 1, and round 2 if needed.

## What a reversal costs

- **Reverting to A** costs only the text. claude-243's numbers are sealed, and the old bar is a count on them.
- **Moving to C** costs a GPU run under its own sheet, plus a scope change for the score writer.
- **D's grade itself** is a laptop count on sealed files, minutes long. It can be re-read under any revised bar.
