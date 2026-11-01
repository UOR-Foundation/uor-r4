# REPORT: harden CPU FBS forward admission, deepseek-2

Lab: deepseek. Item: [cpu-shared-vector-forward](../../../../roadmap/items/cpu-shared-vector-forward.md) (#3504). Base: `070e35b59`.

## What did not work

- The merged first slice accepted a public `FbsImage` without proving that the artifact used the snapped transition and snapped Hamilton gates it implemented. It now accepts a parsed model instead.
- Raw-transition, raw-gate, retention-rate and non-rotating models would execute plausible but wrong recurrence arithmetic. They are refused until their branches are ported.
- Route atoms from a malformed constructed image could panic in gather. All six maps are now bounds-checked at construction.
- Context length was not enforced. The CPU forward now tracks position and refuses its first step past the serving context.

## What changed

- Replaced the public image constructor with `CpuFbsForward::from_model(&IntegerStackModel)`.
- Added strict mode validation and route atom bounds checks.
- Added context exhaustion parity.
- Added raw/retention artifact refusal tests.
- Preserved the one-, three- and six-layer full Q12 query/token/kept-cell parity tests and route-sign negative control.

## Measurements

- `cargo test --locked -p uor-r4-cpu-train`: 18 passed, 0 failed.
- Six-layer forward benchmark: 64 steps in 19.811 ms, 3,230.533 tokens/s, checksum `11743905789448238222`.
- The benchmark is a no-read synthetic forward, not training or language evidence.

## Checks run at the head

- `cargo test --locked -p uor-r4-cpu-train`: PASS, 18 tests.
- `cargo clippy --locked -p uor-r4-cpu-train --all-targets --no-deps -- -D warnings`: PASS.
- `cargo run --locked --release -p uor-r4-cpu-train --bin cpu-fbs-forward-bench`: PASS.
- `uor-harness loop gates --full`: PASS, 11/11.
- `uor-harness loop pr-check --local`: PASS with documented advisories.

## Skills used

No repository skill was invoked. Independent source and code reviewers identified the admission gaps.

## Still open

- Hamming read and exact turn-memory branches.
- Raw-gate, raw-transition, retention-rate and non-rotating model branches.
- Trained-artifact parity and training gradients.
- Item #3504 remains `ready`.

## Plain summary

The CPU FBS forward now refuses artifact modes it does not implement rather than silently computing a different model. The verified snapped no-read path remains exact against serving.
