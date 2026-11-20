# The minimal chat model: real corpus, real gate (19 Nov 2026)

**This supersedes `PLAN-2026-11-18.md` as the primary objective**, and it supersedes the first version
of this document, which specified an authored dialogue shape. That version was wrong for a reason
already in the record, and the owner named it: Decision 15 in `GO-FORWARD-2026-11-16` says the reply
defect is *"corpus shape (one frame per intent)"*, and Decision 17 says a prompt echo passed as an
answer. **Authoring a `User:` / `Assistant:` frame is that same defect with new words.** It is
refused below, in writing, so it is not proposed again.

## The rule that follows from that history

> **No authored frames, templates or frozen prompt shapes, in the corpus or in the gate.**
> The corpus is dialogue that already exists. The capability is *mined* from it. If the capability is
> not in the data, that is a finding, not a reason to manufacture a fixture for it.

This is the specific failure the owner describes: a test tied to a hand-written structure passes only
that structure, then breaks when anything changes and rules out models that were fine.

## What we already have (nothing to invent)

- **The corpus is real dialogue, tokenised, on the pool**: `tokens-8k-clean-2026-11-17`, **385,284,895
  tokens**, 223 donor shards, with a **response mask** marking the reply positions and a sealed
  manifest (`f2437189…`/`452941db…`). It came from real donor prompts and replies through
  `prepare-chat-jsonl`; there is no authored shape in it.
- **The gate content is real too**: `instr-panel-2026-11-17.json`, **196 items** with exact `expected`
  answers (132 `boxed`, 64 `number`), drawn from numina — content-grounded, frame-free, frozen with
  its census recorded before any score.
- **The mechanisms** are all built: FBS transport, quaternion recurrence, Hamming reads (copy-latest,
  **ask**, place, copy-longest), three-key turn memory, pair rows, prompt register, opening read,
  address memory, vector reads, state-flock / prime-route / starling reads, the centroid writer
  address, and a reachable margin.
- **The three fixes from the 18–19 Nov detour**, all portable to this line: a reachable token margin
  (**0.94 bits, +30 % nearest at equal cost**), the centroid writer address (**2.3 % → 15.8 %**
  admission, **0.4 % → 11.9 %** pick on a frozen trunk), and a measurement a parrot cannot pass.

## Where the chat question actually stands

The chat line's recorded blockers are **not** "the corpus is synthetic":

- copy recall **0.749 / 0.759 / 0.782** at three seeds, with the copy read and a memory line;
- **0 complete replies**, because the first reply token takes the *format* from the copy heads rather
  than the asked value (the trace in `GO-FORWARD` §2c.17: the value arrives four rows later);
- write-overs of the "47 → 40" class when a copied value is later overwritten;
- **0 answers, 254/256 replies sharing a 4-token span with the prompt** once echo is measured.

So the work is: the **ask-head in the recipe** (built, never used), the **margin**, the **address**,
and a gate that cannot be passed by echoing.

## The corpus is a distillation, so the teacher is the gate

The corpus is **1 B tokens distilled from DeepSeek**: prompts with the teacher's replies, tokenised to
**385,284,895 tokens** at the 8k vocabulary (`tokens-8k-clean-2026-11-17`, pins `f2437189…` /
`452941db…`), with a 32k tokeniser beside it (`tokenizer-32768-2026-11-17.json`). The `response_mask`
marks exactly the teacher's reply tokens.

Two consequences, and they remove the whole class of problem the owner objected to:

1. **Nothing is authored.** The training targets are the teacher's own replies. There is no frame, no
   template and no fixture anywhere in this line, and there never needs to be.
2. **Every held-out row has a reference.** A held-out conversation already contains the reply the
   model should have produced. The gate does not need a panel we wrote — it compares against the
   teacher, on rows the model never saw.

So "build a corpus" and "mine the corpus" both disappear. The corpus exists, it is a teacher
distillation, and the objective is to **imitate the teacher's replies on prompts it has not seen**.

## Unit 0 — split the distillation, no GPU

- **Held-out split by conversation**, not by row: no conversation contributes to both TRAIN and the
  gate. This is the one constraint the corpus requires, and the existing manifests already name the
  shards.
- **Census, recorded before any score**: held-out reply rows that also appear in TRAIN (the echo risk),
  the distribution of reply openings, and the share of held-out replies that a prompt echo would pass.
  This is measurement of the split, not a licence to reshape the data.
- No capability is "mined" and nothing is generated: if the teacher's replies contain the behaviour,
  the distillation can learn it; if they do not, the model will not, and that is a corpus fact worth
  knowing — not a reason to write a fixture.

## Unit 1 — the session

The proven recipe, on that corpus, with the three fixes:

```
train-in-space --objective nearest --fast-rivals 1 \
  --hamming-ask 1 --hamming-carry-question 2 \
  --token-margin 0.02 --writer-address centroids \
  --batch 16 --context 256 --steps 16384 --lr 0.001 --lr-end 0 \
  --seed {1,2,3} --out runs/chat-min-s{seed}
```

Measured cost: **0.300 s/step on a 4090, ~1.4 h per seed**, ~4.5 h for three (`GO-FORWARD` §6).
Compute is not the bottleneck. **Score the dev-best checkpoint, not the last** (Decision 13).

## Unit 2 — the gate: agreement with the teacher, on held-out conversations

Three measures, all against the teacher's own replies on rows the model never saw. No authored
content, no frame:

- **Non-echo**: the generated reply must not share a 4-token span with the prompt (Decision 17's
  test). This is the measure that every earlier chat number failed.
- **Teacher agreement**: on held-out prompts, how often the model's reply matches the teacher's —
  exact for short answers, and token-level F1 / longest common subsequence for longer ones. This is
  the distillation objective made measurable, and it survives any change to the corpus because the
  reference is in the data.
- **Reply novelty**: reply 4-gram novelty against TRAIN (`m2-evidence` already reports it), so a
  memorised TRAIN reply is visible rather than credited.

The 196-item numina panel stays as an **oracle grade beside** these, not as the chat claim: a 1.11 M
to 30 M-value model will not solve it, and a gate nothing can pass measures nothing.

Bits and top-1 are reported beside the three, never instead of them.

## Then increment

| # | unit | decision it changes |
|---|---|---|
| 3 | slot / latest-wins arm if values are wrong | the "47 → 40" class |
| 4 | export and serve the dev-best checkpoint | whether the read survives integer serving |
| 5 | 2× facts / 2× tokens at the same mixture (Decision 16) | whether volume is a lever at this size |

## What this refuses

- **Authored frames, templates, or frozen prompt shapes** in the corpus or the gate.
- Another language-modelling experiment as the primary objective.
- Any capability number from a metric a prompt echo passes.
- Scoring the last checkpoint instead of the dev-best.
- Calling a built mechanism "untested" when it is simply not in the recipe.
- Reshaping or generating corpus content to make a gate pass: the teacher's replies are the target.
- Any matmul or softmax, at any stage, in any form.
