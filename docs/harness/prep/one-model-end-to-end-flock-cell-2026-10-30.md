# Prep sheet: Flock rank inside the nearest token's cell (one-model-end-to-end, claude-213, 30 Oct 2026)

Lab: claude. Item: [one-model-end-to-end](../../roadmap/items/one-model-end-to-end.md). Order: [claude-213](../../loop/tasks/one-model-end-to-end/claude-213/ORDER.md). Direction: Casey, 30 Oct, "do not rule flock out, improve it instead".

TEST FITNESS: PENDING (a fresh non-author FIT is run on this sheet before any pod).

## The question

Does drawing Flock's harmonic rank only among the tokens the model has not separated from the nearest give replies that are neither loops nor word salad, on prompts whose phrasing TRAIN does not hold?

**What this run measures, from claude-211's grade on P-dev at seeds 1–3 of (r-drop):**

| Rule | Complete, of 256 | Verbatim TRAIN share `v` (references' `v_ref`: 0.566) | Repeated-4-gram rate |
|---|---|---|---|
| Greedy | 186–206 | 0.354–0.484 | 0.563–0.636: it loops |
| `FLOCK_END` | 255–256 | 0.004–0.012: word salad | 0.0006–0.0020 |

## Base artifact: the model and the rules

- **The model:** (r-drop)'s sealed step-16,384 checkpoints at seeds 1, 2 and 3, the recipe the training run ships. They are at `results/claude/one-model-end-to-end-2026-10-29/training-460fe46e/s{S}/r-drop-s{S}/`, with each file checked against that seed's `SHA256SUMS`. `model.safetensors` sha256s:
  - **s1:** `c5fcbb90ab49c284da56abc6e74e74a4d8f1bded2d7815f8fb70de8332c81465`;
  - **s2:** `c8b9b420c4ab5d56cc0dfc4f3cfd68a3aaa06cec7024c5484d6311ff61764f2b`;
  - **s3:** `8727c8a065840811f4e63b74b60ccc8b65d3962bd5394a1c64f17b74b1734e85`.

  Nothing is trained.
- **The rules,** each with the memory on, as served. Each draws with `flock_rank_draw(seed, sample·64 + step, n)`, the harmonic weights `1/(i+1)` over `n` ranks, and no exponential.
  - **Greedy:** the whole table's nearest token.
  - **`FLOCK_END`** (claude-211): a rank drawn over the 16 nearest, and a nearest `<|eos|>` always taken.
  - **`FlockCell { k: 16, margin_milli: m }`** (this unit, `in_space_long.rs`):
    - of the 16 nearest, it admits those whose score (negative mean L1 displacement) is within `m / 1000` of the nearest's (`cell_admitted`);
    - it draws among the `n` admitted;
    - it keeps a nearest `<|eos|>`;
    - it records each step's `n`.
- **The ladder:** `m` ∈ {1000, 500, 250, 125, 62, 31}.
  - **1000 is the cell objective's own margin.** The run trained to `relu(1 + max_r s(r) − s(target))` (`rival_hinge`; `--objective cell` in the training sheet), so a rival nearer than 1.0 is one the model has not separated.
  - **It may admit all 16:** the run's train loss is 0.56 against that margin. The halvings trace the path from `FLOCK_END` toward greedy, and margin 0 would be greedy itself (a GPU test pins this).

## Data split

- **TRAIN, DEV and the tokenizer:** `w2-inputs/2026-10-25/driver-data/`, pinned by the code.
- **DEV: selection only.** It was opened before, for greedy and `FLOCK_END`. Its 256 sample prompts (`sample_windows`) are the same ones claude-211 decoded.
- **p-c: confirmation only.** A new held-out-phrasing panel, drawn on 30 Oct with claude-204's `heldout-panel` at seed 2061, a seed no unit had used:

  `heldout-panel out=p-c cell=dev,dev conversations=400 seed=2061 protocol=2`

  - **The panel:** 51,128 tokens and 15,968 reply targets. `tokens.u16` sha256 `98708797a79bf4ee5460daa65e4f35dede53c5a230aae6106a706cbfa32acce8`, `response_mask.u8` `314f0bac39bdb85443c190f7fb3b533fd382cd1849f6df94a6ea5c904d8073b2`. A second draw is byte-identical. It is stored at `inputs/claude/heldout-panel-p-c-2026-10-30/`, with a `SHA256SUMS` and the census.
  - **The census** ([census-p-c-2026-10-30.json](../../labs/one-model-end-to-end/census-p-c-2026-10-30.json), `train-in-space --census 1`, on the laptop):
    - 0.000 of its 1,430 prompts are in TRAIN exactly;
    - 0.000 as value-masked templates at K = 1,024 (0.041 at K = 256), so it is new-template;
    - 0.644 of its replies are verbatim TRAIN replies, as on p-f (0.625) and P-dev.
  - **Its rows** (`rows-p-c-*.json`, which `--score-rows` writes) are read by no decision here.
- **P-dev and p-f are not used.** P-dev was opened by the training run; p-f is bar (f)'s alone.

## Steps

One pod, one RTX 5090. The job is `local/claude/claude-213/job.sh` in this unit's worktree, run as `job.sh <head>`. Its sha256 is in the pre-run review.

1. **The source,** pinned to this unit's pushed head and clean.
2. **The probe ladder's (a), the mechanism on a model:** the crate's CUDA tests, `--release --locked`. The job stops unless both of these pass:
   - `the_cell_rule_at_margin_0_decodes_as_greedy_on_a_model`: margin 0 decodes greedy's reply token for token and admits one at every step, and the objective's margin admits more than one somewhere;
   - claude-211's `the_flock_rank_rule_at_k1_decodes_as_greedy_on_a_model`.
3. **The build,** `train-in-space`, kept with its sha256.
4. **The data,** every file checked: the panel against its `SHA256SUMS` and its pinned `tokens.u16` sha, and each seed's 11 files against that seed's sealed `SHA256SUMS`. The 11 files are the run config, the 5 checkpoint files, and the 5 registered files of step 7.
5. **The probe ladder's (b), throughput:** seed 1 at `--samples 8`, timed and projected ×32. Past 2 hours per seed, the job stops for a code fix.
6. **The run:** the three seeds side by side on the one GPU, each:

   `train-in-space --score-rows CKPT --out OUT/r-drop-sS --panel p-c=PC --samples 256 --reply-tokens 64 --retain-rates 1 --seed S --flock-end 1 --flock-cell 1000,500,250,125,62,31`

   This writes, on DEV and p-c, greedy (memory on and off), `FLOCK_END`, and the six cell margins (memory on), 256 replies each. Each stdout line carries the decode's reply tokens and wall seconds.
7. **The instrument check:** at every seed, this binary reproduces claude-211's registered files byte for byte. Those are `rows-dev-memory.json`, `rows-dev-no-memory.json`, `samples-memory.json` and `samples-no-memory.json` (from `score-r-drop/`), and `samples-dev-flock-end-memory.json` (from `flock-end-r-drop/`), so 15 files.
   - A miss is an instrument fault, never a result (CHARTER §4.1): it is repaired and the run repeated.
8. **The seal:** everything under `/workspace/flock-cell` goes off the pod, then into the store at `results/claude/one-model-end-to-end-flock-cell-2026-10-30/run-<sha8>/`, with a `SHA256SUMS`, read back before the pod is released and taken down.
9. **The grade, on the laptop:**

   `train-in-space --data-root DATA --cell-grade 1 --runs OUT --panel p-c=PC --flock-cell 1000,500,250,125,62,31`

   It is `cell_grade`, unit-tested with negative gates.

## Bar

Every reading is `VerdictSeed` (claude-211's decision 5), against the split's own references after the same 256 prompts. At each seed:
- **Holds:** decision 4 against greedy on those prompts: the repeated-4-gram rate is below greedy's, and complete ≥ greedy's − `⌈2·√(n·p(1−p))⌉` (`decode_holds`).
- **Close:** "generalizes"' free-generation condition: complete ≥ 64, `|v − v_ref| ≤ tol` with `tol = max(0.10, 3·√(v_ref(1−v_ref)/n_c))`, and a repeat rate at most the references' + 0.10.

**DEV selects the margin** (`select_margin`), from the six margins' three DEV seeds:
1. the widest margin that holds and is close at all three seeds: **qualified**;
2. else, of those that hold at all three, the one with the least mean `|v − v_ref|` over its seeds, ties to the wider: **closest, not qualified**;
3. else none: **no selection**.

**p-c decides** (`panel_outcome`), for the selected margin `m*` against `FLOCK_END` on p-c:

| Outcome | Condition, at all three seeds | What it unlocks |
|---|---|---|
| **improves** | `m*` holds and is close | `FlockCell(m*)` replaces `FLOCK_END` as the one model's Flock decode. The next unit serves it in integers: the Hamming read's ranking with the margin in its units. |
| **narrows** | `m*` holds, and its `|v − v_ref|` is below `FLOCK_END`'s | The cell rule stays the Flock candidate and is iterated. The next narrowing comes from the same geometry: the Hamming read's address cell in place of a score margin. |
| **no gain** | otherwise | The score-margin cell is a negative for this configuration only. `FLOCK_END` stays, and the next unit tries the address cell. |
| **no selection** | no margin holds on DEV | As no gain, recorded as "no margin of the ladder holds decision 4 on DEV". |

**Reported beside the decision, deciding nothing:**
- every margin's p-c readings;
- each cell margin's mean admitted count, which shows the score scale;
- greedy's and `FLOCK_END`'s p-c readings;
- each rule's decode tok/s;
- a fixed reading of the first 20 p-c replies of seed 1 under greedy, `FLOCK_END` and `FlockCell(m*)`, quoted in the REPORT.

## Minimum test power

- **The replies:** 256 per rule, split and seed, the same 256 prompts under every rule.
- **The verbatim tolerance:** at `v_ref` ≈ 0.6 and ~256 complete, `tol` is 0.10 (3·√(0.24/256) = 0.092 is below the floor). `FLOCK_END`'s P-dev gap of 0.55 is 5.5 tolerances; greedy's 0.08–0.21 is 0.8–2.
- **The completion band:** at n = 256 and greedy's ~0.75 complete, 14 replies.
- **Every condition needs all three seeds,** so a single lucky seed decides nothing.

## Stop rule

- **The pod stops** on any failed CUDA test, data check or probe projection, with its own EXIT code.
- **A reproduction miss** (EXIT 6) is an instrument fault, repaired and re-run, never read as a result.
- **The budget is $5.** The laptop watcher (`local/claude/claude-213/watch.sh`) prices the pod over its time up.
  - **A healthy run is not cancelled** for a budget it set itself: it is extended and noted on #3407.
  - **An unhealthy run** is sealed and taken down. Unhealthy means a process exited non-zero, or the log stopped for 30 minutes.
- **A small miss on a bar** is "keep and iterate", not a stop. That is what the narrows row is.

## Cost

- **Per seed:** 18 decodes of 256 replies: DEV and p-c × (greedy with the memory on and off, `FLOCK_END`, six margins). claude-211's (r-drop) scoring ran 6 decodes and the rows in 7 minutes, so about 20 minutes per seed.
- **In all:** three seeds side by side on one GPU (small, latency-bound kernels), so about 30–45 minutes with the build and tests.
- **The lease:** one RTX 5090 at about $0.56/h, so under $1.
  - `uor-pod status` (30 Oct, 03:00Z): 0 pods running across the labs, $0/h of $8/h. This is inside the caps.
