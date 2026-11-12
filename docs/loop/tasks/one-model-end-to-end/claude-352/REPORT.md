# REPORT: one-model-end-to-end, claude-352

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Order: [ORDER.md](ORDER.md). Sheet: [the place read, stage 3](../../../../harness/prep/one-model-end-to-end-place-read-2026-11-11.md). PR #3936.

Stage 3 of the place read (arm p), as the owner released it on 11 Nov ("fine run it i guess..."): one pod, four trainings of 16,384 steps (the place read at seeds 2, 1 and 3, and one control without it at seed 2), then the readings on the laptop from the sealed checkpoints. No memory line is written for any model here and the chat's memory helper is off.

**What the models say first.** The first recall conversations of the panel, in panel order, right or wrong, at the full window. "Place read" is p-s2's integer artifact, "control" is n-s2's:

| Asked (the stated value) | Place read | Control |
|---|---|---|
| "Where did I grow up?" (Jaistchukfield) | You grew up in Ja is Drekmen | You grew up in. You haven't told me yet. |
| "Do you remember what kadtaistfit was?" (446) | That's 44. | That's great news about the dog |
| "What was fotpimbrud again?" (waspoumbroud) | It's boof. | It's bed |
| "Who is my friend?" (Tikgeer) | Your friend's name is Tiktplplik. | Your friend's name is is is iss. |
| "Do you remember what trirchest was?" (19, stated eight turns earlier) | That's 19 | That'sle feel like liker from feel help help help help with. |
| "What color do I love?" (green) | You love? I I'll remember that. | You love with withinginging you you you … |
| "Tell me what woutdai was." (93) | It's 93. | It'se for |

The three scripts, every reply of every model, are in [transcripts.md](transcripts.md). In the four-fact script the place read answers one direct question of four at seeds 2 and 1 and none at seed 3; the control answers none.

| Gate | Reading | Status |
|---|---|---|
| G1, step 2 | try-2's `pre` stage passed I-p, I-g (21 tests, 0 failed), I-3, I-s, I-r and I-b. try-1 had stopped at its scoring check (decision 7) | met on try-2 |
| G2, the four jobs sealed, I-t | four seals in the store, sums read back, each EXIT 0 after 16,384 steps with its snapshot and config | met |
| G3, the pod down | the pod is gone from its provider. I did not take it down: the laptop was off when the chain ended (see below) | met, not by this session's hand |
| G4, the readings in the REPORT | below, with [transcripts.md](transcripts.md) | met |
| G5, the row | **row 0 at I-f: the registered reading is NOT_RUN.** The float and integer replies differ on more rows than the sheet allows, and the control's artifact does not say what its trained model says (decision 11) | read; an instrument failure |
| G6, ratchets | d23, d25, d32: no additions | met |

## What did not work, first

- **The registered reading is NOT_RUN (row 0, I-f).** The sheet allows the float replies of the pod's scoring pass and the integer artifact's replies to differ on at most 0.10 of the rows. They differ on 589 (p-s2), 565 (p-s1), 665 (p-s3) and 851 (n-s2) of 1,108. Arm r's own merged export, read the same way on the same prompts, differs on 232 of 1,108 (0.21), so the bar, set from arm r on the panel with a line, is one no artifact meets on this panel. The control is worse than a bar set on the wrong panel: its trained model says the stated value in 79 of 562 recall replies and its artifact says it in none. The cause is not found. It lies in what only the control uses: ask heads serving as type heads at the readout, with no place read.
- **So the comparison I first gave the owner, 207 against 0, is not the place read's gain.** The 0 is the broken export. On the trained models, scored on the pod at the 192 window, it is 183 against 79 of 562 (seeds 1 and 3: 210 and 173).
- **The model does not hold a four-fact conversation.** Script L, four direct questions: 1, 1 and 0 of 4 at seeds 2, 1 and 3 (bar 3). The one it gets is the lucky number, and at seed 2 it says it as "Your dog is named 47".
- **The place read adds nothing where the question does not name the key.** For "Who is my friend?", "What is my dog's name?" and "Where did I grow up?" the register is not set in 22 of 28, 24 of 28 and 32 of 32 replies (p-s2, full window), and the trained place read says the value no more often than the trained control (dog 7 and 6 of 28, place 14 and 14 of 32). Its whole gain is on keyed facts.
- **Copies break inside the value.** "44." for 446, "88." for 882, "veerpleel." for veerpleelzos, "Tiktplplik" for Tikgeer. With the register at the true place and the row open, the reply holds the value in 94 of 174 (p-s2).
- **The sheet's prediction for exact R missed.** Exact replies at the full window: 37, 44 and 23 of 562 (0.066, 0.078, 0.041), under the predicted range. Exact match was then shown to be the wrong instrument (the hand read, k = 11 of 20, cut 5): a reply like "That's 19" is wrong by exact match against "That's 19.".
- **try-1 was NOT_RUN.** The sheet registered a scoring command the pin refuses (`--score-rows --context 256`). The scoring pass ran at 192 on try-2 (decision 7). My bound on what that costs was the wrong measure (decision 8): 113 of the 562 recall prompts lose the stated value at 192, so I-l's panel half was not met there (449 of 562) and the recall replies were retaken on the laptop at 256 (decision 9).
- **The run-context artifact cannot take the 256 prompts** ("the session is full (256 positions)"): the 256 readings use each model's context-8256 artifact. On the 192 prompts the two differ on 8 (p-s2) and 16 (n-s2) of 1,108 rows; R is the same.
- **The laptop went down from about 22:57Z to 00:12Z and took the watcher with it.** The pod's chain sealed all four jobs alone (last EXIT 00:15:05Z). The lease expired at 00:52:08Z; at 05:05Z the pod was gone from its provider. Its end time is not known to me. If it billed to 05:05Z the run cost at most 8.1 h × $0.908 = $7.35, inside the approved $16. The promise to take the pod down by hand was not kept.
- **What I-w cannot refuse.** Every panel conversation asks for one value, so "no other stated value" has nothing in the kit to refuse. Conversations do state values that are never asked. A bound: of p-s2's 207 right replies, 26 hold any word from an earlier user turn that the reference and the question do not have; read by eye they are "is", "a", "now", "to" and one key name, no second value.
- **Seen before it was registered:** an uncommitted preview of the whole-word count (207, 236, 160, 0). Decision 10 says so. The registered program gave the same four counts.
- **Slower to serve.** Decode on the laptop (Apple M1, two runs each, the same minute, load average about 10): place read 1,360 and 1,202 tokens/s, arm r 1,562 and 1,422. Prefill of the read: 4,261 and 4,010 against 3,788 and 3,548.
- **Not counted apart:** the 126 recall replies that claude-350's census lists as out of the find's reach. No per-row list of them exists.
- **Disk.** The laptop ran out of space during the readings. With the owner's yes I deleted three build folders of the Antigravity lab (6.5 GB of cargo output, git-ignored, no process using them). Decision 9's own time is wrong by its commit (05:13Z).

## What changed

- `docs/labs/one-model-end-to-end/instrument/src/bin/whole-word-recall.rs` (new): the sheet's I-w score, with its four-reply test.
- `docs/labs/one-model-end-to-end/instrument/src/bin/raw-id-replies.rs`: `--place-rows` prints the register (the place held and offered) at every reply row, with the true place from a kit (`--places`); `--kit-tokens` binds the kit, its corpus and the rows; every argument is a name and value pair.
- `crates/uor-r4-integer/src/bin/uor-chat.rs`, `stack/chat.rs`, `stack/tests.rs`: `--print-place` prints the register of the last reply. Print only; the replies of a run with it equal the pinned program's on all three scripts of every model.
- `crates/uor-r4-float-train/src/in_space_long.rs`, `bin/train-in-space.rs`: `--panel-prompts` takes `--context` and `--sample-opening`, so the laptop writes the scorer's own prompts at any window. No training or scoring path changes.
- No bar, status, scope or stop rule changed. Nothing was trained beyond the four registered jobs.

## Measurements

**Sealed runs** (store, `results/claude/one-model-end-to-end-place-read-2026-11-11/run/STAGE/try-2`, source `93e089ff256654e75a19641b7e2fbd2bc8e24524`). Each job: 16,384 steps, about 2,000 s of training, EXIT 0.

| Stage | Artifact at the run's context | Artifact at context 8256 | Place sections |
|---|---|---|---|
| p-s2 | `c94da18f2b3cb9872bfa53c9f24fbbd5f53d3ba4109560ace9a3f8ea0503f75b` | `ae4055a765d60ab44afa7aad0f927a6b321a3a1fc176daf8691acc144316ebe7` | 1 |
| n-s2 | `196261a90bc694b263b9af61454497af5c23cec5ad69bfbb4d368ccca1a278b8` | `f9fa134ebe1763ad48720ddb0fa352aced05109b17f21e83aae1604edd4eb14b` | 0 |
| p-s1 | `ade24df5a1df6aad1bf5150c582fc5e40c7b5a290d06516a2b2cf68356785b1f` | `14c44f899863e437328df8c970430906f6b85d5bf81b7ef0d44ab26f83b72141` | 1 |
| p-s3 | `17a92a95063bb50cb66625fde75c6ec139b8045a68e536732789b1646062fbd5` | `22857c36b25596c4f35804ad233c1e7a24b4a29c784509d0c818e8a8f6c0c50f` | 1 |

**L** (script `L`, four direct questions, the value as a whole word): p-s2 1, p-s1 1, p-s3 0, n-s2 0, arm r without a line 0. Bar 3.

**R, the recall replies that say the stated value** (562 recall replies; I-w's rule, program `whole-word-recall` at `dcd87899c`, SHA-256 `6844852f0bda7d1cd7b0212c5f04a515aa11c0e300d832429d2acf38b134489a`; all 562 references are right by the rule at 256):

| Model | Integer, 256 window: exact | Integer, 256: says the value | Integer, 192 | Trained model (float), 192 | Float with its copy read off, 192 |
|---|---|---|---|---|---|
| p-s2, place read | 37 | **207** | 164 | **183** | 157 |
| p-s1, place read | 44 | **236** | 191 | **210** | 163 |
| p-s3, place read | 23 | **160** | 125 | **173** | 127 |
| n-s2, control | 0 | **0** | 0 | **79** | 37 |
| arm r seed 2 (trained with a line, no line here) | not read | not read | 10 | 9 | 10 |

At 192, 113 of the 562 prompts do not hold the stated value; no model, float or integer, says the value on any of them. At 192 the references are cut at the window, and 544 of 562 are right by the rule; the 192 columns are a description, not the registered reading. The float replies exist only at 192: the pod's scoring pass ran there, and a float pass is GPU work.

**G.** Registered form, integer replies at 256: (207 − 0) / 562 = 0.368 for seed 2, 0.420 and 0.285 for seeds 1 and 3 against n-s2. It is not a reading: its control is the broken export, and I-f is row 0. Trained models at 192: (183 − 79) / 562 = 0.185, and 0.233 and 0.167. Bar 0.10. **Had I-f held, every seed would read row 3 on either side** (L under 3, G at or above 0.10: "the panel shows the value and the script does not"), replicated.

**I-f, float against integer, rows of 1,108 that differ at 192** (each reply cut at its end mark): p-s2 589, p-s1 565, p-s3 665, n-s2 851; arm r seed 2 232. Rows whose first id differs: 28, not read, not read, 140; arm r 82.

**By the kind of question** (rows with the value in the 192 prompt, 449; says the value):

| Kind | Trained control | Trained place read, seed 2 | Integer place read, seed 2 |
|---|---|---|---|
| a key from a list ("What was kadtaistfit again?") | 22 of 267 | 99 of 267 | 86 of 267 |
| code word | 5 of 33 | 19 of 33 | 22 of 33 |
| lucky number | 16 of 28 | 20 of 28 | 20 of 28 |
| friend | 5 of 28 | 9 of 28 | 7 of 28 |
| dog | 6 of 28 | 7 of 28 | 7 of 28 |
| place | 14 of 32 | 14 of 32 | 8 of 32 |
| color, job, food | 11 of 33 | 15 of 33 | 14 of 33 |

**The register by the kind of question** (p-s2, integer, 256 window, all 562): at the true place at some row of the reply: list keys 336 of 380, code word 33 of 33, lucky number 26 of 28, friend 5 of 28, dog 1 of 28, place 0 of 32, color, job, food 8 of 33.

**The split** (register at the value-start row, wrong replies by exact match, 256 window): p-s2 a 146, b 82, c 119, d 143, short 35: the find side 228, the read side 262 of 490, so the read side is named. p-s1: 141, 79, 109, 151, 38. p-s3: 146, 71, 111, 159, 52. I-l's panel half at 256: a true place for 562 of 562. By I-w, replies that say the value, by class, p-s2: a 48 of 152, b 30 of 82, c 21 of 119, d 94 of 174, short 14 of 35.

**The hand read** (done once; p-s2, the read side, its first 20 wrong replies in panel order; register is [held, offered] at the value-start row). k = 11, cut 5. The program's verdict agrees with the hand read on all 20.

| # | Row | Class | Register | True place | Value | Reply | Holds the value |
|---|---|---|---|---|---|---|---|
| 1 | 5 | d | [8,8] | 8 | `Tikgeer` | `Your friend's name is Tiktplplik.` | no |
| 2 | 8 | d | [25,25] | 25 | `19` | `That's 19` | yes |
| 3 | 13 | c | [29,null] | 29 | `836` | `Wendbrundstout handgel is 836.` | yes |
| 4 | 14 | d | [25,25] | 25 | `folfain` | `It's folfain is the capital of haven't yourisna.` | yes |
| 5 | 17 | d | [45,45] | 45 | `numjast` | `It is numjast` | yes |
| 6 | 23 | c | [11,null] | 11 | `986` | `The after Wednesday is 986.` | yes |
| 7 | 24 | c | [9,null] | 9 | `vudwost` | `The code word is is vudwost` | yes |
| 8 | 25 | d | [13,13] | 13 | `veerpleelzos` | `It is veerpleel.` | no |
| 9 | 26 | c | [85,null] | 85 | `chaimpaim` | `It isadaim.` | no |
| 10 | 37 | c | [16,null] | 16 | `4732` | `Lairch says s s s s s …` (the same token to the cap) | no |
| 11 | 43 | d | [15,15] | 15 | `21` | `It's 21.` | yes |
| 12 | 54 | d | [24,24] | 24 | `brostfar` | `It's festrul thank right,,,'ll all that.` | no |
| 13 | 56 | c | [65,null] | 65 | `stourkgroul` | `Taikdreerk.` | no |
| 14 | 65 | d | [27,27] | 27 | `34` | `That's 34` | yes |
| 15 | 69 | c | [13,null] | 13 | `joutrid` | `Hedtions` | no |
| 16 | 70 | d | [12,12] | 12 | `882` | `It's 88.` | no |
| 17 | 71 | d | [21,21] | 21 | `drudrat` | `That's drudrat,'ll them` | yes |
| 18 | 75 | d | [14,14] | 14 | `920` | `It's 92.` | no |
| 19 | 78 | c | [102,null] | 102 | `80` | `Your lucky number is 80.` | yes |
| 20 | 86 | d | [90,90] | 90 | `susttreem` | `The code word is susttreem` | yes |

**Script L's wording against the corpus** (row 3's first suspect, read though row 3 is not called): three of L's four direct questions are forms of the panel's own user turns ("What is my friend's name?" 11 times, "Where did I grow up?" 12, "What is my lucky number?" 14; "What is my dog called?" 0, "What's my dog called?" 9). L fails where the panel fails: the kinds of question the register is not set for.

**Other instrument checks.** I-o: arm r's sealed seed 2 snapshot, exported with the pinned `fbs-export`, is `707dfdc3b564df95497d57303decccebe42ac382045d7a0aae461137fec18ad0`, arm r's sealed artifact. I-c: each `config.json` differs from arm r's in the context, the training corpus, `hamming_ask`, the place read and its pull (p only) and the seed, and in nothing else; p's names the kit. I-t: above. The builder of the 256 prompts was accepted as decision 9 says: at 192 its 1,108 rows equal the pod's in position, prompt and reference; at 256 all 562 recall positions are among its 990 rows, each 192 prompt the end of its 256 prompt.

## Checks run at the head

| Check | Result |
|---|---|
| `cargo test --release --locked --bin whole-word-recall` (instrument crate) | 5 passed |
| the same with the whole-word test in `holds` set aside | 2 failed (`the_four_hand_built_replies`, `a_word_is_matched_whole_and_by_case`), then restored and 5 passed |
| `cargo test --release --locked --features integer-reader --bin raw-id-replies` | 11 passed |
| `cargo test --release --locked -p uor-r4-float-train --lib prompts` | 4 passed |
| `cargo test --release --locked -p uor-r4-float-train --lib the_last_opening_starts` | 1 passed |
| `cargo test --release --locked -p uor-r4-integer --features serving-wasm --lib stack_chat_records_the_place_register` | 1 passed |
| `uor-harness ratchet d23`, `d25`, `d32` | no additions (4451 of 4451; 0; 31 of 31) |
| the builder at 192 against the pod's samples | 1,108 of 1,108 rows equal |
| I-w on the references at 256 | 562 of 562 right |

The two float trainer rows did not compile on their first run at this head ("multiple different versions of crate `serde_core`"). The instrument crate is its own workspace and locks serde 1.0.229; the main workspace locks 1.0.228; I had built both into one target folder, and the trainer's test linked the other workspace's copy of `uor-r4-integer`. After `cargo clean --release -p uor-r4-integer` in that folder both rows pass. No source changed.

Not run on the laptop: the float trainer's GPU tests (they ran on the pod as I-g) and any float pass.

## Skills used

None beyond the project loop. The reviewers were subagents of this session.

## Still open

- **The export of an ask-head model without a place read.** Why n-s2's artifact loses what its trained model says is not known. Finding it needs the float trace of one sealed checkpoint, which a GPU writes; the retake from the sealed checkpoints follows it. Until then no integer control exists for this arm.
- **I-f's bar on this panel.** 0.10 was set from arm r on the panel with a line. On this panel arm r itself differs on 0.21. A bar that fits needs its own decision before the next reading, not after this one.
- **The find for questions that do not name the key.** The register is not set for them; the trained control gets as many of them as the place read does.
- **The copy inside a long value** stops early about half the time when the register is right.
- A float reading at the 256 window does not exist.
- Whether the pod ended at its lease or later is not known to me.

## Plain summary

Four models were trained on one rented machine and read on the laptop. The model with the place read, asked about something said earlier in the conversation with no memory line written for it, says the right value in 207 of 562 test conversations from its laptop artifact, and in 183 of 562 as trained (measured on a shorter window). The same model trained without the place read says it in 79 of 562 as trained. So the place read more than doubles what the model recalls, and the gain is in facts the question names by their key ("What was kadtaistfit again?"). It does not help with questions like "Where did I grow up?", and the model cannot yet answer four facts in a row in a typed conversation: it gets one of four.

One of my own checks failed. The laptop copy of the comparison model does not behave like the model that was trained: it says no values at all, where the trained one says 79. So the laptop comparison of 207 against 0 is not a fair one, and by the rules written before the run the formal result is "not run" until that copy is fixed. The fair comparison I have is 183 against 79.
