# Prep sheet: D87's first sealed donor corpus, 1B kept tokens from DeepSeek V4.1 Flash with reasoning (m2-donor-corpus-generation, claude-400, 15 Nov 2026)

Lab: claude. Item: [m2-donor-corpus-generation](../../roadmap/items/m2-donor-corpus-generation.md) (#2848). Order: [claude-400](../../loop/tasks/m2-donor-corpus-generation/claude-400/ORDER.md). Code: branch `claude/claude-400-donor-corpus`, PR #4028. Release cited at dispatch: D87 (§3.10), once PR #4027 is on `main`.

TEST FITNESS: PENDING - a fresh-context non-author round runs on this sheet before any lease

## The question

At what measured rate and cost does DeepSeek V4.1 Flash on one rented 8×H200 pod produce hard reasoning-plus-answer texts? If the rate fits D87's bounds, the unit produces and seals 1B kept tokens from it. If it does not, D87's fallback (gpt-oss-120b) produces them instead, under the branch registered below.

## Steps

1. **Prompt bank** (laptop, no model): `donor-corpus bank --sources local/m2-sources --panels data/panels --seed 20261116 --out bank.jsonl`, at the instrument commit named in the receipt. The sources are fetched at pinned revisions by `docs/labs/m2-donor-corpus/fetch-sources.sh`, which checks each file against Hugging Face's LFS sha256. The bank's sha256, size and per-source counts are fixed here before the FIT: `BANK_SHA256` (filled in below).
2. **Lease** after D87 is on `main`: one Vast 8×H200 verified datacenter offer at ≤ $35/h, with all labs' running pods counted. The lease uses `UOR_POD_MAX_RATE=35 UOR_POD_VAST_DISK_GB=1200 UOR_POD_VAST_MIN_INET_MBPS=1000 UOR_POD_IMAGE=vllm/vllm-openai:v0.33.0-x86_64 scripts/pod/uor-pod up --gpu h200 --count 8 --no-bootstrap --owner-approved` (the owner approval is D87). The pod has ≥ 1 TB of RAM for two Engram host-table copies. A completion watcher is armed in the same command that launches the run (L-004).
3. **Setup** (`docs/labs/m2-donor-corpus/pod.sh setup`): download `deepseek-ai/DeepSeek-V4.1-Flash-1017` at revision `08c8657f02291e11b6f6b63e0e16014f8e748518` (MIT, not gated, about 511 GB in 48 shards). Check every LFS file against Hugging Face's sha256. Build the client on the pod at the receipt's instrument commit.
4. **Serve:** one vLLM server, `--data-parallel-size 2 --tensor-parallel-size 4`, with the recipe's H200 settings: `--tokenizer-mode deepseek_v41`, `--kv-cache-dtype fp8`, `--engram-config '{"cpu_offload":true}'`, `--reasoning-parser deepseek_v41`, `--language-model-only`, no speculative decoding (the recipe marks it for small batches), `--max-num-seqs 256 --max-num-batched-tokens 8192`.
5. **Smoke** (instrument check, not corpus): `generate --limit 32 --parallel 8` into `smoke/`. It checks that `reasoning` and `answer` arrive in separate fields with no markup, that two-turn prompts run, and that `seal`/`verify` pass on the smoke directory. The smoke output is never part of the corpus.
6. **Tuning, at most 20 minutes:** client `--parallel` 256, then 512, each for 8 minutes from the start of the bank. The setting with the higher kept tokens per minute over its last 5 minutes is kept. These records stay in the corpus, because they are the bank's first prompts at the same sampling.
7. **The measured hour:** 60 minutes at the kept setting, read from `rate.jsonl` (kept tokens in donor tokens per minute). The projection is P_cost = spent + (1B − kept) / R / 3600 × rate_per_h, and P_time = elapsed + (1B − kept) / R / 3600, where R is the hour's mean kept tokens/s.
8. **Continue or switch:** if P_cost ≤ $1,000 and P_time ≤ 48 h, the run continues to the 1B kept-token budget. Otherwise the DeepSeek run stops, its shards are saved and kept as a separate sealed partial set, and the fallback branch runs.
9. **Offload:** every 10 minutes, finished shards and their counts files are copied off the pod to the laptop and checked by sha256. That copy is the watcher's main job. `down` comes only after the last copy is verified.
10. **Seal and verify:** `seal` writes `receipt.json` (shard sha256s and Merkle root, the bank's sha256, the counts, rate and segment logs, the instrument commits, model, engine image and digest, GPUs). `verify` runs on the laptop, and again after a fetch back from the HF store (`results/claude/m2-donor-corpus-2026-11-16/`).
11. **Project tokenizer:** kept texts are counted in the 4,096-token tokenizer (sha256 `d36d3e87…`) on the laptop, if its encoder runs at that scale. This is a count only, with no bar.

**Fallback branch, registered now:** `openai/gpt-oss-120b` at revision `b5c939de8f754692c1647ca79fbf85e8c1e70f8a` (Apache-2.0, not gated), on the cheapest verified B200 offer that fits the cap (2×B200 was $24.79/h on 15 Nov). It uses the same vLLM image, client, bank (continuing after the DeepSeek prompts), sampling and stop rules. Reasoning comes from the analysis channel through vLLM's gpt-oss reasoning parser, with `reasoning_effort` "medium" passed through `--extra-body`. The same smoke, measured hour and bounds apply. The REPORT records why the switch happened.

## Bar

- **B1 size:** kept tokens ≥ 1,000,000,000 in the donor's tokenizer, so the budget stop was reached.
- **B2 receipt:** `verify` passes after the round trip through the store. It covers shard sha256s, the Merkle root, the bank hash, every record's messages against the bank, and the structural check: a record has exactly `id`, `kind`, `source`, `messages` and `turns`, and a turn has exactly `reasoning` and `answer`, with no markup and a non-empty answer.
- **B3 quality floor:** dropped prompts (length, empty, markup, http) ≤ 10% of those dispatched; `http` drops ≤ 1%.
- **B4 panels:** the bank holds no panel string (dropped by the guard; the count is reported).
- **B5 bounds:** dollars spent ≤ $1,100, which is the $1,000 bound plus at most one tuning or setup overrun; wall time from lease to `down` ≤ 60 h.

**Recorded, with no bar:** kept and generated tokens/s per minute; the share of empty reasoning; tokens per record by source and kind; reasoning versus answer bytes; request p50/p95; GPU and host-memory use; and the count in the project tokenizer.

**Reading map** (fixed before the run):
- **P:** B1–B5 hold. The corpus is sealed and stored. The REPORT gives tok/s, wall time and dollars.
- **F (fallback):** the measured hour projects > $1,000 or > 48 h. The fallback branch runs, and its own P/I/Q/S reading applies. The DeepSeek partial set is stored, marked partial, and is not mixed into the 1B corpus.
- **I (instrument):** B2 or B4 fails. The instrument is repaired, and nothing is used until a re-verify passes.
- **Q (quality):** only B3 fails. The corpus is sealed and stored with the failure named. Use waits on the cause.
- **S (short):** B1 fails because of the hard stop or the money bound. The partial corpus is sealed and stored. The REPORT gives the cause and the measured numbers.

## Base artifact

- **Donor:** `deepseek-ai/DeepSeek-V4.1-Flash-1017` at `08c8657f02291e11b6f6b63e0e16014f8e748518`, MIT (card and `LICENSE` read 15 Nov 2026). Recommended sampling is temperature 1.0 and top-p 1.0. Thinking mode is on, with reasoning effort 50 (the card's "low").
- **Engine:** vLLM `v0.33.0-x86_64`, image digest `sha256:91ee972ab751fb7180bc71c175865e36b091686600dca783d52b2511399810b3` on Docker Hub (2 Nov 2026). It runs on the pod, outside every compiled module (§3.4).
- **Instrument:** `docs/labs/m2-donor-corpus/instrument` (Rust, a client with no model code) at the commit each segment records.
- **Machine:** one Vast 8×H200. The offer id, host, GPU UUIDs and driver are recorded in the receipt.

## Data split

- **The prompt sources** are user turns only, at pinned revisions:
  - `allenai/WildChat-4.8M` at `c827c6df` (ODC-BY; English, non-toxic, not redacted, first user turn; 43 of 86 files);
  - `project-numina/NuminaMath-1.5` at `1b05109f` (Apache-2.0);
  - `open-thoughts/OpenThoughts-114k` at `bd093c39` (Apache-2.0);
  - `OpenAssistant/oasst2` at `179dd21f` (Apache-2.0; English root prompts, train file only);
  - `openai/gsm8k` at `740312ad` (MIT; main/train only, no test split).
  Every assistant text in them is ignored. ODC-BY asks for attribution, which the receipt and REPORT give.
- **Two-turn exchanges** (about 10% of the WildChat and oasst2 prompts) add one generic follow-up from a fixed list of 40 written in `bank.rs`. Both assistant turns come from the teacher.
- **Panel guard:** every prompt, every one of its sentences and every follow-up is checked against every string in `data/panels/` after lower-casing and dropping spacing and punctuation (the pilot's guard). A match is dropped.
- **Use:** the corpus is training data only, never a panel, a grader or a comparator (D41 (c)). Public benchmark problems may occur in the NuminaMath and OpenThoughts prompts. They are not the project's evaluation panels, and this is stated in the REPORT.

## Minimum test power

- This is a data-production run, not a mechanism claim. There is no seed requirement.
- The rate is the mean over 60 one-minute windows, reported with the minimum and maximum window, so a warm-up or stall is visible.

## Cost

- **DeepSeek path:** $34.25/h was quoted on 15 Nov. At R kept tokens/s, 1B takes 277,778 / R hours. The $1,000 bound with about 1.5 h of setup (~$51) needs R ≥ about 10,000. The 48 h bound needs R ≥ about 5,800.
- **Unmeasured guess:** 10–30 h and $340–1,030. Weights download at about 1 GB/s; setup is estimated at 1–1.5 h.
- **Fallback:** about 5–10 h on 2×B200 at about $25/h, if the published per-B200 rate holds. That is a hypothesis.
- **Caps:** at $34.25/h this run alone nearly fills the $35/h cap, so it is one pod, not four, and no other lab can lease meanwhile. Other labs' pods are counted at lease time, and the run waits if they would breach the cap.

## Stop rule

- Dispatch stops at 1B kept tokens, and the prompts in flight finish.
- After the measured hour, the switch rule of step 8 applies.
- Hard stops are reported as S:
  - dollars spent reach $1,100;
  - wall time from lease reaches 60 h;
  - the bank runs out before 1B, in which case the REPORT gives the kept total, and a top-up bank is a new sheet.
- **Pod failure:** shards are already off the pod. One re-lease is allowed under this sheet. `generate` resumes from the finished shards, and the cost bounds count both pods.
- One production pass only (D41 (f)). The corpus is not regenerated under this sheet.
