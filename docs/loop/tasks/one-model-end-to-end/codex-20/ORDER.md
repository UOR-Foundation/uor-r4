# ORDER: one-model-end-to-end, codex-20 (carry the vector read in the FBS export)

Lab: codex. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md).
Parent: [codex-19](../codex-19/REPORT.md). Owner direction (2026-11-16): **option 1** — carry the
checkpoint's learned read in the export.

## Why

The ext-s1 checkpoint holds `vector_read.safetensors` (the model's learned Flock/Starling read).
`check_checkpoint_reads_carrying` refuses to drop it: "exporting the trunk without it would serve
another model". The integer artifact already serves a learned read as the `/14` Q31 first-pick read;
this unit converts the float vector read to that read, carries it in the export, and serves it.

## Design

- **The conversion.** `q31_read_from_vector_read(read: &VectorRead, stream_scale: ReplayScales) ->
  (Q31FirstPickRead, depth)`: the `(lanes, 4)` `query_rotation`/`key_rotation` to `Q31Quaternion`
  at `2^31`, the `width` weights to `i64` at `e_w`, and the gain shift. The `Q31FirstPickRead::new`
  shape is `(query_gates, key_gates, weights, gain_shift)`; `to_bytes` is what `/14` stores.
- **The scale.** The Q31 read's weights and the artifact's stream/table must share one `(e_x, e_t,
  e_w, g)`. The `native_replay` computes it from the model's table/weights and the windows' stream
  maxima; the FBS export's integers use the snapshot's own scales. The unit pins one scale for both:
  run the model on the held windows (the probe's own forward), compute `ReplayScales`, and export
  the FBS snapshot and the Q31 read at that scale. A mismatch is refused, not served.
- **The export.** `fbs-snapshot` converts the read after `StackModel::load` sets it, writes the
  Q31 read bytes beside the snapshot, and names `vector_read.safetensors` in the `carried` list only
  because the conversion is written. `fbs-export` adds the `/14` record with
  `add_q31_read_uorlut01`.
- **The relation `/15` is a separate arm.** `add_q31_read_uorlut01` and `add_relation_uorlut01` are
  exclusive (each refuses the other's optional record). The vector-read artifact serves the model's
  learned read; the relation artifact serves the rule. The C2′ panel runs both, one artifact each.

## Files

- `crates/uor-r4-float-train/src/fbs_export.rs` (the `carried` check and the conversion call)
- `crates/uor-r4-float-train/src/vector_reads.rs` (the `VectorRead` accessors the conversion needs)
- `crates/uor-r4-float-train/src/bin/fbs-snapshot.rs` (write the Q31 read beside the snapshot)
- `crates/uor-r4-integer/src/stack/q31_read.rs` / `format.rs` (the `/14` addition and a scale check)
- `crates/uor-r4-integer/src/stack/session.rs` (the served Q31 read, already present)
- `docs/loop/tasks/one-model-end-to-end/codex-20/**`

## Scale derivation (2026-11-16)

The FBS artifact's stream and table are Q12; the read sits after the MLP add at that scale. The
`add_payload` shift is `31 - gain_shift`, so `e_w = e_x - e_t + 31 - gain_shift = 31 - gain_shift`
for `e_x = e_t = 12`. The replay's read is at `e_w = 23` (its `e_t = 32`, `e_x = 24`, gain 0); the
FBS read is at `e_w = 29` with gain 2 (the read's max weight ~3.7 fits `2^31`). The picks are
scale-free (the gates are sign addresses); only the payload moves. `Q31FirstPickRead::rescale_payload`
implements the move and is tested: the picks are identical, the payload moves, an over-range weight
is refused.

## Done when

1. The ext-s1 checkpoint exports with the vector read; the artifact's Q31 read picks match the float
   read on the held windows within the export-gap bar, or the gap is reported.
2. The integer session serves the read from the parsed artifact.
3. The C2′ panel runs on the vector-read artifact (the model's learned read) and on the relation
   `/15` artifact (the rule), each through the served path.
4. A checkpoint with another unsupported read is still refused.

## Cancel

Stop if the Q31 read's scale cannot be aligned with the FBS artifact's stream/table without changing
the served arithmetic; report the gap and the exact scale mismatch.
