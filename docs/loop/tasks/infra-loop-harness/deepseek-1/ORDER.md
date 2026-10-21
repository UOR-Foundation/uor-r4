# ORDER: infra-loop-pr11-progress-window, deepseek-1 (loop PR 11: gates --watch and warnings raised to errors)

Lab: deepseek. Item: [infra-loop-pr11-progress-window](../../../../roadmap/items/infra-loop-pr11-progress-window.md) (#2916). Setup row: 11. Base: `7864b05`.

## Goal

Loop PR 11 from the build plan. The item's done-when:

> `loop gates --watch` shows gate N of M, reds as they land and a finish estimate from the ledger. It opens only when a person starts a full run in a terminal, never in CI or a headless run, and closes after the run. The named warnings are raised to errors. The planted fault turns the check red: **the window gives up after 5 minutes without change.**

The item's plan row leaves "the named warnings" for the ORDER to name, so this ORDER names them as the loop's own arrival and policy warnings, which the loop relies on at every session start and before every push:

- `loop arrive`: `checkout` behind `origin/main`, `binary` older than the checkout's newest build, `hooks` not the text the binary installs, and `read-path` above 76,800 B. Each WARN becomes a FAIL, and the command exits 1.
- `loop pr-check`: the `read-path` warning above 76,800 B, and the over-budget-but-not-grown warning. Each becomes a FAIL in that rule's verdict and in the command's exit code.

## Cancel probe (run first)

Run at base `7864b0516`, in this worktree, with the current binary:

- `UOR_LAB=deepseek uor-harness loop gates --watch --full` exits 2 with "loop gates --watch is not built yet; it is loop PR 11". The feature is absent; the task stands.
- `UOR_LAB=deepseek uor-harness loop gates --full` is GREEN on all three gates (fmt-claim-wording 0.4 s, fmt-harness 0.7 s, claim-wording 4.3 s), so the runner the window attaches to works at the base.
- `UOR_LAB=deepseek uor-harness loop arrive` reports WARN for `hooks` (this shared checkout's pre-commit hook differs from the binary's) and PASS for the rest at the base.
- The four `loop arrive` WARN sites and the two `loop pr-check` read-path WARN sites exist at the base; verified by reading `looparrive.rs` and `loopprcheck.rs`.

Class: instrument (the feature and the stricter severities are absent, not broken).

## Plan

1. Raise the named warnings to errors, with tests: `looparrive.rs`, `loopprcheck.rs`, their tests and docs.
2. Build the watch: the runner writes a status file after every gate, a watcher renders it, and it opens only for a full run in a terminal, never in CI or headless. Closes after the run, gives up after 5 minutes without change.
3. Add the `loop-gates-watch` gate and its break to `docs/harness/gates.json`, then the docs.

## Files in scope

- `crates/uor-r4-harness/src/loopgates.rs`
- `crates/uor-r4-harness/src/looparrive.rs`
- `crates/uor-r4-harness/src/loopprcheck.rs`
- `crates/uor-r4-harness/src/lib.rs`, `crates/uor-r4-harness/src/main.rs` (registration only, kept small)
- `crates/uor-r4-harness/tests/**`
- `docs/harness/gates.json`
- `docs/harness/README.md`
- `docs/loop/README.md` (the Gates section and the read-path sentence only)
- `docs/harness/claims.json` (this branch's claim line)
- `docs/loop/tasks/infra-loop-harness/deepseek-1/**`
- `docs/roadmap/items/infra-loop-pr11-progress-window.md` (its `status:` line only)

Size estimate: about 900 lines (code and tests), plus docs.

## Done when

- `UOR_LAB=<lab> uor-harness loop gates --watch` on a full run in a terminal shows `gate N of M`, each red result as it lands, and a finish estimate computed from the latest seconds per gate in `local/<lab>/gates.jsonl` ("no estimate yet" on a first run).
- The watch opens only for a full run in a terminal: off in CI (`CI` set), in a headless or non-terminal run, and for `--quick`, `--status` and `--break`; the run's own results and exit code are unchanged when the watch is off or when it gives up.
- The watch closes after the run, and gives up after 5 minutes without a change to its status file, saying so.
- The named warnings are errors: `loop arrive` exits 1 on a behind checkout, an older binary, a missing or different pre-commit hook, or a read path above 76,800 B; `loop pr-check` fails `read-path` above 76,800 B, including when it did not grow.
- `docs/harness/gates.json` gains the `loop-gates-watch` gate, and `loop gates --break all` is CAUGHT for it with its own text.
- The harness's tests pass, `cargo fmt -p uor-r4-harness --check` is clean, claim-wording passes, and links on changed files pass.

## Stop and ask if

- A change to any rule outside this ORDER's named warnings would be needed.
- The watch would need a new dependency (none is planned).
- A file outside scope must change.
- Any owner-only path (the charter, the loop page's rules text beyond the Gates section, registries, caps) would change.
