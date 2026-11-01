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
  - The **null share** q is the share of the cell's positions with g ≤ G₁₀. Nearest rank makes q ≥ 0.10, and margins that tie at G₁₀ make it larger: g is an integer, and exact ties at g = 0 can occur.
  - The **lift** L = c / q, reported with no bar. It cannot exceed 1/q, so it is not what the table reads.
  - The **excess concentration** κ = (c − q) / (1 − q): the share of the largest possible excess over the null that c reaches.
    - It is 0 when disagreements fall in the low stratum at the null rate, and 1 when all of them do, at any q < 1.
    - Ties at G₁₀ therefore cannot put a row out of reach.
  - g_max is the largest g at any disagreement. It bounds 2ε from below.
  - z₀ is the share of positions with g = 0 (exact ties), and d_z the disagreements among them. Both are reported with no bar.
- **Pooled over the three seeds,** each cell against its own G₁₀:
  - r₀ = Σd₀ / Σn₀ and r₊ = Σd₊ / Σn₊;
  - c = Σ #(disagreements with g ≤ the cell's G₁₀) / Σd, where d = d₀ + d₊;
  - q = Σ #(positions with g ≤ the cell's G₁₀) / Σ positions;
  - L = c / q and κ = (c − q) / (1 − q).

**Why R's own margin can tell a residual from a fault.** Let ε be the largest difference, over the table's rows, between R's score and F′'s score for the same row at one position. That covers the query's residual, the table's rounding, and any drift built up along the prefix.
- If R picks w_R and F′ picks w_F ≠ w_R, then R's own gap from w_R to w_F is at most 2ε, because F′ prefers w_F.
- So R's top-two gap g is at most 2ε.
- A small ε can therefore flip a pick only where g is small. Under the null that disagreements do not depend on g, c ≈ q, so κ ≈ 0.
- A clear-margin reading therefore says that ε is not small at those positions. It does not by itself say that the cause is something other than numerics.

**The decision table.** The p-w cells decide, pooled over the three seeds. DEV is a reading.
- The table reads κ, with N the pooled positions and N_low the pooled positions with g ≤ their cell's G₁₀.
- At q = 0.10, its thresholds 4/9 and 1/6 are exactly c ≥ 0.50 and c ≥ 0.25.
- Each κ threshold is compared in integers, so no rounding can move a reading across it: κ ≥ 4/9 is 9 d_low N ≥ 4 d N + 5 d N_low, and κ ≥ 1/6 is 6 d_low N ≥ d N + 5 d N_low.
- The near-tie and mixed rows also need the excess to be clear of chance: c − q ≥ 3 √(q(1 − q)/d). Otherwise the reading is the clear-margin row, since the disagreements are then not shown to be concentrated at near ties. This clearance is computed in floating point: its integer form would pass 2^53, and an exact tie at it needs q(1 − q)/d to be the square of a rational.
- A reading with κ ≥ 1/6 that is not clear of chance falls to the clear-margin row and is reported as **clear-margin (inconclusive)**. Its next unit is the same trace.
- The rows are read top to bottom, and the first that holds is the kind.

| Reading (p-w, pooled) | Kind | The next unit |
|---|---|---|
| d < 150, or q = 1 | **Underpowered**: c is reported with its standard error √(q(1 − q)/d), and no kind is read. | The mixed row's trace. |
| κ ≥ 4/9, clear of chance | **Near-tie residual**: the gap is the export's numeric residual where R's readout is nearly tied. | It sizes the fix. If r₊ ≤ 3%, a decision card re-derives the first-token bar from this sheet's measured per-position residual. If r₊ > 3%, the readout's precision is raised. |
| 1/6 ≤ κ < 4/9, clear of chance | **Mixed** | It traces the clear-margin disagreements (g > G₁₀) first, layer by layer: trunk, Hamming read, memory, readout. |
| otherwise | **Clear-margin difference**: most differences are at g > G₁₀, so where they sit, ε exceeds G₁₀/2. This does not say the cause is other than numerics. | The same trace as the mixed row; the trace finds the cause. |

**The opening contrast (p-w, pooled), read beside the kind.** It is **opening-specific** if r₀ > 2 r₊ and r₀ − r₊ > 2 √(r₀(1 − r₀)/n₀ + r₊(1 − r₊)/n₊).
- The kind picks the next unit's work. The contrast picks only where trace or precision work starts; the near-tie row's decision card has no start.
- If the contrast is opening-specific, that work starts at the opening, with the memory's opening keys (`keys-*.json` distances). Otherwise the difference is per position, and it starts at offset 1.

**Also reported, with no bar:**
- r₀, r₊, c, q, L, κ, g_max, z₀ and d_z per cell;
- r₊ against claude-243's 3% bar;
- g's quartiles at disagreeing positions and at all positions;
- the excluded rows' first-token disagreements;
- the same readings on DEV.
- d₀ will not reproduce claude-243's 64, 47 and 78, by design. It leaves the cut windows out, reads F′'s teacher-forced nearest, and counts an opening where both sides pick `<|bos|>` as agreeing. I-Offset0 reproduces the old count with the old rule.

## 4. Instrument checks (any failure is row 0: nothing is counted, then repair and rerun)

1. **I-Inputs:**
   - R's three artifacts pass against `R/a7/SHA256SUMS`;
   - F′'s six rows files and six greedy files pass against `tie-parity/SHA256SUMS`;
   - R's six stored greedy replies pass against claude-243's `seal/laptop/SHA256SUMS`;
   - claude-232's six prompt files pass against its store's sums.
2. **I-NoKey:** no opening of R is served by a point key. The mode refuses an artifact with a point table, and every row of R's stored greedy replies has `keyed` null. The served pick at every offset is therefore the whole table's nearest, which the mode checks at each offset.
3. **I-Tests:** the `--forced` tests pass at the head, and both of the mode's negative gates (the feed and the runner-up) fail with their fix set aside.
4. **I-Forced-Greedy:** in every row, at every offset i inside R's stored greedy reply (claude-243's `integer-greedy-SPLIT.json`) where that reply equals the reference on offsets 0 to i − 1, R's forced pick equals R's greedy token. Every such position must agree.
5. **I-Align:** the same for F′, at every such position of a counted row: the rows' `nearest` must equal F′'s greedy token (`samples-memory.json` on DEV, `samples-p-w-greedy-memory.json` on p-w). Also, the rows' `targets` must equal the reference at every compared position.
6. **I-Offset0:** over all rows, R's forced pick at offset 0 against F′'s greedy first token reproduces claude-243's first-token disagreement in every cell: 4, 4, 5 and 64, 47, 78. It uses claude-243's rule: an opening agrees only when R's pick is not `<|bos|>` (id 0) and equals F′'s greedy first token.
7. **I-Coverage:**
   - every compared position is in F′'s rows;
   - each counted row's recorded offsets number `len(reference)`;
   - n₀ is the count of counted rows;
   - each cell's positions sum to the mode's printed `positions`.
8. **I-RunnerUp:** at every offset the runner-up differs from the pick, and where g = 0 the runner-up's id is larger than the pick's (ties go to the smaller id).
9. **I-Exit:** every forced decode exits 0, and every margin is g ≥ 0.

## 5. Base artifact and data split

- **R:** `a7r-s1/2/3.uorlut` (`f86a7856…`, `0a69a009…`, `66ae49de…`; 51,310,784 bytes each). It is claude-243's raw export of (r-drop) step 16,384, sealed at `results/claude/integer-nearest-readout-tie-parity-2026-11-01/laptop/`.
- **F′:** `results/claude/integer-nearest-readout-tie-parity-2026-11-01/tie-parity/out/r-drop-sN/`, the `rows-dev-memory.json`, `rows-p-w-memory.json` and greedy files.
- **Data split:** DEV and the held-out-phrasing panel p-w. Both were already opened for parity by claude-232 and claude-243, through claude-232's prompt files (`results/claude/integer-nearest-readout-parity-2026-10-31/prompts/`). No TRAIN data and no other panel is read.

## 6. Steps

1. Build `fbs-toks` at the head (`-p uor-r4-integer --release`), then run I-Tests.
2. Run I-Inputs.
3. Run the six forced decodes (I-Exit).
4. Grade with a `jq` script kept beside the outputs, its sha256 recorded in the REPORT. The checks come first (I-NoKey, I-Forced-Greedy, I-Align, I-Offset0, I-Coverage, I-RunnerUp) in all six cells. The readings are shown or written only after every check holds (the checks pass computes them in memory and drops them before any output). The grade refuses to start unless the worktree's crates match the head, and `fbs-toks` is rebuilt at that head right before the grade.
5. Seal the outputs with a `SHA256SUMS` under `results/claude/integer-nearest-readout-margin-2026-11-01/`, and read them back.

## 7. Minimum test power

- **The counted p-w positions per seed:** about 1,067 openings and about 13,000 later offsets. Pooled, that is about 3,200 and 39,000.
- **c:** with at least 150 pooled disagreements, c's standard error under the null is about √(q(1 − q)/150), or 0.025 at q = 0.10. At that q, the two thresholds (c = 0.25 and c = 0.50) sit at least 6 standard errors above chance.
  - At a larger q the standard error grows, to 0.041 at q = 0.5. The 3-standard-error clearance in the table keeps a chance excess from reading as near-tie or mixed.
  - Below 150, the table's underpowered row applies.
- **These standard errors are optimistic.** Pooling counts the same openings once per seed, and positions inside one row are correlated. The 2× ratio in the opening contrast and the 6-standard-error slack on c are there to absorb that.
- **The opening contrast:** at r₀ ≈ 5% on about 3,200 openings, r₀'s standard error is about 0.4%, so a 2× contrast against r₊ is detectable.

## 8. Cost and stop rule

- **Cost:** $0. The laptop runs about 64,000 steps per p-w cell at about 1,200 tok/s, so about a minute each.
- **Stop rule:**
  - Any instrument check that fails is row 0. Nothing is counted, the fault is repaired, and the whole grade is rerun; a partial count is never read.
  - A bar change is only ever a decision card, with my own decision review, logged on #3462.

## 9. Non-author TEST FITNESS review

**Round 1 (fresh reviewer, 1 Nov 2026), before any forced decode of R existed.**

TEST FITNESS: NOT_FIT

- **What it confirmed on the sealed float files:**
  - the cut windows (26 DEV and 135 p-w rows at every seed);
  - the counted rows fit the context (at most 136 and 140 tokens);
  - 1,067 openings and 12,864 later offsets per p-w seed;
  - the `position_k + i` rule (the rows' `targets` equal the reference at every counted position);
  - F′'s half of I-Align and I-Coverage, with 0 failures in all six cells;
  - the 2ε bound.
- **Why NOT_FIT:**
  - With ties at G₁₀, the null for c is q, not 0.10, so the old thresholds could change the reported kind.
  - The near-tie row left its next unit to the author.
- **The fixes, all made above:**
  - c is read as the lift L = c/q, with q and g_max reported;
  - the near-tie row names its next unit by r₊ against 3%;
  - "pooled" is defined;
  - the kind takes precedence over the opening contrast;
  - an underpowered row covers fewer than 150 disagreements, with a note that the standard errors are optimistic;
  - the d₀ line explains why it differs from claude-243's count;
  - I-Inputs now covers R's greedy replies;
  - I-NoKey, the recorded-offsets check and I-RunnerUp are new.

**Round 2 (a second fresh reviewer, 1 Nov 2026), before any forced decode of R existed.**

TEST FITNESS: NOT_FIT

- **What it confirmed:**
  - round 1's eight findings are fixed in the text;
  - the grader matches the sheet's definitions, checks and order;
  - I-NoKey can fail, since every row of R's greedy files carries `keyed`.
- **Why NOT_FIT:** the lift L = c/q cannot exceed 1/q. So with q > 0.2 from ties at G₁₀, a pure near-tie residual would be read as mixed. That is round 1's defect pointing the other way.
- **The fixes, all made above:**
  - the table reads the excess concentration κ = (c − q)/(1 − q), which can reach every row at any q < 1;
  - its thresholds are the original ones at q = 0.10, compared in integers;
  - the near-tie and mixed rows need a 3-standard-error clearance;
  - q = 1 is underpowered, and the rows are read top to bottom;
  - z₀ and d_z report the exact-tie stratum;
  - the contrast's start applies only to trace or precision work.
- **The grader's fixes:**
  - a clean-tree check;
  - an n₀ check that can fail;
  - a count check on R's greedy sums;
  - readings written only after every check holds;
  - known-answer probes for the pooled decision as well.

**Round 3** (a fresh `delegate:reviewer`, 1 Nov 2026, on the sheet at `572666737` and the grader; it read no forced decode of R and ran neither `grade.sh` nor `fbs-toks`):
- **What it ran:** `bash -n` on `grade.sh`, and the grader's probe block, which passes: the good cell reads as hand-counted, the checks pass holds no readings, the bad cell fires I-Forced-Greedy and I-RunnerUp once each, and 7 of 7 pooled cases decide as worked out.
- **What it confirmed:**
  - every round-1 and round-2 finding is fixed in the sheet and the grader;
  - the κ algebra and its integer thresholds are exact (the products stay near 1.6e10, far under 2^53);
  - q = 1 is caught before κ is used, every row is reachable, and each has a fixed next unit and start;
  - the mode checks at every offset that the step's token is the whole table's nearest, and prints no reading.
- **Its findings, none of which changes a reading or a next unit, and what was done:**
  1. Low: the clear-margin row ruled a residual out, against §3's own caveat. Reworded above.
  2. Low: a κ ≥ 1/6 reading not clear of chance carried the plain clear-margin label. It is now reported as clear-margin (inconclusive); the grader's label is unchanged, and the REPORT adds the word from the pooled reading's `kappa_ge_1_6` and `clear_of_chance`.
  3. Info: the chance clearance is floating point, not integer. Said above.
  4. Info: the checks pass computes readings in memory and drops them before output. §6 step 4 now says "shown or written".
  5. Hardening, not a condition: the binary's head check has gaps (untracked files, two timestamps). `fbs-toks` is rebuilt at the head right before the grade; I-Forced-Greedy and I-Offset0 already tie its picks to the sealed greedy replies.

TEST FITNESS: FIT
