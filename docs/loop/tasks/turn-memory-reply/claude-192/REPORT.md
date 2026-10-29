# REPORT: turn-memory-reply, claude-192 (the probe: which memory key could carry a reply past its first word)

Lab: claude. Order: [ORDER.md](ORDER.md). Issue #3393. This unit is the item's done-when 1. It is a count over the pinned corpus on the laptop, with no model and no GPU (CHARTER §4.5). The item is not done: done-when 2 (the read, extended) and 3 (the pre-registered run) follow.

**Cost first.**
- **The probe:** 6.7 s and 0.94 GB on the M1 laptop.
- **The chosen key's table:** 598,755 keys at share ≥ 0.5, each the turn's first 8 ids, a position, one reply id and a stored id, about 11 ids. At u32 that is about 26 MB, by arithmetic; claude-161's opening table has 26,450 keys.
- **Per token in use:** one key and one hash lookup at every reply row, and a 256-coordinate move where the key is kept. claude-161's read did this at openings only.

## What did not work, first

- **A key without the position collapses the reply onto one answer.** "Turn-8 + reply-last-0", the turn alone, keys every row of a reply the same. It names 13,913 rows but gets only 1,455 right, and nothing reaches share 0.5.
- **No key is accurate enough at share 0.** The keys with the most hits at share 0 fall below the 0.83 bar:
  - turn-8 + position + reply-last-1: 9,691 / 12,873 = 0.753;
  - turn-8 + reply-last-2: 9,619 / 12,593 = 0.764.

  The share threshold is what makes a key usable.
- **At share 0.5 the openings fall.** The chosen key names 734 openings and gets 470 right, against 1,194 and 601 at share 0. The extended read should therefore keep claude-161's opening table as it is and add the new key for later rows.
- **The keys see the window, not the whole turn.** `hs::build_document_windows` splits a document's targets into windows of 192 (`hamilton_sheet.rs`, about lines 1352–1358). In a later window the walk back stops at the window's first token, so the prompt comes out short or empty, and the position counts from the window's start. "Turn-8 + position" is therefore the turn and position as the window shows them: claude-161's "prompt its window shows", the same in TRAIN and DEV. The read in done-when 2 sees the same window, so the count holds as a ceiling for that read. How many rows this touches is counted in done-when 2's sheet.
- **The reach depends on DEV's turn openings recurring in TRAIN.** 1,194 of DEV's 1,226 openings have their first-8 prompt prefix in TRAIN. On a panel whose turn openings TRAIN never saw, the memory would name far fewer rows. claude-161's result has the same scope.
- **The probe cannot say what the trunk gets right on the rows the memory names.** It counts the memory alone. Whether it adds to the trunk is done-when 3's run.

## What changed

- **`crates/uor-r4-float-train/src/in_space_long.rs`:**
  - **`reply_rows`:** each DEV or TRAIN reply row. Each row holds:
    - the prompt its window shows for its turn, from the reply's start back to the last response token or BOS;
    - the reply's own earlier tokens;
    - its target;
    - whether it opens the reply (`opens_reply`) or copies the prompt (`copies_prompt`).
  - **`ReplyKey`:** none; `Reply(j)`, the reply's last `j`; `Turn(j)`, the turn's first 8 and the reply's last `j`; `TurnPosition(j)`, the turn's first 8, the reply position and its last `j`. Parts are joined by `u32::MAX`, which no id takes.
  - **`reply_bound_rows`:**
    - The table keeps each key's commonest TRAIN target (ties to the smaller id) where its share is at least `min_share`.
    - DEV rows are counted per kind (all, openings, copies, rest): rows, named, hits.
    - `opening_hits_with_fallback` scores openings as `opening_bound` does.
  - **`reply_bound`:** the keys none; reply-last-1–4; turn-8 + reply-last-0–4; turn-8 + position + reply-last-0–1. Each is run at shares 0, 0.5, 0.8 and 0.95.
- **`crates/uor-r4-float-train/src/bin/train-in-space.rs`:** `--reply-bound 1`, one JSON line per key and share.
- **Three host tests:**
  - `reply_rows_carry_the_turn_the_reply_so_far_and_their_kinds`: rows, prompts, replies, the opening and copy flags, and the key forms;
  - `the_reply_bound_keeps_each_keys_commonest_target_at_its_share`: the table, the tie rule, the share threshold, the per-kind counts;
  - `the_reply_bound_falls_back_to_the_commonest_opening_only_for_openings`.
- **The probe's output:** [`reply-bound.jsonl`](reply-bound.jsonl), 48 lines.

## Measurements

**The instrument against claude-161:**
- turn-8 at position 0 names **1,194** of DEV's 1,226 openings. That equals claude-161's read hits on DEV openings (its I-3, `1,194 / 1,226`).
- With the fallback, it names **607** right, claude-161's probe ceiling.
- TRAIN has 2,425,925 reply rows, and DEV 14,287: 1,226 openings, 4,130 copies, 9,094 rest. These match `NearestReading`'s kinds; 163 rows are both openings and copies.

**The decision rule** (the ORDER, fixed before the run): the most DEV hits on non-opening rows, among keys whose precision on the rows they name is at least 0.83.

| Key | Share | Named (all) | Hits (all) | Precision | Non-opening hits | Rest | Copies |
|---|---|---|---|---|---|---|---|
| **turn-8 + position + reply-last-1** | **0.5** | 10,389 | 9,292 | **0.894** | **8,822** | 7,853 | 1,037 |
| turn-8 + reply-last-2 | 0.5 | 10,387 | 9,247 | 0.890 | 8,777 | 7,850 | 995 |
| turn-8 + reply-last-3 | 0.5 | 10,099 | 9,185 | 0.909 | 8,715 | 7,828 | 955 |
| turn-8 + reply-last-4 | 0.0 | 10,983 | 9,302 | 0.847 | 8,701 | 7,792 | 986 |
| turn-8 + reply-last-4 | 0.5 | 9,782 | 9,069 | 0.927 | 8,599 | 7,745 | 922 |
| turn-8 + position + reply-last-1 | 0.8 | 8,957 | 8,645 | 0.965 | 8,400 | 7,560 | 867 |
| reply-last-4 (no turn) | 0.5 | 8,982 | 8,089 | 0.901 | 8,089 | 7,285 | 804 |
| reply-last-3 (no turn) | 0.5 | 9,220 | 8,064 | 0.875 | 8,064 | 7,221 | 843 |

**Chosen: turn-8 + position + reply-last-1 at share ≥ 0.5.**
- **Non-opening rows:** it names 9,655 and gets 8,822 right (0.914).
- **By kind:** copies 1,037 of 1,573 named (0.659), and the rest 7,853 of 8,169 (0.961).
- **Against the no-turn control:** the turn adds about 730 rows (8,822 against 8,089).

**For comparison:** claude-161 (m), the trunk with the opening memory, gets these right (from its REPORT):
- copies 1,713 / 2,333 of 4,130;
- the rest 7,255 / 7,512 of 9,094.

That is the model, not a lookup, so the two are not additive.

**The next unit (done-when 2).** Keep claude-161's opening table. Add a table keyed by turn-8 + position + reply-last-1 at share ≥ 0.5, read at every reply row past the opening, with the same move `n + w ⊙ (c − n)`. Host and CUDA tests in claude-161's style.

## Checks run at the head

- **`cargo test --locked -p uor-r4-float-train --lib -- reply_ bound`:** 7 passed, the three new tests among them.
- **The negative gate:** with the tie rule reversed (ties to the larger id), `the_reply_bound_keeps_each_keys_commonest_target_at_its_share` fails (`local/claude/claude-192-tmr/neg-tie.out`). Restored, all 7 pass (`neg-restore.out`).
- **`cargo fmt -p uor-r4-float-train --check`:** clean.
- **The pinned splits:** loaded through `load_train_split` and `load_dev`, which check every sha256 pin. DEV's `tokens.u16` begins `0919e90f953e988e`.

- **`cargo test --locked -p uor-r4-float-train --no-fail-fast`** (host): 125 passed, 0 failed.
- **`cargo clippy -p uor-r4-float-train --no-deps --all-targets -- -D warnings`:** no finding in this unit's lines. The one in `in_space_long.rs` is `decode_speed`, dated 27 Oct.
- **`uor-harness loop gates --full`:** 11 of 11 green.

## Skills used

- **holding-the-goal.** The decision rule and its 0.83 bar were committed in the ORDER before the probe ran. The instrument was checked against claude-161's own numbers before any key was read.

## Still open

- **Done-when 2:** the read, extended to the chosen key, with host and CUDA tests.
- **Done-when 3:** a pre-registered two-seed run against an empty-table control, barred on top-1, copies and the rest against claude-161 (m), with the openings held.
- **The scope notes:** the window-limited keys, and the turn openings that recur in TRAIN. Both are counted in the run's sheet, the latter on a fresh panel too.

## Plain summary

The model can already pick the first word of a reply from a stored table keyed by the start of the question. This unit counted what a stored table could do for the rest of the reply, before any training run.

The best key is the start of the question, where we are in the reply, and the reply's last word, stored only where it is usually right. On the test conversations it gets about 8,800 of the later reply words right, about nine times in ten when it answers. Leaving the question out, keying on the reply alone, does noticeably worse. The next step builds this table into the model and trains it.
