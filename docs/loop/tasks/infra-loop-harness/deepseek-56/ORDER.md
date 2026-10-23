# ORDER: infra-loop-harness, deepseek-56 (banned-presence pr-check rule)

Lab: deepseek. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904). Setup row: 2 (tooling). Base: `018954726`. `References #2904`.

## Goal

Close the PM-accepted harness-gap card from #2028 (Claude 16:28Z, PM ruling 16:37Z): the D23/D25/D32 ratchets count banned source sites but cannot see them start compiling when a PR removes or weakens a `#[cfg(any())]` gate. Add a diff-level `loop pr-check` rule that compares the banned sites that compile in each changed source file between the merge base and the head and fails when the head compiles more of any family.

## Cancel probe (run first)

Class instrument/tooling (host-only; no GPU). At the base, `crates/uor-r4-harness/src/loopprcheck.rs` has no `banned-presence` entry in `RULES`, and PR #3079's old heads `1ddfa1cb` / `54de128d` pass pr-check while 19 D23 sites change from gated to compiled. Task stands.

## Files in scope

- `crates/uor-r4-harness/src/loopprcheck.rs` (shared registration file: always allowed), `crates/uor-r4-harness/tests/loop_prcheck.rs` (item scope), `docs/harness/README.md` (shared registration file), this task folder.

## Done when

- `loop pr-check` runs a `banned-presence` rule that counts D23 (word + identifier families with the ratchet's rope exception), D25 (dense-product/tensor-core identifiers) and, in the D32 scan scope only, D32 (float types and literals) sites in compiled code per changed source file, at the merge base and the head, and fails when the head's count rises for any family.
- Compiled code means the file with `#[cfg(any())]`-gated spans blanked (a gate on a parent module covers nested items) and comments, strings and char literals stripped; `cfg(test)` is not an exemption.
- Planted fault: against PR #3079's violating heads `1ddfa1cb` / `54de128d` the rule FAILS (`relation_compiler.rs`: D23 0 -> 19) and against the re-gated head `8f36239a` it passes.
- An independent adversarial review's cases are regression tests: rename of a banned item fails; benign/properties/char-literal items pass; deletion passes; parent-module gate passes; D32 float function un-gated fails, kept-gated passes.
- Docs: `docs/harness/README.md` describes the delta rule; `loop report --check`, `loop pr-check --local`, fmt/clippy, the harness test suite and the ratchets stay green/baseline; counted non-author review.

## Stop and ask if

- The rule would need a charter change, an owner path, or a change outside the shared/loop files in scope.