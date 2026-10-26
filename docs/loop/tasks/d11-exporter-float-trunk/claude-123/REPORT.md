# REPORT: d11-exporter-float-trunk, claude-123

Lab: claude. Order: [ORDER.md](ORDER.md). Pre-run review (D67): #2820 comment `6408456837`, PRE-RUN: GO.

References #2820.

## What did not work, first

- **The float GPU trunk and the converted trunk disagree on most tokens, and this unit does not say why.**
  - On a random-init trunk (vocabulary 256, width 32, MLP 32, 2 layers), the float GPU model picks the converted reference's token on 67 of 256 single tokens. The largest gap between the GPU's final hidden vector and the reference's query is 2.825.
  - Part of this is the known export gap. The integer path, and so the reference, snaps the Hamilton gates and the recurrence rotations to the 120 icosians, while the plain float GPU path keeps them continuous.
  - This run cannot separate that gap from a fault in how the converter maps the copied tables. Separating the two is Stage 4's parity proof against the sheets' export mode, so it is now the first thing Stage 4 owes.
  - What is proven exactly is the routes (below). The other tables were checked by reading the float forward, not by a test:
    - the embedding is a plain row lookup, with no scale;
    - the convolution weights are `[tap][channel]`, with tap 0 on the current position;
    - the gate bias holds the decay gates, then the rotations;
    - `rec.in` holds the drive, then the output gate;
    - the decay offset is added before the clamp;
    - the RMS epsilon is `1e-5`, the CUDA kernel's `RMS_EPSILON`.
- **A regression of mine reached main.** claude-122 (#3255) left `stack_fbs_gather` loading through SIMD registers.
  - The crate's own step-disassembly verifier refuses those registers. So `tests/m5_full_path_cost_controls` (2 of 3 tests) and `uor-r4-stack cost` have failed on main since `611cfb05`.
  - The Python audit allows plain SIMD loads, and CI does not run that integration test, so neither caught it. I found it here, in this unit's full integer suite.
  - It is fixed in this PR: `black_box` on each stored lane, the pattern `kernels.rs` already uses. My first attempt, `black_box` on each load, failed the verifier, because the compiler spilled each value and reloaded it into an `h` register.
  - Noted on #2028 (comment `6408584391`).
- **The design changed after the ORDER was written.** The integer crate's `stack` module compiles only under `serving-wasm`, and that feature also compiles `model`, `sampling` and `tables`. D29's presence rule rests on the feature staying off in the native graph. So:
  - The snapshot types moved to an ungated `fbs_snapshot` module.
  - The converter writes JSON. A second bin, in the serving build, exports that JSON to the artifact.
  - Only the test build enables `serving-wasm`, through a dev-dependency.
  - The new path dependency adds one line to `Cargo.lock`. The item's scope omits that file, so pr-check's scope rule (advisory) WARNs on it.
- **Two of my own checks were wrong at first.**
  - A "scaled value" refusal case halved a coded value. That can land on another atom's slot, which the decoder correctly reports as two atoms. It is now a fractional value.
  - My first set-aside of the two-atom check did not compile, so it proved nothing. A compiling set-aside then failed the test as required.
- **Failures in the integer crate's full suite that are not this PR's:**
  - In a run at this branch before the gather repair, 38 tests failed: my 2 step-audit tests, then 35 that need fixture files a clean clone lacks (bundles and `tokenizer.json`), and one more.
  - That one, `m7_parity_library_cli_browser`, fails because `wasm-pack` calls a bare `cargo`, which Homebrew's `cargo` shadows on this Mac.
- **Pages not updated.** `docs/model/serving-and-export.md` and `docs/mechanisms/lut-serving.md` should name the converter and the two bins. Both are outside this item's scope, and `lut-serving.md` waits for the umbrella (claude-121's note).

## What changed

Size against `origin/main`:
- code about +464/−69, which is `fbs_export.rs` lines 1–256 plus the other code files and `Cargo.lock`;
- tests about +345: the `fbs_export.rs` test module (lines 257–578) and `stack/tests.rs`;
- docs about +180: the ORDER, this REPORT and the notebook row.

pr-check counts the whole of `fbs_export.rs` as code: code +785/−69, tests +23, docs +169, generated +1. The ORDER gave no line estimate.

- **`uor-r4-integer`:**
  - `fbs_snapshot` (ungated) now holds `FbsRoutes`, `FbsTrunkLayer` and `FbsTrunkSnapshot`, with serde that refuses unknown fields. `stack::format` re-exports them.
  - `discrete_d11` adds `spiralcore_signed_axes()` and `spiralcore_orbit_roots()`, both read-only.
  - A new serving-build bin, `fbs-export SNAPSHOT_JSON OUT_PATH`. It parses the artifact back before writing it, never overwrites, and prints the sha256.
  - The `stack_fbs_gather` repair.
- **`uor-r4-float-train`, `fbs_export`:**
  - `fbs_trunk_snapshot(&StackModel)` reads each projection's routes back through the model's own GPU projection, `linear`, now `pub(crate)`. It feeds one coded carrier, where slot `i` holds `i + 1`.
  - `decode_coded_routes` refuses any block that is not a one-atom signed gather with a state in the integer Cl(0,6) table.
  - `check_geometry_tables` holds the float geometry's 64×8 signed axes and 16 orbit roots to the integer crate's, in order.
  - `check_fbs_export_config` refuses each unservable configuration for its own reason.
  - A new GPU bin, `fbs-snapshot MODEL_DIR OUT_JSON`.

## Measurements

| Measurement | Value | Where |
|---|---|---|
| Route read-back on the GPU | bit-identical to the model's projection on 3 random carriers, for all 6 projections in both layers | `gpu_converted_trunk_reproduces_the_projection_and_serves`, RTX 5090, head `d93b4977` |
| GPU set-asides | the GPU test fails at its bit-equality assert (`fbs_export.rs:532`) with the decode's sign bit dropped, and with a valid but wrong state | same pod, `tests-setaside-{1,2}.log` |
| Export, parse, serve | the converted trunk exports and parses; the integer session matches the quantized reference's token on 256 of 256 | same test |
| Float GPU against the converted reference (a reading, no bar) | 67 of 256 tokens agree; largest hidden gap 2.825 | same test |
| Tables identical | float signed axes and roots equal the integer crate's | `geometry_tables_match_the_integer_crate` (host) |
| Host converter tests | 4 pass. Set aside the root check, the two-atom check, the state lookup or the U(1) refusal, and each fails its test | `cargo test -p uor-r4-float-train --lib fbs_export` |
| Snapshot JSON | round-trips bit-exactly and exports to the same bytes; an unknown field is refused | `fbs_trunk_snapshot_json_round_trips_exactly` |
| Step-audit repair | `m5_full_path_cost_controls` passes 3 of 3; with the fix set aside, 1 of 3 | integration test, host |
| Zero-multiply audit | `FULL PASS`: rlib 56 symbol ranges and 41 call-graph functions; `uor-r4-stack` 44 and 90 | `scripts/audit_zero_matmul_serving.py --stack` |
| Integer serving-build library | 314 passed | `cargo test -p uor-r4-integer --features serving-wasm --lib` |
| GPU cost | one RTX 5090 for about 5 minutes, at $1.09/h | Vast `vast-57766451`, deleted |

The GPU evidence is in the HF store at `results/claude/d11-fbs-converter-2026-10-26/`, with a `SHA256SUMS`. The run was at `d93b4977`. Later commits change only the integer gather's code generation (the same values: the FBS control is unchanged) and the docs. The run is repeated at the final head before the merge.

## Checks run at the head

See the exact-head review record on the PR:
- the converter's host tests, and the float-train suite;
- the integer serving-build library, and `m5_full_path_cost_controls`;
- the stub-nvcc `cargo check --features cuda --all-targets`;
- the GPU test at the final head;
- clippy (no findings in new files; the eight in `discrete_d11.rs` are pre-existing);
- `cargo fmt --check`;
- both audits;
- `loop report --check`, `loop pr-check --local` and `loop gates --full`.

## Skills used

- **holding-the-goal.** When the feature boundary appeared, I re-planned the unit around it rather than enabling `serving-wasm`.
- **writing-meaningful-tests.** Every new test was checked against a set-aside of the code it pins, including the GPU test, with two set-asides.

## Still open (Stage 4 and after)

- **The parity proof,** first: the float GPU in the sheets' D11 export mode against the converted trunk, to split the export gap from any mapping fault in the copied tables.
- **The Stage 4 sheet:** a fresh non-author FIT; a two-seed retrain saved with `StackModel::save`; the export gap measured on the trained trunk; laptop tok/s, with decode and context reported separately.
- **Serving front ends:** native-head paths in the `uor-r4-stack` CLI and the WebAssembly front end.
- **The pages:** `serving-and-export.md` and the `lut-serving.md` row.

## Plain summary

This unit builds the bridge from a float-trained SpiralCore/FBS model to the integer serving file.
- The tricky part is each projection's routing choice, which the GPU makes. The converter reads it back by pushing one numbered input through the GPU and reading which numbers come out where, so nothing is guessed. On a GPU this reproduced every projection exactly.
- The converted model exports to the integer file, loads, and runs.
- What is not yet known is why the plain float model and the converted one often pick different next tokens on an untrained model. Part of the reason is that the integer version rounds its rotations to a fixed set. The next unit has to measure how much.
- This unit also fixes a code-generation slip of mine from the previous unit, which had broken one serving test on main.
