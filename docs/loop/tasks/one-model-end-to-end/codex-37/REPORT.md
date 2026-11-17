# REPORT: one-model-end-to-end, codex-37 (the corpus-shape item, and where the answer dies)

Lab: codex. Executes steps 1 and 2 of codex-36's order. No training run.

## Step 1: the item in the corpus's own dialogue shape

`served-reply-panel --rendering corpus-shape/v1` (new): the facts in one user turn, the assistant's
acknowledgment, then the question in its own turn — the shape the corpus writes and the only shape
the model was trained on:

```
<|bos|>User: My current job is basalt loggia. I currently live in porphyry belvedere.
Assistant: Got it.<|eos|>
User: What is my current job?
Assistant:
```

| item shape | value-present | echo | answered |
|---|---|---|---|
| `one-user-turn/v1` (before) | 57 / 256 | 254 | **0** |
| `per-turn/v1` | 51 / 256 | — | 0 |
| **`corpus-shape/v1`** | **0 / 256** | 256 | **0** |

**All 256 replies are the same two words: ` Got it.`** The value-present count falls to zero because
nothing of the value is reproduced at all. The prompt is well formed (checked by decoding the
served ids), so this is the model, not the renderer: **in the shape it was trained on, it answers
nothing.**

## Step 2: where it dies

Teacher-forced ranks through the CPU port, same corpus-shape prompts. Two references: the bare value
(the label's own accepted answer) and the corpus's answer frame.

| reference | first target top-1 / top-8 / top-32 | first-target median rank | all targets rank-1 |
|---|---|---|---|
| bare value (`basalt loggia.`) | 0 / 0 / 0 | 3,112 (min 156) | **2,405 / 2,832 = 84.9 %** |
| answer frame (`You work as a …` / `You live in …`) | 0 / 14 / 161 | **27** (min 7) | **2,792 / 3,728 = 74.9 %** |

Two things follow.

1. **The knowledge is there.** Once the decoder is pushed past the first token, 75–85 % of the
   answer's tokens are rank 1. The value and the frame are both in the model.
2. **The first decision is what fails.** The opening word of the answer is **never rank 1** — best
   rank 7, median 27, top-32 for 63 % of rows — while `Got it.` is rank 1 at the assistant position
   in every greedy reply. The model has the sentence and cannot start it.

This is the same defect the earlier work named "the first token is the writer's prior" (codex-32's
bare-value episode), now measured on the real answer frame and the item's own shape: it is not a
memory failure, not a read-reach failure, and not a data-volume failure. The reply is decided at one
token.

## The next unit

The first token, and only the first token. Two candidate causes, both measurable host-only:

- **the turn memory's reply table**: the ack follows the assertion in TRAIN, so a key on the
  opening may return `Got it.` for the fact. The table's size (143,079 reply keys) and the leave
  rule make this plausible, and it is checkable from the artifact's memory tables.
- **the writer's own prior**: `Got it.` is the commonest assistant turn after a user assertion.

If the memory supplies it, the fix is the read (a key/leave rule, or the answer turn's key); if the
prior does, the fix is training weight on the answer turns. The diagnostic that separates them is
the read-panel's own pick and margin on these 256 rows at the assistant position — no GPU.

Then, and only then, a training unit that targets the first token, judged by `answered` on this item.
