# REPORT: infra-loop-harness, deepseek-56 (banned-presence pr-check rule)

Lab: deepseek. Order: [ORDER.md](ORDER.md). Item #2904. Base: `018954726`. `References #2904`.

## What did not work, first

- **The first design was per-item gating and an independent adversarial review broke it.** It mapped each base `#[cfg(any())]` item to its head counterpart and failed when the gate vanished. A fresh-context reviewer (deepseek subagent, `/tmp` worktrees only) found: `item_key` ignored const/static/type/macro items; renaming or re-headering the gated item bypassed it; duplicate item names made lawful deletion look like removal; a gate moved to a parent module still failed; D32 missed plain float literals and flagged `docs/labs/<x>/src` where the ratchet does not scan; substring matching made `properties` D23 via the rope family and `gemma` D25; char literals (`'{'`) unbalanced brace extraction and swallowed later code; a block comment between gate and item was skipped. Every finding was reproduced in a synthetic repo; the design was replaced rather than patched.
- **The replacement counts compiled sites instead of mapping items.** It fails when the head has more D23/D25/D32 sites in compiled code than the base for a changed file. That catches rename/re-header/duplicate bypasses by construction, passes lawful deletion and re-gating, and needs no item identity at all.
- **The rule then failed its own repo's D23 ratchet.** My first regression test constants were named `GATED_HEAD_PROPERTIES`; the canonical ratchet's rope detector treats the uppercase segment `ROPE` inside `PROPERTIES` as the rope family, adding 2 sites and failing `ratchet d23`. Renaming the fixture identifiers fixed it; the rule itself tests clean at baseline. This is the ratchet's known English-word edge, not a defect in the rule, but it is recorded because it is exactly how a real PR can accidentally raise the ratchet.
- **A fixture edit broke the whole test file** (a write call was concatenated inside a string literal); fixed and superseded by the rewritten fixture.

## What changed

| File | Change |
|---|---|---|
| `crates/uor-r4-harness/src/loopprcheck.rs` | new `banned-presence` rule: `gated_spans` (gate through item end, parent modules included, comments/strings/char literals stripped), `ungated_code` (blank gated spans; `cfg(test)` not exempt), exact D23 segment matching with the ratchet's rope exception, D25 dense-product/tensor-core segments, D32 float types and literals in the ratchet's scan scope, and a per-kind base-vs-head count comparison |
| `crates/uor-r4-harness/tests/loop_prcheck.rs` | second item (`item-b`, `crates/uor-r4-discrete/**`) and eight tests: un-gated banned item fails; `cfg(test)` move fails; rename fails; benign/properties/char-literal un-gates pass; deletion passes; parent-module gate passes; D32 float function un-gated fails; kept gated passes |
| `docs/harness/README.md` | the pr-check rules list documents the compiled-site delta, parent gates, `cfg(test)` not exempt, and comments/strings counted out |
| this folder | ORDER and REPORT |

## Measurements

- **Planted fault against real heads:** `loop pr-check --base origin/main --head <old head> --head-ref t --local` on PR #3079's violating head `54de128d` -> `FAIL banned-presence 1 changed file(s) compile more banned code than their base`, detail `relation_compiler.rs: D23 sites in compiled code rise from 0 at the merge base to 19 at the head`; the same command on the re-gated head `8f36239a` -> PASS. The same run at `1ddfa1cb` also fails.
- **Tests:** `cargo test -p uor-r4-harness --test loop_prcheck` -> 37 passed (29 before this unit; 8 new). `cargo fmt --check` and `cargo clippy --all-targets -D warnings` clean; ratchets d23/d25/d32 at baseline (4451 / 93 / 31).

## Checks run at the head

`cargo test -p uor-r4-harness` (full suite), fmt, clippy `-D warnings`, `uor-harness ratchet d23/d25/d32`, the real-head pr-check runs above, `loop report --check`, `loop pr-check --local`, `uor-harness check`; counted non-author review needed (code PR).

## Skills used

- `solution-integrity`: the independent review was treated as evidence and the design was replaced after reproduction rather than patched; the delta formulation removes the whole class of identity bypasses; every reviewer case became a regression test.
- `scope-guard`: shared registration files only where the rule and its docs must live, the test file inside the item's scope, no rule change, no owner path.

## Still open

- The rule works on source text, so macro-generated banned code and gates hidden behind macro expansion are outside it; D32 literal detection covers decimal/exponent literals and type tokens, not every literal form; the counts are conservative for a presence question, not an exact replacement for the ratchets.

## Plain summary

- Added the PM-accepted diff-level pr-check rule: it counts D23/D25/D32 sites in compiled code per changed file, base vs head, and fails when a gate removal raises them; proven on the real violating #3079 heads (D23 0 -> 19) and the repaired head (pass), with eight regression tests from an independent adversarial review.