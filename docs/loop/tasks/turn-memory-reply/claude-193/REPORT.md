# REPORT: turn-memory-reply, claude-193 (the turn memory's reply table: the read extended to the probe's key)

Lab: claude. Order: [ORDER.md](ORDER.md). Issue #3393. This unit is the item's done-when 2. There is no model run; the pre-registered two-seed run is done-when 3, the next unit.

**Cost first.** There is no training or serving measurement here.
- **Per token, with the flag on:** one hash lookup of a key of at most 12 ids, at every reply row past its opening. Where the key is held, one 256-coordinate move.
- **The table at the probe's numbers:** 598,755 keys, about 26 MB at u32 by arithmetic (claude-192).
- **With the flag off:** nothing runs, and nothing a run writes changes.

## What did not work, first

- **The turn anchor alone would not key a later row as the probe did.**
  - `opening_slots` (the `Turn` anchor) takes the first 8 positions after the last BOS or end-of-turn token. At an opening these are the turn's first 8 ids, which claude-154 checked against the probe at every opening.
  - At a later row of a turn shorter than 8 ids, they run on into the reply. A key built that way would differ from the probe's turn-8 key.
  - So training and DEV rows compute the probe's own key (`reply_key`, from the mask), and the memory only looks keys up.
  - A greedy step takes the turn from the anchor at the prompt's last token, the opening row, where the two agree. A test pins that the greedy key equals the corpus key at the same row.
- **A later window cannot see its turn.** The windows are 192 tokens, and a reply that runs into a second window keys there on the window's own tokens, as the probe counted (claude-192's scope note). A test pins this, with a reply that spans two windows of 4.
- **The CUDA tests do not run here.** They compile against the stub `nvcc` (`cargo check --features cuda --tests`), and run on the pod as done-when 3's I-1, as claude-161's did.
- **Clippy found three issues in this unit's first build:** two complex types and one needless clone. Fixed: `ReplyRows` and `ReplyOffsets` are named types, and the test uses `slice::from_ref`.

## What changed

- **`crates/uor-r4-float-train/src/flock_read_memory.rs`:**
  - **`build_reply_table`:** for each key, the commonest target, ties to the smaller id, kept only where its count is at least `min_share` of the key's rows. This is the probe's rule.
  - **`TurnMemory::with_reply`:** adds a reply table and its own per-coordinate weight, starting at 0. An empty table is the control's. It refuses a key it cannot save: empty, longer than `MAX_REPLY_KEY` (20), or ending in the pad.
  - **`set_reply_rows` / `reply_rows`:** a thread-local naming the later reply rows of the next forward, each with its key.
  - **`stored_reply`.**
  - **`apply_readout`** takes the named reply rows and adds `n + w_r ⊙ (c_r − n)` at the held ones. It refuses a row named both as an opening and as a reply row, and skips the reply move entirely when no reply row is named.
  - **`variables`** adds the reply weight. **`reply_weight_mean_abs`.**
  - **Save:** the reply table goes to its own `turn_memory_reply.safetensors`, written only when the memory has one, so `turn_memory.safetensors` is byte-identical with or without it.
  - **Load:** attaches the reply table when its file is there.
- **`crates/uor-r4-float-train/src/geometric_stack_native.rs`:** the readout hook passes the named reply rows. It runs when either kind of row is named; a greedy step past the first names only a reply row.
- **`crates/uor-r4-float-train/src/in_space_long.rs`:**
  - `ReplyRow` carries its window and offset;
  - `reply_key` is the probe's `TurnPosition(1)`;
  - `reply_offsets` gives each window's rows past their openings with their keys;
  - `greedy_reply_key` keys a greedy step;
  - `REPLY_MEMORY_SHARE` = 0.5, the probe's choice;
  - `LongConfig.turn_memory_reply`, left out of `config.json` when false;
  - `start_memory` builds the reply table from TRAIN's rows past their openings (empty under `--turn-memory-empty`), and refuses the flag without `--turn-memory 8`;
  - the run logs the reply table's TRAIN rows and keys, and its TRAIN and DEV rows and hits;
  - training batches, `nearest_reading`'s DEV rows and `greedy_reply`'s later steps all name their reply rows;
  - `--inspect` gains `memory_reply_weight_mean_abs`, left out when there is no reply table.
- **`crates/uor-r4-float-train/src/bin/train-in-space.rs`:** `--turn-memory-reply 0|1`.

## Measurements

No model measurement. The tests, all host unless named:

| Test | What it pins |
|---|---|
| `the_reply_table_keeps_each_keys_commonest_target_at_its_share` | The rule. A share exactly 1/2 is kept at 0.5 and dropped at 0.51; ties go to the smaller id. |
| `a_reply_row_moves_only_when_named_and_its_key_is_held` | Only named, held rows hit; a row past the forward is refused; no reply table hits nothing. |
| `a_reply_table_refuses_keys_it_cannot_save` | The empty, overlong and pad-ended keys. |
| `a_saved_reply_table_loads_and_leaves_the_opening_file_as_it_was` | The round trip. The opening file is the same bytes with and without a reply table, and the reply file is absent without one. |
| `reply_offsets_name_the_rows_past_each_opening_with_the_probes_key` | Keys equal to the probe's `reply_key`, including a reply that spans two windows. |
| `the_greedy_key_equals_the_corpus_key_at_the_same_row` | The greedy step's key against the corpus row's. |
| `the_reply_table_names_what_the_probe_counts` | The memory's table names exactly the non-opening rows, and hits, that the probe counts at share 0.5. |
| `a_row_that_opens_and_copies_counts_in_both_kinds_and_not_the_rest` | The probe's double count (claude-192's review, Low). |
| `a_config_writes_the_reply_table_key_only_when_on_and_refuses_it_without_the_memory` | The config key, and both refusals. |
| `zero_reply_weight_is_no_read_and_the_reply_move_is_its_pull` (CUDA) | Zero weight is no read, bit for bit. At weight 1 the held row lands on the stored point, and no other row moves. A row named twice is refused. The gradient is the pull. |

**The negative gate:** with the reply hit set aside in `stored_reply`, `a_reply_row_moves_only_when_named_and_its_key_is_held` fails (`local/claude/claude-193/neg-hit.out`). Restored, all 13 reply tests pass (`neg-restore.out`).

**For done-when 3's sheet,** an instrument check: on the pinned splits, the run's log should show the reply table at 598,755 keys and DEV's rows past their openings named 9,655 times. These are the probe's numbers (claude-192).

## Checks run at the head

- **`cargo test --locked -p uor-r4-float-train --no-fail-fast`** (host): 116 library tests and 18 in the test binaries passed, 0 failed. The 116 include the 9 new.
- **`cargo check --features cuda --tests --target aarch64-apple-darwin`,** with the stub `nvcc`: it compiles, the CUDA test included.
- **`cargo clippy -p uor-r4-float-train --no-deps --all-targets -- -D warnings`:** no finding in this unit's lines. Those left are `decode_speed` (27 Oct), `flock_read_register.rs` and `spiralcore_fbs_sheet.rs`.
- **`cargo fmt -p uor-r4-float-train --check`:** clean.
- **`uor-harness loop gates --full`:** 11 of 11 green.

## Skills used

- **holding-the-goal.** The read keys with the probe's own function, so what the run measures is what the probe counted; where the two might drift (the greedy step), a test pins them equal.

## Still open

- **Done-when 3:** a pre-registered two-seed run (a sheet with a fresh FIT). claude-161's (m) recipe with `--turn-memory-reply 1`, against an empty-table control, barred on top-1, copies and the rest against claude-161 (m), with the openings held. Its I-1 runs the CUDA tests on the pod, and its log check reads the 598,755 keys.
- **Integer serving of the reply table:** `turn-memory-integer`, step 2 of the owner's order.

## Plain summary

The model's memory of past conversations used to help only with the first word of a reply. This adds a second table to that memory, for the rest of the reply, keyed exactly as last unit's count chose. It is built into training, into scoring on the test set, and into generating replies, and switched on by one flag. With the flag off the model and its files are exactly as before.

Tests check that the new table finds the same answers the count found and moves the model only where it should. The training run that measures whether it makes replies better is next.
