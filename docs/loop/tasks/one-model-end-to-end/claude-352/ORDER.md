# ORDER: one-model-end-to-end, claude-352

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Base: `93e089ff256654e75a19641b7e2fbd2bc8e24524`.

## Goal

**Run stage 3 of the place read (arm p) as its sheet registers it: one pod, four jobs, then the laptop readings, with the model's replies shown word for word.**

The sheet is [one-model-end-to-end-place-read-2026-11-11](../../../../harness/prep/one-model-end-to-end-place-read-2026-11-11.md), FIT on its fourth read. Its bars, rows, checks and stop rule are not restated or changed here.

**Why now.** [claude-351](../claude-351/REPORT.md) built the place read and merged it with nothing trained. On 11 Nov 2026 (UTC), in chat, the lab asked "Run stage 3 of the place read on one pod?" and the owner answered **"fine run it i guess..."**. That approves this run: one pod at no more than $1.60/h, four jobs, a ceiling of $16. It approves no other arm.

**The question.** Does a model trained with the place read say a stated value back when no memory line is written for it?

## What runs

The pin is `93e089ff256654e75a19641b7e2fbd2bc8e24524`, claude-351's merge commit.

| Step | Where | What |
|---|---|---|
| 1 | laptop | The reader prints the register (`raw-id-replies`), with I-l's test. |
| 2 | pod | I-p, I-g, I-3, then the two 64-step smokes with I-s, I-r and I-b. |
| 3 | pod | p-s2, n-s2, p-s1, p-s3, in that order, each with its scoring pass and each sealed on exit. |
| 4 | pod | The pod's four checks, then the pod goes down. |
| 5 | laptop | The exports with I-o, I-c and I-t; the readings; I-f; the split; the rows. |
| 6 | laptop | Only if p-s2 reads row 2 or row 4: the hand read, and I-w with one retake if it calls one. |

## Decisions of this unit (claude's own decision review; logged for Casey, who can reverse them)

1. **Step 1 runs beside step 2.** The sheet lists the reader's change before the pod. I-l gates the split and the printed register, which are laptop readings, and row 0 says a failed laptop-side check is repaired and the readings retaken from the sealed checkpoint. So the pod does not wait for the reader. No reading that needs the register is taken before I-l passes.
2. **I-b's form.** On each smoke checkpoint the pod runs `fbs-snapshot` as arm r's job took it, `fbs-export --legacy-artifact --raw-gates --raw-transition --run-config`, then `fbs-snapshot` with its three float readouts and `fbs-compare --parity-only`. The check passes when the snapshot and the export exit 0 on both smokes, p's artifact holds a `hamming.d*.place` section and n's holds none, and the readouts and the compare end on p with the same exits as on n. n is `main`'s behaviour, which is the sheet's "as on `main`". `--legacy-artifact` is how arm r's artifacts were written, before the contract binding of schema 10. (Amended before the pod went up: the first wording asked all three programs to exit 0. Whether the readout form runs on a model with these reads on `main` was not known, so n decides.)
3. **The pre-run review** (CHARTER §5.10) is a fresh reader over the pod kit against the pin and the sheet, before the pod goes up. Its verdict goes in the REPORT.
4. **A failed job does not stop the jobs after it.** A job that ends non-zero is sealed and is NOT_RUN; the chain goes on to the next. Only step 2 stops the chain, as the sheet says (I-g, I-3, I-s, I-r, I-b).
5. **The full-size scoring pass also runs once on p's smoke checkpoint.** It writes the panel's prompts at context 256 early, so the laptop can check that all 562 recall replies are among them (I-l's panel half) while the long runs train. Its replies are never read.

7. **The scoring pass runs at context 192, not the sheet's 256 (decided 11 Nov 21:10Z, after try-1; training stays at 256).** try-1's first stage passed I-g (21 tests, 0 failed), I-3, I-s, I-r (1,946 s projected) and I-b, then ended on its last check, exit 5: the pinned scorer refuses `--score-rows --context 256` by name (`row_stride_window`, claude-275: from 193 to 1,536 positions several windows share one scoring forward and the row stride over them has no test). The sheet registered a scoring command the pin cannot run; nobody had run it, and this is the check decision 5 was made for. It is an instrument failure, row 0 for try-1's first stage, repaired under a new root (try-2 on the same pod). The repair uses the scorer's tested layout, 192, arm r's own, and changes no pinned code. Its effect is bounded and counted: a prompt is a conversation from its start, cut only when longer than the context, and on arm r's panel 3 of 1,141 prompts reached 192. The REPORT counts the recall replies whose prompt is cut at 192; they stay in the 562, which can only lower R of arm p. The float against integer check (I-f) reads the same prompts on both sides, so it is not affected. Scoring past 1,536 (one window a forward, also tested) was set aside: 16 times the forwards would not fit the pass's 40 minutes.

8. **Decision 7's bound was the wrong measure; the cut is counted before any reading (decided 11 Nov 21:50Z, while p-s2 trains and before any trained model is read).** A scored prompt starts at its scoring window's start, not at the conversation's, so a prompt is cut long before it reaches 192 ids: the window must also hold the reply. I-l's panel half, run on try-2's smoke checkpoint, found a true place for 449 of the 562 recall replies; for the other 113 the stated value lies before the prompt's first id (their prompts are 181 to 187 ids long). claude-350's census had said so (labelled places within 192: 395 of 490; within 256: 490 of 490), and that is why the sheet registered 256. So at 192: (a) the 113 stay in the 562, which can only lower R of arm p and G, and they are also counted apart, with R over the 449 beside R over 562; (b) G's bar is read over 562, as registered; (c) I-l's panel half is not met (449 of 562), so the split and the printed register are read on the 449 only; (d) if G misses its bar over 562, no row is called until the 562 prompts are rebuilt at the 256 window on the laptop and R is retaken from the sealed checkpoints, which is the sheet's repair for a failed laptop-side check. L and the two conversations are not touched by this: the chat feeds the whole conversation.

6. **A sealed failure holds the pod for an hour.** If a stage ends non-zero and every stage that ran is sealed in the store, the watcher holds the pod for 60 minutes, so the session can read it or rerun under the next try on the same pod, then lets the lease run out. Each try has its own root on the pod and in the store (L-002).

## What a pass is

A pass of this unit is the run completed and read as the sheet says, whatever row it gives.

| # | Gate | Check |
|---|---|---|
| G1 | Step 2 passed, or the run stopped there as row 0. | the sealed `pre` folder |
| G2 | Each of the four jobs is sealed in the store and read back, and I-t holds or the job is NOT_RUN with its reason. | the store's sums against the pod's |
| G3 | The pod is down after its four checks. | `uor-pod status` |
| G4 | The readings of the sheet's "Readings" section are in the REPORT, with L and the two conversations word for word. | the REPORT |
| G5 | The row of p-s2, and of seeds 1 and 3, is read from the top of the sheet's table, after the instrument checks. | the REPORT |
| G6 | `uor-harness ratchet d23`, `d25`, `d32`: no additions. | the three commands |

## Not established by this unit

- Nothing about open text, reworded questions or conversations longer than the context.
- The 126 recall replies the find cannot reach stay out of reach.
- G compares one seed of p with one control seed.

## Cancel probe (run first)

- **What would make this unit unnecessary:** a sealed run of arm p in the store.
- **Result (at claim):** none. `uor-store ls results/claude/` holds no `one-model-end-to-end-place-read-2026-11-11` folder.

## Claim

Written by hand, as claude-347's to claude-351's were: claude-346 (PR #3913, a draft under the hold) holds the item for `uor-harness loop claim`.

## Files in scope

- `docs/loop/tasks/one-model-end-to-end/claude-352/**`
- `docs/labs/one-model-end-to-end/instrument/**` (the reader's register print and its test)
- `crates/uor-r4-integer/src/bin/uor-chat.rs` (only if the scripts' printed register needs it)
- `crates/uor-r4-integer/src/stack/chat.rs` and `crates/uor-r4-integer/src/stack/tests.rs` (added 11 Nov, before any reading: the chat hides its session, so the print `--print-place` needs one accessor for the register's pairs of the last reply, and its test; print-only, no reply changes)
- `docs/research/notebook/**` (one row a result)

## Inputs

- TRAIN without a line and the no-line panel, in the store under `results/claude/one-model-end-to-end-memory-line-2026-11-10/corpus-noline/` and `panel-noline/`. TRAIN's manifest in the store equals the laptop's byte for byte.
- The kit, `local/claude/claude-351/train-places.json`, SHA-256 `8d28571a614e4e038fcb52e2661787f38af62933dbf7ddad2ad64582c8a0a9b3`, put in the store beside the run.
- Arm r's sealed seed 2 run, from the store, for I-o and I-c.
