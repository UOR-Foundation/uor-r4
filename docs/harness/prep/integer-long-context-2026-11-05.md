# Prep sheet: three rules past the trained window, by held-out loss and reply agreement (integer-long-context, claude-287, 5 Nov 2026)

Lab: claude. Item: integer-long-context (#3471). Order: [claude-287](../../loop/tasks/integer-long-context/claude-287/ORDER.md), Casey's plan P4, unit 2 of 3.

TEST FITNESS: PENDING. A fresh non-author FIT is owed before any pod is rented.

**Why.** Casey's target (2 Nov) is a served 4,096-token conversation, with tok/s and RSS measured to 8,192. Every checkpoint is trained at one window W: the served a7r at 192, P2b's (L) at 2,048. The integer session must apply one rule past W, and none has a reading:
- the float scorer refused any context past a checkpoint's window ([cancel probe 1](../../loop/tasks/integer-long-context/claude-287/ORDER.md#cancel-probe-run-first));
- claude-286 found that carry needs no new forward, since no weight is sized by the window, and that its integer speed is measured (1,851 tok/s at 2,112);
- slide's and blocks' integer speeds are not measured.

This unit wires `--score-rows --past-window` and measures the three rules on two trained checkpoints. claude-288 serves the winner.

## Question

1. Past W, does **carry** hold the float model's held-out loss and greedy reply agreement within slide's band? Carry is one window over the whole conversation, with the trunk's state and the read's keys running on past W.
2. Does carry or **slide** keep anything from before W that **blocks** drops? Slide gives every target the last W tokens. Blocks cuts back-to-back windows of W and carries nothing over.
3. What does each rule cost: seconds a pass, and decode tok/s at W and at C?

## Base artifact

- **The pin:** this unit's head at run time, which carries the scorer's past-window code (Code, below). Without `--past-window`, every output and every refusal is the base's, and a test checks that.
- **The checkpoints**, each checked against its sealed run's sum file, pinned here by that file's SHA-256:

  | checkpoint | step | W | run objective | sum file | its SHA-256 |
  |---|---|---|---|---|---|
  | a7r seed 1 | `training-460fe46e/s1/r-drop-s1/models/step-016384` | 192 | `cell` | `s1/SHA256SUMS` | `aff46b2af26fc3da2530364b65d28bd42d8033af4f2d418c64e0d9d0dc40e9f6` |
  | a7r seed 2 | `training-460fe46e/s2/r-drop-s2/models/step-016384` | 192 | `cell` | `s2/SHA256SUMS` | `0950eba2fbd260488db747041bf8070fda7689dd67950c3d1dccc4e8c0fadd48` |
  | (L) seed 1 | `s1/L/models/step-008192` | 2,048 | `cell` | `s1/SHA256SUMS.files` | `ca0b8aeda4e23a6f793df8b503534a48ff090724632dc384c66a0b4b27424b23` |
  | (L) seed 2 | `s2/L/models/step-008192` | 2,048 | `cell` | `s2/SHA256SUMS.files` | `3c26a25280457e5b58c44e1bb7fc82dcd6a51b176fa86f49f6c5161a376598f3` |

  a7r is under `results/claude/one-model-end-to-end-2026-10-29/`, and (L) is under `results/claude/one-model-end-to-end-long-context-2026-11-04/`.
  - **Where W comes from:** the step's `config.json`, `context` 192 or 2,048. The scorer reads it, and refuses a `--context` other than W.
  - **Where the objective comes from:** the run's `config.json`, two levels up, with `objective` `cell`. Both runs have turn memory 8, and neither has `fbs_arm`, so no `--fbs-arm` is passed.
  - **Pod steps:** the job fetches each step folder and its run's `config.json`, then checks every file against the sum file's lines.
- **No training.** The object is the trained float model under the float scorer. No served, integer or milestone claim rests on this run. claude-288 measures the integer session.

## Data split

- **p-long** (claude-275, sealed; SHA-256s on [claude-276's sheet](one-model-end-to-end-long-context-2026-11-04.md#data-split)), for a7r:
  - `cell=dev,dev`, 160 conversations, 313,980 tokens and 609 queries;
  - query distances: 23 under 192, 586 at 192 or more, at most 1,994;
  - its longest conversation fits 2,048, so carry runs at C = 2,048.
- **p-long4k** (this unit), for (L):
  - **The command:** `heldout-panel out=… tokenizer=TOKENIZER cell=dev,dev conversations=160 seed=2771 protocol=2 chain=4096`, with the pinned tokenizer `d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89`. This is a data tool, not an evaluation.
  - **The draw:** 640,796 tokens, 640 queries and 0 metered mismatches. The redraws were 1,770 contradicting, 562 farewell, 94 mentioning, 1,280 too long and 0 dropped.
  - **The checks:** `check-queries` reports 640 checked, 0 failed.
  - **Distances:** 8 under 192, 250 under 2,048, 129 at 2,048–3,071 and 261 at 3,072 or more, at most 4,044. Replies are at most 21 tokens. Carry runs at C = 4,096.
  - **SHA-256s:**
    - tokens `ffefe084ae10d5d049ace0bb5acc49e9f7d2537b4a64038327f283ccf29933d9`;
    - mask `bccfdd7564d9795b4f4de4b260080defa488882a07c60d0a26809f0bb68db95d`;
    - `queries.json` `2f615b45439b6b2d590611420826594054a87451d0483d1099a8f45e4a7aa568`;
    - `user_turns.json` `0b94d34143caac7621cf37bcd0bcbe5fdc475aa865ebf6d6037ad738337452ee`;
    - `manifest.json` `97e7f14a29196cbbb63d5caa09227fe712f0b0185b0fc9b766bc081e56c70882`.
- **The census** is `train-in-space --census 1` against P2b's long TRAIN (`--train-split` with `61019954ef82d7b33d2d5a101f436fece29452f8580f8f3268e35229ccd0cb3c` and `29b34d2c9ed3a56bf638a3afd6863e5744b2c1d3f838a1034f19a77d1e02818d`), run on the laptop at this unit's code. `census.json` is `6c942e8879a23aecc1fe271f8c54d7fd21299513353868c87170379bd0e1244c`.
  - **Prompts:** p-long4k's exact prompt share is 0.0, and its template share at 1,024 is 0.0. So no prompt is shared with TRAIN, and `dev,dev` phrasings are never in TRAIN.
  - **Replies:** its exact reply share is 0.72, because the reply templates are shared by design.
  - **Answers:** 0 of 319 words are in TRAIN, and 12 of 321 numbers are, `found_queries` 31, 72, 328, 336, 340, 396, 452, 495, 555, 590, 591 and 620. p-long's census is [claude-281's](one-model-end-to-end-p2b-classification-2026-11-04.md#data-split): 0 of 319 words and 6 of 290 numbers.
- **The user turns** are recorded in `user_turns.json`. The long TRAIN was drawn before this panel and is not redrawn, so nothing is excluded from it (the manifest's `excluded: none`). The census's exact prompt share of 0.0 is the overlap check.
- **Stored** with a `SHA256SUMS`, read back 6 of 6, under `results/claude/integer-long-context-2026-11-05/inputs/{p-long4k,census}/`.

No model output was read to set any panel, reading, band or bar.

## Arms

There are 12 passes: two checkpoints, two seeds and three rules. Each pass writes memory on and memory out (L-010). The command is P2b's scoring command (claude-276) with the rule added and `--context` dropped:

`train-in-space --data-root DATA --score-rows CKPT --out DIR --samples 16 --reply-tokens 32 --retain-rates 1 --seed S --panel NAME=PANEL --panel-queries NAME=PANEL/queries.json --past-window RULE [--past-window-stride s]`

- **The decode:** the default greedy decode, as in P2b and claude-281; `--flock-end` is not passed. `--seed` is the checkpoint's own seed.
- **a7r** scores `p-long`, and **(L)** scores `p-long4k`.
- **blocks** runs `build_document_windows` at W, the base path.
- **slide** uses stride s = W/16: 12 for a7r and 128 for (L). Each target past W then reads at least W − s + 1 tokens, 181 and 1,921, about 94% of W.
- **carry** sets the model's context to C, the panel's longest conversation rounded up to 2,048 or 4,096, and scores one window per conversation.
- **Queries:** each prompt ends at its reply, and starts where the rule says:
  - slide: the W positions before the reply, cut at `<|bos|>`;
  - blocks: the start of the reply's block;
  - carry: `<|bos|>`.

  The decode runs at W for slide and blocks, and at C for carry. A blocks reply that crosses its block's end slides from there (`blocks_crossing`, reported).
- **DEV:** every pass also scores DEV at 192, as the identity check (I-1). Fresh V2 is refused with the option, and is not scored.
- **Order,** serial on GPU 0 of one pod with one binary, so each pass's tok/s is its own:
  - a7r seed 1: blocks, carry, slide;
  - a7r seed 2: blocks, carry, slide;
  - (L): blocks at seeds 1 and 2, then carry at seeds 1 and 2, then slide at seeds 1 and 2.

  (L)'s slide passes run last. Both read their stride from one file on the pod, written before the first starts (Stop rule).

## Readings

Every reading is per checkpoint, seed, rule and memory setting. Each comes from that pass's `past-window-NAME-{memory,no-memory}.json`, by a fixed `jq`. A row's band is `(position − 1 − doc_start) / W`, so band 0 is each conversation's first W targets. A query's band is its note's distance divided by W.

- **R0, tok/s first (reported).** Each queries file's decode tok/s, from the `decode` pair `--score-rows` prints, and each pass's wall seconds. Set beside claude-281's 137–148 tok/s at 2,048.
- **R1, loss past W (deciding).** The mean of the checkpoint's own `query_loss` over every target at offset W or more. The objective is `cell`, with no softmax or cross-entropy form (§3.2).

  ```
  jq '.window as $w | [.bands[] | select(.from >= $w)]
      | (map(.loss_sum) | add) / (map(.targets) | add)' past-window-NAME-memory.json
  ```
- **R2, reply agreement past W (deciding).** Over the queries whose note is at least W back, the reference tokens the greedy reply gives at the same place, both cut at `<|eos|>`, over the reference's tokens. That is 586 queries for a7r on p-long and 390 for (L) on p-long4k.

  ```
  jq '.window as $w | [.query_bands[] | select(.from >= $w)]
      | (map(.agreed_tokens) | add) / (map(.reference_tokens) | add)' past-window-NAME-memory.json
  ```
- **R3, the curves (reported):** each band's `loss_mean` and nearest-hit rate, and each query band's agreement, from band 0 out to C.
- **R4, hits past W (reported):** the nearest and writer hit rates over the targets at offset W or more.
- **R5, recall (reported, never deciding):** `stated` by query band, and over the census-clean words. P2b and claude-281 read recall at 0 for far notes, so it is not expected to separate the rules (claude-286's probe 5).
- **R6, the memory (L-010, reported):** R1 and R2 with the memory out, and memory on minus memory out for each rule.

## The band, and the decision

**The band.** Within one seed, every rule reads the same model on the same targets (I-3) and the same queries (I-4), on one pod with one binary. So a rule's difference has no sampling noise of its own. The band is what two seeds of one recipe differ by, with a floor:

b_X = max(|X_slide,seed 1 − X_slide,seed 2|, f_X)

- **f_loss** is 1% of the two seeds' mean slide R1;
- **f_agree** is 0.01, one point of agreement.

**The comparisons,** per seed and with the memory on (the served setting):
- **carry is worse** when R1_carry − R1_slide > b_loss, or R2_slide − R2_carry > b_agree. Otherwise **carry holds**.
- **a rule beats blocks** when R1_blocks − R1_rule > b_loss or R2_rule − R2_blocks > b_agree, and it is not worse than blocks on the other reading by more than its band.
- **a rule is worse than blocks** when R1_rule − R1_blocks > b_loss or R2_blocks − R2_rule > b_agree.

**The decision table** (claude-286's), read on each checkpoint across both seeds. The rows are tried in order, and the first that applies decides:

| # | reading | chooses |
|---|---|---|
| 1 | slide and carry are both worse than blocks, at both seeds | blocks |
| 2 | carry is worse, at both seeds | slide, for now. The carry training arm (2 Nov owner log: consecutive chunks, the trunk's state and the read's earlier keys carried, gradients inside a chunk) is the next unit, and P4's bound is checked then |
| 3 | carry holds at both seeds, and carry or slide beats blocks at both seeds | carry |
| 4 | carry holds at both seeds, and neither beats blocks at both seeds (the tie) | carry, provisional, on serving cost. It is re-read on the first checkpoint that recalls far notes inside its window |
| 5 | anything else: the seeds disagree | carry, provisional, with the disagreement named |

- **Blocks is the forgetting control.** A rule no better than blocks past W keeps nothing useful from earlier context. With recall at 0, row 4 is the likely outcome, since the panel then has not tested what the earlier context is for.
- **Which row is served:** a7r's row decides the rule claude-288 serves, since a7r is the served model.
  - **(L)'s row** reads the same table from 2,048 to 4,096, the target's length.
  - **If (L)'s row differs from a7r's,** the served rule is still a7r's, but provisional. (L)'s row is named in the decision review as what the next long checkpoint re-reads.
  - **If (L) reads row 2,** the carry training arm is named for the long window, whatever a7r reads.
- **With the memory out,** the table is read again (R6) and reported. If its row differs from the memory-on row, that is named.

Whatever the outcome, every reading is reported with its numbers. A failed instrument check leaves every reading unread (§4.1).

## Instrument checks

- **I-0, the suite:** the float crate's CUDA library suite passes at the pin on the pod, with `--no-fail-fast`. The past-window tests are CPU tests inside it: the windowers, the cut, the ruled query prompts, the refusals, the saved configs, the bands, and the base path unchanged.
  - **No I-0n:** this unit fixes no defect, so no test pins a fix to set aside. Its tests pin new code.
- **I-1, DEV is the identity:** DEV is scored at 192 in every pass, and its rows file holds no context field.
  - **Rows:** per checkpoint and seed, `rows-dev-memory.json` and `rows-dev-no-memory.json` are equal byte for byte across the three rules, carry included.
  - **Samples:** DEV's sample files are equal across slide and blocks. Under carry the model's context is C, so a DEV sample whose decode passes 192 does not slide; any difference is listed and classed, and reported only.
- **I-2, band 0 is the identity:** each conversation's first W targets read the same prefix from `<|bos|>` under every rule.
  - **slide and blocks** score them in the same first window, so band 0's `targets`, `nearest_hits`, `writer_hits` and `loss_sum` are equal exactly.
  - **carry** scores them in one longer forward. Its band 0 `targets` are equal exactly. Its hits are within 0.3% of band 0's targets of blocks', the scale of the 0.13–0.28% of top-1 picks claude-279 found differing between pods. Its `loss_mean` is within 0.1% of blocks'.
- **I-3, every target once:** per checkpoint, seed and memory setting, each band's `targets` is equal across the three rules.
- **I-4, the same queries:** each query band's `queries` and `replies` are equal across the three rules, and the `[queries]` line counts 0 past the window.
- **I-5, the loss is read:** every band has a `loss_sum`, the file's `objective` is `cell`, and the pass prints `[past-window] model reads W; loss under cell`, with C under carry.
- **I-6, carry runs at C:** carry's `[past-window]` line reads `run at 2048` for p-long and `run at 4096` for p-long4k, with 160 windows.
- **I-7, the inputs:** on the pod, every SHA-256 above equals its file's. `check-queries` passed on the laptop (Data split).
- **I-8, the grade can see a pass:** in every `recall-*.json`, `total.reference_states` equals `total.decoded`.

## Steps

1. On the pod:
   - fetch the pin, build `train-in-space` with `--features cuda --locked`, and record the binary's SHA-256;
   - check that `jq` is there;
   - run I-0.
2. Fetch the inputs: the driver data, p-long, p-long4k, and the four checkpoints with their run configs. Check every sum, then I-7.
3. Run the 12 passes in the order under Arms, with a watcher on the job.
   - After each pass, the job prints the wall seconds and the `[past-window]` and `decode` lines.
   - It stops on a failed I-5 or I-6.
4. Seal every output with a `SHA256SUMS` under `results/claude/integer-long-context-2026-11-05/`, read it back, and only then take the pod down.
5. On the laptop: run I-1 to I-4 and I-8, compute R0 to R6, and read the table.

## What the test can tell apart

- **Within a seed,** the rules are compared exactly, with no sampling noise (I-1 to I-4). The band is the seeds' own spread, so a difference inside it is no larger than what retraining moves.
- **Past W:**
  - every target at offset W or more in 160 conversations, about 283,000 positions in p-long and 313,000 in p-long4k before the mask (the masked count is in the files);
  - 586 queries for a7r and 390 for (L).
- **What it cannot tell:** with recall at 0, agreement past W is mostly template tokens, which every rule can produce without the note. So a tie (row 4) says the panel did not test the earlier context, not that the rules are equal at recall. That is why row 4 is provisional, and is re-read on a checkpoint that recalls.

## Cost

These are estimates from measured passes, not measurements:
- claude-276: a 2,048 scoring pass without the cache took 2,622 s, 2,563 s of it the queries, so about 0.18 s or less per 2,048-token row forward.
- claude-279 and claude-281: a cached 2,048 pass takes 305–441 s, and a 192 pass 78–91 s.

| arm | per seed | both seeds |
|---|---|---|
| a7r: blocks, slide (about 24,000 windows of 192, 16 a forward), carry (decode at 2,048) | about 1.5 + 3 + 8 min | about 26 min |
| (L): blocks, carry (forwards and decode at 4,096), slide (about 2,600 windows of 2,048, one a forward, memory on and out) | about 8 + 12 + 22 min | about 84 min |

- **Building, I-0 and fetching** add about 25 min, and sealing about 5 min.
- **In all,** about 2.3 h on one RTX 5090 at up to $1/h is about $2.30, inside the ORDER's $3 bound.

## Stop rule

- **The run stops** when all 12 passes are scored, the files are sealed and read back, and the pod is down.
- **The bound:** the job is started with the pod's price an hour. Before each pass, it stops once the elapsed hours times the price passes $2.75, which leaves $0.25 for sealing and `down`. The passes left are then recorded as not run, and nothing is cut silently.
- **The stride first (the ORDER's stop row):** if, after a7r's passes and (L)'s blocks and carry passes, the projected total passes the bound, (L)'s slide stride doubles to 256 for both seeds and is recorded. Each target past W then reads at least 1,793 tokens, 88% of W. No arm is cut before the stride.
- **Carry at 4,096:** if the Hamming read refuses, or memory runs out, carry already runs one conversation a forward (`windows_per_forward` at 4,096 is 1). So it is recorded as an instrument limit, and (L)'s row reads not measured.
- **A failed instrument check** is classed, repaired and re-run once inside the bound. Nothing is trained.

## Code in this unit

All of this is new code, with nothing changed on the base path:
- **`hamilton_sheet.rs`:** `build_document_windows_sliding` (slide), and `carry_context` with `carry_windows` (C, and one window per conversation).
- **`in_space_long.rs`:**
  - `PastWindowRule` and its refusals;
  - `past_window_cut` and `query_prompts_ruled`;
  - the bands, and `query_loss` by band in the rows' own batches, with no new arithmetic;
  - `checkpoint_window` and `run_objective`, reading the saved configs;
  - the `past-window-*.json` output.
- **`bin/train-in-space.rs`:** `--past-window` and `--past-window-stride`. A stride without a rule is refused, and so is the option without `--score-rows`.
