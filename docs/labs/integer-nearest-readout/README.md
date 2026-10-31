# Lab record: integer-nearest-readout

Item: [integer-nearest-readout](../../roadmap/items/integer-nearest-readout.md) (#3462). It serves the trained model's readout, whole-table nearest, in the integer artifact.

## End-to-end token parity against the trained model (claude-232, 31 Oct 2026): MISS

- **Sheet:** [integer-nearest-readout-parity-2026-10-31](../../harness/prep/integer-nearest-readout-parity-2026-10-31.md). It passed FIT and a pre-run review.
- **Where it ran:** on the laptop (Apple M1), under the owner's allowance. The integer side is product serving; the float side is claude-220's stored greedy replies.
- **Store:** `caseyallard/uor-r4-store:results/claude/integer-nearest-readout-parity-2026-10-31/`, 40 files under `SHA256SUMS`, listed back. The job is `log/run.sh`.
- **The arm:** (r-drop) step 16,384 at seeds 1–3, exported by `fbs-export --run-config` from claude-227's snapshots:
  - s1 `0064b00a…`, s2 `6879590a…`, s3 `5a02de1f…`;
  - each with head `native_vector_nearest` and `rec_gates` `geometric-retention`, the memory and the Hamming read carried.
  - Decoded greedily with the memory on (`fbs-toks --replies`), on claude-220's own DEV and p-w prompts.

### Instruments: all held

- **`I-Float-Inputs`:** 6 of 6 match claude-220's sums.
- **`I-Head` and `I-Identity`:** hold at every seed.
- **`I-Prompts`:** 310 and 1,202 at every seed.
- **`I-Decode`:** every decode exits 0, with `keyed` 0.
- **`I-Positions`:** every pair matches.
- **The capped counts** are exactly the expected 26 on DEV and 136 on p-w, per seed.

The grader read the known answers before the arm: claude-229's writer decode at 0 of 310, and F against itself at 310 of 310.

### The bar: MISS in every cell

First-token disagreement must be at most ⌊0.03 n⌋: 9 on DEV, 36 on p-w.

| Cell | First token agrees | Disagrees | Whole reply agrees |
|---|---|---|---|
| DEV s1 | 291 of 310 | 19 | 83 |
| DEV s2 | 285 of 310 | 25 | 163 |
| DEV s3 | 284 of 310 | 26 | 127 |
| p-w s1 | 911 of 1,202 | 291 | 60 |
| p-w s2 | 777 of 1,202 | 425 | 113 |
| p-w s3 | 64 of 1,202 | 1,138 | 36 |

The readout was necessary: at s1, under the writer, claude-229's first token agreed on 0 of 310 DEV and 2 of 1,202 p-w prompts; under nearest, on 291 and 911. It is not sufficient: the integer model still decodes differently from the trained one, and far more on p-w.

### Where it differs (readings after the bar; they locate, they do not decide)

- **On DEV s1, by whether the turn memory's exact key fired at the opening** (from claude-229's served decode, whose `keyed` marks an exact-key miss):

  | Openings | Count | First token agrees |
  |---|---|---|
  | Exact key hits | 291 | 280 (96.2%) |
  | Exact key misses | 19 | 11 (58%) |

  At a hit the memory moves the query toward the stored reply, so the nearest pick is clear-cut. At a miss the query is unmoved and the margins are small, so an error in any layer flips it.
- **p-w is mostly misses:** 1,172 of its 1,202 openings at every seed. Its agreement is 76%, 65% and 5%.
  - The misses are the same at every seed, so they do not explain the spread across seeds, which is unexplained.
  - DEV against p-w changes the panel too (this unit's review).
- **The integer model over-concentrates on p-w.**
  - At s3 it collapses: its first token is 351 on 1,128 of 1,202 prompts, where the float model's are spread (318 on 219, 1155 on 130, 2611 on 124, 340 on 94).
  - At s2, 340 is its first token on 496 prompts, against the float model's 192.
  - A collapse like this also fits one readout point winning every nearest search.
- **So the suspects are every layer on the miss path:** the trunk, the Hamming read, and the readout table. These are leads, not findings.
  - claude-227's parity checked the trunk alone, on synthetic sequences.
  - claude-231 checked the readout only against an integer reference scan.
  - The Hamming read's integer form has not been checked on a trained model (integer-hamming-read's step 5, #3385).

### Next

Locate the gap layer by layer, as the sheet's MISS row says (trunk, Hamming read, memory, readout), on the first disagreeing prompts:
- **On the GPU:** claude-227's procedure with the Hamming read in both the snapshot and the readouts, on (r-drop) s3 and s1, on real prompts (DEV and p-w windows), trunk plus read against trunk alone. `fbs-compare`'s token is the writer's on both sides; its query gaps carry the comparison.
- **The readout table:** the integer writer table's rows against the float decode's table, and the nearest pick at the disagreeing openings.
- The item stays `ready`.
