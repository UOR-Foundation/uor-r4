# ORDER: one-model-end-to-end, codex-39 (raise the answer-turn share, then the pinned run)

Lab: codex. Follows codex-38 (the reply opener is the writer's prior: `ĠG` 15.1 % against
`ĠYou`+`ĠYour` 7.4 %, and the memory is ruled out).

## What changes

`milestone_world_v2::relation` ends every episode with **one** query→answer turn after three
acknowledgments (the assertion, the update, the companion). The change asks **the other stated
relation too**, so the same episode carries a second query→answer turn:

```
User: My current job is basalt loggia.
Assistant: Got it.
User: My home is in porphyry belvedere.
Assistant: Got it, you live in porphyry belvedere.
User: What is my current job?
Assistant: You work as a basalt loggia.        <- answer 1
User: Where do I live?
Assistant: You live in porphyry belvedere.     <- answer 2
```

It is a balance change, not new content: both answer frames already exist in `RELATIONS.answers`, and
the second question asks exactly the discrimination the item tests (two facts stated, one asked).
The abstain branch still asks one absent relation and gains no second question.

Both pinned stream digests (`STREAM_DIGEST`, `CHAIN_DIGEST`) move, deliberately, with the reason
recorded on the constants.

## Measured on the corpus before any run

| | before | after |
|---|---|---|
| assistant responses | 214,271 | 228,351 |
| openers `ĠG` | 15.1 % | 14.0 % |
| openers `ĠYou` + `ĠYour` | **7.4 %** | **11.6 %** |
| `ĠG` : answer-frame ratio | 2.04 : 1 | **1.20 : 1** |

## The run and the reading

The pinned recipe (codex-34: `--objective cell --retain-rates 1 --decay-mult 1 --lr 0.001
--lr-end 0 --context 256 --width 256 --hamming-read 4 --batch 16 --turn-memory 8
--turn-memory-reply 1 --turn-memory-dropout 500 --turn-memory-leave-copied 1
--hamming-copy-latest 1 --steps 16384 --eval-every 2048 --seed 2`), same 60,000 conversations and
the same seed, on the rebalanced corpus.

Then: dev-best checkpoint (decision 13), `fbs-snapshot` → `fbs-export --legacy-artifact`, and the
panel on **three items**:

| item | reading that must move | reading that must not fall |
|---|---|---|
| `corpus-shape/v1` | **`answered` (0 → > 0)** | — |
| `corpus-shape/v1` | reply openers other than ` Got it.` | — |
| `one-user-turn/v1` | — | value-present (57) |
| run's own dev | — | token loss (0.931 at the 0.54-epoch point) |

**Cancel:** if `answered` is still 0 and the openers are still all ` Got it.`, the balance change at
1.20 : 1 is not enough, and the next unit is a stronger rebalance (fewer acknowledgments per
assertion, or two answers per acknowledgment) rather than another read or capacity experiment.
