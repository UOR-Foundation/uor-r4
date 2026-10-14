# A2 seed-1 diagnostic pilot

Codex/Sol,2026-10-14; References #2029. This is the operational record for the
owner-delegated D33 release path and the registered1640-batch pilot, not a full
training run or a language result. M2 and all other holds remain HELD.

## Pilot launched

Claude executed the delegated release in
[#2029 receipt 6160973552](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6160973552),
and Kimi verified it in
[receipt 6160999031](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6160999031).
Codex independently checked the on-pod hashes and plan bindings, then started
the qualified executable under its GPU lease at **2026-10-14T04:38:17Z**.
Only seed 1, 1,640 batches and VectorBlock 0..1639 are released.

The initial checkpoint was written and the first observed progress sample had
40 completed batches, 14 accepted updates and 2,560 masked-target exposures.
The CUDA process was live on the RTX 5090. This is a RUNNING observation, not
a completed diagnostic result or a model-quality verdict.

Output: `/workspace/uor-r4/rung0-pilot-20261014/pilot-attempt-1`.
Job log: `/workspace/uor-r4/jobs/codex/20261014T043817Z-a2-pilot-20261014-gpu0.log`.
Admission SHA256:
`be866c7e646e469997dbeb9ffcf8dad92ed46c1f70fd8242f0982d6d47bc9391`.
Owner record SHA256 (read and hashed, not copied by Codex):
`6e6dea5ac3aa3a3a74c9c4d46955cbb4e9add67484dd747c1904393422904ab5`.

Next dependency is completed diagnostics and artifact preservation, owned by
Codex. No further release is requested from Claude or Kimi for this admitted
scope. Final provider lifecycle and diagnostic outcomes remain pending.

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

At the staging checkpoint, the next step was Claude's delegated release followed
by Codex's launch. That handoff is complete as recorded above. The pilot's bars/guard remain
NOT_DECIDED, Route sensitivity NOT_RUN, fresh transfer UNAVAILABLE/NOT_RUN.
Final provider lifecycle and complete diagnostic outcomes remain pending.
