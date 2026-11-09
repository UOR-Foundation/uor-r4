# REPORT: close workspace-origin and unmount PTY cleanup failures, kimi-7

Lab: kimi. Item: [studio-ide-harness](../../../../roadmap/items/studio-ide-harness.md) (#3569). Order: [ORDER.md](ORDER.md). Base: `ed8fdbcaecdde3938b3fb445c90b87ec6740e723`, tree `f1c98b9c866398bde7bd32b926342b1eadadc2cb`; merged main `f75725d3071abb9488e058ac158a04d8d093014d`. Fresh reviewer agent-68 returned CHANGES at that head. PR #3890 remains draft and unqueued.

## What did not work

- A workspace change increments the revision before starting the close. Capturing that new revision on rejection makes the old workspace appear current and restores its session; a stale retry can restore it again. The earlier test started close before the workspace change and did not cover this ordering.
- A rejected close after unmount had no retry UI and only one close attempt, so a backend-reinserted session could remain without cleanup. The existing unmount regression used only successful closes.
- Pending read/input/resize rejections from an old session could overwrite a new session's status. The shared reading flag also suppressed the new session's first read until the old request settled.
- Active current-state, typed-shell and desktop-client records still pointed to intermediate da6/ec49 evidence, and the integer result was written as 499/499 despite one ignored test. The final-head evidence directory did not include the actual red logs.

## What changed

- The runtime now stores the shell's origin workspace revision. Normal close failure restores visible identity only when that origin is still current; workspace-change and stale-retry failures expose cleanup retry without mounting. It no longer decides restoration from the later close-start revision.
- Unmount makes one bounded automatic `closeTerminal` retry. A second failure is logged explicitly and does not create an unlimited retry loop or detached-process-containment guarantee.
- Read ownership is per session. Close clears it so a new shell can read immediately, while an old request's finally cannot clear the new shell's ownership. Read, input, resize and mount error handlers also check session identity before displaying an error.
- Current records name this repair and 21 focused/453 typed tests. Earlier native evidence remains intermediate rather than being relabeled as evidence for this implementation. Integer results now say 499 passed, 1 ignored, and actual red logs are preserved beside the green evidence.

## Measurements

- `never restores an old workspace when close and retry both reject`: **RED old `session-a` restored → GREEN no active session, Retry close only, third close succeeds**.
- `retries a rejected close once after unmount`: **RED one close call → GREEN two close calls for the same session**.
- `drops stale read input and resize completions after a new shell opens`: **RED second session's first read not called → GREEN second read called and no stale error displayed**.
- All terminal tests **21/21** and complete typed suite **453/453 across 52 files** pass, with TypeScript PASS. Full merged acceptance is being rerun at implementation head `dde958250ce8c98a28ac99a7698538eda0d3d872`; final acceptance and record identities belong in the PR and evidence, not a guessed self-hash.

## Checks run at the head

- `npm test --prefix web/studio -- src/components/TerminalSession.test.tsx`: **21/21 PASS**.
- `npm test --prefix web/studio`: **453/453 across 52 files PASS**.
- `npm run typecheck --prefix web/studio`: PASS.
- Full default/GUI, browser, loopback and native checks are running. Final repository gates and fresh exact-head review are still required.

## Skills used

Adversarial review reception, systematic root-cause investigation, red-first test-driven repair, origin-revision/state-ownership analysis, artifact preservation, and verification before completion.

## Still open

Complete merged acceptance, packaged native workflow, current-pointer synchronization, evidence upload/readback, final gates, fresh exact-head review, protected queue and fresh-main verification. Full persistent runtime, parallel tasks, memory, Dispatch, browser/computer workflows, live R4 API, production publication, signed release and useful-model capability remain unestablished.

## Plain summary

Close recovery now depends on the shell's origin workspace rather than the later close time. Unmount gets one bounded cleanup retry, and stale async completions cannot affect a new session. The focused tests are green; final acceptance and protected delivery are pending.
