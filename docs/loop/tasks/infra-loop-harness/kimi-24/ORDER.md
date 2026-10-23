# ORDER: infra-loop-harness, kimi-24 (card #3041: prep::test_fitness must read the LAST verdict)

Lab: kimi. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904). Card: #3041.
Base: `15d5728c7`.

## Goal

Card #3041: `prep::test_fitness` (`crates/uor-r4-harness/src/prep.rs:97`) returns on the FIRST anchored `TEST FITNESS:` verdict line. The normal re-review flow appends a new verdict rather than editing history, so a sheet that earned NOT_FIT and later FIT still parses NOT_FIT — a re-reviewed sheet stays silently blocked at the FIT gate (D22 clears compute per sheet). This is now a live path: #3052 just recorded a NOT_FIT on the m4 flock-coupling sheet, and its later FIT will be appended. Make the last anchored verdict win, with a regression test.

## Cancel probe (run first)

Class **instrument** (harness unit; no GPU). At the base, `prep.rs`'s `test_fitness` loops over lines and `return`s the first anchored verdict (source, lines 97–117). A sheet containing `TEST FITNESS: NOT_FIT` followed by an appended `TEST FITNESS: FIT` therefore reads NOT_FIT. Task stands.

## Files in scope

- `crates/uor-r4-harness/src/prep.rs` (the function and its inline `mod tests`; in the umbrella scope since kimi-23)
- this task folder

## Done when

- `test_fitness` returns the LAST anchored verdict in the text (ABSENT if none), with the anchoring and NOT_FIT-before-FIT prefix rules unchanged.
- A regression test pins it: NOT_FIT-then-appended-FIT reads FIT; FIT-then-appended-NOT_FIT reads NOT_FIT; prose mentions still do not count.
- `cargo test -p uor-r4-harness` passes; `loop report --check` and `loop pr-check --local` pass; a fresh-context review from another vendor is requested (D59 ended 22 Oct — any other-vendor review counts).
- The PR body says `Closes #3041` and `References #2904`.

## Stop and ask if

- A caller depends on first-wins behavior (checked: `preprules.rs`, `main.rs`, `lib.rs` call sites) in a way that makes last-wins wrong — then the fix needs a different shape and I stop and ask.
- Any file beyond prep.rs and this folder needs to change.
