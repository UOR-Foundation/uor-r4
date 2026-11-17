# ORDER: one-model-end-to-end, codex-41 (natural language is the goal)

Lab: codex. Follows codex-40, where the `System: Memory:` turn was found to be a crib the item had
omitted while the corpus always supplied it.

## The owner's goal, stated plainly

**Natural language.** The model talks; it answers a fact stated in the conversation because it read
the conversation, not because a system line handed it the answer. So:

1. **The crib goes.** `recall=off`: no `System: Memory:` turn. Verified on the generator: 0 recall
   lines, everything else identical (228,351 turns, 7.72 M tokens, same seed, same shares).
2. **The item is a conversation.** Facts in earlier user turns, the question in its own turn, no
   restatement — `corpus-shape/v1`, which now matches the training distribution for the first time.
3. **Replies are read as text.** The value and the attribute are checks; the sentence has to be
   well formed and stop. A bare value is acceptable only where a single word is justified (the
   owner's rule of 16 Nov); for these memory questions a sentence is the norm.
4. **`echo` is retired as a gate.** With facts stated earlier in the same conversation, a correct
   answer may share a short span with the prompt; the measure is the value plus the frame, read by
   eye on a sample and counted per stratum.

## The run

The pinned recipe (codex-34), same 60,000 conversations and seed, `recall=off`, 16,384 steps. The
only change from the rebalanced run is the removal of the crib.

## What is being read

| reading | what it means |
|---|---|
| dev writer top-1 / nearest | did removing the crib cost the trunk, or only the copy path |
| panel value-present per stratum | does the value come back at all from conversation |
| the reply text on 20 sampled rows | is it a sentence, the right attribute, and does it stop |
| turn-boundary behaviour | does it answer, or repeat the previous assistant turn |

**Cancel:** if the model again answers by repeating the previous assistant turn on every row, the
trunk is not carrying the question and the next unit is not another corpus balance — it is a read or
capacity question, and it should be measured with the read on/off rather than guessed.
