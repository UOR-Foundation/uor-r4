# REPORT: one-model-end-to-end, deepseek-273 (aligned-corpus GPU smoke)

Lab: deepseek. Item:
[one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md)
(#3407). Order: [ORDER.md](./ORDER.md). Base code: `7a450a88f`; launch code:
`f44afe7a973039fd4b28d69b6050a65ca6680d4d`; contract implementation source:
`4add144ebdc9087e47fe3fa830fb21116278644e`. All invoked-script hashes and
the exact reviewed execution-head procedure are in
[`launch-pins.env`](./launch-pins.env).

**Status:** pre-run, pending fresh-context review 11. No GPU model job has
started.

## What did not work

The first registration did not pass adversarial review: it changed the arm-r
batch without saying so, omitted the scenario panels from the execution path,
used an unbound legacy artifact, had no lawful non-neural baseline, and did not
seal failures or verify current compute capacity. A second review then found
that the source/job identities were only self-referential, the watcher had no
stall/cost control, scoring was not source-pinned, and the first failure seal
started too late. A third review found that the terminal marker preceded
readback, teardown did not wait for a verified seal, the cumulative cost and
time bounds were not implemented, scoring did not compare every row and
identity, the P1 classifier was wrong, and the stack contract omitted the
training seed. A fourth review then found a guaranteed 192-versus-256 corpus
context failure in scoring, stale contract implementation provenance, an
incomplete score command graph, a store check that did not reserve the prefix,
and a healthy-run cancellation that conflicted with CHARTER §5.4. Those
findings were repaired. A fifth review then found that preflight reports were
field-checked but not byte-pinned, a fixed reservation path could overwrite an
older claim, a failed TERM had no escalation, and the score command graph
omitted `--out`. Those findings were repaired. A sixth review then found that
P2 used 914 rows instead of the registered 882, caller-supplied run IDs could
bypass the 128-bit nonce requirement, unsealed exits did not always preserve a
pending-stop record, the command graph still abstracted required paths, and
pre-watcher cleanup did not record its billed interval. Those findings are
repaired. A seventh review then found that the launcher audited its own empty
run directory, both entry points defaulted to the failed review-6 receipt,
pre-watcher and watcher failure exits did not consistently persist both budget
and pending-stop records first, the remote claim was still overwrite-capable,
the command graph omitted smoke paths and had mismatched timeouts, and a score
seal failure could retain a classified decision. Those findings were repaired.
An eighth review then found that an active GPU run could cross the budget
without an extension record, the pre-watcher budget omitted staging and lacked
an inter-process lock, the contract implementation source was stale, and
pre-watcher network calls could hang unmonitored. Those findings are repaired
in the current registration. A ninth review then found that healthy extension
still depended on an idle GPU, budget-result writes were outside the launcher
lock, unknown pods could be marked KEEP, pre-watcher hangs remained, job
failure provenance was incomplete, and the Markdown renderer dropped
timeout-only command details. Those findings are repaired in the current
registration.

The first billed launch at review 10 stopped before watcher or job start:
`uor-pod` now bootstraps source at `/root/build/src-<sha>`, while the launcher
still verified `/workspace/uor-r4`. The launcher recorded the failure and
released and destroyed the pod. The measured failed-launch cost was
`$0.041756`; no model process ran. The reviewed path now binds the bootstrap
source root and writes only inputs, runs and seals under `/workspace/uor-r4`.

## What changed

The scorer keeps the original schema `/1` when the control is off and emits
schema `/2` with an optional previous-token count table on exactly the selected
artifact cases. The planned run uses arm r's batch-16 geometry recipe plus
explicit train-as-served rounding, a complete receipt-bound input bundle, a
bound `/10` stack contract, exact source/job hashes, a same-built-in smoke
score, exact file-set failure sealing, a post-readback teardown sentinel,
persistent cumulative time/cost accounting, stage heartbeats and full
model/table/base row identity checks. The contract now names the implementing
source, describes all four score commands plus classification and preflight,
and an absent internally generated 128-bit prefix is claimed by an exclusive
create/read-back
per-run object before model work. Preflight reports are byte-pinned as well as
structurally checked. Healthy ceiling crossings are explicitly extended and
persisted only while the marker is RUNNING, the job is present, GPU compute is
active, and progress is fresh; stalled process groups are TERMed, retried, then
KILLed with an unsealed pod preserved.
P2 now counts exactly the 882 scored non-copy rows, with the 32 unscored
general rows excluded. Job and scoring failures write row 0 explicitly.
Pre-watcher and watcher failure paths acquire the shared budget lock and persist
budget and pending-stop records atomically before release or keep. An
unverified pod is never marked KEEP. The command graph includes the smoke path,
exact wrappers and matching timeouts.

## Measurements

No GPU measurement exists yet. Preflight arm-r and count-table readings are in
the prep sheet and its HF-store paths. Arm-r reproduced 10 development and 11
Fresh copy hits, zero grounded hits, zero abstentions, and zero distractors;
the count table produced zero on every scored metric.

## Checks run at the head

- `scenario_grade`: 11 passed, including the TRAIN-context mismatch refusal.
- CPU scenario-grade CLI: 3 passed.
- Full `uor-r4-cpu-train` library: 506 passed, 5 ignored.
- Stack-contract generation and check: 5 contracts current.
- `xtask`: 228 passed.
- Byte-pinned preflight gate: both reports match their frozen SHA-256s and pass
  with corpus context 256.
- Decision program: parses the four pinned preflight reports and emits schema
  `/3` row 4 as expected for the unchanged base.
- Shell syntax and ShellCheck: pass.
- Mocked launcher/watch safety: 7 passed, covering an early identity failure,
  untouched unknown and mismatched pods, durable records before owned-pod
  teardown, lock-serialized sealed-failure publication, healthy `gpu=1`
  extension, and stale `gpu=0` stop instead of extension.
- Exact-head transcripts are retained under
  `local/deepseek/scenario-gpu-smoke-2026-11-11/checks/`.
- Clippy with `-D warnings`: stops on pre-existing warnings in
  `prompt_reply_rank.rs` and `suffix_arbitration.rs`, outside this change.

## Skills used

`working-discipline`, `solution-integrity`, `scope-guard` and `context-guard`
kept the correction bounded and checked against the registered evidence path.

## Still open

Review 11, the GPU smoke, bound export, held-out scenario reads, row
classification, final cost and post-run exact-head review remain. This file
will be replaced with the completed report before merge.

## Plain summary

The registration and its control instrument are now ready for review 11. No
model result has been claimed.
