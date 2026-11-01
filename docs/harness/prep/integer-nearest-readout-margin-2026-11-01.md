# Prep sheet: the raw export's per-position margin against the float reference, under teacher forcing (integer-nearest-readout, claude-244), 2026-11-01

- **Item:** [integer-nearest-readout](../../roadmap/items/integer-nearest-readout.md) (#3462). **ORDER:** [claude-244](../../loop/tasks/integer-nearest-readout/claude-244/ORDER.md).
- **What runs:** the laptop only, with no pod. R's integer decodes are product serving (CHARTER §3.6), under the owner's 31 Oct allowance for this chain. The float side is already sealed: F′'s teacher-forced rows from claude-243's GPU run.
- **What it is for:** it is a reading for the next decision, and it decides no bar. claude-243's row stands as a MISS, and nothing here re-grades it.

## 1. Question

claude-243 read R against F′ on first tokens only: 64, 47 and 78 of 1,202 p-w openings disagree, and 4, 4 and 5 of 310 on DEV. This sheet asks three questions at every reply position, with the reference fed in on both sides:

1. How often does R's served pick differ from F′'s?
2. Do the differences sit where R's own readout is nearly tied, the mark of a small numeric residual? Or where R picks by a clear margin, the mark of a systematic difference?
3. Are they special to the reply's opening?

## 2. Arms

- **R_forced:** `fbs-toks a7r-sN.uorlut --forced prompts-SPLIT-sN.json --out forced-SPLIT-sN.json`, from claude-244's new mode.
  - Each prompt is stepped as `--replies` steps it (the reply start marked at its last token). Then the reference is fed in token by token.
  - At offset i it records the served pick, the runner-up, and both `−L1` scores at Q12. The served pick is the whole table's nearest, ties to the smaller id; the mode checks this at every offset.
- **F′_rows:** the field `nearest` of F′'s `rows-SPLIT-memory.json` (claude-243's `tie-parity/out/r-drop-sN/`). It is F′'s teacher-forced nearest over the panel's reference stream, with the memory on and the read's tie rule on. Prompt k's offset i is read at position `position_k + i`.
- **The counted rows:** the prompts that start at the document's `<|bos|>` (`prompt[0] == 0`).
  - The others are the windows the 192-token context cuts: 26 of DEV's 310 and 135 of p-w's 1,202, at each seed. F′'s rows may read those positions from a different window, so they are reported and not counted.
  - In a counted row, prompt and reference together fit the context (at most 193 tokens), so R covers the whole reference.

## 3. Bar and decision table

**Bar:** none. This reading passes or fails nothing; the decision table below picks the next unit. claude-243's 3% first-token bar and its MISS stand unchanged.

**Per cell** (split × seed), over the counted rows:
- **The openings:** n₀ openings, d₀ disagreements, rate r₀ = d₀/n₀.
- **The later offsets (i ≥ 1):** n₊ positions, d₊ disagreements, rate r₊ = d₊/n₊.
- **R's own margin** at every position: g = score − runner-up score ≥ 0, in Q12 `L1` units.
  - G₁₀ is the 10th percentile of g over all of the cell's positions, by nearest rank.
  - The **concentration** c is the share of the cell's disagreements, at any offset, with g ≤ G₁₀.

**Why R's own margin can tell a residual from a fault.** Let ε be the largest difference, over the table's rows, between R's score and F′'s score for the same row at one position. That covers the query's residual and the table's rounding together.
- If R picks w_R and F′ picks w_F ≠ w_R, then R's own gap from w_R to w_F is at most 2ε, because F′ prefers w_F.
- So R's top-two gap g is at most 2ε.
- A small residual can therefore flip a pick only where g is small. Under the null that disagreements do not depend on g, c ≈ 0.10.

**The decision table.** The p-w cells decide, pooled over the three seeds. DEV is a reading.

| Reading (p-w, pooled) | Kind | The next unit |
|---|---|---|
| c ≥ 0.50 | **Near-tie residual**: the gap is the export's numeric residual where R's readout is nearly tied. | It sizes the fix. Either a decision card re-derives the first-token bar from this sheet's measured per-position residual, or the readout's precision is raised. It chooses by r₊ against the bar. |
| 0.25 ≤ c < 0.50 | **Mixed** | It traces the clear-margin disagreements (g > G₁₀) first, layer by layer: trunk, Hamming read, memory, readout. |
| c < 0.25 | **Clear-margin difference**: most differences are where R is not near a tie, which is a systematic difference. | The same trace as the mixed row. A residual explanation is set aside. |

**The opening contrast (p-w, pooled), read beside the kind.** It is **opening-specific** if r₀ > 2 r₊ and r₀ − r₊ > 2 √(r₀(1 − r₀)/n₀ + r₊(1 − r₊)/n₊). Then the next unit starts at the opening, with the memory's opening keys (`keys-*.json` distances). Otherwise the difference is per position.

**Also reported, with no bar:**
- r₀ and r₊ per cell;
- r₊ against claude-243's 3% bar;
- g's quartiles at disagreeing positions and at all positions;
- the excluded rows' first-token disagreements;
- the same readings on DEV.

## 4. Instrument checks (any failure is row 0: nothing is counted, then repair and rerun)

1. **I-Inputs:**
   - R's three artifacts pass against `R/a7/SHA256SUMS`;
   - F′'s six rows files and six greedy files pass against `tie-parity/SHA256SUMS`;
   - claude-232's six prompt files pass against its store's sums.
2. **I-Tests:** the `--forced` tests pass at the head, and both of the mode's negative gates (the feed and the runner-up) fail with their fix set aside.
3. **I-Forced-Greedy:** in every row, at every offset i inside R's stored greedy reply (claude-243's `integer-greedy-SPLIT.json`) where that reply equals the reference on offsets 0 to i − 1, R's forced pick equals R's greedy token. Every such position must agree.
4. **I-Align:** the same for F′, at every such position of a counted row: the rows' `nearest` must equal F′'s greedy token (`samples-memory.json` on DEV, `samples-p-w-greedy-memory.json` on p-w). Also, the rows' `targets` must equal the reference at every compared position.
5. **I-Offset0:** over all rows, R's forced pick at offset 0 against F′'s greedy first token reproduces claude-243's first-token disagreement in every cell: 4, 4, 5 and 64, 47, 78.
6. **I-Coverage:** every compared position is in F′'s rows. n₀ is the count of counted rows, and each cell's positions sum to the mode's printed `positions` over those rows.
7. **I-Exit:** every forced decode exits 0, and every margin is g ≥ 0.

## 5. Base artifact and data split

- **R:** `a7r-s1/2/3.uorlut` (`f86a7856…`, `0a69a009…`, `66ae49de…`; 51,310,784 bytes each). It is claude-243's raw export of (r-drop) step 16,384, sealed at `results/claude/integer-nearest-readout-tie-parity-2026-11-01/laptop/`.
- **F′:** `results/claude/integer-nearest-readout-tie-parity-2026-11-01/tie-parity/out/r-drop-sN/`, the `rows-dev-memory.json`, `rows-p-w-memory.json` and greedy files.
- **Data split:** DEV and the held-out-phrasing panel p-w. Both were already opened for parity by claude-232 and claude-243, through claude-232's prompt files (`results/claude/integer-nearest-readout-parity-2026-10-31/prompts/`). No TRAIN data and no other panel is read.

## 6. Steps

1. Build `fbs-toks` at the head (`-p uor-r4-integer --release`), then run I-Tests.
2. Run I-Inputs.
3. Run the six forced decodes (I-Exit).
4. Grade with a `jq` script kept beside the outputs, its sha256 recorded in the REPORT. The checks come first (I-Forced-Greedy, I-Align, I-Offset0, I-Coverage), then the readings.
5. Seal the outputs with a `SHA256SUMS` under `results/claude/integer-nearest-readout-margin-2026-11-01/`, and read them back.

## 7. Minimum test power

- **The counted p-w positions per seed:** about 1,067 openings and about 13,000 later offsets. Pooled, that is about 3,200 and 39,000.
- **c:** with at least 150 pooled disagreements, c's standard error under the null is about √(0.1 × 0.9 / 150) ≈ 0.025. Both thresholds, 0.25 and 0.50, sit at least 6 standard errors above chance.
- **The opening contrast:** at r₀ ≈ 5% on about 3,200 openings, r₀'s standard error is about 0.4%, so a 2× contrast against r₊ is detectable.

## 8. Cost and stop rule

- **Cost:** $0. The laptop runs about 64,000 steps per p-w cell at about 1,200 tok/s, so about a minute each.
- **Stop rule:**
  - Any instrument check that fails is row 0. Nothing is counted, the fault is repaired, and the whole grade is rerun; a partial count is never read.
  - A bar change is only ever a decision card, with my own decision review, logged on #3462.

## 9. Non-author TEST FITNESS review

TEST FITNESS: PENDING

To fill in by a fresh reviewer before any forced decode of R is read.
