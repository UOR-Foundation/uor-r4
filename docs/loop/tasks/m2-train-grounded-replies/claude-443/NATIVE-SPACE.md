# Native-space items: the saved question reader, claude-443 (D85)

The five items `.agents/skills/native-space-cycle/SKILL.md` asks for, written for the reader that read `m2-fresh256` on 21 Nov [claude]. This file says what is built and measured, and what D85 still asks that is not built [claude].

## What is not yet true

- **One backend only.** The reader's operators have one source definition and one implementation: host Rust. There is no CUDA kernel and no WASM or `discrete_d11` implementation, so there is no differential between device implementations. The replay below compares two machines running the same Rust source (aarch64 macOS and x86_64 Linux), which shows the source is deterministic across targets and nothing more.
- **Not in the trunk state.** The rows are a file of their own (`UORRDR04`). They are not a table family of `TrunkTables`, and the D11 artifact does not carry them. The transported trunk state (S1 of [claude-409](../../written-state-count-model/claude-409/NATIVE-SPACE.md)) takes no part in this read: the key is the question's tokens, not a transported code.
- **TRAIN is a host pass.** The rows are counted by `Reader::train` on the CPU. §3.5 makes GPU the default for training; a 20,000-case count that runs in seconds was run on the laptop and on wick's host. A GPU writer for these rows does not exist.
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

CUDA, WASM and `discrete_d11` columns do not exist (see above).

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

## 4. Update rule

Counting, then written verdicts [claude]. Pass 1 counts forms; pass 2 writes each TRAIN reply token under every held form's rows, after reading what the reader would answer there; pass 3 writes the true reply again under empty-read keys where the reader's own reply met an empty read. Every write lands in the integer rows the server reads. There is no float master and no export step. The served file is those rows less the unreachable ones (see "What is not yet true").

## 5. Lossless format

`UORRDR04`: header integers, forms in key order, held rows in key order, suffix rows, pair rows, then the SHA-256 of everything before it, all little-endian integers [claude]. `reader-save` refuses to keep a file that does not read back to its own bytes; `from_bytes` refuses a changed or cut file. One reader has one byte string: reader A built on wick and on the laptop is the same 35,574,650 bytes.

## Next

In order of what D85 asks: a whole-state save so TRAIN can continue from the file; the rows as a table family the D11 artifact carries, read by `discrete_d11`; a second implementation (WASM or CUDA) held to this replay's word streams.
