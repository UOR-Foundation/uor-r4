# Agent Rules (Codex)

## Git-first non-negotiables  ← READ THESE FIRST

- **The branch name is the canonical project path.** Do not work on `main`.
- On every fresh clone, run `bash setup_hooks.sh` before any research work.
  This activates the versioned git hooks in `.githooks/`.
- Use `make state` (not direct script calls) to validate research consistency.
- Use `make branch` or `make bootstrap` to reload context instead of manually
  reading dozens of markdown files.
- Create new branches with:
  ```
  make new-inc RR=<rr> INC=<inc> SLUG=<slug>
  ```
  Never `git checkout -b` a research branch manually — the Makefile target
  also creates the increment doc skeleton and fires the context hook.
- Commit messages on `codex/*` branches must begin with `[RR-###]`. The
  `prepare-commit-msg` hook provides the template automatically.
- `git push` is blocked by `pre-push` if canonical docs are inconsistent.
  Fix state to pass `make state` before pushing. Never use `--no-verify`
  without immediately filing a follow-up commit to repair state.
- Full onboarding guide: `docs/GIT_ONBOARDING.md`.

## Non-negotiables
- Keep diffs small and PR-shaped.
- Always keep the repo runnable.
- Start every resumed research session with `make bootstrap` (replaces
  manually reading `docs/SESSION_BOOTSTRAP.md`).
- Always reload `CORE_PROJECT_GOALS.md` before choosing the next research step.
- Treat `docs/research/ACTIVE_STATE.md` as the single live queue authority.
- Treat `docs/research/KILL_LIST_TRACKER.md` as the single kill-list status authority.
- Maintain `docs/research/SESSION_LEDGER.md` during long investigations,
  branch pivots, and post-compaction recovery.
- Never use `CURRENT_DIRECTION.md`, `HANDOFF_CURRENT.md`, or a single increment doc
  as the only source of truth.
- If `PROJECT_CONTEXT.md`, `KILL_LIST_TRACKER.md`, `ACTIVE_STATE.md`,
  `ROUTE_MATRIX.md`, and handoff/current docs disagree,
  reconcile that conflict before changing code or rewriting docs.
- After compaction or context loss, rerun in order:
  1. `make state`          — validates canonical docs
  2. `make branch`         — prints current branch context
  3. `make bootstrap`      — prints full startup context packet
  4. Read `docs/research/SESSION_LEDGER.md`
- Before queuing a new branch, be able to say:
  - which kill-list stage it advances
  - which mathematical object it tests
  - why it is not just packet/contract cleanup
- If a later translated, packet, or downstream evaluation branch is active
  while an earlier kill-list gate remains unresolved, explicitly justify that
  choice in the queue docs before continuing.
- Every run produces:
  - a log in `results/raw/`
  - a parsed JSON in `results/parsed/`
  - an appended row in `results/summary.csv`
- Add/update `docs/DECISIONS.md` when conclusions change.

## Experiment protocol
- Screen (1 seed) -> confirm (2 seeds) -> finalize (4 seeds).
- No massive grid sweeps unless there is a decision gate.

## Primary mission
Prove or falsify geometry-native routing as a hardware-relevant alternative to
dense transformer-style routing.

Secondary mission:
- cut runtime
- automate experiments
- keep the human out of the terminal

Those are support goals, not replacements for the core project thesis.
