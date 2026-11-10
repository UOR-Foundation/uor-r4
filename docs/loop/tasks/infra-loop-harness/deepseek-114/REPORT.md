# REPORT: infra-loop-harness, deepseek-114

Lab: deepseek. Order: [ORDER.md](ORDER.md). Base: `8a5bd5278`.

## What did not work, first

- The first push was refused because this code unit had no ORDER or REPORT.
  `loop pr-check` reported both failures before any branch reached GitHub.
- A workspace-wide `cargo fmt --all -- --check` found extensive formatting drift
  outside this unit. Formatting was restricted to the two new Rust files;
  package-scoped fmt passed without rewriting unrelated sources.
- `cargo clippy -p xtask --all-targets -- -D warnings` reached two pre-existing
  `book_stage.rs` and `docs_relink.rs` findings. The run passed after allowing
  only those two unrelated lints; `repo-model` passed strict Clippy unchanged.
- The full `uor-harness check` reports hundreds of pre-existing warnings and
  exits successfully. They are not treated as evidence for this unit.

## What changed

Added `uor-r4.stack-contract/1` as a typed TOML contract. It records:

- model lane, state, source pin, tokenizer and input hashes;
- fixed model shape and artifact family/sections;
- mechanism status, implementation and evidence paths, invariants and
  non-claims;
- reproduction commands and distinct structural, semantic, surface, parity and
  performance readings;
- boundary, metamorphic, parity, malformed and abstention probes;
- explicit statements the contract does not establish.

The first contract is `fbs-b64l-baseline`, the historical FBS T0 configuration.
It is deliberately not presented as the current Claude memory/copy stack.

Added `cargo xtask stack-contract --write|--check`. It validates the contracts
and deterministically regenerates:

- `docs/model/STACK-CONTRACT.md`;
- `conformance/generated/stack-contracts.json`.

Size: **code 1,059 lines; tests inside those files 193 lines; documentation 123
lines; generated and contract data 612 lines**. Estimate was 1,700 lines; the
actual change is 1,766 lines, a ratio of 1.04.

## Measurements

No model result or performance measurement was taken. This is build-time
tooling and has no GPU cost.

`cargo test --offline -p repo-model -p xtask`: **239 passed**, 0 failed. This
includes 7 new contract tests and 2 generator tests.

`cargo xtask stack-contract --check`: **1 contract, generated files current**.

## Checks run at the head

- `cargo fmt --check --package repo-model --package xtask`: pass.
- `cargo test --offline -p repo-model -p xtask`: 239 passed, 0 failed.
- `cargo clippy --offline -p repo-model --all-targets -- -D warnings`: pass.
- `cargo clippy --offline -p xtask --all-targets -- -A
  clippy::unnecessary_map_or -A clippy::manual_repeat_n -D warnings`: pass; the
  two allowances name unrelated pre-existing findings.
- `cargo run --offline -p xtask -- stack-contract --check`: pass.
- `cargo run --offline -p xtask -- check-model`: pass, 33 conformance IDs.
- `uor-harness check --repo .`: exit 0 with repository-wide pre-existing
  warnings.
- `git diff --check`: pass.

## Skills used

None. No available skill matched typed contract generation and repository
validation.

## Still open

- Encode the current Claude arm-r memory/copy configuration after that branch
  lands; do not infer its unmerged fields on this branch.
- Add generated trainer/export/session requirements only after this schema is
  stable.
- Add semantic-trace probes through every execution layer.
- Formal proof integration and a proof build remain separate work.

## Plain summary

The repository now has one typed file that states the model's shape, mechanisms,
artifact sections, commands, measurements and boundary cases. A command checks
that file and regenerates two readable views. If either view is stale or the
contract is internally inconsistent, the command fails. This helps a new session
reconstruct the model without guessing, but it does not make the model smarter
and does not claim that any model can chat.
