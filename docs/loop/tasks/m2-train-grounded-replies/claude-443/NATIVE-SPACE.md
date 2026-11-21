# Native-space items: the saved question reader, claude-443 (D85)

The five items `.agents/skills/native-space-cycle/SKILL.md` asks for, written for the reader that read `m2-fresh256` on 21 Nov [claude]. This file says what is built and measured, and what D85 still asks that is not built [claude].

## What is not yet true

- **A second implementation of the read only.** The Rust source compiles for three targets (aarch64 macOS, x86_64 Linux, `wasm32-wasip1`) with equal word streams, which shows it is deterministic across targets. A CUDA kernel now reads the held rows as a second, separately written implementation and equals the host on readers A and B (section 3). It does not read the suffix ladder and it covers one set of read choices. TRAIN has a second implementation too (CUDA kernels, section 3), for the same one set of choices, with no suffix orders and no token codes.
- **Carried by D11, not part of the trunk state.** A D11 artifact now carries the reader (schema `uor-r4.lut-discrete-question-reader/1`) and `discrete_d11` answers from it. It is carried as the reader's own file, byte for byte, under a header that binds it and the parent artifact by SHA-256. It is not one of the positional table sections of `TrunkTables`, and the transported trunk state (S1 of [claude-409](../../written-state-count-model/claude-409/NATIVE-SPACE.md)) takes no part in this read: the key is the question's tokens, not a transported code. The trunk and the reader share an artifact and a vocabulary, and nothing else yet.
- **Only the form key is read from the trunk state, and it costs replies on the corpus panels.** Since 21 Nov a reader can key its question forms by the trunk's transported frame (section 6). Every other key of the reader is still a token id, the GPU kernels refuse such a reader, and on the corpus-worded panels it answers 15 and 5 fewer replies exactly than the token-keyed reader. The readers the M2 numbers rest on are token-keyed.
- **The carrier is not built in CI.** It is behind the integer crate's `reader-serving` feature, which no CI job selects. Its test and the builds below ran on the laptop. The integer crate with the feature was only type-checked for `wasm32-unknown-unknown`; the browser client does not call it.
- **GPU TRAIN is slower than the host, and the host still lays out its work.** The owner's answer of 21 Nov ("No, GPU first") asked for a GPU writer; `reader_train.cu` now writes reader A's file byte for byte on wick's 4090 (section 3). It took 55.4 s of kernel time where the host's `Reader::train` took 2.1 s, because replies that share a question form must be written in order and the widest group of replies that share none was 17. The host finds the reply runs, groups the pair positions and orders the launches; it computes no row. A wider exact schedule is not available: a held row's key holds the reply-so-far, whose symbols depend on what the reader answered at the place before, so the rows a reply will touch are not known before it is written.
- **The file does not hold the whole learned state.** `to_bytes` leaves out every slot that was right fewer than `ROW_RIGHT` times and every row with no such slot. No reply changes (measured below), but TRAIN cannot be continued from the file: the left-out counts are gone. A whole-state save is not written.
- **No gradient.** Learning is counting and the written verdicts (right and wrong counts a slot). There is no gradient or optimizer state in this line, so D85's gradient clause has nothing to check here.
- **Dev panels only.** The replay ran on `m2-reply512` and the M-world dev panels p14 and p23. It did not read `m2-fresh256`, which is spent for this reader line.

## 1. State schema

Every field is an integer word; no field is floating point [claude]. Source: `crates/uor-r4-discrete/src/conjunction.rs` and `conjunction/saved.rs`.

| Part | Carried fields | Representation | How it learns |
|---|---|---|---|
| Question forms | the asked turn's kept tokens with one gap, or its opening and closing tokens for an open form | three u128 words (twelve bits a token under a leading one), gap place and length (u8); numbered by a u32 id | counted on the first TRAIN pass; a form is kept once seen often enough |
| Held rows | key = (form id, reply-so-far address, tail, empty-read flag); value = slots | history: two u128 words (thirteen bits a symbol); tail: u32; slots: kind byte, three u8 fields, one u32 value | each TRAIN reply token writes its slot; each slot carries a right and a wrong count (u32) |
| Slot kinds | `Token`, `Pointer {back, ahead, wide, before}`, `Next`, `Copy` | the five words of `entry_fields` | the reader's own verdict at that place decides what is written |
| Stand table | where a relation word and the word before a value stand together (`pair`) | token-pair keys | counted from TRAIN |
| Suffix ladder | orders 3..20 suffix rows, off in the saved readers (`ORDERS -`) | packed keys, u32 tokens, counts | counted |
| Session carry | the cursor (the prompt place the last token was copied from) and the reply-so-far history | `Option<usize>`, the history's symbols | none: it is rebuilt for each reply |

## 2. Shared operator paths

One source definition each; TRAIN and serving call the same functions [claude].

| Operator | Meaning | TRAIN caller | Serving caller |
|---|---|---|---|
| `Asked::place` | the window, the reply start and the carried cursor at a reply place | `write_forms` | `generate` |
| `Asked::answer` | the held answer at a place: the best-standing slot across the held forms' rows (`precise,across`) | `write_forms`, before each write; `write_empty` through `generate` | `generate` |
| `History::push`, `address`, `tails` | the reply-so-far key | `write_forms` | `generate` |
| `Asked::read_under` | the forms a turn is read under | `write_under` (its three form sets) | `reply_traced`, `reply_words` |

The serving caller is the same in every build: the trainer crate (`uor_r4_discrete::conjunction`), the portable crate `uor-r4-reader-runtime` (the same source files, compiled without the GPU pipeline, host and `wasm32-wasip1`) and `IntegerDiscreteModel::question_reply` in `discrete_d11`, which calls the portable crate. The CUDA kernels are not callers of these functions: they are a second implementation held to them word for word (read) and byte for byte (TRAIN), with the form and row lookups shared between the two kernel files in `reader_common.cuh`.

## 3. Replay oracle

`conjunction-probe reader-replay` trains the reader in memory with the arguments that wrote a file, requires the in-memory bytes to equal the file, loads the served reader from the file alone, and replays a panel on both [claude]. For every reply token it compares 36 raw u32 words: the step kind, the form's index among the held forms, the form's own fourteen words (a file numbers forms anew, so ids are never compared), the pass and row empty-read flags, the row's tail and eight history words, the carried cursor, the token ahead of it, the slot's five words, the place the slot read and the token id. A third stream is the read half of a TRAIN pass (`forced_words`: `place` and `answer` as `write_forms` calls them, nothing written) forced over the served reply. Each stream's SHA-256 is reported.

The library test `a_replay_compares_raw_step_words` holds the three streams equal on a toy reader and shows the comparison failing when the served reader's held rows are removed.

Measured on 21 Nov (`prompts differing` counts prompts whose words differ; 0 everywhere):

| Panel | Reader (sha256) | Machine | Prompts | Steps (own row / shared row / ladder) | Replies from empty-read rows | In-memory bytes are the file | Prompts differing, served vs trained | Prompts differing, forced vs served | Words SHA-256 (all three streams) |
|---|---|---|---|---|---|---|---|---|---|
| `m2-reply512` | A `44574adb…972c3c` | laptop, aarch64 macOS | 512 | 2,704 / 3,960 / 0 | 80 | yes | 0 | 0 | `440d7cd0df34c1553fc72dab09030f88a1275f4a94d007f45200f121a46923de` |
| `m2-reply512` | A `44574adb…972c3c` | wick, x86_64 Linux | 512 | 2,704 / 3,960 / 0 | 80 | yes | 0 | 0 | `440d7cd0df34c1553fc72dab09030f88a1275f4a94d007f45200f121a46923de` |
| `m2-reply512` | B `078dcfdc…810c2b` | laptop, aarch64 macOS | 512 | 2,702 / 3,962 / 0 | 8 | yes | 0 | 0 | `4c0a0ed4f76b1d5f3ee5d9c2e247b5b267ff8c56c29dc33b40ea8edf639bde8e` |
| p14 | M-world `83c5f327…48339e` | wick, x86_64 Linux | 1,108 | 11,646 / 1,040 / 1,152 | 302 | yes | 0 | 0 | `4dd4f97e4d50b178e02412f9c0ec2645f63ef5f0f345c0b9f4163e497aa31fa3` |
| p23 | M-world `83c5f327…48339e` | wick, x86_64 Linux | 1,110 | 12,232 / 1,185 / 965 | 291 | yes | 0 | 0 | `7a6b1fccac0d047ed97f0b31f15e445759dd637e5a005412a714c29ed02f0ca8` |

Reader A's stream has one SHA-256 on both machines (same TRAIN `bfbf5812…`, same panel `40e12f87…`). The M2 panel has no ladder step; p14 and p23 carry the back-offs (2,117 ladder steps) and 593 replies read from empty-read rows. Reports: laptop worktree `local/claude/d85-replay/`, wick `~/work/claude/c443/runs/conj/replay/` (binary `conjunction-probe-v32`).

The forced stream is a consistency check of the stepper, not independent evidence: it calls the same two operators in a second loop.

### The same streams from the portable build and from a D11 artifact

Measured on 21 Nov (UTC) on `m2-reply512` (panel `40e12f87…`, 512 prompts, 6,664 reply ids, limit 64) [claude]. The trainer rows are the first and third rows of the table above.

| Read by | Reader A words SHA-256 | Reader B words SHA-256 | Reply ids SHA-256 (A and B) |
|---|---|---|---|
| trainer crate, `reader-replay` | `440d7cd0…a46923de` | `4c0a0ed4…639bde8e` | not hashed by this command |
| `uor-r4-reader-runtime`, host (`reader-words` example) | `440d7cd0…a46923de` | `4c0a0ed4…639bde8e` | `4554ffd9…ffe2a1f6` |
| `uor-r4-reader-runtime`, `wasm32-wasip1` under Node 26 WASI | `440d7cd0…a46923de` | `4c0a0ed4…639bde8e` | `4554ffd9…ffe2a1f6` |
| D11 artifact through `IntegerDiscreteModel::question_reply` (`question-reader-d11 words`) | `440d7cd0…a46923de` | `4c0a0ed4…639bde8e` | `4554ffd9…ffe2a1f6` |

The D11 artifacts are the M1 trunk `seed1.d11` (`04652a3d…2c147d`, 86,536,132 bytes, unchanged inside the carrier) behind reader A (artifact `604fcd03…a7e4dd`, 122,111,038 bytes) and behind reader B (artifact `cf9bf6ee…adce1d`, 122,949,062 bytes). Answering the 512 prompts from an artifact, load included, took 2.0 s on the laptop. Readers A and B give the same reply ids on this panel and differ in which rows they read. Reports: `local/claude/d11-reader/`.

The test `a_carried_reader_replies_as_its_file_and_bad_bindings_are_refused` packs a toy reader behind a toy artifact, requires the reply through the model to be the file's reply, and requires a changed reader byte, a changed parent byte, a reader over another vocabulary, a cut reader file and a carrier inside a carrier each to be refused.

### The same streams from a CUDA read kernel (wick's 4090)

What this does not show, first [claude]. TRAIN is still a host count pass: only the read runs on the GPU. The kernel reads held rows only; a place no held row answers (the host's suffix ladder) and a prompt with no kept form stop the thread with a status, and are counted, not filled in. It is laid out for one set of read choices (pair stand, `precise,across`) and refuses any other reader. No reply is graded by this command, so no M2 count comes from it. The M2 headline is unchanged at 0.

`reader_read.cu` is a second implementation of the read, in CUDA C: one thread a prompt finds the turn's kept forms and each held row by binary search in flat `u32` tables (`conjunction/flat.rs`, built from the reader's file) and writes the 36 raw words of every step. `build.rs` admits its PTX only with no floating-point instruction. `conjunction-probe reader-gpu READER_FILE PANEL_JSON LIMIT OUT_JSON` answers each prompt by the host reader and by the kernel and compares reply ids and words.

Measured on 21 Nov (UTC) on wick (RTX 4090, `sm_89`, cubin `5704d89e…ff506175`, binary `conjunction-probe-v35` `f02d2f46…233b413`), reports in wick's `runs/conj/gpuread3/`:

| Reader, panel, limit | Prompts read on the GPU | Stopped at the ladder | Read prompts that differ from the host | GPU words SHA-256 | Host words SHA-256 |
|---|---|---|---|---|---|
| A (`44574adb…`), `m2-reply512`, 48 and 64 | 512 of 512 | 0 | 0 | `440d7cd0…a46923de` | `440d7cd0…a46923de` |
| B (`078dcfdc…`), `m2-reply512`, 48 | 512 of 512 | 0 | 0 | `4c0a0ed4…639bde8e` | `4c0a0ed4…639bde8e` |
| corpus reader (`83c5f327…`), `p14`, 48 | 927 of 1,108 | 181 | 0 | not the host's (181 prompts unread) | `4dd4f97e…7aa31fa3` |
| corpus reader (`83c5f327…`), `p23`, 48 | 944 of 1,110 | 166 | 0 | not the host's (166 prompts unread) | `7a6b1fcc…d02f0ca8` |

Reply ids from the GPU for readers A and B hash to `4554ffd9…ffe2a1f6`, the host's. The comparison can fail: with every slot's times-wrong count zeroed in the tables, the same kernel differs from the host on 716 (`p14`) and 737 (`p23`) prompts; on `m2-reply512` that control changes nothing (0 prompts), so there the evidence that the comparison can fail is the corpus reader's rows alone. The kernel alone took 2.9 to 3.9 ms for the 512 prompts (6,664 ids) and 12.8 to 17.1 ms for the corpus panels; with context and table load 131 to 193 ms; the host read took 7 to 35 ms. The GPU read is not faster than the host at this size.

### TRAIN as CUDA kernels (wick's 4090)

What this does not show, first [claude]. It is slower than the host: 55.4 s of kernel time against 2.1 s for reader A. The host still finds the reply runs, groups the positions of each token pair and orders the launches. It covers one configuration: pair stand, open forms, `precise,across`, no suffix orders, no token codes. Reader B's TRAIN stream is not on wick, so only reader A was written. The corpus reader was not written: the command refused its stream ("more replies than the form table is sized for", `runs/conj/gputrain3/`), because the form table is sized for the most forms a turn can have and is capped at 2^26 slots, and that reader also uses suffix orders, which these kernels do not write. No reply was graded, so no M2 count comes from it and the headline is unchanged at 0.

`reader_train.cu` is a second implementation of `Reader::train` in CUDA C, seven kernels over `u32` tables. `train_pairs` gives each token pair a thread that replays its positions in stream order. `train_count` gives each reply a thread that counts every form of its turn in a hash table; a slot is claimed by compare-and-swap and its key is read only after the slot is seen ready. `train_keep` and `train_rank` keep the forms seen eight times and number them in key order. `train_forms` finds the kept forms each reply is written under. `train_write` writes a reply under its forms exactly as `write_forms` does: it reads the held answer, then writes the slot. `train_missed` generates each reply from the frozen rows and marks those that met an empty read, which `train_write` then writes under empty-read keys.

The host's order of writes matters only between replies that share a form, so the host numbers each job (a reply under its token forms, or under its open forms) with a level: one past the highest level of an earlier job that shares a form with it. Jobs of one level share no form and run as one launch; levels run in order. That schedule gives the host's rows exactly. It is also why this is slow: reader A's 36,707 jobs fall into 7,030 levels and the widest holds 17 jobs.

`conjunction-probe reader-gpu-train` trains the host reader in memory, trains on the GPU twice and once more with the replies written last first, builds a `Reader` from the GPU tables and compares every form, held row, slot and pair row, then the saved bytes, with the host's and with the file. Measured on 21 Nov (UTC) on wick (RTX 4090, cubin `b001e921…a0dc71e3`, binary `conjunction-probe-v37` `c0fe4876…f75419`), report in wick's `runs/conj/gputrain2/`:

| Reader A, TRAIN stream `bfbf5812…` | Host | GPU |
|---|---|---|
| Replies, forms kept, held rows, pair rows | 20,000 / 1,994 / 486,181 / 116,076 | 20,000 / 1,994 / 486,181 / 116,076 |
| Forms, held rows, slots and pair rows that differ from the host | | 0 |
| Saved reader SHA-256 | `44574adb…972c3c` (the file) | `44574adb…972c3c` |
| Same bytes on a second GPU run | | yes |
| Time | 2,051 ms | 55,448 ms in kernels over 8,042 launches; 55,791 ms with context, table load and read-back |

The comparison can fail. With the replies written last first the same kernels give 84,495 held rows the host does not have, miss 77,787 it has, and differ in the slots of 165,633 more; that reader's bytes (`e33aca1c…468b8340`) are not the file's. The read kernel, given the GPU-trained file, gives reader A's step words `440d7cd0…a46923de` on `m2-reply512`, as it must for equal bytes.

The first build (`conjunction-probe-v36`) failed: the kept forms held one form twice. In that build a slot's state word was read as volatile and its key was not, which leaves the compiler free to read a key before the wait for its writer ends. The key is now read by volatile loads after the slot is seen ready, and the wrapper counts repeated forms before it ranks them. The failure did not recur in the three GPU runs above. The cause was not isolated further than that one change, so it is a likely cause and not a shown one.

Later on 21 Nov (UTC), with `conjunction-probe-v39` [claude]: reader B's stream was copied to wick and its file (`078dcfdc…810c2b`) was written byte for byte too (`runs/conj/gputrain5/`, then again in `gpuscore1/`). Under the sheet [`m2-reply512-gpu-reader-2026-11-21.md`](../../../harness/prep/m2-reply512-gpu-reader-2026-11-21.md) both GPU-trained readers were read on the GPU on `m2-reply512`: 512 of 512 replies equal the reference for each, 0 of 512 against the next prompt's reference. That is a development count on a panel in TRAIN's wording, expected from the equalities above; the decision card records it.

## 4. Update rule

Counting, then written verdicts [claude]. Pass 1 counts forms; pass 2 writes each TRAIN reply token under every held form's rows, after reading what the reader would answer there; pass 3 writes the true reply again under empty-read keys where the reader's own reply met an empty read. Every write lands in the integer rows the server reads. There is no float master and no export step. The served file is those rows less the unreachable ones (see "What is not yet true").

## 5. Lossless format

`UORRDR04`: header integers, forms in key order, held rows in key order, suffix rows, pair rows, then the SHA-256 of everything before it, all little-endian integers [claude]. `reader-save` refuses to keep a file that does not read back to its own bytes; `from_bytes` refuses a changed or cut file. One reader has one byte string: reader A built on wick and on the laptop is the same 35,574,650 bytes.

## 6. A form key read from the trunk state (21 Nov)

Negatives first [claude]. On the corpus-worded panels the trunk-keyed reader answers fewer replies exactly than the token-keyed one: p14 445 -> 430 of 1,108, p23 423 -> 418 of 1,110 (host CPU, wick, folder `trunkkey1`, binary v44). Only the form key moved to the trunk: the reply-so-far history, tails, pointer words, pair rows and token slots are still token ids. The CUDA kernels refuse a trunk-keyed reader. No replay oracle was run for it.

What exists: `conjunction/trunk_key.rs` holds the three trunk tables the position transport needs (a token's root in each block, the step frame of each root, the composition of two frames) and gives the frame the trunk holds after the kept tokens of an asked turn. A reader trained with one keys each question form by that frame and the kept count. Its file is kind `UORRDR05`, which carries the SHA-256 of the trunk tables; `from_bytes` refuses it without a trunk, and `from_bytes_beside` refuses a trunk whose tables hash otherwise. `discrete_d11` reads the tables from the parent artifact it serves (`question_trunk_key`), so a carried trunk-keyed reader is read only behind its own trunk.

Measured on `seed1.d11` (4,096 tokens, 4,008 distinct trunk addresses, 16 distinct step frames): on the M2 stream 1,994 forms become 1,959 and 0 of 512 `m2-reply512` replies differ (512 of 512 both arms, and 512 of 512 from the D11 artifact that carries the trunk-keyed reader). On the corpus stream 214,675 forms become 210,492: distinct kept-token sequences that share a frame and a count share a key. The cause of the lost corpus replies was not separated from the 88 tokens that share a trunk address.

## Next

Done since the first version of this file: the portable build, the WASM run, the D11 carrier read by `discrete_d11`, a CUDA read kernel equal to the host on readers A and B, and CUDA TRAIN kernels that write reader A's file byte for byte. Also done: the sheeted, graded GPU read of the GPU-trained readers on the development panel. Still open: an update rule that does not depend on the order of replies, since the exact schedule is 27 times slower than the host and cannot be widened; suffix orders and token codes in TRAIN; the suffix ladder in the read kernel, which the corpus reader needs on about one prompt in six; a whole-state save so TRAIN can continue from the file; the reader keyed by the trunk's transported state, so the two are one state and not two tables in one file; a CI job that builds `reader-serving`.
