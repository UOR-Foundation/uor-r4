# REPORT: one-model-end-to-end, deepseek-273 (aligned-corpus GPU smoke)

Lab: deepseek. Item:
[one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md)
(#3407). Order: [ORDER.md](./ORDER.md). Base code: `7a450a88f`; launch code:
`894869f262973d64c6d44661fa71afaa7ebe9241`. All invoked-script hashes and
the exact reviewed execution-head procedure are in
[`launch-pins.env`](./launch-pins.env).

**Status:** pre-run, pending fresh-context review 4. No GPU job has started.

## What did not work

The first registration did not pass adversarial review: it changed the arm-r
batch without saying so, omitted the scenario panels from the execution path,
used an unbound legacy artifact, had no lawful non-neural baseline, and did not
seal failures or verify current compute capacity. A second review then found
that the source/job identities were only self-referential, the watcher had no
stall/cost control, scoring was not source-pinned, and the first failure seal
started too late. A third review found that the terminal marker preceded
readback, teardown did not wait for a verified seal, the cumulative cost and
time bounds were not implemented, scoring did not compare every row and
identity, the P1 classifier was wrong, and the stack contract omitted the
training seed. Those findings are repaired in the current registration.

## What changed

The scorer keeps the original schema `/1` when the control is off and emits
schema `/2` with an optional previous-token count table on exactly the selected
artifact cases. The planned run uses arm r's batch-16 geometry recipe plus
explicit train-as-served rounding, a complete receipt-bound input bundle, a
bound `/10` stack contract, exact source/job hashes, a same-built-in smoke
score, exact file-set failure sealing, a post-readback teardown sentinel,
persistent cumulative time/cost accounting, stage heartbeats and full
model/table/base row identity checks.

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

Review 4, the GPU smoke, bound export, held-out scenario reads, row
classification, final cost and post-run exact-head review remain. This file
will be replaced with the completed report before merge.

## Plain summary

The registration and its control instrument are now ready for review 4. No
model result has been claimed.
