# Step off a reply's own cycle (claude-217, 30 Oct 2026)

Lab: claude. Item: [one-model-end-to-end](../../../roadmap/items/one-model-end-to-end.md). Order: [claude-217](../../../loop/tasks/one-model-end-to-end/claude-217/ORDER.md). Sheet: [one-model-end-to-end-cycle-step-2026-10-30](../../../harness/prep/one-model-end-to-end-cycle-step-2026-10-30.md): FIT in round 4 (rounds 1–3 were each NOT FIT on one point, repaired), pre-run GO. By Casey's direction of 30 Oct.

## The run

- **The model:** (r-drop)'s sealed step-16,384 checkpoints at seeds 1, 2 and 3. There is no training.
- **The arms,** each with the memory on, 256 replies per split and seed, on DEV and the new panel p-s (seed 2091):
  - greedy;
  - the cycle step (`CycleStep { k: 16, n: 4 }`: the nearest token outside a repeated 4-gram of the reply's own, Casey's rule);
  - the echo step (the same on bigrams);
  - the decode-only cut;
  - the cycle step under the cut.
- **The hardware:** one RTX 5090 (`vast-58411840`), built at `dee65f6f8`. The binary's sha256 is `3f4052582e47597465d82f8e89ebee45a6e7e53a6a92db442309617e1d42c02f`.
- **The instrument checks:**
  - the CUDA tests pass, among them `the_cycle_step_follows_greedy_until_it_steps_off_a_cycle_on_a_model`;
  - every input matches its sha256;
  - claude-211's 15 registered files reproduce byte for byte;
  - every cycle- and echo-step reply carries its marks;
  - every step reply follows its greedy path until its first step off (`follows_greedy`, in the grade).
- **Sealed** at `results/claude/one-model-end-to-end-cycle-step-2026-10-30/run-dee65f6f/`: 103 files, whose `SHA256SUMS` has sha256 `55d13779882299fb19a11b71b3611c95f5d4966d968ffa5d4d019578b20bfc00`. It was read back OK. About $0.90.
- **Graded** on the laptop by `train-in-space --cycle-grade 1` at `dee65f6f8`: [cycle-grade.json](cycle-grade.json). The [fixed reading](fixed-reading-s1.md) is seed 1's first 20 p-s replies.

## Tokens per second

- **Every arm decodes at 51–54 tok/s per process,** with the three seeds sharing the GPU.
- **The cycle and echo steps cost nothing measurable:** 53.7 against greedy's 52.9 at seed 1. Their check is an exact n-gram lookup over the reply.
- **This is not the served speed.**

## The registered outcome

**Every decision reads "underpowered (content)", so greedy stays** (`ships: greedy`).
- **Why:** at seed 1, greedy loops on 59 prompts. Only 11 of their references are verbatim TRAIN replies (`p` = 0.19), below the power condition. The rule needs all three seeds.
- **What this is:** an instrument outcome, not a negative.

## p-s, the deciding panel

Each cell gives seeds 1, 2 and 3.

| Arm | Looping replies | Complete | Matched verbatim (218, 199, 207 prompts) | Mean length | Steps off / replies stepped |
|---|---|---|---|---|---|
| greedy | 59, 63, 86 | 218, 199, 207 | 0.321, 0.442, 0.377 | 20.7, 22.6, 24.5 | — |
| cycle step | 0, 0, 0 | 256, 256, 256 | 0.358, 0.442, 0.377 | 14.2, 12.6, 15.5 | 124/59, 102/63, 171/86 |
| echo step | 0, 0, 0 | 255, 256, 256 | 0.321, 0.442, 0.377 | 13.8, 11.5, 14.3 | 152/61, 106/70, 229/98 |
| decode-only cut | 32, 45, 49 | 236, 245, 246 | 0.317, 0.437, 0.372 | 17.2, 15.7, 16.0 | — |
| cycle step and cut | 0, 0, 0 | 256, 256, 256 | 0.317, 0.437, 0.372 | 13.7, 13.4, 13.9 | 61/32, 73/45, 64/49 |

**On greedy's looping prompts** (59, 63 and 86), against the references:

| | Seed 1 | Seed 2 | Seed 3 |
|---|---|---|---|
| References verbatim in TRAIN | 11 | 40 | 52 |
| Cycle step verbatim | 8 | 0 | 0 |
| Echo step, cut, cut-and-cycle verbatim | 0 | 0 | 0 |
| Any arm equal to its reference | 0–2 | 0 | 0 |

No cycle step was exhausted. Every step reply follows greedy until its first step off.

## What it shows

- **The cycle step ends every loop, exactly where greedy loops, at no measured cost.**
  - It changed 59, 63 and 86 replies: exactly greedy's looping prompts. Every reply completes.
  - The matched content is greedy's, since away from a loop it writes greedy's reply.
- **It does not recover what greedy lost there,** and nor does any other arm.
  - At the two seeds the reading could decide, every arm writes 0 verbatim TRAIN replies on greedy's looping prompts, against 40 and 52 references. That decides nothing alone, since the rule needs all three seeds.
  - The fixed reading shows the replies are already wrong before the loop: "Here are three fruitsat about about it it it…" for "Got it, I'll remember them.". The rules only change how a wrong reply ends ("it it.", "…iteded.").
- **claude-216's decode-only "improves" does not hold up on the looping prompts.** There the cut writes 0 verbatim replies at seeds 2 and 3. Its earlier reading came from matched prompts, which cannot see the looping ones (FIT round 3).
- **The loop is a symptom. The failure is the reply's opening on a new phrasing.**
  - On held-out phrasing, greedy's reply equals its reference in 1–5 of 256, against 38–40 on DEV.
  - Teacher-forced, on p-t (claude-216's sealed rows), the reply's first token is right 12–19% of the time against 81% for later tokens. DEV openings are about 49%.
  - So the model continues a reply well, but on a new phrasing it rarely starts the right one. When the start is wrong, it writes a stock reply that does not fit, or ends in a loop.

## What it unlocks

- **No decode change.** Greedy stays. The cycle step is a cheap, exact backstop that ends every loop, and it is ready if a later unit has content to protect.
- **A rollout hinge (claude-218 as first planned) would not fix this.** It would train the model toward the cycle step's choice, which on these prompts is salad.
- **The next work is the reply's opening on held-out phrasing:** routing a new phrasing to the right reply.
