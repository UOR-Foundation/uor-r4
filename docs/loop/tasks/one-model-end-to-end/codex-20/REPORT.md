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
