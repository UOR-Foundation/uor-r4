# REPORT: one-model-end-to-end, claude-261 (the copy read's two-token key, trained as served)

Lab: claude. [ORDER](./ORDER.md). Sheet: [`one-model-end-to-end-copy-key-2026-11-02.md`](../../../../harness/prep/one-model-end-to-end-copy-key-2026-11-02.md). PR #3579, item #3407. Head of every run: `36ca53c81195969f9487747163cc13c50d145efa`.

**Outputs:** the HF store under `results/claude/one-model-end-to-end-copy-key-2026-11-02/`. Each folder is a tar with `SHA256SUMS`, read back and matched before its pod went down:
- `s1/s1.tar`: seed 1's shared pod, with deepseek-5's four runs (`copy-key/ds5/`, `copy-key-ds5eval/`) and seed 1's (k2). SHA-256 `738457b26af18e9176bd0e3ece76f0502d4505cdf0a3f249381fc0ccf574ad1d`.
- `s2/s2.tar`: seed 2's shared pod, with seed 2's (b). SHA-256 `99f19197adc0b0af37cc6f1bae1507807dfd1fd2134fe162cd797466005ba8ae`.
- `s2-k2/s2-k2.tar`: seed 2's (k2), trained again alone. SHA-256 `2b57b5a9473afecef8847426ee5a4b00493500ddc1071834c79cb120543b3999`.
- `s3/s3.tar`: seed 3's shared pod, with both arms. SHA-256 `5d4bcd44fffbf0e09bf633d6707952c0db76dc9c5af6d4ec90246258d0d22d8f`.
- `s1-b/s1-b.tar`: seed 1's (b), trained again alone. SHA-256 `25a45008f8cf2ed876686923855827927343c9c0153762465df0bee615ba74df`.
- `served/served.tar`: the laptop half. Seed 1's two exports, I-3's parity files, the transcripts, both timings, claude-259's control, and `serve.sh`, `timing.sh` and `grade.sh`. SHA-256 `2b9e9a5b25e6b80befe9c990aafc04795a2f0bfe83e2c4ba6e6bdc4f04889459`, read back and matched.

**Reviews before the run.** The sheet's FIT (round 4, "TEST FITNESS: FIT") and its pre-run review (D67, round 2, PASS) are recorded in the sheet's header and in the launch note on #3407 (comment 6592199906).

**In one paragraph.** The copy read's two-token key (k2) was trained against the one-token key (b) at three seeds, under main's `--train-as-served 1`.
- **It is not a verdict on the key.** On R1, (k2) is better at seeds 1 and 3 and worse at seed 2, so the decision table lands on row 4(ii): (k2) is the candidate, with three more seeds next.
- **The run's main finding is about the training form, not the key.** Under #3565's served form:
  - every arm learned far less per step than claude-259's runs (A);
  - the exported integer model does not reproduce what the pod scored: 297–305 of 310 first tokens differ, against a bar of 8 (B).
- **What else changed.** Every seed was cut to a quarter of its steps by two arms sharing a card. claude-262 found the cause of A and B in the stand-in quantizer and is repairing it in its own unit. Casey's two decisions for the next units are "Arms in turn" and "Yes, scorer next".

## Tokens per second

- **Served decode on the M1, not counted:** 1,357–1,464 tok/s for both seed-1 exports, decoding up to the 192-token context, in the first timing (load 4.7 from other sessions). R6 is an instrument fault on two counts: I-3 failed (B), and the timing ran with other sessions loading the laptop. See "Served" below.
- **Training, context tokens a second on one RTX 5090:**
  - Two arms sharing the card: 2,002–2,128 at I-2's smoke, against claude-259's (b) 10,730–12,618 sharing the same way.
  - One arm alone: 4,324 ((k2), seed 2) and 4,459 ((b), seed 1).
- **Scoring one arm** (`--score-rows`, 1,536 samples, 64 reply tokens, memory on and off), with one CPU core at 100%:
  - 7,322–7,591 s alone on the card;
  - 13,108 and 13,867 s for seed 3's two arms, scoring side by side.

## What did not work, first

**A. Training as served, under main's `--train-as-served 1`, learns far less per step than it did before #3565.** This is the run's largest finding, and it is not about the key.
- At step 2,048, DEV nearest is 921–2,314 of 14,287 here, across all six arms. claude-259's runs, trained under claude-258's form before #3565, were at 11,357 and above at the same step ([claude-259 REPORT](../claude-259/REPORT.md), "Training and DEV (R4)").
- deepseek-5's four runs at this head isolate the cause to the served form. At 256 steps, served reaches DEV nearest 23 and 20, against plain's 225 and 229 (seeds 1 and 2). Over the 256 steps, served's train loss fell 0.00023 and 0.00016, against plain's 0.00103 and 0.00084. Those runs are DeepSeek's to read by their own sheet's bar; the readings are on #3407 (comment 6595148437).
- Served training is also slower: 355 s against 51–52 s for 256 steps (6.8–7.0×).
- **The cause, found in source by claude-262** (#3587, its own unit; its matched GPU rerun is still to come). `activation_quantize16`, the float stand-in for the integer step's `stack_quantize16`, left out the step's exponent (`PRODUCT_EXP = −32`). So the served forward rounded every recurrence and MLP activation below 2^15 to a whole number. Its 49-pass loop over thresholds is also replaced, with six exact halvings.
- **Classed instrument for this question** (CHARTER §4.2, L-007). Both arms train under the same form, so (k2) against (b) is still a paired comparison. But both sit far below that form's earlier level, and both are still rising. The copy-key question cannot be read at this level. The fault is a finding about #3565's served forward, which is now its own unit (the ORDER's last owner decision).

**B. The exported model does not serve what the pod scored (I-3 failed).** This is the same fault as A, seen from the laptop.
- **The measurement.** Seed 1's two exports went through `fbs-toks --replies` on DEV's 310 prompts. Their first tokens differ from the arm's own float `samples-memory.json` at 305 of 310 for (b) and 297 for (k2), against the sheet's bar of 8 (2.9%). claude-259's runs, under claude-258's form, differed at 0–1 of 310.
- **Where the fault is: the served float forward that trained and scored these runs.** Scoring took the served forward from each run's `config.json` (`train_as_served: true`, read by `run_train_as_served`).
  - Between claude-259's run head (`f596a5e631b0f3bc0835a543aaa2e97cc82485e1`) and this unit's (`36ca53c81`), the only change to a copy-key-1 float forward is #3563 and #3565's served stand-ins: `served_activation_quantize16`, called four times in the forward. At claude-259's head, `activation_quantize16` does not exist.
  - The other float-train commits in that range leave copy key 1's forward unchanged. They are the `--served-forward` override (`b44402e0c`), a test (`e1de1ff2c`), the copy key (`987cc3fe1`, where key 1's path is untouched), and the dtype repair to the stand-in itself (`36ca53c81`).
  - So claude-259, also run with `--train-as-served 1`, had no activation stand-in. That is why its float and integer agreed.
  - claude-262 found one concrete defect in that stand-in (A). Whether it is the only one is for its matched rerun to show.
- **Control.** Today's `fbs-toks`, built after main was merged in, reproduces claude-259's integer replies for its b-s1 export byte for byte (SHA-256 `52d3bc00ac3f5828a55eab860441de79c615b27c2b9a4c861168b7ba1d4e0e0e`). `fbs-export`'s binary hash is unchanged. So the serving tools are cleared.
- **Classed instrument** (CHARTER §4.2). By the sheet, R6 and R7 are instrument faults, and R1–R5 stand.
- **The decision.** The sheet says to repair them "in this unit before R6 and R7 are read". The repair needs new training at claude-262's repaired head, so it moves to claude-262's matched rerun and to the next copy-key unit, with I-3 before R6. This was decided on claude's own decision review: round 1 failed on the evidence for the cause, and the evidence above is round 2's answer. Both rounds are logged on #3407 as an owner log.
- **A check not run.** Scoring (b)-s1 on the GPU with `--served-forward 0` would show whether the export equals the plain forward on the same weights. Either outcome leads to the same next step (claude-262's fix, then I-3 at its head), so it was not spent.
- **What R1 to R5 are.** They are readings through the served float forward, which the export does not compute. They compare the two keys under that forward only, and are not readings of the served model.
- **What it means.** No served-chat or served-speed number from a model trained under #3565's form counts until I-3 passes again.

**C. The seeds disagree.**
- At seed 2, (k2) is worse than (b): R1 on p-w b − c = −30 (floor 20.4), and R5 on p-v −20 (floor 12.65).
- At seed 3, (k2) is better: R1 +46 (floor 20.2), and R5 +15 (floor 11.83).
- At seed 1, (k2) is better on R1, +28 (floor 18.11). R5 is level, at +1 (floor 11.14).
- So the run lands on row 4(ii), not a verdict on the key: (k2) is the candidate (below), read under a training form that claude-262 has since found broken (A).

**D. Every seed was cut to 2,048 of 8,192 steps.**
- I-2 measured 2,002–2,128 context tok/s with two arms sharing the card. That fits 2,048 steps in the sheet's 90 minutes, so both arms were cut, at every seed, as the sheet's stop rule says.
- Alone, an arm runs at 4,324 tok/s, which fits 6,144 steps in 90 minutes. Sharing caused the cut.

**E. Two arms ran out of GPU memory while sharing the card:** seed 2's (k2) at 01:12Z and seed 1's (b) at 01:52Z.
- Together, the two arms' memory climbed about 1,000–1,300 MiB a minute, against the card's 32,607 MiB. Alone, an arm held 5,040–7,216 MiB.
- Classed instrument. Each trained again alone at its seed, at 2,048 steps, under the rule decided in the ORDER ("Found in the main run"). Neither failed run is scored.
- Casey's decision for the next units: "Arms in turn".

**F. #3563 and #3565's served quantizer had never run on CUDA, and it refused there.** I-0 on seed 1's first pod (head `af7154c00`) failed 2 of 291 tests: `activation_quantize16` multiplied an F32 scale by untyped F64 literals. It is repaired at `36ca53c81` with the values unchanged, and 291 of 291 passed at I-0 on every pod. No training had started.

**G. deepseek-5's sheet's evaluation steps refuse as written.** They create `EVAL` and copy `config.json` into it before `fbs-snapshot --round-as-exported`. But `fbs-snapshot` copies the config itself and refuses an output folder that exists, so the first pass refused all four. `ds5/DONE` was written anyway, because that pass did not check the round's exit. The evaluation ran again, with `fbs-snapshot` making `EVAL`, and all four exited 0. The sheet needs that one change.

**H. The scorer is the run's longest step.** It decodes one sample at a time, with the forward over the whole window for every reply token. That is 2 hours per arm, against 26–55 minutes of training. Casey's decision: "Yes, scorer next".

**I. At this level, R1 mostly reads openings.** No reply is exact on either panel at seed 2. The prefix sums exceed the openings counts by only 13 ((k2)) and 36 ((b)) tokens over 1,202 prompts.

**J. `fbs-toks --copy-codes` (claude-237) stays copy-key-1 only.** Its bin is outside this unit's scope (the ORDER's decisions), and this run does not use it. Since deepseek-4 (#3584, now merged into this branch), it reads the artifact's header and refuses any copy key other than 1, so it cannot misread a (k2) artifact. A check that understands ordered pairs is still to be written.

## What worked

- **The key, end to end** (`987cc3fe1`). `--hamming-copy-key 1|2` trains the ordered two-token key. It is saved in `hamming_read.safetensors` and `config.json`, carried by `FbsHammingSnapshot` and `StackHamming`, and served by `CopyHeads`. The tests:
  - `copy_key_two_tells_two_facts_after_the_same_token_apart`: in the float read and in the integer read;
  - `the_copy_key_is_saved_only_when_two`, `a_config_writes_the_copy_key_only_when_two` and `the_copy_key_is_recorded_only_when_two`: a copy-key-1 artifact is as before;
  - `a_copy_key_is_one_or_two_tokens_on_halves_of_whole_lanes`: other keys, and halves off whole lanes, are refused.
- **Every arm passed I-1's 256-step smoke** at all three seeds: train, score and snapshot.
- **Every pod's outputs were sealed, stored and read back** before it went down. The SHA-256 values above match.

## What changed

- The copy key (`987cc3fe1`) and the quantizer's dtype repair (`36ca53c81`).
- The sheet, and its stop rule's resource-fault case, added on claude's decision review after the first OOM.
- [claude-259's REPORT](../claude-259/REPORT.md) now names deepseek-3's PR as #3563 (`dab00a9f8`).

## Measurements

### Training and DEV (R4) at step 2,048

| Arm | Seed | Trained | Train wall (s) | Context tok/s | DEV nearest (1,024 → 2,048) | Openings | Copies | Rest | Top-1 |
|---|---|---|---|---|---|---|---|---|---|
| k2 | 1 | shared | 3,284 | 2,029 | 422 → 1,724 | 60 | 56 | 1,612 | 68 |
| b | 1 | alone | 1,521 | 4,459 | 716 → 1,487 | 32 | 89 | 1,372 | 127 |
| b | 2 | shared, then alone | 2,371 | 2,857 | 873 → 2,314 | 70 | 126 | 2,119 | 132 |
| k2 | 2 | alone | 1,565 | 4,324 | 250 → 921 | 39 | 45 | 844 | 116 |
| b | 3 | shared | 3,182 | 2,138 | 653 → 1,345 | 30 | 39 | 1,278 | 47 |
| k2 | 3 | shared | 3,210 | 2,077 | 440 → 1,613 | 71 | 103 | 1,445 | 111 |

- **Every arm is still rising.** The sheet's bar is more than 143 above the eval at half the steps, and every arm here rose by 671–1,441.
- **The strata do not order the arms the same way at every seed.** At seed 1, (k2) has more nearest hits and openings, and fewer copies. At seed 2, (b) is ahead in every stratum. At seed 3, (k2) is ahead in every stratum, copies included (103 against 39).
- **Seed 2's (b) shared the card until (k2) failed at 01:12Z, then trained alone.** Its tok/s is the mean over both stretches.
- Context tok/s is `train_context_tokens_per_second` at step 2,048.

### R1, R2 and R3 on p-w (1,202 prompts), (k2) paired against (b)

| Seed | Prefix sum (k2 / b) | Exact (k2 / b) | Openings (k2 / b) | R1 b | c | b − c | Floor | Verdict |
|---|---|---|---|---|---|---|---|---|
| 1 | 66 / 31 | 0 / 0 | 57 / 30 | 55 | 27 | +28 | 18.11 | better |
| 2 | 51 / 104 | 0 / 0 | 38 / 68 | 37 | 67 | −30 | 20.4 | worse |
| 3 | 132 / 35 | 0 / 0 | 76 / 32 | 74 | 28 | +46 | 20.2 | better |

### R5 on p-v (393 prompts)

| Seed | b | c | b − c | Floor | Verdict |
|---|---|---|---|---|---|
| 1 | 16 | 15 | +1 | 11.14 | level |
| 2 | 10 | 30 | −20 | 12.65 | worse |
| 3 | 25 | 10 | +15 | 11.83 | better |

### The decision table's reading

The rows are read in order, and the first that matches decides:
- **Row 1 does not match.** It needs (k2) better at all three seeds, and seed 2 is worse.
- **Rows 2 and 3 do not match.** They need no seed cut and no arm still rising; every seed is cut, and every arm is rising.
- **Row 4 matches** ("better at some seeds only", "a seed cut", "an arm still rising"). Its first applicable branch:
  - (i) does not apply: (k2) did not meet row 1's other conditions.
  - **(ii) applies.** (k2)'s R1 b − c summed over its measured seeds is +28 − 30 + 46 = **+44**, above 0. So (k2) is the candidate, and the next unit runs three more seeds.
- R6 being unread (B) changes none of this: row 4 names "R6's speed condition failing or unread", and (i) fails on seed 2 before speed is reached.

The table also says what the three seeds must not be read as. The candidate stands on readings through the served float forward, which claude-262 found broken (A) and which the export does not compute (B). Every arm was at 2,048 of 8,192 steps and still rising. So (k2) is a candidate under that forward only, and no verdict on the key comes from this unit. So the three more seeds train at claude-262's repaired head, one arm at a time (Casey's "Arms in turn"), with I-2 projecting each arm alone.

### Served (R6, R7), I-3 and I-5

`serve.sh`, then `timing.sh` twice, on the laptop, with seed 1's two checkpoints from `s1-b.tar` and `s1.tar`.

| Arm | Export SHA-256 | I-5: `copy_key` in the header | I-3: first tokens differ (bar 8) | Whole replies differ | R7: conv-a names Maya |
|---|---|---|---|---|---|
| b | `d19e687496d381ac7d23f4ffed56ae1273e140bf4c6be47c158f11aabf896eec` | none (wants none) | 305 of 310 | 310 | no |
| k2 | `75b44c73daa0bb391adab46505d67751ac863b21ffafe484aa13349f8eb44a9a` | 2 (wants 2) | 297 of 310 | 310 | no |

- **I-5 passes.** Every export, the parity run and the transcripts exited 0, and no prompt came out of order.
- **I-3 fails at both arms** (B). So R6 and R7 below are instrument faults, recorded and not read.
- **A departure.** `serve.sh` writes the transcripts in the same pass as I-3, so they exist from before the fault was known. They are kept in `served.tar` and not read. The sheet puts the repair before R6 and R7, and the timing ran after I-3 had failed.
- **R7, the transcripts, recorded only.** Both exports' conv-a replies are not words, and neither names Maya.

**R6, decode tok/s.** The prompt is each conversation's first turn. The run decodes 175–177 tokens to the 192-token context, over five interleaved rounds. The spread is (max − min) / median of the long runs.

| Conversation | (b), timing 1 | (k2), timing 1 | (b), timing 2 | (k2), timing 2 |
|---|---|---|---|---|
| conv-a | 1,366 (spread 2.6%) | 1,378 (4.2%) | 956 (37.5%) | 1,269 (106.2%) |
| conv-b | 1,357 (16.0%) | 1,389 (1.5%) | 1,155 (98.5%) | 741 (117.4%) |
| conv-c | 1,394 (2.4%) | 1,464 (5.0%) | 1,386 (90.9%) | 1,254 (13.2%) |

- **The timing is an instrument fault, recorded only.** The protocol needs "no build or other model running". The laptop's load average was 4.70 → 4.11 during timing 1 and 6.18 → 11.20 during timing 2, from other sessions; no build of this unit was running.
- Timing 1 spread past 5% at one cell (conv-b, (b), 16.0%), so the protocol ran it again. Timing 2 spread past 5% at all six cells (13.2–117.4%).
- In timing 1, (k2) decodes at 1.01–1.05× (b)'s rate on each conversation. That is reported, not read.

### Cost

Five Vast RTX 5090 pods at the offer price of $0.563 an hour, from each pod's up time to its `down`:

| Pod | What it ran | Hours | Cost |
|---|---|---|---|
| `vast-58894902` | seed 1: I-0, deepseek-5's four runs, (k2), and (b) until it ran out of memory | 3.75 | $2.11 |
| `vast-58895486` | seed 2: (b), and (k2) until it ran out of memory | 3.12 | $1.76 |
| `vast-58896165` | seed 3: both arms | 5.12 | $2.88 |
| `vast-58901106` | seed 2's (k2), alone | 2.86 | $1.61 |
| `vast-58912154` | seed 1's (b), alone | 2.61 | $1.47 |
| **Total** | | **17.46** | **$9.83** |

## Also found during this unit (reported, not acted on in scope)

- `job.sh`'s I-2 line says "both arms CUT" when one arm runs alone. It is cosmetic.

## Checks run at the head

- **I-0, the whole CUDA library suite** (`cargo test --release --locked -p uor-r4-float-train --features cuda --lib`): exit 0 on every pod at `36ca53c81`. The binaries were built there too: `train-in-space` SHA-256 `9a6804db798d33e8d27d09e11b141c83ed1d030a15b346797341b9d1bc87d33c`, `fbs-snapshot` `9eca078519cb861cb1fed86307fe77eac1ed018de70a665dd0feace5cbb185b2`.
- **The laptop's serving binaries** were rebuilt after origin/main (`de9cfc84a`) was merged into this branch (`29b7b6991`), with `cargo build --release --locked -p uor-r4-integer --features serving-wasm`. Seed 1's serving and timing ran on these:
  - `fbs-export` `b2d9aff990e78afa40e0a08d4952ba6bbceeff27a32fd86ba012d725c91c3aa8` (unchanged by the merge);
  - `fbs-toks` `fa0d43c6f62dfba5aada1c834c96310488aa5a4a662b4af3179cb9959f6e839a` (deepseek-4's key guard);
  - `uor-chat` `e2cb9ea5f07dd1078a1546a261754d78d26a14cf8c024f3c09c14d4a84e91f9a` (deepseek-3's protocol-2 default; `serve.sh` passes `--protocol 2` anyway).
- **The laptop suites at `29b7b6991`** (`--no-fail-fast`):
  - `cargo test --locked -p uor-r4-float-train`: exit 0. 212 library tests, the bins' 14, and the two integration files' 14, all passed.
  - `cargo test --locked -p uor-r4-integer --features serving-wasm`: the library's 445 passed (1 ignored), and the bins' 14 passed. 7 integration files failed, every failure for a missing input file: the quaternion or Householder bundle, or `tokenizer.json`. deepseek-3 reported the first of these files (`adversarial_challenger_m1_token_boundary`) failing on main for the same missing inputs; its run stopped there.
  - `m7_parity_library_cli_browser` also failed in that run, because `wasm-pack` found Homebrew's `cargo` first on PATH. Rerun with `~/.cargo/bin` first on PATH, it passed 2 of 2.

## Skills used

None.

## Still open

- **The served-training fault (A, B):** in its own session (claude-262, #3587), by Casey's go-ahead. It was told about B on 3 Nov, so that its matched rerun can check I-3 too.
- **The batched scorer (H):** the next unit in this session.
- **The served quantizer's 7× time (A):** claude-262 replaces the 49-pass loop in the same function; DeepSeek registered the speed repair as `served-rounding-cost` (#3586).
- **The copy-key candidate:** three more seeds at claude-262's repaired head, one arm at a time, with I-3 passing before R6 or R7 is read (row 4(ii)).

The item stays open: its own bars (end to end on unseen prompts, with tokens per second) are not met. References #3407.

## Plain summary

We trained the model twice at three seeds: once with its copy read keyed on one earlier token, once on two in order. The two-token key did better at two seeds and worse at one, so it gets three more runs before it can replace the one-token key. The bigger finding is that the training form now on main, which is meant to train the model exactly as the laptop serves it, does not: the models learned about a fifth as much as the previous form's by the same step, and the laptop's integer copy of each answers differently from the trained model on 297–305 of 310 prompts, in noise. Another session (claude-262) found the cause, one missing exponent in the float stand-in for the integer rounding, and is fixing it. The next runs wait for that fix, train one arm at a time so no card runs out of memory, and the next unit makes scoring, the slowest step, decode many prompts at once. The five pods cost $9.83.
