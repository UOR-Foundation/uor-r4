# REPORT: written-state-count-model, claude-423 (the counter's phase profile)

Lab: claude. Order: [ORDER.md](ORDER.md).

## What did not work, first

- **The GPU is not where the time goes** [claude]. Before this unit, "counting TRAIN is 90%" was read as GPU counting. It is not. Of the 1,817 s the eleven orders take at N = 10 on DEV-tune:
  - host work on every gram takes **1,186 s (65%)**: building records, extension runs, the adjust, the assembler's hash and the filter's scan;
  - device-to-host copies take 91 s (5%);
  - all GPU kernels together take **490 s (27%)**.
- **One GPU phase is mostly waste** [claude]. `select` makes a full pass over all 2.09 B TRAIN tokens for every partition, 256 passes per order. It takes about 24 s per order whatever the order, 261 s in all, which is more than the whole radix sort (198 s).
- **Order k+1 is counted twice, as suspected** [claude]. At every order k < N, the adjusted counter counts order k+1 over order k's ranges to get its extension runs, and then counts it again as its own order. That is 375 s of the counting (21%).
- **No fix is built here**, as the ORDER says. The savings below are bounds from measured times, not results.

## What changed

- `ngram_count_device.rs` [claude]:
  - `CountPhases` and `NgramCounter::take_phases`: integer nanosecond totals in atomics, reset when read;
  - each GPU phase ends with a stream synchronize, so its time is its own;
  - the adjusted counter times its own count, its order-k+1 extension count, the runs and the adjust apart.
- `kn-reference` [claude]: one `phases ms` line per order on stderr, with the sink's `push` (assembler) and `keep` (filter) timed apart. The JSON is unchanged.
- `ngram_count_cuda.rs` [claude]: `ngram_count_cuda_phase_totals_are_recorded_and_reset`.

## Measurements

The run is wick's 4090 at `61d011ee5`, budget 2^24, all of TRAIN against all of DEV-tune, with `--check-host` (job `c423-profile`, `run1/c423/`).
- **N = 10:**
  - wall 1,910 s;
  - counting 1,824 s;
  - GPU scoring 1.1 s;
  - host check 81 s;
  - peak resident memory 8,864,644 KiB.
- **N = 4:** wall 295 s.

The phase totals for N = 10, summed over orders 0–10:

| Phase | Where | Seconds | Share of 1,817 s |
|---|---|---|---|
| filter `keep` | host | 379 | 21% |
| `select` (a whole-stream pass per partition) | GPU | 261 | 14% |
| extension runs | host | 261 | 14% |
| assembler `push` (the whole-order SHA-256) | host | 241 | 13% |
| radix sort | GPU | 198 | 11% |
| record building | host | 185 | 10% |
| adjust | host | 120 | 7% |
| device-to-host copies | transfer | 91 | 5% |
| boundaries, emit, scans | GPU | 31 | 2% |
| histograms | GPU | 0.5 | 0% |
| other: filter set-up, `finish` | host | about 50 | 3% |

The split of the count phases:
- order k's own count: 395 s;
- the order-k+1 extension count: 375 s.

Per order:
- the sort grows with the order, from 3.4 s at order 0 to 34 s at order 9;
- `select` stays at about 24 s;
- the host phases (records, runs, adjust, push and keep) grow with the grams, from 1.6 s at order 1 to 229 s at order 9.

## What the numbers say

Each candidate fix, with the most it can save (bounded by the times above):
1. **Filter on the GPU, before the copy.** Keep only the grams whose context the query reads, and the order-k+1 grams whose suffix is one of them. That leaves about 0.27% of the grams for the host. It removes most of the copies, record building, runs, adjust and keep: **up to about 1,036 s (57%)**. The whole-order digest (`push`, 241 s) still needs every gram, unless it moves to the GPU (**up to 1,277 s, 70%**) or runs as a separate audit pass.
2. **One bucketing pass per order instead of a stream pass per partition.** Bucket targets by their partition token once. That cuts `select` from 261 s to about two passes per order: **up to about 250 s (14%)**.
3. **Count each order once.** Orders k and k+1 split on the same partition token (`partition_offset` = 1), so one count of order k+1 per range can serve both steps. That removes the 375 s extension count, part of which item 1 already removes.
4. **The N sweep.** An order-k adjusted table is the same for every N > k. This is checked here: orders 0–3 of the N = 4 and N = 10 runs have identical triple and context digests, counts of counts and discounts. Order 4 differs, because at N = 4 it is the top order and keeps raw counts. So one N = 10 run already holds every lower N's tables except the top order's raw counts. The 2 h sweep could be one run plus ten top-order raw counts.

With items 1–3, counting is bounded below by the sort and emit: about 230 s, and about half that once each order is counted once. That holds **only if the whole-order SHA-256 also leaves the host**. With it still there, the bound is about 230 + 241 s, roughly 8 minutes. So the 30-minute count could plausibly become **3–5 minutes**, or about 8 with the host digest kept. These are estimates from the profile, not measurements.

## Checks run at the head

- **G1, laptop, at `61d011ee5`:** the stub-nvcc `cargo check -p uor-r4-discrete --features cuda --lib --tests` and the instrument's `cargo check --features cuda` are clean. The host `kn_` and `ngram_count` tests pass, 31 / 31.
- **G2, wick:** the CUDA tests `ngram_count_cuda kn_score_cuda kn_filter` pass, 14 / 14, including the new phase-totals test.
- **G3, wick:** the N = 10 JSON is `cmp`-identical to claude-422's `sweep-10.json` but for `masses_at_floor`. The timers and synchronizes change no output.
- **G4:** this REPORT.
- **Not run:** a negative of the new test. It asserts `extension_count_ns > 0` directly, so dropping that tick would fail it by construction. It was not run on wick.

## Skills used

working-discipline.

## Still open

- Build fix 1 (filter before the copy, with the digest moved to the GPU), then fix 2. Each needs a `cmp` check of its JSON against `sweep-10.json`.
- Fix 3 and the shared N sweep come after.

## Plain summary

We timed every step of the counter that builds R\*. Most of its half hour is not GPU math. It is single-threaded CPU code touching each of more than a billion grams, of which the query keeps a quarter of one percent. Moving that filter onto the GPU, and scanning TRAIN once per order instead of 256 times, could bring the count from about 30 minutes to about 3–5 minutes. Nothing was changed in what is counted.
