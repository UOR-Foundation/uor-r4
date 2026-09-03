# Independent retained-assembly review — #1094

2026-09-03. **`IMPLEMENTATION_ACCEPTED_FOR_METADATA_ASSEMBLY`.**
The independent `/root/retained_admission_audit` reviewer accepts the source
below for committed, metadata-only assembly of the retained evidence. This
decision does not issue `ACCEPTED_FOR_RETAINED_EVIDENCE_COMPARISON`. An actual
assembled envelope and its exact source/path/profile commitments still require
independent release review before withheld access or a model comparison.

## Exact implementation reviewed

The source base is protected `6008e3527cb119d12af4abd99fe86a3d4ebe5a53`.
The reviewer read the complete added retained module and campaign diff, then
independently hashed these actual files:

| File | Bytes | SHA256 |
|---|---:|---|
| `tools/r4-softmax-trainer/src/r4_softmax_trainer/text_clause_adapter/campaign.py` | 65,280 | `755d4d82286f97c2864e233d50b87dae40b08b05f244dce6c16346e2c0429bb4` |
| `tools/r4-softmax-trainer/src/r4_softmax_trainer/text_clause_adapter/retained.py` | 31,130 | `5bb4e1c4b78ed0af4ec25ce70bc2607ae4a16d4bf26e07aea43204a05d9847de` |
| `tools/r4-softmax-trainer/tests/test_retained_campaign.py` | 21,198 | `c178a08b3b2e81658489e8a026567190ce64aebb6c35ab3fd5fcfbfdec70fb7c` |
| `tools/r4-softmax-trainer/tests/test_retained_assembly.py` | 9,546 | `fc99be7818187c94b44e4fb8fe7d93998d489fc6da6085206d781e0685697522` |

The reviewer separately compared adapter, worker, curator, profile generator,
policy, learned reader model and R4 attention source against that base; they
are unchanged. The accepted adapter SHA256 remains
`21af8d2d43f9622a7a6e2d88c0c07e358b1bc8d519cc193b0574c8f6ced4fc27`,
worker `b8c0359edbfe63564bcd811351b06389c2b4384c73489d676ea7c46f4179458c`,
and curator `4d8012f8647cbbf57136999de6522af7b27bf994466744d0efac20e435a5defb`.
No changed comparison arithmetic or empirical criterion was found.

## Admission and accounting findings

The initial [source audit](r4_retained_assembly_1094_source_audit.md) supplied
the requirements and original-research boundary. Subsequent review found and
resolved concrete implementation defects before this acceptance:

1. An interruption during fresh source/runtime checks could leave the envelope
   apparently unused. The coordinator now writes exclusive
   `admission-started.json` after exact release matching and before fresh
   validation. Existing start, progress and terminal markers refuse reuse.
2. A post-worker identity-check failure could mask the original worker failure
   while trying to retain diagnostics. The handler now preserves the original
   cause, records `NOT_VERIFIED` when further checking is unavailable, and
   separately retains a diagnostic-write error where possible. A resource stop
   is not reported as a successful post-worker verification.
3. Checking a clean repository selected by `--repo` did not establish that its
   coordinator was actually executing. The retained module now checks its own
   source path; the campaign checks its loaded campaign, contract and adapter
   paths. The committed source closure must match actual bytes, including the
   complete path set, and the only permitted new package source is `retained.py`.
4. The root reviewer found that a named envelope or release file could be a
   hard link to a payload before approval. The final metadata reader checks
   regular-file type, unique link count and matching opened inode before reading
   bytes, with `O_NOFOLLOW` and `O_NONBLOCK`. The independent reviewer inspected
   this repair and its synthetic refusal assertion before the final acceptance.

The reviewed assembly module imports only the standard library and reads
historical receipts, committed source and public metadata. It opens no withheld
payload, runs no authoring rows and launches no runtime/model worker. It
reproduces the accepted #1096 profile from metadata before rebinding the exact
binding literal and narrowing the old readiness manifest/profile grants.
It retains the old worker source tree and exact binding bytes.

The execution consumer requires the distinct release schema/status and every
assembly, source, profile, environment, runtime and population commitment.
The release file must be the named `release.json` inside the exclusive output.
Unsafe output ancestry/aliases are rejected before output inventory. The
execution clock includes admission and fresh identity checks. The exclusive
execution-start receipt precedes the first withheld hash/read. Before and after
worker attempts, fresh checks bind the release, source, profile, eighteen runtime
files, both interpreter links, hardware and five accepted assets; the actual
worker retains its original single-sentinel/source/asset/runtime checks.

The historical 120 seconds are explicitly a conservative policy debit, not a
measurement. Every retained snapshot counts 3,465,401 old bytes once plus new
output, without walking the corpus. The replay budget carries execution time
and forwards while retaining both original time and byte debits. The original
120/120/360-second, 3-GiB, 128-MiB and 6,400-forward ceilings remain unchanged.
Stopped evidence takes precedence over completion; the new run mode writes
the same authoritative `run-stopped.json` name used by completion records.

## Bounded checks and limits

The reviewer inspected the named synthetic fixtures and the root agent's
direct [combined check transcript](r4_retained_assembly_1094_checks.txt),
3,990 bytes, SHA256
`85cfa49159f653e08fe8eecd716d31c80249359dada001df585da4c76597f91e`.
It records seven retained-module checks passing in 0.183 seconds and eleven
campaign checks passing in 0.034 seconds, both exit 0 at the file hashes above.
They exercise source mismatch, missing/extra source and
aliases; exact profile reconstruction and narrowing; clean environments;
retained-release field refusal; safe output and sealed metadata; historical
time/bytes/forwards into replay; independent resource ceilings; marker reuse
refusal; loaded-coordinator mismatch; start-before-payload ordering; and
before/after identity checks around in-memory worker stubs. The executing-source
mismatch cases fail before source/runtime or payload operations. The reviewer
did not repeat the runs: this acceptance uses the inspected direct execution
transcript plus independent source, fixture and file-hash review.

These checks concern coordinator admission and accounting. They are not
readiness observations or model behavior. The reviewer performed no runtime
hash/launch, model load/forward, preparation, withheld read/hash/traversal,
permission change, fit, comparison or replay. No broad QA was activated. The
source and profile controls are not a mathematical proof or a general defense
against hostile code or concurrent filesystem mutation. Terminal receipt
preservation still depends on successful filesystem I/O.

The original unavailable preparation, accepted #1096 runtime-only result,
#1079 weak control and #1082 descriptive limits remain unchanged. #1094 and
#973 remain open and #954 blocked. This review permits metadata assembly only;
the exact assembled envelope must receive its own independent release decision.
