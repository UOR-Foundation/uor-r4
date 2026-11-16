# REPORT: m2-donor-corpus-generation, claude-401 (Casey's 4-pod corpus caps recorded as D88)

Lab: claude. Order: [ORDER.md](ORDER.md).

## What did not work, first

- **No corpus is measured here.** This unit records a decision only. The 9.5k kept tokens/s figure behind it comes from
  tuning segment A (the last 5 minutes of an 8-minute run), not from the measured hour. The production unit (claude-400)
  reports the measured rate.
- **Two leases failed before the run.** Pod 1 (4×H200 NVL) had no NVLink and was downed at once, for about $0.52.
  Pod 2 (vast-60885691, 4×B200) never reached SSH in 900 s and was deleted by `uor-pod`, for at most about $11.
  Pod 3 (vast-60886842) is the measured pod.
- **"$43.41/h" is a per-pod price.** At 06:50Z only one more Vast datacenter 4×B200 was listed at that price or less
  (machine 137780). An 8×B200 at $85.01/h has the same per-GPU price but is outside Casey's words, so it needs the
  owner's word.
- **"This run only" has no tool enforcement.** The pod tool reads `UOR_POD_MAX_RATE` per command, and D88's text is the
  only limit.

## What changed

- **D88** in `docs/integration/DECISIONS.md`: up to 4 pods at $43.41/h each ($173.64/h in all) for the full 1B corpus
  run, then back to 4 pods and $8/h. It also lists the measured-hour approvals that came before it.
- **The 16 Nov D88 entry** in `docs/rules/changes.md`, quoting Casey's answers.

## Measurements

None. This is a record.

## Checks run at the head

- `loop pr-check --local`: see the PR.
- `git diff --check` is clean.

## Skills used

- `scope-guard`: two owner-path edits named in the ORDER, nothing else.

## Still open

1. The production unit's amendment 4: the fan-out sheet with its caps, the throughput tuning (concurrency, dropped
   tokens, prefix caching) and its non-author FIT.
2. Whether an 8×B200 pod at about $85/h may stand in for two 4×B200 pods. That needs the owner.

## Plain summary

Casey allowed the full 1B corpus to run on up to four rented 4×B200 pods at $43.41/h each, and asked for it to be as
fast and as cheap as possible. This change writes those words into the decision log and the rules change log.
