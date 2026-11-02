# REPORT: m4-export-uorlut01-cli, claude-259 (chat runs on the trained model)

Lab: claude. Item: [m4-export-uorlut01-cli](../../../../roadmap/items/m4-export-uorlut01-cli.md) (#2859). Order: [ORDER.md](ORDER.md). References #2859.

Plan step: PLACE.md's 2 Nov path, step 1 (chat runs on the trained model); the 29 Oct order's step 3, the text path, which feeds step 4. What it changes in the model: nothing trained. It changes the serving path only: `uor-chat` now serves the FBS artifact claude-243 trained (`a7r-sN.uorlut`) with the pick the model was trained to make and with its exact turn memory marked each turn. Before this unit, `uor-chat` refused every FBS artifact on its first reply. The item stays open: its 0.90 bpb bar, the export gap and the restore of a saved chat are not touched here.

## Tokens per second

Decode, `uor-chat ARTIFACT --prompt-ids` on a 14-id prompt, timed as the wall time of a 177-token run minus a 1-token run, so 176 decode steps over contexts 15 to 191. One thread, this M1, on AC power.

| Artifact | Decode |
|---|---|
| a7r-s1 | **1,318 tok/s** |
| a7r-s2 | 1,361 tok/s |
| a7r-s3 | 1,335 tok/s |

These sit beside claude-243's 1,204 tok/s for s1 from `fbs-toks`' own timer, which ran under a load average of 2.6 to 3.0; they were not measured under equal load. The REPL's own "id/s" line (180 to 980 in the transcripts) counts the prompt's feeding too, so it is not a decode number.

## What did not work, first

- **The replies are not a working chat.** This is the trained model's own behavior, now visible because the chat serves it. In 30 turns over 3 seeds and 2 conversations ([conv-a](#measurements): facts then questions; [conv-b](#measurements): small talk):
  - **No grounded answer.** "What is my lucky number?" after "My lucky number is 47." gets "4, 4, 4," (s1, s3) or "4, branches." (s2). The reply starts with the value's first digit, then loops. s1's "Where did I grow up?" gets "I grew up in Lisbon.", but that is token for token its reply to "I grew up in Lisbon." one turn before, so this panel cannot tell it from a copy.
  - **Later turns repeat the first reply.** s1 and s3 give "4, 4, 4," to all four of conv-a's first turns; s2 gives "4, branches." to all five turns before its context reset. In conv-b, s2 answers "hello!", then "1, hello!", then "1, 1, hello!" twice; s3 answers "hello" and then "1, 1, 1," three times.
  - **The user's line comes back.** "hello" opens every conv-b first reply; "I grew up in Lisbon." is answered with itself (s1).
  - **Broken words.** "Telllele" (s1-b), "Where of of of" (s2-a), "I grew word Nokaykay there!" (s3-a).
  - **The one fluent sentence is memorized.** s1-b's "hello" turns go on "Assistant. I can chat, remember what you tell me, and follow simple instructions.": the tail of a self-description reply in the training world ([milestone_world.rs:465](../../../../../crates/uor-r4-training/src/milestone_world.rs)), glued to the echo and a role word.
  - **12 of 30 turns end on the short-cycle stop**, 18 on EOS. Four of the six conversations ran out of the 192-position context before their last turn ("The turn needs 198 of 192 positions") and restarted.
  - The training world's acknowledgements ("Okay, I'll remember that.", "Got it.", "Noted.") never appear. The pattern fits a copy head that copies what followed the previous "Assistant: " rather than reading the question. That is a reading, not a measurement; grounded replies are step 3 of the path.
- **A doubled end-of-turn id, older than this unit.** `StackChat::feed` pushed the pending generated id into the history a second time, so every turn after the first carried two EOS ids (one in the record, one in the context count). The session itself stepped the right tokens, so the replies were unaffected; the history record and the context arithmetic were each off by one per turn. Fixed (28fbdb86b) and pinned by `stack_chat_keeps_a_pending_terminal_id_once` (gate G4).
- **Side findings, not fixed here:**
  - The old chat module doc said pointer-head models are served through the mixture's scores, but the code read `step`'s logits. The claim is removed; no code served the mixture before or after.
  - `save_state` does not carry the turn memory's reply start or anchors, so a restored chat loses the mark. That is the item's restore work (M4).
  - `uor-chat --prompt-ids` feeds raw ids with no roles, so it sets no mark. It serves the right pick but not the turn memory's move.
  - The notebook README says to append rows at the bottom; m4-serving.md lists them newest first. The new row follows the page.

## What worked

- **The chat equals the reference decode on every turn.** For all 30 turns, the reply `uor-chat` printed is the same token list as `fbs-toks --replies`' decode (the library `panel_reply`) of the same history: s1-a 6/6, s1-b 4/4, s2-a 6/6, s2-b 4/4, s3-a 6/6, s3-b 4/4. After the single-copy fix, each turn boundary reads `16, 1, 201, 55` (".", one EOS, "\n", "U").
- **The two causes, fixed.**
  - `StackChat` read `step()`'s logits and took their argmax. FBS's `step()` returns an empty slice (its pick comes from the native head), so the chat stopped with "score width 0, vocab 256". It now calls `serve_step`, which returns the dense argmax for a dense stack and the native head's pick for FBS.
  - Nothing marked the reply start, so the turn memory's reply table never moved the readout. `StackChat` now calls `mark_reply_start` before stepping a prompt's last token, and clears the mark before each new prompt, matching training, where only reply positions take the reply table's move.
- **Negative gates.** Each was run with the fix set aside and then restored (G1 to G3 checked byte-identical against the saved patch):
  - G1, the whole chat.rs fix reverted: 3 tests fail with `ScoreWidth { found: 0, vocab: 256 }`.
  - G2, no mark: 3 tests fail.
  - G3, no reset before a seeded replay: fails on "seeded key [32, 149, 54, 197]".
  - G4, the old double push: `stack_chat_keeps_a_pending_terminal_id_once` fails.
- **Tests:** `cargo test -p uor-r4-integer --features serving-wasm --lib stack_chat`, 9 passed, including the four new FBS tests on a synthetic FBS trunk with a synthetic turn memory: served with its memory, marked before the prompt's last token, marked each turn, and one copy of a pending id.

## What changed

- [crates/uor-r4-integer/src/stack/chat.rs](../../../../../crates/uor-r4-integer/src/stack/chat.rs): picks with `serve_step`; marks the reply start before a prompt's last token and clears it before each prompt; resets before replaying a seeded history into a memory model; keeps one copy of a pending id. The unused `ScoreWidth` error is gone.
- [crates/uor-r4-integer/src/bin/uor-chat.rs](../../../../../crates/uor-r4-integer/src/bin/uor-chat.rs): the banner and records name the decoding (`greedy serve_step`) and print the turn memory's table sizes; `--prompt-ids` steps with `serve_step`.
- [crates/uor-r4-integer/src/stack/tests.rs](../../../../../crates/uor-r4-integer/src/stack/tests.rs): the four FBS chat tests and a sized chat tokenizer.
- [docs/research/notebook/m4-serving.md](../../../../research/notebook/m4-serving.md): one row.

Commits: 3ba027cdd (ORDER), 91adaec2e (the fix and tests), 28fbdb86b (one copy of a pending id), and this REPORT.

## Measurements

- **Artifacts:** `local/claude/claude-243/R/a7/a7r-s{1,2,3}.uorlut` in the main checkout (51,310,784 B each, `uor-r4.lut-stack/7`), checked against claude-243's SHA256SUMS. Tokenizer: the W2 driver-data `tokenizer.json` (vocab 4096, ByteBpe).
- **Conversations** (fed to `uor-chat` on stdin, with `/tokens` after every turn):
  - conv-a: "My lucky number is 47." / "What is my lucky number?" / "My best friend is Maya." / "What is my friend's name?" / "I grew up in Lisbon." / "Where did I grow up?"
  - conv-b: "hello" / "How are you today?" / "What can you help me with?" / "Tell me something about the sea."
- **Parity:** each turn's `/tokens` history, cut at the turn's prompt, went into `fbs-toks ARTIFACT --replies PROMPTS --out FILE --reply-tokens 64`, and its reply was compared with the chat's reply ids.
- **Evidence** (local, not committed): `local/claude/claude-259/` in this worktree holds the inputs, the six transcripts `s{1,2,3}-{a,b}.txt`, and the `fbs-toks` outputs.

## Checks run at the head

- `~/.cargo/bin/cargo test -p uor-r4-integer --features serving-wasm --lib stack_chat`: 9 passed, 0 failed.
- The four negative gates above (G1 to G4): each failed with its fix set aside and passed once restored.
- `cargo fmt --check`: no diff. `cargo clippy`: no warning in this unit's lines (the warnings it prints are in older `uor-chat.rs` code).
- The full crate suite, `loop gates --full`, `loop pr-check --local` and the fresh-context review: recorded below once run.

## Skills used

None.

## Still open

- **Grounded replies.** The served model copies its previous reply instead of reading the question. That is the path's step 3, fixed with geometry; these transcripts are its starting evidence, and they say step 3 may need to come before more training at the same recipe.
- **The trained-as-served run** (one-model-end-to-end, claude-259 prep on draft PR #3558): its sheet, FIT, pre-run review and GPU run at 3 seeds, scored with these same two conversations and M2's panels.
- **The item's own bars:** the 0.90 bpb bar, the export gap, and a restored chat keeping the turn memory's mark (`save_state` drops it).
- **`--prompt-ids` has no roles,** so it cannot mark a reply start; a raw-id path that marks would need a role flag.

## Plain summary

The chat program can now talk to the model we trained. Before, it crashed on the first reply, because it looked for the model's scores in a place this kind of model leaves empty, and it never told the model's memory where a reply starts. Now it asks the model for its own pick, marks each reply's start, and gives on every turn the same answer as the reference decoder, at about 1,300 tokens a second on the laptop. What it shows is that the model does not chat yet: it repeats its first reply, echoes the user, and answers no question from what it was told. That is the next problem on the path.
