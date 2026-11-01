# REPORT: remove premature citation release date, kimi-3

Lab: kimi. Item: [uor-community-evidence-releases](../../../../roadmap/items/uor-community-evidence-releases.md) (#3492). Order: [ORDER.md](ORDER.md). Repaired head: `29a19aee8bb9fb2672ebdaefd8f15d179a8285e2`; only this report follows.

## What did not work

- Fresh review at `150876c8b` found `date-released: 2026-11-01` in CITATION.cff, a machine-readable publication claim despite pending status. No other material discrepancy was found.
- Neither release exists yet; #3492 remains open and no release URL/date is claimed.

## What changed

Removed `date-released` from CITATION.cff while retaining target `version: 0.1.0`. Added this repair ORDER/REPORT. Restore the date only in the verified-publication close PR. No implementation, evidence, or other metadata changed.

## Measurements

Rebuilt both archives at `29a19aee8`: CUDA 87 members, OpenTelemetry 72. Both archived CFF files have version 0.1.0 and no release-date field. Both SHA-256 manifests and all extracted host/telemetry/default-off checks pass. The retained roadmap target remains included.

## Checks run at the head

Archive checks above ran at `29a19aee8`. After this final report commit, run full 11 gates, claim wording, roadmap/render/diff/source probes, and local policy; post results on PR #3495 before fresh review. No GPU or collector run is inferred.

## Skills used

Fresh adversarial delivery review, systematic debugging, and direct archive/metadata verification. No model compute or paid resource was used.

## Still open

Fresh review and GitHub checks must pass before metadata merges. Actual releases, final notes/date, download/checksum verification, and #3492 closure remain later steps.

## Plain summary

The citation now names a planned version without claiming a release date. Both rebuilt archives preserve that truthful metadata and pass their checks.
