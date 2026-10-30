# REPORT: comprehensive 30 Oct documentation refresh, kimi-4

Lab: kimi. Order: [ORDER.md](ORDER.md). References #3380 and #2029.

This is a documentation and evidence-propagation unit. It ran no model training, GPU evaluation, paid compute, or production deployment.

## What did not work

- The first independent exact-head review at `ed1d376d0` returned **CHANGES**, not an approval. It found stale pre-M1/D67 statements in `what-is`, FAQ and glossary; an M1 item whose close-grade and upstream gate letters were conflated; old M1/M7/M8 tracker fields; an admission-controller completion claim contradicted by `NOT_RUN`; and remaining owner-release wording. The broad prose scan had missed those reader-level contradictions, so I did not queue that head.
- The second fresh review at `710f6b441` also returned **CHANGES**. It found additional stale embedded figures and book navigation, mechanism and research-line status, tracker composition/history, a second controller completion sentence, FBS export conflation, and old WebGPU/CDN Studio copy. I expanded the repair to every one of those surfaces and rebuilt the figure inventory instead of treating the first repair as sufficient.
- The first serving-figure regeneration stopped on its text-fit gate: one caption needed about 1,396 px in a 1,120 px field. I split the M1 and FBS measurements into three accurate lines rather than weakening the check. Regeneration then passed.
- The first Studio worker check stopped before testing because `wasm-pack` found the machine's `cargo-ai` shim, which has no `cargo metadata`. Putting real Rust Cargo 1.97.1 first on `PATH` fixed the environment; no project code or assertion changed.
- The first strengthened Studio status check passed before the whole tier description had been reconciled. A final diff review found stale CDN/byte-mod text. I added an assertion, watched it fail, corrected the tier, and watched all three Studio checks pass.
- Claude's #3430 merged during verification. I merged it forward and updated the living pages from “claude-218 active, no result” to its actual offline sign-key result and the remaining served-key/free-generation gap. The figure-hardening work from #3426, adopted through #3438, was also merged forward before regeneration.
- Repository-wide `cargo fmt --check` remains red on pre-existing formatting in files outside this unit, including `crates/repo-conformance/src/meta.rs`, core tokenizer tests, and float-train sources. The scoped `cargo fmt --check -p xtask` passes. I did not reformat unrelated code.
- `uor-harness check` retains one pre-existing error: the untouched `docs/harness/prep/m1-close-saved-model-2026-10-26.md` missing its `Stop rule`. It also emits legacy warnings, including the dated session-goal record not yet stating M1's 6/6, unfrozen M5/M7/M8 panels, stale claims/presence, and old prep/ledger records. No panel or state-format error remains from this unit. I did not rewrite those sealed or out-of-scope records. The separate full loop gates are green.
- The mdBook preprocessor warns that it was built against mdBook 0.4.36 while mdBook 0.4.40 is installed. The build and link check pass. The WASM build retains an existing dead-code warning and package-metadata/update warnings; its tests pass.

## What changed

- Rebuilt the root README and current milestone record around the reviewed M1 close: pre-alpha, 1 of 8 complete, the saved discrete model's six bars at two seeds, and the explicit 62.5% p-f reference-reply limitation. K16 and older failures remain dated history.
- Reconciled the linked living book and crate READMEs. The discrete M1 model and separate float-trained FBS trunk are never presented as one qualified conversational model. M2/M3 are in progress; M4–M8 remain open.
- Propagated claude-217's opening/content limits and the newly merged claude-218 result: on p-u's 1,218 exact-memory misses, sign +46/+37/+33 and point +125/+185/+114 at three seeds. The next unit trains and serves the cheapest passing sign key and tests free generation on another fresh panel. No served-key result is claimed.
- Updated `AGENTS.md`, `CHARTER.md`, `docs/integration/DECISIONS.md`, and `docs/rules/changes.md` only under the owner-approved status scope. D83 records the close without changing a bar or technical rule.
- After the D76 advisory, added `docs/harness/claims.json` to the item scope solely for this branch's existing required Kimi claim. The top-session scope decision is recorded in ORDER; no other claim or lab state changed.
- Updated the M1 item inputs, current milestone cache, and historical fields. The older close grade is described as bars (a)–(e); claude-211 supplies bar (f).
- Repaired the first review's findings: corrected `what-is`, FAQ, glossary and current D67 rules; added a dated close-grade/upstream-gate mapping; moved old M1 fields into history with all three qualification panels current and no invented singular panel key; marked M7 runtime work in progress and removed M8's blanket hold; restored the controller's `NOT_RUN` scope; and regenerated the milestone figure with M1 `COMPLETE` and M7 in progress. The harness caught and I corrected an attempted prose-list panel key before delivery.
- Repaired the second review's findings: updated the embedded hero, product, project, mechanism and FBS export figures plus `docs/SUMMARY.md`; labelled the 27-entry lineage figure as 19 October history; reconciled the mechanism board and research-line pages; recorded the exact M1 composition and prior M2/M8 holds in the tracker; corrected the remaining controller sentence; named the probe teacher-forced throughout; updated the Studio model manager with a red/green assertion; and rebuilt the inventory as 68 SVGs = 17 generated + 51 hand-maintained.
- Regenerated the architecture, milestone, training, serving, geometry, and chart figures; corrected hand-maintained roadmap, pipeline, and LUT figures, captions, alt text, titles, and descriptions. M1 and FBS throughput stay distinct, and D80/D81 integer multiply/divide wording is current.
- Corrected all three Studio status areas to 1/8, native M1 decode, stack-bound ByteLevel BPE, self-hosted assets, and the synthetic stand-in with no bundled qualified model. Strengthened the existing checks against stale status.
- Preserved `archive/**`, `docs/labs/**`, dated notebook rows, `docs/integration/current-state.md`, Claude's research files, and antigravity-113's two mechanism pages. `STATE.md`, `HANDOFF.md`, and `docs/roadmap/README.md` remain for the designated render workflow, not hand edits.
- Diff size: 82 files, 2,285 insertions and 1,637 deletions, including this REPORT and the complete non-archive README inventory.

## Measurements

| Result | Current documented evidence | Limit |
|---|---|---|
| DEV integer top-1 | 7,002 / 7,001 of 14,287 at two seeds; unigram 1,226, T2 6,300 | Saved discrete M1 model, not FBS |
| Fresh V2 | 7,215 / 7,228 of 14,479; unigram 1,279, T2 6,582, band 82 | Transfer within the registered panel protocol |
| Held-out phrasing p-f | 7,850 / 7,872 of 15,953; unigram 1,409, T2 7,375, band 81 | 62.5% of reference replies are verbatim TRAIN replies; not a generated-copy rate |
| Export and serving | Exact 14,287-row agreement and B4; Apple M1 decode 10,876 / 10,566 tok/s with 192-token prefill and 64-token decode | Not browser or useful-chat qualification |
| FBS opening failure | claude-217 p-s openings 17.4%, 13.7%, 18.6% against about 80% on later tokens | Greedy remains the decode; loop suppression is not content quality |
| Offline memory-key probe | claude-218 sign +46/+37/+33 on 1,218 missed p-u rows; point +125/+185/+114 | Teacher-forced/offline evidence; trained served key and free generation remain open |

## Checks run at the head

- `cargo test --locked -p xtask`: **220 passed, 0 failed**.
- `cargo fmt --check -p xtask`: pass. The repository-wide formatting exceptions are listed above.
- `cargo run -q --locked -p xtask -- figures`, `figures-math`, and `figures-charts`: **8, 5, and 4 figures current** respectively.
- `cargo run -q --locked -p uor-r4-claim-wording -- .`: pass, within that tool's supported Markdown surface; changed public prose was also scanned for stale status.
- `cargo run -q --locked -p xtask -- roadmap check`: **114 items OK**. `docs-catalog --check`: **1,175 records, 0 paths not found**.
- `cargo run -q --locked -p xtask -- archive-verify --base ec9a790fe5be73cd673f082eba86ced551b22d12`: **10,810 archive files, 0 modified, 0 deleted, 0 regressions**.
- `cargo run -q --locked -p xtask -- links --baseline … --renames-from …`: **6,001 sources, 23,050 checked, 1,202 baseline broken, 0 new broken**.
- `book-stage`, `mdbook-mermaid install .`, `mdbook build`, and `book-check`: **104 pages and 70 figures staged; 105 pages checked, 0 broken links**.
- `node web/studio/verify_studio_worker.mjs`: pass. Real WASM and the 469,252-byte synthetic stack were built; exact 32-token output, 9,782/5,366-byte checkpoints, bit-identical save/restore, mid-generation guards, six network traps, and zero remote requests passed.
- `node web/studio/verify_studio_page.mjs --strict-router` and `verify_studio_page_load.mjs --strict-router`: both pass; both pages loaded with no external requests, and the planted external trap was detected.
- Scorer evidence-integrity tests in `docs/labs/m1-d11-artifact-scorer/instrument`: **26 passed, 0 failed, 1 ignored** by the test's own missing-DEV-data guard. This was not a new model evaluation.
- Visual inspection covered every changed figure at wide and 390 px layouts, including the dedicated narrow architecture and repaired milestone figure. No clipping or M1/FBS measurement conflation was found; dense wide diagrams retain updated text descriptions.
- Reader-test after repair: the front page and start pages identify M1 on the saved 19-table discrete model, say no useful chat, name the next served-key/free-generation and M2–M8 gaps, and link the M1/claude-218 evidence. A separate stale-prose scan found only explicitly historical or out-of-scope matches and the named D41 exception.
- `uor-harness loop gates --full`: all 11 gates green.
- `uor-harness check`: completed with the pre-existing error/warnings detailed above.
- `uor-harness loop report --check`: **PASS**. `uor-harness loop pr-check --local`: **verdict PASS**. The D76 scope advisory remains by design because `rule_scope` reads the item at the merge base; the new claim-file scope entry takes effect after merge.

## Skills used

`using-superpowers`, `uor-research-architect`, `executing-plans`, `using-git-worktrees`, `test-driven-development`, `verification-before-completion`, `systematic-debugging`, `requesting-code-review`, `receiving-code-review`, `solution-integrity`, and `modern-web-guidance`. The accessibility guide informed the complex-figure descriptions; no icon guidance was applied.

## Still open

- Useful conversation is not established. The next research step is a trained, served sign key and correct free generation on another fresh panel; the larger point-key gain remains an explicit design tradeoff.
- M2 and M3 are in progress; M4–M8 are not complete. The Studio and desktop shell still use a synthetic stand-in without a bundled qualified product model.
- The independent exact-head review follows this commit. Roadmap sync, generated STATE/HANDOFF/roadmap views, issue verification, and the automatic docs-site preview follow the content merge through their designated workflows.
- The pre-existing formatting, harness, preprocessor, and WASM warnings above remain outside this documentation unit's repair scope.

## Plain summary

M1 really is complete: one saved discrete geometric model passed all six bars at two seeds and serves as integer artifacts on an Apple M1. It still does not hold a useful conversation, and p-f's 62.5% reference-reply overlap remains next to that claim.

The documentation now says which model passed, what the numbers mean, and what happens next. The separate FBS line has an encouraging offline memory-key result, but it still must train and serve that key and prove free generation. No product release or later milestone is implied.
