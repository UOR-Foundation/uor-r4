# REPORT: infra-loop-harness, deepseek-74 (record two roadmap items)

Lab: deepseek. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904).

References #3432, #3433

## What did not work, first

- No verifier implementation and no repair adoption happened in this PR; it records the gaps only.
- The first audit pass treated Yassine's suggestion as a nice-to-have. The fresh-context reviews then found that the frozen M6 bar and registry already name the verifier, so this is a record inconsistency as well as an enhancement.
- `cargo xtask ...` is not installed as a cargo alias on this main; the CI-proven form `cargo run -q --locked -p xtask -- roadmap ...` was used.
- The two sync applies are GitHub writes. Each dry run was read in full first. They refreshed #3416's body and the M1/M6 milestone children lists (first apply) and created the umbrella issue (second apply); this REPORT discloses those side effects rather than claiming them as the change.
- The M6 issue was created by an earlier draft apply (16:04Z) before the umbrella item existed; the second sync adopted it by its `<!-- roadmap:id=m6-code-task-rustc-verifier -->` marker instead of duplicating it.
- The first route for the three repairs (push the adoption commits to the fork branch and approve its workflow runs) does not work: on a public repository GitHub gives `pull_request` runs from forks a read-only `GITHUB_TOKEN`, and `required-transport` needs `checks: write` to publish the five ruleset names, so a fork PR can never enter the merge queue. The item records the same-repo re-home route instead (branch from `main`, ORDER commit, `git merge --no-ff` the contributor's commits unchanged, REPORT commit, new PR). The fork was never written to; this was found before any fork write.

## What changed

- New item `docs/roadmap/items/m6-code-task-rustc-verifier.md` (69 lines): M6 / evaluation, status ready, open_to anyone, needs_gpu false, depends_on `m6-reasoning-panel-freeze`. It says plainly that the verifier covers each frozen row's canonical/reference program and the transpiler, not model-emitted code (the graded CodeTasks answer is four integers).
- New item `docs/roadmap/items/infra-outsider-repairs.md`: one maintenance item scoped to the paths in PRs #3424/#3425/#3426, with the owner direction as the reason it runs; it is the item the three adoption ORDERs will name, and the last adoption PR sets it done (D70).
- `docs/roadmap/issues.json`: two lines added, `"m6-code-task-rustc-verifier": 3432` and `"infra-outsider-repairs": 3433`, written by `roadmap sync --apply`.
- `docs/roadmap/README.md`: rendered by `roadmap render` / `sync --apply` (114 items; never hand-edited).
- `docs/loop/tasks/infra-loop-harness/deepseek-74/{ORDER,REPORT}.md`.

Size: docs 4 files; generated 1 file; code 0 files; tests 0 files.

## Measurements

No measurement. Roadmap counts at the head: 114 items OK; by status ready 17, held 10, decision 1, done 86; M6 has 5 items (was 4) and infra has 22 (was 21). Sync plans: first 0 labels, 1 issue to create (#3432), 1 body update (#3416), 0 close, 2 milestone checklists; second 0 labels, 1 issue to create (#3433), 0 updates, 0 closes, 0 checklist changes. Applied: #3432 and #3433 exist, issues.json maps both, README re-rendered at 114 items.

## Checks run at the head

- `cargo run -q --locked -p xtask -- roadmap check` -> 114 items OK (counts above).
- `cargo run -q --locked -p xtask -- roadmap render --check` -> current.
- `cargo run -q --locked -p xtask -- roadmap sync` (dry, both times) -> exactly the plans above.
- `cargo run -q --locked -p xtask -- roadmap sync --apply` (twice) -> created #3432 and #3433 and wrote the generated files.
- `cargo run -q --locked -p uor-r4-claim-wording -- .` -> pass ("no unlinked machine-verified/exact-equivalence wording").
- `git diff --check` -> PASS.
- `uor-harness check` -> exit 0 (only pre-existing warnings; no error).
- `uor-harness loop pr-check --local` -> PASS (order, scope, report, commit-order, owner-paths, rendered-lock, read-path, filemap; review-record and required-checks are N/A locally).

## Skills used

`working-discipline`, `scope-guard`, `solution-integrity`, `uor-research-architect`; a fresh-context audit of Yassine's comment (`inv-yassine`) and a fresh-context layout/validation of both item files were used for the content and the limits stated here.

## Still open

- The verifier itself is not built. Item #3432 is ready and open to anyone, and it is not on the current lab path (M6 sits behind M4); #2865's pre-registration must later bind or drop the `rustc --test` clause (the item's "Cancel if").
- The three outsider repairs are still open. Item #3433 is ready; three adoption units follow, each adding its ORDER/REPORT to the fork branch and merging through the queue, with the last one setting #3433 done.
- The rendered `docs/roadmap/README.md` may be refreshed again by a render PR.
- The sync changed #3416's body and the #2029/#2034 checklists; those are tool outputs, disclosed here, not authored by this PR.

## Plain summary

This records two items: the missing `rustc --test` verifier that the frozen M6 code-task bar names (it checks the panel's own reference programs and transpiler, not code a model writes), and one maintenance umbrella for adopting three outside-contribution repairs. No model, panel, bar, hold or status changes.
