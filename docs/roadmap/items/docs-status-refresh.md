---
id: docs-status-refresh
title: Bring every status page, picture and tracker up to 30 Oct 2026
milestone: research
area: tooling
status: ready
open_to: lab
needs_gpu: false
depends_on: []
done_class: host
scope: [README.md, ROADMAP.md, PLAN.md, AGENTS.md, CHARTER.md, docs/README.md, docs/SUMMARY.md, docs/start/*.md, docs/products/*.md, docs/model/*.md, docs/geometry/*.md, docs/research/README.md, docs/research/findings.md, docs/research/open-questions.md, docs/research/history.md, docs/research/lines/README.md, docs/research/lines/*.md, docs/research/notebook/README.md, docs/mechanisms/*.md, docs/integration/project-track.md, docs/integration/current-state.md, docs/integration/DECISIONS.md, docs/rules/README.md, docs/rules/changes.md, docs/harness/milestones.json, docs/figures/**, xtask/src/figures.rs, xtask/src/figures_charts.rs, crates/*/README.md, web/README.md, web/studio/index.html, web/studio/verify_studio_worker.mjs, docs/roadmap/issues.json, docs/roadmap/items/one-model-end-to-end.md, docs/loop/tasks/docs-status-refresh/**, docs/roadmap/items/docs-status-refresh.md]
---

## Why it matters

The owner, 29 Oct 2026, in claude's chat: "can you please dispatch subagents to go a full repo documentation update including all of the readme's and issues and PR tracking that we have completely let lapse? make sure they all include these latest decisions from last night and the planned path forward - update all documentation that this touches to give a real status of the current project (with the pictures updated as needed and all of the documentation on the website updated if needed too)".

On 30 Oct the owner assigned Kimi the follow-up: "Please make sure the documentation on the github repo is updated so that it accurately reflects where the project currently is (I think we may have M1 completed now). All readme's especially the main one, must be kept clean, up to date, and accurate (plus engaging like it currently is since this is an open research project). If you are able to update the pictures, that would probably be good too because some of them are out of date. There is a large amount of linked documentation that I think is mostly correct, but having all of that cleaned up would be great as well. Claude is currently working on the project, but I am assigning you the job of housekeeping. Please try hard not to make mistakes as this is a highly complex coordination and execution task. Please ask me if you have questions or need something".

## What is missing

- **M1 closed after the 29 Oct refresh.** #2029 and claude-211 record two saved integer artifacts passing all six bars, but the README, root roadmap, book, milestone cache, status figures, crate READMEs and Studio text still carry earlier states.
- **Current pages mix dates and model paths.** K16's dated negative is sometimes presented as the current best, the saved discrete M1 model is conflated with the separate float-trained FBS trunk, and older claims about exports, Studio wiring, training holds and owner releases survive beside newer ones.
- **Generated state needs its own delivery path.** `STATE.md`, `HANDOFF.md` and `docs/roadmap/README.md` must follow their committed inputs through the render workflow, not be hand-edited in a lab PR.

## Done when

1. **Every living status page in scope states the project as of 30 Oct 2026, with a source for each number:** pre-alpha, 1 of 8 milestones complete, M1's two-seed DEV/Fresh V2/p-f evidence and serving rates, and the p-f reference-reply limitation. M1 is not useful-chat qualification; the saved discrete model and FBS trunk stay distinct; current work comes only from merged evidence.
2. **Every non-archive README is inventoried.** Living project READMEs with stale current claims are corrected; dated evidence and unrelated vendor/side-project history are left intact.
3. **The generated pictures are redrawn from `xtask/src/figures.rs` and `xtask/src/figures_charts.rs`,** with current data and accessible descriptions. `cargo xtask figures`, `cargo xtask figures-math` and `cargo xtask figures-charts` pass; genuinely hand-maintained status pictures and their captions are corrected and visually checked.
4. **The trackers agree.** `docs/harness/milestones.json` and the roadmap item inputs reflect the current state without changing any bar. `cargo xtask roadmap sync --apply` runs from fresh `main`, milestone checklists are current, and `STATE.md`, `HANDOFF.md` and `docs/roadmap/README.md` are refreshed through the normal render workflow.
5. **The site source and Studio status build and check cleanly:** `mdbook build`, `cargo xtask book-check`, the docs link check with no new broken links, and the existing Studio status checks.
6. **A source-linked facts sheet, one fresh exact-head review and one REPORT** cover the whole diff, the negative scope and any unrun checks.

## How to start

- Reopen this item through the scope PR, then write the facts sheet from #2029, claude-211 and the latest merged REPORTs before editing public prose.
- Inventory living pages and edit them in coherent front-door, book, figure and tracker commits. Do not race Claude's active research or a competing documentation draft.
- Deliver the content PR, verify it on fresh `main`, then synchronize issues and render the generated views.

## Rules that apply

- **Owner paths are status-only here.** The owner's 30 Oct request and approved plan authorize recording M1's close in `AGENTS.md`, `CHARTER.md`, `docs/integration/DECISIONS.md` and `docs/rules/changes.md`; no acceptance bar, technical rule, GPU cap, `.github/**` or `labs.json` change follows.
- **A record keeps its date.** Notebook rows, REPORTs, `docs/labs/**`, prior task folders and `archive/` are history and are not rewritten. Claude's active code, prep, lab-result, task and notebook files are outside this unit.
- **Rendered views are not hand-edited.** `STATE.md`, `HANDOFF.md` and `docs/roadmap/README.md` follow the render workflow after their inputs merge.
- **Scoped claims** (CHARTER §5.6). A number names its run, seed count and panel; p-f's 62.5% reference-reply overlap is not a generated-copy rate; no milestone claim exceeds the evidence.

## Sources

- The owner's words of 29 and 30 Oct 2026 (quoted above).
- Closed M1 #2029 and `docs/loop/tasks/m1-heldout-phrasing-transfer/claude-211/REPORT.md`.
- Latest merged end-to-end evidence, including `docs/loop/tasks/one-model-end-to-end/claude-217/REPORT.md`.
- `xtask/src/figures.rs`, `xtask/src/figures_charts.rs` and `docs/harness/milestones.json`.
