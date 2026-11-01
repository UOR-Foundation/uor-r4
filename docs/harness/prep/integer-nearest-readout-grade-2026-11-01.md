# Prep sheet: the raw export graded against the trained model, Q1 to Q4 of claude-245's bar (integer-nearest-readout, claude-246), 2026-11-01

- **Item:** [integer-nearest-readout](../../roadmap/items/integer-nearest-readout.md) (#3462). **ORDER:** [claude-246](../../loop/tasks/integer-nearest-readout/claude-246/ORDER.md). **The bar:** claude-245's [decision card](../../loop/tasks/integer-nearest-readout/claude-245/DECISION.md), approved by the claude lab's top session and logged on #3462.
- **What runs:** a count on sealed files, on the laptop at $0. Nothing is decoded, trained or exported: every input is a file already sealed in the store (CHARTER §3.5, grading stored outputs).
- **What it is for:** it reads Q1 to Q3 of the bar and carries Q4. Q5 is not read here, since its rule needs its own card, so done-when 3 stays unmet whatever this reads.

## 1. Question

Does R, the raw integer export, pick the reference token as often as F′, the trained model's own decode, does?
- under teacher forcing, at every position of claude-244's counted rows (Q1 and Q2);
- at the reply's opening, on free decode (Q3).

Q4, that R's differences sit at its own near ties, is already met in claude-244's sealed grade and is carried. Nobody has counted R's or F′'s picks against the reference on these rows, so the bar can fail.

## 2. Arms

- **R_forced:** claude-244's six sealed forced decodes, `R/forced/forced-SPLIT-sN.json`. At each offset i of a row, `picks[i]` is R's served pick with the reference fed in, from `fbs-toks a7r-sN.uorlut --forced`.
- **R_greedy:** claude-243's six sealed greedy replies of R, `R/replies/r-drop-sN/integer-greedy-SPLIT.json`, for the opening on free decode.
- **F′_rows:** the field `nearest` of F′'s `rows-SPLIT-memory.json` (claude-243's GPU run, `tie-parity/out/r-drop-sN/`): F′'s teacher-forced nearest over the panel's reference stream, with the memory on. A row's offset i is read at `position + i`. The field `targets` there is the reference token.
- **F′_greedy:** F′'s greedy replies with the memory on, `samples-memory.json` on DEV and `samples-p-w-greedy-memory.json` on p-w, from the same run.
- **The prompts and references:** claude-232's `prompts/prompts-SPLIT-sN.json`, the files R decoded from.

## 3. The bar, verbatim, and the outcomes

### 3.1 The bar

The bar is claude-245's card, lines 56 to 104, quoted whole. The line references inside it (`grade-cell.jq:20–22`, `:36–42`) are to claude-244's grader, sealed in its store folder. This sheet changes no part of it; a defect in it goes to a new decision card.

> **The positions.** These are claude-244's counted rows, those whose prompt starts at `<|bos|>` (`prompt[0] == 0`), each at offsets i = 0 to n − 1. Here n = min(|reference|, 192 − |prompt| + 1), which in a counted row is the whole reference. F′'s nearest and target are read at `position + i` (`grade-cell.jq:36–42`). The target t equals the reference token there (`targets_wrong` is 0 in every cell).
>
> **The counts.** In each cell (split × seed), R is read against F′:
> - a = positions where R's pick equals t and F′'s nearest does not;
> - b = positions where F′'s nearest equals t and R's pick does not.
> - For a row k, a_k and b_k count that row's positions. V = Σ_k (b_k − a_k)², summed over the rows, so a reply's positions are not treated as independent.
> - **A row is identified by its `position`,** the field `files_ok` aligns across files. In a pooled panel, the rows with one `position` at the three seeds form one row k. Row 0 checks that `position` is unique within each seed and carries the same prompt and reference at every seed.
>
> **The bar:**
> - **Q1, no detectable loss:** b ≤ a, or (b − a)² ≤ 9·V. Compared in integers, in every cell and in both pooled panels.
> - **Q2, a bounded loss:** b − a ≤ ⌊n / 200⌋, at most half a point of next-token accuracy over the n positions read, in every cell and in both pooled panels.
>   - Q2 also covers what Q1 cannot see: a loss confined to 9 rows or fewer never fails Q1, since (Σ_k d_k)² ≤ m·Σ_k d_k² over m rows.
> - **Q3, the opening, read on free decode**, from claude-243's sealed greedy replies.
>   - **R is right at the opening** when its greedy reply is not empty and its first token equals reference[0]. **F′ is right** when its greedy first token equals reference[0].
>   - **It is like for like.** `files_ok` checks that both greedy files carry the row's own prompt and reference (`grade-cell.jq:20–22`). Row 0 also checks F′'s sampler (defined in Row 0 below).
>   - a₀, b₀ and V₀ are formed as above, one position per row, and Q3 is Q1's test on them.
>   - **It is tested in six strata:** the counted rows in each p-w cell, the counted rows pooled per panel (DEV and p-w), and p-w's cut windows pooled.
>   - Two kinds of stratum cannot fail, so they are reported as readings: a DEV cell (at most 4 openings disagree) and DEV's cut windows (at most 2 disagree). DEV's pooled counted rows (11 disagree) can fail only if almost every disagreement goes F′'s way.
>   - On counted rows this is the same opening as teacher forcing's offset 0. claude-244's grader checks both sides there: I-Align for F′ and I-Forced-Greedy for R.
> - **Q4, the rounding signature:** κ ≥ 4/9, clear of chance at 3 SE, in every cell.
>   - For claude-244's R this is its registered row, already met in all six cells. It is carried, and the next unit's sheet cites claude-244's sealed grade.
>   - **For any other artifact or recipe,** such as a finer fixed point or a fix to a located fault, Q4 is read again.
> - **Q5, the clear margins: required, with its rule fixed by its own card.** Done-when 3 also requires an exact trace accounting for every counted disagreement where R's margin exceeds its cell's G₁₀: for this R, 151 on p-w and 11 on DEV. p-w's cut windows add their own clear-margin disagreements: at every position if I-Align holds there, and at the opening only otherwise (below).
>   - **This card does not approve the trace's rule.** Round 2 showed that a rule written here was not yet decidable.
>   - **The Q5 card must meet four requirements:**
>     1. Its rule, and any bound it uses, is fixed before either side's values are read.
>     2. It traces each position's whole causal prefix, not the position alone.
>     3. It has its own Row 0: the float side reproduces F′'s nearest, the integer side reproduces R's pick, and a rerun matches. A position that cannot be traced is an instrument repair, not a fault.
>     4. It states its writer, any scope change, and its cost.
>   - **The leading proposal, for that card:** a three-way reading against R's float twin T, the same artifact evaluated in float by `fbs-snapshot` (in the item's scope).
>     - If T agrees with F′ and not with R, the cause is R's integer arithmetic, anywhere in the causal prefix. If T agrees with R and not with F′, the cause is the exported structure.
>     - It needs no rounding bound. Whether `fbs-snapshot` carries the memory and the read on this path is that card's first check.
>   - Q5 is read again for any other artifact or recipe.
> - **The cut windows' later positions.** Row 0 reads I-Align on the cut windows too: F′'s rows against F′'s greedy reply, wherever its greedy prefix equals the reference.
>   - **If it holds,** F′'s rows read the same window there. It holds when there are 0 violations and every p-w cut-window row at every seed (135 per seed, 405 in all) is checked at least at its opening. Offset 0 is always eligible, so a row goes unchecked only if F′'s greedy reply is empty, and any such row means it does not hold. p-w's cut windows then join Q1 as a pooled stratum, and their clear-margin disagreements join Q5's set.
>   - **If not,** their later positions stay readings with no bar, and only their clear-margin opening disagreements join Q5's set.
> - **Row 0** is the instrument checks. The next unit's grader runs them all again before it reads anything. If any fails, nothing is read and the instrument is repaired. They are:
>   - claude-244's `files_misaligned`, `n_wrong`, `counted_short`, `forced_greedy_violations` (with `forced_greedy_checked` > 0), `align_violations` (with `align_checked` > 0), `targets_wrong`, `coverage_missing`, `margins_negative` and `runner_up_wrong`;
>   - its sheet's I-Inputs, I-NoKey and I-Exit;
>   - the uniqueness of `position`;
>   - **the check on F′'s sampler.** This is a code-path check, made before any file is opened. The next sheet registers it with its file and lines at the commit F′ ran at (claude-243's).
>     - It passes if F′'s greedy decode for a row conditions only on that row's stored prompt, the field `files_ok` matches, and on no reference token and no panel token outside it.
>     - It fails if the path reads anything else.
> - **Pass:** all of the following hold.
>   - Q1 holds in every cell, in both pooled panels, and in p-w's cut windows if they join.
>   - Q2 holds in every cell and in both pooled panels.
>   - Q3 holds in its six tested strata.
>   - Q4 stands.
>   - Q5 holds under its card's rule.

**How the sampler sentence is read** (claude-245's exact-head review, carried Nit 3): "conditions only on that row's stored prompt (the prompt that `files_ok` matches), and on no reference token and no panel token outside it". The parentheses add no condition.

### 3.2 The strata, as the grader forms them

| Test | Tested strata | Readings, never failing |
|---|---|---|
| Q1 (teacher forcing) | the 6 cells; DEV pooled; p-w pooled; p-w's cut windows pooled, only if they join | DEV's cut windows pooled; p-w's cut windows pooled, if they do not join |
| Q2 (teacher forcing) | the 6 cells; DEV pooled; p-w pooled | none |
| Q3 (free-decode opening) | p-w s1, s2, s3; DEV pooled; p-w pooled; p-w's cut windows pooled | DEV s1, s2, s3; DEV's cut windows pooled |
| Q4 (carried) | the 6 cells, from claude-244's sealed `grades/cell-SPLIT-sN.json` | none |

- **In integers.** a, b, V and n are integer counts. Q1 is `b ≤ a` or `(b − a)² ≤ 9V`, and Q2 is `b − a ≤ ⌊n/200⌋`. Every product stays far below 2^53, so `jq`'s doubles hold them exactly.
- **Pooled.** A pooled stratum takes the rows of its three cells. a, b and n are summed, and V = Σ_k (Σ_seeds (b − a))² over the positions k, so one row at three seeds is one row.
- **Q4's carry.** For each of claude-244's cells, κ ≥ 4/9 is read in integers as `9·d_low·N ≥ 4·d·N + 5·d·N_low` (N = `positions_counted`, N_low = `all_low`), and clear of chance as `c − q ≥ 3·√(q(1 − q)/d)` (c = d_low/d, q = N_low/N), as claude-244's sheet defined them. Row 0 checks first that claude-244's sealed κ follows from those fields.

### 3.3 The outcomes and what each unlocks

The outcome is read on the tested strata only, after Row 0 holds.

| Outcome | What holds | What it unlocks |
|---|---|---|
| **LOSS** | Some tested stratum fails, named by test and stratum. | A LOSS of any class fails done-when 3 for R, and no later card re-bars it once read. The next unit classifies the failing strata by trace (CHARTER §4.2) only to choose the remedy: a finer fixed point for rounding, or a located fix for a fault, after which Q1 to Q5 are read again on the new artifact. The one exception is an instrument fault that Row 0 missed, shown by a rerun to have made the count wrong: then this reading is void, repaired and read again. The item stays `ready`. |
| **Held** | Every tested stratum of Q1 to Q4 holds. | Q5's card, then its sheet with a FIT and a pre-run review, then the trace. Done-when 3 is met only if Q5 holds as well. The item stays `ready`. |

This unit does not classify a LOSS; it states it with its failing strata.

## 4. Row 0: the instrument checks, all before any reading

The grader runs every check below in order and stops at the first failure. Nothing is read until all hold. On a failure the instrument is repaired, inside this scope if it can be, the failure is recorded, and the whole grade is rerun into a new folder.

1. **The sheet's gates:** the last `TEST FITNESS:` line here reads `FIT`, a passing pre-run review is recorded in §10, and the output folder does not yet exist (L-002).
2. **I-Inputs:** 43 input files pass against their sealed sums, and each count is checked:
   - claude-244's six forced decodes, six cell grades, `grade.log` and six forced-decode logs (19 files), against its `SHA256SUMS`;
   - F′'s six rows files and six greedy files (12), against `tie-parity/SHA256SUMS`;
   - R's six stored greedy replies (6), against claude-243's `seal/laptop/SHA256SUMS`;
   - claude-232's six prompt files (6), against its `SHA256SUMS`.
   - The four local manifests each equal the store's copy, fetched afresh for this unit into `local/claude/claude-246/probe/`, and each has the sha256 registered in §5.
   - claude-244's check of R's artifacts against `R/a7/SHA256SUMS` is not carried: R is not opened here, and its picks are read only from the sealed decodes above.
3. **I-NoKey:** every row of R's six stored greedy files has `keyed` null, so no opening of R was served by a point key.
4. **I-Exit:** claude-244's sealed `grade.log` ends `GRADE-DONE` and logs six forced decodes. Its runner stopped on any decode's non-zero exit before that line. `margins_negative` is 0 in every cell (check 9).
5. **Q4's carry:** in each of claude-244's six sealed cells, `kappa` equals (c − q)/(1 − q), with c = `d_low`/`d` and q = `all_low`/`positions_counted`, to within 1e-12, and `d` > 0.
6. **The row identity:** at every seed, `position` is unique. The three seeds list the same positions in the same order, and each position carries the same prompt and reference at the three seeds. p-w has 135 cut-window rows (`prompt[0] ≠ 0`) at every seed.
7. **The sampler check** (§4.1) is recorded as passing, and `in_space_long.rs` at F′'s commit is the blob it was made on.
8. **The grader's probes,** on hand-made files with hand-counted answers:
    - a 3-row cell (two rows counted, one cut; 7 positions, 5 counted and 2 cut) reads exactly as worked out: its positions, checks, and every row's a, b, both, a₀, b₀ and both₀;
    - its checks pass holds no readings;
    - a cut row with an empty F′ greedy reply goes unchecked, and a missing F′ row fires the cut-window violation and coverage counts;
    - seven decision cases decide as stated: all hold; a Q1 loss; a Q2 loss; a DEV cell's Q3 is a reading; the cut windows join and fail Q1; an unchecked cut row stops the join; a Q4 loss.
9. **Each cell's checks,** with no reading kept (the checks pass computes its readings in memory and drops them before any output). All of claude-244's: `files_misaligned`, `n_wrong`, `counted_short`, `forced_greedy_violations` with `forced_greedy_checked` > 0, `align_violations` with `align_checked` > 0, `targets_wrong`, `coverage_missing`, `margins_negative` and `runner_up_wrong`, each 0 where it counts failures. Also `position` unique in the cell.
10. **I-Coverage,** in the same pass: each cell's positions sum to claude-244's mode-printed `positions`, and its offset-0 positions number its counted rows.
11. **I-Offset0:** R's forced offset-0 pick against F′'s greedy first token, by claude-243's rule, reproduces 4, 4, 5 on DEV and 64, 47, 78 on p-w.

**The cut windows' I-Align** is read after check 11, over p-w's 405 cut rows. It decides only whether they join Q1; it is not a stop. If they join, their positions must also show `cut_targets_wrong` and `cut_coverage_missing` at 0, or Q1 on them is undefined and the grade stops as Row 0.

### 4.1 The sampler check, at F′'s commit

- **The commit.** claude-243's GPU job built `train-in-space-4170b844` (sha256 `51141f26d36fe14c289778029878291819b7a33bdf710b4a8eb5b341738db26e`, in its `job.log`), named by its build commit. That prefix resolves uniquely to `4170b844744ed5f83f452d87614dc5fc9293bd23`.
- **The command** was claude-220's, as claude-243's sheet registered it: `train-in-space --score-rows CKPT … --samples 1536 --reply-tokens 64 --retain-rates 1 --flock-end 1 --opening-keys 1 --served-key 1 --panel p-w=PW --panel p-v=PV --seed S`.
- **The file:** `crates/uor-r4-float-train/src/in_space_long.rs` at that commit. All line numbers below are at it.

**The path that writes F′'s greedy files:**
1. `score_rows` (5254). For each split, the prompts are `sample_windows`' (5443; defined at 4062). Each row's prompt is `window.input_ids[..=off]` (5449), the window's tokens up to its first scored offset.
2. The first decode rule is greedy over the full read (5431, `(DecodeRule::Greedy, ReadScope::Full)`), with the memory on in the first pass (5403–5404). It calls `decode_reply` on that prompt (5452–5463).
3. It writes the reply to `samples-memory.json` on DEV and `samples-p-w-greedy-memory.json` on p-w (5495–5496, 5511). The row's stored `prompt` is the same slice (5470), the prompt `files_ok` matches.
4. `reference_reply` (5464) and the key hits (5465–5467) are computed after the decode returns, and are only recorded.
5. The call also passes the prompt's response-mask flags (5451, 5462). `decode_reply` reads them only under a read scope other than `Full` (4445–4455, 4511–4517), and the greedy files are written under `Full` (5431).

**Inside `decode_reply` (4427), greedy:**
- `ids` starts as the prompt (4463). Each step's forward reads `window`, the last `SHEET_CONTEXT` tokens of the prompt and the reply so far (4484, 4518), and nothing else from the panel.
- **The forced opening** (4471) is a thread-local that is None unless `score_rows`' forced, keyed or served arms set it (5535). Those arms run after the greedy loop, and each resets it to None right after its decode, before an error can return (5549). So the greedy files never read it.
- **The served point key** (4526–4528, 4371–4373) is likewise None outside those arms (5536, reset at 5550). It is built from TRAIN in any case (5370–5374).
- **The turn memory** (4487–4508) is the checkpoint's own, loaded from its directory by `load_reads` (418, the memory at 432). Training built it from the training split's openings and replies (`start_memory`, 548–616; the split at 569–572). It is keyed by the window's own tokens.
- The pick is the whole table's nearest by score, the arg-max (4529, 4534–4538). It stops at BOS (4587).
- The served arms that set the forced opening and the key run through `opened` (5528), called after the greedy loop (5605).

**No read keeps state from one forward to the next,** so the panel rows scored before a decode cannot reach it. The other files are cited at the same commit:
- `score_rows` takes no optimizer step: its body (5254–5657) has no `backward`, `step` or `Var::set`.
- **The turn memory** (`flock_read_memory.rs`): its tables are `Arc<HashMap>` and are never written after they are built (148–156). `stored`, `stored_reply` and `apply_readout` take `&self` and read only the window's ids and the rows named for that forward (254–359). Its weight is a `Var`, which only an optimizer step could change.
- **The rows named for the memory:** `decode_reply` sets them before each forward and clears them right after (4491, 4503–4506, 4519–4521).
- **The prompt register's slots** (`geometric_stack_native.rs`) are cleared at the start of every forward (1116–1118). They are set within it (1170) and taken at its readout (1193).

So F′'s greedy decode for a row conditions only on the row's stored prompt, the checkpoint and its TRAIN-built tables, and on no reference token and no panel token outside the prompt.

SAMPLER CHECK: PASS

SAMPLER BLOB: 0b71ab376b34827e710d0303cae3e1cf22b6876c

## 5. Inputs, by sealed path and hash

Every input is read from its local copy, checked against the manifest below, and each manifest is checked against the store's.

| Input | Store path (manifest) | Manifest sha256 |
|---|---|---|
| claude-244's forced decodes, cell grades, `grade.log` and forced-decode logs | `results/claude/integer-nearest-readout-margin-2026-11-01/SHA256SUMS` | `671356cc15ebc7ef82503638523ad595d51a40dc5dd8050dea33d64b42098cb1` |
| F′'s rows and greedy files (claude-243's GPU run) | `results/claude/integer-nearest-readout-tie-parity-2026-11-01/tie-parity/SHA256SUMS` | `f4292fe0a3774339301f7e836661c59ac0b468972b5b8c24d09a62f7ca4713bf` |
| R's stored greedy replies (claude-243's laptop run) | `results/claude/integer-nearest-readout-tie-parity-2026-11-01/laptop/SHA256SUMS` | `801b8ad875132237f42f830088c9c53fb72660d03b0c66bc1e7ec02e0a383c3d` |
| claude-232's prompt files | `results/claude/integer-nearest-readout-parity-2026-10-31/SHA256SUMS` | `b739b7789075f46a7c4d4c003ebf7b3a10d47e1c53fe0b77bfe7e5d50db9bb88` |

- **R** is claude-243's raw export of (r-drop) step 16,384, `a7r-s1/2/3.uorlut`. It is not opened here: its picks are claude-244's sealed forced decodes and claude-243's sealed greedy replies.
- **The data split** is DEV and the held-out-phrasing panel p-w, through claude-232's prompt files: 310 rows of DEV (284 counted, 26 cut) and 1,202 of p-w (1,067 counted, 135 cut), at each seed. Both were already opened for parity by claude-232, claude-243 and claude-244. No TRAIN data and no other panel is read.
- **The files' structure,** read with no pick compared, in all six cells: no reference is empty and none opens with `<|bos|>` (id 0). No greedy reply of R or F′ is empty, and none contains id 0. So every cut-window row is eligible for I-Align at its opening.
- **Seen before this sheet** (disclosed, not a reading of the bar): while finding F′'s commit, the author saw claude-243's GPU log print F′'s own `nearest_hits` over every scored row of DEV and p-w at seed 1 (11,841 of 14,287 and 37,049 of 49,251). That is F′ alone, over rows and positions the bar does not count, and it gives neither a nor b. The bar was fixed in claude-245 before it was seen.

## 6. Steps and the grader

1. This sheet gets a fresh non-author FIT (§9) and a pre-run review (§10).
2. Run `local/claude/claude-246/grade.sh` once, its output to `grade.log`. It runs Row 0 (§4) in order, then the six cells' readings, then the decision.
3. Seal `grades/`, `grade.log`, the grader and its probes with a `SHA256SUMS` under `results/claude/integer-nearest-readout-grade-2026-11-01/`, and read them back.
4. Record the outcome: the item's "What is missing", a row in `m4-serving.md`, and the REPORT, negatives first.

**The grader**, kept beside its outputs in `local/claude/claude-246/` and sealed with them:

| File | sha256 |
|---|---|
| `grade.sh` (the runner) | `5433335b85e9912cb9bbdef73292dd66af82778c54fe8144bde21471681df7c0` |
| `grade-cell.jq` (one cell: checks, then readings) | `6bac0aa087ff2db68743cef6340f326dcba1da16a5a8e9b822bebf7040029781` |
| `decide.jq` (Q1 to Q4 over the strata, and the outcome) | `783a058c6c9ff6b615b5ab174205824bef800d6af871ca4ea395928b9e684300` |
| `probe/decide-cases.json` (the seven decision cases) | `2275ed10710f1127a1976e9ee4fea8e03f4d05f9b3fa87e759e289779fd6e94b` |

- `grade-cell.jq` is claude-244's grader with every check unchanged. It adds each row's `position`, each offset's reference token, each row's free-decode openings (R right, F′ right), the cut windows' I-Align, coverage and target counts, and `position`'s uniqueness. Its readings are per row: n, a, b, both, a₀, b₀ and both₀.
- `decide.jq` forms the strata of §3.2 from those rows, applies Q1 to Q4, and states the outcome with its failing strata.

## 7. Minimum test power and the false-fail rate

These are the card's figures, under its assumptions.
- **Q1** detects a net loss of about 3·√V positions. With independent rows V would be about a + b, at most the cell's disagreements: at most 59 to 64 positions in a p-w cell, 20 to 27 in a DEV cell, and 106 of 41,793 (0.25%) on pooled p-w. Correlation within a reply makes V larger, and the clustered V widens the test by what it warrants.
- **Q2's** half point is 69 positions in a p-w cell and 20 in a DEV cell, so it is the tighter test on DEV.
- **Q3** detects a net opening loss of about 17 to 23 of 1,067 in a p-w cell (1.6 to 2.1 points), 34 of 3,201 on pooled p-w (1.1 points) and 23 of 405 in p-w's cut windows. A smaller opening loss passes.
- **The false-fail rate,** with exactly zero-mean flips: each one-sided 3-SE test fails by chance with probability about 0.135%, over at most 15 such tests (Q1 in up to nine strata and Q3 in six). Q2 on the three DEV cells adds at most about 1.3%, 0.8% and 0.2%, even if every disagreement fell in a or b. These are normal approximations without continuity correction, which do not understate them (carried Nit 1). That is about 4.3% in all under independence; Q2 is not clustered, so correlation within a reply raises its share. Rounding that favours F′ raises the rates, and the bar counts that as loss by design.

## 8. Cost and stop rule

- **Cost:** $0. A count in `jq` on the laptop, minutes long. No pod and no lease.
- **Stop rule:**
  - Any Row 0 check that fails stops the grade before any reading. The instrument is repaired, the failure recorded, and the whole grade rerun into a new folder; a partial count is never read.
  - The readings are computed once. A rerun after them is only for an instrument fault, shown by a rerun.
  - A bar change is only ever a new decision card, with its own decision review, logged on #3462.
- **Nothing in CHARTER §3 or the caps is touched.**

## 9. Non-author TEST FITNESS review

**Round 1** (a fresh `delegate:reviewer`, 1 Nov 2026, on the sheet at `2eda041ec` and the grader). It counted no pick against the reference and did not run `grade.sh`.

TEST FITNESS: NOT_FIT

- **Why NOT_FIT:** the reviewer reached its context limit before it finished. It found no Blocker or Major.
- **What it confirmed:**
  - the grader's four hashes, and the source blob and binary;
  - `grade-cell.jq` keeps every claude-244 check unchanged, and its additions match the card's definitions; the probe matches its own hand count;
  - `decide.jq` forms the card's strata, clusters V by `position` across seeds, and compares in integers;
  - Q4's integer form equals κ ≥ 4/9;
  - Row 0 stops before any reading, and each check can fail;
  - every sampler line it read is cited correctly;
  - the outcomes and the power figures match the card.
- **Its findings, and what was done:**
  1. Minor: §4.1 left out the response-mask argument of F′'s call. Added: it is read only under a read scope, and the greedy files are `Full`.
  2. Minor: I-Inputs did not check the manifests against the hashes in §5. The runner now does, for all four.
  3. Nit: the probes ran before the cell checks, against §4's order. §4 is renumbered to the runner's order.
  4. Nit: the probe is 3 rows (7 positions), not 7 rows. Fixed.
  5. Nit: claude-244's check of R's artifacts was dropped without a word. §4 now says why.
- **Its open items, which the author then checked** (each for round 2 to confirm):
  - **Whether the turn memory changes during a run:** it does not, nor does any other read keep state between forwards (§4.1, with lines in `flock_read_memory.rs` and `geometric_stack_native.rs`).
  - The lines of `start_memory` are cited (§4.1).
  - The quoted bar is verbatim: the quote, with its `> ` prefix stripped, matches the card's lines 56 to 104 under `diff`.
  - The files' structure (§5): no empty or BOS-opening reference, and no empty greedy reply or BOS inside one, on either side.

## 10. Pre-run review

Pending, after the FIT.
