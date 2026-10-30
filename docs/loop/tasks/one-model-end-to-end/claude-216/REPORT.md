# REPORT: one-model-end-to-end, claude-216 (train with the reply's read held to the prompt)

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md). Order: [ORDER.md](ORDER.md). Sheet: [one-model-end-to-end-cut-train-2026-10-30](../../../../harness/prep/one-model-end-to-end-cut-train-2026-10-30.md): NOT FIT in round 1, FIT in round 2 (at `c0421966a`; its five notes landed at `5f20e7f71`, which the pre-run review checked), and the pre-run review was GO. All three came from fresh reviews, posted on #3407. Results: [cut-train-2026-10-30](../../../../labs/one-model-end-to-end/cut-train-2026-10-30/README.md). References #3407.

## Tokens per second

- **Training, (r-cut), 16,384 steps of 16 × 192 tokens:**

  | Seed | Time | Context tok/s |
  |---|---|---|
  | 1 | 1,477 s | 34,077 |
  | 2 | 2,159 s | 23,312 (a slower pod) |
  | 3 | 1,492 s | 33,734 |

  (r-drop) trained in about 1,470 s on a fast 5090, so no training cost of the cut is measured.
- **Decode, greedy with the memory on, p-t:**
  - **91–93 tok/s per process** on the two fast pods, and **71–72** on the slow one (70.9 for the decode-only arm). Two scorings shared each GPU, so these are floors.
  - With the cut, decode is at most 1 tok/s slower than without it.
- **This is not the served speed.** The integer path with the row cut is not built.

## What did not work, first

- **The registered outcome is "reduces, content lost."** The recipe does not change: (r-cut) does not become the one model's recipe.
  - **What failed:** the matched content at seed 2.
    - The base, (r-drop) greedy, has 100 of 197 matched prompts verbatim (0.508). The served arm has 69 (0.350).
    - That is a gap of 0.157, against a tolerance of 0.107.
  - **Seeds 1 and 3 keep it:**
    - seed 1: 0.412 against 0.471, a gap of 0.059 against a tolerance of 0.105;
    - seed 3: 0.337 against 0.404, a gap of 0.067 against a tolerance of 0.102.
- **One reading: the content loss is in the trained model, not only in its decode.**
  - (r-cut) decoded without its cut, the uncut arm, is no more TRAIN-like: 0.412, 0.350 and 0.308 on the same prompts. And 131–163 of the matched replies are the same with and without the cut.
  - The uncut arm also loops less than the base: 30 against 60 at seed 2 and 45 against 76 at seed 3, beyond the band at both (it does not hold decision 4 at seed 3). So part of the loop drop is in the trained model too.
  - The uncut arm is one more arm at the same seeds, not a control for the cause.
- **The loops that end are often replaced by salad, or by a loop inside a word.** Examples from the [fixed reading](../../../../labs/one-model-end-to-end/cut-train-2026-10-30/fixed-reading-s1.md) of seed 1's first 20 replies:
  - "Here are twoedkoum."
  - "Got it, your lucky number is tttt,."
  - "Yum,ayee legs."
  - Replies 4, 9 and 19 end in "…inginging…".
- **The served arm starts loops the base did not have:** 17, 12 and 10 replies start looping, while 54, 55 and 67 stop.
- **Replies end much sooner.** The served arm's mean length is 15.2, 11.2 and 12.6 tokens, against the base's 21.4, 22.2 and 24.0. Part of the drop in loops is the ending itself.
- **No arm is "close" on p-t at any seed,** the base included. "Close" needs a repeated-4-gram rate within 0.10 of the references' 0.000.
- **On DEV the loops fall by less.** The base loops in 46, 28 and 29 replies, the served arm in 16, 24 and 26. The fall is beyond the band at seed 1 only, where the served arm is also close (repeat rate 0.057).

## What worked (reported; decides nothing here)

- **The served arm's loops fall beyond the band at every seed, and it holds decision 4.**
  - **Looping replies:** 27, 17 and 19, against the base's 64, 60 and 76.
  - **Complete:** 247, 253 and 249, against 212, 200 and 212.
  - **Repeated-4-gram rate:** 0.192, 0.103 and 0.187, against 0.522, 0.587 and 0.522.
- **Training the cut in costs no teacher-forced accuracy.** (r-cut) against (r-drop) on every p-t row, paired:
  - the differences are −62, +200 and −146 rows, inside the band of 346 at every seed;
  - on DEV (r-cut) is right on 11,902, 11,979 and 11,943 rows, against (r-drop)'s 11,841, 11,846 and 11,820.
- **The decode-only arm, (r-drop) decoded with the cut, reads "improves" by the same rule.**
  - **Loops:** 35, 38 and 52 against 64, 60 and 76, fewer beyond the band and holding at every seed. Paired, 32, 27 and 38 replies stop looping, and 3, 5 and 14 start.
  - **Matched content:** 0.475, 0.503 and 0.399, against the base's 0.471, 0.508 and 0.404. That is kept at every seed.
  - **Why it differs from claude-215's arm:** this arm uses this unit's rule, which also holds an earlier reply in the prompt. And the content is read on matched prompts. claude-215 read its arm's content on each arm's own completions and got "reduces, content lost". Its [REPORT](../claude-215/REPORT.md) named a selection effect as a possible cause. This matched reading fits that, but it is one panel and decides nothing here.

## What changed

- **The cut, one rule from the mask** (`crates/uor-r4-float-train/src/in_space_long.rs`).
  - **The rule:** every row whose input is a reply token reads, in the scoped heads, only before its reply's first token in the window, or nothing when the reply began before the window (`reply_row_limits`).
  - **Where the flags come from:** the split's response mask at each window position (`window_input_mask`). A decode window gets its flags from the prompt's flags plus the reply's tokens (`decode_window_mask`).
  - **`trained_points`** takes each forward's mask from `set_reply_mask`, and refuses a forward under a cut that has no mask, or one of the wrong length.
  - **`decode_reply`** limits every reply row through the read's row cut. It refuses a scoped prompt that ends inside a reply.
  - **`cut_census`** counts the rows the cut limits, and `run_long` logs it (`[cut] TRAIN rows`).
  - **Which decodes carry the cut:** only `--reply-read`'s decodes (`samples-{split}-greedy-read-{scope}-memory.json`, the served and decode-only arms). A cut-trained run's own mid-run samples, and every plain greedy file, decode uncut, as the uncut arm does.
  - **claude-215's uniform decode cut** (`set_read_cut`) stays, with its CUDA test; decode now uses the row cut.
- **The read's row cut** (`crates/uor-r4-float-train/src/flock_read_hamming.rs`): `set_row_cut`, `row_cut_set` and per-row `row_limits` in `choose_sorted`.
- **Training with the cut** (`--reply-read-train all|copy`, `crates/uor-r4-float-train/src/bin/train-in-space.rs`):
  - the run's `config.json` records it as `reply_read_train`;
  - `--score-rows` reads it back, applies it to the teacher-forced rows, and prints it.
- **The grade, `--cut-grade 1`** (`cut_grade`):
  - **Matched content:** the verbatim TRAIN share on the prompts the base and the served arm both complete (`matched_counts`, `MATCHED_BY`). Each arm keeps it within `max(0.10, 3·√(p(1−p)/m))` (`matched_kept`).
  - **The content floor:** 64 matched prompts (`MATCHED_FLOOR`), below which the content reads `underpowered (content)`.
  - **The outcome:** `cut_outcome`, in the sheet's order.
- **The panel p-t** was drawn by `heldout-panel` at seed 2081, with its census: 0.000 of 1,424 prompts are in TRAIN, exactly or as templates at K = 1,024, and 0.610 of its replies are verbatim. It is stored at `inputs/claude/heldout-panel-p-t-2026-10-30/`.
- **Also written:** the prep sheet, the grade ([cut-grade.json](../../../../labs/one-model-end-to-end/cut-train-2026-10-30/cut-grade.json)), the fixed reading, the lab page and one notebook row. The pod job and its watcher are local, in `local/claude/claude-216/`:
  - `job.sh` sha256 `a2bfc191c74cf37689edc051a8e679bbc71b0c2672cbfa546cd6e020e97872f5`;
  - `watch.sh` sha256 `cec529da63a72f9746991897d46608e1eb710753814f95b612c1ecdb971d0c7b`.

## Measurements

| p-t, seeds 1 / 2 / 3 | Base: (r-drop) greedy | Served: (r-cut) cut | Decode-only: (r-drop) cut | Uncut: (r-cut) greedy |
|---|---|---|---|---|
| Looping replies | 64 / 60 / 76 | 27 / 17 / 19 | 35 / 38 / 52 | 62 / 30 / 45 |
| Complete | 212 / 200 / 212 | 247 / 253 / 249 | 237 / 244 / 248 | 205 / 231 / 216 |
| Matched verbatim share | 0.471 / 0.508 / 0.404 | 0.412 / 0.350 / 0.337 | 0.475 / 0.503 / 0.399 | 0.412 / 0.350 / 0.308 |
| Mean length (tokens) | 21.4 / 22.2 / 24.0 | 15.2 / 11.2 / 12.6 | 16.2 / 15.2 / 16.6 | 22.0 / 15.2 / 18.9 |
| Repeated-4-gram rate | 0.522 / 0.587 / 0.522 | 0.192 / 0.103 / 0.187 | 0.293 / 0.241 / 0.195 | 0.544 / 0.434 / 0.530 |

- **Matched prompts:** 204, 197 and 208, past the floor of 64.
- **The references:** `v_ref` 0.516, and a repeat rate of 0.000.
- **The power floor is met:** the base loops in at least 40 replies at every seed.
- **The training cut** limits 2,775,229 of TRAIN's 13,097,856 rows. Of those, 1,885 (0.07%) read nothing, their reply having begun before the window; decode almost never meets that case.
- **The run:** three RTX 5090 pods (`vast-58396104`, `vast-58396424`, `vast-58396525`), $2.03/h together, about $2.4 in all.
  - **Sealed** at `results/claude/one-model-end-to-end-cut-train-2026-10-30/run-5f20e7f7/`:
    - `s1`, 130 files; `s2`, 132; `s3`, 132; and `bin`, the binary, sha256 `7e0415e0cae19cae58fa4e9889c426018613ff0aca129e88a0fb105d69c7a623`, the same on every pod.
    - Each matched the pod file for file, and was read back from the store OK.
  - **Every pod is down.**

## Checks run at the head

- **The crate's host tests** (`cargo test --locked -p uor-r4-float-train --lib`): 169 pass.
- **After the review's fixes** (the grade's rule list as a constant, pinned by a test), `--cut-grade 1` re-run on the sealed outputs gives the committed grade byte for byte.
- **Four negative gates at the final rule** each fail with their line set aside, and pass when restored:
  - **the content floor** (`the_cut_grade_matches_content_and_follows_the_sheet`);
  - **the mask rule's per-window reset** (`every_reply_row_reads_only_before_its_reply` and `a_decode_window_is_limited_as_its_training_window`);
  - **the decode window's prompt flags** (`a_decode_window_is_limited_as_its_training_window`).
  - The earlier gates (the per-row cut, the matched set, the trained cut's record) were run on this unit's earlier commits.
- **The stub-nvcc CUDA clippy run** (`--features cuda --all-targets`): no new findings.
- **On every pod:**
  - the CUDA tests pass;
  - every input matches its sha256;
  - the default path's 256-step probe reproduces claude-211's sealed probe byte for byte, 5 of 5 files, across pods, so the fallback was not needed;
  - the cut limits more than 0 rows;
  - each scoring printed the trained cut it read (`all` for (r-cut), `none` for (r-drop));
  - claude-211's 5 registered files reproduce byte for byte;
  - every expected p-t file is written.
- **`--cut-grade 1`** on the sealed run, at `5f20e7f71`: exit 0.
- **Re-counted independently with `jq` from the sample files:** the matched prompts (204, 197 and 208) and every arm's looping replies. Both agree with the grade.

## Skills used

None.

## Still open

- **The owner's direction** (Casey, 30 Oct, verbatim on #3407 6523338624): "Yes: after claude-216's grade, register a decode unit that steps off a reply's own cycle (the nearest token outside a repeated 4-gram), on a fresh panel, against greedy."
  - **It is the next unit.** By this grade it decodes the sealed (r-drop).
  - **Its sheet can carry the decode-only cut as a second registered arm,** to confirm this unit's reported "improves" on a fresh panel.
- **The registered follow-up for "reduces, content lost":** the turn memory's reply table, keyed by the reply's last id, which the cut leaves reading the reply.
- **Serving in integers,** the shipped model and then its decode tok/s, with the row cut if a cut ships.
- **p-t has now been opened once,** by this decision.

## Plain summary

The model's replies get stuck repeating themselves. Last time, stopping its copy mechanism from looking back at the reply it is writing removed some loops, but garbled other replies. So this time I trained a model that never looks at its own reply, in training or in use.

That model loops far less: under half as often as the current one, and about a quarter as often at two of three seeds. It is just as accurate at predicting the next word when shown the real text. But its own replies are shorter, often garbled, and at one of three seeds clearly less like real replies, so as registered it does not replace the current model. A side reading, which decides nothing yet, is that the current model with the same look-back limit, and no retraining, cut its loops by about a third to a half without losing content. The next unit tests a new way to step out of a loop, as Casey asked, and can check that side reading too.
