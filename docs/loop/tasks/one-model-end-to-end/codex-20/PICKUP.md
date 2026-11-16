# PICKUP: the vector-read export (codex-20)

## Where it stands

The C2′ relation read is proven and shipped as `/15` on the synthetic FBS trunk; the real ext model
is blocked at the export, not at the read:

`fbs-snapshot <ext-s1>/models/step-016384 OUT` →
`Error: Invalid("the checkpoint holds vector_read.safetensors, which the export does not carry yet; exporting the trunk without it would serve another model")`

Reproduce with the c386 ext states at
`/tmp/opencode/c386-ext/c386/try-5/ext-flockg-b2-1/run/run` and `/tmp/opencode/c386-ext2/...`.

## The next unit

`codex-20/ORDER.md`: convert the `VectorRead` to the `/14` Q31 first-pick read, pin one
`(e_x, e_t, e_w, g)` for the FBS artifact and the read, carry it in the export, and run the C2′
panel through the served artifact (the learned read and the `/15` rule as separate artifacts).

## Checkable state

- `CARGO_TARGET_DIR=local/relation-target cargo test --locked -p uor-r4-integer --features
  serving-wasm,fbs-synthetic --lib` → 684 passed, 1 ignored.
- `fbs-snapshot` on the ext state → the refusal above.
- The relation `/15` artifact test: `the_artifact_carries_the_relation_read` passes.
- The float state reproduces the c387 probe (`first_right` 1000 / 990).

## Open elsewhere

- The C2′ control and panel evidence is in `codex-18/` and the control sheet.
- The 1B read is claude's starling line.
- Line 2: the corrected read-binding hinge needs an in-repo Flock trainer.
