# REPORT: one-model-end-to-end, codex-19 (the relation read in the artifact)

Lab: codex. Order: [ORDER.md](ORDER.md).

## What did not work, first

- **The `/15` sidecar is exclusive.** `add_relation_uorlut01` refuses a stack that already carries
  another optional record (the Hamming, address, point, memory, Q31 or Q31 read). The relation read is
  a config-only read on a base FBS stack, so a model with a Hamming read cannot take it yet; the
  combinations are out of scope.
- **The record has no table.** The value row comes from the model's writer table; the artifact only
  names the fact encoding, the buckets, the header and the hard copy's scale. It does not add a new
  table or arithmetic.
- **A reader from before `/15` refuses the stack by its schema** (intended: it would otherwise serve
  without the read). The old artifacts are byte-identical and unchanged.
- **The runtime setter remains** for tests; the artifact path is the shipped one.
- No model or quality claim beyond the C2′ probe.

## What changed

- `crates/uor-r4-integer/src/stack/format.rs`: `STACK_FBS_RELATION_SCHEMA` (`uor-r4.lut-stack/15`),
  `StackRelation` (`check`, `read`), the `Header`/`Container` field, the `/15` parse case and record
  check, `add_relation_uorlut01`, `read_relation_uorlut01`.
- `crates/uor-r4-integer/src/stack/session.rs`: `FbsModel.relation` and the session's
  `ServedRelation` from the parsed artifact; the setter stays for tests.
- `crates/uor-r4-integer/src/stack/relation_read.rs`: the artifact test — export, add, parse, the
  session applies the pick (50) with no runtime setter; a second add and a zero scale are refused;
  the plain artifact applies none.
- `crates/uor-r4-integer/src/stack/mod.rs`: the exports.

## Measurements

- Artifact test: the session from the parsed `/15` bytes picks **50** at the header's last row; the
  plain artifact picks none.
- Seven relation-read tests; integer library **684 passed**, 1 ignored.
- The codex-18 panel and served answer are unchanged: the artifact configures the same rule.

## Checks run at the head

- `cargo test --locked -p uor-r4-integer --features serving-wasm,fbs-synthetic --lib` — 684 passed,
  1 ignored.
- `cargo clippy --locked --no-deps -p uor-r4-integer --features serving-wasm,fbs-synthetic --lib` —
  no warning in the new lines.
- `add_relation_uorlut01` reads the record back from the rebuilt container; the refusals are tested.

## Still open

- The relation read together with a Hamming/Q31 read on one stack (combinations).
- The C2′ panel through the served artifact on the real model: the probe's trunk is the float
  reference, not the integer artifact; the synthetic test proves the path.
- The 1B read is claude's; the discrete read-binding hinge still needs an in-repo Flock trainer.
