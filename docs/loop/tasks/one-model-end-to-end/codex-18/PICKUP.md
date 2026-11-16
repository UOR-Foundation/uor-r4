# PICKUP: C2′ relation read (2026-11-16, codex)

## Where it stands

The C2′ mechanism is proven end-to-end at the runtime level:

- Panel, other fact (`K=1`): exact, both seeds and both splits (35/35, 12/12, 42/42, 8/8 held;
  308/308, 81/81, 296/296, 66/66 TRAIN).
- Determined control (previous fact): stream rule 768/768; learned gate 0.142 / 0.139; salience 0.004.
- Served answer (probe hard copy): `R_oth` 0.406 / 0.436; `R_ask` 0.
- Served integer session: `set_relation_read(read, header, header_last, scale)` applies the value's
  row after the FBS read hook; `last_relation_pick` reports the pick. Six tests; integer lib
  **683 passed**, 1 ignored.

## The next unit

`codex-19`: the `uor-r4.lut-stack/15` config-only relation sidecar, additive; the session takes the
read from the artifact. The full design is in `codex-19/ORDER.md`.

## Checkable state

- `CARGO_TARGET_DIR=local/relation-target cargo test --locked -p uor-r4-integer --features
  serving-wasm,fbs-synthetic --lib` → 683 passed, 1 ignored.
- `git log --oneline -1` → `c34c566d4` (or later).
- The panel: `crates/uor-r4-float-train/src/bin/recall-rule-read.rs --seed 1 --header 4 --windows 256
  --held --out …` → `K=1` 35/35, 12/12.
- The served decode: `recall-payload-probe --native-picks …` on the ext-s1 state → `R_oth`
  0.406/0.436, `R_ask` 0.
- The states/dumps: `/tmp/opencode/c386-ext*`, `/tmp/opencode/dump-ext-s{1,2}-s24`,
  `/tmp/opencode/train-ext-s{1,2}` (may need regenerating on a fresh host; the c386 HF tarballs at
  `results/claude/one-model-end-to-end-vector-read-selection/ext-seed{1,2}-*.tar.gz`).

## Open elsewhere

- The 1B model's read is claude's starling line; the C2′ result informs it, not overwrites it.
- Line 2: the corrected read-binding hinge (`crates/uor-r4-discrete/src/read_binding.rs`) needs an
  in-repo Flock trainer; the external driver is gone.
