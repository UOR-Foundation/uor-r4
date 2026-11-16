# REPORT: one-model-end-to-end, codex-20 (the real checkpoint carries its learned read)

Lab: codex. Order: [ORDER.md](ORDER.md).

## What did not work, first

- **The export has no stack contract or tokenizer for this run.** The ext-s1 run directory does not
  carry them, so the artifact is exported on the explicit `--legacy-artifact` path
  (`uor-r4.lut-stack/14` here). A bound export still needs the contract and tokenizer.
- **The payload scale cannot be a constant.** The first fixed `e_w = 29 / gain 2` assumed the read's
  largest weight was 3.7; the real read's largest weight puts a word past `2^31`, and the artifact
  refused. The gain is now computed from the read's own maximum: the smallest `g` with
  `max|w| · 2^-g <= 1`, then `e_w = 31 - g`.
- **The vector read is process-global, not a `StackModel` field.** `StackModel::load` does not set
  it; the snapshot must load it (`VectorRead::load`) before the capture, or the sidecar is silently
  absent.
- **The `/14` and `/15` records are exclusive.** The learned read and the relation rule ride separate
  artifacts; a stack with both is refused by both adders.
- No language or quality claim; one state, one seed.

## What changed

- `crates/uor-r4-native-vector/src/read.rs`: `rescale_payload(e_w_shift, gain_shift)` with the
  picks-invariance test.
- `crates/uor-r4-float-train/src/fbs_export.rs`: `q31_read_sidecar(model, seed)` — the read's gates
  from the replay capture, the payload moved to the FBS `e_w = 31 - g` with `g` from the read's
  maximum weight.
- `crates/uor-r4-float-train/src/in_space_long.rs`: `native_replay_pass` is crate-visible.
- `crates/uor-r4-float-train/src/bin/fbs-snapshot.rs`: loads the saved read, carries
  `vector_read.safetensors`, and writes `OUT.q31read` (`{depth, read}`) beside the snapshot.
- `crates/uor-r4-integer/src/bin/fbs-export.rs`: `--q31-read SIDE` adds the `/14` record.

## Measurements

- The ext-s1 checkpoint exports: `ext-s1.uorlut01` (2,151,500 bytes), schema
  `uor-r4.lut-stack/14`, `q31_read` record `depth 6`, `width 256`, `bytes 6153`, sha256
  `7d014abe…`. The snapshot and sidecar are `ext-s1.fbs.json` and `ext-s1.fbs.json.q31read`
  (`depth 6`, gain computed from the read).
- Native-vector **39 passed**; integer library **684 passed**.
- The panel through the integer session and the float parity are the next check; this unit proves
  the export and the record.

## Checks run at the head

- `cargo test --locked -p uor-r4-native-vector` — 39 passed.
- `cargo test --locked -p uor-r4-integer --features serving-wasm,fbs-synthetic --lib` — 684 passed.
- `fbs-snapshot <ext-s1> OUT` — writes the snapshot and `OUT.q31read`.
- `fbs-export --legacy-artifact OUT OUT.uorlut01 --q31-read OUT.q31read` — the `/14` artifact parses.

## Still open

- The Q31 read's picks vs the float read on the held windows (the export gap), and the C2′ panel
  through the integer session (the learned read's 0.12; the relation `/15`'s rule).
- A bound export (contract + tokenizer) for this run.
- The 1B read is claude's; the discrete read-binding hinge still needs its Flock trainer.

## The panel through the served artifacts

`crates/uor-r4-integer/src/bin/read-panel.rs` steps the held dump's 256 windows through the parsed
artifact and reads the Q31 read's trace and the relation's pick. Held ext-s1, 1,024 scored rows:

| Artifact | Arm | `chose_other` all | `K=1` other | asked | float parity |
|---|---|---|---|---|---|
| `ext-s1.uorlut01` (`/14`) | learned read | 51/1024 (0.050) | **1/47** | 440/1024 (0.43) | **438/1024 (0.428)** |
| `ext-s1-relation.uorlut01` (`/15`) | relation rule | **431/1024 (0.421)** | **47/47 (1.0)** | 0 | — |

- The relation `/15` passes through the **served integer artifact**: exact on every identifiable
  row, the all-rows ceiling, the asked fact suppressed.
- The learned `/14` read reproduces the C2′ negative (the `K=1` other 1/47) and exposes a large
  **export gap**: its picks agree with the float read on 0.428 of the rows, not the 0.984 the dump's
  converted gates gave. The FBS trunk's stream at the read depth is not the float replay's stream;
  the read's gates see different signs.
- The panel is a host run (34 s per artifact); no GPU.

## Still open

- The `/14` learned read's FBS input parity: the stream at the read depth and the replay's `e_x`
  differ; until that is aligned, the artifact carries the read but serves a different read.
- A bound export (contract + tokenizer).
- The 1B read is claude's; the discrete hinge still needs its Flock trainer.
