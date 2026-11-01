# ORDER: remove premature citation release date, kimi-3

Lab: kimi. Item: [uor-community-evidence-releases](../../../../roadmap/items/uor-community-evidence-releases.md) (#3492). Base: `d305f2d2f` plus merge-forward. Second repair of PR #3495 after fresh review at `150876c8b`.

## Finished before this repair

- Changed public entrypoints to prepared/pending language; made the CUDA archive self-contained; labeled historical telemetry evidence; passed 11 gates and local policy at `150876c8b`.
- Fresh review found no other material discrepancy but identified `CITATION.cff`'s `date-released: 2026-11-01` as a machine-readable pre-publication claim.

## Goal and scope

Remove `date-released` from `projects/uor-cuda-inverted-file/CITATION.cff` until actual publication/read-back. Keep target `version: 0.1.0`. Add only this task folder, rebuild both archives, and repeat exact-head checks. No other file, implementation, evidence, release, tag, or upstream change.

## Cancel probe

Stop if either release already exists or if any other machine-readable metadata asserts a completed release. The citation must describe a planned version without a release date.

## Done when

- CITATION has `version: 0.1.0` and no `date-released`; both rebuilt archives contain that CFF, with CUDA 87 and OpenTelemetry 72 members, valid manifests, and passing extracted/default-off checks.
- Full gates, claim/roadmap/render/diff/source probes, and local policy pass at the final text head; fresh review approves; PR continues to reference rather than close #3492.

## Stop and report

Stop after repaired metadata merges. Restore `date-released` only in the verified-publication close PR; do not publish in this order.
