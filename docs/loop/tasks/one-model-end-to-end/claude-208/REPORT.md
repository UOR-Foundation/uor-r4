# REPORT: one-model-end-to-end, claude-208 (the protocol's scoring instruments)

Lab: claude. Order: [ORDER.md](ORDER.md). Protocol: [one-model-end-to-end-protocol-2026-10-29](../../../../harness/prep/one-model-end-to-end-protocol-2026-10-29.md), instruments I-A, I-E and I-F. References #3407; the item stays `ready`.

**Result: the three scoring instruments exist, and their host-side rules are tested.**
- **I-A:** `--score-rows` scores and samples an unpinned panel, so P-dev can be scored.
- **I-E:** replies can be decoded by the Flock rank rule, the 16 nearest tokens drawn with the integer Flock selector's harmonic weights, as well as greedily.
- **I-F:** every sample records how often the memory's reply table names the key its later steps read, under generation and under teacher forcing.

**Speed.** This unit times nothing. The Flock samples add decode time to each scored arm, which the run's throughput check prices.

## What did not work, first

- **The GPU halves of the checks have not run.** On the run's pod, before any arm:
  - **I-A:** P-dev scores its census's 16,417 rows.
  - **I-E:** on a model, K = 1 decodes as greedy. The unit test checks the draw itself.
  - **I-F, on the laptop, not the pod:** over every one of DEV's 13,061 later rows in its own window, the decoder's key must equal the census's, and the reply table must name exactly 9,655. That needs TRAIN's tables, which the grader unit (I-G) builds, so it runs there. The sampled replies cannot give that count: they are a few prompts' replies, cut at their length and their window.

  `decode_reply`'s GPU path (the top-16 sort, the gather) is not executed on the laptop.
- **The protocol's I-F check was wrong, and is corrected in the sheet.** It said "the 10,849 named rows". That is DEV's 1,194 keyed openings plus its 9,655 named later rows. The hit rate concerns the reply table, whose key includes the reply's own last token, and that is what free generation can change. An opening's key is the prompt's alone, and decoding cannot change it. So I-F counts later steps, and the sheet's I-F row now reads 9,655, with the reason. This is an instrument check, not a bar.
- **In the memory-off sample files,** `reply_named` and `reference_named` are 0 by construction: no table is active. Their key counts are still recorded.
- **The harmonic weights are a copy.** The integer crate's native root (`native_geometry.rs`) does not export `stack`, so the float side cannot import `RAW_RANK_WEIGHTS_Q16`. The first 16 are copied as `FLOCK_RANK_WEIGHTS_Q16`. A test reads `crates/uor-r4-integer/src/stack/flock.rs` and fails if they differ, and checks that they are `65,536/(i+1)`, the first saturated at `u16::MAX`.
- **Ties at the 16th place are broken by the GPU sort, not by id.** Within the 16 the order is ties to the smaller id, as the protocol says. A float tie straddling the 16th place is rare.
- **The Flock rank rule runs with the memory on only,** as the shipped model is served. Greedy runs with the memory on and off.

## What changed

In `crates/uor-r4-float-train` (the item's scope):
- **I-E:**
  - `DecodeRule` (`Greedy`, `FlockRank`), `FLOCK_DECODE_K` = 16, `FLOCK_RANK_WEIGHTS_Q16`;
  - `flock_rank_draw(seed, index, k)`: one xorshift64* step seeded as `epoch_order` seeds, `r = output mod H`, the first rank whose running weight exceeds `r`;
  - `rank_order`: the higher score first, ties to the smaller id;
  - `decode_reply`: the decode loop, with the next token picked by the rule. `greedy_reply` now calls it with `Greedy`, so the greedy path is the old loop.
- **I-F:** `reply_key_hits(prompt, reply, named)`, the key `greedy_reply_key` forms at each later step and whether the table names it. `IdSample` gains `reply_keys`, `reply_named`, `reference_keys` and `reference_named`, which default to 0 when reading older files.
- **I-A:** `score_rows` takes `panels: &[(String, PathBuf)]`, loaded by `load_unpinned_uort`, and records each panel's sha (`ScoredFile.panel_sha256`).
- **The samples it writes:**
  - DEV's greedy samples keep their names, `samples-memory.json` and `samples-no-memory.json`, so claude-205's grader still reads them;
  - a panel's greedy samples are `samples-{name}-greedy-{memory|no-memory}.json`;
  - Flock rank samples are `samples-{split}-flock-memory.json`.
- **The binary:** `--score-rows` passes its `--panel` list, and its usage docs describe the samples.
- **Tests, 5 new:**
  - `the_flock_rank_weights_are_the_integer_selectors_table`;
  - `the_flock_rank_draw_follows_the_harmonic_weights`: K = 1 is greedy, and over 10^6 draws each rank is within 0.002 of `W_i/H`;
  - `candidates_rank_by_score_then_smaller_id`;
  - `the_hit_rate_keys_are_the_teacher_forced_reply_keys`;
  - `an_older_sample_file_still_loads`.

## Measurements

None. This unit is code and its tests.

## Checks run at the head

- `cargo test --locked -p uor-r4-float-train --no-fail-fast`: lib 144 passed, 0 failed (5 new), and every other target passed.
- **The negative gate:** with `reply_key_hits` keying on `reply[..=j]` in place of `reply[..j]`, `the_hit_rate_keys_…` FAILS; restored and rebuilt, it passes.
- A stub-nvcc `cargo check --offline --locked -p uor-r4-float-train --features cuda --bin train-in-space --lib --tests` builds.
- **At `acba137f8`:** `loop report --check` PASS, `loop pr-check --local` PASS, `loop gates --full` 11 GREEN; `git ls-files` lists no `local/` path.

## What it unlocks

Five of the protocol's eight instruments exist. Next, on the laptop:
- **I-D,** the controls (T) and (S);
- **I-G,** the grader's strata and five decisions.

Then the run's pre-run review, its probe ladder (the GPU checks of I-A to I-F, and I-H's reproduction), and the 21-run training.

## Skills used

None was invoked this unit.

## Still open

- **The GPU halves of I-A, I-E and I-F,** on the run's pod.
- **Instruments I-D, I-G and I-H.**
- **I-F's laptop count** over every DEV later row, in the grader unit (I-G).

## Plain summary

The scorer can now score the held-out panel P-dev. It can make the model write replies two ways: always the closest word, or a draw among the 16 closest words weighted 1, 1/2, 1/3 and so on, the same weights the project's integer runtime uses. For every reply it also counts how often the exact memory recognized what the model was writing, which shows whether the memory still helps once the model writes on its own.
