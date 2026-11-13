# REPORT: one-model-end-to-end, claude-360 (what a key must see: the find by frame offset)

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Order: [ORDER.md](ORDER.md). PR #3950.

claude-359 showed that an exact find lets the saved models state a told value on 491 to 513 of 578 fresh questions, against 314 to 325 by their own read, and that the exact find is a counted table the model does not own. This unit asks how much of the table's input the find needs, so that the next unit builds a trained key over that input from the project's geometry. The counted find was fitted again on TRAIN under 30 cuts of its input and its census was read on both panels. Laptop only: no model was read, nothing was trained, no pod was used.

Every figure below is in `local/claude/claude-360/cuts-1/` and `local/claude/claude-360/cuts-2/` in the main checkout, one log per cut (kit files, kept out of the repository by the project's rule on run output).

**How to make any row again.** `place-asks ablate --tokens local/claude/claude-349/train-off/train/tokens.u16 --kit local/claude/claude-351/train-places.json --held local/claude/claude-349/panel-off/train/tokens.u16 --held-kit local/claude/claude-352/panel-kit/panel-places.json --held local/claude/claude-357/fresh16/panel-off/train/tokens.u16 --held-kit local/claude/claude-357/fresh16/panel-places.json --slots 2,3` at this head. The fit reads no clock and no random source.

## What did not work, first

1. **Nothing in the model changed.** This unit measured. The trained key is not built; the unit names what to build.
2. **The key the model trains today names the one offset that finds nothing.** The token before a place, read alone, finds 0 of 562 and 0 of 578 true places: every question is refused. The trained key (claude-351, `place_codes` in the float trainer) is the point of exactly that token.
3. **No narrow read knows when nothing was told.** The cut that finds the most true places (offsets 2 and 3) sets a place on 49 of 79 and 35 of 71 never-told questions. With the question cut to its last three tokens, 47 to 79 of 79 and 44 to 71 of 71 never-told questions get a place. Only the whole question, or its last eight tokens, with the whole frame holds it at 13 to 19.
4. **The find is still counts, not geometry (CHARTER §3.1).** A cut fit is the same counting on less input. It says which input holds the information; it does not say a trained key will learn it.
5. A cut is a mask over offsets and a count of a question's last tokens. Whether the order of the offsets matters was not measured: a fit that reads the tokens at offsets 2 and 3 without telling the two offsets apart does not exist in this unit.
6. "True place" is a census of the find, not a reply. claude-359 showed that 58 to 83 found places per model and panel are never spoken.
7. No bar was registered. The second batch of 12 cuts was chosen after the first 18 were read, to locate where the refusal fails. The first panel chose table v4's features in claude-359; the second panel is the clean check, and the two agree on the direction of every row.
8. The `ablate` mode fits before it reads the held splits, so a held kit that does not belong to its tokens is refused only after the fit has run.
9. One new test indexed the table's two row shapes wrongly and failed on its own slip; it was fixed before the first code commit. The review's two cheap findings (a loose `--slots` parser, untested `ablate` refusals) are fixed at this head.

## What changed

- `crates/uor-r4-integer/src/place_asks.rs`: `Cut { slots, question_last }`, with `Cut::ALL` for no cut; `fit_cut`, which `fit` now calls with no cut; `AsksIndex::with_cut` and the reader's `asked`; the masked frame, applied after the link gate on the raw token before the place. A dropped offset is filled as an offset with no token is, so it is the same at every candidate. The fit's labels still read the whole question.
- `crates/uor-r4-integer/src/bin/place-asks.rs`: the `ablate` mode. It fits under the cut of `--slots` and `--question-last`, prints the census of each `--held` and `--held-kit` pair under the same cut, and writes no file.
- A table file and a session carry no cut: `PlaceAsks` has no new field, `AsksIndex::new` reads everything, and nothing outside the tests and `ablate` calls `with_cut` or `fit_cut`.

## Measurements

A frame is eight offsets around a candidate place: 1 to 5 are the tokens before it, nearest first; 6 to 8 are its turn's last tokens, the last first. Each row is one fit on TRAIN (table v4's input, SHA-256 of the tokens `e25775f3…`) and its census on the first panel (562 questions with a told value, 79 never told) and the second panel (578 and 71). The four counts are true place / wrong place / "not told" raised / nothing. "Never told with a place" is the count of never-told questions on which a place was set; lower is better.

### The frame, with the whole question

| Offsets kept | First panel, of 562 | Never told with a place, of 79 | Second panel, of 578 | Never told with a place, of 71 |
|---|---|---|---|---|
| all eight (table v4) | 539 / 14 / 8 / 1 | 13 | 560 / 9 / 7 / 2 | 19 |
| none | 0 / 0 / 559 / 3 | 0 | 0 / 0 / 572 / 6 | 0 |
| 1 | 0 / 0 / 559 / 3 | 0 | 0 / 0 / 572 / 6 | 0 |
| 2 | 514 / 40 / 5 / 3 | 64 | 530 / 40 / 4 / 4 | 48 |
| 3 | 511 / 42 / 6 / 3 | 64 | 532 / 35 / 7 / 4 | 60 |
| **2, 3** | **546 / 11 / 4 / 1** | 49 | **565 / 5 / 4 / 4** | 35 |
| 1, 2 | 493 / 41 / 25 / 3 | 46 | 514 / 41 / 19 / 4 | 36 |
| 1 to 3 | 519 / 21 / 21 / 1 | 33 | 544 / 13 / 17 / 4 | 19 |
| 1 to 4 | 528 / 16 / 17 / 1 | 29 | 551 / 10 / 14 / 3 | 18 |
| 1 to 5 | 538 / 14 / 9 / 1 | 30 | 560 / 9 / 8 / 1 | 26 |
| 6 to 8 (the turn's end) | 0 / 0 / 559 / 3 | 0 | 0 / 0 / 572 / 6 | 0 |
| 1 and 6 to 8 | 207 / 195 / 157 / 3 | 10 | 190 / 215 / 167 / 6 | 3 |
| 1, 2 and 6 to 8 | 498 / 52 / 11 / 1 | 36 | 517 / 47 / 10 / 4 | 28 |
| 1 to 3 and 6 to 8 | 526 / 26 / 9 / 1 | 24 | 555 / 16 / 4 / 3 | 19 |

### The question, with all eight offsets

| Question tokens read | First panel, of 562 | Never told with a place, of 79 | Second panel, of 578 | Never told with a place, of 71 |
|---|---|---|---|---|
| the last 1 | 0 / 0 / 0 / 562 | 0 | 0 / 0 / 0 / 578 | 0 |
| the last 2 | 273 / 50 / 232 / 7 | 19 | 281 / 45 / 244 / 8 | 10 |
| the last 3 | 520 / 22 / 18 / 2 | 47 | 532 / 24 / 22 / 0 | 44 |
| the last 4 | 530 / 18 / 12 / 2 | 41 | 549 / 15 / 13 / 1 | 33 |
| the last 6 | 527 / 14 / 19 / 2 | 22 | 546 / 10 / 22 / 0 | 21 |
| the last 8 | 539 / 14 / 4 / 5 | 15 | 561 / 10 / 5 / 2 | 17 |
| all (table v4) | 539 / 14 / 8 / 1 | 13 | 560 / 9 / 7 / 2 | 19 |

### Both cut

| Offsets kept, question tokens read | First panel, of 562 | Never told with a place, of 79 | Second panel, of 578 | Never told with a place, of 71 |
|---|---|---|---|---|
| 1, the last 1 | 0 / 0 / 0 / 562 | 0 | 0 / 0 / 0 / 578 | 0 |
| 1 and 2, the last 2 | 242 / 53 / 262 / 5 | 43 | 251 / 39 / 280 / 8 | 30 |
| 1 and 2, the last 3 | 518 / 38 / 1 / 5 | 79 | 533 / 32 / 1 / 12 | 71 |
| 1 to 3, the last 3 | 529 / 29 / 1 / 3 | 79 | 551 / 24 / 1 / 2 | 71 |
| 1 to 3, the last 4 | 534 / 19 / 6 / 3 | 65 | 545 / 13 / 19 / 1 | 51 |
| 1 to 3, the last 6 | 509 / 22 / 25 / 6 | 52 | 534 / 15 / 25 / 4 | 38 |
| 1 to 5, the last 3 | 527 / 22 / 11 / 2 | 60 | 536 / 22 / 19 / 1 | 54 |
| 1 to 5, the last 6 | 530 / 16 / 13 / 3 | 38 | 543 / 11 / 24 / 0 | 24 |
| 1 to 3 and 6 to 8, the last 3 | 518 / 27 / 14 / 3 | 66 | 530 / 25 / 22 / 1 | 55 |
| 1 to 3 and 6 to 8, the last 6 | 506 / 28 / 24 / 4 | 40 | 535 / 14 / 26 / 3 | 30 |

### What the counts say

1. **Where the value is: the two tokens before the token before it.** Offsets 2 and 3 alone find 546 of 562 and 565 of 578 true places, more than all eight offsets (539 and 560). Offset 2 alone finds 514 and 530. Offset 1 alone finds none, and so does the turn's end alone: the token before a value is a link word that stands before every value.
2. **How much of the question the find needs: its last three tokens.** Two find 273 and 281; three find 520 and 532; eight find what the whole question finds.
3. **Whether anything was told is another read.** It needs the wide question and it is helped by the whole frame: 13 and 19 never-told questions get a place with all of both, 15 and 17 with the last eight question tokens, 30 and 26 with offsets 1 to 5 only, 49 and 35 with offsets 2 and 3 only. With the question cut to three tokens the refusal is gone.
4. **The read the model trains today has neither.** Its key is the token at offset 1, and its question is the row that asks with what the question carry adds. The census of offset 1 finds nothing with the question's last token, and nothing with the whole question.

### What the next unit builds

- **The key names offsets 2 and 3.** At a candidate place the key's code is made from the points of the tokens two and three before it. Offset 1 stays what it is today, the gate: a place is a row after a link token.
- **The mechanism, from the project's geometry: a turned hold.** The trainer already sums a hold over the earlier rows of a window (`held_term`, and the question carry of claude-335) and already turns a code by the key's rotation (`key_rotation`, plane rotations). The key at a place becomes the two tokens' points, each turned by the key's rotation once for each step of its offset so that the two offsets stay apart, summed, and read as sign codes. The compare does not change: Hamming distance inside each ask head's margin, as `place_find` does it. Nothing of a new kind enters: token points, plane rotations, a sum, signs, a Hamming compare.
- **The question reaches eight tokens back.** The carry already sums earlier rows' question points. The next unit first reads how far it reaches on the place read, and widens it to eight tokens if it is shorter.
- **The find and the refusal are trained and scored apart.** The find is read on told questions (true and wrong places). The refusal is the ask head's margin, read on never-told questions (places set). A unit that reports one count for both would hide finding 3.
- **Labels.** The place kit already gives each recall reply its place (`PlaceTeach`); the pull moves the question rows toward the new key as it moves them toward today's.
- It is a change to the float trainer, whose tests run on CUDA only: a prep sheet with a non-author FIT, then a pod inside the caps.

## Checks run at the head

Run on the laptop at the head that carries this report; one cargo at a time, `--locked --offline`.

| Check | Result |
|---|---|
| `cargo test -p uor-r4-integer --features serving-wasm --lib --bin uor-chat --bin place-asks` | 573 passed, 1 ignored (library); 9 passed (`uor-chat`); 4 passed (`place-asks`) |
| Instrument: `cargo check --release --features integer-reader` | clean |
| `cargo check -p uor-r4-training -p uor-r4-cpu-train` | clean (run before the review's two parser fixes, which touch the tool only) |
| Negative gate: the reader's frame cut set aside | `a_cut_reader_reads_no_offset_it_drops_and_no_question_token_past_its_count` fails on the two candidates' scores; restored |
| Negative gate: the reader's question cut set aside | the same test fails on the question's tokens; restored |
| Negative gates: the fit's frame cut set aside at true places, then at every candidate | `a_cut_fit_counts_no_token_at_an_offset_it_drops_and_no_question_token_it_does_not_read` fails each time on a row past offset 2; restored |
| Negative gates: the fit's question cut set aside at true places, then at every candidate | the same test fails each time on a row that holds the question's first word; restored |
| Negative gates: `none` mixed with offsets accepted; an offset named twice accepted | `a_cut_is_read_from_its_flags_and_written_back_the_same` fails on `--slots none,6` and on `--slots 1,1`; restored |
| After the first six gates, the working diff against the patch saved before them | equal |
| The table fitted with no cut on TRAIN | SHA-256 `6e5e985cedb7d94305d8dd6d5abb7ef247a7f58787f6ea1997f19745a8de1417`, the bytes of v4 |
| `ablate` with no cut | the census of table v4 as claude-359 measured it: 539 / 14 / 8 / 1 and 13 of 79; 560 / 9 / 7 / 2 and 19 of 71 |
| `ablate --slots 2,3` with the tool rebuilt at this head | the row above: 546 / 11 / 4 / 1 and 49; 565 / 5 / 4 / 4 and 35 |
| `ablate` given the first panel's tokens with the second panel's kit | refused, naming both hashes |

Not run as negative gates: `held` and `same_marks`, whose test reads each function's own result.

## Skills used

None beyond the project loop. The reviewer was a subagent of this session.

## Still open

- The trained key over offsets 2 and 3, the question's reach, and the refusal read apart: the next unit, as written above.
- Whether the two offsets must be told apart (finding 5 of the first section).
- From claude-359, unchanged: a found place that is not spoken (58 to 83 questions per model and panel); a test for the session restore's reset of the spoken token; the three "not told" forms; the reply that runs on after the value.
- Whether a count-fitted table may be served in the product is the owner's call (CHARTER §3.1); this unit serves none.

## Plain summary

The exact find of claude-359 reads eight tokens around each candidate value and the whole question. Cut down piece by piece, it shows that the value is found from the two tokens before the token before it (546 of 562 and 565 of 578), that the token just before it tells nothing, and that the question's last three tokens are enough to find it. The key the model trains today reads only that one token that tells nothing. Knowing that a fact was never told is a separate read that needs the wide question. The next unit builds the trained key over the two tokens the census names, as a turned hold read by sign codes and a Hamming compare.
