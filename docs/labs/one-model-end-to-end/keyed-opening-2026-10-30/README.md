# The keyed opening in free generation (one-model-end-to-end, claude-219, 30 Oct 2026)

Lab: claude. Item: [one-model-end-to-end](../../../roadmap/items/one-model-end-to-end.md). Sheet: [one-model-end-to-end-keyed-opening-2026-10-30](../../../harness/prep/one-model-end-to-end-keyed-opening-2026-10-30.md), FIT in round 2, pre-run GO. Report: [claude-219](../../../loop/tasks/one-model-end-to-end/claude-219/REPORT.md).

**The registered outcome: row 2, the point key ships.** Served as the reply's first token wherever the memory's exact key misses, claude-218's trunk point key carries the reply in free generation at all three seeds on the fresh panel p-v. The sign and hsign keys are too thin to read at this panel's size.

## The run

- **The model:** (r-drop)'s sealed step-16,384 checkpoints at seeds 1–3, the ones claude-217 and claude-218 scored.
- **The code:** `train-in-space --score-rows … --opening-keys 1 --keyed-decode sign=τ,hsign=0,point=inf`, built at `da4a6212f` (binary sha256 `951e2694…98bf`).
- **The compute:** one RTX 5090 (`vast-58445674`, $0.563/h), about 2 h 15 min, about $1.30.
- **The panels:**
  - **p-v decides:** seed 2111, tokens `3488eef5…6f0`; [census](../census-p-v-2026-10-30.json). 393 prompts a seed, every eligible one: each conversation's first opening.
  - **DEV** is the not-worse check: 310 prompts.
- **τ:** claude-218's committed levels. Sign was 0.0078125, 0.01953125 and 0.0078125 at seeds 1–3; hsign 0; point +∞.
- **Sealed** at `results/claude/one-model-end-to-end-keyed-opening-2026-10-30/run-da4a6212/`: 147 files, `SHA256SUMS` sha256 `730026ee5380cbcb7cb3eea01eb4529af277961028327d42209124bd4079b04d`, read back OK.
- **The grade:** [grade-p-v.json](grade-p-v.json), by `--keyed-grade 1 --runs …/out --panel p-v=…`. It raised no instrument refusal: every keyed token is the registered τ's, and every keyed reply is the one its opening fixes.
- **The power check:** [power-p-u.json](power-p-u.json), the same grade on claude-218's p-u replies, run before the pod.

## Tokens per second

- **Decode:** 55–57 tok/s per process, with three seeds sharing the GPU. Greedy runs at 55.3–56.4 and keyed-point at 54.9–56.2 on p-v. The keyed arms decode at greedy's speed, since the key's name is taken once, at the opening. This is not the served speed.
- **Each key's search** takes about 6.8 ms an opening (9.3–9.8 s for p-v's 1,402) against TRAIN's 212,968 opening points (218 MB as f32). That is the two-pass instrument's time, an upper bound on a served key's.

## The forced opening (row 1 does not hold)

A right first token, the reference's, against greedy (memory on), on p-v's 393 prompts:

| Seed | Exact (b, c) | Templates (b, c) |
|---|---|---|
| 1 | (66, 0) | (71, 0) |
| 2 | (63, 0) | (66, 0) |
| 3 | (70, 0) | (73, 0) |

## Each keyed arm against greedy on p-v

b counts the arm right and greedy wrong, c the reverse. For loops, b counts the arm's reply looping where greedy's does not. "Keyed" is how many replies the key opened, and "right" how many of those it opened with the reference's first token. The predicted exact pairs (from greedy and forced alone) equal the observed ones in every cell.

| Key | Seed | Keyed | Right | Exact | Templates | Band | Loops | DEV templates | Carries |
|---|---|---|---|---|---|---|---|---|---|
| sign | 1 | 158 | 52 | (3, 0) | (6, 0) | 4.9 | (22, 21) | (1, 0) | no: thin |
| sign | 2 | 154 | 35 | (1, 1) | (1, 1) | 2.8 | (36, 12) | (1, 0) | no: thin; loops up |
| sign | 3 | 165 | 53 | (0, 0) | (2, 0) | 2.8 | (17, 26) | (0, 1) | no: thin |
| hsign | 1 | 300 | 55 | (1, 2) | (4, 2) | 4.9 | (36, 40) | (0, 0) | no: thin |
| hsign | 2 | 193 | 44 | (3, 0) | (3, 0) | 3.5 | (37, 38) | (0, 0) | no: thin |
| hsign | 3 | 191 | 41 | (6, 1) | (6, 1) | 5.3 | (13, 64) | (1, 1) | no: thin |
| **point** | 1 | 385 | 106 | (12, 3) | **(16, 3)** | 8.7 | (35, 60) | (0, 0) | **yes** |
| **point** | 2 | 385 | 105 | (9, 1) | **(9, 1)** | 6.3 | (47, 85) | (0, 1) | **yes** |
| **point** | 3 | 385 | 119 | (14, 2) | **(14, 2)** | 8.0 | (22, 113) | (0, 1) | **yes** |

On DEV, the keys open only 12–19 of 310 replies, since the exact key fires on most DEV openings. Every DEV template and exact pair is within its band. hsign's DEV loops at seed 1, (7, 0), exceed theirs (5.3), which is reported only and decides nothing.

## What the point key's replies are

- **The gain:** net +13, +8 and +12 template matches a seed, and +9, +8 and +12 exact replies. That is 12–18% of the forced arm's template lift: the key names the right first token on 105–119 of the 385 openings it gives, so 27–31%.
- **Fewer loops.** Its replies loop on 35, 47 and 22 prompts where greedy's do not, against 60, 85 and 113 the other way.
  - **Part of that is short replies.** 49–58 of its openings a seed are " Noted." followed by the end of turn, a common TRAIN opening, and 332–348 of its replies end within 64 tokens against greedy's 291–316.
  - So the loop count is not read as a quality gain. Bar (c) asked only that loops not rise.
- **The prediction, registered before the run,** was exact nets of +10/+13/+12 at 256 prompts (pairs (11, 1), (13, 0), (12, 0)), from claude-218's p-u files. Observed on p-v at 393: +9/+8/+12, with b + c of 15, 10 and 16. Its direction held (point powered, sign and hsign thin); its size fell short of the 1.5 times scale the sheet expected.
