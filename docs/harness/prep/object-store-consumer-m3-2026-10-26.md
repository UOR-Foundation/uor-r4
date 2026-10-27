# Pre-Registration: M3 On-Model Durable-Memory Evaluation (`object-store-consumer-m3`) — 2026-10-26

- **Roadmap item:** `object-store-consumer-m3` (`#2873`, Milestone M3)
- **Lab:** `antigravity` (`antigravity-113`, Draft PR `#3323`)
- **Prior design note & owner bar registration:** [`#2873#issuecomment-6339008473`](https://github.com/UOR-Foundation/uor-r4/issues/2873#issuecomment-6339008473), [`#2028#issuecomment-6339008955`](https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6339008955), and owner decision [`#2028#issuecomment-6356034555`](https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6356034555) (2026-10-23T02:36:36Z, **"Yes, register all six (Recommended)"**).
- **Prior panel FIT:** [`#2873#issuecomment-6394411357`](https://github.com/UOR-Foundation/uor-r4/issues/2873#issuecomment-6394411357) (`TEST FITNESS: FIT`, `5822df5c-462f-417b-b6dd-56f3dd32660d`, 2026-10-25).

## 1. Question

On the two-seed M1-selected 19-table `UORLUT01` (`trunk-joint-pointer-v1-d11`, `uor-r4.lut-stack/3`) artifacts (`s1-trunk-joint-pointer-v1-d11.lut` and `s2-trunk-joint-pointer-v1-d11.lut`), does the object-store durable-memory consumer (`DurableObjectMemoryStore` + `Store` + `PinnedTokenizer` + `Ur4Sess1HeaderView` `UORMEM01`/`UR4SESS1` paired with GPU `DeviceScorer`) pass all six owner-registered M3 durable-memory bars (`provenance`, `correction`, `abstention` + R14, `non_memory_prose` `#1718`, `reset_isolation`, and `restart_restore` 32-token GPU continuation) and beat the paired `no-memory` negative controls (`L-010`) across two independent CUDA builds (`Build A` and `Build B`)?

## Base artifact

- **Selected M1 two-seed 19-table `UORLUT01` (`trunk-joint-pointer-v1-d11`, `uor-r4.lut-discrete-trunk/1`) artifacts** (`caseyallard/uor-r4-store:results/antigravity/m1-close-saved-model-2026-10-26/outputs/build-a/`):
  - **Seed 1 (`t16-frozen-zero-seed1-trunk.d11`):** `86,536,132` B, SHA-256 `04652a3de8e966a23539b04fbd77c7d9872d9f13a541fcb224d48e766e2c147d`.
  - **Seed 2 (`t16-frozen-zero-seed2-trunk.d11`):** `84,943,328` B, SHA-256 `1b49e8bd823a1021e31de55c767111f9cf43807d3e6d007644e49281935877de`.
- **Pinned tokenizer (`tokenizer.json`):** vocabulary `4,096`, SHA-256 `d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89`.

## Data split

- **Frozen M3 durable-memory panel (`docs/labs/object-store-consumer-m3/m3-durable-memory-panel-v1.jsonl`):**
  - `179,096` bytes, `384` canonical JSONL items (`6` sub-panels $\times$ `64` items).
  - Whole-object SHA-256: `582e38798e66432a79fec340a9f87d6f4cb1bfd8c3d2ab594f8e6e305694f9d1`.
  - Folded `Manifest::version()`: `88c27b94526c32b0f06fed7fabeb7a2abcf0b31a2f2534517b8656240eddce70`.
- **Gate R `ft/dev` parity split (`caseyallard/uor-r4-store:w2-inputs/2026-10-25/driver-data/`):**
  - `14,287` scored rows (`tokens.bin` SHA-256 `0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b`, `mask.bin` SHA-256 `a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078`).

## Steps

- **Step 0 (Gate R parity check on GPU):** Score all `14,287` `ft/dev` rows on the GPU via `m1_d11_artifact_scorer_instrument::device::DeviceScorer` before any M3 item is evaluated:
  - Seed 1 must reproduce exact `top1 = 7,002`, `top8 = 7,485`, `admitted = 9,012`, `refused = 0`.
  - Seed 2 must reproduce exact `top1 = 7,001`, `top8 = 7,478`, `admitted = 9,010`, `refused = 0`.
- **Step 1 (384-item M3 durable-memory evaluation per seed on GPU):**
  - `320` single-step GPU prefix evaluations across `provenance` (`64`), `correction` (`64`), `abstention` (`64`), `non_memory_prose` (`64`), and `reset_isolation` (`64`), plus `64` $\times$ (`32` uninterrupted + `32` post-restore) = `4,096` greedy decode steps on `restart_restore` (`4,416` total GPU model steps per seed, `8,832` across both seeds).
- **Step 2 (Two-build CUDA determinism check):**
  - Compile and execute `m3-durable-memory` under two independent CUDA builds (`Build A` and `Build B`) on 1x RTX 5090 (`UOR_NVCC_ARCH=compute_120`) and verify byte-identical `m3-summary.json`, `s1-m3-evaluation.json`, and `s2-m3-evaluation.json`.

## 2. Matched Arms

1. **Primary arm (`DurableConsumer + UORLUT01 + UR4SESS1`):**
   - Executes all `384` items through `DurableObjectMemoryStore` + `Store` + `PinnedTokenizer` (`d36d3e87…`) + `Ur4Sess1HeaderView` (`UORMEM01` + `UR4SESS1`) and GPU `DeviceScorer` on Seed 1 (`04652a3d…`) and Seed 2 (`1b49e8bd…`).
2. **Paired `no-memory` negative control (`L-010`):**
   - Runs every recall and `restart_restore` item against both an empty `DurableObjectMemoryStore` (`no_memory`) and unassisted model greedy decoding without the object store.
3. **Planted tamper & cross-artifact controls:**
   - Tests cross-artifact `UR4SESS1` session restore (`SEED1` snapshot restored onto `SEED2` and vice versa $\to$ `Err(Error::BindingMismatch)`), corrupt `UR4SESS1` magic (`Err(Error::SchemaMismatch)`), forged `UORMEM01` manifest SHA-256 (`Err(Error::IntegrityFailure)`), and cross-scope manifest import (`Err(Error::BindingMismatch)`).

## 3. Owner-Registered Acceptance Bars & Decision Table (`#2028#issuecomment-6356034555`)

| Sub-panel / Control | Metric | Registered Bar |
|---|---|---|
| Gate R (`ft/dev`) | Exact GPU reproduction on `14,287` rows | Seed 1 `7,386 / 8,854 / 12,175`, Seed 2 `7,374 / 8,849 / 12,143`, `refused = 0` |
| `provenance` (`64` items) | Exact value + verified occurrence span match rate | $\ge 0.90$ ($58/64$) on both seeds; `no-memory` $= 0/64$ (`L-010`) |
| `correction` (`64` items) | Current-version accuracy / prior-version lineage accuracy | $\ge 0.90$ current ($58/64$) and $\ge 0.85$ prior ($55/64$) on both seeds; `no-memory` $= 0/64$ |
| `abstention` (`128` queries across `64` items) | Refusal accuracy (`RefusedUnstated` / `RefusedRetracted`) with `0` backend fetches + R14 `AbstentionDiscrimination` | $\ge 0.95$ refusal ($122/128$), $\text{AnsweredFactHitRate} \ge 0.50$ ($\ge 5,000\text{ bps}$), $\text{SelectiveAbstentionGap} > 0$ |
| `non_memory_prose` (`64` items, `#1718`) | Stored fact retention after `4..=8` distractor prose turns | $1.00$ ($64/64$, zero silent overwrites) |
| `reset_isolation` (`64` items) | Post-reset refusal ($32/32$) and cross-scope isolation ($32/32$) with `0` backend fetches | $1.00$ ($64/64$, zero leaks) |
| `restart_restore` (`64` items) | `UORMEM01` + `UR4SESS1` `export_scope` $\to$ fresh `Store` $\to$ `import_scope_expecting` $\to$ `restore_verified_session` + `32`-token GPU greedy continuation | $1.00$ ($64/64$ items, $2,048/2,048$ tokens identical to uninterrupted continuation) |
| Planted tamper & cross-artifact controls | Fail-closed rejection on cross-artifact `UR4SESS1`, corrupt magic, forged manifest digest, and cross-scope import | `all_controls_pass == true` on both seeds |
| Two-build CUDA determinism | `Build A` vs `Build B` output digests | Byte-identical `m3-summary.json`, `s1-m3-evaluation.json`, and `s2-m3-evaluation.json` |

### Decision Table

- **`PASS_M3_DURABLE_MEMORY_TWO_SEEDS` (every bar above passes on both seeds and both builds):**
  - Closes `#2873` (`object-store-consumer-m3` $\to$ `status: done`), promotes `docs/mechanisms/object-store.md` from `READY` to `ACTIVE`, records the M3 milestone evidence in `docs/research/notebook/m3-durable-memory.md` and `docs/labs/object-store-consumer-m3/README.md`, and advances the loop to M4 (`m4-lawful-likelihood-bpb`, `#2861`).
- **`FAIL` on Gate R or any structural invariant (`restart_restore`, `non_memory_prose`, `reset_isolation`, or planted controls):**
  - Classify as `instrument` fault, halt immediately, repair and re-verify before calling any mechanism result.

## 4. Compute Budget & Stop Rules

- **Hardware:** 1x RTX 5090 pod via `scripts/pod/uor-pod` (`<= $1.50/hr`, well within the 4-pod / `$8.00/hr` project cap).
- **Projected wall time:** `< 5 minutes` total (two `nvcc` builds + Gate R + 2 seeds $\times$ 2 builds = `< $0.15` total spend).
- **Stop rule:** Abort immediately if any pinned SHA-256 digest or Gate R `ft/dev` count mismatches.
