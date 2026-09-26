# chat-v0 R1a corpus preparation — 2026-09-25

**Status:** data preparation complete. **No model or training code changed; no training run.** Raw
JSONL stayed read-only on the SSD. This document and the new Rust binary are the only repository
artefacts; the lead commits them. The prepared corpus itself is on the SSD and is **not** committed.

**Task:** R1a of the CAR-LM reordered ladder (`car-lm-direction-update-2026-09-25.md` §4–§7):
turn the acquired permissive dialogue JSONL into a token/mask corpus for the response-masked R1
dialogue objective. Consumes the acquisition receipts (`chat-v0-dataset-receipts-2026-09-25.md`,
`../evidence/chat-v0-dataset-manifest-2026-09-25.json`).

- **Worktree:** `/Users/casey.allard/uor-r4/.worktrees/canonical-address-routing-20260925`
- **Branch:** `codex/canonical-address-routing-20260925`
- **Base commit read at task start:** `52947d4b` (`R0: retained arms fail chat-panel v0; gap is
  objective/data (negative, scoped)`). Current branch HEAD is `0b7b4c97` (the shared D8 cycle
  advanced the branch during this task). The change below is uncommitted on top of HEAD.
- **New file:** `crates/uor-r4-core/src/bin/prepare-chat-corpus.rs`
- **Raw data:** `/Volumes/UOR-Workspace/uor-r4-lab/chat-v0-20260925/raw/data/*.jsonl` (read-only)
- **Prepared data:** `/Volumes/UOR-Workspace/uor-r4-lab/chat-v0-20260925/prepared/`
- **Build target dir:** the owner checkout's `.cargo/config.toml` sets
  `build.target-dir = /Users/casey.allard/uor-r4/target`; nested worktrees share it. The binary
  therefore lands at `/Users/casey.allard/uor-r4/target/debug/prepare-chat-corpus`.

## 1. Tokenizer identity (unchanged)

The binary reuses the shared byte-level BPE engine in `crates/uor-r4-tokenizer` unchanged and the
fixed 4096-entry vocabulary. No vocabulary is added, no tokenizer is trained.

| Field | Value |
|---|---|
| tokenizer.json | `/Users/casey.allard/uor-r4/.uor-models/investigations/integer-serving-20260925/bundle-quaternion-1/tokenizer.json` |
| sha256 | `d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89` |
| `tokenizer_cid` (blake3 of raw bytes) | `blake3:3f42bcfce7728512076549c63b88387e13c8156fe35c0f91d9b112439f3739cc` |
| id slots | 4096 |
| BOS / EOS / UNK | 0 / 1 / 2 (`<|bos|>`, `<|eos|>`, `<|unk|>`) |

The same bytes are present at `research/issue-1017/export/tokenizer.json`, `issue-1014/export/…`,
`issue-1019/tokenizer/…` and the language-continuation preflight bundles — all the same sha256.

## 2. Template

Turns are joined by a single `\n` separator placed **before** every turn after the first, so each
role marker begins a line:

```text
<|bos|>System: {s}\nUser: {u}\nAssistant: {a}<|eos|>\nUser: {u2}\nAssistant: {a2}<|eos|>
```

- Role markers are the literal strings `System: `, `User: `, `Assistant: ` (capitalised, with a
  trailing space). They are encoded as ordinary byte-level BPE text; the vocabulary represents them.
- Every assistant turn ends with `<|eos|>`.
- A document-terminal `<|eos|>` is appended only when the final emitted turn is **not** an
  assistant turn (e.g. `everyday-conversations` rows whose last message is a user turn). The last
  token of every document is therefore `<|eos|>`.
- `messages` with an unknown role or empty/whitespace-only content are skipped and counted.
- `\r\n`/`\r` are normalised to `\n`; only leading/trailing whitespace of each content is trimmed.
  Interior whitespace is preserved.

## 3. Response mask

`response_mask.u8` has exactly one byte per token; length equals the token count (checked).

- **`1`** — every token of an assistant turn's response content, **and its terminating `<|eos|>`**.
- **`0`** — `<|bos|>`, every role marker (`System: `, `User: `, `Assistant: `), turn separators,
  system/user content, and an unmasked document-terminal `<|eos|>`.

The `Assistant: ` marker is prompt context, not response content, so it is `0`; the first masked
token is the first response token, whose loss is conditioned on the marker. This is the standard
"train on completions" masking.

## 4. Outputs and hashes

`tokens.u16` is the canonical `UORT` container (`mmap_corpus.rs`): 64-byte header
(`UORT`, version 1, u64 token count, u32 vocab size) followed by little-endian `u16` tokens.
The existing trainer loads this directly. `response_mask.u8` is a raw byte per token.

### train

| File | Bytes | sha256 |
|---|---:|---|
| `prepared/train/tokens.u16` | 165,059,444 | `4a554b0ef8be12344f21f1bd6bdc9faeeeddcef212a138a8772a8bf42604c4fa` |
| `prepared/train/response_mask.u8` | 82,529,690 | `09dc0fe5d0e2de7b5072e4586b5cb9ef3379856600ff90061c561a48826e75e7` |
| `prepared/train/manifest.json` | 7,147 | `01d7758a7b96223b6c9eecb7d6a0e4523922d19f2b6c03ae2aecf8cc702de9f8` |

- rows used **73,632 / 73,760**; tokens **82,529,690**; response tokens **56,650,286 (68.64 %)**.

### heldout

| File | Bytes | sha256 |
|---|---:|---|
| `prepared/heldout/tokens.u16` | 12,430,864 | `ca5ccf0722d9dcdc5294b612f2c25536fe49192be7bedbc3007a1fcfc0940225` |
| `prepared/heldout/response_mask.u8` | 6,215,400 | `ed43a9eade79d84dc73be608fb788f1e7566021fbdbba35ce3e5eddbc4fce8b4` |
| `prepared/heldout/manifest.json` | 7,079 | `a3633f3013838f5e96423e0e396c9cfd5e2e3a289709b62ca190694b1cc7c024` |

- rows used **6,526 / 6,531**; tokens **6,215,400**; response tokens **3,942,347 (63.43 %)**.
- Held-out files are the official upstream test splits and were written as a **separate split**;
  they are never mixed into train.

Combined `prepared/manifest.json` sha256 `a66d52473cac24b28cc09a751ede7b680c40e7b56218e9074e246cf4fd1a99a5`.

## 5. Per-file counts and input hashes

### train (input order preserved)

| label | rows used / total | tokens | response | dropped (oversized) | input sha256 |
|---|---:|---:|---:|---:|---|
| `smoltalk_smol-magpie-ultra.train` | 13,780 / 13,900 | 35,030,458 | 29,920,386 | 120 | `61f64fae96470193d0d41cf693fd18523c2976d4c5116fe2f9b0f692e0df7d05` |
| `smoltalk_smol-constraints.train` | 17,900 / 17,900 | 6,561,743 | 4,133,184 | 0 | `42bf43c76eb28993db514544cbfe4054186cd69f9a3894373204da9733ce8d10` |
| `smoltalk_smol-rewrite.train` | 17,200 / 17,200 | 9,832,879 | 4,345,493 | 0 | `f4c22e56a5db732f1e0b1084fbd62dbe752dd7e316f7baf70e6af9e275a4a62b` |
| `smoltalk_smol-summarize.train` | 12,300 / 12,300 | 10,041,316 | 2,021,458 | 0 | `a65a88621e34bf369b2d822e5d093caa96db51cd7069c5a568dd5e0e3be1ad4b` |
| `ultrachat_200k.default.train_sft` | 10,192 / 10,200 | 20,456,594 | 15,870,954 | 8 | `cbe48620627a740a97e04849b97d9c47ed387b92e3be059752f8fde48a5a61ce` |
| `everyday-conversations-llama3.1-2k.default.train_sft` | 2,260 / 2,260 | 606,700 | 358,811 | 0 | `ae41a99645f94471eb32b4a717ff984f808ddfb2e7624431d7797dc8413312ed` |

Every input sha256 matches the acquisition manifest exactly (12/12 unchanged).

### heldout

| label | rows used / total | tokens | response | dropped (oversized) | input sha256 |
|---|---:|---:|---:|---:|---|
| `smoltalk_smol-magpie-ultra.test` | 695 / 700 | 1,724,744 | 1,490,840 | 5 | `69b37f3eddd1b2561e8638f0d52a0d1dbfda2d6584a17f7f17c78060b925458b` |
| `smoltalk_smol-constraints.test` | 1,812 / 1,812 | 662,354 | 418,068 | 0 | `825c78187a357642a3260e2027de6407d98c24e7616c85690b3cb89c92ce8463` |
| `smoltalk_smol-rewrite.test` | 1,700 / 1,700 | 994,322 | 438,028 | 0 | `a7e256d2bc7592fed5bacb15174f2ee4d381432a64abfb2d4449cf079e7da327` |
| `smoltalk_smol-summarize.test` | 1,300 / 1,300 | 1,020,064 | 207,269 | 0 | `501de16bdf70064ce1b87e4ac8def1598c9c7443e5a7f6f69bbe0adc416e647e` |
| `ultrachat_200k.default.test_sft` | 900 / 900 | 1,782,377 | 1,369,364 | 0 | `3f50cae6badb837a3a7d92f408e563fa29af9beff2607f7bb9ab2ea6596f2b41` |
| `everyday-conversations-llama3.1-2k.default.test_sft` | 119 / 119 | 31,539 | 18,778 | 0 | `5b263209232ed356e18670ba5b61c88fe23881fdf46f0492b225f297c8785e6c` |

## 6. Dropped / skipped accounting

| Counter | train | heldout |
|---|---:|---:|
| rows dropped: no `messages` | 0 | 0 |
| rows dropped: all content empty | 0 | 0 |
| rows dropped: no assistant response | 0 | 0 |
| rows dropped: oversized (`> 8192` tokens, default) | **128** | **5** |
| rows dropped: malformed JSON | 0 | 0 |
| messages skipped: empty content | **1** | 0 |
| messages skipped: unknown role | 0 | 0 |
| special-token strings inside content | 0 | 0 |

**Dropped-row reasons.** All 133 dropped rows are oversized (`> 8192` rendered tokens): 120
`smoltalk_smol-magpie-ultra.train`, 8 `ultrachat_200k.default.train_sft`, 5
`smoltalk_smol-magpie-ultra.test`. They are long multi-turn/reasoning rows far beyond the R1
256-token context; they are dropped, not truncated. One `ultrachat` assistant message had
empty/whitespace-only content and was skipped. No malformed JSON, no missing `messages`, no rows
without an assistant, and no unknown roles.

## 7. Schema surprises / notes

- Two schema shapes were handled: `smoltalk`/`ultrachat`/`everyday` all carry
  `row.messages = [{content, role}]`. `smoltalk_smol-rewrite`/`-summarize` begin with a `system`
  message (rendered as `System: `, mask `0`); the others begin with `user`.
- `everyday-conversations` contains 255 train / 13 test rows whose final message is a user turn
  (truncated dialogue); they still contain assistant turns, so they are kept and receive an
  unmasked document-terminal EOS.
- `ultrachat` repeats the first user turn in both `prompt` and `messages[0]`; only `messages` is
  used.
- The only `<|` substring anywhere in content is `(<|>)` inside a Python regex in
  `smoltalk_smol-magpie-ultra.train` row 6551; it is not an added token, and the
  `special_token_occurrences` counter is 0 across all 80,291 input rows.

## 8. Exact commands

Build (debug; the shared target dir is the owner checkout's):

```sh
cd /Users/casey.allard/uor-r4/.worktrees/canonical-address-routing-20260925
/Users/casey.allard/.cargo/bin/cargo check -p uor-r4-core --bin prepare-chat-corpus --offline
/Users/casey.allard/.cargo/bin/cargo test  -p uor-r4-core --bin prepare-chat-corpus --offline
/Users/casey.allard/.cargo/bin/cargo build -p uor-r4-core --bin prepare-chat-corpus --offline
```

Prepare (the exact invocation that produced the hashes above; `--out-dir` is recreated):

```sh
BIN=/Users/casey.allard/uor-r4/target/debug/prepare-chat-corpus
TOK=/Users/casey.allard/uor-r4/.uor-models/investigations/integer-serving-20260925/bundle-quaternion-1/tokenizer.json
RAW=/Volumes/UOR-Workspace/uor-r4-lab/chat-v0-20260925/raw/data
OUT=/Volumes/UOR-Workspace/uor-r4-lab/chat-v0-20260925/prepared
rm -rf "$OUT"
"$BIN" prepare --tokenizer "$TOK" --out-dir "$OUT" \
  --train   "$RAW/smoltalk_smol-magpie-ultra.train.jsonl" \
  --train   "$RAW/smoltalk_smol-constraints.train.jsonl" \
  --train   "$RAW/smoltalk_smol-rewrite.train.jsonl" \
  --train   "$RAW/smoltalk_smol-summarize.train.jsonl" \
  --train   "$RAW/ultrachat_200k.default.train_sft.jsonl" \
  --train   "$RAW/everyday-conversations-llama3.1-2k.default.train_sft.jsonl" \
  --heldout "$RAW/smoltalk_smol-magpie-ultra.test.jsonl" \
  --heldout "$RAW/smoltalk_smol-constraints.test.jsonl" \
  --heldout "$RAW/smoltalk_smol-rewrite.test.jsonl" \
  --heldout "$RAW/smoltalk_smol-summarize.test.jsonl" \
  --heldout "$RAW/ultrachat_200k.default.test_sft.jsonl" \
  --heldout "$RAW/everyday-conversations-llama3.1-2k.default.test_sft.jsonl"
```

Structural self-check on the loaded artefacts (recomputes mask length, BOS/EOS endpoints,
document boundaries, and that every masked run is preceded by the exact `Assistant: ` marker
tokens and terminated by EOS):

```sh
"$BIN" verify --tokenizer "$TOK" --dir "$OUT/train"   --preview 64
"$BIN" verify --tokenizer "$TOK" --dir "$OUT/heldout" --preview 64
```

## 9. Verification actually executed

| Check | Result |
|---|---|
| Focused unit tests (`cargo test --bin prepare-chat-corpus`) | **9/9 pass** |
| Synthetic single-turn exact mask + decode round-trip | pass (exact expected mask) |
| Multi-turn boundaries exact mask + decode | pass |
| Trailing-user terminal EOS unmasked | pass |
| System turn mask 0 | pass |
| Empty content skipped / row has no response | pass |
| Unknown role skipped and counted | pass |
| Empty/oversized/malformed drop counts + split files written | pass |
| Two runs byte-identical (unit, incl. `manifest.json`) | pass |
| Full-scale re-run into a scratch dir, all 7 artefacts | **byte-identical** |
| `verify` structural audit train | 82,529,690 tokens, 73,632 docs, 129,486 masked runs, **0 violations** |
| `verify` structural audit heldout | 6,215,400 tokens, 6,526 docs, 10,251 masked runs, **0 violations** |
| Independent `shasum -a 256` of all four stream files vs manifest | 4/4 match |
| `UORT` header parse: version 1, vocab 4096, payload = 64 + 2·tokens, mask len = tokens | train + heldout ok |
| Input file sha256 vs acquisition manifest | 12/12 match |

The mtime on `target/debug/prepare-chat-corpus` and the docs/tests above are engineering evidence
about the **data pipeline only**. This is not a model result and says nothing about language
quality.

## 10. Cost

| Item | Value |
|---|---|
| Full prepare (12 files, 277,173,186 B raw) | **real 226.83 s**, user 144.91 s, sys 4.69 s |
| Focused check + test + build | ~7 min wall including the first full debug build (~4.5 min), concurrent with the D8 cycle |
| Peak RAM | **NOT MEASURED.** Single-threaded line streaming with no full-corpus buffering; bounded by the largest JSONL line plus the shared tokenizer's 65,536-entry per-pre-token cache. |
| New storage on SSD | train 247,589,134 B + heldout 18,646,264 B + manifests ≈ **266 MB** |
| SSD free before / after | 208,168,416 KiB → 208,168,416 KiB (1 % of a 200 GB volume; no pressure) |
| Repo artefacts | one new Rust source file + this document (uncommitted; the lead commits) |
| External/paid compute | none. No fetch, no network, no GPU. |

Deliberately no `--threads` parallelism: the run is short, and single-threaded execution keeps it
deterministic and does not contend with the concurrent D8 cycle's cargo/CPU.

## 11. Limitations and handoff boundary

- **Data only.** No model config, objective, smoke fit or evaluation is implemented here; that is
  R1b. The mask is emitted but nothing consumes it yet.
- The `> 8192`-token rows are dropped rather than windowed; 133 rows (~0.17 %) are lost. If R1b
  wants them, the threshold is a CLI flag (`--max-tokens`) and a re-run is deterministic.
- The corpus concatenates documents with no windowing/packing; R1b owns sequence construction
  (context window, BOS/EOS reset or sliding) and must consume `response_mask.u8` in lock-step with
  `tokens.u16`.
- The template here (`User: `/`Assistant: ` lines) is intentionally the R1a corpus template chosen
  by the task. The frozen chat-panel-v0 instrument uses a different presentation
  (`<|user|>{turn}\n<|assistant|>`); R1b must reconcile the training template with the panel
  presentation before claiming panel behaviour.
- Raw JSONL remains on the SSD and uncommitted; the prepared `UORT`/mask files are also SSD-local.

**Next (R1b):** model config + response-masked objective + smoke fit. Read
`prepared/train/tokens.u16` with `MmapCorpusReader`, read `response_mask.u8` in lock-step, shift the
mask by one position to supervise next-token prediction on assistant tokens only, and report
response-token NLL against the retained count reference on the separate `prepared/heldout` split.
