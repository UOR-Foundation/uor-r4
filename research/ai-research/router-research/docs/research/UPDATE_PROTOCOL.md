# Research Update Protocol

## Purpose
Preserve enough state after every research increment that a later session can resume without reconstructing intent from raw logs.

## Required Updates After Every Closed Increment Or Control
1. Update the active increment or control doc with:
   - hypothesis
   - config path
   - analysis path
   - gate note path
   - screen/confirm/finalize outcome
   - explicit keep/kill/refine decision
2. Update:
   - `docs/research/CURRENT_DIRECTION.md`
   - `docs/research/LIVE_WORKLOG.md`
   - `docs/research/PROGRESS_TRACE.md`
   - `docs/program/ISSUE_REGISTRY.md`
   - `docs/program/PROJECT_BOARD.md`
   - `docs/DECISIONS.md`
   - `docs/routes/ROUTE_MATRIX.md`
   - `docs/reports/REAL_TASK_COMPARISON.md`
   - `results/INDEX.md`
3. If the result changes the open questions or research queue, update:
   - `docs/research/INCREMENTS.md`
   - `docs/research/OPEN_QUESTIONS.md`
   - `docs/research/FLEET_ASSIGNMENTS.md`
4. If a mechanism family name changes, update:
   - `docs/research/PHI_PHI_PHI_FAMILY.md`
   - `docs/research/PHI_PI_LOGPHI_PLAN.md`

## Minimum Resume Packet
If the session may be cut, make sure these files are current:
- `docs/research/CURRENT_DIRECTION.md`
- `docs/research/LIVE_WORKLOG.md`
- `docs/research/PROGRESS_TRACE.md`
- latest file in `docs/research/increments/` or `docs/research/controls/`
- latest gate note in `docs/governance/gates/`

## Long-Run Rule
- Before starting any sweep or confirm that may outlive the current context window:
  - append the exact next action to `docs/research/PROGRESS_TRACE.md`
- After the run lands:
  - append the exact artifact paths and promotion/kill decision to `docs/research/PROGRESS_TRACE.md`

## Branch Naming Rule
- Keep exact artifact labels in configs, logs, parsed JSON, and gate notes.
- Use mechanism-level names in research docs when a family becomes coherent.
- Always map the mechanism-level name back to the exact artifact label.

## Recommendation Rule
- Do not promote a route from screen to confirm without writing down why that promotion happened.
- Do not claim a runtime win unless it survives the relevant control batch.
- When a branch improves the routed family but still loses to `R0`, record both truths explicitly.
