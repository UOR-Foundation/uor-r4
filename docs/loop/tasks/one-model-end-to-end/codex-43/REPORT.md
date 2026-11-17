# REPORT: one-model-end-to-end, codex-43 (the sheet takes a shape; two hard bounds found)

Lab: codex. Follows codex-42 (the corpus). Goal: a 30 M geometric model on the real donor corpus.

## What was built

The trainer's model shape is no longer only the sheet's constants:

- `hamilton_sheet::build_hamilton_model_with(seed, width, layers, vocab, device)` — the sheet's
  heads, MLP, gates, objective and native writer, at an explicit depth and vocabulary.
  `build_hamilton_model_at` now delegates with `(6, SHEET_VOCAB)`, so every existing call is
  byte-identical.
- `spiralcore_fbs_sheet::build_fbs_model_with(arm, seed, width, layers, vocab, device)`; the old
  entry point delegates with the sheet's values.
- `LongConfig` gains `vocab_size` and `layers`, both skipped in `config.json` at the sheet's values;
  `train-in-space` gains **`--vocab-size`** and **`--layers`**.
- The `--init-from` compatibility check now reads the run's vocabulary and pattern instead of the
  constants.
- The Hamming read tables (`with_ask`/`with_hold`/`with_carry`) size to the run's vocabulary.
- **The corpus loader now reads the UORT header's vocabulary** (bytes 16..20) instead of assuming
  4,096; a header of 0 is a legacy fixture and keeps the sheet's. This is what lets a larger-tokenizer
  corpus load at all.

## What ran

A 30 M model — **width 512, 12 layers, vocabulary 8,192, context 512** — built on the GPU, cut the
real donor corpus into **11,237 windows** from 5,738 conversations (4,894,652 tokens), and ran
step 0 with a full DEV evaluation. Then it stopped on the second bound below.

## Two hard bounds found (both real, both about the writer)

1. **The native vector writer's vocabulary is 256..8192** — `CELLS (256) ..= CELLS × MAX_PER_CELL
   (32)`, `native_vector_writer.rs:33`. A 32,768-piece tokenizer **cannot be used with this writer at
   all**; 8,192 is the ceiling. Built an **8,192-piece** tokenizer on the donor text
   (`blake3:226a2016…`, 7,933 merges) and it works; the 32k one is unusable until the writer's
   identity partition changes.
2. **The writer's candidate-vector intermediate is capped at 256 M f32 (1 GiB)**
   (`MAX_CANDIDATE_VECTOR_ELEMENTS`). At 8 kept cells × 32 slots × width 512 that is **≤ 2,048 rows
   per call**, and the DEV panel passes **14,287**. The panel scoring must therefore be
   **row-chunked**; the writer's own error names it ("use explicitly weighted row chunks"). That is
   the next code change, and it blocks any eval at this width and vocabulary.

## Checks

`uor-r4-float-train` lib with CUDA: **625 passed, 2 failed** — the same two pre-existing
decode-cache failures recorded in codex-30 (`the_step_cache_matches_the_whole_window_bit_for_bit`,
`the_decode_cache_decodes_the_same_replies_through_a_window_slide`). The shape defaults are unchanged:
the sheet's model is built by the same values through the new function.
