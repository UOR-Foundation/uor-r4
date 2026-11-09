# REPORT: close remaining PTY identity races and reconcile delivery evidence, kimi-6

Lab: kimi. Item: [studio-ide-harness](../../../../roadmap/items/studio-ide-harness.md) (#3569). Order: [ORDER.md](ORDER.md). Base: `6fa2baf377e74080e57cb64330f091b701059e01`, tree `36d5b6701c9d3ee9bfdbecde9a21de011b4c0aca`; merged main `1e5e84a1d497222099a97e22da024b07759750a8`. Fresh reviewer agent-67 returned CHANGES, recorded on PR #3890 at that head. The PR remains draft and unqueued.

## What did not work

- Agent-67 found that the mutable session id was assigned after the opened snapshot reached component state, leaving an unmount/workspace window that closed nothing. A rejected stale close discarded the returned backend id, and a normal close rejection could restore the old workspace's session. The stale-open tests only used successful close mocks.
- Red-first tests reproduced all three defects. `closes an opened shell when unmount happens before the session effect` observed **zero close calls**. `retains a rejected stale close for retry without mounting it` had no Retry close control. `does not restore an old session when its close fails after a workspace change` restored `session-a`.
- The first read could advance the cursor before xterm mounted; the first-output regression observed **zero writes**. A second shell never emitted a fresh truncation notice. The strengthened observer regression required two distinct notices and observed one.
- The first implementation pass broke two existing contracts: adopting a read snapshot without checking its session id could replace the tracked identity, and removing same-workspace session restoration made the original close-failure regression fail. Those are corrected: mismatched read responses are discarded; only normal same-workspace close failures restore visible identity, and stale/cross-workspace failures expose cleanup retry without mounting.
- The `58d14a29…` directory and `interactive-terminal-clean.log` were worktree-only. The roadmap called the PTY missing, named kimi-4 current, and omitted changed Tauri manifests from scope. kimi-4 said 444 tests, kimi-5 named ec49, and the 445-test notebook row linked to kimi-4. Historical package statements and host identities needed explicit scope.

## What changed

- The mutable runtime now records the complete shell snapshot immediately when `openTerminal` resolves, before state/effect scheduling. Workspace changes update a synchronous revision ref and close through that identity. Stale opens always close without restoring/mounting; failed closes preserve the snapshot for a bounded user retry rather than silently dropping it.
- Normal same-workspace close failure retains visible session identity with the terminal disposed. Stale or cross-workspace failure exposes `Retry close` with no active session or polling. Opening is blocked while any shell or cleanup remains. Close completion cannot restore the old active session after a workspace revision.
- Reads start only after xterm mounts and are rejected if the response belongs to another session. Truncation notice state resets for each new mounted session. Close, input and resize paths check the tracked snapshot identity and disposal state.
- Item scope now includes `src-tauri/Cargo.toml` and `src-tauri/Cargo.lock`. The item names this repair as current, no longer lists PTY as missing, and retains the agent/runtime/API work as open. Historical typed-shell/current-state package wording is explicitly historical; kimi-4/kimi-5 and notebook pointers are reconciled.
- The previously worktree-only negatives and the agent-67 CHANGES result are copied to the main checkout and verified byte-for-byte. Final evidence will be uploaded and read back before worktree cleanup.

## Measurements

- Five new red-first regressions fail before implementation for their intended reasons: pre-effect unmount leak, rejected stale close without retry, cross-workspace session restoration, missing second truncation notice, and first output lost before mount.
- After the repair and compatibility corrections, `src/components/TerminalSession.test.tsx` passes **18/18** and TypeScript passes. Complete typed suite count and the remaining acceptance results are recorded below after execution; no previous count is relabeled as the new candidate result.

## Checks run at the head

- `npm test --prefix web/studio -- src/components/TerminalSession.test.tsx`: **18/18 PASS**.
- `npm run typecheck --prefix web/studio`: PASS.
- The committed lifecycle repair is `e449ae6270ccfc12939405fe66fe8e83f06cd7a8`. Complete serial default/GUI suites, all frontend/browser checks, real loopback and native acceptance, repository gates and local policy are being rerun at that source. Final report-only head/tree belongs in the PR and durable identity.

## Skills used

Adversarial review reception, systematic lifecycle root-cause investigation, red-first test-driven repair, race/readiness analysis, solution-integrity and scope checks, artifact preservation, documentation synchronization, and verification before completion.

## Still open

Complete candidate acceptance and native checks, evidence upload/readback, fresh exact-head review, required GitHub checks, protected queue/merge, and fresh-main verification. Full persistent agent runtime, parallel tasks, project memory, scheduling/Dispatch, browser/computer workflows, live R4 API integration, production publication, signed release, and useful-model capability remain unestablished.

## Plain summary

The terminal now records opened shells before an async cleanup can lose them, keeps failed closes retryable without restoring stale workspaces, and reads output only after the terminal exists. Evidence and current records are being aligned with the repaired source. Protected delivery remains pending; this is not full Cowork parity.
