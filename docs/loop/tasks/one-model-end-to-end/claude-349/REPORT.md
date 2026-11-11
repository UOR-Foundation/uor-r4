# REPORT: one-model-end-to-end, claude-349

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Order: [ORDER.md](ORDER.md). PR #3926.

Stage 1 of the plan for one arm (turn admission): counts on the laptop that decide whether the arm is built. No pod, no training, no model was run, and no bar or status changed. Every claim below is [claude]'s.

| Row of the ORDER's table | Reading | Applies |
|---|---|---|
| 1. Labels on under 0.80 of recall replies | 490 of 562, 0.872 | no |
| 2. Labelled turn holds the value on under 0.90 | 488 of 490, 0.996 | no |
| 3. Equal bags on over 0.02 | 0 of 488 | no |
| 4. Copy inside the right turn under 0.60 for every reply form | best form 0.448; all forms together 135 of 488, 0.277 | **yes** |

**Row 4 decides: the arm is not built as written.** Picking the turn is not the missing step. Copying the value out of the right turn with arm r's copy read is.

## What did not work, first

- **The copy read cannot take the value out of the right turn.** With the labelled turn admitted, the latest exact match on the token before (copy key 1, arm r's rule) names every token of the value on 135 of 488 recall replies (0.277). With the true turn admitted (the latest user turn that states the value) it is 140 of 562 (0.249). The ORDER's line was 0.60.
- **Two reasons, both read in the replies.** (1) 355 of the 562 values sit in a turn of three or more clauses, mostly lists ("Note these down: woutdai is 93, zundtetdroul is 32, feltam is 70, …"). The reply "Woutdai is 93." asks for what follows " is", and the latest " is" in the turn belongs to the last item. (2) Replies worded unlike the statement have no token to match: "That's 836.", "You're …", "You love green." against "I like green best of all the colors." These are 0 of 141, 0 of 14 and 0 of 11.
- **The registered stop list was the wrong size.** Numbers are written one digit a token, and the digits are among TRAIN's 64 most frequent tokens, so a reply like "That's 19." has no content token left. 72 recall replies got no label; the 20 the census prints all have a number as the value. With the 16 most frequent tokens stopped, all 562 are labelled and 560 labelled turns hold the value.
- **No stop list gives both a full label and a clean one.** General replies that get a label, where none is wanted: 587 of 2,945 at 16 (0.199), 227 at 64 (0.077), 28 at 256 (0.010), while the recall replies labelled fall 562, 490, 286. The ORDER's line for this fault was 0.05.
- **The floor was not counted as registered.** The ORDER's floor uses sign codes of table points; row 4 applied before the row that needs it. A table-free form (the question's words) is reported in its place.
- **Not counted:** the slot key's ceiling; labels on TRAIN (the panel only); and the reach in the registered 18-turn script, which the ORDER asked for (TRAIN's and the panel's replies have at most 8 earlier user turns).
- **Read by eye, not token by token.** Row 4 asks for the 20 worst replies read token by token. The census prints 20 copy misses as text (value, question, the turn, the reply); those were read as text.

## What changed

- `crates/uor-r4-float-train/src/turn_census.rs` (new): the census and its six tests. It reads conversations into turns, pairs the panel with its lined twin, labels replies, and counts the copy read inside one turn.
- `train-in-space --data-root DIR --turn-census TRAIN PANEL LINED`: the option that runs it. No other mode changes.
- `ORDER.md`: the review's findings (before the counts) and amendment 1 (below).
- One notebook row.

**Amendment 1 to the ORDER, made while counting.**
- The lined panel is `recall=oracle`, not `recall=reply`. The reply-worded line holds the whole reply, so it cannot say what the value is; the oracle line is "Memory: VALUE.".
- Added beside the registered counts: the stop lists 0, 16 and 256; the ceiling with the true turn admitted; the clauses of the turn that holds the value; and what the question's own words pick with nothing learned.

## Measurements

Inputs, written on the laptop by `m-world-recall-corpus` and equal to the recorded files: TRAIN `conversations=60000 seed=13 recall=off` (7,370,634 tokens, `e25775f3…`), panel `conversations=1000 seed=14 recall=off` (121,894 tokens, `b8ad9fe4…`), panel `recall=oracle` (131,619 tokens, `12d7e9df…`). The panel has 3,586 replies: 2,945 general, 79 abstentions, 562 that state a value. Every share is of panel replies.

**1. Labels.** The label is the earlier user turn that holds the most of the reply's content tokens (ties to the later turn).

| Most frequent tokens stopped | Recall replies labelled, of 562 | Labelled turn holds the value | General replies labelled, of 2,945 | Abstentions labelled, of 79 |
|---|---|---|---|---|
| 0 | 562 | 560 | 868 | 39 |
| 16 | 562 | 560 | 587 | 21 |
| 64 (registered) | 490 | 488 | 227 | 5 |
| 256 | 286 | 284 | 28 | 0 |

All 562 values are stated in an earlier user turn. At 64 the label is the latest such turn on 488. Had the model's own turns been candidates, one would have won on 130 of 562; the turn being answered on 59.

**2. Ceiling.** Recall replies whose every value token the copy read names, with one turn admitted.

| Turn admitted | Replies | Key 1, whole value | Key 1, first token | Key 1, tokens after the first | Key 2, whole value | No turn admitted (today), whole value |
|---|---|---|---|---|---|---|
| The labelled turn (it holds the value) | 488 | 135 (0.277) | 161 | 379 | 185 (0.379) | 59 |
| The latest turn that states the value | 562 | 140 (0.249) | 169 | 411 | 185 (0.329) | 59 |

By reply form, labelled turn admitted, key 1:

| Token before the value in the reply | Replies | Whole value | Share |
|---|---|---|---|
| " is" | 274 | 114 | 0.416 |
| "'s" (That's, It's) | 141 | 0 | 0 |
| " in" | 29 | 13 | 0.448 |
| " named" | 16 | 7 | 0.438 |
| "'re" | 14 | 0 | 0 |
| " love" | 11 | 0 | 0 |
| " a" | 3 | 1 | 0.333 |

**3. Where the value sits.** In the turn that states it, the value's tokens stand together on 562 of 562. That turn has one clause on 144, two on 63, three on 73, four or more on 282.

**4. What the question's words pick, nothing learned.** The pick is the latest earlier user turn, or clause, that shares the most content tokens with the question.

| Most frequent tokens stopped | The right turn is the pick, of 562 | The value's clause shares a question token | The value's clause is the pick |
|---|---|---|---|
| 0 | 491 (0.874) | 333 | 310 (0.552) |
| 16 | 486 (0.865) | 314 | 303 (0.539) |
| 64 | 450 (0.801) | 271 | 268 (0.477) |
| 256 | 253 | 148 | 146 |

**5. Holes and reach.** Of 488: a rival turn with the labelled turn's subject words and another value on 2 (later than the label on 1); a turn with the same tokens in another order on 0. Earlier user turns a reply has, TRAIN (215,017 replies): none 60,000; one 51,799; two 40,318; three 28,789; four 12,150; five 6,972; six 6,968; seven or more 8,021; most 8. From the labelled turn's start to the reply's end fits in 192 tokens on 395 of 490 (0.806) and in 256 on 490.

**What the numbers say.**
- Which turn: found. The reply's words name it (0.996 of labels), and the question's words alone pick it on 0.80 to 0.87.
- Which clause: half found by shared words (0.48 to 0.55). The other half are questions that share no content word with the statement ("Where did I grow up?" and "I am from Jaistchukfield."). Shared words cannot find those; something learned must.
- The value itself: always one unbroken run of tokens in its turn. Once its first token is found, reading on from there gives the whole value.
- So the repair is named: **pick the value's place, not its turn, with the question, and read the value from the pick onward, first token included.** Arm r's read re-finds each token by the reply's previous token, which fails in a list and fails when the reply is worded its own way.

## Checks run at the head

- `cargo test --release --locked -p uor-r4-float-train --lib turn_census`: 6 passed. The tests pin the turn split and its mask check, the pairing and its refusal, the tie to the later turn, the copy rule on "it's maya" against "is maya", the clause cut, and one census end to end.
- The three corpora equal the recorded token files by SHA-256 (listed in [the sheet of 10 Nov](../../../../harness/prep/one-model-end-to-end-memory-line-2026-11-10.md)).
- The count output is `local/claude/claude-349/census.out` on the laptop, SHA-256 `3cb5ea82c8f38a28…`; every number above was checked against it after this page was written.
- The census refuses a panel pair that differs beyond its memory lines and a response mask that disagrees with the turns; neither fired.
- Instrument checks inside the count: the value's tokens were found in the reply on 488 of 488 and 562 of 562, and in the stating turn on 562 of 562. The form table's rows add to 488 and 135.
- `uor-harness ratchet d23`, `d25`, `d32`: no additions.

## Skills used

None beyond the project loop. One fresh-context reviewer read the plan before the counts, and one read the census code at the head; both were subagents of this session.

## Still open

- **The repaired arm is a changed design, so it needs the owner's word.** The owner approved one arm, a learned selector for which fact answers. The counts say the selector must point at a place and the value must be read from there. The parts exist (the ask heads' query from the question, the Hamming keys, the found-copy pull); the wiring and the read-on do not.
- The half of the questions that share no word with the statement is what learning has to add. No laptop count sizes it.
- The floor with table sign codes, and the slot key's ceiling, are not counted.
- A label for training needs a stop list near 16 and a way to keep general replies out; the leak at 16 is 0.199.
- With labels as they are, a 192-token window holds 0.806 of labelled turns; 256 holds all.

## Plain summary

The plan was to let the model pick the earlier turn that holds the answer and copy from it. The counts say picking the turn is easy and the copy is what fails: the answer is in a list, or the reply is worded differently from what the user said, and the copy read finds the value on about a quarter of the replies. The value is always one run of tokens, so the fix is to point at where it starts and read on. That is a changed design and is not started.
