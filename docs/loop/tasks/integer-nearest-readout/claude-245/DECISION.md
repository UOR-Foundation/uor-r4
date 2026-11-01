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
- **The opening is worse than a later position, though not 2×.** On pooled p-w, r₀ is 132 of 3,201 (4.12%) and r₊ is 1,114 of 38,592 (2.89%). That is 1.43×. By the REPORT's counts the two rates are 3.4 SE apart, with a two-sample standard error that does not pool them. On DEV, r₀ is 1.29% and r₊ is 1.59%.
  - claude-244 did not read the margin by offset, so whether the opening has more near ties or an opening-specific cause is not known.
- **p-w seed 3** has r₊ = 397 of 12,864 (3.09%), the only cell over 3%. It also has the smallest G₁₀ of the three p-w seeds (7,760).
- **151 of the 1,246 pooled p-w disagreements (12.1%) sit at clear margins,** above their cell's G₁₀, up to g = 125,797. DEV has 11 of 195. A rounding residual at their own position does not explain them; they are untraced.
- **The cut windows** (rows whose prompt is cut, so it does not start at `<|bos|>`) were left out of claude-244's counts. claude-243's bar counted their first tokens. Their openings disagree on 23, 13 and 21 of 135 p-w rows per seed (10% to 17%), against 41, 34 and 57 of 1,067 for the counted rows (3.2% to 5.3%).
  - claude-244's sheet left them out because "F′'s rows may read those positions from a different window": F′'s teacher-forced rows read the panel's reference stream, while R reads the cut 192-token window.
  - **Free decode compares them like for like.** In claude-243's greedy replies, R and F′ both start from the same cut prompt. Their first tokens, 23 + 41, 13 + 34 and 21 + 57, are exactly claude-243's 64, 47 and 78 (claude-244's I-Offset0).
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
  - For the record: read on claude-244's counts, B's opening test fails pooled p-w (132 against a bound of 121) and seed 3 (57 against 50), and passes seeds 1 and 2. This bound treats r₊ as known, so it is a one-sample test; on that basis the pooled gap is about 4.2 SE. B's failing is not why it is set aside; the reasons above are.
- **C. Margin-aware identity, against a measured residual ε.** No disagreement may sit where the margin exceeds 2ε.
  - That needs F′'s top-two scores at every position, which a GPU run must write. The writer, `in_space_long.rs`, is outside this item's scope.
  - A sign flip in a Hamming code would put a large change in a single score, so ε would be set by the worst position, not a typical one.
  - Kept as a lead: it bounds the residual from both sides.
- **D. Quality parity, plus an exact account of the clear margins.** R must pick the reference token as often as F′ does, its differences must sit at near ties, and every disagreement at a clear margin must be traced to rounding. Defined exactly in the recommendation.

## Recommendation: D

**The bar's purpose:** the served integer artifact must serve the trained model.
- Every finite-precision port flips some picks where the trained model's top two are nearly equal. Those flips change which near-equal token comes out, they cost no quality, and their number follows the panel.
- A fault shows instead in one of two ways: as picks lost where the trained model was right, or as differences at clear margins. Q1 to Q3 test the first on a quantity no one has read. Q5 tests the second by an exact trace, because a count at clear margins has already been read (151 and 11).

**The positions.** These are claude-244's counted rows, those whose prompt starts at `<|bos|>` (`prompt[0] == 0`), each at offsets i = 0 to n − 1. Here n = min(|reference|, 192 − |prompt| + 1), which in a counted row is the whole reference. F′'s nearest and target are read at `position + i` (`grade-cell.jq:36–42`). The target t equals the reference token there (`targets_wrong` is 0 in every cell).

**The counts.** In each cell (split × seed), R is read against F′:
- a = positions where R's pick equals t and F′'s nearest does not;
- b = positions where F′'s nearest equals t and R's pick does not.
- For a row k, a_k and b_k count that row's positions. V = Σ_k (b_k − a_k)², summed over the rows, so a reply's positions are not treated as independent.
- In a pooled panel, the same prompt and reference at different seeds form one row k.

**The bar:**
- **Q1, no detectable loss:** b ≤ a, or (b − a)² ≤ 9·V. Compared in integers, in every cell and in both pooled panels.
- **Q2, a bounded loss:** b − a ≤ ⌊n / 200⌋, at most half a point of next-token accuracy over the n positions read, in every cell and in both pooled panels.
- **Q3, the opening, read on free decode**, so that a cut window compares like for like.
  - R is right at the opening when its greedy reply (claude-243's, sealed) is not empty and its first token equals reference[0]. F′ is right when its greedy first token equals reference[0].
  - a₀, b₀ and V₀ are formed as above, one position per row, and Q3 is Q1's test on them.
  - It is read in three strata: the counted rows in each p-w cell, the counted rows pooled per panel, and the cut windows pooled per panel.
  - In a DEV cell, at most 4 openings disagree, so Q3 there cannot fail. Those cells are reported as readings, not tests.
  - On counted rows this is the same opening as teacher forcing's offset 0: claude-244's I-Offset0 and I-Align tie the two.
- **Q4, the rounding signature:** κ ≥ 4/9, clear of chance at 3 SE, in every cell.
  - For claude-244's R this is its registered row, already met in all six cells. It is carried, and the next unit's sheet cites claude-244's sealed grade.
  - **For any other artifact or recipe,** such as a finer fixed point or a fix to a located fault, Q4 is read again.
- **Q5, the clear margins are explained.** Every counted disagreement where R's margin exceeds its cell's G₁₀ (for this R: 151 on p-w and 11 on DEV) is traced at its position. The trace runs layer by layer, along R's integer path and the trained float forward's path, under its own sheet.
  - That sheet names each integer-valued decision on the served path: a Hamming code bit, a slot the read keeps, a memory key match, and the readout's pick.
  - It derives each one's rounding bound from the export's fixed point before the trace runs.
  - **Rounding-caused:** the first decision that differs does so at a float value within its bound of the decision's threshold, and every earlier value agrees within its bound.
  - **A fault:** anything else.
  - Q5 holds when none is a fault. It is read again for any other artifact or recipe.
- **Pass:** Q1 and Q2 hold in every cell and in both pooled panels; Q3 holds in its seven tested strata; Q4 stands; and Q5 holds.
- **The cut windows' later positions** have no bar, since F′'s rows may read them from a different window. They are reported as readings. A check that both sides read the same window would let a later card put a bar on them.
- **Row 0** is the instrument checks: claude-244's `files_misaligned`, `n_wrong`, `counted_short`, `forced_greedy_violations` (with `forced_greedy_checked` > 0), `align_violations` (with `align_checked` > 0), `targets_wrong`, `coverage_missing`, `margins_negative` and `runner_up_wrong`, and its sheet's I-Inputs, I-NoKey and I-Exit. The next unit's grader runs them all again before it reads anything. If any fails, nothing is read and the instrument is repaired.

**What the bar says about the two named cases:**
- **p-w seed 3** gets no allowance. Its r₊ of 3.09% is no longer a criterion, because a rate reads tie density. Like every cell, it must pass Q1 to Q3 on its own and its share of Q5. Q4 holds for it at κ = 0.838.
- **The 151 clear-margin disagreements** are not excused by any count.
  - Where one side picks the reference, they enter a and b.
  - Where neither does, Q1 to Q3 cannot see them, so Q5 traces every one, and the item is not done until none is a fault.
  - Their trace is the item's first lead in any case.

**The order of the next units:**
1. **Q1 to Q3** are a count on sealed files, run on the laptop at $0, under a sheet with its own FIT.
   - **LOSS** (any of them fails, with Row 0 held): R loses picks that F′ gets right beyond what rounding does. That is a port fault. The item stays `ready`, and the trace starts with the stratum that failed.
   - **Held:** go on to Q5.
2. **Q5** is the trace, under its own sheet with a FIT and a pre-run review, on the GPU for the float side.
   - **PASS** (Q5 holds as well): done-when 3 is met for R. The item names R's recipe (`fbs-export --raw-gates --raw-transition`) as the served artifact's, and closes once its other done-whens are checked at that head.
   - **A fault found:** it is located, fixed with a test, and then Q4 and Q5 are read again on the new artifact, and Q1 to Q3 as well.

## Constraints

- **The bar is fixed before it is read.** The next units' sheets register Q1 to Q5, the positions, the strata and Row 0 verbatim, each with a fresh FIT. Changing them needs a new card.
  - This unit counts no pick against the target, and traces no position. Nobody knows whether D passes: it can fail.
- **The cost.**
  - Q1 to Q3 cost nothing. The inputs are claude-244's sealed forced decodes, claude-243's sealed greedy replies of R and F′, and claude-243's sealed F′ rows, all read back from the store. The grade is a count on the laptop: grading stored outputs, CHARTER §3.5.
  - Q5 needs the float forward's values at 162 positions, a short GPU reading.
- **What stays recorded:**
  - claude-243's free-decode first-token and whole-reply agreement stay reported as readings;
  - claude-243's MISS on the old bar is recorded as it is, not relabelled;
  - the old bar's numbers stay in the item's history.
- **"Done" changes meaning, and the item says so.** It now means three things:
  - R picks the reference as often as the trained model does;
  - it differs from the trained model mostly where the trained model is nearly indifferent;
  - every difference at a clear margin is traced to rounding.
  It no longer means "R's first tokens equal F′'s on 97% of rows".
- **Power and false fails.**
  - Q1 detects a net loss of about 3·√V positions. If the rows were independent, V would be about a + b, and a + b is at most the cell's disagreements. That gives a threshold of at most 59 to 64 positions in a p-w cell, 20 to 27 in a DEV cell, and 106 of 41,793 (0.25%) on pooled p-w. Positive correlation within a reply makes V larger, which widens the test by an estimate of what the correlation warrants (a variance clustered by row).
  - Q2's half point is 69 in a p-w cell and 20 in a DEV cell, so it is the tighter test on DEV.
  - Q3 detects a net opening loss of about 17 to 23 of 1,067 in a p-w cell (1.6 to 2.1 points), 34 of 3,201 on pooled p-w (1.1 points) and 23 of 405 in p-w's cut windows. A smaller opening loss passes.
  - With no loss, each one-sided 3-SE test fails by chance with probability about 0.13%. There are 15 such tests: Q1 in eight strata and Q3 in seven.
  - Q2 on a DEV cell would fail by chance in at most about 1.3% of draws, and that only if every disagreement fell in a or b.
  - By a union bound, the chance of a false fail is under 5%.
- **The residual risks:**
  - **A fault confined to near ties that costs no accuracy** would pass, for example one that breaks near ties toward frequent tokens. It changes outputs only where the trained model is close to indifferent. The item records it as a known blind spot; C, if it is ever run, would close it.
  - **Teacher forcing reads each position given the reference prefix,** so a fault in the serving loop after offset 0 would not show in Q1 or Q2.
    - In part, claude-244's I-Forced-Greedy covers it: R's forced picks equal its own free-decode greedy along every shared prefix, with 0 violations in 9,488 positions.
    - Whole-reply agreement stays a reading.
  - **The cut windows' later positions** are unbarred, as above.
- **Nothing in CHARTER §3 or the caps is touched.**

## The decision review

- **Round 1, at `190eb9e5a`: REVISE.** A fresh-context reviewer read the card, the ORDER, claude-244's REPORT, its sheet and grader, and the sealed files' structure, under a rule to count no pick against a target.
  - **It confirmed:**
    - every number in the card against claude-244's REPORT;
    - the 3.4 SE (unpooled), B's bounds of 121 and 50 (from 120.8 and 49.9), the G₁₀ ratio of 2.52 to 3.10, ⌊n/200⌋, 3·√1,246 ≈ 106, and the union bound;
    - that no record it read counts a pick against the target;
    - that B is set aside for reasons independent of its failing;
    - that §3 and the caps are untouched.
  - **Major 1:** the cut windows were graded on teacher forcing, where F′'s rows may read a different window, so a LOSS there could be an instrument effect. Applied: the opening is read on free decode, where both sides start from the same cut prompt, and the cut windows' later positions are readings with no bar.
  - **Major 2:** D did not test clear-margin differences. A flip where neither side picks the reference is invisible to Q1 to Q3, and Q4, already met, cannot fail. Applied: Q5, an exact trace of every clear-margin disagreement. It is not a count bound, because the count (151 and 11) is already read. The residual risks are completed.
  - **Major 3:** Q4 is a property of this artifact. Applied: Q4 and Q5 are read again for any other artifact or recipe.
  - **Minor 4:** Q3 cannot fail in a DEV cell. Applied: those cells are readings, and the power of the opening test is stated.
  - **Minor 5:** positions within a reply are correlated. Applied: V clusters by row, and in a pooled panel by prompt across seeds.
  - **Minors 6 to 8:** the p-w figure is 59 to 64, not 60. Each SE's convention is now stated. The row rule is cited and Row 0's checks are named.
  - **Nit 9:** the shift to teacher forcing leaves the serving loop after offset 0 partly untested. Applied, under the residual risks.

## What a reversal costs

- **Reverting to A** costs only the text. claude-243's numbers are sealed, and the old bar is a count on them.
- **Moving to C** costs a GPU run under its own sheet, plus a scope change for the score writer.
- **D's Q1 to Q3** are a laptop count on sealed files, minutes long, and can be re-read under any revised bar. Q5's trace is kept, whatever bar it is later read against.
