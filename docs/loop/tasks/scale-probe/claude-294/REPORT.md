# REPORT: scale-probe, claude-294 (the prep sheet)

Lab: claude. Order: [ORDER.md](ORDER.md). Item: [scale-probe](../../../../roadmap/items/scale-probe.md) (#3472), done-when 1. Base: `9577aebc9`. PR #3744.

**Result: the scale probe's prep sheet is written, with its instrument, its mixes and its overlap census.**
- **The sheet** [`scale-probe-2026-11-05.md`](../../../../harness/prep/scale-probe-2026-11-05.md) registers 18 grid runs: widths 256, 512 and 1,024 × D1, D2 and D3 (a quarter, a half and all of claude-293's 27,968,113 human-written TRAIN tokens) × 2 seeds. Every run takes the same 25,165,824 positions. Two templated contrast runs are reported only. On three 1 × RTX 5090 pods, the cost is about 17–20 GPU-hours, $16–19.
- **The bars are set in advance:** R1 (width), R2 (data) and R3 (against its own TRAIN's T2 and unigram), each read on all rows and on the rows whose context D3 does not hold. An ordered six-row table says what each outcome unlocks.
- **The instrument** `docs/labs/scale-probe/instrument` (bin `scale-probe`, laptop counting only) has four commands, `mix`, `seen`, `grade` and `pair`, and 54 tests.
- **The mixes and the census** are stored under `results/claude/scale-probe-2026-11-05/` (`inputs/`, `census/`), each with a `SHA256SUMS` read back.
- **The decision card** moves the item's Data and panel lines to real text (the ORDER's card, and the item file).

**Claim check.** Casey's plan P5, "The bigger train = `scale-probe`. [sheet 1 unit; run 1–2 units]", step 6 of the order decided on #2028 (6634218223). This is the sheet unit. No code that trains or serves changes. The sheet fixes the run that says whether the one model gets better with width and with data, which decides whether the trunk is scaled next.

## What did not work, first

- **Three FIT rounds, not one.** Each round found items in the sheet or the instrument; each is fixed at the head:
  - **Round 1** (FIT WITH CHANGES):
    - the outcome table was not ordered or exhaustive;
    - no band was defined for a third seed, and `pair` could not read one;
    - the width limits were not cited, and no fallback was registered for a refused 1,024;
    - I-2's cut did not end every run on an evaluation;
    - R3 had no mixed reading and an unscoped claim;
    - R8 did not name its DEV source;
    - the power section stated an expectation as if measured.
  - **Round 2,** in four narrow reviews:
    - **The instrument:** FIT WITH CHANGES.
      - `pair clean=` did not check that the seen set was built on a1's TRAIN.
      - Three refusals had no test: a wrong TRAIN SHA-256, a wrong panel mask SHA-256, and a cell whose seeds train on different TRAIN.
    - **The widths:** sound, with wording fixes. Every limit admits 1,024, inclusively. `--hamming-read` is the width over 64. The stack heads' width and the writer's blocks grow with the width.
    - **The scored rows:** FIT after two fixes.
      - The sheet said `--score-rows` drops a window with fewer than four scored offsets. It drops only a window with none, and a document of one token or none (`hamilton_sheet.rs:1880-1920`), so the rows should be every scored position.
      - I-5 did not say that no decision-trace option is passed. The trace is taken in the memory pass only, so it would make the two passes differ.
    - **The training data and the DEV leak:**
      - Sound: `--train-split` replaces TRAIN, and the in-run DEV evaluation leaves no state that training reads.
      - One MAJOR: the sheet called R8's `T` the nearest top-1. It is the writer's top-1 (`dev_top1_hits`, `in_space_long.rs:11586`), picked from the writer's admitted cells, not the whole table.
  - **L-007 wording.** Row 6 read "Flat at these sizes" and closed the configuration. R3 read inside and behind alike as "does not beat T2". Both are now "not distinguishable at this power", and only a mechanism classification closes anything.
- **The new `clean=` refusal changed the readings' row sets.** A pair at D1 or D2 can no longer read D3's seen set. Every pair now reads the seen set of its cell X's TRAIN, X is always the larger mix, and step 8 runs `seen` against D1, D2 and D3. R1 to R3 still read D3's set. R6's adjacent data pairs are written larger-first.
- **The turn memory is out of the recipe.** Recipe (b) is run without it: the probe asks how the trunk scales, Wikipedia has no prompts to key on, and its host cost on 28M real tokens is unmeasured (claude-293: 0.67–0.75 GB per million real TRAIN tokens for the census's reply rows). A recipe `one-model-end-to-end` picks later is a new configuration.
- **The contrast cell's panel changed before the card was logged.** The first draft drew a new templated panel for it. It is now scored on DEV and on the same four real-text panels.
- **The seen share grows with the mix:** 8.8%, 12.1% and 15.6% of the panel's 1,460,697 scored positions (D1, D2, D3). So part of a gain R2 reads on all rows could be recall of contexts D3 holds and D1 does not. That is why R1 to R3 are also read on the unseen rows, and why a disagreement between the row sets is its own outcome.
- **Process slips:**
  - A builder stopped at its context limit with uncompiled code. I compiled it, and it was sound; a fresh builder finished the work.
  - `grade` first read `rows=NAME:all` as a file name. It now refuses it by name.
  - `uor-store put` printed "0/0 uploaded" and then "3/3 uploaded" for seven files. `uor-store ls` and the read-back show all seven stored.
  - The first Stop-rule fallback said "the widest width below 1,024 in steps of 64 (768)"; that width is 960, which needs 15 heads, an odd split. It now names 768 and says why.
  - The loop guard refused a patch file in the session scratchpad. The negative gates ran from the unit's `local/claude/claude-294/` instead.

## Decision: the Data and panel lines (the ORDER's card)

Logged on #3472 as `[claude] owner log:`, on its own fresh decision review; Casey can reverse it.
- **Data:** claude-293's real-text corpus at fractions, human-written text only under the decided order's interim §3.10 reading. OASST2 is English threads only (`real_text.rs:574`), and Wikipedia is the English edition. The templated corpus stays as one contrast cell.
- **Panel:** the corpus's own held-out splits, cut by source id, with the 8-token overlap census against each mix. P-dev stays reserved.
- **Scores:** the decided order says "integer top-1 against unigram and T2 count tables". The card departs from those words: the deciding count is the float model's whole-table nearest top-1, not §4.8's integer top-1 through the D11 export. The D11 export's panel top-1 is owed before any width is served or any claim is made.
- **Reversal:** restore `docs/roadmap/items/scale-probe.md` from this unit's merge-base; "hold" stops any launch. Sealed mixes, census and results stay in the store, and a run already made stays as its own configuration.
- **The decision review** (fresh, non-author): STANDS WITH CHANGES. D1 (Data) and D2 (Panel) stand as written. D3 needed the Scores wording above and the reversal's restore source, and no part is owner-only. Its optional notes are not taken: the item's panel line could carry the row-set rule, and the card could cite claude-293's frozen-panel exclusion screen.

## Measurements

- **The mixes** (`scale-probe mix`, document prefixes root by root, so D1 ⊂ D2 ⊂ D3):

  | Mix | Tokens | Scored | Documents | Passes in 25.2M positions |
  |---|---|---|---|---|
  | D1 | 7,004,928 | 6,865,470 | 4,653 | 3.67 |
  | D2 | 13,989,169 | 13,708,035 | 9,053 | 1.84 |
  | D3 | 27,968,113 | 27,416,554 | 18,016 | 0.92 |

  Store `inputs/`, `SHA256SUMS` `56a7db05a8f35b01e7231e90f95c2e7a095415263e2c16adcd93a20968c60b9c`, fetched back and checked byte for byte.
- **The census** (`scale-probe seen`, every scored panel position against each mix):

  | Against | Seen | Short | Unseen |
  |---|---|---|---|
  | D1 | 129,173 | 4,935 | 1,326,589 (90.8%) |
  | D2 | 176,210 | 4,935 | 1,279,552 (87.6%) |
  | D3 | 227,312 | 4,935 | 1,228,450 (84.1%) |

  Store `census/`, `SHA256SUMS` `88929e87931bef0fd91213ffbeb4701038354aa6cceca8be3c1621f478399f67`, read back. "Seen" is a screen, not a finding of copying: a row whose 8-token context occurs anywhere in TRAIN.
- **The instrument:** 54 tests pass at the head (`cargo test --offline` in its own workspace), and clippy with `-D warnings` is clean. Negative gates run in this unit, each failing with its fix set aside and passing once it is restored:
  - **The three-seed verdict and the three-seed spread:** set seed 3 aside.
  - **`clean=`'s TRAIN check:** set the check aside, and `pair_clean_refuses_other_train` fails.
  - **Seed 3 sharing its cell's TRAIN:** narrow `same_train` to two seeds, and `pair_refuses_cells_with_mixed_train` fails.

## Reviews

- **FIT:** `TEST FITNESS: FIT` at `2c3a96147c98791dfd01d3304c6b0554ae77a12e`, posted on #3472 (comment `6666302038`) and recorded in the sheet. There were three rounds, all fresh and non-author; round 3 returned one must-fix, applied verbatim. The reviewers ran no builds; the tests and gates are the author's, listed above.
- **Decision review:** STANDS WITH CHANGES (above). The card is logged on #3472 as `[claude] owner log:`.
- **The in-unit exact-head review:** owed below.

## Next

The run unit (done-when 2 and 3): the D67 pre-run review right before launch, then the 18 grid runs and 2 contrast runs, sealed and read back, graded on the laptop.
