# Independent comparison review — #1094

## Source launch handoff — 2026-09-03

**`SOURCE_LAUNCH_HANDOFF_CONFIRMED`; result review pending.** The independent
`/root/retained_admission_audit` reviewer read the frozen preparation contract,
implemented handoff, actual bound coordinator CLI, sealed-directory admission
code and original exact release. This checks the one-shot launch arrangement;
it does not repeat the completed implementation/envelope approval or perform
fresh runtime/source-closure/asset verification outside the campaign clock.

The evidence worktree starts at protected
`1ff8b81cb060ca2e8ced409ec78ae566b5b98891`. Execution remains bound to the existing
coordinator source `07ec3f0d39d08ac5bf9c2ba7a6b864229e007867`, actual path
`/Users/casey.allard/.codex/worktrees/r4-retained-assembly/uor-r4`. The separate
new evidence worktree must not replace that path. The existing worker remains
at `/Users/casey.allard/.codex/worktrees/r4-runtime-readiness/uor-r4` under its
accepted source `79c674c8f6179a68878a12ee86e664f1435c3ebf`.

The original output is
`/Users/casey.allard/.codex/uor/issue-1094-retained-assembly01`.
Its unchanged assembly SHA256 is
`48fae2d391e347e89a290b12a8af97cf8266c5913a21e71f21c1bef74ef54c62`,
and the separately accepted release SHA256 is
`5787e4a64113800c5fc82cd1d32d564d9c6e3a344e74ca102a754fe82dccee23`.
The reviewer inspected those named metadata objects without repeating their
previous complete hash/source review. The already accepted release admits one
future comparison plus fresh-process replay; the current user activation
selects that one attempt.

### Required operator handoff

1. Preserve the exact original output and all existing receipts. The consumer
   refuses a prior admission/start/progress/stop/completion marker. Do not make
   a fresh output or repeat preparation to get another attempt.
2. Immediately before the single invocation, the operator changes only the
   original `/Users/casey.allard/.codex/uor/issue-1094-curator/withheld`
   directory from mode `000` to owner-read/execute mode `0500`. No recursive
   permission change, payload preview, copy, filtering or regeneration belongs
   in this handoff. `_sealed(require_sealed=False)` accepts `000` or `0500`,
   but leaving it at `000` would cause the admitted attempt to fail when the
   coordinator reaches withheld reading.
3. Invoke `campaign run-retained` exactly once. It owns both sequential model
   arms and the subsequent fresh-process replay; there is no second replay
   command or new preparation invocation. Its initial clock covers admission,
   fresh source/runtime/release checks and execution. It writes a durable
   admission marker before fresh checks and an execution-start receipt before
   the first withheld hash/read.
4. The coordinator, retained module and worker contain no chmod or reseal
   operation. The operator must restore the same withheld directory to `000`
   in an outer `finally` after the coordinator and its workers have exited,
   including nonzero/exceptional termination. Workers use separate process
   sessions; an external emergency termination must not assume killing only
   the coordinator's process group also kills a worker. Preserve partial
   evidence and report any exceptional cleanup rather than retrying.

The reviewed command arguments are below. The clean coordinator environment
selects the bound package and existing interpreter; the consumer independently
supplies the exact already-reviewed worker environment. Run from the existing
coordinator worktree, with operator permission/reseal handling around this one
invocation and output capture that does not overwrite a retained receipt:

```sh
/usr/bin/env -i \
  PATH=/usr/bin:/bin:/usr/sbin:/sbin \
  HOME=/Users/casey.allard \
  PYTHONPATH=/Users/casey.allard/.codex/worktrees/r4-retained-assembly/uor-r4/tools/r4-softmax-trainer/src \
  PYTHONNOUSERSITE=1 PYTHONDONTWRITEBYTECODE=1 PYTHONUNBUFFERED=1 \
  OMP_NUM_THREADS=4 VECLIB_MAXIMUM_THREADS=4 \
  /Users/casey.allard/.codex/worktrees/r4-language-r4/uor-r4/tools/r4-softmax-trainer/.venv/bin/python -B \
  -m r4_softmax_trainer.text_clause_adapter.campaign run-retained \
  --repo /Users/casey.allard/.codex/worktrees/r4-retained-assembly/uor-r4 \
  --corpus /Users/casey.allard/.codex/uor/issue-1094-curator \
  --output /Users/casey.allard/.codex/uor/issue-1094-retained-assembly01 \
  --python /Users/casey.allard/.codex/worktrees/r4-language-r4/uor-r4/tools/r4-softmax-trainer/.venv/bin/python \
  --assembly /Users/casey.allard/.codex/uor/issue-1094-retained-assembly01/retained-preparation.json \
  --review /Users/casey.allard/.codex/uor/issue-1094-retained-assembly01/release.json
```

The original 120-second preparation allocation remains a policy debit, with
3,465,401 historical bytes. Fresh execution and replay each have at most 120
seconds, within 360 seconds cumulative; the 3-GiB combined RSS, 128-MiB retained
byte and 6,400 logical-forward caps remain unchanged. Fresh identities must be
checked inside these clocks. Any admission marker consumes the envelope and a
`run-stopped.json` takes precedence over completion. No automatic retry, new
budget, input repair, policy widening or tolerance change is admitted.

No source-based launch blocker was found. This reviewer did not invoke the
command, hash runtime/model assets, traverse the source closure or withheld
directory, read a payload, launch a worker or change permissions. The eventual
independent result review must inspect the original terminal, execution/replay,
resource, identity, fidelity and completion/stop receipts. Until those exist,
model behavior is unmeasured. The original unavailable preparation, #1096
runtime-only readiness, #1079 weak control and #1082 descriptive limits remain
unchanged; this handoff supplies no new mathematical proof.
