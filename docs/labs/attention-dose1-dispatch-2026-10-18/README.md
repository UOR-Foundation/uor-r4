# PM Dose 1 dispatch record — 18 October 2026

## Question and scope

Record the actual Project Manager dispatch of the seed-1 paired
`LearnV1`/`FrozenZeroV1` attention dose: its receipts, the packet and source it
binds, what it releases, and what it explicitly does not. Decision:
**DISPATCH_VALID, NOT_RUN**.

This is a record of one coordination act, not a result. No model,
allocation, release-build, plan-review, check-plan or learning outcome is
claimed here. No pod, SSH, build, model compute, training, evaluation or
scoring was started for this record; `$0` and no GPU work. The dose itself
remains **NOT_RUN**.

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
| Driver identity | `9538f5c3908dc7f3dacea13fb97ac081b5ab7a72fcc4deabc519bfc1ff8bd04c`, as bound by the PM receipt |
| Applicable FIT | [#2029 `6230317153`](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6230317153) |
| Owner resolutions | [#2029 `6236444507`](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6236444507) |
| Attempt label | `uor-cf1-dose1-ac60c495ec8d601b267319b8cf548910` |

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
  Casey's two personal live-TTY owner releases. PM does not create those
  records or invoke owner-release.

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

## Allocation and build state

At the base commit the packet carried allocation intent and private provider
logs only: `precreate-inventory.json` was `[]` and the operator log was 0
bytes. No confirmed allocation or release-build receipt existed.

A newer public Codex/cf1 operational review,
[#2029 `6245029696`](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6245029696),
was posted at 2026-10-18T03:07:37Z and verified with `gh` for this record. It
reports **`CONDITIONAL_APPROVE_EXISTING_POD_REVIEWED_POST_UP_TAIL_ONLY`**:

- an own pod `vast-56488188` evidenced by a created-pod marker, own native UP
  count 2 and a renewal, with canonical lease 0/1;
- original operator `37502` absent, outer log 0 bytes with no recorded exit or
  trap, host allocation absent, and only allocation-intent/fleet/UP work
  present;
- **no** local anchor, setup, build or model submission, and the historical
  launch cause **UNVERIFIED**;
- permission for root to complete only the exact unexecuted reviewed post-UP
  steps on the same pod, after fresh attempt/session/provider/lease/ledger
  checks, positive worker absence, remote absence checks and renewed resource
  admission. Never re-run the full script or `CREATE`, never overwrite
  partials, restart jobs or patch source.

Read-only host inspection for this record is consistent with that review: the
create diagnostics hold HTTP 200 and
`{"new_contract":56488188,"success":true}`; a retained renewal runs to
2026-10-18T03:34:58Z; a retained observation lists two RTX 5090 devices and
reports `OWNED_ROOT_PRESENT=0` and `PROJECT_SOURCE_PRESENT=0`. The create
response file also contains a provider instance key; it stays private and is
not copied into this record.

Therefore, precisely:

- **Provider-side pod creation:** evidenced (`new_contract` 56488188).
- **Operator allocation receipt:** **NOT_CONFIRMED** — the host allocation
  record is absent and the launch is under conditional continuation review.
- **Release build:** **NOT_CONFIRMED** — no ELF, source/kernel/geometry/config
  binding or build receipt exists in the reviewed evidence.
- **Model work:** **NOT_RUN**.

This record does not call the pod a confirmed admitted allocation, does not
infer a mechanism negative from the launch hazard, and claims no build or
learning result.

## Decision and next action

`DISPATCH_VALID` stands: the operator may continue the reviewed initial-dose
lease chain, subject to fresh admission and every release-build, per-arm
review, check-plan and Casey-TTY gate. The next decision inputs are the actual
post-allocation bindings above; until they exist, the correct PM state is
**dispatched, allocation receipt NOT_CONFIRMED, build NOT_CONFIRMED, dose
NOT_RUN**.

Fresh V2 content remains sealed pending typed `DEV H_geom` plus a PM decision.
No headline, milestone, progress count, training hold or mechanism status
changes in this record.

## Provenance

Base `35c4c40380ad25b2ba4cde0239b93110b26768db`; branch
`kimi/dose1-dispatch-record-2026-10-18`; worktree
`.worktrees/kimi-dose1-dispatch-record`; lab `kimi`; branch claim recorded in
`docs/harness/claims.json`. Record authored 18 October 2026, host-only.
