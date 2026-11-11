# ORDER: one-model-end-to-end, claude-350

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Base: `8b818936cc6494b2261204f48a840069a8daa5b6`.

## Goal

**Plan the repaired arm: the question finds the place where the value starts, a register holds that place as a number, and the reply reads the value from it token by token. Count on the laptop what the build must contain, before any pod.**

**Why now.** [claude-349](../claude-349/REPORT.md) closed turn admission with arm r's copy read as written. On 11 Nov (UTC), in chat, the lab asked "Plan the repaired arm: point at the value's place and read on from there?" and the owner answered **"Yes, plan it"**. That approves this plan only: no pod, no training, no build, no other arm. The hold of 10 Nov stands for everything else.

**What the record already shows** ([float-training notebook](../../../../research/notebook/float-training.md)):

- **The find works.** Ask heads with the found-copy pull choose a value by a question that is not its key: `R_ask` 1.000 and `R_oth` 0.000 at both seeds, 512 of 512 per far bucket (claude-330, B1).
- **Carrying the value forward as a vector does not.** Held in the stream for 8 rows it reaches the scored row on at most 2 of 512 per far bucket, at 4,096 and 16,384 steps, both seeds (claude-334). Carried on the language run the asked first line is right on 0 and 2 of 256 (claude-339).
- **Reading at a known place works.** With the true address the exact read passed 80 of 80, against 0 of 80 with no memory and 20 of 80 for the sieve ([m3 notebook](../../../../research/notebook/m3-durable-memory.md), 26 Oct); the [board](../../../../mechanisms/README.md) adds that learned address formation is not built. Arm r's copy read names 0.749, 0.759 and 0.782 of values when its source is unambiguous.
- **The value is one unbroken run of tokens** in an earlier user turn on 562 of 562 recall replies, and the reply's wording before it differs from the statement's on 247 of 488 (claude-349).

So this arm carries the place, an integer, where the earlier arms carried the value, a vector.

## The arm: place read (arm p)

| Role | Part | Status |
|---|---|---|
| Find | The ask heads. A question token asks with its point plus a learned question row, against the copy key: the point of the token before each earlier position, or of the two before under key 2. Nearest in Hamming distance, the later position on ties. | exists (`flock_read_hamming.rs`, `stack/hamming.rs`) |
| Hold | One place register per conversation. A find at a question row sets it to that position. It is an integer and does not decay. | new, exact |
| Read | At an open reply row the register offers the shared table's point of the token at its place, through the copy read's own readout mix, `n + w_c ⊙ (c − n)`. | the mix exists; the source is new |
| Step | When the token the reply emits equals the token at the place, the place moves on by one and the read stays open. Otherwise it closes. | new, exact |
| Arm, open, stop | Which question row's find sets the register, which reply row opens the read, and what ends it. Stage 1 chooses among the candidates below. | open |
| Train | Teacher-forced, the same integer logic over the window's tokens; no gradient passes through the register. The pull (`ask_pull`) is aimed from the question's rows at the key of the labelled place. The label is the start of the reply's run in an earlier user turn, read from the tokens. | the pull exists; its aim and the label are new |

Every new step is an integer compare or an index, so §3.1 to §3.3 hold as they do for the copy read. This is not the E8 address memory (PR #3913 stays a draft) and not a read keyed on the trunk's state.

## Stages

1. **Counts on the laptop** (this unit, on the owner's word; no model, no GPU). The census of claude-349 gains the counts below.
2. **Build** (a later unit, only as stage 1 routes it): the register in the float trainer and in the integer serve, export, float against integer parity, gradient checks, and a one-page sheet with a non-author FIT.
3. **One pod, when the owner says run.** Arm p at three seeds beside three controls on the same recipe: the register off (the no-memory control, L-010); the register set to the true place (separates a find failure from a read failure, L-007); arm r with its line as the reference. Judged first by reading replies in the laptop chat with the sieve's line off, then by the exact-value rate on the 562; decode tokens per second reported. A pass removes the sieve's line for this artifact. A fail with the true-place control passing points at the find; with it failing, at the readout mix.

## Stage 1 counts

Panel: the 1,000 conversations of claude-349 (562 recall replies, 1,945 general replies with an earlier user turn, 79 abstentions). The true place `s` is the value's first token in its user turn, from the oracle line. Rules that pick "the best target" are fitted on the panel itself, so they are ceilings.

| # | Count | Settles |
|---|---|---|
| A | Read-on from the true place: whole value read, of 562; how many tokens the reply and the source still share after the value ends. | Instrument check: 562 of 562 by the run fact. |
| B | Stop. The exact value, no token more or fewer, of 562, under B1: stop where the source's next token starts a new word or is punctuation; B2: the best rule that sees only the source's next token. | The stop rule. |
| C | Open. Reply rows opened rightly and wrongly by the best rule that sees only the reply's current token, with the register set. | The open rule. |
| D | Find, by what an ask head can express for a question row `t`. D1: the token before `s` equals `x_t`. D2: it equals one learned target of `x_t`. D3: D1, and the token two before `s` equals `x_{t−1}`. D4: D2, and the token two before `s` equals `x_{t−1}`. For each: the latest earlier position that fits is the true place, of 562, with candidates every earlier position and earlier user turns only. | The ask form. |
| E | Arm. With D's best form, the register is at the true place when the reply starts, of 562, under E1: the last question row that finds anything; E2: the row whose find fits the fewest positions; E3: the best rule that sees only the question token. The same rules on general replies and abstentions: how often the register is set at all. | The arming rule and the leak. |
| F | Label from the tokens alone: the latest earlier user-turn run equal to the reply's, first token outside stop list 64. Equals the true place, of 562; fires on general replies and abstentions; replies labelled in TRAIN; for each learned pair D2 or D4 uses, its count in TRAIN. | Whether training needs marks from the corpus writer, and whether TRAIN holds the pairs. |
| G | Tokens from the value's reply row back to the place: within 192, 256, 512, more. Panel, TRAIN and the laptop chat scripts. | The window the register must cover. |

## What stage 1 decides

Fixed before the count. A row routes the build; only the Find row's last clause stops it.

| Choice | Rule |
|---|---|
| Instrument | A below 562 of 562, or a disagreement with the writer's receipts: repair the instrument and read nothing else. |
| Stop | B1 at or above 0.95: build B1. Else B2 at or above 0.95: a learned gain per source token. Else the stop reads the trunk's point, and the sheet says so. |
| Open | C at or above 0.95 right, with at most 0.02 of rows opened wrongly: a learned gain per reply token. Else the open reads the trunk's point. |
| Find | The simplest of D1 to D4 at or above 0.80 is built. If none reaches it, the misses are traced, the form is widened once to two learned targets per token (two ask heads), and D is recounted. Below 0.50 after that: not built as written; the trace goes to the owner. |
| Arm | The simplest of E1 to E3 at or above 0.90 of what D reaches is built; a register set on more than 0.15 of general replies moves the choice to the next rule. |
| Label | F equal on at least 0.90: labels from the tokens. Else the corpus writer emits place marks, and that change joins stage 2. A learned pair seen fewer than 100 times in TRAIN is named as a gap. |
| Window | More than 0.05 of places beyond 192: stage 2 trains and serves the register over the window that covers 0.95. Beyond 512 on more than 0.05: a plan change, back to the owner. |

## Not established by this plan

- Nothing is trained. No count can show that a trained model takes the register's token or that the question rows learn their targets; only stage 3 can.
- The corpus's values are single made-up words and numbers. Values of several words, a fact spread over two turns and a corrected fact are not counted.
- The 18-turn laptop scripts are read for reach only in stage 1; the replies are read in stage 3.

## Cancel probe (run first)

- **What would make this unit unnecessary:** a read on `main` that holds a position across rows, or a census that already counts a read-on.
- **Result (at claim):** none. `git grep -n -i -E "cursor|place[_ ]register|place[_ ]read" HEAD -- crates/uor-r4-float-train/src crates/uor-r4-integer/src/stack` gives 36 lines, all loop indices over batches and windows. The hold and the carry sum per-token gains over fixed offsets; neither holds a position.

## Claim

Written by hand, as claude-347's to claude-349's were: claude-346 (PR #3913, a draft under the hold) holds the item for `uor-harness loop claim`.

## Files in scope

- `docs/loop/tasks/one-model-end-to-end/claude-350/**`
- `crates/uor-r4-float-train/src/turn_census.rs` and `crates/uor-r4-float-train/src/bin/train-in-space.rs` (stage 1's counts only)
- `docs/research/notebook/**` (one row a result)

## Inputs, all on the laptop

- The three corpora and the count output of claude-349 under `local/claude/claude-349/`, written by `m-world-recall-corpus` (`crates/uor-r4-training`).
- The laptop chat scripts under `local/claude/claude-345t/talk/`.
