# REPORT: one-model-end-to-end, claude-215 (do the copy heads make the reply loop?)

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md). Order: [ORDER.md](ORDER.md). Sheet: [one-model-end-to-end-reply-scope-2026-10-30](../../../../harness/prep/one-model-end-to-end-reply-scope-2026-10-30.md), FIT in round 2 and pre-run PASS from fresh reviews. Results: [reply-scope-2026-10-30](../../../../labs/one-model-end-to-end/reply-scope-2026-10-30/README.md). References #3407.

## Tokens per second

- **Every arm decodes at 28–41 tok/s per process** on one RTX 5090, while all three seeds share it. This pod was slower than claude-213's (an 8-sample probe took 93 s against 25 s). The float GPU path has the memory on.
- **The read's scope costs nothing:** `copy` and `all` decode as fast as greedy at the same seed.
- **This is not the served speed.** The integer path is not built.

## What did not work, first

- **The registered outcome is "reduces, content lost"**, for `all` (the decision) and for `copy` (reported). So the decode does not change, and no causal claim is made.
  - **What failed:** content kept, at seed 2. Greedy's verbatim share is 0.511 there, near the references' 0.563; `all`'s is 0.386, a gap of 0.177 against greedy's 0.052 + `tol` 0.100.
  - **Seeds 1 and 3** keep the content within `tol`.
- **The loops are interrupted, and often not replaced by better text.**
  - In the fixed reading of seed 1's first 20 p-r replies, a loop often ends in salad: "A friendundrot hear aboutdant: A friendunday."; "… all all all all soundededay.".
  - Replies 2, 11 and 12 still loop under every arm.
- **Replies end sooner.** Mean length is 15.1–18.4 tokens under the scoped arms, against greedy's 21.5–25.8 (the references': 12.8). Part of the drop in loops is the ending itself.
- **"Content lost" may be a selection effect the sheet did not control.** `v` counts only complete replies, and greedy completes the fewest: 188 at seed 2, against 246 for `all`.
  - **Matched check:** on the prompts both complete, exact matches to the reference are the same for greedy and the arm (5 and 5, 3 and 3, 7 and 7).
  - **That decides nothing:** the counts are too small, and the registered outcome stands.
- **On DEV the loops fall by less.** Greedy loops in 46, 28 and 29 replies, `all` in 31, 21 and 24. Only seed 1 is beyond the band.

## What changed

- **`ReadScope { Full, Copy, All }`** (`crates/uor-r4-float-train/src/flock_read_hamming.rs`).
  - **`set_read_cut`** sets a decode-time cut. The thread-local is `None` by default, and is cleared before any `?`.
  - **`choose_sorted`** takes an explicit `limit`, through `is_candidate`. `head_limit` limits only the scope's heads: the first `copy_heads()` heads under `Copy`, every head under `All`.
- **`decode_reply`** takes the scope, and sets the cut at `prompt_cut(prompt.len(), start)` around each step's forward pass (`in_space_long.rs`).
  - **`score_rows`** decodes greedy under each `--reply-read` scope (`samples-{split}-greedy-read-{scope}-memory.json`).
- **The grade, `--read-grade 1`:**
  - **`read_grade`** counts `looping_replies` and `fewer_loops` (beyond decision 4's band).
  - **`content_kept`** and **`READ_LOOP_FLOOR`** (40) feed `read_outcome`'s five rows.
  - **Each reading** carries its mean reply length.
- **The shared helper:** `cell_grade`'s reading loop is now `rule_readings`, shared with `read_grade`. claude-213's grade reproduces byte for byte.
- **The panel p-r,** drawn by `heldout-panel` at seed 2071, and its census: 0.000 of 1,453 prompts are in TRAIN, exactly or as templates at K = 1,024; 0.632 of its replies are verbatim. It is stored at `inputs/claude/heldout-panel-p-r-2026-10-30/`.
- **Also written:** the prep sheet, the grade ([read-grade.json](../../../../labs/one-model-end-to-end/reply-scope-2026-10-30/read-grade.json)), the lab page and one notebook row.

## Measurements

- **On p-r, at seeds 1, 2 and 3, `all` against greedy:**
  - **Looping replies:** 31, 39 and 68 against 63, 70 and 92, fewer beyond the band at every seed. Paired, 34, 33 and 39 replies stop looping, and 2, 2 and 15 start.
  - **Complete:** 236, 246 and 240, against 213, 188 and 203.
  - **Repeated-4-gram rate:** 0.293, 0.208 and 0.272, against 0.526, 0.633 and 0.566.
  - **`v`:** 0.347, 0.386 and 0.317, against 0.385, 0.511 and 0.374 (`v_ref` 0.563).
- **`copy`:** 29, 50 and 62 looping replies; complete 239, 240 and 239; repeat 0.258, 0.271 and 0.268.
- **The power floor is met:** greedy loops in at least 40 replies at every seed.
- **The run:** one RTX 5090 (`vast-58379979`), up about 80 minutes, about $1.30.
  - **Sealed** at `results/claude/one-model-end-to-end-reply-scope-2026-10-30/run-4b0b8955/`: 88 files, read back OK.
  - **The pod is down.**

## Checks run at the head

- **The crate's host tests** (`cargo test --locked -p uor-r4-float-train`): 164 lib tests pass, among them seven new ones (the read's limits, the candidate cut, the loop count and outcome, the cut's coordinate) and claude-213's.
- **Eight negative gates** each fail with their line broken and pass when restored:
  - the copy heads only, the cut, the loops cut at the end of turn;
  - the outcome needing decision 4, the loop band;
  - the floor's edge, content kept, the cut's coordinate.
- **The refactor:** `--cell-grade` on claude-213's sealed run gives its committed grade byte for byte.
- **The stub-nvcc CUDA check** (`--features cuda --all-targets`): it compiles.
- **On the pod:**
  - 228 CUDA tests pass;
  - every input matches its sha256;
  - the 15 registered files reproduce byte for byte;
  - all 6 scoped files differ from greedy's;
  - every scoped reply's first token is greedy's.
- **`--read-grade 1`** on the sealed run, at `4b0b8955d`: exit 0.

## Skills used

None.

## Still open

- **The registered follow-up:** train with the same cut, so that a reply's Hamming read is held to the prompt in training as in serving. Then measure on a new panel.
- **The content measure:** `v` over complete replies is biased toward the prompts an arm completes. A matched content measure would give the next decision a cleaner reading: the verbatim TRAIN share on the prompts that every arm completes.
- **The loops that survive the cut** (replies 2, 11 and 12) come from elsewhere: the turn memory's reply table, keyed by the reply's last id, or the trunk itself.
- **Serving (r-drop) in integers,** then its decode tok/s.
- **p-r has now been opened once,** by this decision.

## Plain summary

The model's replies get stuck repeating themselves. I tested whether that is because its copy mechanism looks back at the reply it is writing, by letting it look back only at the prompt.

That removes about half of the loops on new phrasings, at all three seeds, and more replies finish. But the loops are often replaced by garbled endings, and at one seed the replies were less like the training replies. So, as registered, the decoding stays as it is. The next step is to train the model the same way it will be decoded, so it learns to reply without reading its own reply.
