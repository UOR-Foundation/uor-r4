# ORDER: one-model-end-to-end, codex-21 (train the read on the stream the artifact serves)

Lab: codex. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md).
Parent: [codex-20](../codex-20/REPORT.md). Owner direction: align the learned read's input.

## Why

The `/14` learned read is trained on the float stream; the FBS artifact serves the read on the
integer trunk's stream. On held ext-s1 the artifact's stream at the read is at `e_x = 16` (the
replay's 24, ratio 0.0036); its signs agree with the float stream **0.887**; the read's picks agree
with the float read on **0.428** of the rows, and the asked fact on 0.43. The relation `/15` is
unaffected (47/47 on `K=1`). The read's gates are aligned to the wrong stream.

## Design

- **The FBS dump.** A `fbs-read-dump` bin: step the held and TRAIN windows through the parsed
  artifact, take each row's `Q31ReadStep.x_in` (the FBS stream at the read), the writer table at
  Q12 and the window ids, and write a `Q31ReadDump` (`read` = the artifact's `/14` read) beside the
  probe's `.ask/.other/.bucket` targets. A host run; the artifact is the source of truth.
- **The training.** `native_read_train` on the FBS dump: identity start, the A-ext asked target,
  the shift grid, 512 steps; the eval on the FBS held dump; the read's `chose_target` and
  `chose_other` per bucket. The weights' `e_w` is the FBS `35 - gain`; the picks are the gates.
- **The artifact.** The trained read's `to_bytes` under `Q31SessionRead` (the FBS depth), the
  `/14` sidecar, and `fbs-export --q31-read`.
- **The panel.** `read-panel` on the retrained artifact: the A-ext and the C2′ arms against the
  same targets; the relation `/15` as the separate rule artifact.

## Files

- `crates/uor-r4-integer/src/bin/fbs-read-dump.rs` (new) and `Cargo.toml`
- `crates/uor-r4-discrete/examples/native_read_train.rs` (the FBS dump's scale/gain)
- `crates/uor-r4-float-train/src/fbs_export.rs` (the sidecar's `e_w` from the FBS `e_x`)
- `docs/loop/tasks/one-model-end-to-end/codex-21/**`

## Done when

1. The FBS dump reproduces the artifact's own `x_in` rows and the table, and the trainer accepts it.
2. The retrained `/14` read's A-ext `chose_target` on the FBS held rows is measured; the C2′ arms are
   measured against the same targets.
3. The relation `/15` remains exact on `K=1`; the two artifacts are separate.
4. The comparison to the float-trained read (0.428 parity, 0.43 asked) is stated.

## Cancel

Stop if the FBS `x_in` rows are not reproducible across two runs, or if the trainer's target rule
cannot accept the FBS table's Q12 words. Report the gap, not a retry with a loosened gate.
