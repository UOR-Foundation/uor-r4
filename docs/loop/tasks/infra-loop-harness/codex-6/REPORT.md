# REPORT for the November 17 public documentation scope

Lab: codex. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md). [ORDER](ORDER.md). References #3380 and #4060.

## What did not work

The scope check on #4060 reported forty roadmap source files outside the docs-status-refresh registration at its merge base. The existing delivery instructions allowed the generated roadmap only after the input merge, although the proposed content includes a generated preview for review.

## What changed

Appended each missing path to the registered scope. The delivery instructions allow a preview generated from the proposed roadmap inputs and require live synchronization, a fresh render and comparison with the merged sources after the PR lands. STATE and HANDOFF still use their existing loop render workflow.

## Measurements

The complete comparison covers 66 changed paths at content head `9f8f2ec1efb4c6b1c80d138c6c48a138d9d831c2`. Thirty-nine newly named paths are roadmap item files. The fortieth is the CPU training pilot's preserved roadmap record. The content ORDER and REPORT reside in the active task folder, and the generator's README is a shared path under the policy.

## Checks run at the head

Compared the added scope entries, in order, with every path reported by the base policy. The old list is an exact prefix of the new list. The generator command matches the command recorded in the generated roadmap. The whitespace check passed.

## Skills used

Roadmap source inspection and the repository's local PR policy.

## Still open

The content PR owns the public prose, item dispositions and demonstrations. Its live issue synchronization and readback run after merge.

## Plain summary

The registration covers the forty additional roadmap sources for the public refresh. Readers can review the generated roadmap alongside its inputs, then check it against the live tracker after delivery.
