# [codex] D28 evidence custody

## Sealed TRAIN and finite device checks

The finite deposit preserves the 68-prompt frozen TRAIN input and labels, the independently reproduced 226-record fit packet and binding, the confirmatory inputs and blind judgments, both planted device fixtures with their host oracles, the finite device readback, the original frozen bank, and the device executables, launch inputs and source snapshots used to interpret the finite results. The device fit and score outputs compare byte for byte with the independent planted host oracles.

| Storage | Locator | Readback |
| --- | --- | --- |
| Wick | `/home/callard/uor-codex/20261119-value-onset-28/custody-finite-v1` | All 78 archived files rehashed from the retained copy. |
| Private Hugging Face dataset `caseyallard/uor-r4-store` | Revision `e8f278ff26e29d9f1457adc7ee86e38261e050ea`, path `results/codex/2026-11-19-value-onset-28/finite-v1` | Downloaded the pinned delivery, decoded the original archive and rehashed every restored file. |

The delivery manifest SHA-256 is `165f894e6d191cb8282489432155f478c6f99b444f8749d53700fb935e9c7dfb`. The original-file manifest SHA-256 is `2f1d8afcee73d5afe56b32dfe82a2f68083dcb9228482f1944200dc7ac6f9cc7`. The compressed archive SHA-256 is `43072a648abb8b68ef1f4c4fdb7eeda1e07f933e16e46a7125930fe154648e7c`.

From a downloaded delivery directory, decode `finite-originals.base64.txt` with `openssl base64 -d -A` to `finite-originals.tar.zst`. Validate `files.sha256` and `finite-originals.tar.zst.sha256`, extract into a fresh directory with `zstd -dc finite-originals.tar.zst | tar -xf - -C RESTORE`, and check `original-files.sha256` from that directory.
