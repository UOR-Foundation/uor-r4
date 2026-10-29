# REPORT: turn-memory-integer, claude-198 (the export carries the memory, or refuses)

Lab: claude. Order: [ORDER.md](ORDER.md). Refs #3392. The item stays `ready`.

**Speed.** No serving change yet: the session refuses an artifact with a memory until units 2 and 3 serve it, so decode speed is the trunk's, as before.

## What did not work, first

- **Nothing is served yet.** This unit packs the memory into the artifact and reads it back. The session refuses that artifact, by name ("this engine does not serve the turn memory yet (uor-r4.lut-stack/7)"). The session's keys and moves are units 2 and 3.
- **The first reader had two gaps, found on a re-read before the review and fixed (`00ac639af`):**
  - **It bounded a reply key's turn by the opening table's `k`.** The float trainer keys the reply table on the turn's first `TURN_KEY` (8) ids, whatever `k` is (`in_space_long.rs`, `ReplyKey::TurnPosition`). The snapshot's reply table and the header record now carry the turn length (`turn`, `reply_turn`), and the reader bounds by it.
    - **This was not reachable from today's trainer.** The review pointed out that `in_space_long` refuses `--turn-memory-reply` unless `--turn-memory` is `TURN_KEY` (`in_space_long.rs:421`), and claude-194's runs used 8.
    - So the change makes the artifact name its own key form, instead of relying on that coupling. It fixes no checkpoint that exists.
  - **It accepted an empty opening key** (a row of pads). The float memory never stores one, and the reader now refuses it.
- **The first refusal named only the Hamming read. The fresh review found four more reads it let through.** `in_space_long`'s final checkpoint also saves the Flock, content and opening reads and the prompt register beside the trunk. `load_reads` reloads them, and the export would have dropped them silently.
  - The check is now an allow-list. The trunk (`model.safetensors`), the optimizer's state (`optimizer.safetensors`, training only) and the turn memory pass. Any other `*.safetensors` is refused, naming every one.
- **No other read can be exported yet.** `fbs-snapshot` now refuses a checkpoint holding any of those five reads, instead of dropping it. claude-194's checkpoints hold a Hamming read, so they cannot be exported until `integer-hamming-read`'s step 4 lands.
- **The reply keys are padded to 20 ids, where 12 would do.** The ORDER fixes the row at the float `MAX_REPLY_KEY` (20). A reply key is at most `reply_turn + 4` ids, 12 today. At about 574,000 reply keys (the ORDER's figure), the estimated cost is:
  - 46 MB of reply keys at 20 ids a row;
  - about 28 MB at 12.

  These are estimates from the row width, not measurements. The record carries `reply_key_len`, so a later unit can narrow the row without a new schema field.
- **The snapshot's JSON at that size was not measured.** The stop rule for an over-large handoff was not hit, because no real memory was exported in this unit (the export needs the GPU, and the checkpoints hold a Hamming read). The estimate is about 100 bytes a reply entry, or about 60 MB.

## What changed

- **The snapshot** (`crates/uor-r4-integer/src/fbs_snapshot.rs`):
  - `FbsMemorySnapshot` holds `k`, `key_window`, the opening entries and weight, and an optional `FbsReplyTable` (its turn length, entries and weight);
  - it is optional on `FbsTrunkSnapshot`, with `serde(default, skip_serializing_if)`, so every snapshot written before it parses and serializes unchanged;
  - the key-shape constants `MEMORY_REPLY_KEY_LEN` (20), `MEMORY_MAX_K` (16) and `MEMORY_PAD` (`u32::MAX`) live here, where the float crate can see them.
- **The export** (`stack/format.rs`):
  - the schema `uor-r4.lut-stack/7` for a stack with a memory, with or without Hamming reads;
  - a header record `memory` (`StackTurnMemory`): `k`, `key_window`, the counts, the reply turn (`reply_turn`), `reply_key_len` and `weight_exp`, checked at parse;
  - the sections `memory.opening.{keys,values,weight}` and `memory.reply.{keys,values,weight}`. Keys are `u32` rows padded with `u32::MAX`, and weights are `i32` at `2^-24`;
  - the export re-reads what it wrote through the reader. So it never writes a memory the reader would refuse.
- **The reader** (`stack/memory.rs`, `read_memory_uorlut01`) checks:
  - every section's length;
  - the keys, strictly ascending as unpadded rows;
  - the key forms: an opening key of 1 to `k` ids; a reply key `turn (0 to reply_turn ids), MAX, length (1 to key_window), MAX, last id`;
  - every id and value, below the vocabulary.
- **The session** (`stack/session.rs`) refuses an artifact with a memory, by its text.
- **The float side:**
  - `TurnMemory` gains read-only accessors: `opening_entries`, `opening_weight` and `reply_parts`, in the key order `save` writes;
  - `fbs_memory_snapshot(memory, key_window)` (`fbs_export.rs`) builds the snapshot, with the reply turn `TURN_KEY` (now `pub(crate)` in `in_space_long.rs`), and refuses a weight that is not finite;
  - `check_checkpoint_reads(dir)` refuses a checkpoint holding any `*.safetensors` besides the trunk, the optimizer state and the turn memory, naming each;
  - `fbs-snapshot` runs that check, loads the memory with `TurnMemory::load`, and carries it with the 192-token key window.

## Measurements

None: an export and its checks. The tests below are the evidence.

## Checks run at the head

- **Done-when 1, the round trip** (`memories_round_trip_in_lut_stack_7`). Five memories export as `/7` and read back every key, value, weight, record and schema:
  - opening only;
  - both tables;
  - `k = 4` beside reply keys of up to 8 turn ids (the test asserts that some key has more than 4);
  - an empty opening table;
  - both tables beside Hamming reads.
  The trunk without its memory still serves.
- **Done-when 2, unchanged without a memory.** `snapshots_without_hamming_reads_are_unchanged` now also checks that no `memory` key is written. claude-188's pinned sha256s pass in the full suite.
- **Done-when 3, the refusals by their text:**
  - the session, in the round-trip test;
  - through the exporter's re-read (`malformed_memories_are_refused_by_their_text`): keys out of order, a repeated key, a key id past the vocabulary, an empty opening key, a value past the vocabulary, a reply key without its separators, a reply length past the key window, a reply turn longer than the record's, a reply turn of 0 or 17, a key longer than `k`, a weight that does not fit, `k = 0`, and a short weight;
  - the reader alone: a header whose count does not match its sections, a reply count without its turn, and a memory record on a dense stack;
  - `check_checkpoint_reads` (`a_checkpoint_read_the_export_does_not_carry_is_refused`). A directory of the trunk, the optimizer state, both memory files and a `config.json` passes. Adding each of the five reads is refused, naming it, and two at once are both named.
- **Done-when 4, the float side on the CPU** (`a_saved_memory_snapshots_as_it_loads`). The snapshot of a memory equals the snapshot of its saved and loaded copy, and its JSON reads back equal. `the_memorys_key_shapes_are_the_integer_artifacts` pins `MAX_KEY`, `MAX_REPLY_KEY` and `PAD` to the integer crate's constants.
- **Done-when 5, the negative gates.** Each check was set aside, its test was run, and the check was restored:
  - N1, the session's refusal: `memories_round_trip_in_lut_stack_7` FAILED ("opening only: expected the memory refusal, got None");
  - N2, the export's re-read: `malformed_memories_are_refused_by_their_text` FAILED ("opening keys out of order: … got None");
  - N3, the checkpoint refusal (the Hamming read marked carried): `a_checkpoint_read_the_export_does_not_carry_is_refused` FAILED;
  - N4, the empty-key refusal: `malformed_memories_are_refused_by_their_text` FAILED ("an empty opening key: … got None");
  - N5, the reply turn bounded by `k` again: `memories_round_trip_in_lut_stack_7` FAILED ("k = 4 beside reply keys of 8 turn ids: … entry 0 is not a key this engine reads").

  N1 to N3 ran at `ce1d3c61c`, and N4 and N5 at the fix before `00ac639af`; each test passed again once its check was restored.
  The review's further gates, each with a reader check set aside, all ran red in `malformed_memories_are_refused_by_their_text`:
  - the ascending-order check;
  - the reply key's separators;
  - its length bound (refused for order instead, so the expected reason did not match);
  - the value check.

  So the exporter has no check of its own that would hide the reader's. Setting the allow-list back to the Hamming read alone turned `a_checkpoint_read_the_export_does_not_carry_is_refused` red at `flock_read.safetensors`.
- **Done-when 6:**
  - `cargo test --locked -p uor-r4-integer --features serving-wasm --no-fail-fast`, at `00ac639af` with `~/.cargo/bin` first on PATH: 574 passed, 35 failed. All 35 panic on a missing local input (a quaternion or Householder bundle, or `tokenizer.json`), the same count and kind as `main` in claude-181 to claude-194. Every memory test passes. The integer crate is unchanged after `00ac639af`.
  - `cargo test --locked -p uor-r4-float-train --no-fail-fast`, at `e19cf3676`: 138 passed, 0 failed.
  - The stub-nvcc check, at `e19cf3676`, `CUDARC_CUDA_VERSION=12080 NVCC=<stub> UOR_NVCC_ARCH=compute_120 cargo check --offline --locked -p uor-r4-float-train --features cuda --bin fbs-snapshot --lib --tests`: it builds.
  - The serving audit, `audit_zero_matmul_serving.py --stack` on the release `uor-r4-stack`, because `session.rs` changed: FULL PASS, with 0 forbidden instructions across 44 symbol ranges and 109 reachable functions. The allowed scalar multiplies and divides are 47 / 1 in the ranges and 102 / 10 in the call graph.
  - `uor-harness loop gates --full`, at `e19cf3676` with this REPORT: green, 11 gates GREEN.

## The review

Fresh context, in two slices of under 450 lines each, given the item, the ORDER and the diff.
- **Slice A, the integer export and reader, at `00ac639af`: APPROVE WITH NITS.** Its nits, all left as they are:
  - the record does not name the reply key's form. The trainer builds the reply table with the one form `TurnPosition(1)` (`in_space_long.rs`, `reply_key`), and the reader accepts only that form;
  - a malformed memory gets the reader's error, not the session's refusal text. It is refused either way;
  - the export parses its own output a second time when a memory is present. That costs one parse, and it is what makes the export unable to write a memory the reader refuses.
- **Slice B, the float side and the tests, at `00ac639af`: REQUEST CHANGES.**
  - Finding 1, the four unrefused reads: fixed (`cfa887520`).
  - Finding 2, that the reader checks might be shadowed by the exporter: the gates above show they are not.
  - Finding 3, the leaking temp directories: fixed with a guard that empties a directory before use and removes it on drop.
  - Its nit about `turn` is in the first section.
- **Round 2, the delta at `cfa887520`: APPROVE WITH NITS.** Both findings are fixed. Every read file the crate writes ends in `.safetensors`, no legitimate checkpoint is refused, and the check runs before anything loads. Its nits, made at `e19cf3676`:
  - a lone `turn_memory_reply.safetensors` passed and would be dropped, because `TurnMemory::load` needs the opening file. It is now refused (gate G-e red with it set aside);
  - the test named only the five known reads, so a deny-list would pass. It adds `future_read.safetensors` (gate G-f: a five-name deny-list turns it red);
  - the check now runs before the GPU is opened, and a stale comment is fixed;
  - the name sort is not pinned. With the sort set aside, the three-name test still passes (gate G-g), because APFS lists these names in order. The sort stays, for other filesystems.

## Skills used

- None was invoked this unit.

## Still open

- **Unit 2, the session's keys:**
  - the turn anchor after the last `<|bos|>` or `<|eos|>`;
  - the caller's signal for where the reply begins;
  - the 192-token key blocks from the conversation's start, whatever the session's context.
- **Unit 3, the moves:** `n + w(c − n)` at the readout, for each table.
- **Narrowing the reply rows** to `reply_turn + 4` ids, if the artifact's size matters on the devices it ships to.
- **`integer-hamming-read`'s step 4,** before any of claude-194's checkpoints can be exported.

## Plain summary

The model's exact memory can now be packed into the integer file and read back exactly. The memory is the table of what each question's reply begins with, and what comes next. The integer engine does not use it yet, and it says so instead of quietly ignoring it. Using it is the next two units.
