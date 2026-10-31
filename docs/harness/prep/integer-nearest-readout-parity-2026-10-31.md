# Prep sheet: end-to-end token parity of the integer artifact against the trained model (integer-nearest-readout done-when 3, claude-232), 2026-10-31

- Item: [integer-nearest-readout](../../roadmap/items/integer-nearest-readout.md) (#3462). ORDER: [claude-232](../../loop/tasks/integer-nearest-readout/claude-232/ORDER.md).
- **The run, all on the laptop (Apple M1):**
  - the integer decodes are product serving (CHARTER §3.6), under the owner's 31 Oct allowance for this chain;
  - the float side is claude-220's stored GPU replies, read as stored outputs.
  - No GPU is leased, and no code changes.

## 1. Question

**Does the integer artifact of (r-drop), with the nearest readout (claude-231), decode as the trained float model does?**
- claude-229's `/7` served the native writer. Its first token equalled the float model's on 0 of 310 DEV prompts.
- This compares the whole served model: trunk, Hamming read, turn memory and readout. claude-227 checked the trunk alone.

## 2. Arms

- **I, integer greedy:** (r-drop) step 16,384 at seeds 1–3.
  - Exported by `fbs-export --run-config CONFIG SNAPSHOT A7n-sN` from claude-227's full snapshots (`results/claude/integer-retention-rates-parity-2026-10-31/out/full/`), which carry the memory and the Hamming read. The run configs are `results/claude/one-model-end-to-end-2026-10-29/training-460fe46e/sN/r-drop-sN/config.json`.
  - Decoded by `fbs-toks A7n-sN --replies PROMPTS --out FILE --reply-tokens 64` (`panel_reply`): memory on, the reply start marked, then greedy.
  - The integer session cannot slide its window, so a reply also ends where prompt and reply fill the 192-token context.
- **F, float greedy (stored):** claude-220's `samples-memory.json` (DEV) and `samples-p-w-greedy-memory.json` (p-w), for each seed, in `results/claude/one-model-end-to-end-served-key-2026-10-31/run-eb762e60/out/r-drop-sN/`. Each reply is 64 tokens.
- **The prompts:** `train-in-space --panel-prompts 1 --runs <claude-220's out> --samples 1536 --reply-tokens 64 --panel p-w=...`, at each seed (`prompts-dev-sN.json`, `prompts-p-w-sN.json`). These are F's own prompts, which `panel_prompts` checks and claude-224 confirmed.

## 3. Bar and decision table

For each cell (split ∈ {DEV, p-w}, seed ∈ {1, 2, 3}), I and F are paired by index; their `position`s must match (§4). Then:
- **First-token agreement:** I's `reply[0]` equals F's `reply[0]`. An empty I reply, one that ended at BOS, disagrees.
- **Whole-reply agreement (reported, no bar):** I's reply is non-empty and equals F's reply cut to I's length. An early difference cascades, and I's reply may be capped.

The registered grader, run per cell on the stored files (`I` and `F` are the two JSON paths):

```text
jq -n --slurpfile i I --slurpfile f F '($i[0]|length) as $n
  | if ($f[0]|length) != $n then error("counts differ") else . end
  | [range(0; $n) as $k | $i[0][$k] as $a | $f[0][$k] as $b
     | {p: ($a.position == $b.position), first: (($a.reply|length) > 0 and $a.reply[0] == $b.reply[0]),
        whole: (($a.reply|length) > 0 and $a.reply == $b.reply[0:($a.reply|length)])}]
  | {n: $n, positions_match: ([.[]|select(.p)]|length), first_agree: ([.[]|select(.first)]|length),
     whole_agree: ([.[]|select(.whole)]|length)}'
```

| Row | Reading | Decides | Next |
|---|---|---|---|
| 0 | **An instrument fault** (§4) | Nothing is counted. | Repair, then rerun. |
| PASS | In **every** cell, first-token disagreement (n − `first_agree`) is at most ⌊0.03 n⌋: 9 of 310 on DEV, 36 of 1,202 on p-w | Done-when 3 is met; the item is done with done-when 5's records. | point-key-integer's rerun on p-x, with `I-Greedy-Parity`. |
| MISS | Any cell over | The item stays `ready`, and the row is recorded. It is an instrument fault for point-key-integer, never a mechanism verdict. | The next unit locates the gap on the first disagreeing prompts, layer by layer (trunk, Hamming read, memory, readout). |

**Why 3%:** it is the bar claude-227 registered for this exporter, twice claude-126's clean residual, and claude-227's trunk-alone run read 0.33% of sequence positions and 1.78% of single tokens. A first token here follows up to 186 positions, with the memory and the Hamming read on top, so a residual near 1–2% is expected even when correct.

**Readings, no bar:**
- whole-reply agreement per cell;
- each arm's first token equal to the reference's first token;
- the capped count.

## 4. Instrument checks (any failure is row 0)

1. **`I-Float-Inputs`:** the six F files on the laptop equal the store's, by claude-220's `run-eb762e60/SHA256SUMS`.
2. **`I-Head`:** each `/7`'s header carries `head` `native_vector_nearest` and `rec_gates` `geometric-retention`. `--run-config`'s `retain_rates` check passes.
3. **`I-Identity`:** each artifact's printed sha256 equals its file's.
4. **`I-Prompts`:** 310 DEV and 1,202 p-w prompts at every seed; `panel_prompts` refuses no float name.
5. **`I-Positions`:** `positions_match` = n in every cell.
6. **`I-Decode`:** every `fbs-toks` exits 0. Its `keyed` is 0, since `/7` carries no point table.

**The grader's known-answer probes, run before this sheet's arm,** on stored files whose answers are already known:
- **The negative:** claude-229's sealed writer decode (DEV s1, `results/claude/point-key-integer-run-2026-10-31/R/r-drop-s1/integer-greedy-dev.json`) against F reads n 310, `positions_match` 310, `first_agree` 0 and `whole_agree` 0. That is claude-229's 0 of 310.
- **The positive:** F against itself reads 310, 310, 310 and 310.

So the grader can read both ends.

## 5. Base artifact and data split

- **Base:** (r-drop) step 16,384, seeds 1–3: claude-227's full snapshots (`dcbfb69b…`, `d0522d02…`, `9744d385…`), with their run configs (`objective: "cell"`, `retain_rates: true`).
- **Data split:** DEV (`ft/dev`) and p-w (`inputs/claude/heldout-panel-p-w1200-2026-10-31/`), the prompts claude-220 scored. Both are opened panels, read here only against F; neither decides a mechanism. TRAIN is not read, and p-x is not touched.

## 6. Steps

1. Fetch the snapshots, the configs and claude-220's `SHA256SUMS`, and check `I-Float-Inputs`.
2. `fbs-export --run-config` per seed (`I-Head`, `I-Identity`).
3. `--panel-prompts` (`I-Prompts`).
4. The six decodes, one after another (`I-Decode`).
5. The grader per cell (`I-Positions`, then the bar).
6. Seal the outputs to `results/claude/integer-nearest-readout-parity-2026-10-31/` with `SHA256SUMS`, and list them back.

## 7. Minimum test power

- 310 and 1,202 prompts per seed, at three seeds: 4,536 first tokens.
- At a true residual of 1%, a cell expects about 3 (DEV) and 12 (p-w) disagreements, against bars of 9 and 36.
- A writer-like fault is unmistakable either way: claude-229's was 310 of 310.

## 8. Cost and stop rule

- **Cost:** the laptop only. Six decodes at about 1,365 tok/s take a few minutes; the exports take seconds.
- **Stop rule:** any §4 check that fails stops the run as row 0, never counted. The bar is read only after every check holds.

## 9. Non-author TEST FITNESS review

TEST FITNESS: PENDING

## 10. Pre-run review

PENDING
