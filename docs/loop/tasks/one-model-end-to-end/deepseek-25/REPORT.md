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

## Measurements

No model or GPU measurement ran. The repair changes control flow only: job
submission remains one RTX 5090 under the reviewed 36–39 minute, $0.69–0.75
seed-1 envelope.

## Checks run at the head

- `gate.sh`: 85 checks passed.
- `test-mutants.sh`: 28 of 28 semantic mutations caught.
- Full `uor-r4-float-train` library suite: 307 passed, including
  `checkpoint_ready_marker_records_completion`.
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
