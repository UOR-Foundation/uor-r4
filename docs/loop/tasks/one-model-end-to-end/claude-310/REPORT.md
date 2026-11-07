# REPORT: one-model-end-to-end, claude-310

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Order: [ORDER.md](./ORDER.md). Laptop only; no model, no GPU, no spend.

## What did not work, first

- **(b64L) almost never saw EOS as a training target.** D3 scores 27,416,554 targets, and 20,729 of them are EOS: 756 per million. DEV, the conversations its replies are judged on, has 85,812 per million, about 113 times more. This fits claude-309's reading, where the served model stopped on 0 of 256 panel replies and the float model emitted EOS on 0 of 16 DEV prompts. It does not prove the cause: no training run here varied the EOS share.
- **Most of D3's EOS mark the end of a Wikipedia article, not a reply.** Each wiki document carries one scored EOS: 12,945 of the 20,729. Between consecutive scored EOS, the median run is 510 scored targets over all of D3. In DEV it is 11 (max 50).
- **D3's mask is not a reply mask.** It scores 98% of D3's positions: everything except BOS. In oasst2 it scores 3,234,805 of 3,773,419 tokens, so user turns are training targets too. 1,599 of oasst2's 9,383 EOS fall in unscored spans. DEV scores 33% of its positions, the replies alone. So (b64L) learned to continue text, of which only about 14% is dialogue, rather than to answer and stop.
- **oasst2 alone is still sparse:** 7,784 scored EOS in 3,234,805 scored targets, 2,406 per million. The median run between them is 349.

## What changed

- **New bin `corpus-eos-exposure`** in the item's instrument crate (`docs/labs/one-model-end-to-end/instrument/src/bin/corpus-eos-exposure.rs`, no feature needed). It reads a `tokens.u16` (bare, or behind the 64-byte `UORT` header when the header's count equals the mask's length) and its `response_mask.u8`. It refuses a mask byte other than 0 or 1, or a length mismatch. With a manifest, it splits the file into the manifest's roots in order. It checks that their `tokens_taken` sum to the file and that each root's scored count equals its `scored_taken`.
- **The scoring rule is the trainer's**, read at the pin `ff5156bcc` (`crates/uor-r4-float-train/src/hamilton_sheet.rs`, `build_document_windows`):
  - :1868 describes "scoring each eligible target position (`p > doc_start && tokens[p] != bos_id && mask[p] == 1`)".
  - :1903–1905 pushes `tokens[p]` as the target, and :1902 reads its input at `p - 1`. Context never crosses a BOS (:1894).
  - The sliding variant (:1967, :2020) and `in_space_long.rs` (:845, :2682) use the same rule.
- **Per root and in total, it reports:** positions, scored targets, EOS, scored EOS, BOS, and the runs of scored targets that end at each scored EOS (count, median, max). Runs reset at each BOS.

## Measurements

D3 (`tokens.u16` `ed8f2537…`, `response_mask.u8` `ec6e520c…`, its manifest) and DEV (`ft/dev`, tokens `0919e90f…`, mask `a838bb1a…`).

| root | positions | scored | EOS | EOS scored | scored EOS per million | median run |
|---|---|---|---|---|---|---|
| oasst2 | 3,773,419 | 3,234,805 | 9,383 | 7,784 | 2,406 | 349 |
| wiki-00000 | 8,011,538 | 8,010,674 | 864 | 864 | 108 | 6,546 |
| wiki-00020 | 8,255,877 | 8,247,907 | 7,970 | 7,970 | 966 | 466 |
| wiki-00040 | 7,927,279 | 7,923,168 | 4,111 | 4,111 | 519 | 1,223 |
| **D3** | 27,968,113 | 27,416,554 | 22,328 | 20,729 | 756 | 510 (max 96,128) |
| **DEV** | 42,856 | 14,287 | 1,226 | 1,226 | 85,812 | 11 (max 50) |

The per-root rates are computed from the counts, as integer parts per million. The bin's output records the total's rate. Outputs: `out/d3.json` `e6534537f2c2baf285d474da7de47aaa580e0fdea7f77d8abb1508cc25d06ede`, `out/dev.json` `19225cd3fbcad73111d600e2ef0252d3721c5bed9a299fa0ab32179140d1c18a`. Every root's scored count matched its manifest `scored_taken`, and 0 scored targets were left without a closing EOS.

## Checks run at the head

- `cargo build --release --offline --locked --bin corpus-eos-exposure`: exit 0.
- `cargo test --offline --locked --bin corpus-eos-exposure`: 6 passed. The tests cover the scoring rule (where the target at p and at p+1 give different EOS counts), the per-root split and its sum check, the mask and length refusals, and the run statistics.
- **Negative gate:** with the scoring rule swapped for the next-token rule (`mask[p - 1]`), the rule's test and two others failed (3 of 6). They pass again after the restore and a rebuild (the builder's run; I re-ran the 6 tests at the head).
- **No float sites:** none of the float type tokens or float-literal patterns in the bin.

## Skills used

None beyond the loop's own steps.

## Still open

- **The lever this points to:** train on dialogue whose mask scores only the replies, so that EOS is near DEV's rate, or weight the EOS target. Each would be judged by claude-309's `served-reply-panel`: whether replies stop, then whether they are right. That run needs its own prep sheet and pre-run review.
- **Not measured:** the trained model's EOS rank at the end of a reference reply. That would show whether EOS is nearly chosen or nowhere close.

## Plain summary

The model we serve never stops talking. This unit counted why that might be. In the text it trained on, an "end of reply" mark comes about once every 1,300 tokens, mostly at the end of Wikipedia articles. In the conversations it is tested on, one comes about every 12 tokens. The training also scored the user's half of each conversation as text to predict. So the model learned to keep writing, not to answer and stop. The next training change to try is a corpus of conversations where only the replies are scored, judged by whether the served replies stop and are right.
