# REPORT: one-model-end-to-end, claude-217 (step off a reply's own cycle)

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md). Order: [ORDER.md](ORDER.md). Sheet: [one-model-end-to-end-cycle-step-2026-10-30](../../../../harness/prep/one-model-end-to-end-cycle-step-2026-10-30.md): NOT FIT in rounds 1–3, FIT in round 4 (at `b3b46402c`; its note landed at `00bb21c43`), pre-run GO. All from fresh reviews, posted on #3407. Results: [cycle-step-2026-10-30](../../../../labs/one-model-end-to-end/cycle-step-2026-10-30/README.md). References #3407.

## Tokens per second

- **Every arm decodes at 51–55 tok/s per process** (50.8–55.0 per file) on one RTX 5090, with the three seeds sharing it.
- **The cycle and echo steps cost nothing measurable:** 53.7 and 52.1 against greedy's 52.9 at seed 1. Each step is an exact n-gram lookup over the reply.
- **This is not the served speed.** The integer path is not built.

## What did not work, first

- **Every registered decision reads "underpowered (content)",** so the served decode stays greedy (`ships: greedy`).
  - **Why:** at seed 1, only 11 of the 59 looping prompts' references are verbatim TRAIN replies (`p` = 0.19), below the power condition.
  - **What this is:** an instrument outcome, never a negative.
- **Where the reading is powered, every loop fix writes no TRAIN-like reply where greedy looped.** This decides nothing, since the rule needs all three seeds.
  - At seeds 2 and 3, the references on greedy's 63 and 86 looping prompts are verbatim in 40 and 52 cases.
  - The cycle step, the echo step, the decode-only cut and the cycle step under the cut write 0 verbatim replies and 0 replies equal to their reference there.
- **In the fixed reading, the rules end the loop, not the failure.** In seed 1's [fixed reading](../../../../labs/one-model-end-to-end/cycle-step-2026-10-30/fixed-reading-s1.md), the replies are often wrong from the opening.
  - Item 1 reads "Here are three better of help helpave a great." for "Got it, I'll remember them." under every arm.
  - Where greedy loops, the cycle step changes the ending: "Home for for for for" (item 3), and "…What What What What friendundrtttt feel that." (item 5).
  - This is a reading of 20 replies, not a count.
- **claude-216's reported decode-only "improves" does not hold up on the looping prompts.** There the cut writes 0 verbatim replies at seeds 2 and 3. Its earlier reading came from matched prompts, which cannot see the looping ones (this sheet's FIT round 3).
- **On held-out phrasing the model rarely writes the right reply at all.**
  - Greedy's reply equals its reference in 1–4 of 256 on p-s (3, 1 and 4), against 38–40 on DEV. No arm passes 5, the cut's count at seed 1.
  - **Teacher-forced on p-s** (this run's sealed rows, counted with `jq`), the reply's first token is right 17.3%, 13.7% and 18.5% of the time at seeds 1–3, against 80.7%, 80.1% and 79.8% for later tokens. claude-216's sealed p-t rows agree: 18.7%, 12.2% and 17.9%, against 80.8–81.1%.
  - **The turn memory barely reaches a new phrasing.**
    - On DEV, openings are right 49.1–49.3% with the memory and 21.8–23.0% without: it adds 325–337 of 1,226.
    - On p-s it adds 14–16 of 1,400: 17.3/13.7/18.5% with it, against 16.3/12.5/17.4% without.
    - Its key is the exact token ids of the turn's first `k` slots (`flock_read_memory.rs`), so a new phrasing finds no stored key.
- **Matched content is unchanged at seeds 2 and 3, and 0.037 higher at seed 1.**
  - The cycle step reads 0.358, 0.442 and 0.377, against greedy's 0.321, 0.442 and 0.377.
  - The matched prompts include greedy's replies that repeat a 4-gram and still end (21, 6 and 37), and the step changes those. Elsewhere it writes greedy's reply.

## What worked (reported)

- **The cycle step ends every loop, exactly where greedy loops.**
  - Looping replies fall from 59, 63 and 86 to 0, and every reply completes (256 of 256).
  - It stepped off on 59, 63 and 86 replies: greedy's looping prompts exactly. No step was exhausted.
  - Every step reply follows greedy until its first step off (`follows_greedy`).
- **The echo step does the same:** 0 loops, with 255–256 complete. It steps off on more replies (61, 70 and 98), because it also acts on a token's second immediate repeat.
- **The decode-only cut** leaves 32, 45 and 49 loops. Under it, the cycle step ends them too.

## What the readings suggest (inference, not a decision)

This adds this run's readings to the loop diagnosis of 30 Oct (#3407 6524734651).
- **The loop looks like the end state of a reply that has already gone wrong.** It is a one-step echo onto a hub token, where the shared table makes the input token nearest.
- **The larger failure looks like the reply's opening on a new phrasing.**
  - Teacher-forced on p-s, the model continues a reply well (about 80%), but rarely starts the right one (14–19%).
  - The memory that doubles DEV's openings finds no key on a new phrasing.
- **So on these prompts, a decode guard mostly changes how a wrong reply ends.**
  - Seed 1 is the exception: there the cycle step writes 8 verbatim replies where the references have 11.
- **The consequence for claude-218 is an inference.** The rollout hinge as first planned trains each repeat state toward the guard's choice. Where powered here, that choice wrote no TRAIN-like reply, so the hinge would likely train toward salad on these prompts.

## What changed

- **`DecodeRule::CycleStep { k, n }`** and `cycle_step_pick` (`crates/uor-r4-float-train/src/in_space_long.rs`):
  - of the `k` nearest, the first whose append closes no `n`-gram the reply already holds; the nearest when every one would;
  - one mark per step (`IdSample::stepped`, `STEP_*`), not written under other rules;
  - it stops after the end of turn.
  - `CYCLE_STEP` is `n = 4` (Casey's rule) and `ECHO_STEP` is `n = 2`; the file names come from `step_name`.
- **`--score-rows --cycle-step 1`** decodes the cycle step, alone and under each `--reply-read` scope, and the echo step (`crates/uor-r4-float-train/src/bin/train-in-space.rs`).
- **`--cycle-grade 1` (`cycle_grade`):**
  - both decisions by `cycle_outcome`;
  - matched content plus content on the base's looping prompts (`LoopContent`, `kept`, and `powered` with its two conditions);
  - `follows_greedy`, step counts (`StepCount`), and `ships`, which computes the ship table.
  - `loops` is factored out of `looping_replies`.
- **The panel p-s,** drawn by `heldout-panel` at seed 2091, and its census: 0.000 of 1,400 prompts in TRAIN, exactly or as templates at K = 1,024, and 0.624 of replies verbatim. It is stored at `inputs/claude/heldout-panel-p-s-2026-10-30/`.
- **Also written:** the prep sheet, the grade ([cycle-grade.json](../../../../labs/one-model-end-to-end/cycle-step-2026-10-30/cycle-grade.json)), the fixed reading, the lab page and one notebook row. The local job and watcher are in `local/claude/claude-217/`: `job.sh` sha256 `b1afdb67399acf475091a35158dfb8d79f562b3a10df7a371a994fd29b7ecc83`, `watch.sh` `7e1b10c81a989fc691c47060399243d1c8d591af31a406382b7d0151064e5fc0`.

## Measurements

The p-s tables, per arm and seed, are on the [lab page](../../../../labs/one-model-end-to-end/cycle-step-2026-10-30/README.md).
- **The power floor is met:** the base loops in 59, 63 and 86 replies.
- **The matched prompts:** 218, 199 and 207.
- **DEV (reported):** the base loops in 46, 28 and 29 replies, and the cycle step and the echo step in 0. Matched content is unchanged within 0.012.
- **The run:** one RTX 5090 (`vast-58411840`, $1.106/h), up about 50 minutes, about $0.90.
  - Sealed at `results/claude/one-model-end-to-end-cycle-step-2026-10-30/run-dee65f6f/`: 103 files, `SHA256SUMS` sha256 `55d13779882299fb19a11b71b3611c95f5d4966d968ffa5d4d019578b20bfc00`, matched on the pod file for file and read back OK.
  - The pod is down.

## Checks run at the head

- **The crate's host tests** (`cargo test --locked -p uor-r4-float-train --lib`): 173 pass.
- **Negative gates,** each failing with its line set aside and passing when restored:
  - the cycle step's refusal;
  - the loop-prompt reading's condition in `cycle_outcome`;
  - the power condition;
  - after the code review: the matched-content condition, the loop floor, the "no gain" conditions on loops and on holds, and the matched floor.
- **The CUDA test's limit (from the code review).** `the_cycle_step_follows_greedy_until_it_steps_off_a_cycle_on_a_model` checks that k = 1 equals greedy and that the step marks every loop greedy makes. It does not require greedy to loop on its random model. The step-off wiring on a model is shown by the sealed run's grade: every step reply is marked, the steps fall exactly on greedy's looping prompts, and every reply follows greedy until its first step off.
- **The review's last fixes are host-only** (test cases, a seed-pairing refusal in the grade, two comments). They change no output of the sealed run's binary. The grade at the head reproduces `cycle-grade.json`.
- **The stub-nvcc CUDA clippy run:** no new findings.
- **On the pod:**
  - the CUDA tests pass;
  - every input matches its sha256;
  - the 15 registered files reproduce byte for byte;
  - every expected file is written, and every step file has 256 marked replies.
- **`--cycle-grade 1`** on the sealed run, at `dee65f6f8`: exit 0.
- **Counted with `jq` from the sealed samples:** the exact-reference counts, the replies before the loop, and p-t's opening and later-row accuracy.

## Skills used

None.

## Still open

- **claude-218's design:** Casey approved a rollout hinge. This run's readings suggest it would train toward salad, and that the opening's failure is the memory's key. So a redesign aimed at the opening is put to him: a turn-memory key that a new phrasing of a known question can reach.
- **The registered next step for "underpowered (content)":** a content reading that credits a reply equal to its reference. On these prompts that count is 0–2 for every arm, so it would read lost.
- **p-s has now been opened once,** by this decision.

## Plain summary

Casey asked for a rule that stops the model's replies repeating themselves: when the next word would repeat a four-word stretch the reply already has, take the next-best word.

It works as a loop stopper. Every loop is gone, every reply finishes, and it costs nothing in speed. But it doesn't make the replies right. On new phrasings the model was already writing the wrong reply before it looped, and stopping the loop just ends a wrong reply sooner.

The real problem looks like the first word or two. On a new phrasing the model picks the right opening less than a fifth of the time, though it continues well once started. The memory that doubles the right openings on familiar wording looks up the question's exact first words, so a reworded question finds nothing. So the next work should be getting the opening right on new wording, not stopping loops.
