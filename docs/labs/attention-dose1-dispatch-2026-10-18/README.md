# PM Dose 1 dispatch record — 18 October 2026

## Question and scope

Record the actual Project Manager dispatch of the seed-1 paired
`LearnV1`/`FrozenZeroV1` attention dose: its receipts, the packet and source it
binds, the later runtime allocation and release build it produced, and what it
explicitly does not release. Decision: **DISPATCHED; ALLOCATION CONFIRMED;
RELEASE METADATA BUILT; RUNTIME CUDA ADMISSION AND MODEL NOT_RUN**.

This is not a model result: no training, evaluation, scoring or runtime CUDA
qualification is claimed, and the dose remains **NOT_RUN**. No pod, SSH,
build or model compute was started for this record; `$0` and no GPU work. The
allocation and build facts below are read from the operator's receipts at
their exact hashes, not re-executed here.

## Exact authorities

| Authority | Identity |
|---|---|
| PM dispatch receipt | [#2029 `6244964455`](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6244964455), 2026-10-18T02:59:19Z |
| Activation reply | [#2029 `6244978507`](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6244978507) `DISPATCH_VALID`, 2026-10-18T03:01:09Z |
| `dispatch.json` | SHA256 `4f7681690f0ded838f8793a64117f3fdd96f2bf5464b2803730eb12f4fa543bb` |
| Static packet manifest | `REVIEW-SHA256SUMS` SHA256 `5d5481cd6c19513c316d1f7144257bcf623b0c660afb0fb9c34f9052c0fb7c44`, 234 members |
| AGY whole-packet verdict bytes | SHA256 `e3209a2ade698c34743a36ad6c1d6d59894b4211bd33ea2b423676b29d2066df` ([#2029 `6244893735`](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6244893735)) |
| Protected and compiled source | `23d5b146b46cf6479d27695a555e03e3cc98e972` |
| Scientific sheet | `docs/harness/prep/m1-learned-attention-link-2026-10-17.md`, SHA256 `844814009ecc7368c0a3ecabf62434bae7d72850363e19f67c0126810fee7ffc` |
| Release-build contract | `596a978d9f81e7404a858931596accb5cbbdf7695e196ab79ba27520a418a73f` |
| Admission glue (`admission.sh`) | `f0f5c06421c625651c3d483afab7312192cca659e2d6299a4625bcec6b4ab736` (prepare/finalize/collect) |
| Driver identity | `9538f5c3908dc7f3dacea13fb97ac081b5ab7a72fcc4deabc519bfc1ff8bd04c` — SHA256 over the concatenated `git show 23d5b146:crates/uor-r4-discrete/src/` bytes of `corpus.rs`, `training.rs`, `controller.rs`, `completion.rs`, `completion_baseline.rs`, `redirect_admission.rs`, `attention_learning.rs`, `attention_evidence.rs`, `attention_transfer.rs`, `attention_preflight.rs`, `main.rs`, recomputed read-only |
| Applicable FIT | [#2029 `6230317153`](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6230317153) |
| Owner resolutions | [#2029 `6236444507`](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6236444507) |
| Attempt label | `uor-cf1-dose1-ac60c495ec8d601b267319b8cf548910` |
| Runtime allocation | `runtime-allocation.json` SHA256 `1f1ed8dbf12004c5cf5adde82a84a9fefcaae7a73d34d66143232a2e80058515`: pod `vast-56488188`, two GPUs `[0,1]` |
| Release build receipt | `runtime-build.json` SHA256 `eae570b4c6c10504e72fbb89fdd7ed0f13c8906a1b5cc15688257251abe053b7`, `RELEASE_BUILT_METADATA_VERIFIED`, `build_exit` 0 |
| Runtime binary binding | `controller/runtime-binary-binding.json` SHA256 `2921dce26c686e7b48cda535b22b25929d7318bb1dfc5664f38fb0538fbafe60` |
| Per-arm plan candidates | treatment `64869df7d5fde9eaf23162e3046e1fc5cbd9aaaff4d7533cd4274c2d7510113f`; frozen-zero `b52933ecf2f0291877d85b568bb1d6961e6d6d405c0c413eadbe9bc53a64ec7a` (`uor-r4.attention-learning-owner-plan/2`) |

`dispatch.json` is the operator's private local file at
`/Users/casey.allard/uor-r4-local/codex-attention-dose1-operator/dispatch.json`;
it is not committed here. Its `pm_receipt_url` points at the receipt above and
its `test_fitness_reference` at the FIT comment.

Read-only checks run on 18 October for this record:

- `jq -e . dispatch.json` → parses; schema `uor-r4.attention-dose1-dispatch/1`,
  status `PM_DISPATCH_VALID`, seed 1, `max_batches_per_arm` 16404, layout
  `one_pod_two_gpus`, attempt label as above.
- `shasum -a 256 dispatch.json` → `4f768169…543bb`, matching `DISPATCH_VALID`.
- `shasum -a 256 -c REVIEW-SHA256SUMS` → all 234 members `OK`.
- `readiness-verdict.md` → `e3209a2a…066df`, matching the dispatch and the AGY
  verdict bytes.
- `release-binding/release-build-contract.json` → `596a978d…8a73f`, and
  `jq` equality of `dispatch.release_build` with that contract → `true`.
- `admission/admission.sh` → `f0f5c064…ab736`.
- Sheet hash at the base commit → `84481400…fee7ffc`.
- `git cat-file -t 23d5b146` → commit; `git diff --stat
  23d5b146..35c4c4038 -- crates/uor-r4-discrete` → empty, so the discrete
  crate is unchanged from the protected pin to the base.
- Driver provenance: the exact `for f in corpus.rs training.rs controller.rs
  completion.rs completion_baseline.rs redirect_admission.rs
  attention_learning.rs attention_evidence.rs attention_transfer.rs
  attention_preflight.rs main.rs; do git show 23d5b146:crates/uor-r4-discrete/src/$f;
  done | shasum -a 256` → `9538f5c3…bd04c`, matching the build receipt's
  `driver_source_sha256`.
- `runtime-allocation.json` → `1f1ed8db…58515`; its `dispatch_sha256` equals
  the dispatch above, pod `vast-56488188`, count 2, GPUs `[0,1]`.
- `runtime-build.json` → `eae570b4…053b7`, matching the binding's
  `runtime_build.sha256`; the on-host `bin/discrete-stack` → `1f7dd526…dbb3`,
  matching the receipt's ELF.
- `controller/runtime-binary-binding.json` → `2921dce2…afe60`; its
  `construction_manifest.sha256` equals the build directory's `SHA256SUMS`
  (`e2527d24…c5777`).
- `shasum -a 256 -c SHA256SUMS` inside the build directory → every member
  `OK`; cross-receipt equality of `dispatch_sha256` among the allocation,
  build and binding → `true`, and `build.allocation_sha256` equals the raw
  allocation file hash.

The PM activation reply records that `lifecycle/dose-common.sh::dose_authority`
and the dependency pins also pass. That function writes
`lifecycle/latest-packet-check.private.log` inside the operator packet, so this
record did not re-invoke it; it re-ran the function's read-only predicates
above instead and did not mutate the packet.

## What the dispatch releases

- The **initial dose only**: seed 1, two arms at 16,404 batches per arm, one new
  Vast pod with two RTX 5090s, one arm per physical GPU, attempt
  `uor-cf1-dose1-ac60c495ec8d601b267319b8cf548910`.
- Required construction: `cargo +1.97.1 build --release --locked -p
  uor-r4-discrete --features cuda`, with actual post-allocation ELF, source,
  kernel, geometry, config and lineage bindings. No debug or pre-allocation
  binary promotion.
- Fresh native fleet/rate/disk/source admission before `CREATE`, within the
  standing caps (`≤4` pods, `≤$8/h`) and the reviewed reserves plus the 30 GiB
  host floor. The receipt states that the local pod status seen at dispatch
  time was not fleet proof.
- These gates still precede model work, in order: the same-dispatch release
  build; two no-context build identities; actual per-arm `/2` plans with
  genuine non-Codex admission-diff reviews; `attention-check-plan`; then
  Casey's two personal live-TTY owner releases. The build and identities are
  recorded below and the plans are prepared and under AGY review; the later
  gates remain pending. PM does not create those records or invoke
  owner-release.

## What is not released

- No operational retry. The prospective `≤2`-retries/24 h ceiling requires a
  separate actual PM grant after external full-context review; there is no
  automatic `CREATE` and no replay.
- No second seed, no optimization rung and no fresh-panel scoring. The
  pre-registered tokens/s rung stays gated on the dose outcome.
- No model work of any kind before the gates above. Every failure is a blocker
  for review, not a mechanism negative.

## Fresh V2 sealed input metadata

Accepted as **metadata only**, at the exact values bound in the receipt: 400
documents, 44,678 payload tokens, 14,479 eligible response targets, 3,594
copy-successor rows and zero exact full `TRAIN`/`DEV` document overlap. Raw
tokens `0b7e754cfd242bf53145fe9cbca563be8503e6cd75df5449abb325b5c51ee1b8`,
response mask `a1cda5295547bb641b1ea71ba4ebabb58d58c40e6c048cda91f35a878fecbf5e`,
ordered private row binding `c2d4c7e512379650ae3d72703c1affc9a0e85d754da7b73703dc5aad2cee6c06`
and seal report `df5d074ae09c743aa586e3463ffe87faca5703f44f7802bff8ac49ee2990395f`.

Content and labels stay sealed and unused until the typed paired `DEV H_geom`
decision and a separate PM decision. No fresh capture, ablation, selection,
redraw or automatic KEEP is released. The conditional paired fresh remains 227
batches per arm (454 paired) and is **NOT_RUN**.

## Allocation, release build and admission state

At the base commit the packet carried allocation intent and private provider
logs only: `precreate-inventory.json` was `[]` and the operator log was 0
bytes. The sequence after the dispatch is now preserved in three receipts.

### Allocation — confirmed

`runtime-allocation.json` (schema `uor-r4.attention-dose1-allocation/1`, SHA256
`1f1ed8dbf12004c5cf5adde82a84a9fefcaae7a73d34d66143232a2e80058515`)
binds lab `codex`, session `codex-attention-dose1`, pod `vast-56488188`, the
attempt label above, `dispatch_sha256` `4f768169…543bb`, count 2 and GPUs
`[0,1]`. Its `authority_issued` is `false`: the allocation is confirmed, but it
is not an owner release.

### Release build — metadata verified

`runtime-build.json` (SHA256
`eae570b4c6c10504e72fbb89fdd7ed0f13c8906a1b5cc15688257251abe053b7`, under
`metadata-uor-cf1-dose1-ac60c495ec8d601b267319b8cf548910/construction/release-build/`,
directory manifest `e2527d24…c5777`) records
`RELEASE_BUILT_METADATA_VERIFIED` with `build_exit` 0 and the exact required
argv `cargo +1.97.1 build --release --locked -p uor-r4-discrete --features
cuda` at `UOR_NVCC_ARCH=compute_120`.

| Build identity | Value |
|---|---|
| Shared ELF (both arms) | `1f7dd5268b6e7b69c3a22a2c7a4905d5484cdedb720ee89ceb541b18d8b2dbb3`, ELF64_X86_64; the on-host `bin/discrete-stack` matches |
| Protected source commit | `23d5b146b46cf6479d27695a555e03e3cc98e972` |
| Compiled source | `64e484637425f009969c6ec270ccbae6ab7ca50e89fc244240058231d00d9dad` |
| Kernel | `5cd0e73241fd895f2dd0004c83af3d1eb9f89e00ec2e933c1acc44fc7127bfd6` |
| Geometry | `46ddbb76b7fbe6ac6222ea24779fc10055b1d6cea6a8da786895676fd29fe327` |
| Driver | `9538f5c3…bd04c` (composite above) |
| Treatment config / lineage | `c446aa7f…9f216` / `ffad0e1d…ba2e1` |
| Frozen-zero config / lineage | `1435f016…071c3` / `f859bf22…7a9ae` |

`cuda_context_created` is `false` for both arms, `metadata_identity_calls` is
2, `model_commands` and `init_commands` are 0, and `training` is `HELD`. The
receipt's own `validation_scope` is "actual release build and no-context
metadata; no model/runtime CUDA qualification".

`controller/runtime-binary-binding.json` (SHA256
`2921dce26c686e7b48cda535b22b25929d7318bb1dfc5664f38fb0538fbafe60`) repeats
the ELF/source/kernel/geometry/config/lineage identities and records
`cuda_runtime_resource_admission: NOT_RUN`,
`live_resource_admission: NOT_PERFORMED_BY_HOST_BINDER`,
`model_computation: false` and
`typed_config_owner_admission: EXISTING_PROTECTED_SOURCE_REQUIRED`, with
`authority_issued: false`.

**Distinction:** the RELEASE construction and its metadata identities are
built and verified. That is not runtime CUDA resource admission, not an owner
release, and not training. No CUDA context was created and no model command
ran.

### Per-arm plans — prepared, review pending

Both `/2` plan candidates exist as metadata: treatment raw
`64869df7d5fde9eaf23162e3046e1fc5cbd9aaaff4d7533cd4274c2d7510113f` and
frozen-zero raw
`b52933ecf2f0291877d85b568bb1d6961e6d6d405c0c413eadbe9bc53a64ec7a`, scoped
`batches0..16404` with the exact ten-key `ExecutionIdentity` (base
`ef9d3099…`, reviewed source `23d5b146`, sheet `84481400…`, source/kernel/
geometry/executable/driver/config/lineage as above). Their
`.review.candidate.json` records remain `NOT_REVIEWED` / `UNASSIGNED`, and the
candidates are `HELD_OWNER_ACTION_REQUIRED` pending that review.

Codex handed both to AGY for the genuine non-Codex per-arm admission-diff
review in
[#2029 `6245084232`](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6245084232)
(2026-10-18T03:14:32Z, "no new dispatch requested"). Plan finalization, the
actual `attention-check-plan` with no CUDA context, and Casey's two personal
TTY releases remain pending.

### Preserved launch state

The earlier
[#2029 `6245029696`](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6245029696)
review (03:07:37Z) remains the record of the launch hazard:
`CONDITIONAL_APPROVE_EXISTING_POD_REVIEWED_POST_UP_TAIL_ONLY`, original
operator `37502` absent, 0-byte outer log with no recorded exit, no setup or
build at that point, and the historical cause **UNVERIFIED**. The later
receipts show the reviewed post-UP continuation completed setup0, five data
transfers with SHA verification, and the native release-build job terminal at
03:11:34Z with exit 0 — with no `CREATE`, rebuild, retry or source patch. The
unverified launch cause is not a mechanism negative, and the create-response
file's provider instance key stays private and is not copied into this record.

## Decision and next action

`DISPATCH_VALID` stands. As of 2026-10-18T03:14:32Z the correct PM state is
**dispatched; allocation confirmed at `vast-56488188`; RELEASE build metadata
verified; runtime CUDA admission NOT_RUN; model work NOT_RUN**.

The next decision inputs are AGY's genuine per-arm admission-diff verdicts and
six-key `AttentionReview` records, then plan finalization, the actual
`attention-check-plan` with no CUDA context, and Casey's two personal TTY
releases. Until those exist, no model command, training step or dose result may
be claimed, and the metadata-verified build is not runtime qualification. Any
failure returns to PM review as a blocker, not a mechanism negative.

Fresh V2 content remains sealed pending typed `DEV H_geom` plus a PM decision.
No headline, milestone, progress count, training hold or mechanism status
changes in this record.

## Provenance

Base `35c4c40380ad25b2ba4cde0239b93110b26768db`; branch
`kimi/dose1-dispatch-record-2026-10-18`; worktree
`.worktrees/kimi-dose1-dispatch-record`; lab `kimi`; branch claim recorded in
`docs/harness/claims.json`. Record authored and revised 18 October 2026,
host-only; the revision adds the runtime allocation, release-build and
binary-binding receipts and the driver composite provenance.
