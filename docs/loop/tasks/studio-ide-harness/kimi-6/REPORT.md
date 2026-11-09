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
- After the repair and compatibility corrections, `src/components/TerminalSession.test.tsx` passes **18/18** and TypeScript passes. The complete typed suite is **450/450 across 52 files**, plain-JS **113/113**, Playwright **45/45**, launcher **6/6**, worker/page/static/runtime checks and **7/7** WASM, all measured after the lifecycle repair.
- Complete serial default **11/3/21/4/7/13/8** and GUI **13/3/25/4/8/13/9** pass. Lifecycle implementation is `e449ae6270ccfc12939405fe66fe8e83f06cd7a8`; `c67eaa2a5fdd93fc0fa3128f2fc512e685555680`, tree `a43fbb11925acb56c86193c0d562a220a138b71c`, adds the scope/evidence corrections and is the acceptance source. Final report-only head/tree belongs in the PR and durable identity.
- Interactive loopback at c67 returned `ANSWER:fixture-answer`, exact fixture cwd, `24 133`, exit and closed-input refusal, with 15 initial requests and zero external/raw-TypeScript/runtime errors. Host SHA `af58c744e94c9375749fd56a0adf86ab3f61ae8f53bbd35c2ef2f0f8a0fb3f37`, dist `ba21724f166d6720fdbca7edac3772b31247e6178cc67eea5d22f3aecba2a349`.
- Combined loopback restored exact transcript/session identity and native edit/save, with 15 initial/12 reopened requests and zero external/raw-TypeScript/runtime errors. Host SHA `19ed991e93b41506bb5f9e24658726398f660137e18430b97dac6bd6c4dfc1d9`, dist `4da4dea2e67461b4295291f16472e8ae0d9975004309461a31273e26bfca7ee0`.
- Packaged native executable SHA-256 `3b2612c06770f42ae6435e6cef392e3adb94a08bece22dff172c24cf8680c9d2`. Native first output included `NATIVE_FINAL`; `read` returned `ANSWER:fixture-answer`, exact fixture cwd and `24 80`, then exit 0. A distinct second-shell approval was cancelled with no new PTY. App and sidecar were absent after normal quit.
- Previously missing `58d14a29…` negatives, `interactive-terminal-clean.log` and the agent-67 CHANGES review are preserved in the main checkout and project store. File-based storage cannot retain empty fixture directories; a complete archive was uploaded, downloaded, extracted and compared byte-for-byte. Individual logs also match.

## Checks run at the head

- `CARGO_TARGET_DIR="$PWD/target/cowork-tests" cargo test --manifest-path src-tauri/Cargo.toml` and the same command with `--features gui`: complete default **11/3/21/4/7/13/8**, GUI **13/3/25/4/8/13/9**, PASS serially.
- `npm test --prefix web/studio`: **450/450 across 52 files PASS**, including terminal **18/18**. TypeScript and Vite build, **113/113** plain-JS, **45/45** Playwright, **6/6** launcher, worker/page/static/runtime, desktop and **7/7** WASM checks PASS.
- `node web/studio/verify_interactive_terminal.mjs` and `node web/studio/verify_studio_loopback.mjs`: PASS at c67 with exact identities above.
- `UOR_COWORK_NO_OPEN=1 CARGO_TARGET_DIR="$PWD/local/kimi/cowork-parity-target" bash scripts/studio/open-cowork.sh`: PASS, 21.37 MiB unsigned package, followed by the actual native workflow above.
- Evidence is `local/kimi/harness-parity/c67eaa2a5fdd93fc0fa3128f2fc512e685555680/`, including `native-acceptance.md` and three native screenshots; combined identity is in the main checkout's `local/kimi/studio-final-acceptance/c67eaa2a5fdd93fc0fa3128f2fc512e685555680/identity.json`. Final gates and policy run at the report-only head. Earlier `6fa2baf377…`, `da6cbaf4…`, `3e201b987…` and `58d14a29…` evidence is intermediate.

## Skills used

Adversarial review reception, systematic lifecycle root-cause investigation, red-first test-driven repair, race/readiness analysis, solution-integrity and scope checks, artifact preservation, documentation synchronization, and verification before completion.

## Still open

Final report-only head gates/policy, fresh exact-head review, required GitHub checks, protected queue/merge, and fresh-main verification. Full persistent agent runtime, parallel tasks, project memory, scheduling/Dispatch, browser/computer workflows, live R4 API integration, production publication, signed release, and useful-model capability remain unestablished.

## Plain summary

The terminal records opened shells before an async cleanup can lose them, keeps failed closes retryable without restoring stale workspaces, and reads output only after the terminal exists. The repaired lifecycle has complete browser, loopback and packaged native evidence, with prior negatives preserved. Protected delivery remains pending; this is not full Cowork parity.
