# REPORT: integer-nearest-readout, claude-245 (the parity bar, re-derived from the measured residual): decision card approved, Q5 to its own card

Lab: claude. Item: [integer-nearest-readout](../../../../roadmap/items/integer-nearest-readout.md) (#3462). Order: [ORDER.md](ORDER.md). Decision card: [DECISION.md](DECISION.md), approved after two fresh decision-review rounds, for Q1 to Q4 with Q5 as a requirement whose rule goes to its own card, and logged on #3462 for Casey, who can reverse it. References #3462.

## Tokens per second

No new speed is claimed: this unit ran nothing. R's decode speed is still claude-243's: 1,204 tok/s median decode on R s1, on this M1.

## What did not work, first

- **No measurement moved.** This unit read no data. claude-243's p-w MISS on the old bar stands, recorded as it is. The new bar is unread, and it can fail.
- **Q5's rule is not approved.** Round 2 found it undecidable as written:
  - its fault class also caught instrument failures;
  - the method for its bounds was open;
  - it traced one position rather than the position's causal prefix;
  - its bounds could be chosen after R's side was read.
  So the rule goes to its own decision card, which must meet four stated requirements. Until it holds, the item cannot close.
- **"Done" now needs more work than a pass on the old bar would have.**
  - Q1 to Q3 are a free laptop count.
  - Q5 accounts for every clear-margin disagreement by an exact trace: 162 on this R's counted rows, plus p-w's cut windows' own. It needs the float side's values on the GPU under its own card and sheet.
- **Both decision-review rounds were REVISE.** The two allowed rounds are spent, so round 2's fixes rested on this PR's exact-head review.
- **The exact-head review was CHANGES, at `05c6de032`.** It found the card silent on what a LOSS means once classified. A later card could then have re-barred a rounding LOSS after the count was read. Now a LOSS of any class fails done-when 3 for that artifact. The review's five Minors and one Nit are applied too (the card's review section).
- **The rounding residual's size is still unmeasured.** Option C, margin-aware identity against a measured ε, needs F′'s top-two scores. Its writer, `in_space_long.rs`, is outside the item's scope, so C is kept as a lead, not taken.
- **The new bar has named blind spots:**
  - a fault confined to near ties that costs no accuracy;
  - the serving loop after offset 0, covered only in part by claude-244's I-Forced-Greedy;
  - the cut windows' later positions, if I-Align fails there.
- **Round 1's three Majors:**
  - the first draft graded the cut windows on teacher forcing, where the two sides may read different windows;
  - it did not test clear-margin flips where neither side picks the reference;
  - it carried Q4 as if it held for any artifact.
  All three are fixed, and so are round 2's three. The card's review section records each one.

## What worked

- **The card answers the sheet's question with a bar that can fail on a quantity no one has read.** Pass needs all of the following:
  - **Q1:** no detectable loss of picks against the reference, with a variance clustered by row;
  - **Q2:** at most half a point of next-token accuracy lost;
  - **Q3:** the same at the opening, on free decode, so that cut windows compare like for like;
  - **Q4:** claude-244's rounding signature, read again for any new artifact;
  - **Q5:** an exact trace accounting for every clear-margin disagreement, under its own card's rule.
- **Round 2 confirmed Q1 to Q4 sound, in integers, and pre-registrable,** with the row-clustered variance a valid estimate under the null.
- **The two cases claude-244 named are treated with no allowance.**
  - p-w seed 3 must pass every test on its own. Its 3.09% is no longer a criterion, because a rate reads the panel's tie density.
  - The 151 clear-margin disagreements are each accounted for by Q5, whatever Q1 to Q3 read.
- **Q5 has a leading proposal that needs no rounding bound.** It reads R against its own float twin T (`fbs-snapshot`, in the item's scope) and F′:
  - T agreeing with F′ puts the cause in R's integer arithmetic;
  - T agreeing with R puts it in the exported structure.
- **Option B, the literal re-derivation of the rate, is set aside on principle.** A rate cannot tell openings nearer a tie from an opening fault. Its failing on seen counts (132 against 121 pooled, 57 against 50 at s3) is recorded and is not the reason.

## What changed

- [DECISION.md](DECISION.md): the card, with both review rounds recorded.
- [ORDER.md](ORDER.md), with the cancel probe's result (instrument class), and this REPORT.
- The item: done-when 3 now names the card's bar. "What is missing" names the next two units, Q1 to Q3 and then Q5. It stays `ready`.
- The roadmap's rendered view is unchanged. `docs/roadmap/README.md` shows only the item's front matter, which this unit did not touch, and pr-check's rendered-lock confirms that.

## Measurements

None. This unit is a decision card: it ran no decode and read no data, and it counted no pick against the reference.
- Every number in the card is quoted from claude-243's and claude-244's REPORTs and sealed grades, or computed from those counts: the 3.4 SE, B's bounds of 121 and 50, the power figures and the union bound.
- Both reviewers recomputed them independently and found them correct.

## Checks run at the head

These were run first at `efa89f6b6`, then again on the tree with the exact-head review's fixes applied (card, item and REPORT text). The results were the same both times:
- `uor-harness loop report --check --order …/claude-245/ORDER.md`: PASS.
- `uor-harness loop pr-check --local`: verdict PASS. Order, scope, banned-presence, commit-order, report, owner-paths, rule-sources and rendered-lock all pass. There are two D76 advisories, on read-path size and on stale FILEMAP rows; neither is from this diff.
- `uor-harness loop gates --full`: 11 of 11 GREEN (claim-wording, fmt-claim-wording, fmt-harness, git-hooks, loop-drive, loop-gates-watch, loop-migrate, loop-next-owner-direction, loop-next-sources, loop-next-wait, loop-review-changes-blocks).
- No code changed, so no cargo test was run.

## Skills used

None.

## Still open

- **Next unit:** a sheet registering Q1 to Q4, the positions, the strata, the cut-window rule and Row 0 verbatim, with a fresh FIT. Then the grade on the laptop, on claude-244's and claude-243's sealed files.
- **Then Q5:** its decision card fixes the trace's rule, then a sheet with a FIT and a pre-run review, then the trace, on the GPU for the float side.
- **Carried:**
  - read F′'s top two, which is option C;
  - the exact-key split;
  - a cheaper exact nearest for speed;
  - making raw the default, or the snap faithful.

## Plain summary

The integer model and the trained model disagree on about 3% of next words on the panel of new phrasings. In 88% of those cases, the integer model was nearly torn between two words. The old pass mark, "first words match on 97% of prompts", turned out to measure how many such close calls a panel has, as much as the conversion's faithfulness.

The new pass mark asks what matters for serving. Does the integer model pick the right word as often as the trained model? And is every disagreement on a clear choice explained by rounding? Nobody has computed either answer yet, so the next step can still fail.
