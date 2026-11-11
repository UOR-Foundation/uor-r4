# ORDER: one-model-end-to-end, claude-349

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Base: `c0f7db014c28849fb9be11aa5b1f64f6291ec878`.

## Goal

**Plan one arm in which the model itself picks the turn that holds the answer and copies the value from it exactly, and count on the laptop whether that can work before any pod.**

**Why now.** The owner's hold of 10 Nov stopped every new mechanism arm. On 11 Nov (UTC), in chat, the lab asked "Lift the hold for one arm: a learned geometric selector for which fact answers?" and the owner answered **"Yes, plan it"**. The hold stands for everything else. The readings behind the ask:

- With the chat program's rule and its memory line removed, neither arm answers any recall question: 0 of 4 at every seed, against 3, 4, 4 and 3, 3, 3 with it ([claude-347](../claude-347/REPORT.md), reading 2). That reading could not separate picking the fact from reading that far back.
- The rule is `stack/dialogue_memory.rs`, by its own header "an exact lexical sieve, not a learned reader". Nothing the model learned decides which fact answers.
- With an oracle line the float model copies it exactly on 0.759, 0.749 and 0.782 of matched recall pairs (three seeds), and on 0 without a line ([claude-345](../claude-345/REPORT.md)). Copying works when the source is unambiguous.
- The board's exact-memory row: with the true address the read passed 80 of 80, against 0 of 80 with no memory and 20 of 80 for the sieve, and "learned address formation is not built".
- On the two-class task the learned ask heads are exact where the value follows the question (512 of 512, both seeds, claude-330) and, with the question carried, answer 466 to 512 of 512 four tokens later against at most 2 with no read (claude-335). On the human-text mix the same heads did not move recall (claude-339, a negative for that configuration). Their pull toward the found copy (`HammingReads::ask_pull`) has never been given to a read that picks by turn.

## The arm: turn admission (arm s)

Arm r's recipe and copy rule, unchanged, with one read added in front of the copy heads. Every part exists except the turn code and the wiring.

| Step | What happens | Existing part | New |
|---|---|---|---|
| Turn code | Each turn, of either speaker, gets one sign code: the sum of its tokens' table points, each turned lane by lane by a learned unit quaternion, each with a learned weight for its token that starts at 1 | shared table; the Hamming read's lane rotation and `pack_signs`; a learned gain per token, as `--hamming-hold` has | the sum over a turn |
| Ask | At a reply, the code of the turn being answered, through a second rotation | the type heads' second rotation | the same sum |
| Choose | Hamming distance by popcount. The nearest turn inside a learned margin is admitted; if none is inside, none is | margin weights `max(0, m − d)/m`; `choose_sorted_from` | one choice a reply, not one a row |
| Copy | The copy heads (latest exact match, arm r's rule) look only inside the admitted turn, and read nothing when no turn is admitted | `--hamming-copy-latest 1` | the restriction |
| Learn | A margin hinge on the distances: the turn in which the copy read finds the reply's tokens is pulled inside the margin, the nearest other turns pushed outside, and all turns pushed outside for a reply that copies nothing. Signs pass straight through | the found-copy pull (`HammingReads::ask_pull`); the rank-margin hinge | the label by turn, counted on TRAIN with no writer oracle |
| Serve | Codes as u64 words, one a turn, kept by the session; popcount and an integer margin | `stack/hamming.rs` | one header record |

What it replaces: the sieve and the `System: Memory:` line at serving, and the line in TRAIN (`recall=off`). What it is: a learned form of the sieve's shared-word count, in the Hamming read's arithmetic. What it is not: a read of the trunk's state, which is dominated by the last few tokens and "never learned where to look" (`flock_read_hamming.rs`, module text).

**Known risks, stated before the counts.**
- A reply of the form "It's VALUE." repeats no word of the statement before the value, so the latest exact match inside the right turn may still miss the value's first token. Stage 1's ceiling is split by reply form to size this; the repairs on hand are a reply that repeats the statement's words, or a pointer to the value inside the turn.
- TRAIN's conversations hold about 4 turns and the 18-turn script 18, so the served read chooses among more turns than it was trained on. The code of a turn does not depend on how far back the turn is, which is the reason to pick by turn and not by the trunk's state.
- The reply's first word still comes from the turn memory, whose keys are exact wording. Reworded questions are read, not fixed, by this arm.
- The turn memory's rule for leaving a key to the copy read (`--turn-memory-leave-copied`) must be recounted under the admitted turn.
- Serving needs one new header record in the artifact. Existing records do not change.

It should also stop a known fault: today any "… is" in a general reply is filled with what last followed " is" in the conversation ("The opposite of hard is Maya."). With no turn admitted the copy heads read nothing.

## Stages

1. **This unit, laptop only, no pod: three counts** on arm r's conversations without lines (TRAIN `conversations=60000 seed=13 recall=off`, panel `conversations=1000 seed=14 recall=off`), added to the trainer's existing census (`train-in-space --copy-key-oracle`):
   - **Labels.** For each reply and each turn before it (either speaker, the turn being answered included): how many of the reply's tokens the latest exact match names right when it looks only inside that turn. The label is the turn with the most, if it names at least two (ties to the later turn); otherwise none. Reported: replies with a label; on recall replies (found by pairing with the same conversations written `recall=reply`), how often the labelled turn holds the line's value; and how many labelled turns lie whole inside a 192-token training window.
   - **Ceiling.** With the labelled turn admitted, the share of recall replies whose value the copy read names right, split by the reply's form ("It's VALUE." against a reply that repeats the statement's words), beside the same share with no restriction (today).
   - **Floor.** The share of replies for which an untrained code (plain table points, no rotation, weights 1) already picks the labelled turn, on all replies and on recall replies, by the number of turns to choose from.
2. **Next unit, if stage 1 says go: build and register.** Trainer, exporter, session, tests with negative gates; the one-page sheet with bars taken from stage 1's counts; a fresh-context FIT. Laptop only.
3. **Then one pod, when the owner says run:** arm s at seed 2 (arm r took 50 minutes and $0.78 on one 5090). Controls on the same rows: arm s served with admission off (the no-memory control, L-010), the oracle turn on the float panel (the ceiling), and arm r with its line as the reference. Seeds 1 and 3 only after a row-1 reading.
4. **Wording.** The reworded 18-turn script and question templates held out of TRAIN. Read at stage 3 with no bar; its own step after.

**How it will be judged (bars are fixed in the sheet, after stage 1).** On the laptop, sieve off, no line, integer artifact, through `uor-chat`: recall in the registered 18-turn script, of 4 (arm r: 3, 4, 4 with the sieve, 0 without), read word for word. General turns no worse than arm r's. Float panel without a line: exact recall replies (arm r: 0).

## What stage 1 decides

| Reading | Means | Next |
|---|---|---|
| Ceiling at or above 0.60 of recall value tokens, and labels on at least 80% of recall replies | selection is the missing step, and copying inside the right turn works | stage 2 |
| Ceiling under 0.60 with labels present | the copy inside the right turn fails (first word, repeated token, value split across pieces) | the arm is not built; the 20 worst replies are read token by token and the repair is named |
| Labels on under 80% of recall replies | the copy read cannot say where a reply came from | the label is the fault: read 20 unlabelled replies; a writer label is the fallback, stated as an oracle |
| Floor already at or above the ceiling's reach | the untrained code picks the turn | stage 2 without the learned weights first |

The thresholds route the next step; none stops the line of work. A miss closes this configuration only (CHARTER §4.1).

**Claim check.**
- (a) **Step.** The owner direction's last part, "one model end to end" (`PLAN.md`): a grounded multi-turn conversation needs the model, not a rule in the chat program, to find what the user said.
- (b) **What it changes in the model.** This unit: nothing. The arm: one read, about 500 learned values (two rotations of 64 quaternions each, and a margin) plus one weight a token (4,096).

**Rules check** (CHARTER §3). Geometry only: sign codes, lane rotations, Hamming distance, a margin hinge. No squashing function and no exponential anywhere, in the read or its loss. No dense multiply: a sum of table rows and a lane-wise quaternion product. Float training offline on the GPU, integer serving. No neural comparator, teacher or grader. This unit runs no training: counts on the laptop (host checks, §4.5).

**Not in scope.** Any pod or training run. Any other arm: the hold stands. The E8 address memory (PR #3913 stays a draft). DeepSeek's contracts under `conformance/stack/`. `crates/uor-r4-cpu-train`. Any bar or status.

## Cancel probe (run first)

- **What would make this unit unnecessary:** a read on `main` that picks an earlier turn, or a census that already counts the copy read by turn.
- **Result (at claim):** none. `git grep -n -i -E "admit(ted)?_turn|turn_admission|turn admission|fn admit" origin/main -- crates/uor-r4-float-train/src crates/uor-r4-integer/src/stack` finds only `admit_launch`, a CUDA module check.

## Claim

Written by hand, as claude-347's and claude-348's were: claude-346 (PR #3913, a draft under the hold) holds the item for `uor-harness loop claim`.

## Files in scope

- `docs/loop/tasks/one-model-end-to-end/claude-349/**`
- `crates/uor-r4-float-train/src/bin/train-in-space.rs` and the census code it calls (stage 1's three counts only)
- `docs/research/notebook/**` (one row a result)

## Inputs, all on the laptop or in the store

- The writer `m-world-recall-corpus` (`crates/uor-r4-training`), world v2, protocol 2; the no-line panel is sealed under `results/claude/one-model-end-to-end-memory-line-2026-11-10/panel-noline/`.
- Arm r's artifacts and scripts: `local/claude/claude-345r/`, `local/claude/claude-345t/talk/`.
