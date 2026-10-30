# Prep sheet: M1 bar (f), the held-out-phrasing panel (m1-heldout-phrasing-transfer, claude-211, 30 Oct 2026)

Lab: claude. Item: [m1-heldout-phrasing-transfer](../../roadmap/items/m1-heldout-phrasing-transfer.md). Order: [claude-211](../../loop/tasks/m1-heldout-phrasing-transfer/claude-211/ORDER.md). Casey, 30 Oct: "Go at M1 bar (f) now (Recommended)".

TEST FITNESS: FIT. PRE-RUN: PASS. From a fresh non-author review at `88d332b35698d2605550ca1a1d865f5b2f41ab8c`: no blocker or major finding. Its minor findings are about the job's greps and redundant checks, and the scorer's own pin checks already cover them. Lease: 3 pods at $1.68/h before this one.

## The question

Does the saved model that passed M1 bars (a) to (e) read prompts phrased in ways TRAIN never shows? That is bar (f) (D82, CHARTER §4 item 8). It needs "beating the T2 bigram and unigram on a fresh held-out-phrasing panel (drawn by heldout-panel at a new seed, with its own census), with the same paired band as bar (e)".

## Base artifact: the saved model

- **What is scored:** antigravity-110's two saved `UORLUT01` integer artifacts, the ones bar (e) passed on (`docs/labs/m1-fresh-panel-transfer/evidence/`):
  - seed 1: `t16-frozen-zero-seed1-trunk.d11`, sha256 `04652a3de8e966a23539b04fbd77c7d9872d9f13a541fcb224d48e766e2c147d`;
  - seed 2: `t16-frozen-zero-seed2-trunk.d11`, sha256 `1b49e8bd823a1021e31de55c767111f9cf43807d3e6d007644e49281935877de`.
- **Where they are:** both at `results/antigravity/m1-close-saved-model-2026-10-26/outputs/build-a/` in the HF store.
- **The gate:** the committed receipt (`trunk-joint-pointer-v1-d11-fresh-gate-receipt.json`, `PASS_BARS_A_TO_D_TWO_SEEDS`) binds both artifacts. `panel-score` refuses any other artifact.
- **The scorer:** `d11-scorer panel-score` (`docs/labs/m1-d11-artifact-scorer/instrument/`, `4add3a2ae`). It is `fresh-score`'s path, with the rows from bar (f)'s panel. On the GPU, it computes:
  - each artifact's integer top-1;
  - TRAIN's unigram and T2 tables, from the same 16,404 matched batches;
  - the CPU reference check and the planted-fault control B4.

## Data split

- **The panel, p-f:** `dev,dev`, seed 2051, 400 conversations, 15,953 reply targets. `tokens.u16` sha256 `e6abbdf0dd2f2fff1810d5e1fd95f6cfa20582c57c98e990a1d9c852e5e43fef`, `response_mask.u8` sha256 `c63c625a9897b4ee42043028addb3f9f831d7ecc28976cae8223ba0050e5c910`.
  - It is at `inputs/claude/heldout-panels-2026-10-30/` with a `SHA256SUMS`.
  - Its census is against the saved model's TRAIN. 0.000 of its prompts' value-masked templates occur in TRAIN at K = 1,024, and its `train,train` control at seed 2052 has 0.637 (`docs/labs/m1-heldout-phrasing-transfer/`). So it is new-template by claude-202's rule.
- **TRAIN** is `mw-balp/train`, the count tables' only source: `3dee858f84b750f507ceb3dc3f393711741950954735efaaf61bf704c017a4c8` / `b5e7352d67d6883a833155a5fd397a03f6d4f1b2649e3d4ec3e2ea7d95b949e3`.
- **DEV** is used only for the reproduction check: `0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b`.
- **What is never touched:** the sealed Fresh V2, P-dev, and every other panel. `refuse_reserved_panel` refuses them.

## Bar

At **each** of the two seeds, on all 15,953 rows, the saved model's integer top-1 must beat:
- **the unigram** by `max(24, ⌈2√(b+c)⌉, |M(1) − M(2)|)`, where `b` and `c` are the rows only the model gets and only the unigram gets;
- **T2** by the same band, with T2's own `b + c`.

Parity must hold: 0 refused rows, 0 reference mismatches, B4 passes. Both seeds passing is `PASS_BAR_F_HELDOUT_PHRASING`, and anything else is `FAIL_BAR_F_CLOSE_CONFIGURATION`.
- **Bar (e)'s copy-successor condition** is reported beside it, not required: D82 names the unigram and T2 only.
- **A fail is a negative for this saved configuration, not for the mechanism** (CHARTER §4.1). M1 then stays open for another saved model.

## Minimum test power

- **15,953 rows,** 80 times CHARTER §4.3's 200-row minimum and 1.1 times bar (e)'s 14,479.
- **The band** is at least 24 rows and two paired standard errors, `⌈2√(b+c)⌉`, and the two-seed spread.

## Steps

The job is `local/claude/bar-f-job/job.sh` in this unit's worktree, sha256 `c7cce15649dc32fbc608cf927c5b940b5fc6e6e322c37f49198f9fb9ad3ba10b`, with its configs and receipt shipped to `/workspace/barf-job/`. On 1 × RTX 5090:
1. **Source and tests.** Pin the source to this unit's head. Run the scorer's tests with `--features cuda --target x86_64-unknown-linux-gnu`; the job stops unless the bar (f) tests ran and all pass.
2. **Two builds,** A and B, as bar (e) did: `UOR_NVCC_ARCH=compute_120 cargo build --release --locked --manifest-path docs/labs/m1-d11-artifact-scorer/instrument/Cargo.toml --features cuda`, each with its sha256.
3. **The inputs, each checked:**
   - the artifacts against the receipt's sha256s;
   - TRAIN and DEV against their pins;
   - the panel against its `SHA256SUMS`;
   - the shipped receipt byte for byte against the repo's.
4. **The reproduction gate:** build A's `score` must reproduce the saved model's DEV top-1, 7,002 and 7,001 of 14,287. A miss stops the pod before the panel is read: the build does not reproduce the model, an instrument fault.
5. **Bar (f), once per build:** `d11-scorer panel-score ART1 ART2 panel-config.json decision-receipt.json report.json`. The two reports must be byte-identical.
6. **Tokens per second:** `d11-scorer toks` on each artifact, on this laptop, the serving device (CHARTER §3.6).
7. **The seal:** everything goes to `results/claude/m1-heldout-phrasing-transfer-2026-10-30/` with a `SHA256SUMS`, then the pod is released and taken down.

**Evidence only, not this bar:** one-model-end-to-end's (r-drop) is scored on p-f by its own unit's tool once its checkpoints are sealed. It enters no decision here.

## Stop rule

- **The pod stops** on a failed test, a pin mismatch, a missed reproduction, a parity fault, or a disagreement between builds A and B. Each is an instrument fault, repaired and never counted as a negative.
- **The panel is scored once** for this configuration, and its outcome is final for it (CHARTER §4.1, §4.3).

## Cost

- **Time:** bar (e) took under 2 minutes of scoring plus about 2 minutes of nvcc. Here there are two builds, the tests and the reproduction, so about 15 minutes in all.
- **Money:** 1 × RTX 5090 at about $0.56/h, so under $0.50. It is the fleet's fourth pod beside one-model-end-to-end's three, inside the caps (4 pods, $8/h).
