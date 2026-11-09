# REPORT: close PTY lifecycle and evidence findings, kimi-5

Lab: kimi. Item: [studio-ide-harness](../../../../roadmap/items/studio-ide-harness.md) (#3569). Order: [ORDER.md](ORDER.md). Base: `ec49e32b4d5af2f12cd6b84328584336c6b182ba`, tree `fa2e0af9a80c5994616423ce7b02400c6331ca27`; merge base `ad2f12edd8e3c2396b2fe678dbf16d02d128bb9f`. Fresh reviewer agent-66 returned CHANGES; PR #3890 remains draft and unqueued.

## What did not work

- A late `openTerminal` result could mount after unmount and was not closed. A rejected close suppressed its error and discarded the session id. The initial unmount regression observed no close call; the close-failure regression had no visible `reap failed` message and lost the session.
- Closing also needed to block input immediately and prevent a second open while the first close was pending. The existing input/workspace tests exposed those races during repair. A disposed restored session must not remount or restart polling, but must retain retryable close state and the visible error.
- An empty initial cleanup could mark the panel disposed while its first enable was pending, so the ordinary real-browser open stopped before `openTerminal`. The mounted trace showed `disposed=true` after enable. Empty cleanup now marks disposal only after actual unmount; live-shell close still blocks input immediately. A disposed restored session with the same id cannot remount, but a genuinely new session id can mount.
- The approval-consumption RED run is observed in the session, but was not retained as a separate raw log at that time. A session-observation excerpt is now preserved; it is not described as a complete original raw log.
- `kimi-4` retained earlier source/count roles and a stale final-evidence pointer. The notebook said merged before protected merge. Product records had earlier typed counts/package scope, and the parity matrix called 975eb the final native head. A second local body draft was historical but not labeled. Those roles are corrected rather than treated as new product claims.

## What changed

- Unmount/workspace disposal blocks new input and closes any subsequently returned stale shell. A close in progress also prevents another open in both the guard and button state.
- Close captures the previous session and id, disposes the renderer and pollers immediately, and only clears identity after success. Failure restores the session identity and visible error without restarting the terminal; a later retry can close it. An actual unmount does not restore UI state.
- The two regressions pass alongside all prior terminal cases. The unmount assertion checks the close mock's actual argument object. Readiness waits make the existing workspace-change tests exercise the next shell after close completion rather than race it.
- Report/source evidence now distinguishes ec49 and its final checks from the new repair candidate. The final candidate's HEAD/tree belongs in the PR and durable identity, not an invented self-hash in this report. The notebook says pending protected merge, README uses the actual 445 typed count, desktop/package scope includes the measured native acceptance, and the parity matrix binds ec49. `pr-body.md` is explicitly historical; `pr-cowork-body.md` and the live PR are authoritative.
- Existing earlier screenshots/JSON, including `canonical-launch.jpg` (SHA-256 `25e570b3ca2b44c886135527cf98e95cc389502e1e731789ccbcdc929d4ef009`), are copied to the main checkout without overwriting. No unsupported missing-file claim is introduced.

## Measurements

- `closes a shell whose opening completes after the panel unmounts`: **RED no close call → GREEN one close call** for the returned shell.
- `retains the session and visible error when close fails, then permits retry`: **RED missing error/dropped session → GREEN retained error/session and second close succeeds**.
- All terminal component tests **13/13**; all typed tests **445/445 across 52 files**; TypeScript and Vite build PASS.
- Earlier ec49 exact-head evidence remains: default **11/3/21/4/7/13/8**, GUI **13/3/25/4/8/13/9**, plain-JS **113/113**, Playwright **45/45**, launcher **6/6**, worker/page/static/runtime checks, **7/7** WASM, **11/11** repository gates. Actual PTY, loopback and packaged native approval/cancellation are recorded there.
- This repair changes `TerminalSession.tsx`, its tests, and evidence records only. No model/API/engine/worker/tokenizer, new dependency, or public service is added.

## Checks run at the head

- `npm test --prefix web/studio -- src/components/TerminalSession.test.tsx`: **13/13 PASS**.
- `npm test --prefix web/studio`: **445/445 across 52 files PASS**.
- `npm run typecheck --prefix web/studio` and `npm run build --prefix web/studio`: PASS.
- After this record commit, repeat browser/loopback/native acceptance, default/GUI suites, local policy/full gates, and fresh exact-head review at the actual new candidate. Do not substitute ec49's evidence for that new head.

## Skills used

Systematic lifecycle root-cause investigation, red-first unmount/close-failure tests, adversarial review reception, race/readiness analysis, artifact preservation, documentation synchronization, and verification before completion.

## Still open

Fresh candidate acceptance and native checks, fresh exact-head review, required GitHub checks, protected queue/merge, and fresh-main verification. Full persistent agent runtime, parallel tasks, project memory, scheduling/Dispatch, browser/computer workflows, live R4 API integration, production publication, signed release, and useful-model capability remain unestablished.

## Plain summary

The terminal now closes late opens, blocks input while closing, and exposes failed-close state with a retry instead of silently losing the session. The evidence now describes the real candidate and pending protected merge, with unique artifacts preserved outside the worktree.
