# M1 bar (f): the held-out-phrasing panel

Lab: claude. Item: [m1-heldout-phrasing-transfer](../../roadmap/items/m1-heldout-phrasing-transfer.md). Order: [claude-211](../../loop/tasks/m1-heldout-phrasing-transfer/claude-211/ORDER.md).

Bar (f) (D82, CHARTER §4 item 8): the saved model beats TRAIN's unigram and T2 bigram tables on a fresh held-out-phrasing panel, with bar (e)'s paired band. This page records the panel. It is opened once, for bar (f), and enters no other decision.

## The panel and its control

Both were drawn on 30 Oct by claude-204's `heldout-panel` (`docs/labs/one-model-end-to-end/instrument/`), with the pinned tokenizer of `w2-inputs/2026-10-25/driver-data/`. Each seed is one no unit has used; claude-204 used 2041–2043.

```text
heldout-panel out=p-f       tokenizer=TOKENIZER cell=dev,dev     conversations=400 seed=2051 protocol=2
heldout-panel out=p-f-train tokenizer=TOKENIZER cell=train,train conversations=400 seed=2052 protocol=2
```

| Panel | Cell | Seed | Tokens | Reply targets | `tokens.u16` sha256 | `response_mask.u8` sha256 |
|---|---|---|---|---|---|---|
| **p-f** (bar (f)) | `dev,dev` | 2051 | 50,061 | 15,953 | `e6abbdf0dd2f2fff1810d5e1fd95f6cfa20582c57c98e990a1d9c852e5e43fef` | `c63c625a9897b4ee42043028addb3f9f831d7ecc28976cae8223ba0050e5c910` |
| **p-f-train** (control) | `train,train` | 2052 | 50,950 | 16,800 | `b3bae826a3e25b18e163ea6d785593344af42e7076a1138c2c65a94e20a319a4` | `00c9b3b3e3e4208fc5e7de4f43a568cf88edd8a0fc9f76996dfdf453096d7934` |

**Where they are stored:** the HF store at `inputs/claude/heldout-panels-2026-10-30/`, with a `SHA256SUMS` and the census.

## The census, against the saved model's TRAIN

- **TRAIN** is `mw-balp/train` under `w2-inputs/2026-10-25/driver-data/`. It is pinned by `hamilton_sheet.rs:41-46`: `tokens.u16` `3dee858f84b750f507ceb3dc3f393711741950954735efaaf61bf704c017a4c8`, `response_mask.u8` `b5e7352d67d6883a833155a5fd397a03f6d4f1b2649e3d4ec3e2ea7d95b949e3`.
- **It is the same file the M1 sheets pin** (`m1-fresh-panel-transfer-2026-10-26.md:62-64`). That sheet gives it 26,214,400 tokens; the census reads the file as it is, 7,512,797 tokens, as the item asks.
- **Command:** `train-in-space --data-root DATA --census 1 --fresh-v2 FRESH --panel p-f=DIR --panel p-f-train=DIR`. Output: [census-2026-10-30.json](census-2026-10-30.json).

| Panel | Prompts | Exact prompts in TRAIN | Value-masked prompt templates in TRAIN, K = 1,024 | New-template (≤ 0.5)? |
|---|---|---|---|---|
| **p-f** | 1,409 | 0.000 | **0.000** | **yes** |
| **p-f-train** (control) | 1,482 | 0.611 | 0.637 | no, as a control must |
| DEV, for scale | 1,226 | 0.608 | 0.642 | no |
| Fresh V2 (bar (e)'s panel) | 1,279 | 0.639 | 0.665 | no |

- **At K = 1,024, no p-f prompt's value-masked template occurs in TRAIN.** At K = 256, 0.050 do. Fresh V2, bar (e)'s panel, has 0.665 at K = 1,024.
- **Replies are still template-shared:** 0.625 of p-f's replies are verbatim TRAIN replies. The generator shares reply templates across phrasings (claude-204 found the same on P-dev). So bar (f) tests reading a new prompt into a known kind of reply.

## The result: bar (f) passes (30 Oct, `74675bb3f`)

**Tokens per second first.** The saved artifacts serve at a median 10,876 decode tok/s (seed 1) and 10,566 (seed 2) on an Apple M1 laptop, with prefill at 2.67M tok/s ([toks-s1.json](evidence/toks-s1.json), [toks-s2.json](evidence/toks-s2.json)).

| On p-f, 15,953 rows | Seed 1 | Seed 2 |
|---|---|---|
| Saved model, integer top-1 | **7,850** | **7,872** |
| T2 bigram | 7,375 | 7,375 |
| Rows only the model gets, vs T2 (`b`) / only T2 gets (`c`) | 1,038 / 563 | 1,051 / 554 |
| Lead over T2, against its band of 81 | **475** | **497** |
| Unigram | 1,409 | 1,409 |
| Lead over the unigram, against its band of 162 | **6,441** | **6,463** |
| Refused rows, reference mismatches, B4 | 0, 0, pass | 0, 0, pass |

- **The outcome is `PASS_BAR_F_HELDOUT_PHRASING`** ([panel-score.json](evidence/panel-score.json)). The band is `max(24, ⌈2√(b+c)⌉, |M(1) − M(2)|)`, and the two-seed spread is 22.
- **Builds A and B** (`0960545437603586493f54524545728c9065d5f0219a7b21d2105aa3bb579d14` and `bcba357778c445dfd96d8817c683f9fe7b19e6bca4ddaeeca6d694339c4eec37`) gave byte-identical reports.
- **Build A first reproduced the saved model's DEV top-1,** 7,002 and 7,001, before it read the panel.
- **Bar (e)'s copy-successor condition, reported only:** the model gets 1,903 and 1,904 of the 4,077 copy-successor rows, against T2's 1,260. So it passes that condition too.
- **The first attempt was an instrument fault.** It built without `--target` and failed to compile, `E0463`, before reading any panel byte. It was repaired, and is not counted.
- **The seal** is at `results/claude/m1-heldout-phrasing-transfer-2026-10-30/run-74675bb3/` in the HF store, with a `SHA256SUMS` of 61 files.
- **What this shows:** a model that passed (a) to (e) also beats both count tables on prompts whose phrasing templates TRAIN does not hold (0.000 at K = 1,024, 0.050 at K = 256). It does so at both seeds, by 5.9 and 6.1 times the band against T2. Its replies still come from templates TRAIN shares (0.625 verbatim), so this is reading new prompts into known kinds of reply.
- **Its close grade pins this panel.** M1's close grade accepts a bar (f) report on p-f only (`P_F_TOKENS_SHA256` and `P_F_MASK_SHA256`), as bar (e) pins Fresh V2. `panel-score` refuses the template-sharing controls (P-train, P-ret, p-f-train). Both rules come from the review.
- **The grade reads flags, not margins.** Like bar (e)'s, it trusts the report's `bar_f_pass` and `outcome`, and does not recompute the margins from the per-seed numbers.
