# Track B workspace restoration checkpoint

The owner's later reconnection review is recorded in the
[volume/path reconciliation](volume-reconciliation.md): current mount and
aliases resolve, other labs run from internal executable caches, and the
backing image's parent directory carries a quarantine attribute. This adds
execution-policy evidence without proving that ancestor caused the mount flag.

Lab 1's [notice on #820](https://github.com/UOR-Foundation/uor-r4/issues/820#issuecomment-5899196492)
records the owner's 21:19 UTC switch to a restored workspace image. Writes after
about 21:05 UTC may be missing or stale. Codex did not alter the storage image.

The retained build log and exit receipt bind source
`0f1f41e4801d992a098c29b5c8fd5d1572bd1592` and an empty tracked diff. The build
exited 101 with E0463 dependency errors after **22.97 seconds** (24 seconds in
the shell receipt). **No tests ran.** The error coincided with mount changes;
source correctness and restored-cache usability remain unverified.

A follow-up standalone core-verifier compile exposed the specific loader
failure: macOS rejected `libserde_derive-916883dc6a4602e7.dylib` with
`library load disallowed by system policy`. `codesign --verify --verbose=2`
reports that this dylib is valid on disk and satisfies its designated
requirement. The current APFS mount includes `quarantine`; neither the image
bundle nor this dylib has a `com.apple.quarantine` extended attribute. Ten
contemporary direct rlib/rmeta pairs are readable with matching embedded
metadata. The [diagnostic was sent to Lab 1](https://github.com/UOR-Foundation/uor-r4/issues/1515#issuecomment-5899377775).
No mount, code-signature or security-attribute changes were made by Codex.

The subsequent [metadata-only recheck](compiler-policy-recheck-3.log) still
rejects the same procedural macro with `library load disallowed by system policy`
(exit 1, 5.53 seconds, peak RSS 74,661,888 bytes). It used the same tiny
core-verifier source and existing rlib, wrote its proposed metadata path under
internal `/tmp`, and loaded no model. The compiler-access dependency was sent
to the [infrastructure steward](https://github.com/UOR-Foundation/uor-r4/issues/1514#issuecomment-5899632333).
The new shared model remains uncompiled; further model retries require normal
compiler access and the shared compute slot. Independent B2 draft engineering
and nine std-only accounting checks are preserved in the
[draft record](../track-b-b2-drafts-2026-09-29.md); they do not resolve this gate.

The log's trailing `BUILD_WALL_BOUND` is **not a real timeout**. The original
wrapper's cleanup killed a watchdog sleep, whose shell then continued to its
next statement. The closed `/usr/bin/time` record and exit receipt establish
the actual duration. The original log and wrapper are preserved unchanged.
The next attempt must stop the watchdog on interrupted sleep and clear the
completed child PID before cleanup.

The restored model weights, config, tokenizer and tokenizer-config SHA-256
identities match the pinned values in the conversion result. The shared ledger
retains all three Codex receipts and cumulative 785,486,862 / 1,130,000,000 ms.
No later Codex charge had been written before restoration. Both parity roots'
manifest identities, exact regular-file sets, all twelve BLAKE3 content hashes
and listed sizes match. Independent verification checked schema, root, duplicate
paths and the absence of extra files, symlinks or subdirectories. A 0.47-second
standalone std-plus-BLAKE3 reader was built internally from the existing pinned
library without procedural macros; it only reads and hashes files. This is
equivalent content/inventory verification, not a claimed rerun of the core
`report_output::verify` function, whose helper compilation was policy-blocked.
No root was changed or resealed.

After validating that ledger state, checkpoint 4 charged the complete elapsed
interval from epoch-ms 1790715650681 through 1790717645392: **1,994,711 ms**.
The fresh locked ledger became **787,481,573 / 1,130,000,000 ms**. This includes
recovery, static review, the failed 22.97-second build and delivery; model
inference and training during this interval are zero. Component times are not
added again.

The new source-data and transfer drafts were absent externally. They were
recovered from recorded tool patches into the internal worktree; provenance and
qualification limits are in the [draft README](../../../crates/uor-r4-training/drafts/README.md).
No corpus processing, fit or numerical parity result is claimed.

Recovery observed 450,475,996 KiB available on the restored workspace and
541,036,544 KiB on its backing drive. The existing lab cache is reused; no new
cache or deletion is needed for the next bounded attempt. The conservative
24 GiB plus 128 MiB guard and 256 MiB additional-build cap remain in place.

SHA-256 of preserved evidence:

- `build-shared-1.log`: `7c0768ab9553051fc18446743711d9aafe70a8584037af2389940b0e72393cb0`
- `build-shared-1-exit.json`: `afb0a90de05fc466a69f59e4923ac7c2f6cf68cd181724a32c1217100b70036b`
- `codex-track-b-build-shared-1.sh`: `658037788febfb5544115329134c6224b4d304e26dc029d6b0a01d6ed365e677`
