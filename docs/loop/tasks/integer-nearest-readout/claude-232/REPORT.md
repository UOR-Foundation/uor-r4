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
  - p-w is mostly misses: 1,172 of 1,202 openings at every seed. Its agreement is 76%, 65% and 5%.
  - The misses are the same at every seed, so they do not explain the spread across seeds, which is unexplained (this unit's review).
- **The integer model over-concentrates on p-w.**
  - At s3 it collapses: 351 is its first token on 1,128 of 1,202 prompts, where the float model's are spread.
  - At s2, 340 is its first token on 496 prompts, against the float model's 192.
- **Even exact-key hits disagree on 3.8%** (11 of 291 on DEV s1), which is over the 3% bar on its own.
- **The cause is not located,** and the suspects are every layer on the miss path: the trunk, the Hamming read, and the readout table.
  - At a hit, the memory moves the query toward a stored reply, so the nearest pick is clear-cut. At a miss, the query is unmoved and its margins are likely small (not measured here), so an error in any layer can flip the token.
  - A one-token collapse also looks like one readout point winning every nearest search (this unit's review).
  - **What has been checked:**
    - claude-227's parity checked the trunk alone, on synthetic sequences;
    - claude-231 checked the readout only against an integer reference scan, never against the trained model;
    - the Hamming read's integer form has not been checked on a trained model (integer-hamming-read step 5, #3385).
  - These are leads, not findings.
- **The grader's known-answer probes** (0 of 310 and 310 of 310) are recorded in the sheet, but their outputs are not among the sealed files.

## What worked

- **The readout was necessary.** At s1, the first token agreed under the writer (claude-229) on 0 of 310 DEV and 2 of 1,202 p-w prompts. Under nearest it agrees on 291 and 911.
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

- **Locate the gap layer by layer, as the sheet's MISS row says** (trunk, Hamming read, memory, readout), on the first disagreeing prompts, under a sheet. The candidates:
  - **On the GPU:** claude-227's trunk parity again, with the Hamming read in the snapshot and the readouts, on (r-drop) s3 and s1, on real DEV and p-w windows, trunk plus read against trunk alone. `fbs-compare`'s query gaps carry the comparison. This is also integer-hamming-read's step 5 (#3385).
  - **The readout table:** the integer writer table's rows against the float decode's table, and the nearest pick at the disagreeing openings against the float query's nearest.
- **Then fix what it finds, and rerun this sheet.**
- **The item stays `ready`.** point-key-integer's rerun waits on it.

## Plain summary

With its new way of picking words, the converted model now usually starts its answer the way the trained model does. On the main check set, that's 284 to 291 times out of 310, up from never before. But it still falls short of the bar, and on a panel of new phrasings it often doesn't match at all. At one seed it nearly always picks the same word.

The mismatch shows up mostly where the model can't look up a stored answer and has to work from its own reading of the prompt. So the next step is to find where its reading of the prompt differs from the trained model's.
