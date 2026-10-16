# Counterfactual decomposition execution packet — source stage

**NOT_RELEASED.** The [design](../../harness/prep/m1-counterfactual-decomposition-2026-10-16.md)
has [PM-seat TEST FITNESS: FIT (design)](https://github.com/UOR-Foundation/uor-r4/pull/2545#issuecomment-6219037898).
The [instrument commission](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6220832102)
permits source/host checks and exact-head compile-only CUDA gates. It permits
**no model computation, optimization, training or launch-variant execution**.
Training/M2/Stage B HELD; Q4 NOT_EXECUTABLE. This is not an owner-release record.

## Six binding FIT requirements

1. **Pair order:** each policy has one excluded warmup per mode, then three
   matched U/D pairs in order **U→D, D→U, U→D**. Within each mode, Route(0)
   precedes VectorBlock(0). Four cells have three timed pairs each: 24 timed +
   8 warmup proposals, 32 total. Each mode carries the identical window anew,
   with explicit carry/propose/observation full-unit sums and diagnostic wall.
2. **Denominator:** H2/H3 shares divide by **that lane's end−start cycles**,
   including setup/tail and uncovered clock/control overhead. They exclude the
   final terminal-record stores, which stay inside the enclosing CUDA event.
   Baseline numerator is baseline read + writer + eligibility; writer numerator
   is baseline + virtual writer. Nested writer phases partition their parent;
   they are never added to outer phases. Every raw lane and its uncovered
   cycles remain visible. Lane cycles are not summed into GPU elapsed seconds.
3. **Resource parity:** record runtime registers/local/shared bytes for U/D and
   independent ahead-of-time ptxas registers/stack/spill-load/spill-store bytes.
   The declared instrumentation budget is **U+16 registers** for runtime and
   ptxas, **U+1024 runtime local bytes**, identical shared bytes, and **no new
   or increased spill loads/stores**. Forty u64 words require 320 bytes; the
   local allowance covers their alignment/bookkeeping, not measured occupancy.
   No assertion that the sealed 21,536 local bytes means spills. Missing report
   or allowance failure is **UNRESOLVED / INSTRUMENT BLOCKER** before the dose,
   never a model negative; PM decides a repair, without on-lease tuning.
4. **Retained/stable:** all sixteen completed nonfaulted lanes in each of the
   three samples are required. No lane/sample subset is dropped. All 48 lane
   shares per cell must remain on the same side of each prospective threshold
   (H2 ≥20%, H3 ≥80%); any straddle is UNRESOLVED. Every lane's visited/eligible
   masks and early exits are stored; baseline eligibility must agree on every
   pairwise intersection of visited rows. E=max absolute paired D−U event time;
   S=max within-mode range. max(E,S) ≥20% of minimum U event is UNRESOLVED.
5. **Verbatim carried limitation:** no profiling-off control for the already-merged enclosing host/event instrument — E/S bounds only the incremental internal clocks.
6. **H1 magnitude:** NOT_APPROVED. The fixed grid=(1,1,1), block=(16,1,1)
   is unchanged in U/D. No parallel-launch variant, buffer pool, baseline cache,
   membership index or model mechanism change is part of this instrument.

## Identities and bridge

The ordinary `src/discrete.cu` is unchanged. U is its separately compiled
production module; D is a one-entry diagnostic module generated from the exact
helper/writer/counterfactual source with fail-closed anchors. The generator
inserts scoped integer clocks and counters, not a second handwritten model.
Both modules pass the integer-instruction/entry allowlist before inclusion.
The composite source identity includes the diagnostic generator, observation
and device code plus both PTX modules.

`decompose-identity` (CUDA build) emits compiled hashes **without creating a
CUDA context or executing a model**. Capture it with the gate packet, along
with generated source, ordinary/D PTX, ptxas resource logs and exact toolchain.
Fill [pin-template.json](pin-template.json) with those exact target hashes only
in the separately reviewed execution packet. Its pending placeholders are
intentionally refused; they grant no execution permission.

The new adapter pins original snapshot
`9a76743e401759b7d3aabc708ab0a26a9470ea474dc140f328c7b1aab59e49e4`,
original source `f6c4cc9f7a842ecb7a58fb569dcb7af61f84eb533cb4a059ee614be9c0f18fa7`,
original/ordinary PTX `1a21d6c3e2773a427c14ebff9499e03386147ab41b3ce413b7b5c9123925fd28`,
and the design's config/geometry. It requires an exact target composite-source
and diagnostic-PTX pair matching the executable. It admits seed1/epoch0/K16
Route only, changes **only** source identity in memory, hashes the unchanged
non-source payload and records both module identities. Original bytes are
never rewritten; training resume is false. The unchanged production loader
still separately admits the in-memory snapshot on device. Editing the old
bridge's pin or bypassing loader refusal is not a substitute.

## Prospective commands and gates

The existing admitted standalone manifest remains:
`docs/labs/k16-timing-2026-10-15/instrument/Cargo.toml`.

- Host `decompose-check|decompose-plan --pin FILE`: schema/dose plan only; no
  context or model. A valid plan is not an execution release or device admission.
- CUDA `decompose-identity`: compiled identity metadata only.
- CUDA `decompose --pin FILE`: the bounded matched diagnostic; **NOT_RUN and
  NOT_AUTHORIZED now**. It checks file/config/source/PTX pins, restores each
  policy before device admission, retains raw receipts on faults, compares all
  sixteen losses/status/five proposal words, and checks unchanged snapshots,
  epoch0 and original artifact bytes. It emits no throughput or training rate.

Before that last command: source review, compile-only check + strict-clippy at
exact head, protected merge/fresh-main verification, final reviewed executable/
source/U-D PTX/config/data pin packet, focused finite GPU parity controls
(ineligible/refusal/overflow/identity/tie/virtual endpoint/stale binding) and
explicit PM execution release/resource admission. These GPU controls remain
**NOT_RUN**; host receipt tests are not their substitute. No fixtures or
model calls piggyback on the compile-only lease.

The compile-only recipe is `scripts/pod/uor-cuda-check-remote` +
[compute.md](../compute.md#reproducible-fresh-pod-cuda-compile-gates-16-october).
Fresh Vast5090, one GPU, enforced ≥30 GiB disk preflight, fleet caps ≤4/≤$8/h,
EXIT trap before creation, exact pushed source, no on-lease source repair/retry.
Archive SHA256 + verified cloud MD5 round-trip **before** release/delete.
Record provisioning and provider uptime × actual quote, bytes, cumulative cost;
storage/transfer USD remain UNMEASURED. The diagnostic's ~20 min / ~$0.303
planning allowance is separate from this compile lease and grants no release.
