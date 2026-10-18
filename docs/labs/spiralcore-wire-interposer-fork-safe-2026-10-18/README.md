# Interposer fork-safe close  2026-10-18

Closes the outstanding Codex P1 from the wire side-exploration review: `close()` called
`shutdown(SHUT_RDWR)` to wake a blocked reader, which disturbs a socket shared after `fork` (a child
closing an inherited routed descriptor kills the parent's connection).

## Change

- `wire_interposer.c`: receive now waits via a 50 ms `poll()` loop (`wait_readable`) that checks
  `state->active`; `recv` still serializes on `io_lock`, `close` only marks the fd inactive and calls
  `sys_close` (no `shutdown`), and a reused-fd `connect` waits on `io_lock` for at most one poll
  timeout before resetting state. No socket-wide shutdown is issued from close.
- `wire_interposer_fork_test.c`: new stock test binary. It opens a routed wire connection through the
  interposer, verifies a round trip, forks, closes the inherited descriptor in the child, then verifies a
  second (larger) round trip in the parent. The test fails on the old shutdown-in-close behaviour and
  passes with the fork-safe path.
- Both the macOS and Linux interposer smokes now build and run the fork test and report `fork_ok=true`.

## Verification (this worktree)

    WIRE_INTERPOSER_PASS        macOS, rc=0, fork_ok=true
    WIRE_INTERPOSER_LINUX_PASS  Linux mvm guest, rc=0, fork_ok=true

## Trade-off

The poll wait adds a bounded wake latency (<=50 ms) and one poll syscall before each frame read; the
alternative (socket shutdown) is not fork-safe. Throughput numbers are in the smoke output.

## Limits

- Concurrent recv on one fd remains unsupported.
- Per-fd receive buffers are reset on the next connect rather than in close (bounded by MAX_FD).

## Next

Adversarial review, protected PR, merge, cleanup and hygiene. PR: (added after creation).
