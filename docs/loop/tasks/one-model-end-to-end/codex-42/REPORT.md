# REPORT: one-model-end-to-end, codex-42 (the general-text corpus is real and built)

Lab: codex. The owner's goal is a general chat/agentic model. This unit builds the two assets the
project did not have: a general tokenizer and a general-text corpus in the trainer's own format.

## 1. The donor corpus, and where its replies actually are

`/mnt/plexpool/uor-r4-corpus/m2-donor-corpus-2026-11-16/` holds **692,364 prompts** in
`inputs/bank.jsonl.zst` (+491,578 in `bank-a`) — **prompts only, no replies**. The donor's replies
are in **`api-a/shard-*.jsonl.zst`**, 225 shards, 1.0 GB compressed, each row a conversation with
`turns: [{answer, reasoning}]`.

The first tokenization produced **0 rows** because the bank has no assistant turn to mask. The
shards are the corpus.

## 2. A 32,768-piece tokenizer on the donor's own text

`train-bpe` on 407 MB of the corpus' message text: **32,509 merges**, `blake3:d0edd92e…`, stored at
`/mnt/plexpool/uor-r4-corpus/tokenizer-32768-2026-11-17.json`. The project's pinned tokenizer is
4,096 pieces; 4k cannot carry general English.

## 3. The corpus, in the trainer's format

`prepare-chat-jsonl` (new, the JSONL twin of `prepare-chat-parquet`: same encoding, response mask,
drop rules and manifest) over all 225 shards with the 32k tokenizer:

| | |
|---|---|
| rows read | 457,601 |
| **rows used** | **457,021** |
| dropped | 580, all `oversized` (>8,192 tokens); 0 empty, 0 no-response, 0 special |
| **tokens** | **332,425,562** |
| assistant (masked) targets | **229,432,600** |
| files | `tokens.u16` 664,851,188 B, `response_mask.u8` 332,425,562 B |
| pins | tokens `507cf2717fdaa14d…`, mask `4865da8d32c5ea55…`, vocab 32,768 |

The donor's `reasoning` is kept only where an answer is missing, so this first corpus is plain chat.
332 M tokens is **43× the current 7.7 M**; the "1 B" figure belongs to a larger collection than the
457 k answered rows that exist on disk today.

## 4. What still blocks the model: the sheet is compiled in

`train-in-space` builds the model from constants, not flags:

```
SHEET_VOCAB = 4,096   SHEET_WIDTH = 256   SHEET_CONTEXT = 192   pattern "rrrrrr" = 6 layers
```

`StackConfig` itself already carries `vocab_size`, `width`, `heads`, `mlp_hidden`, `context` and
`pattern`, so the shape is available — but the *trainer* threads `SHEET_VOCAB`/`SHEET_CONTEXT`
through everything that touches the vocabulary or the window: the Hamming read tables
(`with_ask`/`with_hold`/`with_carry`), the DEV slot-rank table and its boundary, the token bound
checks, the window cutting, the eval batch and the checkpoint's own shape check. A 32 k run is
therefore a **new sheet** plus threading the corpus vocabulary through those sites — a multi-file
change, not a flag.

## 5. The model this makes possible

| | today | new sheet |
|---|---|---|
| vocabulary | 4,096 | **32,768** |
| width | 256 | **512** |
| layers | 6 | **12** |
| context | 192 | **512** |
| values | 1.11 M | **≈ 30 M** |
| corpus | 7.7 M tokens | **332 M tokens** |

At the measured 24.4 k tokens/s, one epoch is **≈ 3.8 h** and three epochs **≈ 11–13 h** on this
4090 — inside the project's own 20 h/epoch budget.

## 6. Unit order from here

1. **The sheet**: parameterise the vocabulary and the window, build the 30 M model, and prove a
   1,024-step smoke run on the real corpus with the real tokenizer.
2. **The run**: three epochs, dev readings on a held-out slice of the donor corpus.
3. **The chat panel**: held-out conversations, read as text — the first evaluation this project would
   have that means anything about general language.

The narrow-world `recall=off` run is a separate track (closed-world baseline) and is unaffected.
