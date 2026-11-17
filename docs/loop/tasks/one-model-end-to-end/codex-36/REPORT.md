# REPORT: one-model-end-to-end, codex-36 (the panel scores echoing, not answering)

Lab: codex. Supersedes the capability reading in codex-32/33/34/35. Instrument only; no training.

## The finding

The panel's `value_present` asks whether the asked value appears in the reply. It does not ask whether
the model **answered**. Those are different, and on this artifact they are not the same event.

**Every one of the 57 value-present replies shares a four-token span with the prompt** (57 of 57; 0
with no shared span). The prompt says `My current job is basalt loggia. I currently live in porphyry
belvedere. Where do I currently live?` and the replies are that statement's frame with a pronoun
swapped. The value is present because the **prompt stated it and the model repeated it**.

## The direct probes

Served through `uor-chat` on the same artifact, one episode each, decoding the ids:

| prompt shape | reply |
|---|---|
| facts + question in one user turn (the panel's) | `My currently live in porphyry belvedere.` |
| facts in one turn, question in the next (no assistant turn between) | `Where do I currently live` |
| one fact, `Assistant: Got it.`, then the question (the corpus's own shape) | `Got it.` |
| one fact and the question in one turn | `My current job is basalt lo…` |

The corpus's answer frame is `You live in {v}.` / `You work as a {v}.` (`RELATIONS.answers`). **The
model never produces it.** It echoes the user's statement, echoes the question, or acknowledges.

## What stands and what does not

- **Stands:** the value appears, the right attribute of the two is chosen, the artifact is
  deterministic, integer, and byte-reproducible. 57 / 256 is a true count of "the asked value is in
  the reply".
- **Does not stand:** that any of those 57 is an answer. Under the owner's rule — a general model
  answers in sentences, bare only where justified — an echo of the user's own sentence is not an
  answer, and the panel cannot tell the difference.
- The earlier "template convention" reading (codex-32) and the "frame blend / corpus shape" reading
  (codex-33/34) were both symptoms of the same thing: the reply is the prompt's sentence, so its
  frame is the prompt's frame, and it blends with the prompt's other clauses.

## Why the measure missed it

The accepted answers are the bare values (`membership_only`), so any sentence containing the value
passes. A user turn that states the fact in the same frame an answer could take makes echoing
indistinguishable from answering. That is a test-validity defect: the falsifier was not inspected
against the test, and it scored the echo.

## The fix, in order

1. **The measure**: `value_present` must require the reply to be *the model's own* answer. The
   cheapest sound rule: the reply does not share a ≥4-token span with the prompt, and the reply is
   not the prompt's statement about the asked attribute. Both are computable from the panel's
   windows, which it already writes.
2. **The item**: state the facts in one turn and ask in another **with the corpus's own shape**
   (`User: fact`, `Assistant: ack`, `User: question`, `Assistant:`) — the shape the model was
   trained on, rather than a jammed single turn it never saw. On that shape today the model says
   `Got it.`, which is the honest baseline and the thing to move.
3. **Only then** a capability number, and only then the read/capacity work — those cannot be judged
   against a measure that a parrot passes.

## Checks

57 of 57 present replies share a four-token span with their prompt (computed from
`--windows-out` and the artifact's reply ids). Four `uor-chat` episodes on
`/tmp/opencode/armr-step002048.uorlut` (sha `5bb0f95d…`), greedy, 12–16 tokens, decoded by hand from
the tokenizer. No GPU, no training run.

## The measure, now in the tool

`served-reply-panel` reports three counts beside `value_present` (`codex-36`):

- `echo` — the reply shares a four-token span with its prompt (`ECHO_SPAN = 4`);
- `answered` — the value is present **and** the reply is not an echo.

Both artifacts, same panel, default rendering:

| artifact | value-present | echo | **answered** | of |
|---|---|---|---|---|
| small, step 2,048 (`5bb0f95d…`) | 57 | **254** | **0** | 256 |
| 8×, step 6,144 (`98e6e3a3…`) | 56 | **223** | **0** | 256 |

**No reply on either artifact is both value-correct and not a prompt echo.** The two non-echo replies
are degenerate loops (`I now live.`, `I now live now live now live…`). Spot checks confirm the flag:
`My current job is basalt loggia.` against the prompt's `My current job is basalt loggia.` is an echo,
as it should be.

So the honest baseline of the item is **0 answers**, and every earlier capability number on this
panel — 3, 19, 25, 56, 57 — counted the right value appearing in a repetition. The checkpoint
comparison (57 against 25) did measure something real, but it measured **echo fidelity**, not
answering.

This is the position the item's own discipline asks for: the falsifier was inspected against the
test, and it failed.
