# REPORT: claude-400, m2-donor-corpus-generation

Lab: claude. Order: [ORDER.md](ORDER.md). Sheet: [`docs/harness/prep/m2-donor-corpus-2026-11-15.md`](../../../../harness/prep/m2-donor-corpus-2026-11-15.md) (amendments 1–6). Releases: D87, D89.

## What did not work, first

- **[claude] The API probe's pre-registered math criterion failed as measured.** Exact-string agreement with pod 3 on 38 math prompts was 16/38 at `high` and 17/38 at `low`. Every one of the 38 replies was read: the mismatches are differences in how the answer is written (form, units, wording), with the same value. I classified this after the fact as an instrument failure (an exact-string check is the wrong instrument for free-form math answers) in a decision review (§5.1), and chose `low`. Casey can reverse it; the probe files are in `local/m2-run/api-probe/` and the numbers are in the sheet's amendment 6.
- **[claude] The API set has no seed.** DeepSeek's API takes none, so the 457,601 API records cannot be regenerated bit for bit. Part 0 (pod 3, vLLM) was seeded. The two parts are sealed and stored apart (`part-0`, `api-a`) so the seeded part stays identifiable.
- **[claude] The served model is not pinned to a checkpoint for the API set.** The API exposes no revision or weights hash. Every API record's `model` field reads `deepseek-flash`; part 0 names `deepseek-ai/DeepSeek-V4.1-Flash-1017` at revision `08c8657f…`.
- **[claude] Instrument negative: the live `errors` field in rate rows is unreadable against DeepSeek.** `post()` counts a request as an error once `send_json` returns and uncounts it once the body parses. DeepSeek sends its 200 headers at once and holds the body open while the model thinks, so mid-segment the field is about the number of requests reading their body (1,936 of 2,000 at one point). Only the end-of-segment value is a failure count; every segment was judged by that value. Not patched mid-run (it would change the pinned binary).
- **[claude] Instrument negative: one straggler held A3 open 37 minutes.** A3 reached its budget at about 11:19Z; one request stayed in flight from 11:20 to 11:57Z, retrying, with no kept tokens added. A per-request wall-time cap is the fix; it cost time, not money.
- **[claude] The 4,096-token count was not done.** The instrument has no `count` command. The ORDER asks for it only "if that is feasible"; it is left open below.
- **[claude] Two operational slips, caught before any harm.** (1) The first HF and wick copy jobs used `set -- $pair` under zsh, which does not word-split, so both failed on a path named "out part-0" and copied nothing; they were stopped and rerun with explicit arguments. (2) Pod 3's guard downed the pod after its 1,200 s no-run grace, about 30 minutes after the STOP; that grace is pod time the corpus did not use (about $21 at $43.41/h).
- **[claude] Earlier pods** (pod 1 had no NVLink and was downed at once, about $0.52; pod 2 never reached SSH and was deleted by `up`, about $11) produced no data. Both are in the shared notes and the sheet's amendments.
- **[claude] The ORDER's stop list says "Nothing on wick".** The corpus backup on wick's `plexpool/uor-r4-corpus` dataset follows Casey's later instruction relayed by the parent session (quoted below); nothing was written to wick's root disk and no other process there was touched.

## What changed

- `docs/labs/m2-donor-corpus/`: the standalone Rust client (`bank`, `generate`, `seal`, `verify`, `balance`), with an OpenAI-compatible API mode (bearer key read from a file at segment start, never logged), the served `model` and cache-hit tokens recorded per record, and `api-watch.sh`, the balance-based spend watcher (sums every fall in the balance, fails closed after 10 unreadable readings). `pod.sh` and `drive.sh` run and guard the pods.
- The sheet's amendments 1–6 and this REPORT. One notebook row in `docs/research/notebook/data-tokenizer.md`.
- No model code, no training, no evaluation. The corpus itself is not in the repository; it is in the HF store and on wick.

## Measurements

Casey's words for the API path, relayed by the parent session (verbatim): "ok sure keep the pod up i guess since it wins until 5am est... i will reload the deepseek api key, please go ahead and build that and begin (also be conscious of wick's hard drive space and the hard drive space on this local laptop as well) wick has tons of other hard drive space but it may be slower so please be smart about what you are doing"; and "I already have deepseek installed on wick through opencode". The provider's terms are quoted in the sheet's amendment 6.

**The corpus: 1,009,576,044 kept tokens in 528,615 records**, counted in the donor's own tokenizer (the server's usage counts).

| Part | Engine | Records | Kept tokens | Generated | Dropped | Merkle root |
|---|---|---|---|---|---|---|
| part-0 | vLLM 0.33.0 on pod 3 (4×B200, Vast offer 51062009), seeded | 71,014 | 136,098,802 | 153,480,092 | 1,596 | `ecdceae33a6891f5cdac2da2cf9a3c1dba85d907e52cdad87e33d6d7ec32e153` |
| api-a | DeepSeek API `deepseek-flash`, `reasoning_effort: low`, off-peak | 457,601 | 873,477,242 | 992,569,834 | 10,095 | `f8a845b79bb5f1a9b12c25891eda2e90772a48175e54ee637e27fcbf8e645254` |

- **Throughput.** Pod 3's measured hour (06:54:27–07:57:10Z, 256 in flight): R = **9,271 kept tok/s**, about 9,516 over its last 50 minutes, kept fraction 85.9%. API, by segment (`generate --api openai`, laptop client, instrument `4ad0c58d9`):

  | Segment | In flight | Window (UTC) | Kept tok/s | Errors at end |
  |---|---|---|---|---|
  | A1 | 256 | 10:01:03–10:07:43 | 27,376 | 0 |
  | A2 | 1,000 | 10:08:31–10:35:19 | about 132,500 | 522 / 121,264 (0.43%) |
  | A3 | 2,000 | 10:38:19–about 11:19 (budget), 11:57:11 end | about 274,000 (16.0–16.8M per minute) | 791 / 370,910 (0.21%) |

  API request latency in A3: p50 7.3 s, p95 44.4 s, max 130.6 s. All API generation ran in the off-peak window (from 10:00Z on a Monday); no request crossed into a peak window.
- **Quality counters.** API: 3 turns without reasoning in 488,320; drops 9,689 for length (the 16,384-token cap), 401 empty, 5 markup. Sources: WildChat 304,584, Numina 77,233, OpenThoughts 71,766, OASST2 3,300, GSM8K 718; 30,719 two-turn records.
- **Wall time.** Generation 06:28:53Z to 11:57:11Z (5 h 28 min); the unit from the first lease at 03:46Z, about 8 h 11 min.
- **Dollars: about $879 balance-based, about $854 corpus-only.** Pod 3 $225.82; pod 2 about $11; pod 1 about $0.52. API: the balance fell **$641.64** from the watcher's first reading (10:00:40Z, $1,167.50) to $525.86. That figure may include other use of the same account (OpenCode on wick). The corpus-only cost from our usage counts is **about $617**: 992,569,834 generated × $0.60/M = $595.54, plus 141,093,485 prompt tokens × $0.15/M = $21.16 (counted as misses; 15,817,451 were cache hits), plus about $0.5 for dropped requests' prompts. The probes cost about $0.40 more. The watcher's $850 STOP never fired.
- **Storage.** zstd `-9`; each file checked by decompressing and comparing its sha256 with the raw hash taken before compression; raw copies deleted only after their check. Part-0: 79 files, 610,297,799 → 164,656,256 bytes. API-a: 455 files, 3,948,381,311 → 1,056,982,828 bytes. Each folder's `storage-manifest.jsonl` lists the raw and compressed sha256 and sizes; `receipt.json` stays uncompressed.

## Checks run at the head

- **Instrument tests:** `CARGO_TARGET_DIR=local/m2-instr-target CARGO_INCREMENTAL=0 ~/.cargo/bin/cargo test` in `docs/labs/m2-donor-corpus/instrument/`: 26 passed, 0 failed, 1 ignored (lib 15, e2e 8, records 3).
- **Seal and verify** (`donor-corpus seal`, then `verify --prompts local/m2-run/bank.jsonl`, the full bank): part 0 and the API set each `verified: true` with the roots above. A sealed copy of the API set after A2 verified too (117,148 records, root `517da6c0…`) before pod 3 was stopped.
- **HF readback:** `uor-store get results/claude/m2-donor-corpus-2026-11-16/{part-0,api-a}/` into a scratch folder; every file's sha256 matched the manifest (81 and 457 files); after decompression `verify` gave `verified: true` with the same roots, records and kept tokens. The scratch folder was deleted.
- **Wick copy:** `rsync -a --partial` to `/mnt/plexpool/uor-r4-corpus/m2-donor-corpus-2026-11-16/{part-0,api-a}/`, then `sha256sum -c` run on wick against the manifests: all 81 and 457 files OK.
- **Pods:** `uor-pod status` after pod 3's down: 0 running.
- `uor-harness prep-check` on the sheet and the loop preflight, run before the PR was marked ready (results in the PR record).

## Skills used

None were invoked as skills. The `native-space-cycle` SKILL was read and does not apply (an inventory-class unit, as the ORDER says).

## Still open

- The 4,096-token count of the corpus (ORDER "Done when" 1, "if that is feasible"): needs a `count` command. A follow-up, not tracked yet.
- The two instrument negatives above (live `errors` field; no per-request wall-time cap), for any later run of this client.
- Laptop swap stood at 11.8 GB during the run, mostly Antigravity's `language_server` (7.7 GB); free disk fell from 21 to 12 GiB at its lowest. That is the owner's call.

## Plain summary

The project wants to teach a small model from a stronger one's written answers. This unit made the first batch of that teaching text: about one billion tokens (word pieces) of questions answered by DeepSeek V4.1 Flash, each answer stored with the model's reasoning and its final reply as two separate pieces of text, and nothing else (no scores or probabilities). The first 136 million tokens came from the model running on a rented GPU server; the rest came from DeepSeek's own paid service, which turned out about thirty times faster and cheaper at night-time prices. Each of the two parts is sealed with a fingerprint (a Merkle root) that changes if any record changes, compressed, and stored in the project's Hugging Face store and on the owner's machine wick. It cost about $854 to $879 in all. No model was trained here.
