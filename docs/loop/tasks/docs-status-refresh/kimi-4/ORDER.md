# ORDER: comprehensive 30 Oct documentation refresh

Lab: kimi. Item: [docs-status-refresh](../../../../roadmap/items/docs-status-refresh.md) (#3380). Original base: `ec9a790fe5be73cd673f082eba86ced551b22d12`; current PR base: `4195ce2bf83d703b1acd3eb868c888d1da053479`. Earlier exact-head reviews inspected `f6acd4b8a9e01acfca4506ac070f72349cfe46de`, `9fb906dc37ab6fc1ec2b36d5ae007c44b1053394` and `95cb7388ada0386e9c3c1bea31f1766809f7938b`. Those are historical provenance, not the final head; the final head is the commit named in the PR and its fresh review.

## Goal and chain

Reconcile the living public documentation with the reviewed M1 close and the latest merged research, without overstating capability or touching other labs' work. Deliver pre-alpha, 1 of 8 milestones complete, the two-seed six-bar evidence, and the distinct limits on useful chat and new-phrasing replies through the README, linked book, living crate READMEs, figures, tracker inputs and Studio status. Then synchronize issues and render generated views.

## Scope

The item's committed scope at base, with these further limits:
- Living public prose and generator inputs only. No dated evidence, notebook rows, `docs/labs/**`, previous task folders, `docs/integration/current-state.md` or `archive/**`.
- No Claude active code/prep/lab/task/notebook files and no antigravity-113 `exact-memory.md` or `object-store.md`.
- No hand-edited `STATE.md`, `HANDOFF.md` or `docs/roadmap/README.md`; those follow the separate render workflow.
- New records only in `docs/loop/tasks/docs-status-refresh/kimi-4/**` (and a numbered repair if required).

Estimated: 50–80 living files, about 2,500 changed lines; size is reported, never capped (D48).

## Owner approval

The owner approved the documentation plan in Kimi's chat, explicitly authorizing status-only edits to `AGENTS.md`, `CHARTER.md`, `docs/integration/DECISIONS.md` and `docs/rules/changes.md`.

Answer: **"Approved Plan"**, `2026-10-30T15:30:07Z`.

No acceptance bar, technical rule, GPU cap, `.github/**`, `labs.json`, evidence or production deployment changes follow. The top Kimi session owns the status decision after its fresh decision review; no subagent decides it.

## Cancel probe

#2029 must be CLOSED; the older close-grade JSON must record bars (a)–(e) as `CLOSED_PASS`, and claude-211's separate `panel-score.json` must record `bar_f_pass: true`. If either changes or #2029 reopens, stop the M1 status rewrite. If an open PR overlaps a living path, preserve its work and merge main forward instead of overwriting it.

## Done-check

Every numbered done-when in the item is met: sourced M1/current-status prose, a complete non-archive README inventory, regenerated and visually checked figures, current tracker inputs with historical fields retained, roadmap sync/checklists and separately rendered views, mdBook/link/Studio checks, a source-linked FACTS sheet, full exact-head review and negatives-first REPORT. No useful-chat claim, no model-path conflation, no unmerged result.

## Stop list

No model/GPU experiment, paid compute, new dependency, workflow edit, direct main push, force push, destructive deletion or production `gh-pages` publication. Do not edit outside the item's scope; STOP and re-scope first. Do not treat claim-wording's supported Markdown surface as a whole-repository scan: run a separate changed-public-prose check for the other paths.

## Scope decision: required branch claim

After `loop pr-check --local` reported the D76 advisory, the top Kimi session reviewed the whole project context, item, ORDER, claim diff and delivery rules. `docs/harness/claims.json` is added to the item scope solely to carry this branch's existing Kimi claim. No other claim, lab state, rule, bar or capability changes. Approved in this top session on 30 Oct 2026; reverse by removing that scope entry and claim together.

## Scope decision: evidence-critical public status pages

The fourth independent review identified a small set of public pages that either make current main-project status claims or contradict completed evidence. The top Kimi session reviewed the actual A8 finite/full-dose/Stage T and M1 close records and adds only `projects/README.md`, `docs/harness/README.md`, `docs/roadmap/items/m1-a8-full-dose-rerun.md`, `legal-construction-exact-solve.md`, `recurrence-gates-six-recurrence-fit.md`, `hamilton-gate-offline-float-sheet.md`, `r-hamilton-gate-suppression.md`, and `m1-w2-capacity-capture.md` to the item scope. These are status/evidence corrections only: no acceptance text, bar, rule, model, GPU run, sealed lab file or other lab's active work changes. Approved in this top session on 30 Oct 2026; reverse by removing these scope entries and their status edits together.

## Scope decision: historical session guide

The historical session guide's counts no longer match the corrected registry. Only `docs/labs/session-goal.md` is added for a current registry cross-check and an explicit historical marker for its old prompts. Harness source and fixture files are excluded from this documentation-only unit. No model code, bar, evidence record or other lab file changes. Approved in this top session on 30 Oct 2026; reverse by removing this scope entry and its text correction together.

## Scope decision: living contributor status links

The changed-page scan found retired plan/status links in the living contributor guide. `docs/contributing/OUTSIDE.md` is added only to replace those links with `PLAN.md` and rendered `STATE.md`. No guidance, bar, code or other lab file changes. Approved in this top session on 30 Oct 2026; reverse by removing this scope entry and its link correction together.

## Scope decision: living Studio README

The README inventory identified `web/studio/README.md` as a living public page with a stale 13 October live claim. It is added only to record DeepSeek's verified 30 October live publish and local build instructions, not Studio source, deployment or other lab work. Approved in this top session on 30 Oct 2026; reverse by removing this scope entry and its documentation correction together.

## Scope decision: living labs onboarding hub

The exact-head review identified `docs/labs/README.md` as the live labs onboarding hub, not dated evidence. It is added only to make M1, the current planning pointers and lab-entry status lead the page, with the dated joining records preserved below as history. No dated `docs/labs/**` evidence, model code, acceptance rule or other lab file changes. Approved in this top session on 30 Oct 2026; reverse by removing this scope entry and its documentation correction together.

## Scope decision: live Studio evidence and its public item

The exact-head review found the live/staged M1 distinction and a future-dated redesign in `docs/roadmap/items/m7-studio-live-model-sources.md`. Kimi verified the existing live gh-pages `e62ea481` bundle and M1 manifest directly; this item is added only to record that evidence, remove the future date and make the non-repository workflow-proposal attribution consistent. No deployment, Studio source, owner workflow, model code or other lab evidence changes. Approved in this top session on 30 Oct 2026; reverse by removing this scope entry and its documentation correction together.

## Scope decision: stale roadmap status items

The final review identified nine roadmap items whose current text still describes pre-M1 results, retired holds, or a paused Studio deploy: `m1-fresh-panel-transfer`, `m1-export-gap-and-toks`, `m2-train-grounded-replies`, `m2-fresh-heldout-panel`, `m2-pin-train-mask-sheet`, `m3-qualify-memory`, `m7-studio-connect-wasm-engine`, `m7-studio-self-host-page-assets`, and `m8-release-packaging`. They are added only for sourced status corrections and current pointers. No acceptance bar, model code, evidence, owner workflow, or other lab's active research changes. Approved in this top session on 31 Oct 2026; reverse by removing this scope entry and its documentation corrections together.

## Scope decision: claude-222 implementation status

`point-key-integer` is added only to record the merged claude-222 first unit: the integer point table, search and `/8` record, with its 380 library tests, 18 negative gates and full serving audit, plus what was then the remaining unit-2 panel decode/grade and subsequent real-table export, p-x and laptop measurements. No acceptance bar, model code, evidence or other lab's active research changes. Approved in this top session on 31 Oct 2026; reverse by removing this scope entry and its documentation correction together.

## Scope decision: claude-224 implementation status

During repair, #3454 merged claude-224 on `main` as `18c4c3da70f35b5c8c9348297df05cfd4d8e452d`. The same living documentation scope records its completed unit-2 panel prompts, integer decode and grade, its 383 integer-library and 185 float-library tests, 13 negative gates and full serving audit, the shared 192-token context cap, and unit 4's remaining real export, fresh p-x score and laptop measurements. No acceptance bar, model code, evidence or other lab's active research changes. Approved in this top session on 31 Oct 2026; reverse by removing this scope entry and its documentation corrections together.

## Scope decision: retention dependency registration

#3455 then merged claude-225 on `main` as `a5eb6863a05b0177de189c7ca39da4025843c167`. Its source inspection found that the integer artifact does not serve (r-drop)'s trained retention recurrence and registered `integer-retention-rates` (#3456) before the point-key run. The same living documentation scope records that prerequisite and the unchanged NOT_RUN scope; no model code, acceptance bar, evidence or other lab's active research changes. Approved in this top session on 31 Oct 2026; reverse by removing this scope entry and its documentation corrections together.

## Scope decision: retention implementation status

#3457 then merged claude-226 on `main` as `bd9660fa76cf14e5a96d9c87b53788797d6d2fbd`. `integer-retention-rates.md` is added to record the completed flag and retained recurrence, their 386 integer-library and 186 float-library tests, nine negative gates and full serving audit, and the remaining GPU parity before point-key unit 4. No acceptance bar, model code, evidence or other lab's active research changes. Approved in this top session on 31 Oct 2026; reverse by removing this scope entry and its documentation corrections together.

## Scope decision: retention parity result

#3458 then merged claude-227 on `main` as `cd328d3af791fbc8b3b38a11e40bf65f2dbf3f97`. The same living documentation scope records G-Ret-Parity PASS: 1,531 / 1,536 sequences and 4,023 / 4,096 single tokens within the 3% bar, the decay control, the unexplained 1.78% residual, the closed retention item, and point-key unit 4's remaining real export, fresh p-x score and laptop measurements. No acceptance bar, model code, evidence or other lab's active research changes. Approved in this top session on 31 Oct 2026; reverse by removing this scope entry and its documentation corrections together.

## Scope decision: run instruments status

#3459 then merged claude-228 on `main` as `c71e85788da3a6ef09909bb4d525dd2e53ff30b3`. The same living documentation scope records the independent reference scan, table read-back and BOS refusal, 390 integer-library tests with one ignored, four negative gates and full serving audit, and unit 4's remaining real export, fresh p-x score and laptop measurements. No acceptance bar, model code, evidence or other lab's active research changes. Approved in this top session on 31 Oct 2026; reverse by removing this scope entry and its documentation corrections together.

## Scope decision: real-run instrument fault

#3460 then merged claude-229 on `main` as `856edac67ab94d42e564700e7650229b2968efde`. The same living documentation scope records row 0: the integer session used the native writer instead of the trained whole-table nearest readout, so every grade pair is (0, 0). This is an instrument fault, not a mechanism verdict or throughput result. Unit 5 repairs the readout and checks integer/float greedy parity before rerunning p-x. No acceptance bar, model code, evidence or other lab's active research changes. Approved in this top session on 31 Oct 2026; reverse by removing this scope entry and its documentation corrections together.

## Scope decision: readout repair registration

#3461 then merged claude-230 on `main` as `54f20ac7bb9203cb4949f8161a72d1c5fba53408`, registering `integer-nearest-readout`. The same living documentation scope links the repair and its required `I-Greedy-Parity` check before the p-x rerun; it is not a repair result. No acceptance bar, model code, evidence or other lab's active research changes. Approved in this top session on 31 Oct 2026; reverse by removing this scope entry and its documentation corrections together.

## Scope decision: nearest-readout implementation

#3463 then merged claude-231 on `main` as `62308fcbce74170316fd5672716fc313b37aca16`. `integer-nearest-readout.md` is added to record the completed native/browser nearest readout and export, 395 integer-library tests with one ignored, seven negative gates and full serving/WASM audits, and the remaining end-to-end greedy parity and p-x rerun. No acceptance bar, model code, evidence or other lab's active research changes. Approved in this top session on 31 Oct 2026; reverse by removing this scope entry and its documentation corrections together.

## Scope decision: nearest-readout parity result

#3464 then merged claude-232 on `main` as `4195ce2bf83d703b1acd3eb868c888d1da053479`. The same living documentation scope records the parity MISS in every cell: DEV first-token agreement 291/285/284 of 310 and p-w agreement 911/777/64 of 1,202 at seeds 1–3. The next work localizes the mismatch across the trunk, Hamming read and readout before parity and p-x can pass. No acceptance bar, model code, evidence or other lab's active research changes. Approved in this top session on 31 Oct 2026; reverse by removing this scope entry and its documentation corrections together.

## Scope decision: active M1/M3/M4 dependency pages

The exact-head review at `340337195` identified three active dependency pages whose current text still asserted pre-M1 status, retired owner-release rules or a retired zero-multiply/divide audit. The top Kimi session reviewed the A8 qualification record, M3 consumer thresholds and M1/M4 serving evidence and adds only `m1-a8-finite-cuda-qualification.md`, `object-store-consumer-m3.md` and `m4-export-uorlut01-cli.md` to the item scope. These edits record existing evidence and D67/D80/D81 status without changing an acceptance bar, technical rule, model, sealed record or other lab's active work. Original item-start status is retained only as explicit history. Approved in this top session on 31 Oct 2026; reverse by removing these scope entries and their documentation corrections together.

## Scope decision: W2 and Stage T completion records

The exact-head review at `c14f12362` found the same completed W2/Stage T sequence still described as future work on three living item pages. The top Kimi session reviewed the W2 code-utilization, Stage T and C2 records and adds only `m1-w2-code-utilization.md`, `m1-bvg-k-stage-t-training.md` and `m1-bvg-k-block-apply-c2.md` to the item scope. These edits record completed runs and their exact negative/diagnostic scopes, preserve the original acceptance text as history, and do not turn Stage T's diagnostic stop into a Stage 2 pass. No acceptance bar, model, sealed record or other lab's active work changes. Approved in this top session on 31 Oct 2026; reverse by removing these scope entries and their documentation corrections together.
