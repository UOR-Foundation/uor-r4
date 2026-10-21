# ORDER: infra-loop-pr11-progress-window, deepseek-1 (loop PR 11: gates --watch and warnings raised to errors)

Lab: deepseek. Item: [infra-loop-pr11-progress-window](../../../../roadmap/items/infra-loop-pr11-progress-window.md) (#2916). Setup row: 11. Base: `7864b05`.

## Goal

The item's done-when: `loop gates --watch` shows gate N of M, reds as they land and a finish estimate from the ledger. It opens only when a person starts a full run in a terminal, never in CI or a headless run, and closes after the run. **The named warnings are raised to errors.** The planted fault: **the window gives up after 5 minutes without change.**

The item's plan row leaves "the named warnings" for the ORDER to name, so this ORDER names the loop's own arrival and policy warnings, which the loop relies on at every session start and before a push: `loop arrive`'s checkout, binary, hooks and read-path WARNs, and `loop pr-check`'s read-path WARN. Each becomes a FAIL.

## Cancel probe (run first)

At base: `loop gates --watch --full` exits 2, "not built yet; it is loop PR 11"; `loop gates --full` is GREEN on three gates; `loop arrive` WARNs only hooks in this checkout. The six WARN sites exist in `looparrive.rs` and `loopprcheck.rs`. Class: instrument; the task stands.

## Files in scope

- `crates/uor-r4-harness/src/{loopgates,looparrive,loopprcheck,doccheck}.rs`, `lib.rs`, `main.rs` (registration only).
- `crates/uor-r4-harness/tests/**`.
- `docs/harness/gates.json`, `docs/harness/README.md`, `docs/loop/README.md` (Gates section and read-path sentence).
- `docs/harness/claims.json`, the item's `status:` line, `docs/loop/tasks/infra-loop-harness/deepseek-1/**`.

## Plan

1. Raise the named warnings to errors, with tests and docs.
2. Build the window: a status file after every gate, a renderer on stderr, open only for a full run in a terminal, close after the run, give up after 5 minutes without change.
3. Add the `loop-gates-watch` gate and its break; update the docs.

## Done when

- `--watch` shows gate N of M, the running gate, reds as they land and the ledger estimate ("no estimate yet" on a first run).
- It opens only for a full run in a terminal (off in CI, headless and non-terminal runs; `--quick`, `--status` and `--break` refuse it) and never changes the run's results or exit code.
- It closes after the run and gives up after 5 minutes without a change, saying so.
- The named warnings FAIL: arrive on checkout, binary, hooks or read path above 76,800 B; pr-check on read path above 76,800 B, grown or not.
- `loop gates --break all` is CAUGHT for `loop-gates-watch` with its own text.
- Tests, `cargo fmt -p uor-r4-harness --check`, claim-wording and links on changed files pass.

## Stop and ask if

- Any rule beyond the named warnings must change; the window needs a new dependency; a file outside scope must change; an owner-only path would change.

Size estimate: about 900 lines.
