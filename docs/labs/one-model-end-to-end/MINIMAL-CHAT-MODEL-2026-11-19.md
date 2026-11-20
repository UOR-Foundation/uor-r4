# The minimal chat model: one corpus, one session, one gate (19 Nov 2026)

**This supersedes `PLAN-2026-11-18.md` as the primary objective.** That plan's objective was a
general-text next-token model measured in bits. The product is a chat model, and the two are not the
same project. The language-modelling line continues only as an **instrument** (does the transport
extract information at scale), not as the thing being built.

## Why this document exists

`GO-FORWARD-2026-11-16.md` is a durable 13-item plan whose item 13 is *the run*: arm r's memory-line
recipe, 16,384 steps, ~1.4 h per seed, gated on the protocol panel. On 18 Nov a different plan
(`PLAN-2026-11-18`, M1–M6 / C1–C8) replaced its objective, and the run never happened:

- no task directory after 16 Nov corresponds to it (the newest are `codex-49..53`, all the other plan);
- no run directory exists for a memory-line or chat run;
- nothing in the record disproves it.

**It was abandoned, not refuted.** That is the failure this document is written to correct, in the
same way `GO-FORWARD` was written to correct the one before it.

## The honest position after today

Two lines have been running, and they answer different questions:

| | chat line (arm r) | general-text line |
|---|---|---|
| trunk | 1.11 M values | ~30 M values |
| data | memory-line dialogue curriculum | 385 M tokens of donor text |
| measures | answering about a fact in context | next-token on general text |
| state | copy recall 0.749–0.782; **0 complete replies** | top-1 17.5 % (ahead of a bigram), bits behind it |

The general-text line cannot become a chat model by more training: it is a next-token predictor with
no dialogue objective and no memory curriculum. **The chat line is the product.**

## What the detour bought, and it is portable

1. **The objective was broken on both lines.** `TOKEN_MARGIN` defaulted to 1.0 against a score that
   is a mean over 512 lanes and cannot express it. A reachable margin was worth **0.94 bits and +30 %
   nearest at equal cost** on the LM line, and the chat recipe trains with the same default.
2. **The writer's address was the readout defect**: 2.3 % → 15.8 % admission and 0.4 % → 11.9 % pick
   on one frozen trunk. The chat line serves through the same writer.
3. **A measurement top-1 cannot fool.** Bits on bit-identical rows with a count baseline; and the
   lesson that a metric a parrot passes (Decision 17's echo) is not a capability metric.

## What already exists (do not rebuild it)

Mechanisms in the trainer today, none of which need to be invented: FBS transport; quaternion
recurrence; Hamming reads **copy-latest, ask, place, copy-longest**; three-key turn memory; pair rows;
prompt register; opening read; address memory; vector reads; state-flock, prime-route and starling
reads; the centroid writer address; a reachable margin.

**The ask-head — the mechanism the first-reply-token trace names — is already implemented**
(`--hamming-ask 1 --hamming-carry-question H`) and **has never been in the recipe**, which uses the
place read instead. The two are exclusive.

## What is missing, in dependency order

1. **A dialogue-shaped memory corpus** with held-out values, paraphrases and distractors, where the
   answer is the value and not a repeat of the user's sentence. Every chat number so far counted
   echo fidelity: Decision 17 measured **0 answers and 254/256 replies sharing a 4-token span with
   their prompt**. This is the blocker, and it is data work, not model work.
2. **One composition**: arm r's recipe + the ask-head + `--token-margin 0.02` + the centroid address.
3. **The non-echo gate** on a held-out panel.
4. **The served artifact** carrying the read (the export path exists; it has been exercised once).

## The corpus spec (unit 0)

- Shape: `User: <fact>` / `Assistant: <ack>` / `User: <question>` / `Assistant: <value>`.
- Each fact has **≥ 10 paraphrased question forms** and 2–4 **distractor facts** in the same
  conversation, so answering requires picking the asked fact rather than the nearest one.
- A **held-out value set** never seen in TRAIN; the panel's facts and values are disjoint from it.
- Census before training, in the project's own way: TRAIN/held-out overlap, paraphrase count per
  fact, distractor count, and the share of replies that a prompt echo would pass (**must be 0**).
- Tooling that exists: `prepare-chat-jsonl` (JSONL `messages` → UORT), `heldout-panel`,
  `train-in-space --census 1`.

## The session (unit 1)

```
train-in-space --objective nearest --fast-rivals 1 \
  --hamming-ask 1 --hamming-carry-question 2 \
  --token-margin 0.02 --writer-address centroids \
  --batch 16 --context 256 --steps 16384 --lr 0.001 --lr-end 0 \
  --seed {1,2,3} --out runs/chat-min-s{seed}
```

Cost from the measured table in `GO-FORWARD` §6: **0.300 s/step on a 4090, ~1.4 h per seed**, ~4.5 h
for three. Compute is not the bottleneck.

## The gate (fixed before the numbers)

- **≥ 25 of 40 held-out memory items answered and non-echo**, at all three seeds.
  (The bar of 25/40 is the project's own, from the memory line; the non-echo requirement is
  Decision 17's correction.)
- **Right attribute ≥ 80 %** of the answered items.
- **Zero write-over errors of the "47 → 40" class** (a copied value overwritten by an unrelated one).
- Bits and top-1 are reported **beside** these, never instead of them.

## After the first model — the increment ladder

| # | unit | gate |
|---|---|---|
| 2 | slot / latest-wins arm if values are wrong | write-over errors → 0 |
| 3 | export and serve the **dev-best** checkpoint (Decision 13), not the last | read+answer parity on the artifact |
| 4 | pair rows, then 2× facts at the same recipe | the same 25/40 gate at double the facts |
| 5 | scale tokens at the same mixture (Decision 16: volume is not the lever at 1.11 M) | measured, then decide |

## What this refuses

- Another language-modelling experiment as the primary objective.
- Any capability number from a metric a prompt echo passes.
- Scoring the last checkpoint instead of the dev-best.
- Calling a mechanism "untested" when it is built and simply not in the recipe.
- Any matmul or softmax, at any stage, in any form.
