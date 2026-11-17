# REPORT: one-model-end-to-end, codex-40 (the item was missing the corpus's Memory turn)

Lab: codex. Corrects codex-36/37/38/39. The model was asked a question it was never trained on.

## The finding

Both training corpora (`mw-balp` and the rebalanced one) were generated with **`recall=reply`**, which
puts the answer into a **`System: Memory: …` turn between the question and the reply** — the corpus
says so in its own manifest (`one system turn per MQAR and relation query, "Memory: {reply}"`). The
served shape is:

```
User: Call me Meerkveend.
Assistant: Nice to meet you, Meerkveend!
User: What is my name?
System: Memory: You're Meerkveend.
Assistant: You're Meerkveend.
```

`corpus-shape/v1` — the item built in codex-37 — omitted that turn. On the same artifact, same words:

| prompt | reply |
|---|---|
| facts, ack, question, **no Memory line** | ` Got it.` |
| … with `System: Memory: You're Meerkveend.` | ` Meerkveend.` |
| … with `System: Memory: You live in Paris.` | ` mean can welcome's, you live in Paris.` |

So the model **does** answer when the prompt has the shape it was trained on. `answered = 0` is an
**item defect**, and every reading built on it (codex-36's echo finding, codex-37's "dies at the
first token", codex-38's "writer's prior", codex-39's cancel) was measuring a request the model had
never seen: recall from context with no Memory turn.

## What still stands

- The measurements are true of what they measured: with no Memory turn the reply is ` Got it.`
  (234–256 of 256 rows), and it follows the ack word in the prompt (`Okay, noted.` → ` Okay, I…`).
- Turning the turn memory off and the read off changes nothing — so the reply at that position is the
  trunk's own choice, not a read's.
- The teacher-forced ranks (74.9 % of answer tokens rank 1 past the opener, opener median rank 27)
  were on the no-Memory prompt, so they describe the wrong request too.
- **The `echo` measure is wrong for this protocol.** With the Memory line the answer is in the prompt
  by construction, so a correct reply is necessarily a 4-token span of the prompt. `echo` must
  exclude the `System: Memory:` turn, or be dropped in favour of "contains the value and is the
  answer frame".

## A second, real defect it exposed

With the panel's novel two-word values (`basalt loggia`, `porphyry belvedere`) the Memory line still
fails: the reply is ` Memory: Memory: Memory:`. The model copies values it has seen; a **novel
two-word value** in the Memory turn is beyond it at this size. That is a genuine capability limit,
separate from the item.

## The fork

1. **The item, matching the trained protocol** — include `System: Memory: <answer>` and measure
   whether the reply reproduces the value in the answer frame. That is what `recall=reply` trains, and
   it is honest as long as the write-up says the answer is carried in the prompt.
2. **A `recall=off` run** — no Memory turn, so the model must recall from the conversation. The
   generator supports it (verified: `recall=off` writes 0 recall lines), the pinned recipe is known,
   and the panel item then needs no change. This is the only route to a model that answers a fact
   **stated earlier in the prompt** rather than restated beside the question.

The owner's goal — "answers a fact stated in the prompt" — is (2). (1) measures the copy path the
current model was built for.
