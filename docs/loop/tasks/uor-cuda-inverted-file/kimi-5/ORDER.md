# ORDER: repair community evidence release registration, kimi-5

Lab: kimi. Item: [uor-cuda-inverted-file](../../../../roadmap/items/uor-cuda-inverted-file.md) (#3475). Base: `83d54d168` from fresh `origin/main`. Repair of PR #3494 after fresh review at `1d710e969`.

## Finished before this repair

- `43fda2583` ordered canonical registration; `276950deb` added the item, existing #3492 mapping, and render; `1d710e969` added the first report.
- Fresh review found two wording drifts, no gate ledger at the report commit, and an unresolved expected scope advisory for registering a new item file. No finding identified implementation, evidence, generated-roadmap, or issue-duplication defects.

## Goal

Correct the evidence wording, explicitly decide the canonical new-item scope case at top-session level, and bind full gates and local policy to the final text head before a new exact-head review.

## Scope decision and files

The top session reviewed CHARTER §5.3/§5.10, D76, the parent item, and the reviewer finding. The single advisory for `docs/roadmap/items/uor-community-evidence-releases.md` is explicitly accepted **only for this canonical new-item registration**: a new item cannot already be in the parent's base scope, D76 makes scope advisory, and this PR changes no implementation scope. This is not a blanket waiver; the other new item remains outside the parent's scope until registration merges.

- `docs/roadmap/items/uor-community-evidence-releases.md`
- `docs/loop/tasks/uor-cuda-inverted-file/kimi-5/**`
- Canonical roadmap sync/render outputs only if the item edit requires them

No other model, artifact, implementation, Claude-owned, owner-path, release, tag, asset, or upstream change.

## Cancel probe

Stop if the review's generated-file or issue-mapping findings were wrong, if a second issue would be created, or if correcting wording requires changing measured evidence. Otherwise keep #3475 and #3492 open and make no release claim.

## Done when

- The item says **official-collector** interoperability and keeps SDK/multi-runtime interoperability NOT_RUN; the report keeps #3475 open at its declared evidence scope.
- Full 11-gate run and `loop pr-check --local` execute at the final text commit; only the explicitly reviewed new-item scope advisory and known FILEMAP advisories remain.
- A fresh-context reviewer approves the corrected exact head, GitHub required checks are green, and the PR body still uses References only.

## Stop and report

Stop after the repaired registration merges. Do not publish releases, close either issue, or start upstream outreach in this PR.
