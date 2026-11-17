# ORDER: one-model-end-to-end, codex-34 (the data-scale arm, on a pinned recipe)

Lab: codex. Follows codex-33 (the dev-best checkpoint is 57/256; the recipe overfits from 0.54
epochs) and claude-152 ("13 times more training text, as mixed, hurt").

## Why this unit exists, and why it is not "more steps"

`codex-33` measured the run's own curve: dev loss bottoms at **0.54 epochs** (2.247 at step 2,048)
and rises after, while train loss falls. The corpus is **8,131,713 tokens** for a trunk of **1,112,576
values** (GO-FORWARD §6) — 7.3 tokens a value, where language models are usually trained at 20 or
more. The limit is data volume, not optimisation.

claude-152 already tried more text and it hurt, but the data was **13× larger *mixed* text, mostly
long general chat, unlike the test's dialogues**. So the lesson is about the *mixture*, not the
volume. This unit holds the mixture fixed and raises only the volume.

## 1. Pin the recipe (must pass before any training is spent)

The command line of the arm r copy run was never recorded. The run **does** print fingerprints
before training, and the run is deterministic (a restart reproduced steps 2,048 and 4,096 exactly).
So the recipe is recovered by reproduction, not by guessing: run `train-in-space` for a few steps
and compare these lines from `/tmp/opencode/armr-copy-s2.log`:

```
[in-space] usable TRAIN windows 60366 (batch 16, 16384 steps = 4.34 epochs), DEV windows 428
[memory] opening keys left to the copy read 6021 of 13584
[memory] key 8 ids; TRAIN openings 214271; table keys 7563
[memory] reply keys left to the copy read 186261 of 329340
[memory] reply table: TRAIN rows past their openings 2236785; keys at share 0.5 143079
[memory] TRAIN opening rows 214190, hits 163540
[memory] DEV opening rows 1226, hits 950
[memory] TRAIN reply rows past their openings 2233156, named 1393810
[memory] DEV reply rows past their openings 13061, named 7641
[in-space] step 0 ... nearest 5 (openings 3/1226, commonest 161) top-8 27 admitted 526 dev_loss 2.78726
```

They print in seconds, so a mismatch costs seconds, not a run. Then train to step 2,048 and require
the run's own readings exactly: `train_loss 0.97650`, `dev top-1 447/14287`, `nearest 11916
(openings 600/1226)`, `dev_loss 2.24677`.

**Done — pinned on the first attempt.** The recipe is documented in
`docs/model/STACK-CONTRACT.md:117`; run as

```text
train-in-space --data-root /tmp/opencode/next02-inputs \
  --train-split /tmp/opencode/next02-inputs/mw-balp/train \
  --train-sha256 1fc3f4ec…941f --train-mask-sha256 81aa5ed0…b0b7 \
  --out OUT --objective cell --retain-rates 1 --decay-mult 1 --lr 0.001 --lr-end 0 \
  --context 256 --width 256 --hamming-read 4 --batch 16 \
  --turn-memory 8 --turn-memory-reply 1 --turn-memory-dropout 500 \
  --turn-memory-leave-copied 1 --hamming-copy-latest 1 \
  --steps 16384 --eval-every 2048 --samples 6 --reply-tokens 48 --seed 2
```

**All nine `[memory]` fingerprints and step 0 match exactly** — 60,366 TRAIN windows, 428 DEV, 6,021
of 13,584 opening keys left, table keys 7,563, 186,261 of 329,340 reply keys left, 143,079 reply keys
at share 0.5, TRAIN openings 214,190/163,540, DEV 1,226/950, TRAIN reply rows 2,233,156/1,393,810,
DEV 13,061/7,641, and `dev_loss 2.78726` with `nearest 5 (openings 3/1226, commonest 161)`, `top-8
27`, `admitted 526`. The default TRAIN pin (`3dee858f…`) is the older corpus; the run's is
`1fc3f4ec…`, so `--train-split` plus both sha flags are required.

## 2. The 8× corpus (already generated)

```text
m-world-recall-corpus out=/tmp/opencode/data8/mw-balp tokenizer=/tmp/opencode/data8/tokenizer.json \
  world=v2 seed=13 conversations=480000 protocol=2 context=384 recall=reply \
  mqar_share=0.35 copy_share=0.10 relation_share=0.30 other_share=0.25 closed_share=0.20
```

Same generator, same seed, same mixture, same shares: **only the document count changes**, 60,000 to
480,000 (~64 M tokens, 7.9×). `ft/dev` and the tokenizer are copied from `next02-inputs`, so DEV is
the same pinned split. At `batch 16` this is 8,400 windows an epoch, so 16,384 steps is **0.49
epochs** — the same pass count as the 2,048-step point being compared against, with 8× the unique
tokens and the same wall clock.

## 3. The arm

Same recipe at `--steps 16384` against `--data-root /tmp/opencode/data8`, everything else pinned by
step 1. Then:

1. take the **dev-best** checkpoint (the rule from codex-33), not the last;
2. `fbs-snapshot` → `fbs-export --legacy-artifact` (the recipe that reproduces `f2104682…`);
3. `served-reply-panel` on the fresh panel, default rendering, `--reply-tokens 16`.

## 4. What counts as the answer

| reading | codex-33 baseline | this unit |
|---|---|---|
| dev loss at the trained window | 2.247 (step 2,048, 0.54 epochs) | lower, at ~0.49 epochs of 8× data |
| panel value-present | 57 / 256 | higher |
| right attribute | 57 / 57 | not lower |
| grammatical frame | 34 / 57 | not lower |

If dev loss is *not* lower at the same pass count with 8× the data, the trunk is not data-limited and
the next lever is capacity (codex-30 slice 3, the k-term arm) — that is the fork this unit settles.

## 5. Not in this unit

- The reply-frame grammar fix (23 of 57 are `I current job is` blends): a corpus-shape change, and it
  would confound the volume reading if mixed in here.
- The k-term arm, the `/17` contract, compression.
