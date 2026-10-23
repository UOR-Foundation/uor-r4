# ORDER: infra-loop-harness, deepseek-56 (banned-presence pr-check rule)

Lab: deepseek. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904). Setup row: 2 (tooling). Base: `9a34eef06`. `References #2904`.

## Goal

Close the PM-accepted harness-gap card from #2028 (Claude 16:28Z, PM ruling 16:37Z): the D23/D25/D32 ratchets count banned source sites but cannot see them start compiling when a PR removes or weakens a `#[cfg(any())]` gate. Add a diff-level `loop pr-check` rule that fails when a PR removes a `#[cfg(any())]` gate, or moves the gated item to a test-only gate, on an item whose gated text carries a banned site; deleting the gated item stays lawful, keeping the gate stays lawful.

## Cancel probe (run first)

Class instrument/tooling (host-only; no GPU). At the base, `crates/uor-r4-harness/src/loopprcheck.rs` has no `banned-presence` entry in `RULES`, and PR #3079's old heads `1ddfa1cb` / `54de128d` pass pr-check while carrying un-gated D23 softmax/classifier heads. Task stands.

## Files in scope

- `crates/uor-r4-harness/src/loopprcheck.rs` (shared registration file: always allowed), `crates/uor-r4-harness/tests/loop_prcheck.rs` (item scope), `docs/harness/README.md` (shared registration file), this task folder.

## Done when

- `loop pr-check` runs a `banned-presence` rule that reports, per changed source file, each base `#[cfg(any())]`-gated item whose gate is removed or moved to `cfg(test)` at the head and whose gated text carries a D23/D25 site (D32 where its ratchet scans).
- Planted fault: against the real #3079 heads `1ddfa1cb` / `54de128d` the rule FAILS naming the 12 un-gated banned items (Softmax, SparseSoftmax, RelationRoute, SpanHead structs and impls) and passes against the re-gated head `8f36239a`; unregistering the rule makes the new tests fail; a benign un-gated item and a deleted banned item pass.
- Docs: the pr-check rules list in `docs/harness/README.md` describes the rule; `loop report --check`, `loop pr-check --local`, `uor-harness check`, fmt/clippy and the harness test suite pass; ratchets stay at baseline; counted non-author review.

## Stop and ask if

- The rule would need a charter change, an owner path, or a change outside the shared/loop files in scope.