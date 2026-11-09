# REPORT: one-model-end-to-end, claude-342

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). ORDER: [ORDER.md](ORDER.md). Base `67972456d`. No GPU was used and no model was trained.

**In one line:** By the rule fixed before the count, 32 rows does not reach the question in TRAIN, at 1.2% of oasst2's scored rows and 76.3% of the capped mw-balp's against a bar of 80%. But the same data puts the question's first word within 32 rows at 93.1% of mw-balp's reply first rows, so a longer carry is not the next step. The decision is a control ladder at the new trainer commit, to find what took the stopping away.

## What did not work, first

- **The count as first built measured the wrong thing on the capped roots (instrument, repaired).** It treated a whole unscored run as the question. `reply-cap` keeps a capped reply as unscored context, so on `cap1` to `cap6` one run can hold several whole turns: the first-turn run averages 84.9 rows (a one-off count over `cap1`'s tokens, not in the JSON), against a question of 11 (median). That first count read 34.3% for `cap1`.
  - **The repair:** the tool also reads the question from the tokens, from the word after the last `User:` before a reply to the word before the `\nAssistant:` header. It placed every reply on `oasst2` and `cap1` (0 not placed). The run-based numbers stay in the JSON and below, as the ORDER asked for them. The rule and its bar did not change, but I changed the instrument after seeing its first output, and the reader should weigh that.
- **By the pre-stated rule, 32 does not reach the question on either dialogue root.** At least 80% of a root's scored rows needed the question's start within 32 rows. `oasst2` has 1.2% and `cap1` has 76.3%, so both miss. The run-based count misses too (0.8% and 34.3%).
- **The rule counts every scored row, and that weakens what it can say about the carry.** `oasst2`'s replies run to hundreds of rows, and a row deep in a long reply is far from the question whether or not it needs it. The rule cannot say which rows need the question. So its "does not reach" is recorded as stated, and the decision below does not rest on it alone.
- **Windows are ignored.** The count is an upper bound on what a 192-token training window can see.
- **Most of TRAIN has no question at all.** The three wiki roots hold 24,181,749 of the mix's 33,364,756 scored rows (72.5%), with no `User:` turn before them. Only 13.7% of all scored rows (4,582,684) have a question's first word within 32 rows. A question exists only in the other 27.5%.
- **The six capped roots are one set of counts.** `cap1` to `cap6` have the same token and mask hashes, so their counts are the same by construction and the mix weighs that one data set six times.

## The count

Tool: `train-question-reach` over TRAIN d3mwcap6, whose `tokens.u16` and `response_mask.u8` match the manifest's hashes (`0273173e…`, `da506e85…`). Whole output: [train-question-reach.json](train-question-reach.json). It took about 4 s on the laptop. Percent of rows whose question's first word (read from the tokens) is within that reach:

| root | rows | 4 | 8 | 16 | **32** | 64 | 128 | median question |
|---|---|---|---|---|---|---|---|---|
| `oasst2`, all scored rows | 3,234,805 | 0 | 0 | 0 | **1.2** | 6.1 | 18.1 | 28 |
| `oasst2`, reply first rows | 7,784 | 0 | 0.7 | 10.3 | **47.7** | 79.8 | 94.7 | |
| `cap1`, all scored rows | 991,367 | 0 | 0 | 9.9 | **76.3** | 92.2 | 100.0 | 11 |
| `cap1`, reply first rows | 70,672 | 0 | 0 | 48.4 | **93.1** | 99.9 | 100.0 | |

- A row's distance counts from the question's first word, so the 6-token `\nAssistant:` header and the words before it sit between. Every reply first row reaches the question's last word within 8 rows.
- The panel's question is 6 to 13 tokens (median 10), and its first word is at most 26 rows back from the row emitting the value (claude-339's sheet). `cap1`'s median question of 11 matches that family.
- `oasst2`'s questions are longer (median 28), and they are what 32 rows can only half reach at a reply's first row.

## The decision

**Decided: the next training unit is a cause ladder at the new trainer commit. Not a longer carry, and not a question-row placement yet.** It is my own decision review (§5.1), logged as `[claude] owner log:` on #3407, and Casey can reverse it.

| option | what the evidence says | call |
|---|---|---|
| **A longer carry** | The panel's questions are in reach at 26 rows, and 93.1% of `cap1`'s reply first rows are within 32. Only `oasst2`'s longer questions argue for it. claude-335's `h = 8` also passed on the recall task. | Not next. |
| **A question-row placement on chat windows** | The most direct recall lever left, since the scored placement split on the recall task (B2-aC: seed 2 passed, seed 1 did not). It needs new trainer code: `--hamming-attract-at question` and `both` are refused on a language run. A run now would start from a recipe where nearly every reply runs to the 64-token cap. | Next after the ladder, on a recipe that stops. |
| **A cause ladder** | L-a (no carry) stops on 6 and 0 of 256, against claude-322's 109 and 86, and has first-line counts of 2 and 0 against 14 and 2. It differs in the ask heads, the pull and the trainer commit (20 commits touch the trainer crate between the pins). No run separates them. | **Next.** |

**The ladder**, on L-a's recipe, TRAIN and step count, two seeds, under its own sheet, FIT and pre-run review, at about the cost of claude-339's four pods ($5.86 for that round):

| arm | flags | what it isolates |
|---|---|---|
| **N0** | no ask heads, no pull, no carry | the trainer commit, against claude-322 |
| **N1** | ask heads, no pull, no carry | the ask heads, against N0 |
| L-a (sealed) | ask heads and pull | the pull, against N1 |

**How it will read,** using claude-339's floor of 64 EOS at both seeds:
- **N0 under the floor:** the commit lost it. Bisect the 20 commits before any ask run.
- **N0 above, N1 under:** the ask heads did.
- **N0 and N1 above:** the pull did. The placement work then starts from the ask heads and changes the pull.

## Checks run at the head

Run at the code head before this REPORT; the REPORT and the notebook row change no code.

- `cargo test --offline -p uor-r4-float-train --bin train-question-reach`: 4 passed.
- `train-question-reach` exits 0 on TRAIN. It refuses a mismatch against the manifest's scored count or a bad mask value.
- **Self-checks in the output:** 0 replies not placed on `oasst2` and `cap1`, and per-root scored counts equal to the manifest's (the tool errors otherwise). The per-root scored rows sum to the mix's 33,364,756.
- **The tool** builds no model and no tensor, and it uses no GPU.

## Skills used

- None beyond the loop itself: the ORDER, the fresh exact-head review and the decision review.

## Still open

- **The ladder** (N0 and N1, two seeds each): the next unit, under its own sheet.
- **The question-row placement:** needs new trainer code, and comes after the ladder.
- **Windows:** this count ignores them. If the placement work needs the windowed figure, the trainer's window rule can give it.
- **mw-balp's 35 paraphrase files** are not published, and **`test-watch`** still depends on signal timing.

Closes nothing: #3407 stays open.

## Plain summary

We wanted to know whether a 32-token memory is long enough to hold the question while the model writes its answer. So we counted, in the training data, how far back each question sits.

In the synthetic conversations that resemble our test, the start of the question is within 32 tokens at the beginning of 93% of replies. In real human chat it is 48%, because people ask longer questions, and 72.5% of the training text is encyclopedia prose with no question at all. By the strict rule we set beforehand, 32 tokens does not reach (76.3% against 80% on the synthetic data), but a longer memory is not what the evidence points to first.

The more urgent problem is that our last model stopped knowing when to stop talking, and we don't know which change caused that. So the next step is a small controlled experiment: take the new pieces out one at a time and see which one lost it. Nothing was trained in this step.
