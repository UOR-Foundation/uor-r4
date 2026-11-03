# REPORT: one-model-end-to-end, claude-266 (the one model trained as served at full length serves as it was trained: all six exports pass I-3; the two-token copy key splits by seed, so (b) stays the base)

Lab: claude. [ORDER](./ORDER.md). PR #3605, item #3407.
Sheet: [one-model-end-to-end-full-unsnapped-2026-11-03](../../../../harness/prep/one-model-end-to-end-full-unsnapped-2026-11-03.md).

**In one paragraph.** Both arms, (b) with copy key 1 and (k2) with `--hamming-copy-key 2`, trained as served for 8,192 steps on the unsnapped D11 contract at seeds 1–3, one Vast RTX 5090 per seed, arms in turn, at `59f7770ae2d936cfb3ac7337cd041cb3f25ead0f`. All six runs trained, scored and snapshotted with exit 0. Exported raw and served in integers on the laptop, every export passes I-3: first tokens differ on 0, 0, 0, 0, 0 and 1 of DEV's 310 prompts, against a bar of 8. claude-265's 256-step models missed on 32 and 14. Whole replies match too, up to the artifact's 192-token context: 0–3 real divergences per export. So the sheet's second question is answered yes: the model trained as served serves as it was trained, at this scope (first tokens and whole replies on DEV). Part B's table is therefore read. (k2) is better than (b) at seed 1 and worse at seeds 2 and 3 on p-w, and R6's speed condition is unread. Row 4 (iii) decides: (b) stays the base, and the next copy-key unit is the slot key. The served-contract readout that Casey approved on condition ("if you still need") is not needed for this line.

## Tokens per second first

- **The integer model, laptop, `fbs-toks`, prefill included** (19,274 reply tokens per export, the serve run): (b) 796.3, 773.5 and 785.3 tok/s at seeds 1–3; (k2) 784.3, 760.1 and 717.5 tok/s.
- **Decode tok/s, R6's protocol** (`timing-266.sh`, seed-1 exports, 5 interleaved rounds of a 1-step and a 175–177-step run): **unread**. The protocol says a spread past 5% means the timing is run again, and past 5% again leaves the speed condition unread. The first run's conv-c (b) spread was 9.9%; the second run's conv-a spreads were 95.0% (b) and 37.1% (k2). The numbers, for the record only:

  | Run | conv-a b / k2 | conv-b b / k2 | conv-c b / k2 |
  |---|---|---|---|
  | 1 (`decode-final`) | 1,338 / 1,382 | 1,351 / 1,342 | 1,378 (9.9%) / 1,355 |
  | 2 (`decode-final2`) | 1,091 (95.0%) / 1,357 (37.1%) | 1,417 / 1,340 (5.0%) | 1,363 / 1,341 |

  The k2/b ratios range from 0.946 to 1.24, and only conv-b in run 2 is below 0.95. No speed claim is made.
- **Training, GPU, served, 8,192 steps:** about 26,000 context tokens/s at seeds 1 and 2 (26,266 / 26,145 and 26,010 / 25,851 for b / k2), and 22,354 / 22,143 at seed 3, a slower shared host.
- **The float scorer, GPU, serial:** 3,980–4,211 s per arm at seeds 1–2, and 7,612–7,891 s at seed 3. That is the instrument's speed, not the model's.

## What did not work

1. **R6 is unread.** Both timing runs had a reading past 5% spread, so the speed condition is unread (row 1's speed condition, and row 4's list). It changes nothing in the outcome, because (k2) did not meet row 1's other conditions. **Instrument note:** the measured window is short. The 1-step run's median is 0.24 s of process start and model load; the long run adds only about 0.13 s. So a 50 ms hiccup on a desktop with other apps open moves one reading by 40%. A decode timing that repeats the long run inside one process would remove the start-up from the difference. That is a change to the timing protocol, for a later sheet, not made here.
2. **(k2) does not lead.** Its R1 verdicts against (b) are better, worse, worse at seeds 1, 2 and 3, and its R1 b − c sums to −172 over the three seeds. On p-v (R5) it is better, worse and level. Its DEV copies (R4) never beat (b)'s by more than 143: they are lower at seeds 1 and 2 (1,102 vs 1,157; 1,058 vs 1,198) and 10 higher at seed 3 (1,106 vs 1,096).
3. **Arms are still rising at the end.** (b) at seed 2 gained 156 copies after step 4,096, and (k2) at seed 3 gained 229. Both are past R4's 143, so row 4 would also apply on that ground alone.
4. **Neither seed-1 export names Maya (R7).** Turn 3, "My best friend is Maya.", gets "Nice,ir lucky mesalsol sounds like a good friend." from (b) and "Got it, your friend is Ming me." from (k2). The question at turn 4 gets "Sure! What would you didn't tell me." and "A tomatoesday you.". The lucky number at turn 1 comes back as 3 and 7, not 47. The replies are now mostly English words, which claude-261's were not, but they do not recall.
5. **Exact replies on p-w are rare:** 0–90 of 1,202 per export (R2), with right openings on 65–325 (R3). This is the model's level on held-out phrasings at this data, not a parity effect, since the integer replies match the float ones.
6. **The 192-token context cuts 26 replies per export.** On those prompts the integer reply is an exact prefix of the float reply and prompt plus reply reach 192 positions, while the float scorer runs on to 64 reply tokens. They are counted as whole-reply differences by `serve-266.sh`, and they are not divergences.

## Measurements

**I-3, I-5 and I-6, every export** (`serve-266.sh`, log `serve-out.log`):

| Export | First tokens differ (bar 8) | Order mismatches | Whole replies differ | of which context cap | real divergences | I-5 copy key | I-6 raw contract |
|---|---:|---:|---:|---:|---:|---|---|
| b-s1 | 0 / 310 | 0 | 26 | 26 | 0 | [] ok | ok |
| k2-s1 | 0 / 310 | 0 | 27 | 26 | 1 | [2] ok | ok |
| b-s2 | 0 / 310 | 0 | 26 | 26 | 0 | [] ok | ok |
| k2-s2 | 0 / 310 | 0 | 29 | 26 | 3 | [2] ok | ok |
| b-s3 | 0 / 310 | 0 | 28 | 26 | 2 | [] ok | ok |
| k2-s3 | 1 / 310 | 0 | 27 | 26 | 1 | [2] ok | ok |

claude-265 at 256 steps: 32 and 14 first tokens, 287 and 240 whole replies. Seeds 1 and 2 were also served early, before seed 3 finished, and gave the same counts (seed 1 twice).

**R1–R3 on p-w (decides) and R5 on p-v (confirms)**, `grade.sh`, memory on, greedy:

| Seed | p-w prefix-sum b / k2 | exact b / k2 | openings b / k2 | R1 verdict (k2 vs b) | p-v prefix-sum b / k2 | R5 verdict |
|---|---|---|---|---|---|---|
| 1 | 560 / 1,103 | 90 / 0 | 156 / 282 | **better** (280 vs 154, floor 41.7) | 157 / 303 | better (74 vs 51) |
| 2 | 1,492 / 241 | 48 / 10 | 325 / 65 | **worse** (64 vs 324, floor 39.4) | 391 / 58 | worse (22 vs 86) |
| 3 | 973 / 808 | 0 / 3 | 238 / 210 | **worse** (74 vs 112, floor 27.3) | 211 / 210 | level (28 vs 26) |

I-4 held: every arm at a seed holds the same prompts in the same order.

**R4, DEV strata at the last eval (step 8,192), with step 4,096 in brackets:**

| Seed | b openings / copies / rest | k2 openings / copies / rest |
|---|---|---|
| 1 | 153 / 1,157 / 8,217 (148 / 1,031 / 8,163) | 163 / 1,102 / 8,205 (44 / 1,002 / 8,044) |
| 2 | 264 / 1,198 / 8,196 (186 / 1,042 / 8,144) | 122 / 1,058 / 8,204 (202 / 1,002 / 8,057) |
| 3 | 242 / 1,096 / 8,175 (76 / 999 / 8,098) | 259 / 1,106 / 8,106 (50 / 877 / 7,668) |

## The Bar, applied

Every export passes I-3 (exit 0, no order mismatch, at most 8 of 310 first tokens, I-5 and I-6), so part B's table is read in order, unchanged, with row 1's I-3 condition read as "first tokens match on DEV".

- **Row 1** needs (k2) better at all three seeds: no (seeds 2 and 3 are worse).
- **Row 2** needs (k2) worse at all three: no (seed 1 is better).
- **Row 3** needs level or worse at every seed: no (seed 1 is better).
- **Row 4** applies: better at some seeds only, R6 unread, arms still rising. Its clauses in order:
  - (i) needs every row-1 condition but speed: no;
  - (ii) needs (k2)'s summed R1 b − c above 0: it is 126 − 260 − 38 = −172, so no;
  - (iii) applies: **(b) stays the base, and the slot key (entity × attribute atoms) is the next copy-key unit.**

"The base" is the base trained as served at `59f7770ae`. Nothing is closed: row 2's close needs worse at all three seeds.

## Plan step (the claim check)

Order step 4 asks for one model trained, exported, served and scored end to end, with tok/s. This unit trained it (8,192 steps, three seeds), exported it raw, served it in integers on the laptop with first tokens and whole replies matching the trained model on DEV, scored it on p-w and p-v, and measured its laptop serve speed (717–796 tok/s, prefill included). R6's decode-only speed is unread. The model holds no useful conversation (R7), so this is the end-to-end path working, not a capability.

## The readout question

Casey's answer on #2028 ([6614481479](https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6614481479)), verbatim: "ok well based on the outcome I do approve you to do that other piece if you still need". All six exports pass I-3, so the readout is not needed for this line and is not built. If a later unit finds a parity gap, the approval stands for it.

## Cost

Three Vast pods: seed 1 `vast-58977590` ($0.563/h, 3.13 h), seed 2 `vast-58978404` ($0.563/h, 3.18 h), seed 3 `vast-58978093` ($0.975/h, 5.25 h). Total about **$8.67**, inside the sheet's $11–20 estimate and the $24 watcher budget. All three are down.

## What changed

No code. The new sheet, this REPORT, one notebook row in `docs/research/notebook/float-training.md`, and the item's line.

## Evidence

- HF store `results/claude/one-model-end-to-end-full-unsnapped-2026-11-03/s1`, `s2` and `s3` (256 files each). Each was read back: `SHA256SUMS.files` and (k2)'s `samples-memory.json` match the local copies, and each pod's 252-file seal checks.
- Local (not committed): `local/claude/full-266/serve-out/` (exports, parity files, transcripts, `decode-final`, `decode-final2`), `serve-out.log`, `grade-p-w.txt`, `grade-p-v.txt`, and the scripts `job-266.sh`, `watch-266.sh`, `serve-266.sh`, `timing-266.sh`.
