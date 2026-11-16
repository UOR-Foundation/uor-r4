# ORDER: one-model-end-to-end, codex-23 (the `Read` seam, the bound form, and the ledger)

Lab: codex. Items: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) and
[end-to-end-integration](../../../../roadmap/items/end-to-end-integration.md). Parent:
[codex-22](../codex-22/REPORT.md). Owner direction: resolve the gaps between the proved mechanisms
and one served model.

## Why

ADR-0007 is accepted, and its I1 is the one piece that makes the read mechanisms comparable: a
single `Read` stage interface whose reference is the native Q31 first-pick read. Today every read is
a bespoke call and a bespoke schema (`/14`, `/15`), the bound product artifact (`/10`) refuses
reads, and there is no standard exactness reading. That is three of the gaps named in codex-22's
close, and each is buildable without a training run.

The fourth gap — the `Context` ring, I2 — is a design unit with a witness-pair gate; it is ordered
here, not built. The fifth — the protocol run — needs a run that carries a contract and a tokenizer.

## Design

1. **The seam.** `ReadMechanism`, `Query`, `ContextView` and `Read::select(&Query, &mut
   ContextView) -> Result<ReadOut>`; the result is the existing `ReadOut`. The reference adapter is
   `ServedQ31Read`, calling exactly `read_out` — the call the session made before the seam, so the
   interface lands bit-identically (ADR §1 rule 2). `RelationRead` is the second consumer.
   The view carries the reference read's scan because the session owns it and `reset` clears it;
   the trait stays `&self`.
2. **The bound form.** A bound `/10` stack gains the native read as `/17`: the binding is kept, the
   schema says the artifact needs an engine that serves the read, and an engine that does not know
   `/17` refuses it. The relation read stays refused on a bound stack (ADR parks it; one mechanism
   per stage per artifact).
3. **The ledger.** `ReadLedger::compare`/`compare_on` over two pick arrays: rows, agreement,
   first divergence (absolute row), and rows a second at a supplied clock. A `read-ledger` bin.
4. **The renumber.** The relation record moves `/15` → `/16`, because claude-384's line claims
   `/15` for the pair rows.

## Files

- `crates/uor-r4-integer/src/stack/read_stage.rs` (new), `read_ledger.rs` (new), `read_out.rs`
- `crates/uor-r4-integer/src/stack/q31_read.rs`, `relation_read.rs`, `format.rs`, `session.rs`, `mod.rs`
- `crates/uor-r4-integer/src/bin/read-ledger.rs` (new) and `Cargo.toml`
- `docs/loop/tasks/one-model-end-to-end/codex-23/**`

## Done when

1. The seam has a bit-identical adapter test against the session's own read, and a second consumer.
2. A bound stack carries the read as `/17`; the binding and the read both survive; an engine
   refusing the read refuses the schema.
3. The ledger reproduces the panel's exactness on the real probes (C1 440/1,024; the FBS-trained
   read 1,015/1,024) and names the first divergent row.
4. The relation artifact re-reads at `/16` with the same 47/47.
5. Every test suite at the head is green.

## Cancel

Stop if the seam changes a pick or a payload on any fixture, if `/17` cannot keep the binding
through the add and the parse, or if the ledger disagrees with the panel on the same inputs.
