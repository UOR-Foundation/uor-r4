# REPORT: turn-memory-reply, claude-194 (the reply table, trained)

Lab: claude. Order: [ORDER.md](ORDER.md). Sheet: [turn-memory-reply-2026-10-29](../../../../harness/prep/turn-memory-reply-2026-10-29.md) (TEST FITNESS: FIT, PRE-RUN: PASS, round 2). Code: `fc66ed745`, built and tested on each pod. Closes #3393.

**Result: cell A at both seeds.** The reply table carries the reply past its first word: the later rows carry, the openings and copies hold, and the memory beats no memory.

**Speed.** Decode is the float trainer's greedy decode on the GPU (`--decode-speed`, 64 tokens). It is not the integer serving path, which serves neither table yet. At 192 context tokens, (r) decodes at 176 / 112 tok/s (seed 1 / seed 2) against (m)'s 179 / 118 and (r0)'s 175 / 112. The two seeds ran on different hosts, so only arms within a seed compare.

## What did not work, first

- **The opening memory alone falls below no memory on the totals, at both seeds.** The empty control (r0) beats claude-161's (m) on `T̄`: 9,993.3 against 8,905.7 at seed 1, and 11,156.3 against 10,471.3 at seed 2.
  - The cause is copies. (m) copies 1,209.3 / 2,404.7 rows against (r0)'s 2,282.3 / 3,315.0 (seed 1 / seed 2).
  - The memory still lifts openings, from (r0)'s 260.3 / 344.7 to (m)'s 597.3 / 598.0.
  - So on today's code, the opening memory trades copies for openings. The pre-registered bar compares (r) with (m) and with (r0), and (r) clears both. Read against (m) alone, however, the gains would look larger than they are.
- **Against no memory, (r)'s gain at seed 2 is thin.** It is `T̄(r) − T̄(r0)` = 258.7 rows against a margin of 148.4. At seed 1 the gain is 1,533.0 rows, against 164.4.
- **The trained trunk now depends on the memory.** Taking the whole memory out of (r) at the final checkpoint (`--inspect`'s `memory_drop`) drops it to 4,101 / 3,807 hits. That is far below the empty control's 10,079 / 11,171. A model served without both tables would be broken, not merely weaker.
- **Reach does not grow in the sense of the earlier bars.**
  - (r)'s `R16` (the mean shift at distance 16 over the shift at distance 0) is 0.133 / 0.275, against (m)'s 0.015 / 0.030 and (r0)'s 0.023 / 0.025.
  - Most of that is the denominator. With the memory moving the prediction toward a stored token, the shift at distance 0 falls from 3.90 / 4.27 to 0.81 / 0.58.
  - The shift at distance 16 itself rises only 1.8× / 1.2×.
- **The sheet's greedy samples were not produced.** None of the job's commands prints them, so the REPORT has none.
- **The first seed-1 host never answered SSH,** and `uor-pod` deleted it. Seed 1 ran on a second host, started four minutes after seed 2.

## What changed

- **No code.** The run uses claude-193's merged read at this unit's head, `fc66ed745`.
- **This unit adds:**
  - the sheet ([`turn-memory-reply-2026-10-29.md`](../../../../harness/prep/turn-memory-reply-2026-10-29.md)), with its FIT and pre-run review;
  - this REPORT;
  - one notebook row (`docs/research/notebook/float-training.md`);
  - the item's `status: done`, re-rendered into `docs/roadmap/README.md`.
- **The job and its watcher** are `local/claude/claude-194/job.sh` and `watch.sh` on the laptop. They are not committed (no ad-hoc scripts in the repository).

## Measurements

### The bar (pre-registered)

`X̄` is the mean over steps 12,288, 14,336 and 16,384. `m_n(X̄) = 3·√(X̄ (1 − X̄/n))`. Computed with `bc` from the sealed `progress.jsonl` (`local/claude/claude-194/readings.md`).

| Condition | Seed 1 | Seed 2 |
|---|---|---|
| `R̄(r) ≥ R̄(m) + m_9094` | 7,908.7 ≥ 7,291.1 (m 116.7) | 7,889.7 ≥ 7,651.3 (m 107.6) |
| `T̄(r) ≥ T̄(m) + m_14287` | 11,526.3 ≥ 9,079.4 (m 173.8) | 11,415.0 ≥ 10,630.0 (m 158.7) |
| openings hold, `Ō(r) ≥ Ō(m) − m_1226` | 612.3 ≥ 544.8 | 610.0 ≥ 545.5 |
| copies hold, `C̄(r) ≥ C̄(m) − m_4130` | 3,093.7 ≥ 1,121.6 | 3,001.3 ≥ 2,309.6 |
| memory against no memory, `T̄(r) ≥ T̄(r0) + m_14287` | 11,526.3 ≥ 10,157.8 (m 164.4) | 11,415.0 ≥ 11,304.7 (m 148.4) |
| **Cell** | **A** | **A** |

### Readings

DEV: 14,287 rows. `T` is hits; `O` is opening hits of 1,226; `C` is copied hits of 4,130; `R` is rest hits of 9,094. Each is the mean over the three bar checkpoints.

| Arm | Seed | T̄ | Ō | C̄ | R̄ | memory taken out (T) | reply weight `|w|` |
|---|---|---|---|---|---|---|---|
| (m) opening memory | 1 | 8,905.7 | 597.3 | 1,209.3 | 7,174.3 | 8,436 (from 9,033) | none |
| (m) | 2 | 10,471.3 | 598.0 | 2,404.7 | 7,543.7 | 9,992 (from 10,480) | none |
| (r) reply table | 1 | 11,526.3 | 612.3 | 3,093.7 | 7,908.7 | 4,101 (from 11,513) | 0.990 |
| (r) | 2 | 11,415.0 | 610.0 | 3,001.3 | 7,889.7 | 3,807 (from 11,361) | 0.992 |
| (r0) empty | 1 | 9,993.3 | 260.3 | 2,282.3 | 7,552.0 | 10,079 (unchanged) | 0.0 |
| (r0) | 2 | 11,156.3 | 344.7 | 3,315.0 | 7,618.3 | 11,171 (unchanged) | 0.0 |

The per-checkpoint readings are in `local/claude/claude-194/readings.md` and in the sealed `progress.jsonl` files.

### Instrument checks (all pass, at both seeds)

- **I-1:** the crate's CUDA tests: 177 passed, 0 failed, 6 ignored, the memory's three included.
- **I-2:** at step 0, every arm equals claude-152's (a): seed 1 hits 2, opening hits 0, 730 distinct picks; seed 2 hits 5, 3, 681. `config.json` is width 256, 4 heads, memory 8 and the pinned TRAIN.
- **I-3:**
  - **opening table:** 26,450 keys for (m) and (r), 0 for (r0);
  - **reply table, on the probe log and the training log:** it reads exactly 2,212,957 TRAIN rows past their openings. For (r) it holds 573,801 keys (more than 0 and fewer than the probe's 598,755) and names exactly 9,655 DEV rows. For (r0) it holds 0 keys and names 0;
  - **weights:** (r)'s reply weight is 0.990 / 0.992 at the end; (r0)'s stays 0.0.
- **I-4:** all 17 losses finite in every arm.
- **I-5:** removing the memory changes (m)'s and (r)'s readings, and leaves (r0)'s unchanged.

## Checks run at the head

- **On each pod, at `fc66ed745`:**
  - the source was pinned to that commit (the job's `source-pin` step);
  - the trainer was built with `--locked` and its sha256 recorded (`binary-sha256`);
  - the crate's CUDA tests passed (I-1).
- **On the laptop, at this PR's head:** `uor-harness loop report --check` and `loop pr-check --local`. Their results are on the PR.

### Cost and seal

- **Pods:** two RTX 5090 pods on Vast, about $2.5 in all:
  - seed 2 on `vast-58276485`, from about 16:36Z to 18:17Z, at about $0.98/h;
  - seed 1 on `vast-58277706`, from 16:43Z to 18:10Z, at $0.563/h.

  Each ran the CUDA tests, then (m), (r) and (r0), every step exiting 0.
- **Sealed** from each pod to the HF store, with a `SHA256SUMS`:
  - `results/claude/turn-memory-reply-2026-10-29/s1-fc66ed74`: 263 files, 667.7 MB, `SHA256SUMS` sha256 `8d40536e162866d0944e85fe8a476042b266f722a1256a91d87001c945017625`;
  - `results/claude/turn-memory-reply-2026-10-29/s2-fc66ed74`: 263 files, 667.7 MB, `e4e5a5f53187e3cec14f20ca193fbb0e5944e92a28a2315b36835bf55204cef2`.
- **Read back on the laptop before `down`:**
  - seed 1: 13 of 13 logs and readings;
  - seed 2: the same 13, plus `r-s2/models/step-016384/model.safetensors`.

  Every file checked against its `SHA256SUMS`. The seed-2 upload log shows 0 B transferred for its large files: the reply tables are built from TRAIN alone, so they are identical across seeds and were deduplicated. The file read back above confirms the content.
- **Both pods** were released and deleted, with `--confirm-archived` after the store listing. 0 pods remain.

## What it unlocks

Cell A's line on the sheet: the memory carries the reply. Next, `turn-memory-integer` serves both tables in the integer runtime (step 2 of the owner's order), and then the end-to-end model scores typed replies. The dependence above makes that serving necessary, not optional: the trained trunk does not work without the memory.

## Skills used

- None was invoked this unit. The bar, the cells and the stop rule were fixed in the sheet before the run, and applied as written.

## Still open

- **`turn-memory-integer`:** serve both tables in the integer runtime. The next step, and a necessary one, since the trunk falls apart without them.
- **The opening memory against no memory.** On today's code, (m) is below (r0) on totals at both seeds, through copies. Whether the opening table should keep its weight, or act only where the reply table does not, is a question for the next decision on the memory, not this item.
- **The greedy samples:** the trainer prints none in these modes. A later run that wants them needs the decode to print them.

## Plain summary

The model now looks up, in an exact table built from the training conversations, what usually comes next at each word of a reply, not only at its first word. On held-out conversations that adds several hundred correct next words at both seeds, against the old memory and against no memory at all. The model has learned to lean on the table, so the integer runtime must serve it next.
