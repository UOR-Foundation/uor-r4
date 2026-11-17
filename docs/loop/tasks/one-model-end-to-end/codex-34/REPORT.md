# REPORT: one-model-end-to-end, codex-34 (the data-scale arm)

Lab: codex. Order: [ORDER.md](ORDER.md). Follows codex-33 (dev-best checkpoint 57/256).

## 1. The recipe was pinned exactly

`STACK-CONTRACT.md:117` documents the arm r copy line; run as written (plus `--context 256` and the
current corpus pins) it reproduces **all nine `[memory]` fingerprints and step 0** bit for bit:
60,366 TRAIN windows, 428 DEV, 6,021/13,584 opening keys left, 7,563 table keys, 186,261/329,340
reply keys left, 143,079 at share 0.5, TRAIN openings 214,190/163,540, DEV 1,226/950, TRAIN reply
rows 2,233,156/1,393,810, DEV 13,061/7,641, and `dev_loss 2.78726` with `nearest 5 (openings
3/1226, commonest 161)`, `top-8 27`, `admitted 526`. The trainer's default TRAIN pin is the older
`3dee858f…`, so `--train-split` and both sha flags are required. **Every run from here is
reproducible; the historical one was not.**

## 2. The 8× corpus is new data, not the same templates redrawn

Same generator, seed 13, and shares; only `conversations` changes, 60,000 → 480,000.

| | original | 8× |
|---|---|---|
| tokens | 8,131,713 | **64,923,870** (7.98×) |
| episodes Mqar / Copy / Relation / Other | 20,981 / 5,905 / 18,125 / 14,989 | 167,740 / 48,259 / 143,916 / 120,085 (7.94–8.17×) |
| distinct responses | 117,370 | **703,473** (6.0×) |
| of the 8× distinct set, absent from the original | — | **94.9 %** |

So this is not claude-152's failure: that was 13× *more mixed text* (long general chat). Here the
mixture, seed and shares are unchanged.

## 3. The result: the data lever does not move the panel

Trained at the pinned recipe, 16,384 steps = **0.54 epochs** of the 8× corpus — the same pass count
as the 2,048-step baseline point — for the same wall clock (2,743 s against 2,747 s).

| checkpoint | epochs | dev loss (token + address) | dev top-1 | value-present | length2 | length4 | length8 | reassert | update |
|---|---|---|---|---|---|---|---|---|---|
| small, step 2,048 | 0.54 | **2.2468** (0.9312 + 1.3156) | 447 | **57** | 27 | 12 | 1 | 16 | 1 |
| 8×, step 2,048 | 0.07 | 2.4458 (0.9237 + 1.5222) | 363 | 23 | 13 | 3 | 0 | 7 | 0 |
| **8×, step 6,144** | 0.20 | 2.4958 (0.7197 + 1.7762) | **475** | **56** | **33** | 6 | 0 | 17 | 0 |
| 8×, step 16,384 | 0.54 | 2.5253 (0.5715 + 1.9539) | 450 | — | — | — | — | — | — |

- **Value-present is flat: 56 against 57**, at 3× the optimisation steps and 4.3× the tokens seen —
  and 8× the unique data. The frame reading is flat too: **31 of 56** right-attribute and
  grammatical, against **34 of 57**.
- **The decomposition is the interesting part.** Token loss improves a lot with more data (0.931 →
  0.571), while the **address term worsens almost as much (1.316 → 1.954)**, so their sum rises. The
  panel tracks the sum, not the token part.
- `length2` reaches **33 / 64**, the best this stratum has scored, while `length4` falls 12 → 6 and
  `update` goes 1 → 0. The model gets better at the simplest copy and no better at composing.
- The reply-frame blends (`I current job is`, `My currently live in`) **do not improve with
  volume**, because the corpus's phrasing mix is unchanged by construction: more of the same mixed
  frames is more blends.

## 4. What the fork says

The order pre-registered: *if dev loss is not lower at the same pass count with 8× the data, the
trunk is not data-limited.* Dev loss is **not** lower (2.5253 against 2.2468), so:

1. **Data volume is not the lever at this scale.** 8× is measured, not argued.
2. The reply-frame defect is **corpus shape, not volume**: one frame per intent would fix what 8×
   more of the same frames cannot. That is a corpus unit with no GPU spend, and it targets 23 of the
   57 successes directly.
3. The **address term rising with the corpus** is a lead, not a conclusion: the read's tables grow
   with the data (143,079 → 541,807 reply keys), so more data adds distractors to the exact-key
   memory. That is a property of the read, and it is testable on artifacts we already have.
4. Capacity (the k-term arm, codex-30 slice 3) remains the untested lever for **multi-token
   composition**, which is exactly where this run lost ground (`length4` 12 → 6). It stays on the
   list, behind the cheaper frame fix.

## Checks

Pinned recipe: nine fingerprints and step 0 matched exactly. Two snapshots, two artifacts, two
panels on the 8× run (23 and 56 of 256), one panel on the baseline (57, deterministic). Dev readings
are the run's own `progress.jsonl`. Corpus novelty and mixture read from both manifests and the token
files directly.

## Correction (codex-38): the novelty figures were read 32 tokens out of alignment

`tokens.u16` opens with a 32-token header, so the response mask aligns with `tokens[32:]`, not with
`tokens[0:]`. Recomputing every figure in §2 on the aligned stream:

| | original | 8× |
|---|---|---|
| assistant responses | 214,271 | 1,717,821 |
| distinct responses | **61,766** (28.8 %) | **450,501** (26.2 %) |
| of the 8× distinct set, absent from the original | — | **442,542 = 98.2 %** |
| repeats inside the 8× corpus | — | 1,267,320 = 73.8 % |

The conclusion stands and is stronger: 8× the volume carries **7.3× the distinct replies**, 98.2 %
of them new. The earlier 117,370 / 703,473 / 94.9 % were the same measurement shifted, and are
superseded by these. The 73.8 % internal repeat rate is the generator's finite template space, and
it is why the distinct count grows slightly sublinearly.
