# Lab record: integer-nearest-readout

Item: [integer-nearest-readout](../../roadmap/items/integer-nearest-readout.md) (#3462). It serves the trained model's readout, whole-table nearest, in the integer artifact.

## Locating claude-232's gap: the trunk and the Hamming read on the GPU (claude-233, 31 Oct 2026): row R

- **Sheet:** [integer-nearest-readout-locate-2026-10-31](../../harness/prep/integer-nearest-readout-locate-2026-10-31.md). It passed FIT in round 2, then a pre-run review (GO). Both rounds are recorded in the sheet.
- **Where it ran:** one RTX 5090 (Vast pod `vast-58521974`), at the pushed head `ecabcc93b`. The job ran 10:32–10:37Z with EXIT 0; the pod was up about 16 minutes, roughly $0.30, then deleted.
- **Store:** `caseyallard/uor-r4-store:results/claude/integer-nearest-readout-locate-2026-10-31/`, listed back.
  - **`out/`:** 80 files under `SHA256SUMS`, each checked on the pod before upload.
  - **The job:** `job.sh` (sha256 `1f8c4b9c…`), beside `out/`.
  - **`reading/`:** the reading (`read.sh`), the diagnostics (`diag.sh`), their outputs and the watcher, under their own sums.
- **The arms:** (r-drop) step 16,384, without the turn memory:
  - t1 and t3: the trunk alone, at s1 and s3;
  - h1 and h3: the trunk with its Hamming read, at s1 and s3.

  Each arm ran `fbs-snapshot` with its three readouts, then `fbs-export` and `fbs-compare --parity-only` on a writer copy of the snapshot: the snapshot less its 23-byte `readout_nearest` field. The D11 readout's token is the writer's, so both sides read the writer.

### Instruments: all held

- **`I-Data`:** each seed's 6 files pass the run's sums.
- **`I-Export-Tests`:** the `fbs_export::` GPU tests pass, 12 of 12.
- **`I-Reading-Carried`:** holds on every arm:
  - `retain_rates` once;
  - the read on the h snapshots only;
  - `readout_nearest` once, with each writer copy exactly 23 bytes shorter;
  - no `native_vector_nearest` in any artifact.
- **`I-Artifact-Identity`:** holds for all four arms.
- **`I-Finite`:** all four `compare.json` files exist.
- **`I-Repro`:** t1 is claude-227's run again, count for count (d of 5 and 73). Its writer snapshot, artifact and D11 readout are byte-identical to claude-227's stored ones (`e8f2f2e6…`, `003c65fd…`, `e7a50dba…`).

### The reading: row R

The bar is d ≤ 46 of 1,536 sequence positions and ≤ 122 of 4,096 single tokens. d is the number of positions where the D11 float token and the integer token disagree.

| Arm | Sequences d | Single tokens d | Positions with a query gap over 1e-3 (sequences) | Largest query gap (sequences) |
|---|---|---|---|---|
| t1, trunk, s1 | 5 | 73 | 435 | 3.89 |
| t3, trunk, s3 | 11 | 104 | 513 | 2.83 |
| h1, trunk + read, s1 | **694** | 73 | 1,526 | 8.04 |
| h3, trunk + read, s3 | **823** | 104 | 1,528 | 8.19 |

- **The trunk alone passes at both seeds.** At s3, the seed where claude-232 collapsed, it agrees with its own D11 forward on 1,525 of 1,536 sequence positions.
- **With the read, both seeds miss, and each exceeds its own trunk:** 689 > 2√699 ≈ 52.9, and 812 > 2√834 ≈ 57.8.
- **The single tokens are unchanged by the read on both sides.** A one-position sequence has no earlier position to read.
- **`R-Exercised`: the read is measured on the sequences.** The float read alone moves the D11 token on 1,129 (s1) and 1,117 (s3) of 1,536 sequence positions, against a bar of 46.
  - It moves none of the 4,096 single tokens.
  - The pre-run review expected the read to be barely exercised, since no token repeats inside these sequences. That expectation did not hold.
- **The integer read is not inert.** An integer read that did nothing would match the float h token where the float read left t's token in place:
  - 402–412 positions at s1 (1,536 − 1,129, ± t1's 5);
  - 408–430 at s3 (± t3's 11).

  It matches on 842 and 713. So it carries part of the float read's effect, but not all of it.

**Row R:** the Hamming read's integer form diverges from its float D11 form, while the trunk agrees at both seeds. This configuration of the integer read is negative: it closes this form of the integer read, never the read as a mechanism (§4.1).

### Diagnostics after the reading (not registered; leads)

These come from `reading/diag.sh` over the stored float readouts.

- **The served export is snapped.**
  - `fbs-export` snaps the Hamilton gates and the transition to the 120 icosians unless it is given `--raw-gates --raw-transition`.
  - claude-232 exported with `--run-config` alone, as this run did.
- **For the trunk alone, the snapped D11 float forward sits far from the trained forward.**
  - Over t1's 1,536 sequence positions, the D11 query differs from the plain (trained) float query by a median 46% of the plain query's L1 norm (range 18–83%). The norm's median is 580.
  - The export gates alone account for a median 28%, and the snapped tables with the Q12 query a further 40%.
  - The writer's token differs between plain and D11 on 1,336 of 1,536 positions, and 1,334 at s3.
- **What that means for the earlier records:**
  - claude-227's parity, and this run's t rows, hold the artifact against its own D11 forward, not against the trained one.
  - claude-227's record says "within the exporter's known residual". On this model that residual is not small.
  - claude-232's float side is the trained model, so this gap is on its path.
- **claude-180 to claude-182 found the gate snap carries most of the D11 gap** (d11-exporter-float-trunk, #2820). They built the unsnapped contract (`lut-stack/5`), which was inside the export-gap bar on dual-c131 at both seeds.
  - The format accepts unsnapped forms beside retention, the read and the memory: a geometric transition needs the icosian snap only when the transition is snapped.
  - The unsnapped contract has not been run on (r-drop).
- **The nearest token is not measured here.** Whether this gap moves the nearest token, the one the trained model decodes, is not measured. The writer's pick is a fragile probe, and this run did not measure the nearest token on these readouts.

### Next

1. **Re-export and rerun claude-232's parity.** Re-export claude-227's full (r-drop) snapshots unsnapped (`--raw-gates --raw-transition --run-config`), and rerun claude-232's end-to-end parity on the laptop against the same stored float replies, under a new sheet.
   - This reads the export gap's share of the miss on real prompts, with the memory on.
   - Before that, check that the unsnapped session serves the nearest head, the read and the memory together.
2. **The read's trace:** integer-hamming-read's step 5 (#3385). Trace the read's hooks, integer against float, on a stored h snapshot: which heads and which positions diverge. Its float side should be the trained forward, or an unsnapped one, so the two gaps are not confused.
3. **The item stays `ready`.**

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

  At a hit the memory moves the query toward the stored reply, so the nearest pick is clear-cut. At a miss the query is unmoved and its margins are likely small (not measured here), so an error in any layer can flip it.
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
