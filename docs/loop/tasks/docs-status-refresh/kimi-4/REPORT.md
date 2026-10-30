# REPORT: comprehensive 30 Oct documentation refresh, kimi-4

Lab: kimi. Order: [ORDER.md](ORDER.md). References #3380 and #2029.

This is a documentation and evidence-propagation unit. It ran no model training, GPU evaluation, paid compute, or production deployment.

## What did not work

- The first independent exact-head review at `ed1d376d0` returned **CHANGES**, not an approval. It found stale pre-M1/D67 statements in `what-is`, FAQ and glossary; an M1 item whose close-grade and upstream gate letters were conflated; old M1/M7/M8 tracker fields; an admission-controller completion claim contradicted by `NOT_RUN`; and remaining owner-release wording. The broad prose scan had missed those reader-level contradictions, so I did not queue that head.
- The second fresh review at `710f6b441` also returned **CHANGES**. It found additional stale embedded figures and book navigation, mechanism and research-line status, tracker composition/history, a second controller completion sentence, FBS export conflation, and old WebGPU/CDN Studio copy. I expanded the repair to every one of those surfaces and rebuilt the figure inventory instead of treating the first repair as sufficient.
- The third fresh review at `178c58b8a` returned **CHANGES** as well. It found stale glossary and research-line entries, mechanism rule checks, close-item upstream evidence, project-track preservation, remaining owner-release wording, and figure status/D80 labels. I corrected those and restored the upstream G-Adm/G-Occ and project-record pointers before another broad scan.
- The fourth fresh review at `512ac2a2e` returned **CHANGES**. It found A8/Stage T evidence misattribution, unsupported M1 FBS-projection/normalization claims, a public `0/8` side-project page, an invalid M2 count, more geometry/status contradictions, and unrecorded baseline negatives. I checked the sealed A8/Stage T/M1 records and corrected both the evidence scope and the affected public pages.
- Retained baseline negatives: the earlier refresh named two broken book links from `docs/research/notebook/m4-serving.md`; current book-check reports zero after merged tooling, so no current broken book link is attributed here. Scoped formatting now passes. `cargo clippy -p xtask -- -D warnings` still fails on the two pre-existing findings: `xtask/src/book_stage.rs:342` (`unnecessary_map_or`) and `xtask/src/docs_relink.rs:250` (`manual_repeat_n`). I did not reformat or suppress them outside scope.
- The first serving-figure regeneration stopped on its text-fit gate: one caption needed about 1,396 px in a 1,120 px field. I split the M1 and FBS measurements into three accurate lines rather than weakening the check. Regeneration then passed.
- The first Studio worker check stopped before testing because `wasm-pack` found the machine's `cargo-ai` shim, which has no `cargo metadata`. Putting real Rust Cargo 1.97.1 first on `PATH` fixed the environment; no project code or assertion changed.
- The first strengthened Studio status check passed before the whole tier description had been reconciled. A final diff review found stale CDN/byte-mod text. I added an assertion, watched it fail, corrected the tier, and watched all three Studio checks pass.
- Claude's #3430 merged during verification. I merged it forward and updated the living pages from “claude-218 active, no result” to its actual offline key result. Claude's #3439 then supplied the free-generation test: the point key passes partially at three seeds while the sign key is inconclusive at p-v's size. I merged that result and #3441 forward and replaced the now-superseded sign-key next step throughout the living pages. DeepSeek's #3442 rebuilt the Studio and #3444 recorded its verified live publish; I preserved both rather than restoring the retired page or calling the publish pending. The figure-hardening work from #3426, adopted through #3438, was also merged forward before regeneration.
- Repository-wide `cargo fmt --check` remains red on pre-existing formatting in files outside this unit, including `crates/repo-conformance/src/meta.rs`, core tokenizer tests, and float-train sources. The scoped `cargo fmt --check -p xtask` passes. I did not reformat unrelated code.
- `uor-harness check` retains one pre-existing error: the untouched `docs/harness/prep/m1-close-saved-model-2026-10-26.md` missing its `Stop rule`. It also emits legacy warnings, including the dated session-goal record not yet stating M1's 6/6, unfrozen M5/M7/M8 panels, stale claims/presence, and old prep/ledger records. No panel or state-format error remains from this unit. I did not rewrite those sealed or out-of-scope records. The separate full loop gates are green.
- The mdBook preprocessor warns that it was built against mdBook 0.4.36 while mdBook 0.4.40 is installed. The build and link check pass. The WASM build retains an existing dead-code warning and package-metadata/update warnings; its tests pass.

## What changed

- Rebuilt the root README and current milestone record around the reviewed M1 close: pre-alpha, 1 of 8 complete, the saved discrete model's six bars at two seeds, and the explicit 62.5% p-f reference-reply limitation. K16 and older failures remain dated history.
- Reconciled the linked living book and crate READMEs. The discrete M1 model and separate float-trained FBS trunk are never presented as one qualified conversational model. M2/M3 are in progress; M4–M8 remain open.
- Propagated claude-217's opening/content limits, claude-218's offline key evidence and claude-219's free-generation result. On fresh p-v, the point key improves templates by +13/+8/+12 and exact replies by +9/+8/+12 at three seeds. It reaches only 12–18% of the forced-opening lift; the sign key is inconclusive at this panel size. The next unit trains and serves the point key in one integer model and scores it end to end. No useful-chat result is claimed.
- Updated `AGENTS.md`, `CHARTER.md`, `docs/integration/DECISIONS.md`, and `docs/rules/changes.md` only under the owner-approved status scope. D83 records the close without changing a bar or technical rule.
- After the D76 advisory, added `docs/harness/claims.json` to the item scope solely for this branch's existing required Kimi claim. The top-session scope decision is recorded in ORDER; no other claim or lab state changed.
- Updated the M1 item inputs, current milestone cache, and historical fields. The older close grade is described as bars (a)–(e); claude-211 supplies bar (f).
- Repaired the first review's findings: corrected `what-is`, FAQ, glossary and current D67 rules; added a dated close-grade/upstream-gate mapping; moved old M1 fields into history with all three qualification panels current and no invented singular panel key; marked M7 runtime work in progress and removed M8's blanket hold; corrected the controller's scope to finite qualification PASS and the full-dose diagnostic stop, with the next rerun NOT_RUN; and regenerated the milestone figure with M1 `COMPLETE` and M7 in progress. The harness caught and I corrected an attempted prose-list panel key before delivery.
- Repaired the second review's findings: updated the embedded hero, product, project, mechanism and FBS export figures plus `docs/SUMMARY.md`; labelled the 27-entry lineage figure as 19 October history; reconciled the mechanism board and research-line pages; recorded the exact M1 composition and prior M2/M8 holds in the tracker; corrected the remaining controller sentence; named the probe teacher-forced throughout; updated the Studio model manager with a red/green assertion; and rebuilt the inventory as 68 SVGs = 17 generated + 51 hand-maintained.
- Repaired the third review's findings: corrected the remaining glossary and research-line entries, discrete-learning and Hamilton rule checks, recurrence status, owner-release wording, figure D80 labels and recurrence chip; restored the M1 upstream evidence and project-record pointers; and labelled the legacy WebGPU probe as unrelated to the integer WebAssembly runtime. Also corrected the catalog/figure counts, p-f parity wording and the point-key size tradeoff.
- Repaired the fourth review's findings: separated A8's `5/5` finite qualification and Q1 diagnostic full-dose result from Stage T's `5,009` admission and M1's actual close; removed FBS projection and a separate normalization claim from M1; corrected the geometry and done-item records, the public side-project status, M2's historical line count, the rules-page close mapping and the pipeline figure. Also recorded the scope expansion and retained baseline negatives explicitly.
- Regenerated the architecture, milestone, training, serving, geometry, and chart figures; corrected hand-maintained roadmap, pipeline, and LUT figures, captions, alt text, titles, and descriptions. M1 and FBS throughput stay distinct, and D80/D81 integer multiply/divide wording is current.
- Corrected all three Studio status areas to 1/8, native M1 decode, stack-bound ByteLevel BPE, self-hosted assets, and the synthetic stand-in with no bundled qualified model. Strengthened the existing checks against stale status.
- Preserved `archive/**`, `docs/labs/**`, dated notebook rows, `docs/integration/current-state.md`, Claude's research files, and antigravity-113's two mechanism pages. `STATE.md`, `HANDOFF.md`, and `docs/roadmap/README.md` remain for the designated render workflow, not hand edits.
- Diff size: 108 files, 2,493 insertions and 1,792 deletions, including this REPORT and the complete non-archive README inventory.

## Measurements

| Result | Current documented evidence | Limit |
|---|---|---|
| DEV integer top-1 | 7,002 / 7,001 of 14,287 at two seeds; unigram 1,226, T2 6,300 | Saved discrete M1 model, not FBS |
| Fresh V2 | 7,215 / 7,228 of 14,479; unigram 1,279, T2 6,582, band 82 | Transfer within the registered panel protocol |
| Held-out phrasing p-f | 7,850 / 7,872 of 15,953; unigram 1,409, T2 7,375, band 81 | 62.5% of reference replies are verbatim TRAIN replies; not a generated-copy rate |
| Export and serving | Exact 14,287-row agreement and B4; Apple M1 decode 10,876 / 10,566 tok/s with 192-token prefill and 64-token decode | Not browser or useful-chat qualification |
| FBS opening failure | claude-217 p-s openings 17.4%, 13.7%, 18.6% against about 80% on later tokens | Greedy remains the decode; loop suppression is not content quality |
| Offline memory-key probe | claude-218 sign +46/+37/+33 on 1,218 missed p-u rows; point +125/+185/+114 | Teacher-forced/offline precursor, not the current free-generation result |
| Free-generation point-key opening | claude-219 fresh p-v templates +13/+8/+12 and exact replies +9/+8/+12 at three seeds | Partial: 27–31% correct keyed openings, 12–18% of forced lift; sign key inconclusive; integer serving and useful chat open |

## Checks run at the head

- `cargo test --locked -p xtask`: **220 passed, 0 failed**.
- `cargo fmt --check -p xtask`: pass. The repository-wide formatting exceptions are listed above.
- `cargo run -q --locked -p xtask -- figures`, `figures-math`, and `figures-charts`: **8, 5, and 4 figures current** respectively.
- `cargo run -q --locked -p uor-r4-claim-wording -- .`: pass, within that tool's supported Markdown surface; changed public prose was also scanned for stale status.
- `cargo run -q --locked -p xtask -- roadmap check`: **115 items OK**. `docs-catalog --check`: **1,175 records, 0 paths not found**.
- `cargo run -q --locked -p xtask -- archive-verify --base ec9a790fe5be73cd673f082eba86ced551b22d12`: **10,810 archive files, 0 modified, 0 deleted, 0 regressions**.
- `cargo run -q --locked -p xtask -- links --baseline … --renames-from …`: **6,013 sources, 23,104 checked, 1,202 baseline broken, 0 new broken**.
- `book-stage`, `mdbook-mermaid install .`, `mdbook build`, and `book-check`: **104 pages and 70 figures staged; 105 pages checked, 0 broken links**.
- `web/studio/build_serve_wasm.sh` and its worker verification: pass. The rebuilt WASM is 495,334 bytes and the synthetic stack 469,252 bytes; exact 32-token output, 9,782/5,366-byte checkpoints, bit-identical save/restore, mid-generation guards, six network traps, four model-source choices, and zero remote requests passed.
- `node web/studio/verify_studio_page.mjs` and `verify_studio_page_load.mjs`: both pass; both pages loaded with no external requests, and the planted external trap was detected. `verify_m7_parity.mjs` passed all 8 synthetic prompts; `verify_desktop_client.mjs` passed with `pageWired:false`. Its stale-lockfile refresh was restored after verification and is not part of the diff.
- Scorer evidence-integrity tests in `docs/labs/m1-d11-artifact-scorer/instrument`: **26 passed, 0 failed, 1 ignored** by the test's own missing-DEV-data guard. This was not a new model evaluation.
- Visual inspection covered every changed figure at wide and 390 px layouts, including the dedicated narrow architecture and repaired milestone figure. No clipping or M1/FBS measurement conflation was found; dense wide diagrams retain updated text descriptions.
- Reader-test after repair: the front page and start pages identify M1 on the saved 19-table discrete model, say no useful chat, name the point key's partial free-generation gain and the remaining integer-serving and M2–M8 gaps, and link the M1/claude-219 evidence. A separate stale-prose scan found only explicitly historical or out-of-scope matches and the named D41 exception.
- `uor-harness loop gates --full`: all 11 gates green.
- `uor-harness check`: completed with the pre-existing error/warnings detailed above.
- `uor-harness loop report --check`: **PASS**. `uor-harness loop pr-check --local`: **verdict PASS**. The D76 scope advisory remains by design because `rule_scope` reads the item at the merge base; the new claim-file scope entry takes effect after merge.

## Skills used

`using-superpowers`, `uor-research-architect`, `executing-plans`, `using-git-worktrees`, `test-driven-development`, `verification-before-completion`, `systematic-debugging`, `requesting-code-review`, `receiving-code-review`, `solution-integrity`, and `modern-web-guidance`. The accessibility guide informed the complex-figure descriptions; no icon guidance was applied.

## Still open

- Useful conversation is not established. The point key gives a partial free-generation opening gain; the next research step trains and serves it in one integer model and scores the whole model end to end. The sign key is inconclusive at p-v's size, not ruled out.
- M2 and M3 are in progress; M4–M8 are not complete. The Studio and desktop shell still use a synthetic stand-in without a bundled qualified product model.
- The independent exact-head review follows this commit. Roadmap sync, generated STATE/HANDOFF/roadmap views, issue verification, and the automatic docs-site preview follow the content merge through their designated workflows.
- The pre-existing formatting, harness, preprocessor, and WASM warnings above remain outside this documentation unit's repair scope.

## Plain summary

M1 really is complete: one saved discrete geometric model passed all six bars at two seeds and serves as integer artifacts on an Apple M1. It still does not hold a useful conversation, and p-f's 62.5% reference-reply overlap remains next to that claim.

The documentation now says which model passed, what the numbers mean, and what happens next. The separate FBS line has a real but partial point-key gain in free generation. Next it must train and serve that key in one integer model and score the whole model end to end. No product release or later milestone is implied.
