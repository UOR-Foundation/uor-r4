# Decision card: the parity bar, re-derived from the measured residual (integer-nearest-readout, claude-245, 1 Nov 2026)

Lab: claude. ORDER: [claude-245](ORDER.md). This card changes the bar of the item's done-when 3, which is a change to a bar (CHARTER §5.1, §5.2). claude-244's sheet names it as its next unit: the near-tie residual row, with pooled r₊ ≤ 3%, "re-derives the first-token bar from this measured per-position residual".

## The question

**What must R, the raw integer export, show against F′, the trained model's own decode, for done-when 3 to be met?**

Done-when 3's bar is first-token disagreement ≤ ⌊0.03 n⌋ in each cell: 9 of 310 on DEV and 36 of 1,202 on p-w. It was "registered in the sheet against the exporter's known rounding residual (claude-227: 0.33% of sequence positions)", measured on the snapped export against a D11 forward that snaps too. claude-243's decision card moved both sides to R against F′, and kept the 3% without re-deriving it.

## What is measured

- **claude-243, free decode, R against F′** ([REPORT](../claude-243/REPORT.md)). First-token disagreement is 4, 4 and 5 of 310 on DEV (it meets the bar), and 64, 47 and 78 of 1,202 on p-w (a MISS at every seed).
- **claude-244, teacher forcing, R against F′'s sealed rows** ([REPORT](../claude-244/REPORT.md)). The disagreements sit where R's own top two are nearly tied. F′'s own margin there is not measured (see below), so this reading is one-sided.
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
- **D. Quality parity, plus an exact account of the clear margins.** R must pick the reference token as often as F′ does, and its differences must sit at its own near ties. Every disagreement at a clear margin must also be accounted for by an exact trace, under a rule that its own card fixes. Defined exactly in the recommendation.

## Recommendation: D

**The bar's purpose:** the served integer artifact must serve the trained model.
- Every finite-precision port flips some picks where the two sides' top two are nearly equal. Those flips change which near-equal token comes out, and their number follows the panel.
- A fault shows instead in one of two ways:
  - as picks lost where the trained model was right, which Q1 to Q3 test, on a quantity no one has read;
  - as differences at clear margins. A count of those has already been read (151 and 11), so it cannot be a bar. Q5 accounts for them by an exact trace instead.

**The positions.** These are claude-244's counted rows, those whose prompt starts at `<|bos|>` (`prompt[0] == 0`), each at offsets i = 0 to n − 1. Here n = min(|reference|, 192 − |prompt| + 1), which in a counted row is the whole reference. F′'s nearest and target are read at `position + i` (`grade-cell.jq:36–42`). The target t equals the reference token there (`targets_wrong` is 0 in every cell).

**The counts.** In each cell (split × seed), R is read against F′:
- a = positions where R's pick equals t and F′'s nearest does not;
- b = positions where F′'s nearest equals t and R's pick does not.
- For a row k, a_k and b_k count that row's positions. V = Σ_k (b_k − a_k)², summed over the rows, so a reply's positions are not treated as independent.
- **A row is identified by its `position`,** the field `files_ok` aligns across files. In a pooled panel, the rows with one `position` at the three seeds form one row k. Row 0 checks that `position` is unique within each seed and carries the same prompt and reference at every seed.

**The bar:**
- **Q1, no detectable loss:** b ≤ a, or (b − a)² ≤ 9·V. Compared in integers, in every cell and in both pooled panels.
- **Q2, a bounded loss:** b − a ≤ ⌊n / 200⌋, at most half a point of next-token accuracy over the n positions read, in every cell and in both pooled panels.
  - Q2 also covers what Q1 cannot see: a loss confined to 9 rows or fewer never fails Q1, since (Σ_k d_k)² ≤ m·Σ_k d_k² over m rows.
- **Q3, the opening, read on free decode**, from claude-243's sealed greedy replies.
  - **R is right at the opening** when its greedy reply is not empty and its first token equals reference[0]. **F′ is right** when its greedy first token equals reference[0].
  - **It is like for like.** `files_ok` checks that both greedy files carry the row's own prompt and reference (`grade-cell.jq:20–22`). Row 0 also checks, in F′'s sampler, that its decode conditions only on the stored prompt.
  - a₀, b₀ and V₀ are formed as above, one position per row, and Q3 is Q1's test on them.
  - **It is tested in six strata:** the counted rows in each p-w cell, the counted rows pooled per panel (DEV and p-w), and p-w's cut windows pooled.
  - Two kinds of stratum cannot fail, so they are reported as readings: a DEV cell (at most 4 openings disagree) and DEV's cut windows (at most 2 disagree). DEV's pooled counted rows (11 disagree) can fail only if almost every disagreement goes F′'s way.
  - On counted rows this is the same opening as teacher forcing's offset 0. claude-244's grader checks both sides there: I-Align for F′ and I-Forced-Greedy for R.
- **Q4, the rounding signature:** κ ≥ 4/9, clear of chance at 3 SE, in every cell.
  - For claude-244's R this is its registered row, already met in all six cells. It is carried, and the next unit's sheet cites claude-244's sealed grade.
  - **For any other artifact or recipe,** such as a finer fixed point or a fix to a located fault, Q4 is read again.
- **Q5, the clear margins: required, with its rule fixed by its own card.** Done-when 3 also requires an exact trace accounting for every counted disagreement where R's margin exceeds its cell's G₁₀: for this R, 151 on p-w and 11 on DEV, plus the cut windows' own if they join (below).
  - **This card does not approve the trace's rule.** Round 2 showed that a rule written here was not yet decidable.
  - **The Q5 card must meet four requirements:**
    1. Its rule, and any bound it uses, is fixed before either side's values are read.
    2. It traces each position's whole causal prefix, not the position alone.
    3. It has its own Row 0: the float side reproduces F′'s nearest, the integer side reproduces R's pick, and a rerun matches. A position that cannot be traced is an instrument repair, not a fault.
    4. It states its writer, any scope change, and its cost.
  - **The leading proposal, for that card:** a three-way reading against R's float twin T, the same artifact evaluated in float by `fbs-snapshot` (in the item's scope).
    - If T agrees with F′ and not with R, the cause is R's integer arithmetic, anywhere in the causal prefix. If T agrees with R and not with F′, the cause is the exported structure.
    - It needs no rounding bound. Whether `fbs-snapshot` carries the memory and the read on this path is that card's first check.
  - Q5 is read again for any other artifact or recipe.
- **The cut windows' later positions.** Row 0 reads I-Align on the cut windows too: F′'s rows against F′'s greedy reply, wherever its greedy prefix equals the reference.
  - **If it holds** (0 violations, with positions checked), F′'s rows read the same window there. p-w's cut windows then join Q1 as a pooled stratum, and their clear-margin disagreements join Q5's set.
  - **If not,** their later positions stay readings with no bar, and only their clear-margin opening disagreements join Q5's set.
- **Row 0** is the instrument checks. The next unit's grader runs them all again before it reads anything. If any fails, nothing is read and the instrument is repaired. They are:
  - claude-244's `files_misaligned`, `n_wrong`, `counted_short`, `forced_greedy_violations` (with `forced_greedy_checked` > 0), `align_violations` (with `align_checked` > 0), `targets_wrong`, `coverage_missing`, `margins_negative` and `runner_up_wrong`;
  - its sheet's I-Inputs, I-NoKey and I-Exit;
  - the uniqueness of `position`;
  - the check on F′'s sampler.
- **Pass:** all of the following hold.
  - Q1 holds in every cell, in both pooled panels, and in p-w's cut windows if they join.
  - Q2 holds in every cell and in both pooled panels.
  - Q3 holds in its six tested strata.
  - Q4 stands.
  - Q5 holds under its card's rule.

**What the bar says about the two named cases:**
- **p-w seed 3** gets no allowance. Its r₊ of 3.09% is no longer a criterion, because a rate reads tie density. Like every cell, it must pass Q1 to Q3 on its own and its share of Q5. Q4 holds for it at κ = 0.838.
- **The 151 clear-margin disagreements** are not excused by any count.
  - Where one side picks the reference, they enter a and b.
  - Where neither does, Q1 to Q3 cannot see them, so Q5 accounts for every one, and the item is not done until it holds.
  - Their trace is the item's first lead in any case.

**The order of the next units:**
1. **Q1 to Q3,** with Row 0, are a count on sealed files, run on the laptop at $0 under a sheet with its own FIT.
   - **LOSS:** some test fails, with Row 0 held, so b − a is beyond the null. A LOSS is not yet called a port fault.
     - Pure rounding can itself push b − a a little above zero: a flip goes to R's runner-up, which is the reference less often than the top pick is.
     - So the trace classifies the failing stratum first (CHARTER §4.2). The item stays `ready`.
   - **Held:** go on to Q5.
2. **Q5's card,** then its sheet with a FIT and a pre-run review, then the trace.
   - **PASS** (Q5 holds as well): done-when 3 is met for R. The item names R's recipe (`fbs-export --raw-gates --raw-transition`) as the served artifact's, and closes once its other done-whens are checked at that head.
   - **A fault found:** it is located, fixed with a test, and then Q1 to Q5 are read again on the new artifact.

## Constraints

- **The bar is fixed before it is read.** The next unit's sheet registers Q1 to Q4, the positions, the strata, the cut-window rule and Row 0 verbatim, with a fresh FIT. Q5's card registers Q5. Changing any of them needs a new card.
  - This unit counts no pick against the target, and traces no position. Nobody knows whether D passes: it can fail.
- **The cost.**
  - Q1 to Q3 cost nothing. The inputs are claude-244's sealed forced decodes, claude-243's sealed greedy replies of R and F′, and claude-243's sealed F′ rows, all read back from the store. The grade is a count on the laptop: grading stored outputs, CHARTER §3.5.
  - Q5's cost, its writer and any scope change are stated in its card. With the leading proposal, it is a GPU reading of `fbs-snapshot` at the traced positions and their prefixes.
- **What stays recorded:**
  - claude-243's free-decode first-token and whole-reply agreement stay reported as readings;
  - claude-243's MISS on the old bar is recorded as it is, not relabelled;
  - the old bar's numbers stay in the item's history.
- **"Done" changes meaning, and the item says so.** It now means three things:
  - R picks the reference as often as the trained model does;
  - its differences from the trained model sit mostly where R's own top two are nearly tied. κ ≥ 4/9 puts at least half of them in R's lowest-margin decile;
  - every difference at a clear margin is accounted for by the trace.
  It no longer means "R's first tokens equal F′'s on 97% of rows".
- **Power and false fails.**
  - Q1 detects a net loss of about 3·√V positions. If the rows were independent, V would be about a + b, and a + b is at most the cell's disagreements. That gives a threshold of at most 59 to 64 positions in a p-w cell, 20 to 27 in a DEV cell, and 106 of 41,793 (0.25%) on pooled p-w. Positive correlation within a reply makes V larger, which widens the test by an estimate of what the correlation warrants (a variance clustered by row).
  - Q2's half point is 69 in a p-w cell and 20 in a DEV cell, so it is the tighter test on DEV.
  - Q3 detects a net opening loss of about 17 to 23 of 1,067 in a p-w cell (1.6 to 2.1 points), 34 of 3,201 on pooled p-w (1.1 points) and 23 of 405 in p-w's cut windows. A smaller opening loss passes.
  - **The false-fail rate.**
    - With no loss, each one-sided 3-SE test fails by chance with probability about 0.135%. There are at most 15 such tests: Q1 in up to nine strata and Q3 in six.
    - Q2 on the three DEV cells adds about 1.3%, 0.8% and 0.2%, even if every disagreement fell in a or b.
    - That is about 4.3% in all, under independence. Q2 is not clustered, so correlation within a reply raises its share.
- **The residual risks:**
  - **A fault confined to near ties that costs no accuracy** would pass, for example one that breaks near ties toward frequent tokens. It changes outputs only where R's top two are nearly tied. The item records it as a known blind spot; C, if it is ever run, would close it.
  - **Teacher forcing reads each position given the reference prefix,** so a fault in the serving loop after offset 0 would not show in Q1 or Q2.
    - In part, claude-244's I-Forced-Greedy covers it: R's forced picks equal its own free-decode greedy along every shared prefix, with 0 violations in 9,488 positions.
    - Whole-reply agreement stays a reading.
  - **The cut windows' later positions** are unbarred if I-Align fails there.
- **Nothing in CHARTER §3 or the caps is touched.**

## The decision review

- **Round 1, at `190eb9e5a`: REVISE.** A fresh-context reviewer read the card, the ORDER, claude-244's REPORT, its sheet and grader, and the sealed files' structure, under a rule to count no pick against a target.
  - **It confirmed:**
    - every number in the card against claude-244's REPORT;
    - the 3.4 SE (unpooled), B's bounds of 121 and 50 (from 120.8 and 49.9), the G₁₀ ratio of 2.52 to 3.10, ⌊n/200⌋, 3·√1,246 ≈ 106, and the union bound;
    - that no record it read counts a pick against the target;
    - that B is set aside for reasons independent of its failing;
    - that §3 and the caps are untouched.
  - **Major 1:** the cut windows were graded on teacher forcing, where F′'s rows may read a different window, so a LOSS there could be an instrument effect. Applied: the opening is read on free decode, where both sides start from the same cut prompt, and the cut windows' later positions get the I-Align rule above.
  - **Major 2:** D did not test clear-margin differences. A flip where neither side picks the reference is invisible to Q1 to Q3, and Q4, already met, cannot fail. Applied: Q5, an exact trace of every clear-margin disagreement. It is not a count bound, because the count (151 and 11) is already read.
  - **Major 3:** Q4 is a property of this artifact. Applied: Q4 and Q5 are read again for any other artifact or recipe.
  - **Minors 4 to 8 and Nit 9** were applied: DEV's vacuous Q3, a variance clustered by row, the 59-to-64 figure, each SE's convention, the row rule, Row 0 named, and the serving-loop risk.
- **Round 2, at `44f8f9703`: REVISE.** A second fresh-context reviewer checked round 1's fixes and the new claims, under the same rule.
  - **It confirmed:**
    - Q1 to Q4 sound, in integers, and pre-registrable;
    - V a valid clustered estimate under the null;
    - Q3 on counted rows equal to teacher forcing's offset 0, with empty replies wrong on both sides;
    - the power figures;
    - round 1's fixes applied.
  - **Major 1:** Q5's rule was not decidable. Its fault class also caught instrument failures, its bounds' method was open, it traced one position rather than its causal prefix, and its bounds could be chosen after R's side was read. Applied: Q5's rule is taken out of this card and given to its own, which must meet the four requirements above. The three-way reading against R's float twin is recorded as the leading proposal, not approved.
  - **Major 2:** Q5's writer and scope were unstated, though the same point was held against C. Applied: the Q5 card states its writer, scope and cost. The leading proposal uses `fbs-snapshot`, which is in the item's scope.
  - **Major 3:** the cut windows were barely barred. Applied, as the reviewer proposed: I-Align on the cut windows joins Row 0's readings. If it holds, p-w's cut windows join Q1 and Q5; if not, their clear-margin openings join Q5.
  - **Minors 4 to 9 and Nit 10** were applied:
    - a LOSS is classified by the trace before it is called a fault, since pure rounding can push b − a above zero;
    - DEV's cut-window Q3 is a reading, leaving six tested strata;
    - the false-fail rate is stated as about 4.3% under independence, higher for Q2;
    - the join key, `position`, is named, with a uniqueness check;
    - the near ties are named as R's own, since F′'s margin is not measured;
    - like for like is cited to `files_ok`, with a check on F′'s sampler;
    - Q2 covers a loss confined to a few rows.
- **Approved** by the claude lab's top session, 1 Nov 2026, after both rounds, for Q1 to Q4, the positions, the strata, the cut-window rule and Row 0, and for Q5 as a requirement. Q5's rule is not approved here.
  - The two rounds allowed are spent, so round 2's fixes are checked by this PR's exact-head review.
  - Logged on #3462 for Casey, who can reverse it.

## What a reversal costs

- **Reverting to A** costs only the text. claude-243's numbers are sealed, and the old bar is a count on them.
- **Moving to C** costs a GPU run under its own sheet, plus a scope change for the score writer.
- **D's Q1 to Q3** are a laptop count on sealed files, minutes long, and can be re-read under any revised bar. Q5's trace is kept, whatever bar it is later read against.
