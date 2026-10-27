# REPORT: in-space-trunk-to-text, claude-142

Lab: claude. Order: [ORDER.md](ORDER.md). Prep sheet: [in-space-trunk-to-text-2026-10-27](../../../../harness/prep/in-space-trunk-to-text-2026-10-27.md) (TEST FITNESS: FIT in round 2; D67 pre-run review PASS; both on #2028). Direction: Casey, 27 Oct 2026, in claude's chat: train in the same space used for transport and prediction, decode only at the end, always look at context, and show the text.

The item `in-space-trunk-to-text` is new in this PR; its issue is made at the next roadmap sync from main, and it stays open.

**Speed.** DECODE_SPEED_LINE Training on one RTX 5090: the best run (whole-table nearest objective, `lr 1e-3`) ran at `13,794` context tokens/s with three runs sharing the GPU; a writer-objective run alone ran at `35,597`.

## What did not work, first

- **The replies still do not answer their prompts.** The best model's greedy replies at step `16,384`:

  | Prompt (end) | Model | True |
  |---|---|---|
  | "Remember these: chadhik is stoumvisplund, …" | "Got it, your friend is  is a a a a a a …" | "Got it, I'll remember them." |
  | "… Can you fix it?" | "It's 1 I'll remember that." | "Could you tell me more about what you mean?" |
  | "Please remember: sheerkkleetmeer is 23, …" | "Got it, your friend is G." | "Got it." |
  | "Make a sentence with the word river." | "Got it, your friend is Gist is a I'll remember that." | "The river made me smile." |

  The model has learned the corpus's common reply templates, not what each prompt asks; long replies fall into repetition.
- **The model reads about 6 tokens back.** Replacing the token `k` places before a reply opening with BOS moves the opening point by `4.37` at `k = 0` and `0.049` at `k = 16` (`R16 = 0.011`). The untrained trunk reads much further (`R16 = 0.70`): training cuts the reach about 60–100 times. Item done-when (2) is **not met**.
- **Lanes end at the shortest memory.** `61–63%` of lane positions select rate entry 0 after training, against `33%` at step 0.
- **Three ways to keep memory long all stop training.** Train loss stays at the margin (`1.002–1.004`) and whole-table nearest top-1 stays near `500–1,300` against the clamped twins' `8,097–8,607`:
  - the rate score folded into `[0, 1]` (the registered run: **negative**, cell C with flag F at both rates; its reach "pass" is the untrained trunk's);
  - a reflecting floor `min(|x|, 1)` (exploration);
  - the shared table placed by the output side only (exploration, with the clamp and with the fold).
- **Two causes I proposed were measured and refuted:**
  - the optimizer's global clip (Adam normalizes per element; the step-0 norms are `12.55` fold and `1.24` clamp, almost all of it the table's);
  - the table's input-side gradient drowning its output side: it does, `1/410` under the clamp and `1/4,300` under the fold, but cutting the input side stalls the clamp too (`1,118` against `7,425` at step `4,096`).

  **Why the long-memory variants stall is not known.**
- **The writer's admission decoding fails at higher rates, and that was an instrument fault, not the model.** The writer's address hinge asks a margin of `1`, while its 16 roots are at least `0.5` apart in mean L1, so by the triangle inequality it is never met. At `lr 1e-3` the model flattened its address ports until the hinge read exactly `1.0000`; admission then picked arbitrary cells (writer top-1 `345`) while the whole-table nearest decode of the same checkpoint read `7,813`. The 25 Oct six-recurrence sheet's `S^3` seed 1 shows the same pattern (top-1 collapsing while DEV loss descends) and was not re-read here.
- **The long writer-objective run collapsed after its peak:** `7,593` at step `12,288`, then `1,363` at `18,432` (train loss `1.55 → 2.19`). The cause is not known.
- **Decode speed here re-runs the whole window per token,** so it times this runner, not a served recurrent decode.

## What changed

- `crates/uor-r4-float-train/src/in_space_long.rs` (new) and `src/bin/train-in-space.rs` (new): the long in-space runner, with:
  - three objectives (`sheet`, `voronoi`, `nearest`);
  - DEV text at every checkpoint;
  - and five readers of saved checkpoints:
    - `--inspect`: the loss in its parts, nearest top-1, openings, query spread;
    - `--reach`: the opening point's shift by distance, and each layer's rate entries;
    - `--grad-norms`: per-parameter gradient norms, and the table's input and output sides;
    - `--decode-speed`;
    - `root_separation`.
- `geometric_stack_native.rs`: four switches, all off by default, so every recorded run reproduces:
  - retention rates;
  - folded scores;
  - a reflected floor;
  - a fixed input table.
- `secant_descent.rs`: a capture helper for rate entries.
- Tests:
  - host: epoch order, tail, root separation;
  - CUDA: the fold's values and slopes, run on the pod in the `fold-test` step (`4 passed`).
- Docs:
  - the new item;
  - the prep sheet;
  - this ORDER and REPORT;
  - two notebook rows in `float-training.md`.

**Evidence.** Every run's `progress.jsonl`, `samples.md` and checkpoints are in `caseyallard/uor-r4-store:results/claude/train-in-space-2026-10-27/`. The run folders are:

- `run1`, `sweep`, `sweep2` and `long2`: the writer objective;
- `long3`: nearest and Voronoi, with the twins' re-read in `long3/reread2`;
- `long4`: fold;
- `long5`: floor;
- `long6`: fixed table;
- `gradnorms` and `gradsplit`;
- `long3b`: the first reach and inspection of `long2`.

## Measurements

**What worked.**

- **Next-token top-1 passes both count tables, in the trunk's own space.** The whole-table nearest objective (the target must be nearer than the table's 8 nearest wrong tokens; decoding is the nearest token):

  | Run (`16,384` steps, retention on, `lr → 0`) | Nearest top-1 on DEV |
  |---|---|
  | `lr 1e-3` | **`8,607 / 14,287` (`60.2%`)** |
  | `lr 5e-4` | `8,097` |
  | bigram count | `6,300` |
  | M1 count table | `7,002` |

- **The first prompt use at reply openings.** At `lr 1e-3` the model gets **`220`** of `1,226` openings right, with 3 distinct opening tokens, against `161` for always naming the commonest opening (`+59`, three binomial standard deviations are `36`). Item done-when (1) is **met** by this run.
- **The retention reading of the rate table** (a lane carries `1 − 2^-(k+1)`) took the writer-objective run from `3,508` to `6,929` at `8,192` steps, everything else equal.
- **A satisfiable address margin** (half the nearest root separation, `0.25`) gave the best writer-decoded score, `7,906` at `16,384` steps, without the collapse.

## Checks run at the head

CHECKS_LINE

## Skills used

- **holding-the-goal.** The goal stayed Casey's: train in the trunk's own space, decode at the end, show the text, look at context. After the n-gram line was rejected, next-token top-1 alone was not taken as success. Reply openings and reach were measured, and the replies are shown as they are.
- **root-cause-debugging.** Each stall was measured before it was fixed. The gradient-norm probe refuted the clip theory. The table split was measured, then refuted as the cause by the fixed-table arm.

## Still open

- **The next unit: a read over the context.** The recurrence learns short memory, and changing how its memory is bounded did not change that. A geometric read of earlier positions, the project's exact memory, is what would let a reply opening see a prompt 20–80 tokens back. That is a new item, not this one's.
- **Why the long-memory variants stall.** Recorded above as unknown.
- **Re-read the 25 Oct six-recurrence `S^3` seed 1 checkpoints with whole-table nearest decoding,** if they were saved.
- **The writer-objective run's collapse after step 16,384.**

## Plain summary

The model was trained much longer than before, in its own geometric space, and asked to write replies.
- **It predicts the next word well.** It gets about 60% right on held-out text. Counting word pairs gets 44%, and the earlier counting model 49%.
- **It barely reads the prompt.** When it starts a reply, it looks back only about 6 words, so it cannot see what a long prompt asked. It starts a reply correctly a little more often than always guessing the commonest first word (220 against 161 of 1,226). That is the first sign of it using the prompt at all.
- **Its replies are not real answers yet.** It writes the corpus's common phrases ("Got it, your friend is …"), then repeats itself.
- **Three attempts to give it longer memory made it stop learning.** Why is not known yet. The next step is a different mechanism: letting the model look back over the prompt directly, the project's own exact memory.
