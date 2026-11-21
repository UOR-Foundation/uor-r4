# Native-space items: the saved question reader, claude-443 (D85)

The five items `.agents/skills/native-space-cycle/SKILL.md` asks for, written for the reader that read `m2-fresh256` on 21 Nov [claude]. This file says what is built and measured, and what D85 still asks that is not built [claude].

## What is not yet true

- **A second implementation of the read only.** The Rust source compiles for three targets (aarch64 macOS, x86_64 Linux, `wasm32-wasip1`) with equal word streams, which shows it is deterministic across targets. A CUDA kernel now reads the held rows as a second, separately written implementation and equals the host on readers A and B (section 3). It does not read the suffix ladder, it covers one set of read choices, and there is no second implementation of TRAIN.
- **Carried by D11, not part of the trunk state.** A D11 artifact now carries the reader (schema `uor-r4.lut-discrete-question-reader/1`) and `discrete_d11` answers from it. It is carried as the reader's own file, byte for byte, under a header that binds it and the parent artifact by SHA-256. It is not one of the positional table sections of `TrunkTables`, and the transported trunk state (S1 of [claude-409](../../written-state-count-model/claude-409/NATIVE-SPACE.md)) takes no part in this read: the key is the question's tokens, not a transported code. The trunk and the reader share an artifact and a vocabulary, and nothing else yet.
- **The carrier is not built in CI.** It is behind the integer crate's `reader-serving` feature, which no CI job selects. Its test and the builds below ran on the laptop. The integer crate with the feature was only type-checked for `wasm32-unknown-unknown`; the browser client does not call it.
- **TRAIN is a host pass.** The rows are counted by `Reader::train` on the CPU. §3.5 makes GPU the default for training; a 20,000-case count that runs in seconds was run on the laptop and on wick's host. A GPU writer for these rows does not exist; the owner's answer of 21 Nov ("No, GPU first") makes it the next unit.
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

The serving caller is the same in every build: the trainer crate (`uor_r4_discrete::conjunction`), the portable crate `uor-r4-reader-runtime` (the same source files, compiled without the GPU pipeline, host and `wasm32-wasip1`) and `IntegerDiscreteModel::question_reply` in `discrete_d11`, which calls the portable crate. A CUDA column does not exist.

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

## 4. Update rule

Counting, then written verdicts [claude]. Pass 1 counts forms; pass 2 writes each TRAIN reply token under every held form's rows, after reading what the reader would answer there; pass 3 writes the true reply again under empty-read keys where the reader's own reply met an empty read. Every write lands in the integer rows the server reads. There is no float master and no export step. The served file is those rows less the unreachable ones (see "What is not yet true").

## 5. Lossless format

`UORRDR04`: header integers, forms in key order, held rows in key order, suffix rows, pair rows, then the SHA-256 of everything before it, all little-endian integers [claude]. `reader-save` refuses to keep a file that does not read back to its own bytes; `from_bytes` refuses a changed or cut file. One reader has one byte string: reader A built on wick and on the laptop is the same 35,574,650 bytes.

## Next

Done since the first version of this file: the portable build, the WASM run, the D11 carrier read by `discrete_d11`, and a CUDA read kernel equal to the host on readers A and B. Still open: a CUDA writer (TRAIN on the GPU) whose reader file is the host's byte for byte, which the owner's "GPU first" asks for before any M2 count; the suffix ladder in the read kernel, which the corpus reader needs on about one prompt in six; a whole-state save so TRAIN can continue from the file; the reader keyed by the trunk's transported state, so the two are one state and not two tables in one file; a CI job that builds `reader-serving`.
