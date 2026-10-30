# REPORT: one-model-end-to-end, claude-213 (Flock rank inside the nearest token's cell)

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md). Order: [ORDER.md](ORDER.md). Sheet: [one-model-end-to-end-flock-cell-2026-10-30](../../../../harness/prep/one-model-end-to-end-flock-cell-2026-10-30.md), FIT and pre-run PASS from fresh reviews. Results: [flock-cell-2026-10-30](../../../../labs/one-model-end-to-end/flock-cell-2026-10-30/README.md). References #3407.

## Tokens per second

- **Every rule decodes at 58–60 tok/s per process** on one RTX 5090, with three processes sharing it, about 180 tok/s together. This is the float GPU path with the memory on.
- **The cell rule adds no cost over `FLOCK_END`,** which reads the same top-16 sort.
- **This is not the served speed.** The integer path for (r-drop) is not built.

## What did not work, first

- **No margin of the ladder is close, at any seed, on DEV or p-c.** "Close" is the free-generation condition: a verbatim share within `tol` of the references', and a repeat rate within 0.10 of theirs.
  - **The registered "improves" outcome is not met.** The score-margin cell at these six margins is a negative for this configuration only: (r-drop) at step 16,384, top-16, harmonic weights.
- **Narrowing the draw trades salad for loops; it does not remove both.** On p-c, the verbatim share `v` and the repeated-4-gram rate rise together, from margin 1.0 (`v` 0.016–0.035, repeat 0.001) to 0.031 (`v` 0.343–0.353, repeat 0.553–0.614, greedy's level). The references sit at `v` 0.59 and repeat 0.00.
- **The objective's own margin is no narrowing.** At 1.0, the margin the run trained to, 15.1–15.2 of the 16 candidates are admitted, and 195–209 of 256 p-c replies are identical to `FLOCK_END`'s.
- **The loops are not near-ties.** With 2–3 candidates admitted (margin 0.031), the repeat rate equals greedy's. Greedy's loops sit at steps where the nearest token is already well ahead, so no tie-breaking rule reaches them.
- **The selected margin still gives mostly salad.** At 0.25, 59–101 of 256 p-c replies are character for character `FLOCK_END`'s. In the fixed reading of seed 1's first 20 p-c replies:
  - nine are `FLOCK_END`'s salad unchanged;
  - two loop;
  - two become the reference's kind of reply: "Okay, I'll remember that." and "That's great news about the bike!".

## What changed

- **`DecodeRule::FlockCell { k, margin_milli }`** (`crates/uor-r4-float-train/src/in_space_long.rs`).
  - **The rule:** of the `k` nearest, it draws the harmonic rank only among those within the margin of the nearest's score (`cell_admitted`, edge included, never fewer than one). It keeps a nearest `<|eos|>`.
  - **The record:** each step's admitted count is kept in `IdSample.admitted`, which is not written for any other rule, so their files are byte for byte as before.
  - **The constants:** `CELL_MARGIN_MILLI` (1,000, `rival_hinge`'s margin) and `FLOCK_CELL_LADDER_MILLI`.
- **`--flock-cell M,...`** adds each margin's samples to `--score-rows`. Each samples file's stdout line now carries its decode tokens and seconds.
- **`--cell-grade 1`** grades on the laptop: `cell_grade`, `select_margin` and `panel_outcome`, from stored samples only.
- **The panel p-c,** drawn by `heldout-panel` at seed 2061, and its census ([census-p-c-2026-10-30.json](../../../../labs/one-model-end-to-end/census-p-c-2026-10-30.json)):
  - 0.000 of its 1,430 prompts are in TRAIN, exactly or as templates at K = 1,024;
  - 0.644 of its replies are verbatim TRAIN replies.

  It is stored at `inputs/claude/heldout-panel-p-c-2026-10-30/`.
- **The prep sheet,** the grade ([cell-grade.json](../../../../labs/one-model-end-to-end/flock-cell-2026-10-30/cell-grade.json)), the lab page and one notebook row.

## Measurements

- **The registered outcome** (`cell-grade.json`): DEV selects margin **0.25** ("closest, not qualified"). p-c's outcome is **narrows**: at all three seeds, 0.25 holds decision 4 against greedy and is nearer the references than `FLOCK_END`.
  - **Holds:** complete 229–251 of 256 against greedy's 192–202, and a repeat rate of 0.076–0.332 against greedy's 0.559–0.628.
  - **Nearer:** `v` 0.116–0.184 against `FLOCK_END`'s 0.000–0.008.
  - **How to read it:** as the sheet registered, narrows means "holds, not close".
- **The ladder's full p-c table, and DEV's,** are on the [lab page](../../../../labs/one-model-end-to-end/flock-cell-2026-10-30/README.md). Mean admitted counts by margin, from 1.0 down to 0.031: 15.2, 14.0, 10.7, 7.4, 4.8 and 2.7 (seed means on p-c).
- **The run:** one RTX 5090 (`vast-58366166`), up about 42 minutes, about $0.40. Sealed at `results/claude/one-model-end-to-end-flock-cell-2026-10-30/run-c9fab457/` (120 files, read back OK). The pod is down.

## Checks run at the head

- **The crate's host tests** (`cargo test --locked -p uor-r4-float-train`): 160 lib tests pass, among them the three new ones (the admitted count, the files left byte for byte, the selection and outcome).
- **The four negative gates** (the margin's edge, the empty-record skip, widest first, "improves" needing close): each test fails with its line broken and passes with it restored.
- **The stub-nvcc CUDA check** (`--features cuda --all-targets`): it compiles.
- **On the pod:**
  - 222 CUDA tests pass, among them `the_cell_rule_at_margin_0_decodes_as_greedy_on_a_model` and claude-211's K = 1 test;
  - every input matches its sha256;
  - the binary reproduces claude-211's 15 registered files byte for byte, with three seeds side by side.
- **`--cell-grade 1`** on the sealed run, at `c9fab4576`: exit 0, in 38 s.

## Skills used

None.

## Still open

- **The next narrowing (the registered "narrows" row):** the Hamming read's address cell in place of a score margin.
  - **Its design input:** the loops sit at steps where the nearest is confident, so the rule must act there too, not only on near-ties.
- **No decode rule tried yet meets the free-generation condition** on new phrasings. Greedy loops, `FLOCK_END` is salad, and the cell rule trades one for the other. No rule yet makes replies both complete and TRAIN-like.
- **Serving (r-drop) in integers,** then its decode tok/s.
- **p-c has now been opened once,** by this decision. A later decision needs a new panel.

## Plain summary

Casey asked to improve Flock's decode rather than drop it. I built a version that only draws among the tokens the model can't tell apart from its top pick, measured at six widths on three seeds and a fresh panel.

It helps a little: at the selected width, replies finish and some become sensible. But mostly it trades word salad for greedy's loops, and no width gives both clean and on-topic replies. The loops come from confident steps, not near-ties, so the next attempt has to act on those steps.
