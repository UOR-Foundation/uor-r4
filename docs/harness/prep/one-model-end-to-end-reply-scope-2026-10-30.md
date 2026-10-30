# Prep sheet: do the copy heads make the reply loop? (one-model-end-to-end, claude-215, 30 Oct 2026)

Lab: claude. Item: [one-model-end-to-end](../../roadmap/items/one-model-end-to-end.md). Order: [claude-215](../../loop/tasks/one-model-end-to-end/claude-215/ORDER.md). Scope decision: claude-214 (#3407 6518346519, decision review PASS).

TEST FITNESS: PENDING. Round 1 (`ba6099583`) was NOT FIT, on two blocking findings, both answered here and in `read_outcome`:
- **A quiet panel would read as a negative.** The sheet now has a pre-registered floor on greedy's p-r loop count: below it, the outcome is "underpowered".
- **"Reduces" changed the decode without a content check.** It now needs the content kept, and otherwise reads "reduces, content lost", which unlocks no decode change.

The notes are taken too: the arithmetic, the reported reply length, the cut's coordinate (now `prompt_cut`, unit-tested with a negative gate), the question's wording, and the copy arm's limits.

## The question

Does holding the Hamming read (its copy heads, or all its heads) to the prompt while a reply decodes end greedy's loops on prompts whose phrasing TRAIN does not hold, without losing the replies' content? The turn memory's reply table, which is keyed by the reply's last id, is untouched in every arm.

**Why this, now.**
- **The loops are not ties.** claude-213 found that inside a loop the nearest token is confident: at margin 0.25, 95.9–99.0% of loop steps admit exactly one candidate. No draw rule can break such a loop.
- **The copy heads are the candidate cause** (`flock_read_hamming.rs`). At position `t`, a copy head finds an earlier `s` whose previous token is the current token, and brings back the token at `s`. In free generation it also sees the reply's own earlier tokens, so a repeated token brings back what followed it last time.
- **The two decode rules both fail on p-c**, at seeds 1–3 of (r-drop):
  - greedy loops, with a repeat rate of 0.559–0.628, and 68–93 of 256 replies repeating a 4-gram of their own;
  - the Flock draws avoid loops only as word salad.

## Base artifact: the model and the arms

- **The model:** (r-drop)'s sealed step-16,384 checkpoints at seeds 1, 2 and 3, the recipe the training run ships. They are at `results/claude/one-model-end-to-end-2026-10-29/training-460fe46e/s{S}/r-drop-s{S}/`, and each file is checked against that seed's `SHA256SUMS`. The `model.safetensors` sha256s:
  - s1 `c5fcbb90ab49c284da56abc6e74e74a4d8f1bded2d7815f8fb70de8332c81465`;
  - s2 `c8b9b420c4ab5d56cc0dfc4f3cfd68a3aaa06cec7024c5484d6311ff61764f2b`;
  - s3 `8727c8a065840811f4e63b74b60ccc8b65d3962bd5394a1c64f17b74b1734e85`.

  Nothing is trained.
- **The arms:** greedy decoding with the memory on, as served. They differ only in which earlier positions the Hamming read may choose at each reply step (`ReadScope`, set by `decode_reply`, `in_space_long.rs`).

  | Arm | The read | File |
  |---|---|---|
  | **greedy** | As trained: every earlier position `1 ≤ s < t`. | `samples-{split}-greedy-memory.json` (DEV: `samples-memory.json`) |
  | **copy** | The copy heads (the first half) choose only `s < P`. The type heads are as trained. | `samples-{split}-greedy-read-copy-memory.json` |
  | **all** | Every Hamming head chooses only `s < P`. | `samples-{split}-greedy-read-all-memory.json` |

- **The cut:** `P = prompt.len() − start` is where the prompt ends in the decode window (`choose_sorted`'s explicit `limit`, `flock_read_hamming.rs`).
  - So a limited head's key and payload are both prompt tokens.
  - A reply's first token is decoded from a window that is all prompt, so it is unchanged.
  - The cut is set only around `decode_reply`'s forward and cleared before any `?`. Training and every other reading run with no cut.
- **`FLOCK_END` is decoded too,** for the instrument's byte check only (step 7). It enters no decision.

## Data split

- **TRAIN, DEV and the tokenizer:** `w2-inputs/2026-10-25/driver-data/`, pinned by the code (`pinned_tokenizer`, `load_train_split`, `load_dev`).
- **DEV:** reported only. Its 256 sample prompts are claude-211's and claude-213's.
- **p-r: decides.** A new held-out-phrasing panel, drawn on 30 Oct by claude-204's `heldout-panel` at seed 2071, a seed no unit had used:

  `heldout-panel out=p-r cell=dev,dev conversations=400 seed=2071 protocol=2`

  - **The panel:** 51,555 tokens and 16,156 reply targets.
    - `tokens.u16` sha256 `ed46e2947b043eb257d986f4806a720e803b9ba07986bfa3418f4aebbf46dddb`;
    - `response_mask.u8` sha256 `aa47372ae30114682bb3c2073646c118e9936a84c0adf5ef591c25b865f0aae6`.
    - A second draw is byte-identical.
  - **Stored** at `inputs/claude/heldout-panel-p-r-2026-10-30/`, with a `SHA256SUMS` and the census.
  - **The census** ([census-p-r-2026-10-30.json](../../labs/one-model-end-to-end/census-p-r-2026-10-30.json)):
    - 0.000 of its 1,453 prompts are in TRAIN exactly;
    - 0.000 are in TRAIN as value-masked templates at K = 1,024 (0.041 at K = 256): new-template;
    - 0.632 of its replies are verbatim TRAIN replies.
  - **Its rows files** (`rows-p-r-*`, which `--score-rows` writes) are read by no decision.
- **p-c, P-dev and p-f are not used.** Each has been opened by an earlier decision.

## Steps

One pod with one RTX 5090. The job is `local/claude/claude-215/job.sh` in this unit's worktree, run as `job.sh <head>`. Its sha256 is in the pre-run review.

1. **The source,** pinned to this unit's pushed head, and clean.
2. **The CUDA tests** (`--release --locked`), the mechanism on a model. The job stops unless each of these passes:
   - `a_cut_keeps_every_pick_before_it`: with a limit, every real pick is before it; rows whose candidates all lie before the cut keep the uncut choice.
   - `the_read_scope_starts_as_greedy_and_clears_its_cut`: a scoped reply's first token is greedy's, and the cut is cleared after every decode.
   - claude-211's and claude-213's decode tests: K = 1, and margin 0, decode as greedy.
3. **The build,** `train-in-space`, kept with its sha256.
4. **The data,** every file checked:
   - the panel, against its `SHA256SUMS` and its pinned `tokens.u16` sha;
   - each seed's 11 files, against that seed's sealed `SHA256SUMS`: the run config, the 5 checkpoint files, and the 5 registered files of step 7.
5. **Throughput:** seed 1 at `--samples 8`, timed and projected ×32. Past 2 hours per seed, the job stops for a code fix.
6. **The run:** the three seeds side by side on the one GPU, each with:

   `train-in-space --score-rows CKPT --out OUT/r-drop-sS --panel p-r=PR --samples 256 --reply-tokens 64 --retain-rates 1 --seed S --flock-end 1 --reply-read copy,all`

7. **Instrument checks.** A miss is an instrument fault, repaired and re-run, never a result (CHARTER §4.1).
   - **Reproduction:** at every seed, this binary reproduces claude-211's registered files byte for byte. They are `rows-dev-memory.json`, `rows-dev-no-memory.json`, `samples-memory.json` and `samples-no-memory.json` (`score-r-drop/`), and `samples-dev-flock-end-memory.json` (`flock-end-r-drop/`): 15 files in all.
   - **The scopes did something:** at every seed, each scope's p-r file differs from greedy's. A scope that changed nothing did not run. That is EXIT 8, an instrument question, classified before anything is read.
8. **The seal:** everything under `/workspace/reply-scope` goes off the pod to `results/claude/one-model-end-to-end-reply-scope-2026-10-30/run-<sha8>/` in the store, with a `SHA256SUMS`, and is read back before the pod is released and taken down.
9. **The grade, on the laptop:**

   `train-in-space --data-root DATA --read-grade 1 --runs OUT --panel p-r=PR`

   That is `read_grade`, unit-tested with negative gates. It ran on stand-in files before the run.

## Bar

Each reading is one arm's 256 replies on one split at one seed, against that split's own references after the same prompts.
- **Loops:** the replies that repeat a 4-gram of their own before their end of turn (`looping_replies`).
- **Fewer loops:** the arm's loops plus decision 4's band, `⌈2·√(n·p(1−p))⌉` at greedy's count, are fewer than greedy's loops (`fewer_loops`).
- **Holds:** decision 4 against greedy (`decode_holds`). The repeated-4-gram rate is below greedy's, and complete ≥ greedy's − the band.
- **Close:** "generalizes"' free-generation condition (`VerdictSeed::generalizes`):
  - complete ≥ 64;
  - `|v − v_ref| ≤ tol`, where `v` is the verbatim TRAIN share and `tol = max(0.10, 3·√(v_ref(1−v_ref)/n_c))`;
  - a repeat rate at most the references' + 0.10.

- **Content kept:** the arm's `|v − v_ref|` is no wider than greedy's on the same prompts by more than the arm's `tol` (`content_kept`).
- **The power floor:** greedy loops in at least 40 of the 256 p-r replies, at every seed (`READ_LOOP_FLOOR`). Below that, no arm can show fewer loops beyond the band.

**p-r decides,** from the `all` arm at all three seeds (`read_outcome`). The rows are tried in this order:

| Outcome | Condition | What it unlocks |
|---|---|---|
| **underpowered** | Greedy loops in fewer than 40 p-r replies at any seed. | Nothing is decided, and nothing is closed. It is an instrument outcome: the panel shows too few loops to measure a reduction. The next run needs a panel, or a reply length, where greedy loops. |
| **cures** | At all three seeds: fewer loops, holds, and close. | The prompt-held read becomes the one model's decode. The next unit serves it in integers: the integer Hamming read with the same cut. |
| **reduces** | At all three seeds: fewer loops, holds, and content kept. | Reading the reply's own tokens is a cause of the loops, and the prompt-held read joins the decode. The content gap that remains is next. The first candidate is training with the reply read held to the prompt, so that training matches serving. |
| **reduces, content lost** | At all three seeds: fewer loops and holds, but content not kept at some seed. | No decode change and no causal claim: the arm may only have pushed the model off what it was trained on. It unlocks one follow-up only, training with the same cut, so that the trained model meets the cut. |
| **no gain** | Otherwise. | A negative for this cause, in this configuration only. The next candidates are the turn memory's reply table, or the trunk itself. |

**Reported beside it, deciding nothing:**
- **The copy arm's outcome, by the same rule.** If copy matches all, the copy heads alone can carry the effect. If only all reduces, either the type heads take part, or the larger cut simply perturbs the model more. With no type-only arm, the REPORT says no more than that.
- **Each arm's mean reply length** to its end of turn, beside its loop count: a reply that ends sooner has fewer 4-grams to repeat.
- **A paired loop count,** computed from the sealed samples for the REPORT: the replies that loop under greedy but not under the arm, and the reverse.
- **The DEV readings.**
- **The decode tok/s per arm.** It is measured with three seeds sharing the GPU, so it is a floor.
- **A fixed reading** of the first 20 p-r replies of seed 1 under greedy, copy and all.

## Minimum test power

- **The sample:** 256 replies per arm, split and seed, the same prompts under every arm.
- **The loop band:** greedy looped in 68–93 of 256 p-c replies (claude-213's samples, recounted by `read_grade` on stand-in files).
  - At 68, the band is ⌈2·√(256·0.266·0.734)⌉ = 15, so "fewer loops" needs 52 or fewer.
  - At 93, the band is 16, so it needs 76 or fewer.
  - The band is an unpaired 2σ on greedy's count, which is conservative for a paired design.
- **The floor:** p-r has never been decoded, so its greedy loop count is not known in advance. The 40-reply floor turns a quiet panel into "underpowered", never "no gain". At 40, the band is 12, so the arm needs 27 or fewer.
- **The verbatim tolerance:** 0.10 at `v_ref` ≈ 0.6 and about 256 complete replies.
- **The seeds:** every condition needs all three.

## Stop rule

- **The pod stops** on any failed CUDA test, data check or probe projection, with its own EXIT code.
- **EXIT 6 and EXIT 8 are instrument faults:** a reproduction miss, or a scope that changed nothing. Each is classified and repaired, never read as a result.
- **The budget is $5.** The laptop watcher (`local/claude/claude-215/watch.sh`, claude-213's, reviewed) prices the pod over its time up.
  - **A healthy run is not cancelled** for a budget it set itself.
  - **An unhealthy run** is sealed and taken down. Unhealthy means a process exited non-zero, or 30 minutes passed with no new log line and no new file.
- **A small miss on a bar** is "keep and iterate", not a stop. That is the reduces row.

## Cost

- **The decodes:** per seed, 10 decodes of 256 replies: DEV and p-r × (greedy with the memory on and off, `FLOCK_END`, copy, all).
- **The time:** claude-213's 18 decodes per seed took about 35 minutes with three seeds side by side, so this run takes about 20–25 minutes, plus about 3 minutes of build and tests.
- **The lease:** one RTX 5090 at about $0.56/h, so under $1.
  - **The caps:** `uor-pod status` (30 Oct, 08:40Z) showed 0 pods running across the labs, $0/h of $8/h. This is inside the caps.
