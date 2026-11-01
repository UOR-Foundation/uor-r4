# REPORT: integer-nearest-readout, claude-247 (Q5's rule, the exact trace of every clear-margin disagreement): decision card approved

Lab: claude. Item: [integer-nearest-readout](../../../../roadmap/items/integer-nearest-readout.md) (#3462). Order: [ORDER.md](ORDER.md). Decision card: [DECISION.md](DECISION.md), approved after two fresh decision-review rounds (REVISE, then APPROVE), and logged on #3462 for Casey, who can reverse it. References #3462.

## Tokens per second

No new speed is claimed: this unit ran nothing. R's decode speed is still claude-243's: 1,204 tok/s median decode on R s1, on this M1.

## What did not work, first

- **No measurement moved, and Q5 is still unread.** This unit read no trace value. Done-when 3 still waits on Q5, and Q5 can fail.
- **The trace this rule needs does not exist yet.** The instruments at the base fall short:
  - they record codes and kept sets, but no coordinates, rate scores or float scores;
  - none of them can force a decision;
  - `fbs-snapshot` has no turn memory or nearest readout, and does not run on the panel's rows.

  The next unit builds the instrument, and it needs five files outside the item's scope (the card's Constraints).
- **Round 1 was REVISE, with a blocker.** The first draft listed four kinds of discrete decision and missed a fifth: the recurrence's rate bucket, where a clamped score chooses one of 16 entries. That is 384 decisions per position, 6 layers × 64 lanes. A replay that never forced them could have called an honest rounding flip a fault, or missed a real one. The card now lists five kinds, and a kind I-Complete finds that the card does not list voids the card.
- **The float rate score has three forms in the code** (clamp, fold and reflecting floor, `geometric_stack_native.rs:39–69`), and R only clamps. The card makes I-Complete check that F's run clamps. If it does not, Row 0 fails, and nothing is read.
- **Round 1 also found two majors and five minors:**
  - the float scores' conversion was left to the sheet;
  - the prefix's exclusion of the readout read's earlier choices was not cited;
  - the forcing claim was too strong;
  - G₁₀ had no exact rank;
  - the replay's cost was unbounded;
  - two smaller gaps.

  All are fixed in the card.
- **Round 2 approved, with three minors left to the sheet:**
  - under a 16-bit GPU build, values must be judged as the decision reads them;
  - a rate family's decile can be loose when its scores crowd at 0 or 1, so each family's G₁₀ is reported against 1/32;
  - where each stream read runs, including at depth 0, is cited only in the sheet.
- **The rule has named blind spots** (the card's residual risks):
  - a fault confined to near ties passes;
  - forced replay shows that the forced set suffices, not that each member matters;
  - T's f32 arithmetic is the arithmetic step's reference by convention;
  - teacher forcing misses faults in the serving loop after offset 0.
- **A disclosure:** while listing the set, the author saw which side names the reference at each position: R at 49, F′ at 48 and neither at 87. This only splits claude-246's a and b. The rule never compares a pick with the reference.

## What worked

- **The cancel probe passed, as an instrument call** (ORDER). Q5 had no rule. Q1 to Q3 had HELD. The set lists from sealed files, and no other order holds the item.
- **The set is named exactly, from sealed files only.** It is 184 positions in 169 windows (cell × row): 162 counted (11 on DEV, 151 on p-w) and 22 on p-w's cut windows. The listing reproduces claude-244's (d, d_low) in all six cells, and R's gaps run from 7,779 to 125,797.
- **The card meets claude-245's four requirements:**
  1. **Fixed before it is read.** Every bound is a rule: the cell's sealed G₁₀ for the pick, and a rank-defined G₁₀(φ, P) for each code and rate-bucket family, computed from the reference side's unforced trace. The float scores' conversion is fixed in the card.
  2. **The whole causal prefix.** At every position up to p: every stream read's codes and kept sets, every layer's rate buckets and the turn memory. At p: the readout read and the nearest. The exclusion of the readout read's earlier choices is cited from the code.
  3. **Its own Row 0:** I-F, I-R, I-Twin, I-Rerun, I-Force, I-Align, I-Complete, I-Set and I-Inputs. A position that cannot be traced is an instrument repair.
  4. **Writer, scope change and cost:** the claude lab, in the next unit, under its own sheet. The scope change is five files. The GPU run takes under an hour, about $1 to $2; R runs on the laptop.
- **The rule separates structure from arithmetic without a rounding bound.** F against T differs only in the export's rounding, and T against R only in arithmetic. Each step whose picks differ is replayed with its decisions forced in the forward's order, so a fault is placed by step, and its remedy with it.
- **Round 2 confirmed the card against the code at `230cef2cb`:**
  - the rate bucket's boundaries agree exactly (k/16 in f32, k · 4096 at Q16);
  - on both sides, the score only chooses the entry;
  - the conversion's factor, 2^20 from the mean form;
  - the readout payload's exclusion;
  - the replay order in `stack_step_fbs`.

## What changed

- **`DECISION.md` (new):** Q5's rule, approved after two decision-review rounds. It covers:
  - the set;
  - the five decision kinds;
  - the three sides and two steps;
  - near ties and forced replay;
  - Row 0 and the outcomes;
  - the writer, the scope change and the cost;
  - both review rounds.
- **`ORDER.md` (new),** with the cancel probe's result.
- **The item** ([integer-nearest-readout](../../../../roadmap/items/integer-nearest-readout.md)):
  - done-when 3's Q5 cites this card;
  - a history bullet records the rule;
  - "What is missing" names the next unit: the trace instrument with its scope change, then the sheet that reads Q5;
  - the status stays `ready`.
  - `cargo xtask roadmap render` leaves the rendered roadmap unchanged.
- **The decisions this unit made:**
  - **Option C, three-way with forced replay,** over A (R against F′ alone), B (attribution only), D (a per-operation ε) and E (revising claude-245's bar). The card's options section gives each reason.
  - **Strict by design.** A clear flip anywhere in the prefix fails the position, even if forcing it was not needed. A clear pick difference left after every forcing fails it too, even when a clamp or the export's rounding caused it. A FAIL from drift alone is a real FAIL, and its remedy is a finer format or a fix.
  - **R's cell G₁₀ applies to the float sides' pick margins,** by convention, as the card states.
- **No notebook row.** The unit has no result, as the ORDER says.

## Measurements

None in the result sense: no trace value was read, and nothing ran on a GPU. The set's listing is a count over sealed files, made with `local/claude/claude-247/list-set.jq` and kept out of the repo:

| Cell | (d, d_low), as claude-244 | Set | Counted | Cut windows | Windows |
|---|---|---|---|---|---|
| DEV s1 | (80, 78) | 2 | 2 | 0 | 2 |
| DEV s2 | (46, 42) | 4 | 4 | 0 | 3 |
| DEV s3 | (69, 64) | 5 | 5 | 0 | 5 |
| p-w s1 | (409, 356) | 60 | 53 | 7 | 54 |
| p-w s2 | (383, 351) | 34 | 32 | 2 | 34 |
| p-w s3 | (454, 388) | 79 | 66 | 13 | 71 |
| **All** | | **184** | **162** | **22** | **169** |

## Checks run at the head

- `loop report --check`, `loop pr-check --local` and `loop gates --full`: recorded in the review record at the exact head.
- The set's listing reproduced claude-244's (d, d_low) in all six cells (the cancel probe, check c).

## Skills used

None.

## Still open

- **The next unit: the trace instrument.** It records and forces the five kinds on F, T and R, writes F's top two with their scores, and builds T's rounded copy. Its ORDER carries the card's scope change, five files.
- **Then the sheet that reads Q5.** It registers the card's rule verbatim, with a fresh FIT and a pre-run review, and round 2's three minors: the `act_t` build, G₁₀ against 1/32, and the depth-0 citation.
- **Carried:**
  - **Speed:** the nearest scan is 65% of a decode step (claude-243), so a cheaper exact nearest is the speed lead.
  - making raw the default, or the snap faithful;
  - the leads in the item's "What is missing".

## Plain summary

The served integer model and the trained model sometimes choose different words at places where the integer model was not close to undecided. There are 184 such places across the two test panels and three seeds. This unit wrote down, before anyone looks, how each of them will be explained.

Three versions of the model are run side by side:
- the trained model;
- the trained model with its numbers rounded exactly as the integer file stores them;
- the integer model itself.

Every yes-or-no choice the models make on the way to a word is recorded, in the order they make it. Where two versions first part ways, the later one is made to follow the earlier one's choice, and the run goes on. A difference counts as explained only if every choice that had to be overridden was nearly a coin flip, and the final word, if still different, was a near tie too. Any clear divergence counts against the integer model.

Nothing was measured. The tool that records and overrides those choices is the next piece of work.
