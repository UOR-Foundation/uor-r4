# REPORT: one-model-end-to-end, deepseek-25

Lab: deepseek. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md)
(#3407). Order: [ORDER.md](./ORDER.md). Laptop only; no pod, model step or
spend.

## What did not work, first

- **The controller waited on the wrong process.** `uor-pod run` starts a
  detached remote job, prints its log path and returns after submission. The
  old controller waited only on that submission process, so it could check the
  store and tear down the pod while training was still running.
- **The first parser expected the wrong output shape.** `uor-pod` puts
  `Log: PATH` at the end of its one-line “Started on…” message, not on a
  separate line.
- **Directory appearance was not checkpoint completion.** The trainer creates
  `step-N/` before writing its tensor and sidecar files. Freezing on the
  directory could copy a partial checkpoint and fail or seal invalid evidence.
- **An existing pin was not a launch-pin check.** The controller could upload a
  job from a different or dirty local checkout while the remote source still
  checked out the named pin.
- **A log file alone did not prove the job was alive.** If the remote wrapper
  died without writing `# exit=N`, polling the existing log could continue
  indefinitely while the pod billed.
- **The store preflight treated failure as an empty output root.** A failed
  listing could therefore rent a pod that could never upload its seal.
- **A missing expected checkpoint was silently skipped.** A trainer that
  exited zero after writing only some of the four checkpoints could pass the
  post-run loop.
- **The registered export command could not export the train-as-served base.**
  `cpu-fbs-export` requires both `--raw-gates` and `--raw-transition`; without
  them the job would fail before the first base control.
- **The generated pod session failed uor-pod's lowercase validator.** The UTC
  timestamp introduced uppercase `T` and `Z`.
- **The launcher's `$1.15/h` cap was global, not per-pod.** With other labs
  already running, it could reject a valid one-GPU launch; it also left the
  owner's actual four-pod/$8-per-hour caps dependent on ambient environment.
- **A live hung job could renew forever, and cleanup could not delete it while
  its GPU lock remained held.** There was no file-progress watchdog or active
  process-group termination.
- **A failed panel or copy could be reported as a completed checkpoint.** The
  training function ran in a negated conditional, which suppresses Bash
  `errexit`; panel failures were hidden by the function's final `printf`, and
  the checkpoint could get its scored marker anyway.
- **The seal could race its own asynchronous `tee`.** The log writer was not
  joined before the hash manifest was made.
- **The seal budget was false.** It retained duplicate checkpoints and raw
  snapshots, while the packet promised an under-100-MiB seal.
- **An interrupt could archive a failed run while returning success.** The
  signal trap preserved a zero status when failure cleanup itself succeeded.
- **The seal included the panel build target and duplicate downloads.** That
  contradicted the sheet's retained-storage projection and risked uploading
  build products as research evidence.

## What changed

- `run-s1.sh` submits once and parses the log path from the actual `uor-pod`
  success message.
- It polls the remote log for the terminal `# exit=N` footer while the existing
  lease renewal loop remains active.
- A missing log, nonzero remote exit or missing/failed seal fails the
  controller before a successful teardown path.
- `gate.sh` pins the detached-job rules and rejects any return of the old
  submission-wrapper wait.
- `test-mutants.sh` exercises the actual `uor-pod` log-line shape and proves
  submission-only waiting, ignored remote exits and missing log paths are
  rejected.
- The trainer now writes `.checkpoint-ready` only after the model, sidecars and
  served restore complete. The job waits for that marker before freezing and
  refuses an unmarked checkpoint after trainer exit.
- `train_and_score` is no longer called in a negated conditional. It records a
  failed trainer in `TRAIN_FAILURE`; every panel, export, grade and reading is
  explicitly checked, and a checkpoint is marked scored only after all of them
  succeed.
- `gate.sh` pins both sides of the completion protocol and its mutation suite
  rejects a return to directory-only checkpoint detection.
- The base control now runs teacher-forced ranks under both renderings and
  checks the registered one-user 18/138/873, per-turn 5/7/13, per-turn top-8
  198/256, free-generation 0/256, and continuation 132/512 readings exactly
  before training.
- The launcher now requires local `HEAD == UOR_SFT_PIN` and no tracked
  modifications before it may upload the controller or rent a pod.
- The remote probe reports the terminal exit when present, the GPU lock while
  the job runs, or a stopped state when the log exists but the job no longer
  holds its lock. Three consecutive stopped probes fail the controller.
- Store access is checked against the existing topic before rental. A listing
  failure aborts; only a successful listing can prove the new run root is
  absent.
- The job now requires the exact 512-row sample fingerprint on every grade,
  verifies `init.json` against the registered model/config/run hashes, and
  reproduces the registered corpus-panel overlap audit before training. The
  overlap report is checked by content because its JSON embeds input paths and
  therefore has a run-root-dependent file hash.
- A missing expected checkpoint fails unless a registered stop already ended
  the schedule.
- Base export and every checkpoint export now pass both raw served-snapshot
  flags, and the base artifact must reproduce `9a370566…`.
- Run IDs are lowercased before the uor-pod session is formed. The global caps
  are pinned to four pods and $8/hour, the provider is pinned to Vast, and the
  launch remains one RTX 5090.
- The exact pin must be contained in freshly fetched `origin/main`.
- Launch records free space, force-reads memory state, and authenticates or
  records absence of `model-time.json` and the storage receipt before rental.
- The controller fails after 30 minutes without a newer file under the run
  root. Failure cleanup terminates the exact remote process group, waits for
  GPU 0's lock to clear, then seals the pod through `down --to-store`.
- The base archive is extracted only for the checkpoint and run config; the
  archive and temporary download root are removed before training. The panel
  build uses `/workspace/uor-r4/build`, outside the sealed run.
- The sealed job log is written directly, not through asynchronous `tee`.
  Duplicate checkpoint copies and raw snapshot files are outside or removed,
  and the job fails before upload if the completed seal exceeds 100 MiB.
- The first trainer attempt is micro-batch 16. Only a classified OOM permits
  one fresh micro-batch-8 attempt; both attempts keep separate trainer,
  artifact, grade, panel, scored-marker and reading roots.
- The launcher builds the Vast provider adapter from the reviewed checkout
  before calling `uor-pod status` or `uor-pod up`.
- Failure cleanup forces a nonzero status for every non-sealed run, including
  SIGINT and SIGTERM paths, after attempting the archive and teardown.
- The outcome map's partial-movement row now covers every guard-passing case
  with any EOS/generation/content movement that does not meet rows 3–5,
  removing the prior logical gap before the catch-all.

## Measurements

No model training or GPU measurement ran. Local preflight downloaded and
re-hashed all 18 runtime data/base files; every hash matched the registered
value. The exact base checkpoint exported to
`9a370566cd9da163191c0196763c68ddb395927fcf61fdecc1736b04941a4bd9`.
The 512-row grade reproduced fingerprint `b724656f…` and `132/512`.

The current overlap instrument reproduced 25,071 documents, 79,679 assistant
replies and zero findings in all four overlap categories. Its path-bearing JSON
hashed to `587a24c1…`, demonstrating why the controller checks its content
rather than the historical path-dependent file hash. The base per-turn teacher
ranks were 5/7/13 with top-8 198; one-user ranks were 18/138/873 with top-8 0.

The repair leaves the planned workload unchanged: one RTX 5090, 36–39 minutes
normally, with the registered one-retry projection at about $1.50. The launch
now enforces a 100 MiB seal cap and keeps the panel build outside that seal.

## Checks run at the head

- `gate.sh`: 148 checks passed.
- `test-mutants.sh`: 49 of 49 semantic mutations caught.
- `CARGO_TARGET_DIR=target cargo test --offline --locked -p uor-r4-float-train
  --lib`: 307 passed, including `checkpoint_ready_marker_records_completion`.
- The bare suite invocation omitted `CARGO_TARGET_DIR`; the existing save/load
  test requires an owned artifact directory and failed that precondition. The
  environment-qualified run above passed. The pod job exports the same target
  variable before building and testing.
- `bash -n` and ShellCheck on `job.sh` and `run-s1.sh`: passed.
- `git diff --check`: passed.

Not run: pod creation, remote build, model training or store write.

## Skills used

No system-provided Codex skill applied. The repository's loop, compute and
shell-controller rules were used directly.

## Still open

- Fresh FIT and D67 reviews of the exact merged repair, then the seed-1 run.
- `m2-reply512` remains unavailable unless exact bytes are recovered.

## Plain summary

The SFT controller now waits for the actual remote training job, rather than
mistaking job submission for completion.
