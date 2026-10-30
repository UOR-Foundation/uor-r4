# Prep sheet: train with the reply's read held to the prompt (one-model-end-to-end, claude-216, 30 Oct 2026)

Lab: claude. Item: [one-model-end-to-end](../../roadmap/items/one-model-end-to-end.md). Order: [claude-216](../../loop/tasks/one-model-end-to-end/claude-216/ORDER.md). It follows claude-215 ([result](../../labs/one-model-end-to-end/reply-scope-2026-10-30/README.md)), whose registered outcome, "reduces, content lost", unlocked exactly this follow-up.

TEST FITNESS: FIT (round 2, a fresh non-author reviewer, 30 Oct; its five notes are taken in below). Round 1 was NOT FIT on two points, both repaired here:
- the training cut and the decode cut were different rules, so earlier replies inside a window were read one way in training and another in serving; now one rule, built from the response mask, holds everywhere ("The cut");
- an empty matched set would have read as a content negative; it now reads `underpowered (content)` below a registered floor ("Bar").

## The question

Does training the cut in, so that every reply position's Hamming read chooses only positions before its reply's first token in training as in serving, keep claude-215's loop reduction without its content loss? And does it cost the model its teacher-forced accuracy?

**Why this, now.**
- **claude-215 decoded the sealed (r-drop) with the cut and nothing else.** On p-r it removed 26–51% of greedy's loops, beyond the band at every seed. But the model had been trained to read its own reply, so the cut took it off what it had learned: loops were often cut into salad, and at one seed the replies were less TRAIN-like.
- **Training with the cut** is the registered next step: a model that has never read its own reply should not need to.

## Base artifact: the model and the arms

- **(r-cut), trained here:** (r-drop)'s recipe (the training sheet's command) plus `--reply-read-train all`:

  `train-in-space --objective cell --hamming-read 4 --retain-rates 1 --decay-mult 1 --batch 16 --lr 0.001 --lr-end 0 --turn-memory 8 --turn-memory-reply 1 --turn-memory-dropout 500 --reply-read-train all --steps 16384 --eval-every 2048 --seed S`

  at seeds 1, 2 and 3.
  - **The cut, one rule everywhere:** every row whose input is a reply token reads, for every Hamming head, only positions before its reply's first token in the window, or none when the reply began before the window. Every other row reads as before, so a reply's first scored row, whose input is the prompt's last token, keeps the whole prompt (`reply_row_limits`, `in_space_long.rs`).
    - **Its flags** come from the split's response mask at each window position (`window_input_mask`), not from the scored rows. A window that ends a long conversation carries up to 192 earlier tokens whose targets belong to the window before it, earlier replies among them; those reply rows are limited too.
    - **Training, the teacher-forced rows and decode** all apply it. A decode window's flags are the prompt's own, then one per reply token decoded (`decode_window_mask`). So an earlier reply inside a decode prompt reads only before itself, exactly as in the training window that holds the same tokens, and the reply decoded reads only the prompt's positions, claude-215's cut (`a_decode_window_is_limited_as_its_training_window`).
    - **The training log** counts the TRAIN rows the cut limits, and the rows that read nothing because their reply began before the window (`[cut] TRAIN rows`).
    - **The one difference left, stated:** training cuts documents into fixed windows, and decode slides its window. A training window that opens partway through a reply trains that reply's rows with nothing to read in the limited heads (56 of 365 reply rows in the unit's fixture). Decode almost never meets this, since a decoded reply keeps the prompt's end in view unless the whole prompt scrolls out, past 192 reply tokens; this run decodes 64. It is a property of the configuration, not an instrument fault. The REPORT gives the real share from the log beside the results.
  - **The record:** the run's `config.json` records `"reply_read_train": "all"`, and `--score-rows` applies the same cut to the checkpoint's teacher-forced rows.
- **(r-drop), sealed:** step-16,384 at seeds 1–3, at `results/claude/one-model-end-to-end-2026-10-29/training-460fe46e/s{S}/r-drop-s{S}/`, checked against that seed's `SHA256SUMS`. The `model.safetensors` sha256s:
  - s1 `c5fcbb90ab49c284da56abc6e74e74a4d8f1bded2d7815f8fb70de8332c81465`;
  - s2 `c8b9b420c4ab5d56cc0dfc4f3cfd68a3aaa06cec7024c5484d6311ff61764f2b`;
  - s3 `8727c8a065840811f4e63b74b60ccc8b65d3962bd5394a1c64f17b74b1734e85`.
- **The decode arms:** greedy, with the memory on, 256 replies per split and seed. Every arm answers the same prompts.

  | Arm | Model | Decode |
  |---|---|---|
  | **base** | (r-drop) | as trained (`greedy`) |
  | decode-only | (r-drop) | with the cut (`greedy-read-all`): claude-215's arm, now also holding an earlier reply in the prompt to before itself |
  | **served** | (r-cut) | with its cut (`r-cut/greedy-read-all`): decides |
  | uncut | (r-cut) | without the cut (`r-cut/greedy`): reported |

  The decode-only and served arms use one decode rule; only their training differs.

- **Teacher-forced:** (r-cut)'s nearest picks on every p-t row, with its trained cut, against (r-drop)'s as trained.

## Data split

- **TRAIN, DEV and the tokenizer:** `w2-inputs/2026-10-25/driver-data/`, pinned by the code.
- **p-t: decides.** A new held-out-phrasing panel, drawn on 30 Oct by claude-204's `heldout-panel` at seed 2081, a seed no unit had used:

  `heldout-panel out=p-t cell=dev,dev conversations=400 seed=2081 protocol=2`

  - **The panel:** 50,646 tokens and 16,465 reply targets.
    - `tokens.u16` sha256 `6ec5e4cf7a090e51e5bf071d9af85355140c16111407cffdb94a035b65cbf6e6`;
    - `response_mask.u8` sha256 `da37dd5961fc5762facef7782cc1dabc471367b9d0007d24dbe2f1c28f5a3354`.
    - A second draw is byte-identical.
  - **Stored** at `inputs/claude/heldout-panel-p-t-2026-10-30/`, with a `SHA256SUMS` and the census.
  - **The census** ([census-p-t-2026-10-30.json](../../labs/one-model-end-to-end/census-p-t-2026-10-30.json)):
    - 0.000 of its 1,424 prompts are in TRAIN exactly;
    - 0.000 are in TRAIN as value-masked templates at K = 1,024 (0.041 at K = 256);
    - 0.610 of its replies are verbatim TRAIN replies.
- **DEV:** reported.
- **p-r, p-c, P-dev and p-f are not used.** Each has been opened by an earlier decision.

## Steps

Three pods, one RTX 5090 each, one per seed. Each runs `local/claude/claude-216/job.sh <head> <seed>` in this unit's worktree; its sha256 is in the pre-run review.

1. **The source,** pinned to this unit's pushed head, and clean.
2. **The CUDA tests** (`--release --locked`). The job stops unless each of these passes:
   - `a_cut_keeps_every_pick_before_it`: a uniform cut, and a per-row cut where every row's picks are before its own limit;
   - `the_read_scope_starts_as_greedy_and_clears_its_cut`;
   - claude-211's and claude-213's decode tests.
3. **The build,** `train-in-space`, kept with its sha256.
4. **The data,** every file checked:
   - the panel, against its `SHA256SUMS` and its pinned `tokens.u16` sha;
   - the seed's 11 sealed files: the run config, 5 checkpoint files and 5 registered files;
   - claude-211's s1 `probe-r-drop` step-256 checkpoint, against s1's `SHA256SUMS`.
5. **The default path:** a 256-step (r-drop) probe (seed 1, `--eval-every 256`) must reproduce claude-211's sealed probe checkpoint byte for byte. That shows this unit's edits leave training without the cut unchanged.
   - **The registered fallback:** training's bytes were shown to repeat on one GPU (claude-211's I-H0), never across pods. So on a cross-pod miss, the base commit (claude-215's merge, `0c3c95777638f31263e1ba2297c83594a61a7983`) is built and trains the same probe on the same pod, and this head's probe must match it byte for byte.
   - A miss against the base is EXIT 6, an instrument fault: this unit changed the default path.
6. **The (r-cut) probe:** 256 steps.
   - Its `config.json` must record the cut, and its log must count more than zero TRAIN rows the cut limits.
   - At seed 1, its checkpoint must differ from the (r-drop) probe's. Any change in the random draws would pass this too, so it only shows the flag reached training; the unit tests pin the rule itself.
   - Its time is projected ×64; past 2 hours, the job stops.
7. **The training:** (r-cut), 16,384 steps.
8. **The scorings,** side by side, (r-cut)'s step-16,384 and the sealed (r-drop)'s, each with:

   `--score-rows CKPT --out DIR --panel p-t=PT --samples 256 --reply-tokens 64 --retain-rates 1 --flock-end 1 --reply-read all --seed S`

   This writes the rows on DEV and p-t (with the memory on and out), and the samples greedy (with the memory on and out), `FLOCK_END`, and greedy with the cut. Each scoring prints the trained cut it read from its run's `config.json`: the job stops (EXIT 6) unless (r-cut)'s is `all` and (r-drop)'s is `none`.
9. **Reproduction:** the (r-drop) scoring reproduces claude-211's 5 registered files byte for byte (EXIT 6 otherwise). Every expected p-t file is written (EXIT 8 otherwise).
10. **The seal:** each pod's `/workspace/cut-train` (without its inputs) goes into the store at `results/claude/one-model-end-to-end-cut-train-2026-10-30/run-<sha8>/`, with a `SHA256SUMS`, and is read back before the pod is released and taken down.
11. **The grade, on the laptop:**

    `train-in-space --data-root DATA --cut-grade 1 --runs RUNS --panel p-t=PT`

    That is `cut_grade`, unit-tested with negative gates. `RUNS` holds `r-drop-s{1,2,3}` and `r-cut-s{1,2,3}`.

## Bar

Each reading is one arm's 256 replies on one split at one seed, against that split's own references after the same prompts.
- **Loops, fewer loops, holds, close:** claude-215's, unchanged:
  - **Loops:** replies that repeat a 4-gram of their own before their end of turn.
  - **Fewer loops:** the arm's loops plus decision 4's band at greedy's count are fewer than greedy's.
  - **Holds:** decision 4 against greedy.
  - **Close:** "generalizes"' free-generation condition.
- **Matched content:** the verbatim TRAIN share on the prompts the base and the served arm both complete (`matched_counts`, `MATCHED_BY`), the two the decision compares. A share neither arm's completions select, which answers claude-215's selection effect. The decode-only and uncut arms, which decide nothing, are counted on the same prompts (a reply of theirs that does not complete counts as not verbatim), so they cannot thin the deciding set.
  - **Kept:** the arm's matched share is below the base's by at most `max(0.10, 3·√(p(1−p)/m))`, with `p` the base's share and `m` the matched prompts (`matched_kept`).
  - **The content floor:** the base must have at least 64 matched prompts at every seed (`MATCHED_FLOOR`). Below it, or with none counted, the content reads `underpowered (content)`, an instrument outcome, never a negative.
- **The rows' cell:** (r-cut) against (r-drop) on every p-t row, paired. It uses the protocol's band, `max(⌈2·√(b+c)⌉, the three-seed spread)`, and is ahead or behind only at all three seeds.
- **The power floor:** greedy (base) loops in at least 40 of 256 p-t replies at every seed.

**p-t decides,** from the served arm at all three seeds (`cut_outcome`). The rows are tried in this order:

| Outcome | Condition | What it unlocks |
|---|---|---|
| **underpowered** | The base loops in fewer than 40 replies at any seed. | Nothing is decided: an instrument outcome. A new panel or reply length is next. |
| **no gain** | Not fewer loops and holds, at all three seeds. | A negative for the trained cut, in this configuration only. The next candidates are the turn memory's reply table, or the trunk itself. |
| **costs accuracy** | Fewer loops and holds, but the rows' cell is behind. | The cut trades teacher-forced accuracy for fewer loops. Next: train with the copy heads' cut only, which carried most of claude-215's effect. |
| **underpowered (content)** | Fewer loops and holds, rows not behind, but fewer than 64 matched prompts at some seed. | The loops are decided; the content is not. A longer reply or a larger sample is next, before any recipe change. |
| **cures** | Also close and the matched content kept, at all three seeds. "Close" alone is on each arm's own completions, so it cannot cure a content loss. | (r-cut) becomes the one model's recipe and served decode. The next unit serves it in integers: the integer Hamming read with the row cut, from the response mask, and its tok/s. |
| **improves** | The matched content kept at all three seeds, but not close at all three. | (r-cut) becomes the one model's recipe and served decode, as for cures. The content gap to the references is the next work. |
| **reduces, content lost** | Otherwise: fewer loops and holds, but the matched content not kept at some seed. | The cut, trained in, still costs content. No recipe change. Next: the turn memory's reply table, keyed by the reply's last id, which the cut leaves reading the reply. |

**Reported beside it, deciding nothing:**
- **The decode-only arm's outcome,** by the same rule (claude-215's arm, on a fresh panel).
- **The uncut arm's readings.**
- **The DEV readings.**
- **The training tok/s,** and each arm's decode tok/s (with two scorings sharing the GPU, a floor).
- **Each arm's mean reply length,** and a paired loop count against the base.
- **A fixed reading** of the first 20 p-t replies of seed 1 under base, decode-only and served.

## Minimum test power

- **The sample:** 256 replies per arm, split and seed; the same prompts under every arm.
- **The loop band:** at greedy's 63–92 looping replies (claude-215's p-r), the band is 14–16 (`count_band`, `⌈2·√(g(n−g)/n)⌉`). The floor of 40 gives a band of 12.
- **The matched prompts:** a looping reply does not complete, so at claude-215's p-r loop counts the base completes at most about 164–193 of 256, and the three-arm set is smaller still. The floor of 64 is registered for that reason. The tolerance is 0.10 down to about 225 matched prompts at `p` ≈ 0.5, and widens below that (0.19 at 64).
- **The rows' cell:** about 16,000 p-t rows per seed, with the protocol's paired band.
- **The seeds:** every condition needs all three.

## Stop rule

- **A pod stops** on any failed CUDA test, data check, default-path check (EXIT 6), probe check or projection, with its own EXIT code.
- **EXIT 6 and EXIT 8 are instrument faults,** repaired and re-run, never read as results.
- **The budget is $10** across the three pods. The laptop watcher (`local/claude/claude-216/watch.sh`, claude-211's three-pod form) prices each pod over its time up.
  - **A healthy run is not cancelled** for a budget it set itself.
  - **An unhealthy run** is sealed and taken down. Unhealthy means a process exited non-zero, or a pod's line was unchanged for 30 minutes.
- **A pod is sealed and taken down** as soon as its job has an EXIT.
- **A small miss on a bar** is "keep and iterate", not a stop.

## Cost

- **Training:** claude-211's (r-drop) trained 16,384 steps in about 1,470 s on a fast 5090 (34,000 context tok/s). A slow pod may take twice that. The row cut adds one limit vector per batch and head.
- **Scoring:** two checkpoints per pod side by side, about 10 decodes of 256 replies each, about 30–60 minutes.
- **In all:** about 1.5–2 hours per pod.
- **The lease:** 3 pods of 1 × RTX 5090 at $0.56–1.0/h, so about $3–6.
  - **The caps:** `uor-pod status` showed 0 pods running across the labs, $0/h of $8/h. Three pods is inside the caps.
