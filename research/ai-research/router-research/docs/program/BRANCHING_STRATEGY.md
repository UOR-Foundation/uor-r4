# Branching Strategy

## Purpose
Use git branches to separate hypothesis families, systems rescues, and packaging work so evidence stays reviewable.

## Current Constraint
The parent repo is still in an unborn state with no initial commit.
So this branch policy is fully defined, and the current working branch can be named, but multi-branch git history needs an initial commit before additional branch refs can be created normally.

## Default Pattern
- `main` or `master`
  - current reproducible frontier
  - current docs and tracking state
- `codex/RR-###-short-slug`
  - one active issue / increment per branch

## Branch Types
- Research increment branch
  - example: `codex/RR-052-retrieval-amortization-confirm`
  - contains only the code, configs, and docs needed for that increment
- Deep math branch
  - example: `codex/RR-050-dynamic-h4-state`
  - can diverge harder, but should not carry unrelated systems work
- Systems packaging branch
  - example: `codex/RR-053-index-reuse-packaging`
  - only opened after a research result justifies operationalization

## Rules
1. One hypothesis family per branch.
2. One merge-worthy decision per branch when possible.
3. Do not mix geometry-law changes with systems-harness changes unless the increment explicitly requires both.
4. Every branch must map back to one `RR-###` issue in `docs/program/ISSUE_REGISTRY.md`.
5. Before merge, update:
   - `docs/research/CURRENT_DIRECTION.md`
   - `docs/research/HANDOFF_CURRENT.md`
   - `docs/research/LIVE_WORKLOG.md`
   - `docs/research/PROGRESS_TRACE.md`
   - relevant increment doc
   - `docs/program/ISSUE_REGISTRY.md`
   - `docs/program/PROJECT_BOARD.md`

## Current Recommended Branch Map
- `codex/RR-052-retrieval-amortization-confirm`
  - closed negative systems branch
- `codex/RR-050-dynamic-h4-state`
  - current recommended next branch
- `codex/RR-053-index-reuse-packaging`
  - only reopen if a future translated branch clears confirm

## Merge Rule
Merge only when the branch has:
- config path
- analysis artifact
- gate note
- explicit keep/kill/refine decision

## Why This Helps Here
- keeps translated retrieval work from contaminating deep geometry branches
- makes agent fleets parallelizable
- makes post-compaction recovery much easier because branch name, issue ID, and increment doc all point to the same unit of work
