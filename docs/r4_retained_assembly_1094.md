# Retained preparation assembly and launch gate — #1094

**Implementation checkpoint, 2026-09-03.** This step implements the
[preparation contract](r4_text_clause_preparation_1094.md) delivered in protected
PR #1099 at `6008e3527cb119d12af4abd99fe86a3d4ebe5a53`. Its named decision is
whether the exact retained evidence can be assembled and independently admitted
without repeating preparation or resetting its budget. No real comparison,
replay, preparation worker, fit or withheld access is part of this step.

## Implemented boundary

`text_clause_adapter.retained` is a standard-library-only metadata assembler.
It verifies the pinned original #1094/#1096 evidence, original public population
commitments and committed source closures. It keeps the accepted worker source
tree and binding bytes, generates the exact new binding-path profile, and
records the complete profile delta, clean child environment, runtime identities,
sealed-directory metadata and original time/byte debit. It neither imports the
adapter nor hashes/loads runtime or model assets during assembly.

`campaign run-retained` requires the distinct assembly and exact independent
`release.json` from its exclusive output. Lightweight path/provenance admission
precedes an exclusive `admission-started.json`; fresh source/runtime checks then
run under the execution clock. An interruption after admission consumes that
envelope. An exclusive `execution-started.json` must exist before the first
withheld payload hash/read. Reuse, legacy approval, path aliases and changed
identities fail closed.

Every snapshot carries the policy debit of **120 seconds** and original
**3,465,401 bytes**, counting the corpus once. Execution and replay retain their
120-second ceilings and the unchanged 360-second cumulative cap, 3-GiB combined
RSS bound, 128-MiB byte ceiling and 6,400 logical forwards. New output and active
oracle streams count in addition to history. A fresh output cannot reset it.

The actual worker receives the exact clean environment and unchanged accepted
worker source. Parent checks bind the profile, release, sources, eighteen
runtime files, both interpreter links, hardware and five assets before/after
worker attempts. The child retains its existing single harmless denied-read
receipt; this is not a new four-probe #1096 readiness result. If a resource stop
prevents a post-worker identity check, it is recorded as `NOT_VERIFIED`, and
post-worker diagnostic write errors do not replace the original cause. Any
`run-stopped.json` overrides completion. No automatic retry is admitted.

The accepted reader/core, adapter, worker, curator, vocabulary/query, four-fact
context, populations, full soft outputs/logits/decisions, refusal rules and
comparison criteria remain unchanged. Source additions are restricted to the
retained protocol; only the existing coordinator may change.

## Verification and evidence

The [independent source audit](r4_retained_assembly_1094_source_audit.md) follows
the knowledge index to the original contract and records the actual
NEMESIS/W33/UOR source pins. Their relevance is explicit admission, mapping and
identity. They supply no proof or model-preservation result.

Named synthetic checks exercise admission failure before payload/launch,
metadata/source/profile/approval drift, unsafe output and reuse rejection,
historical time/byte carry into replay, resource caps, clean environment and
pre/post-worker cleanup. They use temporary synthetic text and in-memory worker
stubs. They are code checks, not preparation, runtime readiness or empirical
model comparisons. Broad QA remains dormant; protected queue acknowledgements
are transport only.

This record will append the committed source, exact assembled envelope,
independent review and delivery evidence when each exists. The original
#1094 `UNAVAILABLE_REFERENCE_REPLAY`, #1096 `ISOLATED_RUNTIME_READY`, #1079
`LANGUAGE_R4_PRESERVED_CONTROL_WEAK` and #1082 descriptive limits are preserved.
Supplied segmentation remains the qualified model entry. #1094/#973 remain
open and #954 blocked until the separately defined scientific terminals change.

## Direct implementation check record

The root coordinator directly ran the two named synthetic check files: **7/7**
retained-metadata checks and **11/11** campaign admission/accounting checks passed.
The [complete transcript](r4_retained_assembly_1094_checks.txt) has SHA256
`85cfa49159f653e08fe8eecd716d31c80249359dada001df585da4c76597f91e`.
Claim-wording and diff-whitespace checks passed. No broader QA or real model
operation was used. The [independent implementation review](r4_retained_assembly_1094_review.md)
binds the four source/check files and accepts metadata assembly from committed bytes.
