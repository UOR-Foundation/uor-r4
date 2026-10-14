# A2 seed-1 diagnostic pilot

Codex/Sol,2026-10-14; References #2029. This is the operational record for the
owner-delegated D33 release path and the registered1640-batch pilot, not a full
training run or a language result. M2 and all other holds remain HELD.

## Staging observation

Both requested Codex gates passed; see [the exact handoff](STAGING.md) and
[#2029 receipt6160885825](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6160885825).
Qualified pilot executable SHA256`d12877b6d8adc6a60445a014be682ab82459a781f760ac99a226b07335cbfcac`
returned the exact registered config and lineage digests on the restored
TRAIN/DEV/tokenizer files. Its metadata command created no CUDA context.

The separate D33 release tool is built from`ae9bfffc8` without the CUDA feature;
its SHA256 is`1dc78b6d0d58ad113730fe3c108e22501b47f87f30da653ea349a015741703bc`.
It is not the numerical pilot binary. Codex never invokes owner-release or
creates/copies the owner record; Claude's named delegation under D33 does not
extend to any other lab, configuration or hold.

Pod`vast-55864069`,1xRTX5090,$0.892/h, lease`codex/a2-pilot-20261014`;
working directory`/workspace/uor-r4/a2-stage`. Plan paths and bytes preserved;
only the parent of the run-state was created. The staged launcher is dormant
until Claude posts the delegated release receipt and the qualified binary
validates its identities. The entire pilot remains NOT_RUN at this staging
checkpoint; no release is inferred from build success or a FIT verdict.

Staging evidence is preserved at
`icloud:UOR-R4/results/codex/a2-pilot-staging-20261014.tar`,499,712bytes,
MD5`8016c93d0bd0fe0978109971534867db`, cloud-store download round-trip verified.
Nested staging-evidence.tgz SHA256
`0165dbb1174a425e5b16c3414cbec5e7cdabc98bf7cc1b98a78a4888bace3402`
matched the pod and local download. Corpus files remain in their pinned HF store;
this archive contains no owner-release record and no model output.

Next: Claude creates and posts the D33 release; Codex validates that receipt and
runs only the registered1640-batch seed1 pilot. The pilot's bars/guard remain
NOT_DECIDED, Route sensitivity NOT_RUN, fresh transfer UNAVAILABLE/NOT_RUN.
Final provider lifecycle and complete diagnostic outcomes remain pending.
