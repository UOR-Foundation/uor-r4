# Prep sheet: R\*, the sealed modified Kneser–Ney reference of bar Q (written-state-count-model, claude-422, 17 Nov 2026)

Lab: claude. Item: [one-model-end-to-end](../../roadmap/items/one-model-end-to-end.md) (#3407). Order: [claude-422](../../loop/tasks/written-state-count-model/claude-422/ORDER.md). Code: branch `claude/kn-sealed-422`, off `main` at `9465b92d6`. Release cited at dispatch: D86 (§3.10), wick is the owner's GPU.

## The question

What is R\*'s BPB on sealed D87 DEV-test, on all targets and on the novel subset? Bar Q (claude-409 DECISION §2) asks a model for at most 0.80 × that number, so R\* must be sealed before any model is scored on DEV-test.

R\* is defined as follows. It is interpolated modified Kneser–Ney:
- three discounts per order, from that order's continuation count-of-counts n1..n4 (Chen–Goodman);
- full N1/N2/N3+ counts per context, unpruned;
- adjusted (continuation) counts below the top order; raw counts at the top order and for grams that start at BOS;
- the uniform 1/4096 floor under order 0;
- integer Q48 masses, with bits by `lawful_likelihood::bits_q12`.

It includes **claude-421's fallback rule**. An order whose n1..n4 holds a zero, or whose Chen–Goodman estimate D_i leaves [0, i], takes KenLM's `--discount_fallback` discounts (1/2, 1, 3/2) for the whole order. On all of TRAIN at N = 2, order 0 takes the fallback (n1..n4 = [7, 6, 11, 11]).

**N** is the longest context, in tokens: an N of k scores (k+1)-grams. The decision card expects 8–12 as n-gram length, which is N = 7..11 here. The key holds at most 10 context tokens (`NGRAM_MAX_ORDER`), so the sweep is N = 1..10.

The counts are exact. They are hashed per partition on the GPU and assembled in canonical order, not taken from a suffix array, but they give the same numbers. The digests and SHA are claude-418's.

## Steps

All on wick's RTX 4090, from a source tarball of this branch's head. The instrument is `docs/labs/kn-reference/instrument` (claude-421).
1. **Build and CUDA tests.** `cargo test --release --features cuda --lib kn_score` passes (claude-421's 10 tests).
2. **Sweep on DEV-tune**, N = 1..=10:

   `kn-reference train.tokens.u16 dev-tune.tokens.u16 id-bytes.u8 dev-tune.novel.u8 N 134217728 sweep-N.json --check-host`

   This uses the same budget as claude-421's scale run (2^27 positions per partition). Each run counts all of TRAIN at orders 0..N and scores all of DEV-tune. The host scorer checks every mass.
3. **Choice.** N\* is the N whose `all.bits_q12` is smallest. The bytes are the same at every N, so this is the smallest BPB. A tie goes to the smaller N. The choice is made from the ten JSON files by a fixed `jq` line in the job script, written before the run.
4. **Oracle at N\*.** The ignored test `kn_score_cuda_real_slice_equals_the_cpu_oracle` with `UOR_KN_N=N*` and `UOR_KN_PER_MILLE=2`. It takes the first 2‰ of TRAIN and of DEV-tune, each cut at a record start, and requires every GPU mass to equal `KnOracle`'s. `KnOracle` is the independent CPU oracle: ordered maps keyed by the context tokens, with its own discount rule. The 2‰ slice keeps its maps small at high N.
   - **Negative:** `oracle_discounts`'s range test is replaced by `true` (no fallback on an out-of-range estimate). The check must then fail, provided some order of the slice has an out-of-range estimate. If none does, the negative instead sets the oracle's gamma to 0, and the check must fail.
   - The test is restored afterwards, and a rebuild into a fresh target folder confirms it passes again.
5. **Reproducibility.** A second DEV-tune run at N\*, into a new file, must be `cmp`-identical to the first.
6. **Sealed run.** Run once only:

   `kn-reference train.tokens.u16 dev-test.tokens.u16 id-bytes.u8 dev-test.novel.u8 N* 134217728 sealed.json --check-host --sealed-run`

   It runs only after steps 1–5 pass.
7. **Storage.** The JSON files and logs are fetched to the laptop and committed under `docs/loop/tasks/written-state-count-model/claude-422/run1/`. They hold counts, digests and hashes, never a stream, mask or table. Wick keeps the outputs at `/workspace/out/c422`.

## Bar

This unit is a measurement: it produces R\*. Its bars decide whether the sealed number is valid, not whether it is good.
- **B1 instrument, every run:** exit 0, and host = GPU at every target.
  - Targets and bytes equal claude-420's counts:
    - DEV-tune: all 10,807,417 targets / 21,928,475 bytes; novel 7,566,174 / 15,995,822;
    - DEV-test: all 21,349,576 / 43,326,982; novel 14,795,227 / 31,328,319.
  - `train_tokens` is 2,091,560,001.
  - Every input's sha256 equals claude-420's:
    - TRAIN `c6cba29e…`;
    - DEV-tune `75c6d1ac…`;
    - DEV-tune novel `7df42437…`;
    - DEV-test `023784b7…`;
    - DEV-test novel `5bf3881b…`;
    - id-bytes `befb22f6…`.
- **B2 oracle:** step 4 passes at N\*, and its negative fails.
- **B3 reproducibility:** step 5's files are `cmp`-identical.
- **Recorded, with no bar:** for each N, all and novel BPB (micro-bits per byte), Q12 bits, each order's grams, kept records, discounts and fallback flag, the counting and scoring times, and the peak device memory.

**Reading map** (fixed before the run):
- **P (sealed):** B1–B3 hold on DEV-tune, and the sealed run passes B1. R\*'s record is the sealed JSON:
  - N\*;
  - DEV-test all and novel `bits_q12`, `bytes`, `bpb_micro`;
  - `masses_sha256`, the per-order digests, discounts and fallback flags;
  - the input shas.

  The REPORT writes out the bar Q lines in micro-BPB, for all and for novel: 0.80 × R\* (the bar), and the 0.90 × and 0.95 × checkpoints. Each is rounded down to an integer.
- **I (instrument):** B1, B2 or B3 fails on DEV-tune. The instrument is repaired under this sheet, and DEV-test is not touched until all three pass. If the sealed run fails B1, its number is not R\*. The failure is classified, and a repaired sealed run needs a new sheet, which names the first run.
- **Edge:** N\* = 10, the key's limit. A longer context might score lower, which would make R\* stronger and bar Q harder. R\* is still sealed at N = 10 and labelled "edge-limited". The REPORT gives DEV-tune's gain from N = 9 to 10. A follow-up unit widens the key, and a stronger R\* found later replaces this one under its own sheet.
- **Short:** N\* ≤ 3, below the card's expectation. This is not an instrument failure by itself. B2 at N\* and at N = 7 (both run) must pass before the sealed run, to rule out a scorer fault that grows with N.

## Base artifact

- **Code:** this branch's head (`main` at `9465b92d6` plus the test settings commit). The instrument and every run's JSON record no commit themselves, so the job writes `git rev-parse HEAD` of the tarball's source to `head.txt`.
- **Streams:** claude-420's `/workspace/out/c420/run1/` on wick: `train.tokens.u16`, `dev-tune.tokens.u16`, `dev-test.tokens.u16`, `dev-tune.novel.u8`, `dev-test.novel.u8`, `id-bytes.u8`. These are read only. Their sha256 values are claude-420's committed `run1/*.json`.
- **Machine:** wick, NVIDIA GeForce RTX 4090, in Claude's container (`local/claude/wick/wick.sh`).

## Data split

- **Split v1, by prompt id** (claude-411; claude-417 kept it): TRAIN gives the counts, DEV-tune chooses N, DEV-test is sealed.
- **Novel subset** (claude-411's rule, claude-420's masks): a target is novel when the 13 tokens before it in its record form no 13-gram of TRAIN. Bar Q gates on all targets and on the novel subset.
- **DEV-test** is scored once, in step 6. No earlier run reads it: the instrument refuses its sha256 without `--sealed-run`.

## Minimum test power

- R\* is deterministic. Counts, masses and bits are exact integers, so it has no seeds and no sampling error, and step 5 checks the run-to-run claim.
- N is chosen on 10.8 M DEV-tune targets. A difference of one Q12 unit decides between two N, so the choice is exact. Whether it generalizes is the usual DEV-tune-to-DEV-test question. The REPORT gives DEV-tune's BPB at N\* beside DEV-test's.
- The oracle check at N\* covers about 21,000 DEV-tune targets at 2‰ and requires more than 1,000 distinct masses.

## Cost

Wick is the owner's GPU, so there is no rental (D86). Claude-421 timed about 10 s per order on all of TRAIN. The sweep counts 2 + 3 + … + 11 = 65 orders, about 11 minutes, plus host checks of about 5 s per run. The oracle checks take a few minutes; the second DEV-tune run and the sealed run about 2 minutes each. In all, under an hour. No pod. `wick.sh gpu` is read before the job, which runs only if no one else's work holds the GPU.

## Stop rule

- **Hard stop:** a job that passes 2 hours is stopped by its own PID, and the finished runs are reported.
- **A sweep run that fails B1** ends the sweep. It is classified (instrument or memory) and repaired, and the sweep is rerun from N = 1 under this sheet. DEV-test is untouched.
- **The sealed run is run once.** Its first written JSON is the record. A crash before any JSON is written means no score was seen, so the run is repeated once. After that, a new sheet is needed.
- **No number seen on DEV-test feeds back into the choice of N.**

TEST FITNESS: PENDING
