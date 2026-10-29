# REPORT: one-model-end-to-end, claude-205 (the overturn check)

Lab: claude. Order: [ORDER.md](ORDER.md). Sheet: [one-model-end-to-end-overturn-2026-10-29](../../../../harness/prep/one-model-end-to-end-overturn-2026-10-29.md) (TEST FITNESS: FIT, PRE-RUN: PASS, round 2). Code on the pod: `b6f7bdc9256ec2466dacde789a28885b24676164`. Binary sha256: `31c0d2df9ddb94105b0f6dedf91399f141965ef1061b4bdf73e3670e4d7708a1`. Grading: [overturn-2026-10-29.json](../../../../labs/one-model-end-to-end/overturn-2026-10-29.json). References #3407; done-when 2 is met, and the item stays `ready` for done-when 3 and 4.

**Result: mixed.** The rows in question are DEV's 3,438 rows that the turn memory names none of.
- **Seed 1:** the trunk trained with the memory, (r), is ahead of the empty-table control (r0) by 548 rows.
- **Seed 2:** it is behind by 367.
- **Both seeds are past their paired floors** (77 and 61), but in opposite directions. So the two-seed spread, 915, sets the band, and both seeds are inside it.
- **The rule fixed in advance says:** no default changes. The training run carries (r) and a memory-dropout arm as equal candidates, and the protocol (done-when 3) chooses between them.

**Speed.** This unit times nothing. Each arm's scoring on one RTX 5090 took 72–77 s. That covers DEV and Fresh V2, with the memory on and off, and 128 greedy replies of 64 tokens; the run's own decode rate is not separated out. claude-194's decode reading stands: (r) 176 / 112 tok/s at 192 context, from the float trainer's greedy decode.

## What did not work, first

- **Two seeds cannot say which way co-adaptation goes.** The paired difference changes sign between seeds, and each seed's difference is about seven and six times its own paired floor. At this scale, the trunk's reading of the rows the memory is silent on depends on the seed more than on the memory. A decision this narrow needs more seeds per arm. The training run should carry at least three, so that the band rests on more than one spread.
- **(r) without its memory cannot finish a reply.** It finishes 1 and 0 of 64 greedy replies within 64 tokens, against 44 and 41 with the memory and 63 and 56 for (r0). Its repeated-4-gram rate is 0.692 / 0.770. claude-194's totals (4,101 / 3,807 hits without the memory) said the same thing under teacher forcing.
- **(r) loops even with its memory.**
  - Its repeated-4-gram rate is 0.435 / 0.547. DEV's own replies after the same prompts score 0.000; (r0) scores 0.112 / 0.500.
  - Its high new-n-gram share (0.758 / 0.785 at n = 4, against DEV's 0.328) is mostly those loops, not new phrasing. That measure has to be read beside the repeat rate.
- **Neither trunk copies rare values from the prompt.** Seed 1's first 6 of 64 samples, prompt by prompt, with the same prompt in each row (`sample_windows` is deterministic). The text is in each arm's `samples-memory.json`, cut here after the first `<|eos|>`:

  | True reply | (r) | (r0) |
  |---|---|---|
  | "Got it, I'll remember them." | "Got it." | **exact** |
  | "Got it, you grew up in Hirkdoulhaven." | echoes the user's line, in a loop: "I come from Hirkdoulhaven. Assistant: I come from …" | "I'm glad you feel happy!" |
  | "Got it, your dog is Kleelkouk." | "Got it, your dog is Kleeres bTh lock …" (no end) | "Got it, you grew up in Ttay r How." |
  | "The day after Monday is Tuesday." | "It's bicy lizard V W waterme …" (no end) | "Which one do you mean?" |
  | "Okay, I'll remember all of that." | **exact** | "Got it, I'll remember them." |
  | "Okay, I'll remember all of that." | "Got it." | "Okay, I'll remember that." |

  - **What each gets.** Each trunk gets one of the six verbatim, and both produce the generator's acknowledgement templates. (r) reaches the right slot more often ("your dog is …"), then breaks inside the value and runs on. (r0) ends cleanly, but on the wrong template or the wrong value.
  - **What neither does.** Neither reads a rare value out of the prompt, or answers the one question.
- **The served writer decodes far below the nearest readout.** Of DEV's 14,287 rows with the memory on:

  | | (r), seed 1 | (r0), seed 1 | (r), seed 2 | (r0), seed 2 |
  |---|---|---|---|---|
  | Writer's pick right | 674 | 342 | 77 | 1,280 |
  | Nearest token right | 11,513 | 10,079 | 11,361 | 11,171 |

  These checkpoints were trained with `--objective cell`, a nearest objective. The integer session serves the writer today, so the readout must be served nearest before an integer end-to-end score means anything.
- **No arm names an unkeyed opening.** Of the 32, (r) gets 0 and 0, and (r0) gets 1 and 3.
- **The read's parity did not run here.** By the decision review (#3407, 6509925608), kept sets and top-1 stay `integer-hamming-read` step 5's. This run sealed the memory-off float picks and samples it may use.

## Measurements

### The decision (pre-registered)

`b`: rows only (r) gets. `c`: rows only (r0) gets. `d = b − c`. Floor: `⌈2·√(b+c)⌉`. Band: `max(floor, |d(1) − d(2)|)`.

| Seed | (r) hits | (r0) hits | b | c | d | Floor | Band | Cell |
|---|---|---|---|---|---|---|---|---|
| 1 | 2,078 | 1,530 | 999 | 451 | +548 | 77 | 915 | inside |
| 2 | 1,929 | 2,296 | 273 | 640 | −367 | 61 | 915 | inside |
| **Verdict** | | | | | | | | **mixed** |

### Readings, every arm (nearest top-1)

| Arm | DEV, memory on | DEV, memory out | Decision rows (of 3,438) | Unkeyed openings (of 32) | Fresh V2 on / out (of 14,479) |
|---|---|---|---|---|---|
| (r), seed 1 | 11,513 | 4,101 | 2,078 | 0 | 11,653 / 3,892 |
| (r0), seed 1 | 10,079 | 10,079 | 1,530 | 1 | 10,295 / 10,295 |
| (r), seed 2 | 11,361 | 3,807 | 1,929 | 0 | 11,487 / 3,626 |
| (r0), seed 2 | 11,171 | 11,171 | 2,296 | 3 | 11,268 / 11,268 |

Fresh V2 is descriptive only. It repeats DEV's pattern: the memory lifts (r) and the trunk falls apart without it.

### Free generation (descriptive)

64 greedy replies per arm, nearest-decoded, 64 tokens each. A reply is cut after its first `<|eos|>`; "complete" means it has one. Measures are against TRAIN's 212,968 replies.

| Arm | Complete | Verbatim TRAIN | New 4-gram / 8-gram share | Distinct-1 / -2 | Repeated 4-grams |
|---|---|---|---|---|---|
| DEV's own replies | 64 | 41 | 0.328 / 0.612 | 0.312 / 0.677 | 0.000 |
| (r), seed 1, memory on | 44 | 35 | 0.758 / 0.898 | 0.259 / 0.439 | 0.435 |
| (r), seed 1, memory out | 1 | 0 | 0.929 / 1.000 | 0.135 / 0.238 | 0.692 |
| (r0), seed 1 | 63 | 36 | 0.398 / 0.568 | 0.208 / 0.393 | 0.112 |
| (r), seed 2, memory on | 41 | 31 | 0.785 / 0.904 | 0.207 / 0.340 | 0.547 |
| (r), seed 2, memory out | 0 | 0 | 0.930 / 1.000 | 0.090 / 0.148 | 0.770 |
| (r0), seed 2 | 56 | 35 | 0.658 / 0.852 | 0.157 / 0.254 | 0.500 |

(r0)'s rows are the same with the memory on and out: its tables are empty.

## Instrument checks (all pass)

- **I-1:** the crate's CUDA tests on the pod's build: 194 passed, 0 failed, 6 ignored, `row_picks_fold_into_the_nearest_reading` among them.
- **I-2:** every arm's DEV totals equal claude-194's sealed last-checkpoint readings exactly: 11,513 / 4,101, 11,361 / 3,807, 10,079 and 11,171. So `row_picks` reproduces `nearest_reading`, and `inspect_run`'s extra forwards left no state behind.
- **I-3:** 0 decision rows pick differently with the memory taken out, in every arm. The memory names none of them, as its module says.
- **I-4:** the grader recomputes 3,438 decision rows (3,406 + 32), the census's own count, and every arm scores the same 14,287 rows.
- **I-5:**
  - each seed's sealed `SHA256SUMS` hashed to its pinned value;
  - all 10 checkpoint lines per seed passed `sha256sum -c`;
  - Fresh V2's tar passed its sums and its own sha, and its members passed the code's pins.

## What changed

- **Code**, in `crates/uor-r4-float-train` (in the item's scope):
  - `in_space_long.rs`:
    - **The split:** `nearest_reading` now uses `row_picks` (its loop, unchanged) and `fold_picks` (its counts).
    - **The scorer:** `score_rows` writes `ScoredRows` and `IdSample` files, never replacing one.
    - **The grader:** `overturn`, with `paired`, `pick_differences`, `seed_cell`, `two_root_ceil`, `overturn_verdict` and `generation`.
    - **The shared helpers:** `decision_positions` (shared with `census`), `load_fresh_v2` and `reference_reply` (shared with `run_long`).
    - **Tests:** 9 new ones.
  - `bin/train-in-space.rs`: `--score-rows`, `--overturn 1` and `--scored NAME=DIR`, which refuses a name given twice, and `--scored` without `--overturn`.
- **The census is byte-identical** after the refactor (`cmp` against `census-2026-10-29.json`).
- **Docs:**
  - the sheet;
  - this REPORT;
  - the grading JSON and a section in `docs/labs/one-model-end-to-end/README.md`;
  - one notebook row;
  - done-when 2's parity line, narrowed by the decision review.
- **The job, watcher and renew loop** are `local/claude/claude-205/{job,watch,renew}.sh` on the laptop, and are not committed.

## Checks run at the head

- **On the laptop:**
  - `cargo test --locked -p uor-r4-float-train --no-fail-fast`: the lib's 134 passed, 0 failed, and every other target passed.
  - A stub-nvcc `cargo check --features cuda` of the lib, the tests and `train-in-space` builds.
  - `cargo xtask roadmap check`: 111 items OK.
  - The grader on fake all-hit and all-miss files gave 3,438 rows and "ahead", in 1.65 s.
- **On the pod:** I-1, the source pin, and the build with its sha recorded.

## Cost and seal

- **Pod:** `vast-58316782`, 1 × RTX 5090 (driver 580.178.04), $0.975/h.
  - **Timeline:** up from 22:32Z, the job from 22:34Z to 22:41Z, deleted by about 22:46Z. About 14 minutes, about $0.23.
  - **Closing:** released, then `down` with `--confirm-archived` after the store listing. 0 pods remain.
- **Sealed** to `caseyallard/uor-r4-store:results/claude/one-model-end-to-end-2026-10-29/overturn-b6f7bdc9/`.
  - **Contents:** 102 files, including the binary, every arm's rows and samples, and the job's logs.
  - **`SHA256SUMS` sha256:** `3c3c9e999af33c1023764ba7fc95ed94845d50a53ebe3d1b0ced29a8870aed29`.
  - **Read back:** the sums, the binary and one rows file, each passing against the sums.

## What it unlocks

**Step 3, the protocol (done-when 3).** It now has its arms, and four findings to build on:
- **The arms.** Its arm list adds a memory-dropout arm beside (r), as an equal candidate. Run at least three seeds per arm, since two seeds disagreed in sign here.
- **The decode rule.** Greedy loops on these trunks, so the Flock rank rule (`stack/flock.rs`'s harmonic weights) is a needed decode rule, not an extra one.
- **The new-template stratum.** It should come from a panel no decision has touched. claude-204's P-dev qualifies: 0.000 of its prompt templates are in TRAIN, and nothing here scored it.
- **Copying values.** Both trunks fail it, so copied rows need a stratum of their own, and the training run's read-by-meaning arms (the Flock read, the type spin) are where a fix would show.

## Still open

- **Done-when 3:** the protocol, with a fresh FIT, before the training run.
- **`integer-hamming-read` step 5:**
  - kept sets and top-1, on the checkpoint its sheet names;
  - a scope fix for a float capture hook in `flock_read_hamming.rs`;
  - the memory off on its float side if it uses claude-194's (r).
- **The served readout:** the integer session decodes with the writer, which is right on 77–1,280 rows here, where nearest is right on 10,079–11,513.

## Skills used

None was invoked this unit. The arms, the band, the cells and the stop rule were fixed in the sheet before the run, and applied as written.

## Plain summary

The question was whether a model that learned to lean on its exact memory reads worse than one that never had it, on the parts of a reply the memory cannot help with. The two training seeds disagree: one says better, one says worse, each by far more than chance. So the next training run will train both recipes, with more seeds, and let the scoring choose.

The model's own replies also show where it fails. The memory helps it start the right kind of sentence, but neither model copies an unusual name from the conversation. Without the memory, the model cannot finish a reply at all.
