# REPORT: integer-retention-rates, claude-226 (the retained recurrence, its flag and its export)

Lab: claude. Item: [integer-retention-rates](../../../../roadmap/items/integer-retention-rates.md) (#3456). Order: [ORDER.md](ORDER.md), written by hand: `loop claim` refuses on Kimi's #3436 false glob overlap.

This unit is the item's done-when 1, 2 and 4, on the laptop, with finite tests and no run. Parity on the GPU (done-when 3) is the next unit's.

## Tokens per second

No measurement. The step reads one of two tables where it read one before, and the serving audit sees no new instruction kind.

## What did not work, first

- **`fbs-snapshot`'s float readouts were taken under the decay reading.**
  - The float trunk's retention is a process switch (`set_retention_rates`), and `fbs-snapshot` never set it. So the readouts a parity unit compares against were not the trained model's.
  - The bin now reads the run's config and turns retention on before the readouts. Any earlier readouts of a retained checkpoint are not that model's.
- **One negative gate first passed through another check.**
  - Widening `validate` to accept an unknown `rec_gates` value left the test green: the artifact was still refused, by a later structural check (`rec_geometric()` is false for it, so the FBS layers' tables do not fit).
  - The refusal an older engine makes is its `validate`'s, so the test now calls `validate` on the shape directly, and the gate fails.
- **`fold_scores` and `reflect_floor` are not served** (claude-225's review). `fbs-snapshot` refuses a run with either on, until their integer forms exist.

## What worked

- **The flag:** `rec_gates = "geometric-retention"`.
  - `rec_geometric()` is true for both values, so every geometric path holds, and `rec_retention()` is true for the new one alone.
  - `validate` allows it. An engine before this unit refuses it by its own `validate` ("the recurrence gates are not geometric").
  - The FBS export writes it from `FbsTrunkSnapshot.retain_rates`, which is false and not written by default. So every earlier snapshot exports byte for byte as before.
- **The retained recurrence:**
  - `RETAIN_LAMBDA_Q31[i] = (1 << 31) − (1 << (30 − i))`, exactly 1 − 2^−(i+1) at Q31;
  - `RETAIN_KEEP_Q31[i] = isqrt(2^62 − λ_i²)`, within one unit of √(1 − λ²);
  - both are built by `const fn` and read at the geometric recurrence under `rec_retention()`;
  - entry 0 is one half in both readings, and entry 1 is the first to differ.
- **The f64 reference (`RefRecurrence`) reads retention** when the snapshot says so. A retained synthetic stack's integer session serves it within 2 Q12 units at every row, and does not serve the decay reading's reference.
- **The run's reading** (`run_rate_reading`) is its `config.json`'s `retain_rates`, false when absent. `fbs-snapshot` sets the snapshot's flag from it.

## What changed

- **`crates/uor-r4-integer/src/stack/format.rs`:** `rec_geometric`, `rec_retention`, `validate`, the FBS export's `rec_gates`, and `RefRecurrence::retained`.
- **`stack/session.rs`:** `RETAIN_LAMBDA_Q31`, `RETAIN_KEEP_Q31` and the recurrence's branch.
- **`fbs_snapshot.rs`:** `FbsTrunkSnapshot.retain_rates`.
- **`stack/synthetic.rs`:** the field.
- **`stack/tests.rs`:** three tests.
- **`crates/uor-r4-float-train/src/fbs_export.rs`:** `run_rate_reading` and a test.
- **`bin/fbs-snapshot.rs`:** the run's config, the snapshot's flag, and retention before the readouts.

## Measurements

- **The integer crate's library tests** (`--features serving-wasm`): 386 pass, against 383 at the base.
- **The float crate:** 186, against 185.
- **The serving audit** (`audit_zero_matmul_serving.py --stack`): FULL PASS, with 0 forbidden instructions across 44 symbol ranges and 123 reachable functions.

## Checks run at the head

- **Eight negative gates,** each failing at its own assertion with its rule set aside, and passing when restored:
  - the λ table;
  - the recurrence reads it;
  - the export writes the flag;
  - the reference reads retention;
  - the keep table;
  - `validate` refuses an unknown reading;
  - the config's refusals;
  - false is not written.
- **The stub-nvcc CUDA check** (`cargo check --features cuda --all-targets`): clean, `fbs-snapshot` included.
- **The WASM library build:** finishes.

## Skills used

None.

## Still open

- **Done-when 3, parity on the GPU,** under a prep sheet with a fresh FIT and a pre-run review:
  - `fbs-snapshot` on (r-drop) s1 with readouts (now under retention);
  - `fbs-export`;
  - `fbs-compare`, the integer readout against the float one.
- **Then `point-key-integer`'s run.**

## Plain summary

The trained model keeps a long memory in one internal step, and the shipped whole-number engine only knew the short-memory version. This unit teaches the engine the long-memory version, with exact whole-number tables. The file format now says which version a model needs, so an older engine refuses a model it cannot run correctly.

The tool that converts a trained model now reads which version the model trained with. A check on a GPU that the converted model matches the trained one comes next.
