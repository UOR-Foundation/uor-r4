# ORDER: one-model-end-to-end, codex-19 (the relation read as an artifact field)

Lab: codex. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md).
Parent: [codex-18](../codex-18/REPORT.md). Owner direction: the C2′ control and the served read are
proven; ship the read in the artifact (ADR-0007: a new stage mechanism is a new schema field, not a
session flag).

## Why

The relation read is a runtime session setting (`set_relation_read`). The artifact does not carry it,
so a saved model cannot serve the mechanism without a caller configuring it. This unit adds the
`uor-r4.lut-stack/15` config-only sidecar and makes the session take the read from the artifact.

## Design

- `STACK_FBS_RELATION_SCHEMA = "uor-r4.lut-stack/15"`, a config record, no table:
  `relations: [lo, hi]`, `values: [lo, hi]`, `query_shift`, `buckets: [[lo, hi]; 2]`, `header`,
  `header_last`, `scale`.
- `add_relation_uorlut01` / `read_relation_uorlut01`, following `add_q31_read_uorlut01` /
  `read_q31_read_uorlut01`: parse the container, refuse a second relation or a conflicting optional
  record, set `header["relation"] = json!(record)` and `header["schema"]`, rebuild, read back.
- The container parse accepts `/15` with the relation record and the base FBS tables; a `/15` with
  no record, a second relation, an out-of-range bucket, `header == 0`, or a non-power scale is
  refused.
- `IntegerStackModel::parse` carries the relation into `FbsModel`; the session initializes its
  `ServedRelation` from it (the same `apply_relation` at the header's last row); a model without
  `/15` is byte-identical and its session has no relation.

## Files

- `crates/uor-r4-integer/src/stack/format.rs` (the schema, record, add/read, parse case)
- `crates/uor-r4-integer/src/stack/session.rs` (the `FbsModel` field and the session init)
- `crates/uor-r4-integer/src/stack/relation_read.rs` (tests if the conversion lives there)
- `docs/loop/tasks/one-model-end-to-end/codex-19/**`

Out of scope: changing the container version, the Hamming/place/Q31 records, the float trunk, and the
1B run.

## Done when

1. A synthetic trunk with the relation sidecar exports and parses; a session from the parsed bytes
   applies the pick at the header's last row (`last_relation_pick`), with no runtime setter.
2. A trunk without the sidecar is byte-identical and its session applies no relation.
3. The refusal cases are tested.
4. The C2′ panel runs through the parsed artifact (the host count plus the served session).
5. The integer library passes.

## Cancel

Stop if `/15` cannot be additive without a container-version change, or if the relation record must
carry a table. Redesign as a `/N` container field with the owner's decision.
