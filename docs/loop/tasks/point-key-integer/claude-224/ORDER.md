# ORDER: point-key-integer, claude-224 (unit 2: the panel decode and the grade)

Lab: claude. Item: [point-key-integer](../../../../roadmap/items/point-key-integer.md) (#3450). Base: `4f8f62ecd`.

**Written by hand.** `loop claim point-key-integer` refuses on a false overlap with Kimi's #3436 glob `crates/*/README.md` (claude-222's ORDER).

## Goal

The item's unit 2 ("The units"), on the laptop, with finite tests and no run. This unit gives the run (unit 3) its instruments:
- the panel prompts, from the float side's own rule;
- an integer decode of them, for greedy (`/7`) and served (`/8`);
- the grade: the integer arms read by their own tokens, parity against claude-220's float names in the item's fixed classes, and each seed's reading of done-when 4's bar.

## Cancel probe (run first)

- `point-key-integer`'s scope holds `in_space_long.rs`, `train-in-space.rs` and `fbs-toks.rs` (claude-223, `4f8f62ecd`).
- No open PR touches them, or the integer stack.
- No integer panel decode exists: `fbs-toks` times one fixed prompt.

If any fails, stop and report.

Run on 31 Oct at the base:
- all three files are in the item's scope;
- no open PR touches them or the integer stack;
- `fbs-toks` has no panel mode.

Result: **go**. The probe reads the setting, so it is an instrument check.

## The design

1. **The prompts** (`train-in-space --panel-prompts 1`).
   - For each `--panel NAME=DIR`, and DEV, it writes `prompts-{split}.json`: `{position, prompt, reference}`.
   - The rule is `sample_windows` and `reference_reply`, unchanged and with no model (`--samples 1536 --reply-tokens 64`, as claude-220). So every split's prompts are the ones a `--score-rows` run decodes.
   - A test checks the mode against `sample_windows` on a small panel. A host check compares p-w's prompts with claude-220's sealed p-w files.
2. **The decode** (`fbs-toks ARTIFACT --replies PROMPTS --out FILE [--reply-tokens 64]`).
   - Each prompt runs in a fresh session, with the reply start marked at the prompt's last token, then greedy.
   - **It stops** at BOS, at `--reply-tokens`, or when prompt and reply fill the artifact's context, whichever comes first.
   - **Why it stops at the context (the design call, decided):** float re-reads the last 192 tokens at each step and stops keying the reply table once the window passes the prompt's last token. The integer session cannot slide. Both integer arms are capped the same way, so the bar's paired test stays fair. Parity's opening name needs only the prompt. The count of capped replies is reported per split.
   - **Each sample** carries `position`, `prompt`, `reference` and `reply`.
     - `keyed` and `served_distance` are given where the point key named the opening. They come from a read-only session accessor for the step's served name and least distance.
     - Where the prompts file gives the float side's name, `float_distance` is the least distance of a row naming it. That comes from a model method, `point_key_distance(query, target)`, off the step path.
3. **The grade** (`train-in-space --integer-grade 1`).
   - **The arms:** for each split and seed, the integer served arm against integer greedy, by their own tokens (rule (a)). This uses claude-220's `keyed_reading`, `carries`, `harmless` and `generation`, reported per seed.
   - **Parity, reported, deciding nothing (done-when 3):** at each opening both serve, the integer name against claude-220's float served name. Each difference is classed with the item's fixed rule: within the 512 Q12-unit rounding bound (`float_distance − served_distance ≤ 512`), or trunk drift.
   - **The bar:** done-when 4's per-seed reading on p-x. The rows are unit 3's sheet's.
4. **Tests,** each with a negative gate:
   - the prompts mode equals `sample_windows`;
   - the decode's cap, BOS stop and keyed fields (synthetic stack);
   - `point_key_distance`;
   - the grade's parity classes and its readings on hand-made files.

## Files in scope

- `crates/uor-r4-integer/src/bin/fbs-toks.rs`
- `crates/uor-r4-integer/src/stack/{session.rs, point_key.rs, mod.rs, tests.rs}`
- `crates/uor-r4-float-train/src/in_space_long.rs` and `crates/uor-r4-float-train/src/bin/train-in-space.rs`
- `docs/loop/tasks/point-key-integer/claude-224/**`

## Done when

- **The tests above pass,** each with its negative gate.
  - `cargo test --release --locked -p uor-r4-integer --features serving-wasm --lib` passes.
  - So do `cargo test --release --locked -p uor-r4-float-train` and the stub-nvcc CUDA check.
- **The serving audit still passes:** the accessor is off the step's arithmetic.
- **The host check:** p-w's prompts equal claude-220's sealed p-w files' prompts.
- **The REPORT** gives the negatives first, including the cap. A fresh review runs in slices of at most 600 lines.

## Stop and ask if

A file outside scope is needed, the prompts cannot be made without a model, or the grade would change claude-220's decided rows.
