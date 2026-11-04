# REPORT: public 19-table CPU count training, deepseek-124

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](ORDER.md). Prep:
[trunk count training](../../../harness/prep/cpu-training-pilot-trunk-count-2026-11-04.md).
Base: `2326bece3`.

## What did not work, first

This is the in-progress record. No public comparison has run. The current CPU
trunk reader is read-only and the four-table corpus driver correctly refuses
the 19-table M1 artifact.

The first FIT review returned **NOT_FIT**. Aggregate table totals did not prove
which `(key, token, count)` entries changed, and the unigram best-token tie had
no negative test. The driver now independently rebuilds every expected table
from the training rows and refuses any decoded difference; a finite tie test
requires the smaller token id, the exact and shuffled artifacts must differ,
and changed served-decision rows are counted rather than represented only by a
boolean checksum difference.

## What changed

Implementation will add exact CPU derivation of trunk training rows and checked
additive updates across the unigram and 12 sparse count tables, then export and
re-import the complete 19-table artifact.

## Measurements

None yet. The pre-registration is PENDING and must be FIT before public compute.

## Checks run at the head

Focused finite tests and final checks will be recorded here.

## Skills used

No system-provided skill applied.

## Still open

- Implement and FIT-review the writeback path.
- Run the document-disjoint public artifact evaluation.
- Record final evidence, notebook/roadmap updates and delivery.

## Plain summary

This unit makes the project's public 19-table model trainable on CPU rather than
read-only, while checking every decision through both CPU and integer serving.
