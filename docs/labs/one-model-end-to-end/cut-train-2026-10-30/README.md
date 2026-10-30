# Train with the reply's read held to the prompt (claude-216, 30 Oct 2026)

Lab: claude. Item: [one-model-end-to-end](../../../roadmap/items/one-model-end-to-end.md). Order: [claude-216](../../../loop/tasks/one-model-end-to-end/claude-216/ORDER.md). Sheet: [one-model-end-to-end-cut-train-2026-10-30](../../../harness/prep/one-model-end-to-end-cut-train-2026-10-30.md): FIT in round 2, pre-run GO.

## The run

- **The models:**
  - **(r-cut):** (r-drop)'s recipe plus `--reply-read-train all`, trained here for 16,384 steps at seeds 1, 2 and 3.
  - **(r-drop):** its sealed step-16,384 checkpoints at the same seeds.
- **The cut, one rule everywhere:** every row whose input is a reply token reads, in every Hamming head, only before its reply's first token in the window, or nothing when the reply began before the window. Its flags come from the response mask. Training, the teacher-forced rows and decode all apply it.
- **The arms**, each greedy with the memory on, 256 replies per split and seed, on DEV and on the new panel p-t (seed 2081):
  - **base:** (r-drop) as trained;
  - **decode-only:** (r-drop) with the cut;
  - **served:** (r-cut) with its cut, which decides;
  - **uncut:** (r-cut) without it.
- **The hardware:** three RTX 5090 pods, one per seed, built at `5f20e7f71`. The binary's sha256, the same on every pod, is `7e0415e0cae19cae58fa4e9889c426018613ff0aca129e88a0fb105d69c7a623`.
- **The instrument checks, on every pod:**
  - the CUDA tests pass;
  - every input matches its store sha256;
  - a 256-step (r-drop) probe reproduces claude-211's sealed probe byte for byte (5 of 5 files, across pods);
  - the (r-cut) probe records its cut and limits 2,775,229 of 13,097,856 TRAIN rows (1,885 of them read nothing);
  - each scoring printed the trained cut it read;
  - claude-211's 5 registered files reproduce byte for byte.
- **Sealed** at `results/claude/one-model-end-to-end-cut-train-2026-10-30/run-5f20e7f7/`, each part read back OK. The run's own `SHA256SUMS`, sha256 `1cc9365a95afe5cd486132e7c5039a1c4e661068e7cff360d2d9d909486568b4`, lists the four below. Each seed folder's `SHA256SUMS` has this sha256:
  - `s1`, 130 files: `a6edc174ade23f24bb03c4992d785ccfce6b3b375ff35fc45a6aae8273a41f66`;
  - `s2`, 132 files: `032eda568e5d6a436027265e8c59872f61bdf5f76e3f16852618831230829562`;
  - `s3`, 132 files: `102fd62064a6f61808befe3aefbf6eae8571fc70ef1fb9aaed0328401caf5edc`;
  - `bin` holds the binary.
- **Graded** on the laptop by `train-in-space --cut-grade 1` at `5f20e7f71`: [cut-grade.json](cut-grade.json), sha256 `3b3ab7d306d066527dcbd0bf76c7e00c75133f9c6d6d80e7e7b9b232018c017c`. The [fixed reading](fixed-reading-s1.md) is seed 1's first 20 p-t replies under base, decode-only and served.

## Tokens per second

- **Training:** 34,077, 23,312 and 33,734 context tok/s (seed 2 ran on a slower pod). (r-drop) trained at about 34,000 on a fast 5090, so no training cost of the cut is measured.
- **Decode:** 91–93 tok/s per process on the fast pods and 71–72 on the slow one (70.9 for the decode-only arm), with two scorings sharing each GPU. The cut costs at most 1 tok/s.
- **This is not the served speed.** The integer path with the row cut is not built.

## The registered outcome

**Served: reduces, content lost. Decode-only, which decides nothing, reads "improves".**

At all three seeds, on p-t, the served arm:
- **has fewer loops** than the base, beyond decision 4's band;
- **holds decision 4;**
- **keeps teacher-forced accuracy:** the rows' cell is inside the band;
- **loses the matched content at seed 2.** It has 0.350 of 197 matched prompts verbatim, against the base's 0.508: a gap of 0.157, against a tolerance of 0.107. At seeds 1 and 3 the content is kept.

The power floors are met: the base loops in 64, 60 and 76 replies, and there are 204, 197 and 208 matched prompts.

As registered, "reduces, content lost" changes no recipe: (r-cut) does not become the one model's recipe. The registered next work is the turn memory's reply table. Before it, by Casey's direction of 30 Oct, comes a decode unit that steps off a reply's own cycle.

## p-t, the deciding panel

The references' `v_ref` is 0.516 and their repeat rate 0.000. Each cell gives seeds 1, 2 and 3. "Matched" is the verbatim TRAIN share on the prompts the base and the served arm both complete (204, 197 and 208).

| Arm | Looping replies, of 256 | Complete | Matched verbatim | Repeated-4-gram rate | Mean length |
|---|---|---|---|---|---|
| base: (r-drop) | 64, 60, 76 | 212, 200, 212 | 0.471, 0.508, 0.404 | 0.522, 0.587, 0.522 | 21.4, 22.2, 24.0 |
| served: (r-cut), cut | 27, 17, 19 | 247, 253, 249 | 0.412, 0.350, 0.337 | 0.192, 0.103, 0.187 | 15.2, 11.2, 12.6 |
| decode-only: (r-drop), cut | 35, 38, 52 | 237, 244, 248 | 0.475, 0.503, 0.399 | 0.293, 0.241, 0.195 | 16.2, 15.2, 16.6 |
| uncut: (r-cut) | 62, 30, 45 | 205, 231, 216 | 0.412, 0.350, 0.308 | 0.544, 0.434, 0.530 | 22.0, 15.2, 18.9 |

**Paired against the base:**
- **Served:** 54, 55 and 67 replies stop looping, and 17, 12 and 10 start.
- **Decode-only:** 32, 27 and 38 stop, and 3, 5 and 14 start.

**The teacher-forced rows:** (r-cut) against (r-drop) on every p-t row (about 16,465 per seed) gives d = −62, +200 and −146, inside the band of 346 at every seed. On DEV, (r-cut) has 11,902, 11,979 and 11,943 nearest hits, against (r-drop)'s 11,841, 11,846 and 11,820.

**On DEV,** whose prompts are TRAIN-templated, the base loops in 46, 28 and 29 replies and the served arm in 16, 24 and 26. That is beyond the band at seed 1 only.

## What it shows

- **Training the cut in removes 58–75% of the base's loops on new phrasings,** against 26–51% for claude-215's decode-only cut on its own panel, p-r. It does so with no measured cost in teacher-forced accuracy.
- **But the trained model writes less like TRAIN's replies.** Replies are short (11–15 tokens), and the fixed reading shows salad and loops inside a word ("…inginging…") where the base had loops of words.
  - **One reading is that the loss is in the model, not only its decode.** The uncut arm's matched share (0.412, 0.350 and 0.308) is no higher than the served arm's.
  - **The uncut arm also loops less** than the base at seeds 2 and 3 (30 against 60, 45 against 76, both beyond the band). So part of the loop drop is in the trained model too.
  - It is one more arm at the same seeds, not a control for the cause.
- **One reading: held to the prompt, the reply learned to stop more than to answer.** Without its own earlier tokens to read, the trained model's reply leans on the prompt and the trunk alone. That is the role the turn memory's reply table, keyed by the reply's last id, was built to fill. It is not a tested cause.
- **The decode-only arm is the surprise, and it decides nothing.** Decoded with the cut, the sealed (r-drop) loses about a third to nearly a half of its loops (45%, 37% and 32%), with its matched content unchanged (0.475 against 0.471, 0.503 against 0.508, 0.399 against 0.404).
  - **Why it differs from claude-215:** claude-215's arm read "reduces, content lost". There, content was read on each arm's own completions, which claude-215's [REPORT](../../../loop/tasks/one-model-end-to-end/claude-215/REPORT.md) named as a possible selection effect; matched prompts remove it. This arm also holds an earlier reply in the prompt, as claude-215's did not.
  - **It is one panel.** A fresh panel would confirm it.

## What it unlocks

- **No recipe change.** (r-drop) stays the one model's recipe, and the served decode stays greedy as trained.
- **Next, by Casey's direction (#3407 6523338624):** a decode unit that steps off a reply's own cycle (the nearest token outside a repeated 4-gram), on a fresh panel, against greedy, on (r-drop). Its sheet can register the decode-only cut beside it, to test this panel's "improves".
- **Then the registered follow-up:** the turn memory's reply table.
