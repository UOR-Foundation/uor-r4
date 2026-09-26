# chat-v0 R1 dataset acquisition receipts — 2026-09-25

**Status:** data acquisition complete. Raw data is on the external SSD and is **not** committed.
This document and the companion manifest are the only repository artefacts; the lead commits them
(R1 is the follow-on pipeline task; this task does not tokenize or convert anything).

**Task:** acquire a bounded (150–400 MB) deterministic, resumable raw-text/JSONL subset of
clearly permissive dialogue/instruction data for the chat-v0 R1 response-masked dialogue model
(`car-lm-direction-update-2026-09-25.md` §5–§7). Data only; no cargo, no compilation, no builds,
Python standard library only.

- **Worktree:** `/Users/casey.allard/uor-r4/.worktrees/canonical-address-routing-20260925`
- **Branch / base commit:** `codex/canonical-address-routing-20260925` @ `b7162e607272af1b9c950eb531f77d26b2bf06b3`
- **Raw data root (SSD):** `/Volumes/UOR-Workspace/uor-r4-lab/chat-v0-20260925/raw/`
- **Acquisition UTC:** 2026-09-26T02:47Z–03:19Z (task date label 2026-09-25)
- **Companion evidence:** [`chat-v0-dataset-manifest-2026-09-25.json`](../evidence/chat-v0-dataset-manifest-2026-09-25.json)

## 1. Licence verification (exact field found)

Licence was read from the HF Hub API `cardData.license` and, where absent, from the dataset card
README. Only datasets with a clearly permissive verdict were fetched.

| Dataset | Exact URL | Licence field found | Verdict |
|---|---|---|---|
| `HuggingFaceTB/smoltalk` | https://huggingface.co/datasets/HuggingFaceTB/smoltalk | `cardData.license` **absent**. Card README §License: the four *new* subsets (Smol-Magpie-Ultra, Smol-constraints, Smol-rewrite, Smol-summarize) are **Apache-2.0**; the aggregate `all` config mixes public datasets under their own licences | permissive **only** for the four Apache-2.0 configs (fetched); `all` withheld |
| `HuggingFaceTB/smoltalk2` | https://huggingface.co/datasets/HuggingFaceTB/smoltalk2 | `cardData.license` **absent**. Card README §License: named new splits (e.g. `aya_dataset-Qwen3-32B`, `multi-turn-reasoning-if`, `smolagents-toolcalling-traces`, `smoltalk-multilingual8-Qwen3-32B`, `smoltalk-systemchats-Qwen3-32B`, `table-gpt-Qwen3-32B`, `tulu_3_8b_pref_mix_Qwen3-32B-Qwen3-0.6B-think`) are **Apache-2.0**; existing public datasets keep their own licences | verified; **not fetched** (budget already met by uniformly permissive sources) |
| `roneneldan/TinyStoriesInstruct` | https://huggingface.co/datasets/roneneldan/TinyStoriesInstruct | No `cardData`; `readme.md` body: **`License: CDLA-Sharing-1.0`** | **withheld** — data share-alike, not on the clearly-permissive allow-list |
| `databricks/databricks-dolly-15k` | https://huggingface.co/datasets/databricks/databricks-dolly-15k | `cardData.license` = **`cc-by-sa-3.0`** (+ tag `license:cc-by-sa-3.0`) | **withheld** — CC-BY-SA share-alike |
| `HuggingFaceH4/ultrachat_200k` | https://huggingface.co/datasets/HuggingFaceH4/ultrachat_200k | `cardData.license` = **`mit`** (+ tag `license:mit`) | permissive — fetched |
| `HuggingFaceTB/everyday-conversations-llama3.1-2k` (added) | https://huggingface.co/datasets/HuggingFaceTB/everyday-conversations-llama3.1-2k | `cardData.license` = **`apache-2.0`** (+ tag `license:apache-2.0`) | permissive — fetched (simple everyday dialogue side) |

Note on `smoltalk` ("SmolTalk-2"): neither `HuggingFaceTB/smoltalk` nor `HuggingFaceTB/smoltalk2`
carries a dataset-wide `cardData.license`. Both cards declare only their **new** subsets
Apache-2.0 and defer the rest to the original public datasets. This task therefore fetches only the
four card-declared Apache-2.0 SmolTalk configs; it does **not** fetch either mixed aggregate.

## 2. What was fetched

12 files, **80,291 rows**, **277,173,186 bytes (277.2 MB)** raw JSONL. Every output file's sha256 was
recomputed from disk and matches `manifest.json`.

| label | dataset | config | split | role | licence | rows | bytes | sha256 |
|---|---|---|---|---|---:|---:|---|
| `smoltalk_smol-magpie-ultra.train` | `HuggingFaceTB/smoltalk` | `smol-magpie-ultra` | `train` | train | Apache-2.0 | 13,900 | 100,627,145 | `61f64fae96470193d0d41cf693fd18523c2976d4c5116fe2f9b0f692e0df7d05` |
| `smoltalk_smol-constraints.train` | `HuggingFaceTB/smoltalk` | `smol-constraints` | `train` | train | Apache-2.0 | 17,900 | 20,069,314 | `42bf43c76eb28993db514544cbfe4054186cd69f9a3894373204da9733ce8d10` |
| `smoltalk_smol-rewrite.train` | `HuggingFaceTB/smoltalk` | `smol-rewrite` | `train` | train | Apache-2.0 | 17,200 | 30,055,950 | `f4c22e56a5db732f1e0b1084fbd62dbe752dd7e316f7baf70e6af9e275a4a62b` |
| `smoltalk_smol-summarize.train` | `HuggingFaceTB/smoltalk` | `smol-summarize` | `train` | train | Apache-2.0 | 12,300 | 30,099,230 | `a65a88621e34bf369b2d822e5d093caa96db51cd7069c5a568dd5e0e3be1ad4b` |
| `ultrachat_200k.default.train_sft` | `HuggingFaceH4/ultrachat_200k` | `default` | `train_sft` | train | MIT | 10,200 | 70,639,403 | `cbe48620627a740a97e04849b97d9c47ed387b92e3be059752f8fde48a5a61ce` |
| `everyday-conversations-llama3.1-2k.default.train_sft` | `HuggingFaceTB/everyday-conversations-llama3.1-2k` | `default` | `train_sft` | train | Apache-2.0 | 2,260 | 6,034,143 | `ae41a99645f94471eb32b4a717ff984f808ddfb2e7624431d7797dc8413312ed` |
| `smoltalk_smol-magpie-ultra.test` | `HuggingFaceTB/smoltalk` | `smol-magpie-ultra` | `test` | heldout | Apache-2.0 | 700 | 5,001,797 | `69b37f3eddd1b2561e8638f0d52a0d1dbfda2d6584a17f7f17c78060b925458b` |
| `smoltalk_smol-constraints.test` | `HuggingFaceTB/smoltalk` | `smol-constraints` | `test` | heldout | Apache-2.0 | 1,812 | 2,018,618 | `825c78187a357642a3260e2027de6407d98c24e7616c85690b3cb89c92ce8463` |
| `smoltalk_smol-rewrite.test` | `HuggingFaceTB/smoltalk` | `smol-rewrite` | `test` | heldout | Apache-2.0 | 1,700 | 3,027,261 | `a7e256d2bc7592fed5bacb15174f2ee4d381432a64abfb2d4449cf079e7da327` |
| `smoltalk_smol-summarize.test` | `HuggingFaceTB/smoltalk` | `smol-summarize` | `test` | heldout | Apache-2.0 | 1,300 | 3,078,157 | `501de16bdf70064ce1b87e4ac8def1598c9c7443e5a7f6f69bbe0adc416e647e` |
| `ultrachat_200k.default.test_sft` | `HuggingFaceH4/ultrachat_200k` | `default` | `test_sft` | heldout | MIT | 900 | 6,205,514 | `3f50cae6badb837a3a7d92f408e563fa29af9beff2607f7bb9ab2ea6596f2b41` |
| `everyday-conversations-llama3.1-2k.default.test_sft` | `HuggingFaceTB/everyday-conversations-llama3.1-2k` | `default` | `test_sft` | heldout | Apache-2.0 | 119 | 316,654 | `5b263209232ed356e18670ba5b61c88fe23881fdf46f0492b225f297c8785e6c` |

- Capacity split: **train 257,525,185 B (92.9%)**, **held-out 19,648,001 B (7.1%)**.
- Source composition: SmolTalk (Apache-2.0) 180.9 MB over 4 configs; UltraChat-200k (MIT) 76.8 MB;
  everyday-conversations (Apache-2.0) 6.4 MB.
- `heldout` files are the **official upstream test splits**. They must never be used for training.
  The `.test` files are bounded **prefixes** of the official split except
  `smoltalk_smol-constraints.test` (all 1,812) and `everyday-conversations…test_sft` (all 119).
- Dataset revisions observed and recorded: `HuggingFaceTB/smoltalk` `5feaf2fd3ffca7c237fc38d1861bc30365d48ffa`,
  `HuggingFaceH4/ultrachat_200k` `8049631c405ae6576f93f445c6b8166f76f5505a`,
  `HuggingFaceTB/everyday-conversations-llama3.1-2k` `14f543216b9ba42b6b951dc5bd199460d193b162`.

## 3. Story / dialogue side

The task asked for a balance between instruction/chat and simple story dialogue. The simple-story
half is deliberately **not** re-fetched: the owner checkout already holds
`/Users/casey.allard/uor-r4/.uor-models/research/issue-1014/raw/TinyStoriesV2-GPT4-train.txt`
(2,227,753,162 B, sha256 `6418d412de72888f52b5142c761ac21a582f7d1166f0bfbdb5f03ccfdec90443`, mtime
2026-08-30), plus `tinystories_train.u16` (1,110,771,074 B). Per the task these are left in place
and not copied. They are the natural story/R1 narrative source.

`everyday-conversations-llama3.1-2k` (Apache-2.0) adds a small permissively licensed simple,
multi-turn everyday-dialogue component (6.4 MB).

## 4. Withheld (content NOT fetched — awaiting owner decision)

| Dataset | Licence | Why withheld |
|---|---|---|
| `databricks/databricks-dolly-15k` | `cc-by-sa-3.0` | Share-alike. Whether share-alike obligations attach to a trained model artifact is unresolved and risky. Owner decision required. |
| `roneneldan/TinyStoriesInstruct` | `CDLA-Sharing-1.0` | Community data share-alike agreement; not on the clearly-permissive allow-list. The pre-existing local TinyStories corpus already covers the story side. Owner decision required before its *instruction-formatted* variant is fetched. |
| `HuggingFaceTB/smoltalk` aggregate `all` config | mixed | Includes public datasets under their own (unverified here) licences. Only the four card-declared Apache-2.0 configs were fetched. Owner decision if the mixed aggregate is wanted. |
| `HuggingFaceTB/smoltalk2` | mixed (named splits Apache-2.0) | Verified, not fetched: the bounded budget was already met with uniformly permissive sources. Named Apache-2.0 splits remain available as a second instruction source on owner request. |

**Caveat:** the pre-existing local TinyStories corpus itself derives from `roneneldan/TinyStories`,
which is also CDLA-Sharing-1.0. It is flagged here for the same owner decision; this task neither
fetched it nor modified it.

## 5. Exact reproduction

Requires only Python 3 standard library (verified 3.14.6) and network access to
`datasets-server.huggingface.co` and `huggingface.co`.

```sh
# Full deterministic re-fetch (resumable; skips completed sources):
python3 /Volumes/UOR-Workspace/uor-r4-lab/chat-v0-20260925/raw/fetch_chat_v0.py

# Single source:
python3 /Volumes/UOR-Workspace/uor-r4-lab/chat-v0-20260925/raw/fetch_chat_v0.py \
  --only smoltalk_smol-magpie-ultra.train

# Regenerate manifest.json + README.md from existing state (no network):
python3 /Volumes/UOR-Workspace/uor-r4-lab/chat-v0-20260925/raw/fetch_chat_v0.py --manifest-only

# Clean re-fetch (destructive to the SSD raw dir only):
#   rm -rf <raw>/data <raw>/state
```

Fetcher identity: `fetch_chat_v0.py` sha256
`3d8fd333dcaa4b63350f8970a7510793a6c4b8f174ab020ee6864a91afeba3ca`. It pages the datasets-server
`rows` API in fixed 100-row pages from offset 0 (maximum `length` is 100), requests `truncate=0`,
and writes each line as `{"row_idx": <int>, "row": <original record>}`.

One-page manual probe:

```sh
curl -s 'https://datasets-server.huggingface.co/rows?dataset=HuggingFaceTB%2Fsmoltalk&config=smol-magpie-ultra&split=train&offset=0&length=1&truncate=0' | python3 -m json.tool
```

## 6. Verification actually executed

| Check | Result |
|---|---|
| sha256 recomputed from every output file vs manifest | 12/12 match |
| JSONL parse + envelope (`row_idx`,`row`) present | 80,291/80,291 lines valid |
| `row_idx` contiguous 0..N-1, no gaps/duplicates | 12/12 files contiguous |
| Record shape (messages / prompt / completion present) | 80,291/80,291 |
| `truncated_cells` returned by API | 0 total (no truncation) |
| Resume equivalence (3-page pause+resume vs single pass) | byte-identical: `42bf43c7…` on `smoltalk_smol-constraints.train` |
| Transient API errors | 89× HTTP 429 + 2× HTTP 502, all recovered by backoff (max depth 6/8, max sleep 48 s); 0 exhausted, 0 `WARN`/`ERROR`/traceback |

## 7. Cost

- Wall time: ~32 min total (22 min full fetch run + ~10 min probe/verification; SSD is USB).
- CPU: 10.5 s user + 5.0 s system for the full run (network-bound).
- Network: ~277 MB downloaded (plus small metadata).
- New storage on SSD: 281 MB in `raw/` (raw dir `du` = 281 M at completion).
- Free space after: SSD unaffected materially (200 GB volume); no project ledger charge beyond
  the raw dir. No cargo, no build, no external/paid compute.

## 8. Limitations and blockers

- Reproduction is bounded by the upstream dataset revision: the `rows` API serves the current
  revision and cannot be revision-pinned. Revision shas observed at fetch time are recorded; a
  future upstream change can alter results.
- No tokenizer, model format, or train/held-out split construction was produced — that is R1.
- The balance is instruction/chat-heavy in the *fetched* set (92.9% train) by design, because the
  permissive story source (`TinyStoriesInstruct`) is withheld and the story side is the existing
  local TinyStories corpus. The R1 pipeline should combine the two.
- **Blocker for a broader claim:** owner decision on the withheld share-alike sources
  (`databricks-dolly-15k`, `TinyStoriesInstruct`, and the local TinyStories corpus provenance).
  No other blocker; the acquisition itself is complete.

## 9. Handoff boundary

- Raw data stays on the SSD at `/Volumes/UOR-Workspace/uor-r4-lab/chat-v0-20260925/raw/`; it is not
  committed and no model/token artefacts were produced.
- Only these two files are handed to the lead for commit:
  `docs/evidence/chat-v0-dataset-manifest-2026-09-25.json` and this document.
- R1 consumes `data/*.train.jsonl` for training and `data/*.test.jsonl` for the frozen held-out
  panel, applying the response mask at the JSONL level.
