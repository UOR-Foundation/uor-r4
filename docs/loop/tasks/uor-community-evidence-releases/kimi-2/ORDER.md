# ORDER: repair public release definitions, kimi-2

Lab: kimi. Item: [uor-community-evidence-releases](../../../../roadmap/items/uor-community-evidence-releases.md) (#3492). Base: `d305f2d2f` plus merge-forward. Repair of PR #3495 after fresh review at `2bd8976c3`.

## Finished before this repair

- Prepared and extracted both trial archives; passed host/telemetry/default-off checks and full repository gates; merged fresh `main` without changing release files.
- Fresh review rejected premature published/downloadable wording, a CUDA archive link to an omitted roadmap item, and unlabeled historical telemetry evidence. It found the measured scopes, asset commands, and non-claims otherwise accurate.

## Goal

Make every public status truthful before publication, make retained archive links self-contained, distinguish historical host-only telemetry evidence from the later official-collector PASS, and bind exact source-diff proof to the repaired head.

## Scope

- `projects/uor-cuda-inverted-file/README.md`, `OUTREACH.md`, `RELEASE.md`
- `docs/telemetry/uor-conventions.md`, `RELEASE.md`
- `docs/loop/tasks/uor-community-evidence-releases/kimi-2/**`
- Final archive input additionally includes `docs/roadmap/items/uor-cuda-inverted-file.md` as data only; no roadmap edit or generated view change

No implementation, measured evidence, model, artifact, Claude-owned, owner-path, actual release, tag, asset publication, or upstream change.

## Cancel probe

Stop if either release already exists, if any command in the pending documents requires a live release URL, or if `git diff 719a9a786…HEAD -- projects/uor-cuda-inverted-file/{src,tests,include,Makefile}` is nonempty. Documentation may announce only a prepared/pending release until publication and read-back complete.

## Done when

- Public repository text uses prepared/pending language and has no broken live-release claim. Actual publication wording belongs to the later release notes/close PR.
- The CUDA archive includes the roadmap item linked by retained kimi-4/5/6 records; rebuild and recheck its new member count and links.
- OpenTelemetry notes explicitly identify `collector-results.json` as the later official-collector PASS and `evidence.json`/`delivery-checks.txt` as earlier host-only historical records, including their NOT_RUN/124-item scope.
- The exact source-diff probe is empty; trial archives, full gates, claim/roadmap/render/diff checks, and local policy pass at the final text head; fresh review approves.

## Stop and report

Stop after repaired metadata merges. Do not mark #3492 done, flip repository status to published, or create releases until the next order performs publication and read-back verification.
