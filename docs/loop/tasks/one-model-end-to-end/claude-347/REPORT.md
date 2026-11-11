# REPORT: one-model-end-to-end, claude-347

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Order: [ORDER.md](ORDER.md). PR #3924.

Five laptop readings on the replicated arm r model, asked for by an outside review. No pod, no training, no bar. Every claim below is [claude]'s.

| Reading | State |
|---|---|
| 1. Float against integer, row by row | done, with one part not readable on this path (below) |
| 2. Selection from the model's own state | not run yet |
| 3. General turns in wording TRAIN does not hold | not run yet |
| 4. What the model holds, in full | done |
| 5. The BPB scorer's floor | done |

## What did not work, first

- **Reading 1 could not compare the replies with the turn memory on.** `uor-chat --prompt-ids` steps raw token ids and never tells the session where the reply begins (`mark_reply_start`, `crates/uor-r4-integer/src/stack/session.rs`); only the text chat does. So on this path the integer artifact answers with its turn memory off. The first comparison, against the float replies scored with the memory on, showed 678, 691 and 644 of 1,141 rows equal, and that number is an instrument reading, not a model reading: against the float replies scored with the memory **off**, the same integer replies are equal on 981, 1,022 and 1,048 rows. The memory-on comparison needs a reader that marks the reply start on raw ids. It is not in this REPORT.
- **With the memory off on both sides, 160, 119 and 93 of 1,141 rows still differ** between the float model and its integer export (seeds 1, 2, 3). Most of them are rows without a memory line: 146, 94 and 93.
- **The samples do not name which rows are recall rows.** `reference_named` is 0 on every row. The split used here is read from the prompt itself: a row is a line row when its prompt holds the memory line's opening ids (`53,91,353`). 641 rows are line rows and 500 are not, at every seed.
- **Reading 4 cannot give a count of learned values for the trunk.** The trunk's tables are stored packed (`*.fbs`, 32-bit words), so the artifact gives stored words and bytes, not learned values.
- **The item's claim was refused.** `uor-harness loop claim` answered that claude-346's worktree holds the item; the ORDER records it.

## What held

- **The integer export keeps the float model's right answers.** Memory off on both sides, replies that equal the reference word for word: float 537, 503, 498; integer 532, 503, 497. Rows right in float only: 14, 2, 3. Rows right in the integer only: 9, 2, 2.
- **On the 641 line rows the export is close to exact.** Equal replies: 627, 616 and 641 of 641. Seed 3 is identical on every line row.
- **No row failed to run.** 1,141 rows a seed, no error line, an empty error file.

## Measurements

### Reading 1: float against integer, the same prompt, row by row

Each of the 1,141 panel prompts in arm r's sealed float score went through the exported artifact: `uor-chat-bd2532281 --stack r-s<seed>-c8256.uorlut --tokenizer tokenizer.json --prompt-ids <the row's prompt> --max-tokens 48`, greedy `serve_step`. The two replies are compared up to and including the first end-of-reply id; the float score keeps 48 ids and runs on past it.

Float replies with the turn memory off (`samples-p-greedy-no-memory.json`), the like-for-like comparison:

| | Seed 1 | Seed 2 | Seed 3 |
|---|---|---|---|
| Rows | 1,141 | 1,141 | 1,141 |
| Replies equal | 981 | 1,022 | 1,048 |
| Replies different | 160 | 119 | 93 |
| First difference at reply token 0 | 70 | 51 | 32 |
| First difference at tokens 1 to 5 | 53 | 47 | 40 |
| First difference at token 6 or later | 37 | 21 | 21 |
| Float reply equals the reference | 537 | 503 | 498 |
| Integer reply equals the reference | 532 | 503 | 497 |
| Right in float only | 14 | 2 | 3 |
| Right in the integer only | 9 | 2 | 2 |

Split by whether the prompt holds a memory line:

| | Seed 1 | Seed 2 | Seed 3 |
|---|---|---|---|
| Line rows | 641 | 641 | 641 |
| Line rows, replies equal | 627 | 616 | 641 |
| Line rows, float equals the reference | 495 | 479 | 490 |
| Line rows, integer equals the reference | 490 | 479 | 490 |
| Rows without a line | 500 | 500 | 500 |
| Without a line, replies equal | 354 | 406 | 407 |
| Without a line, float equals the reference | 42 | 24 | 8 |
| Without a line, integer equals the reference | 42 | 24 | 7 |

The instrument reading, kept so it is not taken for a model reading: against the float replies scored with the memory on (`samples-p-greedy-memory.json`), 678, 691 and 644 rows are equal, and the integer reply equals the memory-off float reply but not the memory-on one on 307, 332 and 404 rows. The reverse case is 4, 1 and 0 rows.

**Instrument or model?** The memory-on gap is the instrument: the raw-id path does not switch the turn memory on. The memory-off differences are the model's export, and they fall where neither side reproduces the reference; they move 5, 0 and 1 right answers.

The chat program printed a median of 1,009, 1,015 and 1,010 generated tokens per second a row. These are short prompts (17 to 192 tokens), each in a fresh process; it is not a reading at context 8,256.

### Reading 4: what the model holds

Identical at seeds 1, 2 and 3 of each arm.

Arm r (`r-s<seed>-c8256.uorlut`, 14,186,496 bytes, `uor-r4.lut-stack/7`, a header of 8,777 bytes, 90 tables):

| Section | Tables | Stored words | Bytes | Share of table bytes |
|---|---|---|---|---|
| Turn memory | 6 | 3,007,301 | 12,029,204 | 84.8% |
| Token table (4,096 × 256, 16-bit) | 1 | 1,048,576 | 2,097,152 | 14.8% |
| Trunk, 6 layers (packed) | 72 | 14,640 | 40,128 | 0.28% |
| Copy reads | 10 | 2,568 | 10,272 | 0.07% |
| Final norm | 1 | 256 | 512 | under 0.01% |
| All tables | 90 | 4,073,341 | 14,177,268 | |

The turn memory holds 7,414 opening keys and 140,003 reply keys of 20 ids each; the reply keys alone are 2,800,060 words, 11,200,240 bytes.

Arm rt (`rt-s<seed>-c8256.uorlut`, 55,424,640 bytes, `uor-r4.lut-stack/9`, chain tail 9): the turn memory is 13,316,831 words, 53,267,324 bytes, 96.1% of the table bytes, with 7,414 opening keys and 630,933 reply keys. The other four sections are the same as arm r's.

A session snapshot was 48,069 bytes at position 198 (one row). It was not measured at the full context.

By bytes, arm r is 85% stored TRAIN replies and arm rt is 96%. The figure quoted before this unit, 1,112,576 values, leaves the turn memory out.

### Reading 5: the BPB scorer's floor

The M4 sheet ([m4-lawful-likelihood-bpb](../../../../harness/prep/m4-lawful-likelihood-bpb-2026-10-23.md), §6) scores a target by rank: the predicted token gets potential 9 × 2^24, the next seven ranked tokens get 7 × 2^24 down to 1 × 2^24, every token then gets 1 more, and the 4,096 potentials are normalized. The stream holds 2.837427 bytes a token.

| Quantity | Value |
|---|---|
| Sum of the potentials | 37 × 2^24 + 4,096 = 620,761,088 |
| Cost when the target is the predicted token | 2.0395 bits |
| Cost when the target is outside the top eight | 29.2095 bits |
| The 0.90 BPB target, in bits a token | 2.5537 |
| Lowest BPB this scorer can report (every target predicted) | 0.7188 |
| Largest share of targets outside the top eight that still reaches 0.90, every other target predicted | 1.892% |

So this scorer reports 0.72 BPB for a model that is right on every token, and one target in 53 outside the top eight puts 0.90 out of reach however good the rest is. The number measures the fixed share the scorer gives the prediction more than it measures the model. The sheet's other branch scores Q24 candidate scores; it was not computed here. The sheet and the M4 bar are outside this unit's scope: this is a reading, proposed as an addendum to the sheet.

## Evidence

Laptop files under `local/claude/claude-347/` (not committed; sums below are SHA-256).

| What | Seed 1 | Seed 2 | Seed 3 |
|---|---|---|---|
| Float replies, memory off | `2cb6a3ddb4b64009e56698499982d9f1d4ebc96288d26fc53bc926877cc119ec` | `745456abf55b0f4fe853278361c2e9192fecc9fb8dfd5e8567d99031be99e621` | `4b33e9d1454a8541aaaaad06f1577739393f661d43c9823411a29c96ae5ce1ae` |
| Float replies, memory on | `1fa6a8108cd74d20bb24e3a67de8029c1e17369b9a683dbf6cd3b0b15d77aee3` | `53d966dcfd63ff71e4b2d30f9af0fdbdc0607ca6c6d96c7d09aa4fb37e99a2e2` | `2f09f3526eaa285a65b89d0fc3a83c6dc163145769d5d22599febcbc3609244c` |
| Integer artifact | `f052d8069ed2e9ddbb87fd2feaace2ef2bd45b07580015f49fb4deefb87362c5` | `707dfdc3b564df95497d57303decccebe42ac382045d7a0aae461137fec18ad0` | `d229cf1b32c9eb7deed6698be98a3370ce4f18108063126f69b70f5aebcd309a` |
| Integer replies (`fi/r-s<seed>/integer.jsonl`) | `a82422cf4eb222d5854d299e0046209b03664074e7c6a13f0f063def9e2ecaae` | `5952639cd7d9f3189fcccdf725ea45c423ee65715ff66d50b38c64933ea518ad` | `2608ae69377fada8fb636071c3fc41b3b396b81c309839935e53a97b48228776` |

Chat program `uor-chat-bd2532281`: `69bc535af70325fd8362818a1f07f549e91cdbcb28d0457a79aa56c1ab58344c`. Tokenizer: `d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89`. The artifact sums equal the ones in [claude-345's REPORT](../claude-345/REPORT.md). The float files are the pods' sealed scores, `score/r-line/` of each run.

## Still open

- Readings 2 and 3.
- The memory-on comparison of reading 1: a reader that steps raw ids and marks the reply start.

## Plain summary

The exported integer model says what the float model says on 86% to 92% of the panel's prompts when both run without the turn memory, and it keeps the float model's right answers (532, 503, 497 against 537, 503, 498). On the rows that carry a memory line it is identical on 627, 616 and 641 of 641. The comparison with the turn memory on could not be made on this path. By bytes the artifact is mostly stored replies: 85% for arm r, 96% for arm rt. The BPB scorer in the M4 sheet cannot report below 0.72 and stops being able to reach 0.90 once 1.9% of targets fall outside its top eight.
