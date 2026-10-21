# REPORT: infra-loop-harness, claude-3 (P-1 and P-2 re-run for Claude Code)

Lab: claude. Order: [ORDER.md](ORDER.md). The cancel lines are the ones in [claude-2's ORDER](../claude-2/ORDER.md), committed before the first run.

## What did not work, first

- **The first stop test stopped the session too early.** It sent SIGTERM before the session had started its tool, so it could not show whether a running tool outlives the stop. It was repeated: the second attempt waited until the PreToolUse hook showed the tool running, then stopped the session.
- **The compaction hook is still untested.** It is configured, but a run this short never compacts.
- **The session ignored the requested reply word.** It answered with a summary instead of "DONE", which does not affect any probe.

## What was measured

All runs used `claude` 2.1.288 with `--model claude-haiku-5-5`, in `local/claude/loop-probes/claude-p1/` (not committed). The commands and outputs are in the [#2028 comment](https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6306306665).

| Probe | Result | Evidence |
|---|---|---|
| P-1, Claude Code | passes, except compaction (untested) | One headless run exited 0 in 19.1 s and fired these events in order: SessionStart; PreToolUse (Bash `echo top-level`); PreToolUse (Agent); PreToolUse (Bash `echo inside-subagent`, with `agent_id`); Stop; SubagentStop (same `agent_id`); Stop. |
| P-2, Claude Code | passes | The sign-in persisted, there were no prompts, and only `claude-haiku-5-5` appears in the model usage. The session pid was 11781 (group 11780); `sleep 60` ran as pid 12168 (group 12167). `kill -TERM 11781` ended the session about 2 s later (`wait` exit 143), and `sleep 60` was gone afterwards. |

## What it means for the build

- **Subagent fences.** Subagent scopes can be enforced mechanically in Claude Code: the PreToolUse event names the calling subagent, so the guard (PR 9) can apply a narrower sub-order to that subagent alone.
- **Stopping a session.** The supervisor (PR 10) must signal the session pid, not the process group: the tool runs in a group of its own, and Claude Code cleans it up when its own pid gets SIGTERM.
- **Startup time.** The first tool call comes about 20 s after launch. The cap clock should start at launch, and a "no activity" stall check should allow for this.

## What changed

| File | Lines | Class | What |
|---|---|---|---|
| `docs/loop/tasks/infra-loop-harness/claude-3/ORDER.md` | new | docs | the re-run order |
| `docs/loop/tasks/infra-loop-harness/claude-3/REPORT.md` | new | docs | this report |
| `docs/loop/README.md` | 2 cells | docs | Claude Code's P-1 and P-2 results |

Nothing outside scope was touched, and no global client settings were changed.

## Still open

- Codex, Kimi, Antigravity and DeepSeek each still owe P-1 and P-2 for their own clients (#2028).
- The compaction hook, for every client.

## Plain summary

- **Claude Code can be driven unattended.** It runs headless with a pinned model, its hooks see every tool call, including a subagent's, and stopping it by pid cleans up its running tools.
- **The other four clients still need their own probes.**
