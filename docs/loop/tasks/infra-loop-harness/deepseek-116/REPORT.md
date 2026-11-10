# REPORT: infra-loop-harness, deepseek-116

Lab: deepseek. Order: [ORDER.md](ORDER.md). Base: `ce3c3e57b`.

## What did not work, first

- **No store artifact was available in this worktree, so the real
  cross-representation audit is `NOT_RUN`, not `MATCH`.** Running
  `stack-audit` against arm r validated all five fixture files and reported
  each as `NOT_RUN` because no artifact, tokenizer or chat executable was
  supplied. This proves the refusal path, not artifact parity.
- **Arm rk has no exported integer artifact.** The sealed run has checkpoints,
  but the report records neither an `.uorlut` hash nor a byte size. Its contract
  records `integer-export = not-run`; no hash was invented.
- **The first `rustfmt` pass followed out-of-line modules and touched unrelated
  integer-stack files.** Those accidental formatting edits were removed. Only
  the three files with intentional integer changes retain diffs.
- **Generated schema `/1` expectations failed after the renderer moved to
  `/2`.** The exact schema assertion was updated and now also checks the
  semantic hash and evidence-status table.
- **The local direct `cargo xtask` alias is absent.** Commands in this report
  use the package binary form
  `cargo run --offline -q -p xtask -- ...`; the checked-in documentation keeps
  the repository's established shorthand.

## What changed

Stack contract schema `/2` now records:

- exact training and serving domains, trained context and served context;
- exact artifact schema and section owner/format/exponent;
- structured evidence results and hashes;
- exact mechanism flags, dependencies, conflicts, sidecars and lifecycle order;
- reading units, directions and registered bars;
- hashed executable probe fixtures and required execution layers.

Schema `/1` remains readable. The compiler rejects missing v2 identities,
unresolved paths, incorrect hashes, exact-flag mismatches, self relations,
dependency cycles, duplicate load/cleanup orders, empty required collections
and malformed fixtures.

Four contracts now describe the active history:

| Contract | State | Schema | Fact |
| --- | --- | --- | --- |
| `fbs-arm-r` | current | `/7` | replicated at three seeds; 14,186,496-byte artifacts |
| `fbs-arm-rt` | candidate | `/9` | tail-nine memory chain; larger general-question gain and copied-value loss |
| `fbs-arm-rk` | historical | intended `/7` | measured-negative mixed-key configuration; no exported artifact |
| `fbs-b64l-baseline` | historical | intended `/6` | T0 negative; checkpoints not exported |

Five probe suites are checked in under `conformance/stack/fixtures/` for context
boundaries, copy behavior, memory precedence, cycle stops and representation
parity.

`uor-chat --script` runs machine-readable multi-turn scripts through the actual
product reply path. Each turn can set or clear automatic memory, or supply an
explicit memory value; the output records source, text, ids, stop, history size
and timing.

`cargo run -p xtask -- stack-audit` compares the contract against a run config,
checkpoint sidecars, artifact header, tokenizer, integer chat record and probe
fixtures. Its states are `MATCH`, `MISMATCH`, `UNAVAILABLE` and `NOT_RUN`; an
absent input cannot become a pass.

`fbs-export --stack-contract PATH` writes optional diagnostic provenance under
the artifact header's `source.stack_contract`. A focused test shows that:

- omitting the option is byte-for-byte the existing export;
- adding it changes only the diagnostic source object;
- shape and section inventory remain identical.

The generated model view now begins with an evidence-status table. Prep sheets
accept a stack-contract path, and new reports have a stack-contract section.

## Measurements

No model result was measured and no GPU was used.

- `cargo test --offline -p repo-model -p xtask`: **248 passed**, 0 failed
  (21 repo-model, 227 xtask).
- `cargo test --offline -p uor-r4-harness --lib`: **677 passed**, 1 ignored
  because it needs `gh` and network.
- `cargo test --offline -p uor-r4-integer --features serving-wasm --bin uor-chat`:
  **9 passed**, 0 failed.
- `cargo test --offline -p uor-r4-integer --features serving-wasm --bin fbs-export`:
  **3 passed**, 0 failed.
- `cargo test --offline -p uor-r4-integer --features serving-wasm --lib`:
  **517 passed**, 1 ignored because it needs `UOR_FBS_SNAPSHOT`.
- `stack-contract --write`: **4 contracts**; Markdown and JSON generated.
- Arm-r `stack-audit` with no artifact: overall **NOT_RUN**; five valid fixtures,
  each `NOT_RUN`.

## Stack contract

- Contract: `conformance/stack/fbs-arm-r.toml`, schema
  `uor-r4.stack-contract/2`, revision 2.
- Semantic hash at the checked-in source:
  `9a9cf79398532e7776077097b8ce6bf065b4bff8111183ee8c1e33c6f64e9027`.
- Candidate: `conformance/stack/fbs-arm-rt.toml`.
- Measured-negative history: `conformance/stack/fbs-arm-rk.toml`.
- T0 history: `conformance/stack/fbs-b64l-baseline.toml`.

## Checks run at the head

The final head's exact commands and results are appended to the PR review
record after the commit. This section records the pre-head checks named above;
it will not be treated as the review.

## Skills used

None. No available skill matched stack-contract schema repair, artifact
metadata auditing or the project's loop workflow.

## Still open

- Download an arm-r store artifact and run `stack-audit --artifact ...` with
  the chat binary. Until then, real cross-representation parity is `NOT_RUN`.
- The probe fixtures are executable, but no fixture has been passed against a
  local sealed artifact in this worktree.
- Arm rk remains without an exported integer artifact.
- Existing `UORLUT01` artifacts predate tokenizer provenance. New exports can
  bind the tokenizer SHA-256 under `--tokenizer`; an existing artifact reports
  that one diagnostic as `NOT_RUN` while its shape and serving path are still
  audited.
- The deterministic properties are held by executable Rust tests and fixtures,
  not by a Lean proof. Copy tie order, Hamming boundaries, memory precedence,
  EOS/cycle stops and artifact round trips retain their existing focused tests;
  this unit adds script and provenance coverage.
- A future report/prep renderer can embed the semantic hash automatically. This
  unit adds the input field and section but does not rewrite historical records.

## Plain summary

This change makes the project's description of a model precise enough to audit.
It names the exact training and serving shape, records what evidence supports
each mechanism, pins the flags and checkpoint files, hashes executable test
cases, and can compare those claims with a run configuration, checkpoint,
artifact, tokenizer and real chat process. When an input is missing it says so.
It does not claim that any model has become capable, and no model was trained or
served in this unit.
