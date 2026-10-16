# Kimi PM bootstrap (kimi/pm1) — 2026-10-16

How to resume the UOR-R4 PM seat in a fresh window. Owner pastes the session
prompt; this file carries the operating details so nothing lives only in
context.

## Identity and seats

- Lab: `kimi`, session `pm1`, milestone `m1`. PM seat: routing, TEST FITNESS,
  merge-queue admission, merge verification, hygiene, rulings.
- **DeepSeek lab is shut down (owner, 2026-10-16).** Its seats run in-process:
  dispatch `Agent(subagent_type: "coder" or "uor-*", model default = DeepSeek
  flash)` for exact-head source review of Codex-authored code and other
  independent-review work. No more cross-session pastes for that seat.
- Codex: executor + pod recipe holder; source reviewer of non-Codex code.
  Reached via GitHub + owner paste (unreliable self-wake).
- AGY (antigravity/ag1): source-only, NO pods/GPU ever. Owner pastes prompts.
- Claude: retired until Saturday evening (owner, weekly cap).

## Every Bash call

`cwd: /Users/casey.allard/uor-r4/.worktrees/kimi-pm` — the PM worktree, branch
`kimi-pm-main`. After `git fetch origin`, sync with
`git checkout -B kimi-pm-main origin/main`. Owner checkout `~/uor-r4` is
READ-ONLY (ff-only pulls only). Harness: `UOR_LAB=kimi
~/.local/share/uor-r4/bin/uor-harness heartbeat -m m1 --session pm1 ...`
Cargo: `~/.cargo/bin/cargo` (shim broken). `gh` is authed; all labs post as
Casey-allard — reviews go as PR/issue comments, never `gh pr review`.

## Tick procedure (cron-driven, */12)

The tick is driven by a session-scoped cron job (`*/12 * * * *`, prompt =
the tick procedure below). **Cron jobs do not carry into a new window** — a
fresh PM window must recreate it: CronCreate with cron `*/12 * * * *` and a
prompt containing this procedure.

1. `git fetch origin`; note main movement; verify any merge:
   `git diff <approved-head> origin/main -- <delivered paths>` must be empty,
   merge commit = origin/main tip.
2. Board: `gh pr list --state open`. Queue only one PR at a time (R-queue):
   `gh pr merge N --auto` then GraphQL-verify `mergeQueueEntry` (CLI lies).
3. Comment scans:
   `gh api "repos/UOR-Foundation/uor-r4/issues/<N>/comments?since=<ISO>"`
   for N in 2028, 2029, plus any live PR.
4. Heartbeat every tick.
5. If quiet: fetch + scan + heartbeat, STOP. Do not invent work.

## Merge-forward pattern (force-push is blocked by Codex's hook)

`git merge origin/main` → union `claims.json` → newest-first
`current-state.md` → plain push → verify delivered-paths diff byte-identical
to approved head. Re-queue after checks go green; verify queue entry via
GraphQL.

## Gates in force

- Pre-merge pod transcript: any PR touching `#[cfg(feature="cuda")]` code
  needs pod-side `cargo check --features cuda` AND
  `cargo clippy --features cuda --all-targets -- -D warnings` transcripts at
  the exact head. Pod recipe: `scripts/pod/uor-cuda-check-remote` +
  `docs/labs/compute.md` (any lab can run it now; DeepSeek's by-use
  verification of the doc is still owed).
- No compute without pre-reg + TEST FITNESS (PM seat: dispatch independent
  reviewer subagent + verify myself). FIT releases design only.
- Pods only via `scripts/pod/uor-pod`, caps ≤4 pods / ≤$8/h, trap teardown,
  MD5 archive round-trip BEFORE teardown, df preflight ENFORCED (under 30Gi
  free = stop, not proceed-and-report).
- Training/M2/Stage B HELD; Q4 NOT_EXECUTABLE; seed-2 needs owner release.
- Negatives close configurations, never mechanisms (D22); arbitrary test
  values never veto mechanisms; no softmax/sigmoid/tanh/softplus/GELU/matmul
  anywhere.

## Lane state (2026-10-16 ~15:15Z)

- origin/main `90c2d2b9d`; board clean.
- Sealed: propose = 99.99% of batch (#2540); counterfactual kernel =
  99.9958–99.9981% of propose (#2543). Root cause: single 16-thread block,
  64 serial row revisits/lane, per-row baseline recompute, per-call allocs.
- In flight: Codex drafts counterfactual-kernel decomposition pre-reg
  (PM commission 6217437395) → PM-seat FIT → kernel optimization → 100-batch
  timing re-run → owner D-Release decision.
- AGY: RoPE interlock hardening (5 files, PM ruling 6218323961 on #2028).
- F2 closed by evidence (AGY audit 6206702939): RoPE excluded pre-merge,
  pinned by entry allowlist.
- Episode cost: ~$2.23 + receipts; storage/transfer UNMEASURED.

## Key references

- Tracker #2028, milestone #2029; current-state.md newest-first.
- Session-goal contract: docs/labs/session-goal.md.
- Owner escalations: mechanism stops, D-Release, new spending class.
