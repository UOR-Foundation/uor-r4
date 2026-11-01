# ORDER: make release registration claimable, kimi-6

Lab: kimi. Item: [uor-cuda-inverted-file](../../../../roadmap/items/uor-cuda-inverted-file.md) (#3475). Base: `83d54d168` from fresh `origin/main`. Second repair of PR #3494 after fresh review at `6a0e01da5`.

## Finished before this repair

- Registered #3492 with one mapping and a valid 125-item render; narrowed collector wording and recorded the canonical-new-item D76 decision; bound 11/11 gates and local policy to `6a0e01da5`.
- The next fresh review found that the new item's scope listed shared `docs/roadmap/README.md`, which makes `loop claim` refuse the item. It also identified stale public collector/replication wording and two omitted telemetry limits.

## Goal

Make the registered item claimable and align every public evidence statement used by the release with the accepted CUDA and official-collector records. Preserve the earlier repairs; do not publish.

## Scope decision and files

The kimi-5 top-session decision covers only the expected advisory for adding `docs/roadmap/items/uor-community-evidence-releases.md`. The public package README is already inside parent item #3475's scope. FILEMAP warnings were checked against the base and are pre-existing missing rows, not new release warnings.

- `docs/roadmap/items/uor-community-evidence-releases.md`: remove shared `docs/roadmap/README.md` from `scope:`; retain `issues.json`, which is not shared; add loaded-model and experimental-attribute limits.
- `projects/uor-cuda-inverted-file/README.md`: official-collector PASS and corrected independent replication PASS, with SDK/multi-runtime/production/GPU telemetry limits unchanged.
- `docs/loop/tasks/uor-cuda-inverted-file/kimi-6/**`
- Canonical sync/render outputs only if required

No implementation, model, artifact, Claude-owned, owner-path, release, tag, asset, or upstream change.

## Cancel probe

Stop if `docs/roadmap/README.md` is not in the harness's shared-file list, if the README corrections contradict sealed evidence, or if any additional scope warning appears. The shared roadmap file remains writable without being in item scope.

## Done when

- No new-item scope entry covers a shared file; the item is claimable by the harness's actual rule.
- README and item match official-collector PASS, corrected independent replication PASS, SDK/multi-runtime and real-loaded-model NOT_RUN, and experimental local `uor.*` attributes.
- Full 11-gate run, roadmap/render/diff checks, shared-file harness tests, and local policy execute at the final text commit; exact results are posted on the PR.
- A fresh-context reviewer approves the corrected exact head; GitHub required checks are green; References only.

## Stop and report

Stop after the claimable registration merges. Do not publish releases, close either issue, rewrite earlier repair history, or start upstream outreach.
