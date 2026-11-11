# ORDER: one-model-end-to-end, claude-350

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Base: `8b818936cc6494b2261204f48a840069a8daa5b6`.

## Goal

**Plan the repaired arm: the question finds the place where the value starts, a register holds that place as a number, and the reply reads the value from it token by token. Count on the laptop what the build must contain, before any pod.**

**Why now.** [claude-349](../claude-349/REPORT.md) closed turn admission with arm r's copy read as written. On 11 Nov (UTC), in chat, the lab asked "Plan the repaired arm: point at the value's place and read on from there?" and the owner answered **"Yes, plan it"**. That approves this plan only: no pod, no training, no build, no other arm. The hold of 10 Nov stands for everything else.

**What the record already shows** ([float-training notebook](../../../../research/notebook/float-training.md)):

- **A question can choose a value it is not keyed by.** On a synthetic task trained 4,096 steps (claude-330, B1), ask heads with the found-copy pull name the asked value on 512 of 512 per far bucket at both seeds (`R_ask` 1.000, `R_oth` 0.000). `R_ask` scores the value named at the output, so it measures the find together with its read, not the position alone. With the pull at the question row, the ask heads keep the asked value at that row in 376 to 512 of 512 rows (B2-aq).
- **A carry over a short, fixed distance works; the language run did not move.** The question's ask point carried 4 or 8 rows to the readout read passes the same task four tokens after the question: 466 to 512 of 512 per far bucket against at most 2 with no read, and the carry cut drops it to at most 54 (claude-335). Holding the value in the stream did not rise above its no-read control, at most 2 of 512 against at most 5 (claude-334, configuration closed). On the language run a 32-row carry read 0 and 2 of 256, the same as with the carry cut (claude-339, that configuration only), and a count of how far back the question sits was negative for a reach of 32 on that TRAIN (claude-342; the count alone does not settle the carry).
- **Reading at a supplied place works, in two other settings.** An object-store oracle with the address supplied passed 80 of 80 (40 current, 20 previous, 20 typed abstentions) against 0 of 80 with no memory and 20 of 80 for the sieve ([m3 notebook](../../../../research/notebook/m3-durable-memory.md), 26 Oct); the [board](../../../../mechanisms/README.md) adds that learned address formation is not built. Arm r's copy read is exact on 0.749, 0.759 and 0.782 of 399 matched recall pairs at three seeds, with an oracle-written line in the reply's wording in the context.
- **The value is one unbroken run of tokens** in an earlier user turn on 562 of 562 recall replies, and the reply's wording before it differs from the statement's on 247 of 488 (claude-349).

A carry has a fixed reach and brings one point. A reply's value sits any distance after its question and can run over several tokens. This arm holds the place, an integer with no reach, and steps it.

## The arm: place read (arm p)

| Role | Part | Status |
|---|---|---|
| Find | An ask head's question: the token's point plus a learned question row, through the key's rotation (at zero rows, exactly a copy head's question), against the copy key, the point of the token before each earlier position. The found-copy pull trains the rows. | exists (`flock_read_hamming.rs`; served by `stack/hamming.rs`, claude-336) |
| One position | The nearest position per ask head, the later on ties: the rule serving's copy heads have as `CopyRule::LatestNearest`. Today an ask head reports the mean of its 16 nearest, ties to the earlier. | new |
| Two-token question | Only if stage 1 routes it: the previous token's half asked plainly beside the current token's half with its row. Today only copy heads ask with two halves. | new, conditional |
| Hold | One place register per conversation. A find at a question row sets it; the next user turn clears it. It is an integer: no decay, no reach. | new, exact |
| Read | At an open reply row the register offers the shared table's point of the token at its place, through the copy read's own readout mix, `n + w_c ⊙ (c − n)`. | the mix exists; the source is new |
| Step | At an open row, when the reply's token equals the token at the place, the place moves on by one and the next row is open. A miss closes that row only; a token emitted at a closed row moves nothing. When the stop ends a read, the register clears. | new, exact |
| Arm, open, stop | Which question row's find sets the register, which reply row opens the read, and what ends it. Stage 1 chooses among the candidates below. | open |
| Train | Teacher-forced, the same integer logic over the window's tokens; no gradient passes through the register. The pull (`ask_pull`) is aimed from the question's rows at the key of the labelled place. The label is the start of the reply's run in an earlier user turn, read from the tokens. | the pull exists; its aim and the label are new |

Every new step is an integer compare or an index, so §3.1 to §3.3 hold as they do for the copy read. This is not the E8 address memory (PR #3913 stays a draft) and not a read keyed on the trunk's state.

## Stages

1. **Counts on the laptop** (this unit, on the owner's word; no model, no GPU). The census of claude-349 gains the counts below. A second fresh review of this text runs before the first count.
2. **Build** (a later unit, only as stage 1 routes it): the register in the float trainer and in the integer serve, export, float against integer parity, gradient checks, and a one-page sheet with a non-author FIT.
3. **One pod, when the owner says run.** Arm p at three seeds beside three controls on the same recipe: the register off (the no-memory control, L-010); the register set to the true place (separates a find failure from a read failure, L-007); arm r with its line as the reference. Judged first by reading replies in the laptop chat with the sieve's line off, then by the exact-value rate on the 562; decode tokens per second reported. A pass removes the sieve's line for this artifact. A fail with the true-place control passing points at the find; with it failing, at the readout mix.

## Stage 1 counts

Panel: the 1,000 conversations of claude-349 (562 recall replies, 1,945 general replies with an earlier user turn, 79 abstentions). The true place `s` is the value's first token in its user turn, from the oracle line. A "majority rule" on a token fires on that token when firing is right more often than wrong on the panel; such rules, and C2's and C3's targets, are fitted on the panel itself, so they are ceilings.

| # | Count | Settles |
|---|---|---|
| A | Read-on from the true place, stepping while the reply's token equals the source's: the whole value is read, of 562. | Instrument check: 562 of 562 by the run fact. |
| B | Values of one token, of 562. The exact value, no token more or fewer, of 562, under B1: stop where the source's next token starts a new word or is punctuation; B2: the majority rule on the source's next token. | Whether a step is needed, and the stop rule. |
| C | Find, by the question an ask head can form at a row `t` of the asking user turn. C1 (zero rows): the token before `s` is `x_t`. C2 (learned row): the token before `s` is the one target that fits `x_t` most often. C3 (two-token question): C2, and the token two before `s` is `x_{t−1}`. A reply passes a form if, at some row of its question, the latest earlier position that fits is the true place. Of 562, with candidates every earlier position and earlier user turns only. | The ask form. |
| D | Arm, with C's chosen form: the register is at the true place when the reply starts, of 562, under D1: the last question row that finds anything; D2: the row whose find fits the fewest positions, the later row on ties; D3: the majority rule on the question token. Under each rule: general replies with the register set, of 1,945; abstentions with it set, of 79. | The arming rule and the leak. |
| E | Open, with the register as D's chosen rule sets it, under the majority rule on the reply's current token. Right: value-start rows opened, of 562. Wrong: other rows opened while the register is set, of all reply rows, for recall replies, general replies and abstentions separately. Rows the step opens are left out. | The open rule. |
| F | Label from the tokens alone: the latest earlier user-turn run equal to the reply's, first token outside stop list 64. Equals the true place, of 562; fires on general replies and abstentions; replies labelled in TRAIN; for each target C2 or C3 uses, its count in TRAIN's labels. | Whether training needs marks from the corpus writer, and whether TRAIN holds the targets. |
| G | Tokens from the value's reply row back to the place: within 192, 256, 512, more. Panel, TRAIN and the laptop chat scripts. | The window the register must cover. |

## What stage 1 decides

Fixed before the count. A row routes the build. Only the Find and Arm rows' last clauses stop it, and the Stop, Open and Window rows can send a plan change to the owner.

| Choice | Rule |
|---|---|
| Instrument | A below 562 of 562, or a disagreement with the writer's receipts: repair the instrument and read nothing else. |
| Stop | Every value one token: no step and no stop are built. Else B1 at or above 0.95: build B1. Else B2 at or above 0.95: a learned gain per source token. Else a plan change, back to the owner: a stop that reads the trunk's state is a state-keyed read, which the hold covers. |
| Find | Read after taking out replies whose only fitting target TRAIN holds fewer than 100 times (count F); their share is named. The simplest of C1 to C3 at or above 0.80 is built. If none reaches it, the misses are traced, the form is widened once to two targets per token (two ask heads), and C is recounted. Then at or above 0.50 the best form is built and the REPORT names the share it cannot reach; below 0.50 it is not built as written, and the trace goes to the owner. |
| Arm | The simplest of D1 to D3 that sets the true place on at least 0.90 of the replies C reaches, with the register set on at most 0.15 of general replies and 0.10 of abstentions. If none meets all three, the rule with the fewest wrong sets among those at or above 0.50 is built, and stage 3 reads general replies and abstentions first, register on against off. If none reaches 0.50: a plan change, back to the owner. |
| Open | E right at or above 0.95 and wrong at most 0.02 in each of the three kinds of reply: a learned gain per reply token. Else a plan change, back to the owner, for the Stop row's reason. |
| Label | F equal on at least 0.90: labels from the tokens. Else the corpus writer emits place marks, and that change joins stage 2. |
| Window | More than 0.05 of places beyond 192: stage 2 trains and serves the register over the window that covers 0.95. Beyond 512 on more than 0.05: a plan change, back to the owner. |

## Not established by this plan

- Nothing is trained. No count can show that a trained model takes the register's token, that the question rows learn their targets, or that the mix ignores a wrongly opened read; only stage 3 can.
- The corpus's values are single made-up words and numbers. Values of several words, a fact spread over two turns and a corrected fact are not counted.
- The laptop chat scripts are read for reach only in stage 1; the replies are read in stage 3.

## Pre-count review

One fresh-context reviewer read the first text at `c85e30a8ae92946508071f2c68a0b1b5f7b6e196` against the spec, the cited rows and the code: **fit with findings**, eleven of them, all taken here. The arm table said an ask head already keeps one later-on-ties position and already asks with two tokens; neither is on `main`, and both are now marked new. The open count had no denominators and the find count did not say which row counts; both are defined. Four cases had no route (a find between 0.50 and 0.80, no arming rule at 0.90, a leak with no next rule, abstentions); each has one. The record's claims gained their qualifiers, and the author added claude-335's passing carry, which the first text left out. No reviewer has read this revised text; stage 1 opens with that review.

## Cancel probe (run first)

- **What would make this unit unnecessary:** a read on `main` that holds a position across rows, or a census that already counts a read-on.
- **Result (at claim):** none. `git grep -n -i -E "cursor|place[_ ]register|place[_ ]read" HEAD -- crates/uor-r4-float-train/src crates/uor-r4-integer/src/stack` gives 36 lines, all loop indices over batches and windows. The hold and the carry sum per-token gains over fixed offsets, and the prompt register (`flock_read_register.rs`, claude-155) keeps weighted means of the current turn's points; none holds a position.

## Claim

Written by hand, as claude-347's to claude-349's were: claude-346 (PR #3913, a draft under the hold) holds the item for `uor-harness loop claim`.

## Files in scope

- `docs/loop/tasks/one-model-end-to-end/claude-350/**`
- `crates/uor-r4-float-train/src/turn_census.rs` (stage 1's counts), and in `crates/uor-r4-float-train/src/bin/train-in-space.rs` only the `--turn-census` argument arm, written already formatted
- `docs/research/notebook/**` (one row a result)

## Inputs, all on the laptop

- The three corpora and the count output of claude-349 under `local/claude/claude-349/`, written by `m-world-recall-corpus` (`crates/uor-r4-training`).
- The laptop chat scripts under `local/claude/claude-345t/talk/`.
