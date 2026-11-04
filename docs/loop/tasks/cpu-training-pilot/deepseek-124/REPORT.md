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

The second review returned FIT at docs head
`3ff9ca748ad76cd22f5fa2ae8c27e35a1ac4c249`, bound to source
`b70ae7de6c2a6fe0946ca14fffe0a63bb62525d2`.

That FIT run was executed inadvertently. It completed the mechanism checks but
reached **3,063,024 KiB** peak RSS, above the registered 2 GiB cap. The run is
preserved as a resource-stop attempt at
`local/deepseek/trunk-count/2026-11-04-attempt-1/`; its outputs are not accepted
as this unit's evidence. Source `d5108222a` drops the source serving model and
the training reader before later verification and evaluation. Attempt 2 uses a
fresh build and output root.

The third review returned FIT at docs head
`e94f21783e10e1a1d570d34d634698ea870657a0`, bound to source
`d5108222aef6882007aba31eeadc95796b9c8aca`.

Attempt 2 also hit the registered stop at **2,297,824 KiB**, despite dropping
the model and reader. It is preserved at
`local/deepseek/trunk-count/2026-11-04-attempt-2/` and is not accepted.
Source `1ca202ef2` now consumes each training reader, writes and releases one
artifact before constructing the other, and reloads each artifact only for its
own verification/evaluation phase. Attempt 3 uses new roots and requires a new
FIT.

## What changed

Implementation will add exact CPU derivation of trunk training rows and checked
additive updates across the unigram and 12 sparse count tables, then export and
re-import the complete 19-table artifact.

## Measurements

None yet.

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
