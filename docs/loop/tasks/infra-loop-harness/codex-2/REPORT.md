# REPORT: infra-loop-harness, codex-2

Lab: codex. Order: [ORDER.md](ORDER.md).

## What did not work, first

- The codex-1 fresh-context review rejected the control because matching vocabulary was enough:
  “D85 is rejected; the float master remains the default” could pass a rule that only called
  `contains`.
- The first decision fixture similarly proved that a phrase was absent, not that the decision
  cell contradicted the policy. The old check also scanned the whole row, so a correct sentence
  in the rationale or rejected-alternatives column could satisfy it.
- The first model pages described the native path as implemented while the roadmap and REPORT
  said its state schema, operator registry, replay and trainer did not exist.
- The first report used a whole-repository formatter as task evidence; that formatter proposed
  thousands of unrelated historical changes. Only package-scoped Rust formatting is used here.
- The prebuilt `cargo xtask` alias was absent; the equivalent repository checks were invoked with
  `cargo run --locked -p xtask -- ...`.
- The full harness check still reports 311 existing warnings, including stale claims, presence
  and prep records. It has zero errors; those unrelated warnings are not hidden as passes.

## What changed

`native-space.json` now stores one canonical assertion. The checker requires its exact marker in
the charter, plan, roadmap item, protocol, change log and every model surface; requires the AGENTS
bullet and milestone rule to equal the assertion; and requires the D85 decision cell to equal the
canonical assertion plus its fixed historical-boundary suffix. Matching text cannot hide in the
rationale or rejected alternatives. The checker also proves every declared parity field and
forbidden default-path phrase is part of the canonical assertion.

The test suite now covers a complete fixture, every missing source, a malformed or contradictory
charter block, a decision corrected only in its rationale, wrong skill name and pointer, missing
plan/roadmap markers, a contradictory AGENTS sentence, a reworded milestone rule, missing model
markers and weakened canonical/parity/forbidden fields.

Model documentation now distinguishes the D85 contract from the implementation that is still
pending. `docs/harness/claims.json` is identified as generated claim state. The rule registry's
stale 53/13 prose is corrected to 54/14.

Size: about 730 added/deleted lines across 21 files, about 2.1 times the repair ORDER's 350-line
upper estimate. The overrun is the cost of replacing ambiguous phrase checks with exact markers in
every authoritative record and adding isolated contradiction tests. Size is reported, not capped
(D48); no unrelated source was changed.

## Measurements

- Native-space focused tests: 5 passed.
- Native-space rule on the real repository: 1 selected, 0 failed.
- Full harness inventory: 346 rules, 0 errors, 311 existing warnings, exit 0.
- Harness tests: 682 library tests passed, 1 ignored; every integration and doc test passed.
- Roadmap registry: 145 items valid, 26 ready, 11 held, 1 decision and 107 done.
- Stack contracts: 5 valid; generated views current.

No model, training, speed, capability or export result was measured.

## Checks run at the head

Run at the final task head unless explicitly identified as an earlier superseded run:

- `cargo fmt --package uor-r4-harness -- --check` - passed.
- `cargo fmt --package xtask -- --check` - passed.
- `git diff --check` - passed.
- `CARGO_TARGET_DIR=local/harness-target cargo test --locked -p uor-r4-harness` - all targets
  passed.
- `CARGO_TARGET_DIR=local/harness-target cargo clippy --locked -p uor-r4-harness --all-targets --
  -D warnings` - passed.
- `CARGO_TARGET_DIR=local/harness-target cargo build --locked --release -p uor-r4-harness` -
  passed.
- `UOR_LAB=codex local/harness-target/release/uor-harness check --only
  harness.native_space_contract` - passed.
- `UOR_LAB=codex local/harness-target/release/uor-harness check` - 346 rules, 0 errors, 311
  existing warnings.
- `CARGO_TARGET_DIR=local/harness-target cargo run --locked -p xtask -- stack-contract` - passed.
- `CARGO_TARGET_DIR=local/harness-target cargo run --locked -p xtask -- roadmap check` - passed.
- `UOR_LAB=codex local/harness-target/release/uor-harness loop report --check --order
  docs/loop/tasks/infra-loop-harness/codex-2/ORDER.md` - passed after this REPORT was added.

## Skills used

- `uor-research-architect`: kept the repair at the contract boundary and made no model claim.
- `working-discipline`: re-read the reviewed failure, used focused probes and reported actual
  commands and limitations.
- `scope-guard`: separated the active codex-2 repair from historical codex-1 and checked the
  generated claim record explicitly.
- `solution-integrity`: replaced token presence with a structured affirmation and planted
  contradiction cases.
- `native-space-cycle`: supplied the state, operator, transition, optimizer and serialization
  terms encoded in the canonical assertion.

## Still open

- The new repair head needs its own fresh-context exact-head review.
- The first native implementation remains `native-space-state-contract`; no model run is open.
- Existing float/export measurements remain historical and do not qualify D85.
- PR delivery, queue and post-merge verification remain after review.

## Plain summary

The first version checked that important words appeared somewhere in each file. That was too weak:
it could accept a sentence that rejected the policy. The repair defines one exact statement of the
policy and requires that statement, not a paraphrase, in each authoritative record. It also reads
only the D85 decision itself rather than letting matching words from the reasons or rejected
alternatives satisfy the check. Model pages now say clearly that the native path is the required
destination, while its state schema, shared operators and bit-identical replay are still the next
bounded implementation task.
