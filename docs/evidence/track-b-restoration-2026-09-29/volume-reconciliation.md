# Workspace reconnection reconciliation

Observed September 29, 2026, approximately 22:10–22:15 UTC, after the owner
requested a review of the reconnected volume and other labs' references.
This is a bounded read-only diagnosis, not a filesystem health certificate or
an acceptance report for any lab's model.

## Storage and paths

- `/Volumes/UOR-Workspace` is writable APFS on `disk6s1`, UUID
  `DAB4FDD9-1A7A-4ECB-A6D7-41D6D853F7CF`, matching the earlier restored copy.
- Its image is `/Volumes/X10 Pro/UOR-R4/UOR-Workspace.sparsebundle`, backed by
  ExFAT `disk4s2`, volume UUID `FAA849BB-9211-3E8D-A75D-DC3B45E087E6`.
  `hdiutil info` identifies the existing disk-image helper, PID 74072; its
  start time is 21:21:30 UTC. No duplicate suffixed workspace mount was found.
- The owner checkout's `.uor-models` is a directory containing 28 top-level
  symlinks into the current volume. All 28 resolve. Its ledger alias and the
  direct path identify device 16777242, inode 1299558, size 59.
- All Git-registered worktree directories exist. The active worktrees have
  local `.uor-models/corpora` skeletons, not the owner's entire model alias tree.
  Relative `.uor-models/sources` assumptions there would be incorrect. The
  inspected jobs and Codex's parity driver use existing absolute source paths.
- SmolLM2 config and tokenizer-config SHA-256 values still match the pinned
  `17f71c5442c79d4137c45bc79043f085c485bd69a26748080bcf2c11e0cc33a3` and
  `a27f638bd2831f5c3dea654a75838930f2b11fbe550c4d4e1d5d7bd07157b2ee`.
  Model/corpus payloads were not rehashed during other labs' live work.

## Compiler library diagnosis

The rejected `libserde_derive-916883dc6a4602e7.dylib` is a regular arm64 Mach-O
at the exact path in the rejection. No path component is a symlink. It is
4,368,016 bytes, UID 501/GID 20, mode 755, with only `/usr/lib/libSystem.B.dylib`
as a dynamic dependency. Its own install name uses the present cache path.

`codesign --verify --verbose=4` succeeds. The ad-hoc signature CDHash is
`da62b4a8febfad3222a1538463d374564f81f9a6`; file SHA-256 is
`2a07f5f2e8ec614a1749e8637e5cf56b3af97c36d91ffc3503e7e5928f8f2140`.
No pre-restoration file checksum was found for a byte-for-byte comparison.
Active rustc 1.97.1, commit `8bab26f4f68e0e26f0bb7960be334d5b520ea452`, matches
the cache metadata and the compiler string embedded in serde metadata.

The earlier and freshly repeated error is a library-load policy rejection,
not an ordinary missing-path or permission error. The mounted workspace has
the `quarantine` flag. A newly inspected ancestor, `/Volumes/X10 Pro/UOR-R4`,
has `com.apple.quarantine` value **`0081;00000000;;`**. The backing drive root,
image bundle, mounted volume root and rejected dylib do not have that xattr;
the image and dylib have provenance attributes. The ancestor is a plausible
source of the mount's provenance condition, not a demonstrated causal proof.
Apple DTS also recommends checking image/root quarantine metadata when a
signed executable on an image is rejected ([discussion](https://developer.apple.com/forums/thread/767612)).

Preserve the current cache: no examined defect justifies a cache clean,
compiler upgrade, signing change or rebuild. A repair must address the actual
execution-policy condition, with the owner/infrastructure steward coordinating
any shared-volume handling after active jobs release it. No security attribute,
mount, code signature or other lab's process was changed in this review.

## Other labs: live snapshots

| Lab | Observed process and executable location | Current volume use |
| --- | --- | --- |
| DeepSeek B0 | PID 13871, ignored exact-F32 G1a parity, executable and log in internal `/var/folders/.../T/opencode/` | Absolute model/token inputs on current workspace; log records checkpoint load and first Candle forward. No completion inferred. |
| Anti-Gravity B3 | PID 10183, `/tmp/uor-target/release/b3-e8-smollm2`, 32 layers/1 window/1024 tokens | Cwd `/Volumes/UOR-Workspace/Worktrees/d4-qat-adapters`; output `uor-r4-lab/b3-e8-smollm2-mlp/probe-1win-32layers` has matching claim, no completed report/manifest at inspection. |
| Kimi runner | No deployed runner process or matching LaunchAgent found | `/Volumes/UOR-Workspace/runner` absent; internal runner source exists with uncommitted/untracked work. This does not prove a deployed runner was lost. |
| Lab 1 cleanup | Delayed shell PID 97208 still armed | Matches the announced 23:35 UTC cleanup; no action taken on it. |

The earlier DeepSeek PID 9180 is absent. PID 13871 is a distinct, confirmed live
job; its PPID 1 is consistent with the inspected detached launch and is not a
failure signal. B3's stdout/stderr are its terminal, so its precise current arm
was unavailable. Its source times float and per-arm evaluation. Concurrent B0
work means this interval is not a quiet-machine throughput measurement.

The different executable/cache locations are consistent with these concurrent
observations, but do not identify the policy rule responsible for Codex's
rejection. Their activity does not establish that all external executable
loading works. The new findings
must be shared with the infrastructure steward before coordinated repair;
the fixed shared-model parity gate remains unchanged.

## Independent internal-build option

A later metadata audit identified a normal internal build path that does not
require modifying the shared volume's trust settings or copying its rejected
library. B0's internal release cache is approximately 0.62 GiB and B3's release
cache approximately 0.69 GiB; both bind the same rustc and Candle core/nn 0.9.2.
Neither contains the required Candle Transformers or Metal fingerprints, so
neither qualifies the unchanged gate. B3's full target includes about 4.77 GiB
of debug output and should not be duplicated for this purpose.

At the observation point, internal free space was 28,458,104 KiB (27.14 GiB).
A prospective private build can retain a 24 GiB plus 128 MiB free-space reserve
with at most 2 GiB new build output and 128 MiB temporary allowance. Refresh
physical free space and current jobs before admission. Prefer a fresh private
internal target from the pinned source; a private copy-on-write snapshot of
stable internal CPU artifacts is another possibility, but their active cache
was still growing and must not be snapshotted mid-build.

This is a feasible bounded build plan, not an executed workaround or a claim
that the external mount is repaired. No cache copy, new compile or model run
was launched during this reconciliation. Current lab timing work must finish
before the next build. The required Metal feature, exact tokenizer, windows,
all-vocabulary comparisons and 1e-4 tolerance remain unchanged.
