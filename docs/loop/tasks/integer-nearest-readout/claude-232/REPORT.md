# REPORT: integer-nearest-readout, claude-232 (end-to-end token parity against the trained model): MISS

Lab: claude. Item: [integer-nearest-readout](../../../../roadmap/items/integer-nearest-readout.md) (#3462). Order: [ORDER.md](ORDER.md), written by hand, since `loop claim` refuses on #3436's false glob overlap. Sheet: [integer-nearest-readout-parity-2026-10-31](../../../../harness/prep/integer-nearest-readout-parity-2026-10-31.md).

## Tokens per second

No new measurement; claude-231 timed the nearest head at 1,365 tok/s decode on (r-drop) s1. Here the six decodes took about 24 s for DEV's 310 prompts and 98 s for p-w's 1,202, per seed (`log/decode-*.time`).

## What did not work, first

- **The bar missed in every cell. The integer artifact still does not decode as the trained model does.**

  | Cell | First token agrees (bar: ≤ 9 or 36 disagree) | Whole reply agrees |
  |---|---|---|
  | DEV s1, s2, s3 | 291, 285, 284 of 310 | 83, 163, 127 |
  | p-w s1, s2, s3 | 911, 777, 64 of 1,202 | 60, 113, 36 |

  - The sheet's MISS row applies: the item stays `ready`, and the next unit locates the gap.
  - For point-key-integer this is an instrument fault, so its rerun on p-x waits on it.
- **The gap is largest where the turn memory's exact key misses.**
  - On DEV s1, exact-key hits agree on 280 of 291, and misses on 11 of 19. The key's status comes from claude-229's served decode.
  - p-w is mostly misses (1,172 of 1,202 openings), and its agreement is 76%, 65% and 5%.
- **On p-w s3 the integer model collapses to one token:** 351 is its first token on 1,128 of 1,202 prompts, where the float model's are spread.
- **Even exact-key hits disagree on 3.8%** (11 of 291 on DEV s1), which is over the 3% bar on its own.
- **The cause is not located.** The first suspect is the trunk with its Hamming read on real prompts.
  - claude-227's parity checked the trunk alone, on synthetic sequences.
  - The Hamming read's integer form has not been checked on a trained model (integer-hamming-read step 5, #3385).
  - That the gap sits on the memory's miss path, and is worst at one seed, fits a fault in the trunk or read, not in the readout or the memory's move. It is a lead, not a finding.

## What worked

- **The readout was necessary.** Under the writer (claude-229), the first token agreed on 0 of 310 DEV prompts; under nearest it agrees on 284–291.
- **Every instrument held:**
  - `I-Float-Inputs`, 6 of 6;
  - `I-Head`: `native_vector_nearest` and `geometric-retention` at every seed;
  - `I-Identity`;
  - `I-Prompts`: 310 and 1,202;
  - `I-Decode`: every exit 0, `keyed` 0;
  - `I-Positions`: every pair.
  - The capped counts were exactly the predicted 26 and 136.
- **The grader read its known answers before the arm:** claude-229's writer decode at 0 of 310, and F against itself at 310 of 310.
- **The sealed run:** `results/claude/integer-nearest-readout-parity-2026-10-31/`, 40 files under `SHA256SUMS`, listed back. The three `/7` artifacts are `0064b00a…`, `6879590a…` and `5a02de1f…`.

## What changed

- `docs/harness/prep/integer-nearest-readout-parity-2026-10-31.md`: the sheet, its FIT and the pre-run review.
- `docs/labs/integer-nearest-readout/README.md`: the lab record.
- `docs/research/notebook/m4-serving.md`: one row.

The item's status stays `ready`, so its file is unchanged.

## Measurements

- The six cells above.
- The breakdown by exact-key hit and miss on DEV s1.
- The first-token distribution on p-w s3.

## Checks run at the head

- The sheet's §4 checks, each read from the run's files, as above.
- `the_real_sheets_pass_the_shape_rule` passes locally.

## Skills used

None.

## Still open

- **Locate the gap on the GPU,** under a sheet:
  - claude-227's trunk parity again, with the Hamming read in the snapshot and in the readouts, on (r-drop) s3 and s1;
  - on real prompts (DEV and p-w windows) in place of the fixed sequences;
  - trunk plus read against trunk alone.

  `fbs-compare` compares the writer's token on both sides, and its query gaps carry the comparison. This is also integer-hamming-read's step 5 (#3385).
- **Then fix what it finds, and rerun this sheet.**
- **The item stays `ready`.** point-key-integer's rerun waits on it.

## Plain summary

With its new way of picking words, the converted model now usually starts its answer the way the trained model does. On the main check set, that's 284 to 291 times out of 310, up from never before. But it still falls short of the bar, and on a panel of new phrasings it often doesn't match at all. At one seed it nearly always picks the same word.

The mismatch shows up mostly where the model can't look up a stored answer and has to work from its own reading of the prompt. So the next step is to find where its reading of the prompt differs from the trained model's.
