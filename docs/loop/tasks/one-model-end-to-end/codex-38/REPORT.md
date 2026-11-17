# REPORT: one-model-end-to-end, codex-38 (the first token is the writer's prior, and the corpus says why)

Lab: codex. Settles the fork in codex-37 with three host-only measurements. No training run.

## 1. Not the turn memory

`served-reply-panel` gains `--no-turn-memory 1` (`IntegerStackModel::drop_turn_memory`, the control
`uor-chat` already had), applied to the corpus-shape item:

| turn memory | value-present | echo | answered | replies |
|---|---|---|---|---|
| on | 0 / 256 | 256 | 0 | **256 × ` Got it.`** |
| dropped | 0 / 256 | 256 | 0 | **256 × ` Got it.`** |

Identical. The memory's reply table is **not** what supplies the acknowledgment. That leaves the
writer.

## 2. Not a corpus-shape artifact either

The corpus-shape prompt reproduces the corpus's own dialogue: assert → `Got it.` → question →
`. The model emits the acknowledgment in exactly the slot where the corpus puts it. It is not
confused by the prompt; it is answering with the most likely reply, and the most likely reply is not
the answer.

## 3. The corpus says why: the reply-opening distribution

First tokens of all 214,271 assistant responses in `mw-balp/train` (aligned to `tokens[32:]`, the
file's 32-token header):

| opener | share |
|---|---|
| ` Got` | **15.1 %** |
| ` It` | 9.0 % |
| ` I` | 8.7 % |
| ` Okay` | 8.0 % |
| ` A` | 6.7 % |
| ` No` | 5.5 % |
| ` That` | 4.7 % |
| ` The` | 4.1 % |
| **` You`** | **3.8 %** |

The corpus's assistant turns open with an acknowledgment or a conversational stem four times more
often than with the second-person answer frame. A greedy decoder that has learned the opening
distribution will say ` Got` — and it does, in every row, on every item shape, with or without the
memory.

Together with the rank reading (codex-37: 74.9 % of the answer's tokens are rank 1 once past the
opener; the opener is median rank 27), the mechanism is now complete:

> The model knows the value and the frame. The first token of the reply is decided by a prior the
> corpus set at 15.1 % against 3.8 %, and no read or memory supplies the answer opener.

## The next unit

**Raise the answer-turn share in the corpus**, then the pinned 16,384-step run, judged by `answered`
on the corpus-shape item (currently 0). The corpus already contains the answer frame
(`RELATIONS.answers`), so this is a balance change, not new content: more query→answer turns per
episode (or fewer acknowledgments per assertion) so the opener's prior stops favouring `Got it.`.

The reading to move is the opener distribution above and `answered`; the reading that must not fall
is the value-present count on the one-user-turn item (57) and the token loss.
