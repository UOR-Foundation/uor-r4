# REPORT: infra-loop-harness, deepseek-56 (banned-presence pr-check rule)

Lab: deepseek. Order: [ORDER.md](ORDER.md). Item #2904. Base: `9a34eef06`. `References #2904`.

## What did not work, first

- **The first end-to-end run looked like a false negative.** I fetched PR #3079's current head and the new rule said PASS. That was correct: antigravity had pushed a re-gated head (`8f36239a`) after the counted CHANGES. The proof needed the earlier violating heads fetched by object id (`1ddfa1cb`, `54de128d`), where the rule then produced the expected 12 findings.
- **A one-line fixture edit broke the test file.** Replacing the base repo's `lib.rs` write accidentally concatenated the new `gated.rs` write inside the string literal; the compiler reported 79 errors. Fixed by splitting the two writes; the rule and tests are green after it.
- **A plain `git checkout -- loopprcheck.rs` during the mutation test also reverted the uncommitted rule** (the worktree's HEAD is the claim ORDER only), so the restored run still failed; the rule patch was reapplied and re-verified.

## What changed

| File | Change |
|---|---|
| `crates/uor-r4-harness/src/loopprcheck.rs` | new `banned-presence` rule and helpers: whitespace-insensitive `#[cfg(any())]` recognition, item-key parsing (struct/enum/fn/mod/trait/union/impl), brace-balanced gated-body extraction on comment/string-stripped text, head gate state (still gated / removed / moved to a test-only gate), and a banned-site scan across D23 identifiers and raw word, D25 dense-product and tensor-core tokens (assembled with `concat!`), and D32 float tokens where the D32 ratchet scans |
| `crates/uor-r4-harness/tests/loop_prcheck.rs` | base fixture with one gated item carrying a D23 site and one gated benign item; four tests: un-gating the banned item fails, moving it to `cfg(test)` fails, un-gating the benign item passes, deleting the banned gated item passes |
| `docs/harness/README.md` | the pr-check rules list now documents `banned-presence` (shared registration file) |
| this folder | ORDER and REPORT |

## Measurements

- **Real PR evidence (the planted fault):** `loop pr-check --base origin/main --head <old head> --head-ref antigravity/m3-memory-turn-discriminator-gate --local` -> `FAIL banned-presence 12 gated banned item(s) un-gated or moved to a test-only gate`, naming `struct Softmax` (merge-base line 365), `impl Softmax`, `struct SparseSoftmax`, `impl SparseSoftmax`, `struct RelationRoute`, `impl RelationRoute`, `struct SpanHead` and four more impls, each carrying a D23 site, for both `1ddfa1cb` and `54de128d`.
- **Repaired head:** the same command against `8f36239a` (where the gates were restored) -> `PASS banned-presence no gated banned item is un-gated or moved to a test-only gate`.
- **Mutation:** with the rule's `RULES` entry removed, the three new failure tests fail (`test result: FAILED. 0 passed; 3 failed`); restored, all green.
- **Suite:** `cargo test -p uor-r4-harness --test loop_prcheck` -> 33 passed; fmt clean; clippy `-D warnings` clean; ratchets d23/d25/d32 at baseline on this worktree (4451 / 93 / 31).

## Checks run at the head

`cargo test -p uor-r4-harness` (full suite), `cargo fmt --check -p uor-r4-harness`, `cargo clippy --all-targets -D warnings`, `uor-harness ratchet d23/d25/d32`, the real-head pr-check runs above, `loop report --check`, `loop pr-check --local`, `uor-harness check`; counted non-author review needed (code PR).

## Skills used

- `solution-integrity`: the first PASS on a real PR was treated as a result to explain, not to trust; the rule was proven with the actual violating heads, a mutation that removes it, and by re-running after a revert that had silently restored the wrong file.
- `scope-guard`: shared registration files only where they must be changed (the rule and its docs line), the test file in the item's scope, no rule change, no owner path.

## Still open

- The rule recognises the repo's `#[cfg(any())]` gate and moves to `cfg(test)`; other weakening forms (a feature-gated or renamed gate, macro-generated items) are follow-ups, and the D25/D32 token checks are substring checks on stripped text rather than the ratchets' exact site grammar (a deliberately conservative superset for the presence question).

## Plain summary

- Added the PM-accepted diff-level pr-check rule: it fails a PR that un-gates or moves to `cfg(test)` a `#[cfg(any())]`-gated item carrying a D23/D25/D32 site, proven against the real #3079 violating heads (12 findings) and the repaired head (pass), with four unit tests and a mutation check.