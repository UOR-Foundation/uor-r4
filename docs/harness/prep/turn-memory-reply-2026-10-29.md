# Prep sheet: the turn memory's reply table, trained (turn-memory-reply, claude-194, 29 Oct 2026)

Lab: claude. Item: [turn-memory-reply](../../roadmap/items/turn-memory-reply.md), done-when 3. Order: [claude-194](../../loop/tasks/turn-memory-reply/claude-194/ORDER.md). Code: this unit's head, built and tested on each pod from that exact commit.

TEST FITNESS: FIT, round 2. PRE-RUN: PASS. Both by one fresh non-author reviewer, at `ebc683ea8`. Round 1 (NOT FIT, FAIL) found that the reply table's exact key count was the probe's, which also counts position-0 opening keys; its three changes are applied above (I-3, the probe-log checks, the no-pass cells). Its one remaining note: that the 212,968 TRAIN openings are the rows `start_memory` drops. They are, by construction: `opening_rows` and `reply_rows` mark an opening by the same `opens_reply` test over the same `build_document_windows` of the pinned TRAIN.

## The question

**Does the reply table carry a reply past its first word?**
- **The diagnosis:** the trunk routes, and exact memory stores (claude-155).
- **The opening memory:** claude-161's memory moved only the first word. It took reply openings to 598.7 / 598.3 of 1,226, but left the rest of each reply to the trunk's templates.
- **The probe (claude-192):** for the rows past the opening, a table keyed by the turn's first 8 ids, the reply position and the last reply id, kept at stored share ≥ 0.5, names 9,655 of DEV's 13,061 rows past their openings and gets 8,822 right (0.914). That is a count, not a model.
- **The read (claude-193):** it builds that table into the memory, `--turn-memory-reply 1`.

This run measures whether it adds to the model.

## The read

The turn memory ([`flock_read_memory.rs`](../../../crates/uor-r4-float-train/src/flock_read_memory.rs)) has two tables. Both move the readout point `n` toward a stored token's table point `c` (detached), `n + w ⊙ (c − n)`, each with its own per-coordinate weight starting at 0. Every other row is unchanged.
- **The opening table** (claude-161, unchanged): at a row that predicts a reply's first token, keyed by the turn's first 8 ids. About 26,450 keys.
- **The reply table** (claude-193): at every row past the opening, keyed by `turn-8 + position + reply-last-1`, the probe's key exactly, at share ≥ 0.5. It is built from TRAIN's 2,212,957 rows past their openings (2,425,925 reply rows less 212,968 openings). It holds somewhat fewer than the probe's 598,755 keys, which also count the position-0 opening keys that pass share 0.5. Opening keys and later keys never collide, so on DEV it names exactly the probe's 9,655 rows past their openings.

## Steps: the arms

Each arm runs `train-in-space --objective cell --hamming-read 4 --retain-rates 1 --decay-mult 1 --batch 16 --steps 16384 --eval-every 2048 --lr 0.001 --lr-end 0 --seed S` on the pinned TRAIN, at width 256 and seeds 1 and 2. That is claude-161's (m) recipe, with these flags:

| Arm | Extra flags | Role |
|---|---|---|
| (m) opening memory | `--turn-memory 8` | claude-161's (m), re-run on today's code |
| (r) reply table | `--turn-memory 8 --turn-memory-reply 1` | the read |
| (r0) empty | `--turn-memory 8 --turn-memory-reply 1 --turn-memory-empty 1` | the same code with both tables empty: the no-memory control (L-010) |

**Why (m) is re-run and not read from its REPORT.** claude-161 trained before the fixed-order backward (claude-179, 29 Oct). Single runs of the old backward spread widely (claude-140). Comparing a new (r) with an old (m) would mix two code versions. All three arms here run the same binary at the same seeds, deterministically.

## Readings

At each checkpoint, `nearest_reading` reads DEV's 14,287 rows. The memory is on, at the rows the run names, exactly as in training. Kinds overlap: 163 rows both open a reply and copy the prompt. The readings:
- **`T`:** hits of 14,287.
- **`O`:** opening hits of 1,226.
- **`C`:** copied hits of 4,130.
- **`R`:** rest hits of 9,094.

`X̄` is the mean over the checkpoints at steps 12,288, 14,336 and 16,384. Write `m_n(X̄) = 3·√(n·p(1−p))`, with `p = X̄/n` and `n` the kind's row count.

## Bar (pre-registered)

At **both** seeds:

- **The later rows carry:**
  - `R̄(r) ≥ R̄(m) + m_9094(R̄(m))`, **and**
  - `T̄(r) ≥ T̄(m) + m_14287(T̄(m))`.
- **The openings hold:** `Ō(r) ≥ Ō(m) − m_1226(Ō(m))`. A miss is flag O.
- **The copies hold:** `C̄(r) ≥ C̄(m) − m_4130(C̄(m))`. A miss is flag K.
- **The memory against no memory** (L-010): `T̄(r) ≥ T̄(r0) + m_14287(T̄(r0))`.

| | No flag | Flag O or K |
|---|---|---|
| **Later rows carry, and the memory beats no memory** | **A.** The reply table carries the reply past its first word. | **B.** It does, at a cost on openings or copies. |
| **Later rows do not carry** | **C.** If `T̄(r)` is within `m` of `T̄(m)`: the lookup does not add to the trunk on these rows. | **D.** If `T̄(r) < T̄(m) − m`: it hurts. |

A or B needs "the later rows carry" (both of its parts) and "the memory against no memory", all at both seeds. **Every other combination is no pass:**
- `R̄` carries but `T̄(r)` misses `T̄(m) + m`;
- `T̄(r) ≥ T̄(m) + m` but `R̄` misses;
- the later rows carry but the memory against no memory fails;
- a pass at one seed only.

Each is reported as no pass, with its readings. C and D name only the cases their own conditions describe; a no pass outside both is reported as "no pass, no cell".

**Readings, not bars,** reported for every arm and seed:
- the reply weight's mean magnitude;
- the reply table's DEV rows named (I-3);
- the removal reading (I-5);
- `R16` from `--reach`;
- decode tok/s from `--decode-speed`;
- the greedy samples.

## Instrument checks

- **I-1.** The crate's CUDA tests pass on each pod's build. Among them, the memory's three:
  - `zero_weight_is_no_read_and_the_move_is_the_stored_points_pull`;
  - `zero_reply_weight_is_no_read_and_the_reply_move_is_its_pull`;
  - `a_saved_memory_loads_with_its_table_and_weight`.

  The test count is recorded.
- **I-2.** At step 0, before any update, each arm's DEV reading equals claude-152's (a) at its seed, since both weights are 0:
  - seed 1: hits 2, opening hits 0, 730 distinct opening picks;
  - seed 2: hits 5, opening hits 3, 681.

  `config.json` holds width 256, 4 Hamming heads, `turn_memory` 8, and the pinned TRAIN. It holds `turn_memory_reply` for (r) and (r0), and `turn_memory_empty` for (r0) only.
- **I-3.**
  - **The opening table:** about 26,450 keys for (m) and (r), and 0 for (r0).
  - **The reply table,** checked on each arm's 256-step probe log before it trains, and again on its training log:
    - for (r) and (r0), it reads exactly **2,212,957** TRAIN rows past their openings;
    - for (r), it holds more than 0 and fewer than 598,755 keys. That is the probe's count, which includes the position-0 opening keys. It names exactly **9,655** of DEV's rows past their openings, the probe's count;
    - for (r0), it holds 0 keys and names 0 rows;
    - (m) has none.
  - **The weights:** (r)'s reply weight is non-zero at the end, and (r0)'s stays 0.
- **I-4.** Every loss is finite.
- **I-5.** The removal reading: at the final checkpoint, `--inspect`'s `memory_drop` re-reads DEV with the whole memory taken out. For (m) and (r) that must change the reading. For (r0) it must change nothing.

## Data split

- **TRAIN:** the pinned `mw-balp` (`PINNED_TRAIN_*`), checked by sha256 at load. Both tables are built from it alone.
- **DEV:** the pinned DEV, read at every checkpoint. No DEV row enters either table.
- **Scope:**
  - 1,194 of DEV's 1,226 openings have their first-8 prompt prefix in TRAIN (claude-192), so DEV's turns are not new to TRAIN.
  - The keys see the window, not the whole turn: in a later window, the turn part is short or empty, and the position is window-relative (claude-192, claude-193).

  A pass is a pass on this panel and these windows.

## Base artifact

None. Each arm starts from its seed's initialization, as (a) did.

## Minimum test power

- **At the expected levels:**
  - `R̄ ≈ 7,300` of 9,094 (`p ≈ 0.80`) gives `m ≈ 114` rows;
  - `T̄ ≈ 9,900` of 14,287 (`p ≈ 0.69`) gives `m ≈ 166`.
- **What the probe suggests:** on the 8,169 rest rows it names, the table is right 96% of the time, against the trunk's about 80% on all rest rows (claude-161). A gain of several hundred rest rows is possible.
- **Below the margin:** a gain inside the margin does not pass.

## Cost

- **Pods:** two RTX 5090 pods on Vast, at about $1.11/h each. Each runs one seed: the CUDA tests, then (m), (r) and (r0) in turn, each after its 256-step probe.
- **Time:** each arm is 16,384 steps of 16 × 192 tokens. At claude-161's 31,851 context tokens/s that is about 26 minutes, plus reach, inspect and decode. That is about 2.5 hours a pod, and up to about 6.5 at the throughput gate's limit.
- **The total:** about $5 to $15.
- **Throughput:** each arm's probe must project it under 2 hours (`≥ 6,991` context tokens/s), or the job stops.
- **Lease:** 8 hours, renewed rather than cut if a healthy run needs longer, inside the caps: 2 of 4 pods, about $2.2/h of $8/h.
- **Results:** sealed to `results/claude/turn-memory-reply-2026-10-29/` and read back before `down`.

## Stop rule

- **I-1 or I-2 fails:** stop. The read is not off at weight 0, or the build is not the head's.
- **Any of I-3's reply-table checks fails,** on the probe's log or the training log: stop. Check the instrument before any reading.
  - TRAIN rows other than 2,212,957: the table is not built from the right rows.
  - Keys outside the bound: the share rule is not the probe's.
  - A DEV count other than 9,655: the run's key is not the probe's.
- **A probe projects past 2 hours:** stop and profile.
- **The pod fails or the lease would pass the caps:** stop, seal what exists, and report.

## What each result unlocks

- **A:**
  - the memory carries the reply;
  - next, `turn-memory-integer` serves both tables (step 2 of the owner's order);
  - then the end-to-end model scores typed replies.
- **B:** the move's weight or its rows are narrowed, for example a higher share for the reply table, before it is served.
- **C:**
  - the lookup does not add where the trunk already reads;
  - the next decision is the key: the slot-address key, or a share chosen on TRAIN alone;
  - the removal reading says whether the move was used.
- **D:** the configuration closes (CHARTER §4.1), and the instrument is checked first.
