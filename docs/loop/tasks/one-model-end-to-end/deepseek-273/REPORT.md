# REPORT: one-model-end-to-end, deepseek-273 (aligned-corpus GPU smoke)

Lab: deepseek. Item:
[one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md)
(#3407). Order: [ORDER.md](./ORDER.md). Base code: `7a450a88f`; launch code:
`f7f7ac97c67a784b7261abdbdb95cf0fa785f241`; job SHA-256:
`0dbd7f05ba6c1214f6e3f8ff2e7dc1c2ded53966e5f577b98445d0d2aa5adfe4`.

**Status:** pre-run. No GPU job has started.

## What did not work

The first registration did not pass adversarial review: it changed the arm-r
batch without saying so, omitted the scenario panels from the execution path,
used an unbound legacy artifact, had no lawful non-neural baseline, and did not
seal failures or verify current compute capacity. A second review then found
that the source/job identities were only self-referential, the watcher had no
stall/cost control, scoring was not source-pinned, and the first failure seal
started too late. Those findings were repaired before launch.

## What changed

The scorer keeps the original schema `/1` when the control is off and emits
schema `/2` with an optional previous-token count table on exactly the selected
artifact cases. The planned run uses arm r's batch-16 geometry recipe plus
explicit train-as-served rounding, a complete receipt-bound input bundle, a
bound `/10` stack contract, exact source/job hashes, a same-built-in smoke
score, failure sealing, and a cumulative cost review.

## Measurements

No GPU measurement exists yet. Preflight arm-r and count-table readings are in
the prep sheet and its HF-store paths. Arm-r reproduced 10 development and 11
Fresh copy hits, zero grounded hits, zero abstentions, and zero distractors;
the count table produced zero on every scored metric.

## Checks run at the head

- `scenario_grade`: 10 passed.
- CPU scenario-grade CLI: 3 passed.
- Full `uor-r4-cpu-train` library: 505 passed, 5 ignored.
- Stack-contract generation and check: 5 contracts current.
- Shell syntax and ShellCheck: pass.
- Clippy with `-D warnings`: stops on pre-existing warnings in
  `prompt_reply_rank.rs` and `suffix_arbitration.rs`, outside this change.

## Skills used

`working-discipline`, `solution-integrity`, `scope-guard` and `context-guard`
kept the correction bounded and checked against the registered evidence path.

## Still open

The GPU smoke, bound export, held-out scenario reads, row classification, final
cost and post-run exact-head review remain. This file will be replaced with the
completed report before merge.

## Plain summary

The registration and its control instrument are now ready for a second
adversarial review. No model result has been claimed.
