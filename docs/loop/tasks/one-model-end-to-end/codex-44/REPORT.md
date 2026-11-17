# REPORT: one-model-end-to-end, codex-44 (the writer chunks by rows; a 30 M model trains)

Lab: codex. Follows codex-43 (the shape, and the 1 GiB candidate bound).

## What changed

`NativeVectorWriter::candidates` is now **row-chunked**. `max_query_rows` divides the 1 GiB
candidate-vector budget (`MAX_CANDIDATE_VECTOR_ELEMENTS`, 256 M f32) by one row's
`kept_cells × slots × width`, clamped to `MAX_QUERY_ROWS`. Below that it is the old function
(`candidates_unchunked`), so at the sheet's own shape — 8 cells × 16 slots × 256 = 32,768 elements a
row, 8,192 rows — **no chunking happens and every existing call is unchanged**. Above it, the query
is narrowed into row chunks and the parts are concatenated, so a 14,287-row panel scores at width
512 / 8,192 vocabulary, where one call would need 1.87 GiB.

## What ran

A **30 M model** (width 512, 12 layers, vocabulary 8,192, context 512) on the real donor corpus
(5,738 conversations, 4,894,652 tokens, 11,237 windows):

```
[in-space] usable TRAIN windows 11237 (batch 8, 8 steps = 0.01 epochs), DEV windows 428
step 0  dev top-1 0/14287 nearest 4  dev_loss 2.83205 (token 1.00191 address 1.83014)
step 8  dev top-1 0/14287 nearest 2  dev_loss 2.81000 (token 1.00189 address 1.80811)  train 7s
```

It builds, cuts, evaluates and steps. **7 s for 8 steps at batch 8 × context 512 ≈ 4,500 tokens/s**,
so one 332 M-token epoch is **≈ 20 h** here — the project's own 20 h/epoch figure.

## What this does not yet mean

The DEV numbers above are deliberately meaningless: `load_dev_split` reads the **narrow pinned
`ft/dev`** (its own sha256 pins, token count, eligible-target count and `SHEET_VOCAB`), which was cut
with the 4,096 tokenizer on the synthetic world. A donor run needs a **donor DEV slice with its own
pins** (or an unpinned `--dev DIR`); until then no number from this shape says anything about general
text.

## Checks

`uor-r4-float-train` lib with CUDA: **625 passed, 2 failed** — the same two pre-existing decode-cache
failures recorded in codex-30. The chunked path is exercised by the run above; the unchunked path is
the one every existing test takes.
