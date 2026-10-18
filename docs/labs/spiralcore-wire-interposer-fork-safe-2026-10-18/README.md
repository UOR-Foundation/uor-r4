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

## Trade-off and measured cost

- A reader blocked with no data wakes every 50 ms while idle (20 wakeups/s per blocked routed fd).
- Each read is chunked with MSG_DONTWAIT; when data is already buffered no poll is needed; on EAGAIN
  the reader polls once (50 ms slice).
- macOS interposer smoke, 2 MiB text probe: interposed wall about 28 ms vs 19 ms before this hardening
  (about +9 ms per 2 MiB probe, including the fork/interrupt tests in the same binary). This is the
  cost of an interruptible, fork-safe read path; the earlier shutdown-based wake was cheaper but not
  fork-safe.
- A reader is woken by close between non-blocking chunks, not instantly inside a blocking recv.

## Limits

- Concurrent recv on one fd remains unsupported.
- Per-fd receive buffers are reset on the next connect rather than in close (bounded by MAX_FD).
- Fork safety means a child closing an inherited routed fd does not disturb the parent. If parent and
  child both keep using the same inherited routed fd, the per-process compression state makes the wire
  stream unsound; only the closer's process may treat the descriptor as closed.
- The fork test now includes a negative control: an unsafe build (WIRE_UNSAFE_CLOSE_SHUTDOWN) must fail
  the same test, proving the test is sensitive to the old shutdown-in-close behaviour.

## Next

Adversarial review, protected PR, merge, cleanup and hygiene. PR: https://github.com/UOR-Foundation/uor-r4/pull/2625
