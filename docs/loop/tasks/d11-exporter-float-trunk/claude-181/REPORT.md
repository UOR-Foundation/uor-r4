# REPORT: d11-exporter-float-trunk, claude-181 (the unsnapped D11 contract, on the host)

Lab: claude. Order: [ORDER.md](ORDER.md). References #2820. The item is not done: the new artifact's export gap needs the GPU evaluator updated, and that is the next unit.

**Speed.** `fbs-toks` decode tokens/s on this M1 (AC power), claude-126's seed-1 snapshot. Each figure is the median of 20 timed runs, and each form was run twice, alternating:

| Form | Decode tok/s | Context tok/s |
|---|---|---|
| snapped (`lut-stack/4`) | 4,827 / 4,905 | 4,823 / 4,877 |
| gates unsnapped | 4,455 / 4,494 | 4,327 / 4,389 |
| transition unsnapped | 4,679 / 4,673 | 4,679 / 4,669 |
| both unsnapped | 4,240 / 4,228 | 4,220 / 4,216 |

## What did not work, first

- **Unsnapped serving is slower, not faster.**
  - Against the snapped mean of 4,866 decode tok/s, unsnapping the gates costs about 8%, the transition about 4%, and both about 13%.
  - The snapped path has no general multiply left: its products are exact shifts (claude-169), and its snap is a few additions (claude-164, claude-168). An unsnapped block pays an integer square root, four 128-bit divides and sixteen full multiplies.
- **With unsnapped gates, the query drifts further from the f64 reference.**
  - On claude-126 seed 1, the median per-position worst channel is 3 LSB snapped, against 6 to 7 LSB with unsnapped gates.
  - The likely cause is `stack_rotation`'s unit, which is rounded toward zero at Q30, so every gate is a little short. That is not tested here.
  - Token and kept-cell agreement still rise (Measurements).
- **My first trained-snapshot floor was wrong.** It required 95% of queries within 2 LSB. The snapped contract itself keeps only 1,379 of 4,096 single tokens there, while 98% of its tokens and cells agree.
  - That floor was dropped before any unsnapped number was read, and query drift became a reading.
  - Single tokens cannot test the transition, which moves a state only from the second token. So 64 sequences of 32 tokens were added.
- **The WASM serving audit reads 5 of 7, on `origin/main` too.**
  - The build was a plain `cargo build --release --target wasm32-unknown-unknown --features serving-wasm --lib`. With it, `step` and `IntegerStackSession::step` carry 236 and 232 unclassified instructions, the 0xFC (bulk-memory) prefix that is likely this toolchain's default. Both read 14 multiplies, 9 divides and 0 float.
  - This unit's build gives the same counts. claude-175's 7 of 7 did not record its build flags (FRICTION.md).
- **The whole `uor-r4-integer` suite has 36 failures**, as `main` has (claude-177): tests that need the local chat bundle (`load bundle: NotFound`), and the m7 parity tests.

## What changed

- **`stack/format.rs`:**
  - `StackShape` gains `raw_gates` and `raw_transition`. Both are serialized only when true, so a snapped header is unchanged.
  - Either flag selects `STACK_FBS_RAW_SCHEMA` (`uor-r4.lut-stack/5`), so an engine that only snaps refuses the artifact rather than snapping it.
  - `validate` keeps both flags to FBS stacks.
  - `Container::parse` refuses a `transport_snap` beside `raw_transition`, and still requires one otherwise.
  - `export_fbs_trunk_uorlut01_with(snap, FbsExportSnaps)` writes the choice, and the old `export_fbs_trunk_uorlut01` is its snapped case. `FbsExportSnaps::reference()` gives the f64 reference snaps for each choice.
- **`stack/session.rs`:**
  - Both Hamilton gate sites call `stack_gate_quaternion`: the icosian snap as before, or under `raw_gates` the unit `g/|g|` from `stack_rotation`, at Q31. The product's `>>15` and its bound are unchanged.
  - `stack_rotation` takes its epsilon: 1e-6 for the transition, as before, and 0 for a gate, which is the float gate's 1e-12 at that scale.
  - An unsnapped transition uses the runtime's existing `stack_rotation` branch.
- **`bin/fbs-export.rs`:** `--raw-gates` and `--raw-transition`.
- **Tests (`stack/tests.rs`):**
  - every combination against its f64 reference on the synthetic control;
  - each unsnapped choice changes the served queries;
  - the header's snap statements must agree, and the snapped export is the `lut-stack/4` bytes;
  - an ignored trained-snapshot parity test (`UOR_FBS_SNAPSHOT`).
- **`tests/memory_shape.rs`:** the new fields.

## Measurements

**Parity on the synthetic control** (`synthetic_fbs_trunk(20261026)`): all 256 single tokens, plus an 8-token sequence. In all four combinations, every query is within 2 LSB at Q12, with 0 near-tie exceptions in token or kept cells (the bar allows 4).

**Parity on claude-126's seed-1 snapshot** (`a24ea8cf…4545`, checked against the store): the integer session against the f64 reference with the same snaps. Single tokens are all 4,096; the sequences are 64 of 32 tokens each, 2,048 positions. Each cell gives tokens and kept-cell sets agreeing, then the median per-position query drift in Q12 LSB.

| Form | Single tokens | Sequences |
|---|---|---|
| snapped | 4,045 / 4,022; 3 | 2,015 / 1,995; 3 |
| gates unsnapped | 4,087 / 4,085; 6 | 2,042 / 2,039; 7 |
| transition unsnapped | 4,045 / 4,022; 3 | 2,020 / 2,008; 3 |
| both unsnapped | 4,087 / 4,085; 6 | 2,048 / 2,044; 6 |

- The largest drift falls from 19,887 and 16,274 LSB (snapped: a snap flipping at a near-tie) to 658 and 1,840 with both unsnapped.
- The test's floor of 95% agreement holds in every combination.

**No regression.**
- This head's snapped export of claude-126 seed 1 is `314c3909c66b58522a641c3f5b51c25dda4bda5ab2274bcfd0b5ca5038fdb305`, the artifact claude-163 to claude-177 served.
- The unsnapped exports: `5d452a59…0613` (gates), `50b10386…3084` (transition) and `9e80cc6a…8694` (both).

**Audit.** `audit_zero_matmul_serving.py --stack` on the release `uor-r4-stack` (`bf0b3b40…`) gives FULL PASS: 0 forbidden instructions across 44 symbol ranges and 93 reachable functions. The allowed scalar multiplies and divides are 47 and 1 in the ranges, and 65 and 9 in the call graph, as claude-177 read them.

**Negative gate.** With this unit's `session.rs` change set aside (`git diff origin/main`, reversed), `each_unsnapped_choice_changes_the_served_forward` and the parity test both fail: "gates: false … served the snapped forward". They pass with the change restored.

## Checks run at the head

- `cargo test -p uor-r4-integer --features serving-wasm --no-fail-fast`: 549 passed. The 36 failures are `main`'s (above), and every new test passes.
- The ignored trained-snapshot test, run with `UOR_FBS_SNAPSHOT`: passes.
- `cargo test -p uor-r4-float-train --no-fail-fast`: 104 library tests and the rest pass.
- `cargo clippy -p uor-r4-integer --features serving-wasm --all-targets`: no warnings in the changed code. Two older hits in `tests.rs` remain, at lines 864 and 1794.
- `cargo fmt`.
- `loop report --check`, `loop pr-check --local` and `loop gates --full`: in the review record.

## Skills used

- **holding-the-goal.** The change is the one claude-180's reading named, and the snapped contract stays byte for byte. A floor that turned out to be wrong was fixed before any unsnapped reading was taken. The slowdown is reported as found.

## Still open

- **The next unit: the export gap of the unsnapped artifact, on the GPU.**
  - The GPU evaluator (`docs/labs/d11-exporter-float-trunk/instrument`, whose `fbs_core.h` mirrors the snap) and the float D11 readout gain the unsnapped forms. They are requalified bit-exact against this session.
  - The export gap of `raw_gates` and of both unsnapped is then measured on the eight models claude-180 used.
- **Rounding the unit to nearest** in `stack_rotation` for the gates may halve the query drift. It needs its own parity reading.
- **A speed lever for unsnapped serving:** one reciprocal per block in place of four divides.
- **The WASM audit's build command** should be recorded with its result (FRICTION.md).
- **The mechanism page** `docs/mechanisms/lut-serving.md` is out of this item's scope (claude-121).

## Plain summary

The integer version of the model used to round each of its rotations to the nearest of 120 fixed ones. Now that integer multiply and divide are allowed, it can keep each rotation exactly as the trained model computes it. The model file says which way it was built, and an old file still produces exactly the same bytes and answers. On a trained model, the exact rotations agree with the trained model's arithmetic more often: on every token of the test sequences, against 98% before. They cost some speed, about 4,200 to 4,700 words a second on this laptop, against 4,900. The next step measures, on the GPU, whether this closes the gap between the integer model and the trained one.
