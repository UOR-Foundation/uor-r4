# lab-runner

Detached local job runner for the UOR-R4 labs. It exists because agent
harnesses kill their children after a few minutes: a long training or
evaluation job must outlive the session that submitted it. A launchd
LaunchAgent keeps a foreground daemon alive (`KeepAlive`); the daemon services
a plain filesystem queue that every lab's CLI calls can use in seconds.

## Queue layout

Default root `/Volumes/UOR-Workspace/runner` (override with `--root` or
`UOR_RUNNER_ROOT`):

```
runner/
  queue/<id>/spec.json     validated, waiting for admission
  running/<id>/            admitted; pid, started_utc, started_ms, logs
  done/<id>/               finished; exit.json, logs, peak RSS
  daemon.out.log / daemon.err.log
```

## Job spec (`uor-r4.lab-runner-job/1`)

```json
{
  "schema": "uor-r4.lab-runner-job/1",
  "id": "keys-12",
  "lab": "lab/kimi/aerm",
  "cwd": "/path/to/worktree",
  "argv": ["cargo", "run", "--release", "--example", "aerm-keys", "--", "keys"],
  "env": {"CARGO_TARGET_DIR": "...", "RAYON_NUM_THREADS": "2"},
  "threads": 2,
  "rss_gib": 4.0,
  "gpu": false,
  "wall_s": 7200,
  "kill_criterion": {"kind": "wall", "value": ""},
  "exclusive": false
}
```

`wall_s` and `kill_criterion` are mandatory; a spec without them is rejected
with the missing field named. Kill kinds: `wall` (only the wall bound),
`log_match` (also kill when `value` appears in the job's logs as a literal
substring), `command` (also run `sh -c value` each monitor tick; a nonzero
exit kills the job).

## Admission control

Over the union of running jobs: total threads at most 8, total declared RSS
at most 11 GiB (a GPU job's `rss_gib` counts as GPU unified memory), at most
one GPU job, and `exclusive` jobs run alone (and block further admissions
until they finish). Every admitted job is killed at `wall_s`.

## CLI

```
lab-runner submit <spec.json>     validate + queue; prints the id
lab-runner status [id]            JSON summary (works with the daemon down)
lab-runner tail <id>              last ~50 lines of stdout/stderr
lab-runner cancel <id>            drop a queued job; kill a running one
lab-runner install-agent          write ~/Library/LaunchAgents/org.uor.lab-runner.plist
                                  and print the `launchctl bootstrap` command
lab-runner ledger rebuild         recompute model-time.json from records
lab-runner daemon [--poll-ms N] [--monitor-ms N] [--stop-file PATH]
```

Every CLI call returns in seconds. `submit` only writes the queue; the daemon
(launchd-managed) does the rest, so the job survives the submitting session.

## Completion records and ledger charging

On completion the daemon writes `exit.json`
(`uor-r4.lab-runner-exit/1`: outcome `completed|wall_killed|criterion_killed|
cancelled|error`, exit status, elapsed ms, peak RSS in KiB) under
`done/<id>/` and appends a ledger charge `charge-{utc}-{lab}-{id}.json`
(`uor-r4.model-time-charge/1`, canonical `charged_ms`) to the ledger
directory (default `~/.uor-models/native-joint-learning-2026-09-04/`,
override with `--ledger-dir`). Charges and rebuilds rewrite
`model-time.json` under an exclusive `File::lock` on `model-time.json.lock`
with a create-new-temp + atomic rename, so concurrent writers cannot lose
each other's records.

`ledger rebuild` folds the `charge-*.json` / `extension-*.json` records alone:
records are ordered by `recorded_utc`; the fold anchors at the last record
carrying an explicit cumulative anchor (a charge with
`base_read_cumulative_ms`, or an extension with `after.cumulative_ms`), adds
each later charge's additive `*_ms` fields and each later extension's
`increment_ms` to the limit. Without any anchor it refuses and says a genesis
record is needed. The crate's tests fold fixture copies of the live records
to the committed totals (782,538,181 of 1,130,000,000 ms as of
2026-09-29 20:05–20:08 UTC) and prove two concurrent charges both land.

## Concurrency note

`cancel` writes `cancel.json` into the job directory before signalling the
process tree; the daemon honors the marker, so the two finalizers always
converge on outcome `cancelled`. Finalization writes are idempotent: a
finalizer that loses the directory rename treats the rename's `NotFound` as
"the other finalizer completed first".

## Testing

`cargo test -p lab-runner` exercises only temporary queue roots, ledgers and
plist paths; nothing touches the real runner root, the real ledger, or
launchd. The live smoke (a >10-minute job outliving its session) is run
deliberately by an operator after `install-agent`, not by the test suite.
