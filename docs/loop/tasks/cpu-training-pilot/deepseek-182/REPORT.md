# REPORT: cpu-training-pilot, deepseek-182

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](ORDER.md). Base: `5bd24915e`.

## Status

Complete. This releases the merged deepseek-181 writer-anchor audit claim.

## What did not work

Nothing failed in this coordination-only unit. The released audit remains an
instrument: it does not show that Claude's anchored writer run preserves its
ports until that run's artifact is supplied.

## What changed

- Removed the completed `deepseek/writer-anchor-audit` claim.
- Recorded PR #3732, reviewed head `1052f9255`, protected merge `5bd24915e`
  and the fresh-main verification in deepseek-181.
- Changed no model, serving, training, CUDA, float-inference, artifact or
  Claude-owned source.

## Measurements

- merged implementation head: `1052f9255`;
- protected merge: `5bd24915e92f434d902c55b0db46063f5ab40851`;
- fresh-main CPU library: **340 passed**, 4 expected ignores;
- fresh-main CPU binaries: **101 passed**;
- fresh-main strict Clippy, report/evidence, rustfmt and diff checks: passed;
- fresh-main loop gates: **11 / 11 green**.

## Checks run at the head

- Claims JSON, scope, ORDER, report, policy and loop gates.
- Exact-head review and all required checks before the protected merge.
- Fresh detached main at `5bd24915e`, including the full CPU package and direct
  evidence inspection.

## Size

This bookkeeping unit changes only its task records, the deepseek-181 delivery
record and the completed claim.

## Skills used

No system-provided Codex skill applied. Repository claim and delivery
procedures were used.

## Still open

Run `cpu-writer-anchor-audit` on Claude's first exported anchored training
artifact. A passing audit establishes port preservation after export, not a
writer-admission or language-quality gain.

## Plain summary

The writer-port audit is on main and independently verified there. This PR
removes its stale claim and records the merge and verification trail.
