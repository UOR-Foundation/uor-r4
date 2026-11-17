# REPORT: one-model-end-to-end, codex-35 (where the reply frame comes from)

Lab: codex. Corrects the frame reading in codex-32/33/34 before a corpus change was built on it.

## The recommendation this unit stopped

Codex-34 concluded that 23 of 57 replies blend the corpus's reply phrasings (`I current job is`,
`My currently live in`) and recommended **one frame per intent in the corpus**. That was checked
against the test before being built, and it is wrong.

## The prompt already holds the frame

The panel's user turn, decoded from the served ids:

```
User: My current job is basalt loggia. I currently live in porphyry belvedere.
      Where do I currently live?
Assistant: My currently live in porphyry belvedere
```

The reply is the **prompt's own sentence frame** with a pronoun substituted. And `current`,
`currently` are **not tokens in the vocabulary at all** — the reply spells `Ġc ur re nt ly` from five
pieces, so the model is not reproducing a corpus reply template: it is reproducing the *user's*
sentence.

Of the 57 value-present replies, **21 are verbatim spans of the prompt** and 36 are recombinations
of it; none uses the corpus's second-person answer frame (`You live in X.`, `You work as a X.`).

## Not the rendering

The same artifact under `per-turn/v1` (facts and question in separate turns):

| rendering | value-present | what the replies do |
|---|---|---|
| `one-user-turn/v1` (default) | 57 / 256 | mirror the prompt's assertion, first person included |
| `per-turn/v1` | 51 / 256 | echo the **question** (`Where do I currently live`) or the assertion |

So the blend is present in both renderings: it is a property of the **prompt's wording**, not of how
turns are joined.

## What this means

1. **The frame is not a corpus-shape defect.** A corpus with one frame per intent would not change
   these replies, because the words come from the prompt. Recommendation 1 of codex-34 is withdrawn
   before it spent a run.
2. The defect is a **perspective error fed by the copy read**: the read is trained to reproduce the
   user's text, and it reproduces the user's *sentence* rather than just the *value*. The corpus's
   own answer frame is second person and is not being used.
3. The item's value-present measure still discriminates: both facts are in the prompt, and the
   accepted answer is the *asked* attribute's value, so echoing the wrong clause does not score. The
   57 stand as "the right fact was chosen".
4. **The panel's item format is what makes the frame ambiguous**: the user turn states the fact in
   the same frame an answer could take. A variant that states facts in one turn and asks in another
   without restating them is what separates "answered" from "echoed" — and it is cheap, since it is
   a panel, not a run.

## The honest next unit

Not a corpus frame change. Either

- **a panel unit** with a question turn that does not restate the fact, to measure answering as
  distinct from echoing (no GPU, uses artifacts on disk); or
- **a read unit** that makes the copy contribute the value and not the sentence frame (the same
  families of defect as "the first token is the writer's prior", codex-32).

The first is the instrument, the second is the mechanism. The instrument should come first, because
every number in codex-32/33/34 is read through it.
