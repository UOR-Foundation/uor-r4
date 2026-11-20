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

## Unit 0 — mine the capability from the real corpus (no GPU)

Not "build a corpus". **Measure what the corpus already contains**, and stop if it is not there:

1. Reconstruct conversations from the tokenised shards and the dialogue protocol's own separators.
2. Census, with the existing tooling (`train-in-space --census 1`, `heldout-panel`):
   - conversations with **more than one user turn** (the shape any "answer about something said
     earlier" capability needs);
   - conversations where a **later question refers to an earlier stated span** (the memory shape);
   - the distribution of **reply openings** and reply lengths (is there one dominant frame?);
   - **held-out rows that repeat a TRAIN reply** (the echo risk, measured before training).
3. **Gate on the census, before any run:** if the corpus holds ≥ 2,000 multi-turn conversations and
   ≥ 500 with a later question about an earlier span, the capability can be measured and trained. If
   it holds far fewer, **say so and stop** — the finding is that the corpus lacks the capability, and
   the response is a real-data source that has it (e.g. instruction/QA data), not a fixture we write.

This unit costs no GPU and decides whether the rest is possible. That is the measurement the last
three weeks lacked.

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

## Unit 2 — the gate (content-grounded, frame-free)

All three are measured on **held-out real data**, never on a shape we wrote:

- **Non-echo**: the reply must not share a 4-token span with the prompt (Decision 17's test), on
  held-out real prompts.
- **Novelty**: reply 4-gram novelty against TRAIN (the `m2-evidence` instrument already reports it),
  so a memorised TRAIN reply is visible rather than credited.
- **Exact answers where the data has them**: the 196-item panel's exact-answer scoring, reported as
  an oracle grade beside the two above — not as the chat claim, because a 1.11 M-value model will not
  solve numina, and a gate the model cannot pass at all measures nothing.

Bits and top-1 are reported **beside** these, never instead of them.

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
- Any matmul or softmax, at any stage, in any form.
