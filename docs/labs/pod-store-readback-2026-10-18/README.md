# Pod store read-back before down  2026-10-18

Closes an evidence-loss hole in Vast pod custody: `uor-pod down --to-store` uploaded
each result directory with `/root/uor-store put` and then destroyed the instance without
ever reading the upload back. A silent partial upload would therefore destroy the only copy of
the results, while the ledger reported success.

## Change

- `scripts/pod/uor-pod` (`vast_archive_store`) now, for every uploaded directory:
  1. builds a sorted SHA-256 manifest of the pod-side files,
  2. uploads the manifest as the sidecar `vast-archive/<pod>/<dir>.manifest.sha256`,
  3. compares the store file count (`uor-store ls`) with the local count,
  4. reads the sidecar back and compares it byte-for-byte with the local manifest.
  On any mismatch it warns `READ-BACK FAILED ... pod retained` and returns nonzero, so `down`
  aborts before deleting the instance. The ledger `vast-archive` event now records
  `verified: true, files, manifest_sha256`.
- `scripts/pod/tests/uor-pod-dryrun.sh` stubs `uor-store ls/get/sha256sum` and adds two
  cases: an injected short listing must abort with the pod retained, and a normal upload must
  report `read-back verified N files` and a verified ledger event. Dry-run suite: 368 passed,
  0 failed.
- `docs/labs/compute.md` documents the read-back before destroy.

## Verification (host-only, no pods, no model compute)

    bash scripts/pod/tests/uor-pod-dryrun.sh   # 368 passed, 0 failed

The failure case uses `FAKE_STORE_TRUNC=1` to simulate a store listing one file short; the
resulting `down` aborts and the fake instance remains.

## Limits

- The read-back verifies file count + the manifest sidecar content, not every payload byte; the
  manifest binds hashes for a later restore check.
- It adds one small sidecar upload and several SSH round trips per result directory, and needs
  `/root/uor-store` on the pod (already required by `--to-store`).
- A stale extra store object under the same prefix makes the count check fail closed; an operator
  then inspects or chooses a fresh archive path. That is deliberate: retaining a pod is cheaper
  than destroying the only copy.
- `reap` and laptop-archive paths are unchanged.
