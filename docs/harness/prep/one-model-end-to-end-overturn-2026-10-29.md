# Prep sheet: the overturn check (one-model-end-to-end, claude-205, 29 Oct 2026)

Lab: claude. Item: [one-model-end-to-end](../../roadmap/items/one-model-end-to-end.md), done-when 2. Order: [claude-205](../../loop/tasks/one-model-end-to-end/claude-205/ORDER.md). Code: this unit's head, built and tested on the pod from that exact commit. Plan: [the assessment](../../loop/tasks/infra-loop-harness/claude-201/ASSESSMENT.md) §5–§6, step 2.

TEST FITNESS: (to be filled by the fresh reviewer). PRE-RUN: (to be filled).

## The question

**Does the trunk that learned to lean on the turn memory read worse than the trunk that never had it, on the rows the memory cannot name?**

- **turn-memory-reply (claude-194, cell A at both seeds):** the reply table carries a reply past its first word. But taking the memory out of (r) at its last checkpoint drops DEV from 11,513 / 11,361 hits to 4,101 / 3,807, far below the empty control (r0)'s 10,079 / 11,171 (seed 1 / seed 2).
- **So the trunk co-adapted with the memory** where the memory names a row. That is expected, since the memory moves those rows' readouts. The fear (the assessment §5; Engram arXiv:2601.07372 and RETRO arXiv:2112.04426 report the same pattern) is that it also reads worse **where the memory names nothing**: the rows the shipped model must earn from the prompt.
- **The memory moves the readout only, and only at rows its tables name** (`flock_read_memory.rs:1-25`). So on those rows (r) with and without its memory pick the same token, and the comparison is purely trunk against trunk: (r)'s against (r0)'s.

## The arms

The four step-16,384 checkpoints of claude-194, from `caseyallard/uor-r4-store`:

| Arm | Seed | Store folder | `model.safetensors` sha256 |
|---|---|---|---|
| (r) reply table | 1 | `results/claude/turn-memory-reply-2026-10-29/s1-fc66ed74/r-s1/models/step-016384/` | `b89505e125de187116311530af1f42cdd186d84013627278a47ef0fc7d41f310` |
| (r0) empty tables | 1 | `…/s1-fc66ed74/r0-s1/models/step-016384/` | `fdb218288f4f6dc53241a72b2eb8709259ea54596aae2e8568f81b0dad9f2125` |
| (r) | 2 | `…/s2-fc66ed74/r-s2/models/step-016384/` | `c55401a49a14c0bb45426c99d5a776f1ea67a5126cebd3abb764d185a9613019` |
| (r0) | 2 | `…/s2-fc66ed74/r0-s2/models/step-016384/` | `5a33ad329b73906f7785eae9cff57f8ec1d05d65726cb0606fea326cf4677deb` |

- **Every file is checked against its sealed list.** Each folder's five files are checked against the seal's `SHA256SUMS` before use: seed 1's list has sha256 `8d40536e162866d0944e85fe8a476042b266f722a1256a91d87001c945017625`, seed 2's `e4e5a5f53187e3cec14f20ca193fbb0e5944e92a28a2315b36835bf55204cef2` (claude-194 REPORT, "Cost and seal").
- **The five files** are `config.json`, `model.safetensors`, `hamming_read.safetensors`, `turn_memory.safetensors` and `turn_memory_reply.safetensors`.
- **(r) memory-off** is (r) with its memory taken out at load (`score_rows`'s second pass), not a separate checkpoint.
- **(m) is not scored.** The question compares a trunk trained with the memory against one trained without, and (m)'s opening-only memory answers neither side.

**How each is scored.** Each checkpoint is scored by `train-in-space --data-root ROOT --score-rows CKPT --out DIR --fresh-v2 FRESH --samples 64 --reply-tokens 64 --retain-rates 1 --seed S`. These are claude-194's `--inspect` flags, at its seed. The binary is built on the pod from this unit's head, `--locked`, and its sha256 is recorded before the first arm (the job's `binary-sha256` step).

## The data

All pinned by sha256 in the code (`load_dev`, `load_train_split`, `pinned_tokenizer`, `load_fresh_v2`):
- **DEV** (`ft/dev`), 14,287 scored rows.
- **TRAIN** (`mw-balp/train`), 7,512,797 tokens.
- **The tokenizer.**

These three come from `w2-inputs/2026-10-25/driver-data/`, as claude-194's job fetched them. **Fresh V2** comes from `inputs/fresh-v2-sealed/2026-10-26/codex-attention-dose1-fresh-v2-preservation.tar` (sha256 `2cd13ef672990a01f7a9e63023e03afacecb9ba8883cd9d7d844ca12cfb60143`). Its two members are `tokens.u16` (`0b7e754c…`) and `response_mask.u8` (`a1cda529…`), checked by the code's pins.

## The decision rows

- **Which rows.** DEV's 3,406 later rows the reply table does not name at share 0.5, and its 32 openings no first-8 key names: 3,438 rows, judged **pooled**. The 32 alone are too few to decide anything, so they are reported beside the pool, never judged apart.
- **How they are computed.** The grader recomputes them from TRAIN and DEV with the census's own function (`decision_positions` over `census_decision_rows`, shared with `census`). The census output is byte-identical after that refactor (`cmp` against [census-2026-10-29.json](../../labs/one-model-end-to-end/census-2026-10-29.json), at `df7675b31`).

## The rule, fixed in advance

At each seed `s`, on the decision rows:
- **The counts.** `b(s)` is the number of rows (r) gets and (r0) misses, and `c(s)` the number (r0) gets and (r) misses. `d(s) = b(s) − c(s)`, the nearest-decoded top-1 difference.
- **The band.** `band(s) = max(⌈2·√(b(s) + c(s))⌉, |d(1) − d(2)|)`: two paired standard errors, or the two-seed spread of the paired difference, whichever is wider. This is the Fresh V2 sheet's band, with its spread taken on the paired difference because both arms vary by seed here.
- **The cell at seed `s`.** **Ahead** if `d(s) > band(s)`, **behind** if `d(s) < −band(s)`, and **inside** otherwise. A difference exactly at the band is inside.

| Verdict | When | What it unlocks |
|---|---|---|
| **ahead** | ahead at both seeds | The co-adaptation fear is not borne out on DEV's unnamed rows. The memory recipe stays the training default; no memory-dropout arm. |
| **behind** | behind at both seeds | The trunk reads worse where the memory is silent. The training run carries a memory-dropout arm beside (r) and (r0). |
| **mixed** | the seeds disagree, or either is inside | No change to the recipe. The training run carries both (r) and a memory-dropout arm. |

Every outcome changes what the training run (step 5 of the assessment) trains, so none is wasted. The grader (`train-in-space --overturn 1`) applies the rule in code, with unit tests for the band (`the_band_floor_is_twice_the_root_rounded_up`), the cells (`a_seed_is_ahead_or_behind_only_beyond_the_band`), the verdict (`the_verdict_needs_every_seed`) and the pairing (`pairing_counts_discordant_rows_on_the_decision_rows_only`).

## Descriptive only, with no threshold

- **The 32 unkeyed openings,** each arm's hits.
- **Fresh V2,** each arm's hits with the memory on and off. Fresh V2 was opened once for M1 bar (e) and is never a decision stratum here or later.
- **64 greedy samples per arm,** nearest-decoded, 64 tokens each, after `sample_windows`' 64 DEV prompts, with the memory on and off. They are scored against TRAIN's replies:
  - the share of complete replies that are verbatim TRAIN replies;
  - new n-grams for n = 4–8;
  - distinct-1 and distinct-2;
  - the repeated 4-gram rate.

  DEV's own replies after the same prompts are scored the same way, as the anchor.
- **The writer's picks** for every row, beside the nearest picks. With the memory off, they and the memory-off samples are the float reference for `integer-hamming-read` step 5's top-1 agreement and first divergence. They are sealed here, not read here.

## Instrument checks

- **I-1.** The crate's CUDA tests pass on the pod's build: `cargo test --release --locked -p uor-r4-float-train --features cuda --lib -- --test-threads=1`, every test passed, with the new ones among them.
- **I-2: the readings reproduce.** Each arm's DEV nearest hits must equal claude-194's sealed last-checkpoint readings (its REPORT's "memory taken out" column):

  | Arm | Memory on | Memory taken out |
  |---|---|---|
  | (r), seed 1 | 11,513 | 4,101 |
  | (r), seed 2 | 11,361 | 3,807 |
  | (r0), seed 1 | 10,079 | 10,079 |
  | (r0), seed 2 | 11,171 | 11,171 |

  Exact equality is expected: the same checkpoint, the same code path (`row_picks` is `nearest_reading`'s loop), and the same flags. If any differs, the REPORT gives the difference, and the cell is read only if every difference is at most 14 rows (0.1% of DEV), classed as GPU float order across hosts. Past that, the instrument is repaired before any cell is read (L-007).
- **I-3: the memory names no decision row.** For every arm, the number of decision rows picked differently with the memory taken out (`decision_pick_differences`) is 0. A non-zero count is classified before the cell is read: either the census's rows are not the memory's, or the memory reaches rows it does not name.
- **I-4: the rows are the census's.** The grader's decision rows number 3,438 (3,406 + 32), and every arm's files score the same 14,287 rows in the same order (`paired` refuses otherwise).
- **I-5: the inputs are the sealed ones.** Every checkpoint file passes `sha256sum -c` against its seed's sealed `SHA256SUMS` (above), and the data and Fresh V2 pass the code's pins.

## The probe ladder

- **(a) Synthetic, done on the laptop at `df7675b31`.**
  - The crate's tests: 134 lib tests passed, 0 failed, the new ones among them.
  - The grader run on fake score files built from the census's decision positions. All-hit (r) against all-miss (r0) gives 3,438 rows, `b` 3,438, `c` 0, band 118 and "ahead" at both seeds, in 1.65 s and 0.93 GB.
  - A stub-nvcc `cargo check --features cuda` of the crate and the binary builds.
- **(b) Throughput.** The first arm's scoring time, from the job's step timestamps, projects the rest. claude-194's decode ran at about 112–176 tok/s, so 64 samples × 64 tokens × 2 passes is about 1 minute per arm, and the four DEV and Fresh V2 passes are seconds. If the first arm takes more than 15 minutes, the job stops and the code is fixed.
- **(c) The run:** the four arms, one after another, on one GPU.

## Cost, lease and stop rule

- **One RTX 5090 on Vast,** through `scripts/pod/uor-pod up --lab claude --session claude-205 --count 1 --gpu 5090 --hours 2`, at about $0.5–1.0/h. It sits inside the caps (at most 4 pods and $8/h across labs; `uor-pod status` is read first).
- **Projected time:** about 45 minutes. That is about 10 for the bootstrap and source pin, about 10 for I-1's tests, about 3 for the build, and about 4 × 3 for the arms, so about $1.
- **Stop** if the pod passes $3, or projects past 1 hour, or I-1 fails.
- **Watching:** a completion watcher is armed in the same turn the job starts, and the lease is kept by `uor-harness loop drive --watch --lease --interval 300`.
- **The output** goes off the pod first, then to `results/claude/one-model-end-to-end-2026-10-29/` with a `SHA256SUMS`, and is read back. Then the pod is released and `down`.

## After the run

- **Grading on the laptop:** `train-in-space --data-root ROOT --overturn 1 --scored r-s1=DIR --scored r0-s1=DIR --scored r-s2=DIR --scored r0-s2=DIR`, one JSON line, committed under `docs/labs/one-model-end-to-end/`.
- **The REPORT** leads with tok/s where it has one, gives the negatives first, and adds one notebook row.
