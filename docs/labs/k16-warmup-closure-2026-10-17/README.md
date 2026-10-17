# K16 completed-count hardening — 17 October 2026

**Source prepared; build/tests and model execution NOT_RUN.** This narrow change
makes the original K16 cell refuse rates and a successful cell summary unless
its completed timed-batch count exactly equals its pin. It retains the existing
partial-ledger fault path. It changes no model law, CUDA/kernel, checkpoint
bridge, generator, cell selection, warmup order, timing arithmetic or dose.

## Existing correction and current scope

PM commission6229239369 closes the warmup/count source line. The historical
99/100 allocation bug was already repaired by protected **#2539,
`bb87d56ec`**: `windows_for(pin)` allocates timed batches plus the excluded warmup,
and the actual loop skips that warmup. Thus a 100/1 pin already allocates101
windows and times100; a diagnostic3/1 pin allocates4 and times3. This delivery
adds a completion check and meaningful regression coverage; it is **not a fresh
repair of that arithmetic**.

In [main.rs](../k16-timing-2026-10-15/instrument/src/main.rs):

- A small shared `timed_windows` iterator contains the unchanged
  `enumerate().skip(warmups)` schedule and is used by the actual CUDA loop.
- After the loop and unchanged zero-apply assertion, and before any rate
  calculation or successful cell record, `validate_completed_batches` requires
  exact equality with `pin.batches`. Short or excess completion calls the
  existing `Ledger::fault("completed_batch_count", ...)`, flushes the partial
  receipts and returns an error. No cell success/rate summary is returned.
- Regression source consumes that same iterator with integer window labels.
  It checks the exact scheduled indices for100/1 and3/1, then exercises short
  and excess schedules for both doses. Mismatches must preserve all accumulated
  batch receipts in a partial record with the count fault and no rate fields.
  These are schedule/serialization metadata tests, not a CPU model oracle.

No schema or declared cell changes. The separate original/reuse fixed32 timing
path and frozen timing3 packet remain unchanged. This count guard belongs to
the original K16 `run_cell` path and its diagnostic dose.

## Checks and outstanding gate

Executed here: source/diff inspection, Rust1.97.1 formatting with
`skip_children=true`, formatting verification and whitespace checks only.
**Host tests, Cargo build/check/Clippy, CUDA compilation/Clippy and all runtime
work are NOT_RUN** because the parent reports available disk below the30GiB
floor. No target tree, provider/API/pod call or model/reference run was created.

After storage admission is restored, validation still needs the focused
schedule/count/partial-ledger tests and exact-head standalone compile/checks,
including the CUDA feature and Clippy for the changed CUDA-loop call site.
Formatting and source reasoning are not compile or test results. Any later
executable/source identity must be reviewed and pinned before execution.

**Separate source identity / 100-batch checkpoint bridge is BLOCKED**, awaiting
PM scope disposition6229280061. No bridge is added or changed here; this source
hardening does not make the original100-batch path executable. The existing
frozen timing3 packet is not edited, rebuilt, repinned or released by this work.

## Evidence scope

No new leases or GPU cost; no timing/model observation, apply, training,
checkpoint0 parity, p95, speedup, language or energy result. A count mismatch is
an instrument blocker, never a mechanism negative. Training/M2/Stage B **HELD**;
Q4 **NOT_EXECUTABLE**; seed2 excluded. Protected delivery and independent
exact-head review remain pending; the author does not self-approve.
