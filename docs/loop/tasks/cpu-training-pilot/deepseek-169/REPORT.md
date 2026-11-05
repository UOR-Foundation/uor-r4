# REPORT: cpu-training-pilot, deepseek-169

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](ORDER.md). Base: `97c4c5a4a`.

## Status

Prepared and authorized. The fresh seed-2764 panel has not been generated.

## Question

Can candidate-conditioned one-step model lookahead improve exact-copy
arbitration after the no-lookahead learned read failed?

## What did not work

Deepseek-167's row-level learned readout tied longest-32 on one seed and gained
one body position on the other. This unit keeps the readout but scores the
state reached after each exact-copy candidate, directly isolating the value of
model lookahead.

## What changed

- Added learned and shuffled candidate-lookahead arms.
- Added cloned-forward candidate rollouts for unique depths one through four.
- Added candidate-conditioned next-state readout scoring and threshold
  fallback.
- Added independent tests for lookahead ordering, threshold fallback and
  decision identity.
- Extended the driver to compare lookahead against the previous no-lookahead
  learned read.

## Measurements

- implementation pin: `466a68b2fb240cb861d399124ce0eda046c1b74c`;
- fresh-panel result: **not run at registration**.

## Checks run at the head

- CPU library: **304 passed**, 4 expected ignores.
- All CPU-training binary test targets passed.
- Focused learned/lookahead tests: **9 passed**.
- Scoped Clippy with `-D warnings`: passed.
- rustfmt and `git diff --check`: passed.
- Fresh output root and seed 2764: absent before compute.

## What remains

Obtain FIT and GO, run both artifacts and deterministic repeats, publish compact
evidence, classify the result and complete reviewed delivery.

## Size

Final source, tests, preparation and compact evidence are measured at delivery.

## Skills used

No system-provided Codex skill applies. Project delivery and research
procedures were used.

## Still open

All measurement and delivery work is open. This report makes no complete-reply,
language or serving claim.

## Plain summary

Candidate-conditioned exact-model lookahead is implemented. Fresh evaluation is
pending.
