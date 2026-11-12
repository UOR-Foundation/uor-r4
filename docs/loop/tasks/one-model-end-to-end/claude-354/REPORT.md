# REPORT: one-model-end-to-end, claude-354

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Order: [ORDER.md](ORDER.md). PR #3944.

A laptop-only hunt, approved by the owner on 12 Nov ("Yes, hunt it"), for why the laptop copies still give other replies than their trained models on about a quarter of the rows. No pod, no training, no float pass. The five models are the four sealed stage 3 models of [claude-352](../claude-352/REPORT.md) and arm r's export; no artifact and no reader code is changed.

**No second reader fault was found.** The laptop copy and the trained model choose the same token on 98.4% to 99.0% of the panel's 40,719 scored rows and are right equally often. They part where the model itself is nearly undecided between two tokens, and one such token in a reply of about fifteen is enough to make the whole reply read differently.

**What the models say first.** The first seven replies of the panel, in panel order, right or wrong, where the place read's laptop copy (p-s2) leaves its trained model. A long repeat is cut with "…". "Rank" is the place of the trained model's token among the laptop reader's 4,096 choices at that token; "gap" is how much farther it is than the reader's own choice, in the reader's distance units (a typical row's first choice leads its second by 503,070).

| Asked (the stated value) | Trained model | Laptop copy | First differing token | Trained / laptop | Rank | Gap |
|---|---|---|---|---|---|---|
| (no value asked) | wushodeer buldreeststst… | wushodeer buldreeststast stoudhutlur ramdark / Assia s s s… | 11th | "st" / "ast" | 2 | 2,875 |
| "Where did I grow up?" (Jaistchukfield) | You grew up in Jaistant: You grew up in in in… | You grew up in Jaistantelly ba know. | 9th | ":" / "elly" | 11 | 3,643 |
| "What was fotpimbrud again?" (waspoumbroud) | It's b all of them. | It's boof. | 4th | " all" / "oof" | 2,583 | 24,529 |
| (no value asked) | That's great news about the camera! | That's great news about the like like like… | 8th | " camera" / " like" | 14 | 12,377 |
| (no value asked) | The capital of Norway is blue. | The capital of Norway is M's's's… | 9th | " blue" / " M" | 2 | 2,228 |
| (the stated value is folfain) | It's S all them. | It's white. | 3rd | " S" / " white" | 5 | 2,972 |
| (no value asked) | zourkferknat / Assh | zourkferknat thanksvaibim pourkferknat | 8th | " / " / " thank" | 2 | 797 |

Neither side is right in any of the seven. Most differing replies are like these: both replies are already poor, and the two readers fall to different poor continuations.

| Gate | Reading | Status |
|---|---|---|
| G1 | The table of step 1 for all five models is under Measurements. | **met** |
| G2 | No fault was found, so there is no change and no test of one. | **does not apply** |
| G3 | No fault was found, so no model was re-read with a changed reader. | **does not apply** |
| G4 | How far each model is from 0.10, and what is not explained, is under Measurements and Still open. | **met** |

## What did not work, first

- **Nothing is repaired.** Every reply the laptop gave before this unit it gives now. The share of replies that differ from the trained model's is unchanged: 0.27, 0.25, 0.18 and 0.22 for the four stage 3 models and 0.21 for arm r, against a bar of 0.10.
- **The bar cannot be met by this export on replies of this length.** With one token in 61 to 96 chosen differently (table 1) and replies of 12 to 16 tokens, about a fifth to a quarter of the replies must differ somewhere.
- **The control loses stated values on the laptop and the place read does not** (79 to 69, against 183 to 183, 210 to 209 and 173 to 170). The control's loss is 14 replies lost and 4 gained; that lean is not explained beyond table 4.
- **Where the rounding comes from inside the reader is not measured.** It would take the trained model's own vectors at the differing rows, which no saved file holds.
- claude-352's registered reading stays NOT_RUN. No bar or status is changed.

## What changed

- A new laptop program, [`row-picks`](../../../../labs/one-model-end-to-end/instrument/src/bin/row-picks.rs), in the item's instrument crate. It reads integer artifacts only.
  - **Rows:** it cuts a panel into the windows the float scoring pass cuts, steps the laptop reader through each with the true text before every row, and writes the reader's choice at every scored row beside the pod's saved choice (`rows-p-memory.json`, `rows-p-no-memory.json`). It refuses to write unless its row positions and targets equal the pod's file.
  - **Replies:** for each decoded reply that is not the trained model's, it feeds the prompt and the tokens both replies share, checks that the reader's next token is the saved laptop reply's, and writes the rank and the gap of the trained model's token.
- No change to the reader (`crates/uor-r4-integer`), to any artifact, or to the float side.

## Measurements

Laptop only. The reader is built from `de0881fa2`; `row-picks` SHA-256 `1a9a30d1274bde9ab3f633b917310d53c76dbbc82b572ee588558682a1421305`. The panel is claude-352's no-line panel (tokens SHA-256 `b8ad9fe4…5fe4`, the SHA the pod's score names), read at the 192 window the pod scored at. The trained models' numbers are the pod's saved files; the float column of table 1 equals the pod's own receipts (34,625 for the control, 33,543 for arm r).

**1. Row by row, with the true text before each row (40,719 rows a model).**

| Model | Turn memory | Rows chosen differently | Share | Right: trained model | Right: laptop copy |
|---|---|---|---|---|---|
| Control (n-s2) | on | 504 | 1.2% | 34,625 | 34,618 |
| Place read (p-s2) | on | 670 | 1.6% | 34,838 | 34,802 |
| Place read (p-s1) | on | 520 | 1.3% | 34,968 | 34,971 |
| Place read (p-s3) | on | 597 | 1.5% | 34,823 | 34,775 |
| Arm r (r-s2) | on | 424 | 1.0% | 33,543 | 33,535 |
| Control (n-s2) | off | 1,521 | 3.7% | 31,303 | 31,286 |
| Place read (p-s2) | off | 2,003 | 4.9% | 32,531 | 32,412 |
| Place read (p-s1) | off | 1,676 | 4.1% | 32,643 | 32,509 |
| Place read (p-s3) | off | 1,888 | 4.6% | 32,683 | 32,555 |
| Arm r (r-s2) | off | 983 | 2.4% | 30,281 | 30,223 |

With the memory off the first token of a reply is the weak row: 630 to 713 of the 3,586 first tokens differ on the stage 3 models (283 on arm r), against 31 to 34 (90 on arm r) with it on. The turn memory decides the first token; without it the model is near a tie there. So the turn memory hides differences and does not cause them.

**2. The differing rows are the rows where the reader's first two choices are nearly tied** (turn memory on). Each cell is rows chosen differently of rows in the band; the band is how far the reader's first choice leads its second.

| Model | under 1,000 | 1,000 to 4,000 | 4,000 to 16,000 | 16,000 to 64,000 | 64,000 and up |
|---|---|---|---|---|---|
| Control (n-s2) | 210 of 364 | 173 of 521 | 78 of 797 | 38 of 1,542 | 5 of 37,495 |
| Place read (p-s2) | 191 of 322 | 235 of 601 | 177 of 1,041 | 65 of 5,844 | 2 of 32,911 |
| Place read (p-s1) | 164 of 340 | 184 of 555 | 118 of 703 | 50 of 3,805 | 4 of 35,316 |
| Place read (p-s3) | 211 of 389 | 209 of 740 | 147 of 999 | 29 of 4,858 | 1 of 33,733 |
| Arm r (r-s2) | 184 of 330 | 133 of 519 | 69 of 697 | 35 of 1,385 | 3 of 37,788 |

About half differ in the nearest band, about a third in the next, and 1 to 5 rows of more than 32,000 in the last. A typical row's lead is 476,967 to 584,048 (the median); a differing row's is 1,411 to 2,331. On the differing rows the trained model's token is the reader's second choice on 263, 285, 261, 272 and 236 rows, its third to fifth on 132, 181, 158, 155 and 111, its sixth to twentieth on 73, 122, 84, 123 and 59, and beyond on 36, 82, 17, 47 and 18. Rows where the target is copied from the prompt differ least: 13 to 88 of 10,556.

**3. The G1 table: where a decoded reply first leaves the trained model's** (1,108 replies a model, turn memory on, as the models are served). Every one of the 1,249 replies was fed again and the reader gave the saved laptop reply's token each time.

| Model | Replies that differ | First leaves at the 1st token | 2nd to 4th | Later | Trained model's token is the reader's 2nd choice | 3rd to 5th | 6th to 20th | Beyond | Gap (median) |
|---|---|---|---|---|---|---|---|---|---|
| Control (n-s2) | 300 (0.27) | 16 | 109 | 175 | 146 | 78 | 52 | 24 | 3,831 |
| Place read (p-s2) | 275 (0.25) | 21 | 73 | 181 | 146 | 67 | 32 | 30 | 4,716 |
| Place read (p-s1) | 197 (0.18) | 17 | 57 | 123 | 110 | 59 | 21 | 7 | 5,982 |
| Place read (p-s3) | 245 (0.22) | 20 | 52 | 173 | 131 | 58 | 42 | 14 | 5,161 |
| Arm r (r-s2) | 232 (0.21) | 82 | 100 | 50 | 122 | 59 | 39 | 12 | 2,055 |

What the trained model says at that token with its turn memory off:

| Model | The same token as with the memory on | Not comparable: its memory-off reply had already left | The laptop's token | A third token |
|---|---|---|---|---|
| Control (n-s2) | 67 | 233 | 0 | 0 |
| Place read (p-s2) | 96 | 179 | 0 | 0 |
| Place read (p-s1) | 39 | 151 | 6 | 1 |
| Place read (p-s3) | 80 | 163 | 1 | 1 |
| Arm r (r-s2) | 202 | 28 | 0 | 2 |

In at most 7 replies of any model could the turn memory have a part in the first differing token.

Two checks tie table 3 to table 1 (laptop kit, not committed; see Checks):

- Where the trained model's reply follows the panel's text up to the differing token, that token is one of table 1's rows. There are 59, 68, 51, 58 and 104 such replies, and every one is among table 1's differing rows.
- Table 1's rates, taken on the rows of the decoded replies and multiplied out over each laptop reply's length, give 202, 292, 221, 274 and 226 differing replies, against 300, 275, 197, 245 and 232 seen. Four agree within 13%. The control has 98 more than the rates give: its replies leave the panel's text most often, and off that text more rows are near a tie.

**4. Who is right where they differ** (turn memory on).

| Model | Rows chosen differently | Trained model right | Laptop copy right | Neither |
|---|---|---|---|---|
| Control (n-s2) | 504 | 69 | 62 | 373 |
| Place read (p-s2) | 670 | 106 | 70 | 494 |
| Place read (p-s1) | 520 | 58 | 61 | 401 |
| Place read (p-s3) | 597 | 95 | 47 | 455 |
| Arm r (r-s2) | 424 | 49 | 41 | 334 |

The trained model is right on 377 of these rows and the laptop copy on 281: the rounding costs 96 rows of 203,595 (0.05%). Stated values of 562 at 192, trained then laptop, with the replies lost and gained: control 79 and 69 (14 lost, 4 gained); place read 183 and 183 (8, 8), 210 and 209 (6, 5), 173 and 170 (6, 3).

**What this rules out.** A wrong window, a cut row or a shifted row (the rows equal the pod's, 40,719 of 40,719 on ten runs); the turn memory (table 3); a fault that acts on a kind of row, such as copied rows or first tokens (tables 1 and 2: the differing rows follow the lead, not the kind); a fault of the place read alone (the control and arm r, which have none, differ as much).

**What a float trace would have to record** to go further: at table 1's differing rows, the trained model's final vector before the nearest-token choice, and each read's kept position. The first gives the size of the rounding a coordinate; the second says how many of the far cases (rank beyond the twentieth, 17 to 82 rows a model) are a read that kept another position. The float scoring pass has a query cache option; this unit did not check what it holds. Either way that is a pod run and is not asked for here.

## Checks run at the head

| Check | Result |
|---|---|
| `cargo test --locked --release --bin row-picks --features integer-reader` (instrument crate) | 5 passed, 0 failed |
| `cargo build --locked --release --bin row-picks --features integer-reader`, tree clean at `de0881fa2` | built; the SHA-256 above |
| the ten row runs: positions and targets against the pod's files | equal on every run (the program refuses otherwise) |
| the nine reply runs: the reader's token against the saved laptop reply's | equal on every differing reply (the program refuses otherwise) |
| the float column of table 1 against the pod's score receipts | equal where a receipt is on the laptop (control 34,625; arm r 33,543) |
| the program's breakdown against the kit's first count of the same files | equal |

Not run: anything on a pod, any float pass, the integer crate's tests (its code is not changed). The two tying checks under table 3, the memory-off column of table 3 and the stated-value counts are counts by the laptop kit (`jq` files under `local/claude/claude-354/` and `local/claude/export-repair/`), not by committed code. `uor-harness loop pr-check` runs after this commit and its result is in the review record on the PR.

## Skills used

None beyond the project loop. The reviewer was a subagent of this session.

## Still open

- **The whole-reply bar.** 0.10 of replies differing measures the length of a reply as much as the copy. A row-by-row check with the true text (this program, about a minute a model on the laptop) with the right counts beside it measures the copy. Whether a later sheet registers that is the owner's call; nothing is changed here.
- **The control's lean** (14 stated values lost, 4 gained) and its 98 replies beyond what the rates give.
- **The far cases:** 17 to 82 differing rows a model where the trained model's token is beyond the reader's twentieth choice. A read keeping another position would look like this; it needs the float trace above.
- **A float reading at the 256 window** does not exist (carried from claude-353).
- claude-352's open items on the place read itself are untouched.

## Plain summary

The laptop copies are faithful. Token by token they choose what their trained models choose on about 99 rows in 100 and are right as often. Where they differ, the model was nearly undecided between two tokens and the laptop's rounding tipped it the other way; because a reply is many tokens long, about one reply in four ends up worded differently, usually where both replies were already poor. Nothing in the laptop reader needed repair, and nothing was changed. The laptop numbers of claude-353 stand as readings of the models: at the full window the place read says the stated value on 238, 269 and 223 of 562 and the control on 76. What stands between this and a model worth talking to is the trained model's own replies, not the copy.
