# REPORT: point-key-integer, claude-224 (unit 2: the panel decode and the grade)

Lab: claude. Item: [point-key-integer](../../../../roadmap/items/point-key-integer.md) (#3450). Order: [ORDER.md](ORDER.md), written by hand: `loop claim` refuses on Kimi's #3436 false glob overlap.

This unit is the item's unit 2, on the laptop, with finite tests and no run.

## Tokens per second

No measurement. The integer decode's tok/s is unit 3's, under its sheet.

## What did not work, first

- **The integer arms cannot slide their window, so their replies are capped.**
  - Float's greedy decode re-reads the last 192 tokens at each step. The integer session holds one context and cannot slide.
  - Float itself stops keying the reply table once its window passes the prompt's last token (`prompt_cut`). An integer re-prefill would have to reproduce that rule row by row.
  - So both integer arms end where prompt and reply fill the artifact's context (192), as well as at BOS or 64 tokens. They are capped identically, which keeps the bar a fair paired test.
  - Parity's opening name needs only the prompt.
  - The longest p-w prompt is 186 tokens, which leaves it 6 reply tokens. The grade reports the capped count per split and seed.
  - A prompt of the context's full 192 tokens still decodes its opening, with no step past the context. At first `panel_reply` refused it; the review's fix is pinned by a test.
- **claude-220's `keyed_reading` could not be reused whole.** It reads the keys files and the forced arm, and the integer arms have neither. The integer reading keeps its measures (exact replies, templates, loops, `right`) and builds a `KeyedReading`, so `carries` and `harmless` read it unchanged.
- **`--out` counted as the training mode,** so `--panel-prompts` with `--out` was refused. The mode list now exempts it, as it already did `--controls`.

## What worked

- **The prompts** (`train-in-space --panel-prompts 1`). They are `score_rows`' own, with no model:
  - the windows are `sample_windows`';
  - each prompt is its window through the first scored offset;
  - each reference is its `reference_reply`.

  On claude-220's sealed run, every split's prompts equal its served files' (1,202 on p-w, 393 on p-v, 310 on DEV), and the float served names carried are its at each of the three seeds (1,172, 385 and 19 at every seed). A mismatch would have been refused (`attach_float_names`). A `--runs` folder without DEV's served files at seeds 1–3 is refused too.
- **The decode** (`fbs-toks ARTIFACT --replies PROMPTS --out FILE [--reply-tokens N]`, over `panel_reply`).
  - Each prompt runs in a fresh session, with the reply start marked at its last token, then greedy.
  - It records the opening's point-key name and least distance at Q12 where the key served. It also records the float name's least distance in the table, from `point_key_distance`, off the step's path.
- **The grade** (`train-in-space --integer-grade 1`).
  - The integer served arm is read against integer greedy by their own tokens.
  - Parity against the float key uses the item's fixed classes: agree; within the 512 Q12-unit rounding bound (`POINT_KEY_ROUNDING_BOUND`); or drift, a float name no row holds included.
  - It also gives the capped counts, the free-generation measures, and each panel's per-seed `carries`.
  - Two instrument faults are refused: a served reply that does not open with its keyed token, and a reply the key did not open that differs from greedy's (claude-222 showed the two artifacts decode alike there).

## What changed

- **`crates/uor-r4-integer/src/stack/session.rs`:**
  - each step records `last_point` (the served name and distance, or `None`), read by `last_point_key()`;
  - the model gains `point_key_rows()` and `point_key_distance()`.
- **`stack/point_key.rs`:** `least_distance_to`; `reply_stop` and `ReplyEnd` (Bos, Length, Context); `panel_reply` and `PanelReply`.
- **`stack/mod.rs`:** exports.
- **`stack/tests.rs`:** two tests; `point_key.rs` one more (`least_distance_to`).
- **`bin/fbs-toks.rs`:** the `--replies` mode.
- **`crates/uor-r4-float-train/src/in_space_long.rs`:** `PanelPrompt`, `split_prompts`, `panel_prompts`, `IntegerSample`, `POINT_KEY_ROUNDING_BOUND`, `IntegerParity`, `integer_parity`, `integer_reading`, `attach_float_names`, `check_float_names`, `IntegerGrade` and `integer_grade`, with four tests. The grade needs `--keys-runs` (claude-220's `out`), and refuses an integer served file whose float names are not that seed's.
- **`bin/train-in-space.rs`:** the two modes, and the mode list.

## Measurements

- **The integer crate's library tests** (`--features serving-wasm`): 383 pass, against 380 at the base.
- **The float crate:** 185 pass in the library, against 181 at the base.
- **The serving audit** (`audit_zero_matmul_serving.py --stack`): FULL PASS, with 0 forbidden instructions across 44 symbol ranges and 123 reachable functions.
- **The host check:** the prompts of p-w, p-v and DEV equal claude-220's sealed served files', and so do the keyed counts.

## Checks run at the head

- **Thirteen negative gates,** each failing at its own assertion with its rule set aside, and passing when restored:
  - **the decode:**
    - the stop at BOS;
    - at the length;
    - at the context;
    - the served record;
    - the float name's rows;
    - the L1 sum (a `max` in its place);
    - a full-context prompt's opening;
  - **the grade:**
    - the prompt through the first scored offset;
    - the rounding bound's `≤`;
    - an unkeyed reply is greedy's;
    - a served reply opens with its key;
    - the float names attach only to their prompts;
    - the integer file carries this seed's names.
- **The review's fixes** (`5ad69ab93`): the float names checked per seed, `attach_float_names` tested, DEV's seeds required, a full-context prompt's opening decoded, and `least_distance_to` pinned at nonzero distances.
- **The stub-nvcc CUDA check** (`cargo check --features cuda --all-targets`): clean.
- **The WASM library build:** finishes.

## Skills used

None.

## Still open

- **Unit 3, the run,** under a prep sheet with a fresh FIT and a pre-run review:
  - (r-drop)'s snapshots on a pod;
  - `fbs-export` and `fbs-export --point-key` over TRAIN on the laptop;
  - the fresh panel p-x drawn with its census, and its prompts by `--panel-prompts`;
  - `fbs-toks --replies` for both arms at seeds 1–3, then `--integer-grade`;
  - tok/s.
- **The cap** is a limit of the integer arms. Its count is reported, and the sheet states it.

## Plain summary

This unit builds the tools to test the whole-number model on the same test questions the decimal model answered. One tool picks the exact questions the earlier test used. Another has the whole-number model answer them. A third scores the answers, and checks how often its lookup picks the same first word as the decimal model's.

The whole-number model holds a fixed amount of text at a time, so its answers to long questions are cut shorter than the decimal model's. Both of its versions are cut the same way, so their comparison stays fair. Running the tools on the trained model is the next step.
