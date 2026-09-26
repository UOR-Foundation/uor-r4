# chat-v0 R1 dataset acquisition receipts — 2026-09-25

**Status:** data acquisition complete. Raw data is on the external SSD and is **not** committed.
This document and the companion manifest are the only repository artefacts; the lead commits them
(R1 is the follow-on pipeline task; this task does not tokenize or convert anything).

**Task:** acquire a bounded deterministic, resumable raw-text/JSONL subset of dialogue/instruction
data for the chat-v0 R1 response-masked dialogue model
(`car-lm-direction-update-2026-09-25.md` §5–§7). Data only; no cargo, no compilation, no builds,
Python standard library only.

**Extension (2026-09-25 owner direction):** the owner approved the two share-alike sources for this
local research workstream. `databricks/databricks-dolly-15k` (CC-BY-SA-3.0) and
`roneneldan/TinyStoriesInstruct` (CDLA-Sharing-1.0) were added with their official held-out split
where one exists. This is an **UPDATE** to the prior 12 permissive sources, which are byte-identical
with unchanged sha256. Licence obligations are recorded in §5.

- **Worktree:** `/Users/casey.allard/uor-r4/.worktrees/canonical-address-routing-20260925`
- **Branch / base commit:** `codex/canonical-address-routing-20260925` @ `b7162e607272af1b9c950eb531f77d26b2bf06b3`
- **Raw data root (SSD):** `/Volumes/UOR-Workspace/uor-r4-lab/chat-v0-20260925/raw/`
- **Acquisition UTC:** 2026-09-26T02:47Z–03:19Z (12 permissive sources); extension 2026-09-26T04:17Z–04:36Z (share-alike fetch) plus a 04:42Z–04:44Z resume-equivalence replay; receipts updated 2026-09-26T04:45Z at worktree HEAD `0b7b4c970e43176f909657247ebde5749a3558d0`.
- **Fetcher identity:** `fetch_chat_v0.py` sha256 `497cf1a19a4ec0854ffecd6f891151e27e083c69b1ca12cf6e88c212da5575f7` (extension fetch ran `6643d223…`; the final `497cf1a1…` differs only by a generated-README heading text correction). The earlier 12-source generator is preserved as `fetch_chat_v0_v1_2026-09-25.py` (sha256 `3d8fd333dcaa4b63350f8970a7510793a6c4b8f174ab020ee6864a91afeba3ca`).
- **Companion evidence:** [`chat-v0-dataset-manifest-2026-09-25.json`](../evidence/chat-v0-dataset-manifest-2026-09-25.json)

## 1. Licence verification (exact field found)

Licence was read from the HF Hub API `cardData.license` and, where absent, from the dataset card
README. Permissive datasets were fetched directly; the two share-alike datasets were fetched only
after explicit owner approval on 2026-09-25, with obligations recorded in §5.

| Dataset | Exact URL | Licence field found | Verdict |
|---|---|---|---|
| `HuggingFaceTB/smoltalk` | https://huggingface.co/datasets/HuggingFaceTB/smoltalk | `cardData.license` **absent**. Card README §License: the four *new* subsets (Smol-Magpie-Ultra, Smol-constraints, Smol-rewrite, Smol-summarize) are **Apache-2.0**; the aggregate `all` config mixes public datasets under their own licences | permissive **only** for the four Apache-2.0 configs (fetched); `all` withheld |
| `HuggingFaceTB/smoltalk2` | https://huggingface.co/datasets/HuggingFaceTB/smoltalk2 | `cardData.license` **absent**. Card README §License: named new splits (e.g. `aya_dataset-Qwen3-32B`, `multi-turn-reasoning-if`, `smolagents-toolcalling-traces`, `smoltalk-multilingual8-Qwen3-32B`, `smoltalk-systemchats-Qwen3-32B`, `table-gpt-Qwen3-32B`, `tulu_3_8b_pref_mix_Qwen3-32B-Qwen3-0.6B-think`) are **Apache-2.0**; existing public datasets keep their own licences | verified; **not fetched** (budget already met by uniformly permissive sources) |
| `roneneldan/TinyStoriesInstruct` | https://huggingface.co/datasets/roneneldan/TinyStoriesInstruct | No `cardData`; repo `readme.md` body (file is lowercase): **`License: CDLA-Sharing-1.0`** | **owner-approved 2026-09-25 → fetched** (train + official `validation` held-out); revision `ee050ed1f8720795be342921335e821856a2b42e` |
| `databricks/databricks-dolly-15k` | https://huggingface.co/datasets/databricks/databricks-dolly-15k | `cardData.license` = **`cc-by-sa-3.0`** (+ tag `license:cc-by-sa-3.0`) | **owner-approved 2026-09-25 → fetched** (full `train`, no official held-out split exists); revision `bdd27f4d94b9c1f951818a7da7fd7aeea5dbff1a` |
| `HuggingFaceH4/ultrachat_200k` | https://huggingface.co/datasets/HuggingFaceH4/ultrachat_200k | `cardData.license` = **`mit`** (+ tag `license:mit`) | permissive — fetched |
| `HuggingFaceTB/everyday-conversations-llama3.1-2k` (added) | https://huggingface.co/datasets/HuggingFaceTB/everyday-conversations-llama3.1-2k | `cardData.license` = **`apache-2.0`** (+ tag `license:apache-2.0`) | permissive — fetched (simple everyday dialogue side) |

Note on `smoltalk` ("SmolTalk-2"): neither `HuggingFaceTB/smoltalk` nor `HuggingFaceTB/smoltalk2`
carries a dataset-wide `cardData.license`. Both cards declare only their **new** subsets
Apache-2.0 and defer the rest to the original public datasets. This task therefore fetches only the
four card-declared Apache-2.0 SmolTalk configs; it does **not** fetch either mixed aggregate.

## 2. What was fetched

15 files, **155,402 rows**, **299,947,261 bytes (299.9 MB)** raw JSONL. The 12 prior permissive
files are byte-identical with unchanged sha256; the 3 share-alike files were added 2026-09-26. Every
output file's sha256 was recomputed from disk and matches `manifest.json`.

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
| `dolly_15k.default.train` | `databricks/databricks-dolly-15k` | `default` | `train` | train | CC-BY-SA-3.0 | 15,011 | 13,261,723 | `96fe2257ef4202020e4f4e64e0853a4cd256f26553039db22d1bafe6e22353eb` |
| `tinystoriesinstruct.default.train` | `roneneldan/TinyStoriesInstruct` | `default` | `train` | train | CDLA-Sharing-1.0 | 50,500 | 8,005,244 | `5fdc6ac0cb4b4af0349be4691b53a31f73c7d978a34064d17a8457c41c78b825` |
| `tinystoriesinstruct.default.validation` | `roneneldan/TinyStoriesInstruct` | `default` | `validation` | heldout | CDLA-Sharing-1.0 | 9,600 | 1,507,108 | `b667c7b804002b3debccb685e89a49c1b655ca3007ddb776ad0fa158695c8954` |

- Share-alike extension total: **75,111 rows, 22,774,075 B (22.8 MB)** added on top of the original
  12 permissive sources.
- Capacity split (all 15 files): **train 278,792,152 B (93.0%)**, **held-out 21,155,109 B (7.0%)**.
- Source composition: SmolTalk (Apache-2.0) 180.9 MB over 4 configs; UltraChat-200k (MIT) 76.8 MB;
  everyday-conversations (Apache-2.0) 6.4 MB; Dolly-15k (CC-BY-SA-3.0) 13.3 MB;
  TinyStoriesInstruct train (CDLA-Sharing-1.0) 8.0 MB; TinyStoriesInstruct validation (CDLA-Sharing-1.0) 1.5 MB.
- `heldout` files are the **official upstream test splits**. They must never be used for training.
  The `.test` files are bounded **prefixes** of the official split except
  `smoltalk_smol-constraints.test` (all 1,812) and `everyday-conversations…test_sft` (all 119).
  `tinystoriesinstruct.default.validation` is a bounded prefix of the official `validation` split
  (9,600 of 218,380 rows). **Dolly-15k has no official held-out split** (single `train` split only),
  so no Dolly held-out file is fabricated.
- Dataset revisions observed and recorded: `HuggingFaceTB/smoltalk` `5feaf2fd3ffca7c237fc38d1861bc30365d48ffa`,
  `HuggingFaceH4/ultrachat_200k` `8049631c405ae6576f93f445c6b8166f76f5505a`,
  `HuggingFaceTB/everyday-conversations-llama3.1-2k` `14f543216b9ba42b6b951dc5bd199460d193b162`,
  `databricks/databricks-dolly-15k` `bdd27f4d94b9c1f951818a7da7fd7aeea5dbff1a`,
  `roneneldan/TinyStoriesInstruct` `ee050ed1f8720795be342921335e821856a2b42e`.

## 3. Story / dialogue side

The task asked for a balance between instruction/chat and simple story dialogue. Two story sources
are relevant:

- **Fetched here (share-alike extension):** `roneneldan/TinyStoriesInstruct` (CDLA-Sharing-1.0) —
  50,500 instruction-formatted train rows (8.0 MB) plus the official `validation` held-out prefix
  (9,600 rows, 1.5 MB).
- **Pre-existing, not copied:** the owner checkout holds
  `/Users/casey.allard/uor-r4/.uor-models/research/issue-1014/raw/TinyStoriesV2-GPT4-train.txt`
  (2,227,753,162 B, sha256 `6418d412de72888f52b5142c761ac21a582f7d1166f0bfbdb5f03ccfdec90443`, mtime
  2026-08-30), plus `tinystories_train.u16` (1,110,771,074 B). They are left in place and not copied;
  they derive from `roneneldan/TinyStories` (also CDLA-Sharing-1.0).

`everyday-conversations-llama3.1-2k` (Apache-2.0) adds a small permissively licensed simple,
multi-turn everyday-dialogue component (6.4 MB).

## 4. Withheld (content NOT fetched)

The two share-alike sources previously listed here were **approved by the owner on 2026-09-25** and
are now fetched under §2. Remaining withheld items:

| Dataset | Licence | Why withheld |
|---|---|---|
| `HuggingFaceTB/smoltalk` aggregate `all` config | mixed | Includes public datasets under their own (unverified here) licences. Only the four card-declared Apache-2.0 configs were fetched. Owner decision if the mixed aggregate is wanted. |
| `HuggingFaceTB/smoltalk2` | mixed (named splits Apache-2.0) | Verified, not fetched: the bounded budget was already met with uniformly permissive sources. Named Apache-2.0 splits remain available as a second instruction source on owner request. |

**Note:** the pre-existing local TinyStories corpus derives from `roneneldan/TinyStories`
(CDLA-Sharing-1.0). The same obligations as §5 apply; this task neither fetched it nor modified it.

## 5. Licence obligations

Recorded so downstream corpus preparation, training and any publication can comply. These are
licence-term summaries, not legal advice; the canonical licence text governs.

### CC-BY-SA-3.0 — `databricks/databricks-dolly-15k`

Canonical text: <https://creativecommons.org/licenses/by-sa/3.0/legalcode>.

- **Attribution (BY):** credit the source `databricks/databricks-dolly-15k` and its authors, link the
  licence, and indicate if changes were made.
- **ShareAlike (SA):** if the data or adapted/derived material is distributed, it must be under
  CC-BY-SA-3.0 or a licence compatible with it.
- **No additional restrictions:** do not apply legal terms or technological measures that restrict
  others from exercising the licensed rights.
- **Scope:** covers the raw data and adapted/derived data. Whether trained model weights are a
  "derivative work" is legally unsettled. Downstream publication of a model trained on Dolly data
  should be treated as share-alike-encumbered unless separately cleared.

### CDLA-Sharing-1.0 — `roneneldan/TinyStoriesInstruct` (and the pre-existing TinyStories corpus)

Canonical text: <https://cdla.dev/sharing-1-0/> (SPDX `CDLA-Sharing-1.0`).

- **Notices:** retain copyright, licence and attribution notices and provide recipients a copy of the
  agreement.
- **ShareAlike:** distribution of the Data, or of "Enhanced Data" derived from it, must be made under
  CDLA-Sharing-1.0.
- **No additional restrictions** on a recipient's exercise of the granted rights.
- **Scope:** "Enhanced Data" means data resulting from modifying or augmenting the Data (for example
  combining it with other data). Whether trained model weights are "Enhanced Data" is legally
  unsettled. Downstream publication of a model trained on this data should be treated as
  share-alike-encumbered unless separately cleared.

The permissive sources carry no share-alike condition: Apache-2.0 (SmolTalk, everyday-conversations)
requires attribution/notice retention; MIT (UltraChat-200k) requires the copyright/permission notice.

## 6. Exact reproduction

Requires only Python 3 standard library (verified 3.14.6) and network access to
`datasets-server.huggingface.co` and `huggingface.co`.

```sh
# Full deterministic re-fetch (resumable; skips completed sources):
python3 /Volumes/UOR-Workspace/uor-r4-lab/chat-v0-20260925/raw/fetch_chat_v0.py

# Single permissive source:
python3 /Volumes/UOR-Workspace/uor-r4-lab/chat-v0-20260925/raw/fetch_chat_v0.py \
  --only smoltalk_smol-magpie-ultra.train

# Share-alike extension sources (owner-approved 2026-09-25):
python3 /Volumes/UOR-Workspace/uor-r4-lab/chat-v0-20260925/raw/fetch_chat_v0.py \
  --only dolly_15k.default.train
python3 /Volumes/UOR-Workspace/uor-r4-lab/chat-v0-20260925/raw/fetch_chat_v0.py \
  --only tinystoriesinstruct.default.train
python3 /Volumes/UOR-Workspace/uor-r4-lab/chat-v0-20260925/raw/fetch_chat_v0.py \
  --only tinystoriesinstruct.default.validation

# Resume-equivalence proof (pause 3 pages, then resume):
python3 .../fetch_chat_v0.py --only tinystoriesinstruct.default.validation --page-limit 3
python3 .../fetch_chat_v0.py --only tinystoriesinstruct.default.validation

# Regenerate manifest.json + README.md from existing state (no network):
python3 /Volumes/UOR-Workspace/uor-r4-lab/chat-v0-20260925/raw/fetch_chat_v0.py --manifest-only

# Clean re-fetch (destructive to the SSD raw dir only):
#   rm -rf <raw>/data <raw>/state
```

Fetcher identity: `fetch_chat_v0.py` sha256
`497cf1a19a4ec0854ffecd6f891151e27e083c69b1ca12cf6e88c212da5575f7` (the extension fetch ran
`6643d223ad20932c32581f05e009b13dd9db8c6d164bf0588f5978dae769a140`; the final hash differs only by a
generated-README heading text correction, with identical fetch logic and identical data sha256). The
12-source predecessor is preserved byte-for-byte as `fetch_chat_v0_v1_2026-09-25.py` (sha256
`3d8fd333dcaa4b63350f8970a7510793a6c4b8f174ab020ee6864a91afeba3ca`). The fetcher pages the
datasets-server `rows` API in fixed 100-row pages from offset 0 (maximum `length` is 100), requests
`truncate=0`, and writes each line as `{"row_idx": <int>, "row": <original record>}`.

One-page manual probe:

```sh
curl -s 'https://datasets-server.huggingface.co/rows?dataset=HuggingFaceTB%2Fsmoltalk&config=smol-magpie-ultra&split=train&offset=0&length=1&truncate=0' | python3 -m json.tool
```

## 7. Verification actually executed

| Check | Result |
|---|---|
| sha256 recomputed from every output file vs manifest | 15/15 match (12 original + 3 extension) |
| JSONL parse + envelope (`row_idx`,`row`) present | 155,402/155,402 lines valid |
| `row_idx` contiguous 0..N-1, no gaps/duplicates | 15/15 files contiguous |
| Record shape (messages / prompt / instruction / text present) | 155,402/155,402 |
| `truncated_cells` returned by API | 0 total (no truncation) |
| Resume equivalence (3-page pause+resume vs single pass, original) | byte-identical: `42bf43c7…` on `smoltalk_smol-constraints.train` |
| Resume equivalence (extension) | byte-identical (`cmp` clean): `b667c7b8…` on `tinystoriesinstruct.default.validation` |
| Transient API errors | original: 89× HTTP 429 + 2× HTTP 502; extension: 67× HTTP 429, 0× 5xx. All recovered by backoff; 0 exhausted retries, 0 `WARN`/`ERROR`/traceback |

## 8. Cost

- Original 12-source run: ~32 min wall (22 min fetch + ~10 min probe/verification);
  10.5 s user + 5.0 s system; ~277 MB downloaded.
- Share-alike extension: ~17 min wall (2026-09-26T04:19Z–04:36Z) for 22.8 MB of new JSONL,
  dominated by datasets-server 429 backoff; ~23 MB network plus metadata, CPU negligible (network-bound).
- New storage on SSD: raw dir `du` = 288 MB; volume free = 208,157,092 KiB (~198.5 GiB of 200 GB).
- No cargo, no build, no external/paid compute; no destructive cleanup and no impact on the
  concurrently running corpus/training processes (read-only observation only).

## 9. Limitations and blockers

- Reproduction is bounded by the upstream dataset revision: the `rows` API serves the current
  revision and cannot be revision-pinned. Revision shas observed at fetch time are recorded; a
  future upstream change can alter results.
- No tokenizer, model format, or train/held-out split construction was produced — that is R1.
- **Row/byte reconciliation (measured):** TinyStoriesInstruct records are only ~150–190 B of JSONL,
  so the requested "100–200 MB" band would require ~0.65–1.3M rows and ~2 h of datasets-server 429
  backoff. The request's explicit **~40–50k row** bound was used instead, yielding 8.0 MB train +
  1.5 MB validation. Dolly-15k was fetched in full (13.3 MB). The extension therefore adds 22.8 MB,
  not 100–200 MB; extending to the byte band is a one-line `byte_goal` change plus a re-run if wanted.
- Capacity split is **93.0% train / 7.0% held-out** across the 15 files.
- **Dolly-15k has no official held-out split** (single `train` split); no held-out file was
  fabricated for it. TinyStoriesInstruct's official `validation` split is the only new held-out.
- **Licence obligations remain in force** (see §5): both new sources are share-alike. Any downstream
  publication of a model trained on them should be treated as share-alike-encumbered unless
  separately cleared. This is a compliance item, not an acquisition blocker.
- No blocker remains on the acquisition itself; all 15 files are verified and the extension is complete.

## 10. Handoff boundary

- Raw data stays on the SSD at `/Volumes/UOR-Workspace/uor-r4-lab/chat-v0-20260925/raw/`; it is not
  committed and no model/token artefacts were produced. The extended fetcher (`fetch_chat_v0.py`),
  its preserved predecessor (`fetch_chat_v0_v1_2026-09-25.py`), the per-source `state/`, and the
  regenerated `manifest.json`/`README.md` also live there and are not committed.
- Only these two files are handed to the lead for commit (the lead commits; this task did not):
  `docs/evidence/chat-v0-dataset-manifest-2026-09-25.json` and this document.
- R1 consumes `data/*.train.jsonl` for training and the official held-out files
  (`data/*.test*.jsonl`, `data/tinystoriesinstruct.default.validation.jsonl`) for the frozen
  held-out panel, applying the response mask at the JSONL level. The 12 prior files are unchanged;
  the 3 extension files and their share-alike obligations are additive.
