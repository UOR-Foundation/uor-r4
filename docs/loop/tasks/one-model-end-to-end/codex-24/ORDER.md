# ORDER: one-model-end-to-end, codex-24 (the verdict rule as a command)

Lab: codex. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md).
Parent: [codex-23](../codex-23/REPORT.md). Owner direction: make the run's judgment executable
before the run.

## Why

Done-when 3 fixes the verdict rule in prose: "memorized" when the gains appear only on keyed rows
and generated replies are mostly TRAIN replies, "generalizes" when the arm beats (r0) and the
longest-suffix table on unkeyed and new-template rows with novelty near DEV's own replies. A rule
that is applied by hand after a run is not pre-registered in any enforceable sense. This unit makes
it a function with tests and a bin, and it repairs the instrument lane that produces the readings:
`docs/labs/one-model-end-to-end/instrument/src/lib.rs` registers neither `evidence` nor
`carry_geometry`, so `m2-evidence` and `real-carry-geometry` cannot build on `origin/main`.

## Design

- Register the two modules under `cpu-parity`, the feature the bins already require.
- `verdict.rs`: `Stratum { targets, correct }`, `ArmStrata { keyed, unkeyed, new_template }`,
  `VerdictInput` (the arm, `memory_off` as (r0), `table` as the longest-suffix control, the reply
  share, the novelty shares and two sheet-fixed knobs), `Verdict`, `VerdictReport`, `verdict()`.
- The rule refuses to choose when a reading it needs is absent: an empty stratum or a missing
  novelty share yields `Undetermined { reason }`, never a guess.
- `protocol-row` bin: read a `VerdictInput` JSON, print or write the report; never write over.
- The two knobs the item does not fix numerically — "mostly" and how near DEV's novelty must be —
  are inputs, not code: the sheet sets them.

## Done when

1. The rule's cases are tested: memorized, generalizes, mixed on far novelty, mixed with no gains,
   mixed with few TRAIN replies, undetermined on an empty stratum, undetermined on missing novelty,
   and the integer shares.
2. `m2-evidence` and `real-carry-geometry` compile again under `cpu-parity`.
3. `protocol-row` runs on a fixture and prints the report.

## Cancel

Stop if the rule needs a number the item does not fix; it goes into the input, not the code. Do not
report a verdict from a panel that is not the protocol's.
