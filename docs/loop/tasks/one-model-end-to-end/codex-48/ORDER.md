# ORDER: one-model-end-to-end, codex-48 (an instruction panel graded by oracles)

Lab: codex. The test the 3-epoch run needs, so the run lands with its test ready.

## Why the existing readings cannot be the test

- The narrow value-recall panel measured **memory** (does the reply contain a stated value), not
  instruction following, and it scored echoing until the item was fixed.
- The donor DEV token term measures **next-token prediction on held-out chat**. That is the
  precondition — has the trunk learned language at all — not the capability.
- The capability is **instruction following**: a prompt asks for a transformation, and the reply
  either satisfies it or does not.

## The panel

**Items** are held-out donor conversations whose correct answer is checkable without a model. The
donor bank labels its sources, so the oracle is available:

| source | oracle | example |
|---|---|---|
| `numina`, `openthoughts` | the answer's final `\boxed{…}` (or the last explicit result) | `\boxed{8\text{ years}}` → `8 years` |
| `gsm8k` | the final number the answer states | `23 cars … 1 car per day` → `1` |
| `wildchat` copy prompts | the exact string the prompt asks to repeat | `Say exactly this: …` |
| all sources | a refusal where the prompt's answer is impossible | "Not enough information" |

**Split.** Shards 3–4 are held out of training. The DEV curve uses the first 200 k tokens of them
(`--dev-tokens`); the panel draws from the **rest**, so no panel item is ever in TRAIN or in DEV.

**Size.** 200 items, sampled by a hash of the item id so the draw is deterministic and reproducible:
40 arithmetic with a boxed answer, 40 gsm8k-style numeric, 60 copy-exact, 20 format transforms
(lowercase, one line, a list), 20 refusals, 20 free-form chat read by eye.

**Grading.** Deterministic only (D28 forbids neural graders):
- arithmetic/numeric: the reply contains the expected string after whitespace/case normalisation,
  within `--reply-tokens 96`;
- copy-exact: the reply equals the target exactly, trimmed;
- format: the reply satisfies a stated regex;
- refusal: the reply names the missing information and states no number that is not in the prompt.

**Reported**: per-stratum counts, the reply text of 20 sampled items, tokens/s, and the same counts
for two controls — the empty reply and the copy-the-prompt reply — because a bar without a control is
not a reading (L-010).

## The bar

Set before the run, as a **count**, never a composite:

1. arithmetic + numeric: the model beats the copy-the-prompt control by at least the paired band
   `2·sqrt(b + c)` over the 80 items, on **both** of two seeds;
2. copy-exact: at least 50 % of 60, again above the copy control;
3. format and refusal: reported, not barred, until the first reading exists;
4. free-form chat: read, never counted as a pass.

A pass on (1) and (2) is the first evidence of instruction following the project would have. A fail
on (1) with a falling DEV token term says the trunk learns text but not the requested transformation
— which points at the writer's admission, not at more data.

## Frozen (2026-11-17)

`/mnt/plexpool/uor-r4-corpus/instr-panel-2026-11-17.json`: **196 items — 132 `boxed`, 64 `number`**,
drawn from the held-out shards 3–4, ordered by the sha256 of the item id. Census, taken before any
model was scored: 4,096 rows seen, **200 with a checkable oracle**, 2,936 without (free chat), 0
without an answer.

**The copy/format/refusal strata are empty** — the donor's wildchat prompts do not phrase a copy
instruction in a way this extractor recognises, so the bar's copy clause (50 % of 60) cannot be met
from this panel. The first bar is therefore the arithmetic one, on the 196 items that exist, and the
copy stratum needs its own extractor or a synthetic set; it is not silently dropped.

## Build order

1. `instr-panel`: read a shard range, filter and sample by hash, extract the oracle, encode with the
   pinned tokenizer, serve a checkpoint, grade, write `instr-panel-<artifact>.json`.
2. Freeze the panel (ids, oracles, controls) and record its census before any run is scored.
3. Score the 3-epoch checkpoint and the 1-epoch checkpoint on the same panel.

## Out of scope

Neural graders; any item whose answer cannot be checked exactly; the narrow value panel; a bar set
after seeing the result.
