# D-R Qualification Runbook — `C-R1..C-R6`, `G-Pre` (`K=16` Checkpoint-0 Ceiling), and 100-Batch `K=16` Synchronized Timing (`NOT_TRAINING`, 15 October 2026)

- **Lab:** `antigravity`
- **Date:** `2026-10-15`
- **Line:** `starling read / exact addressed memory` — `count 1/3` (unchanged; runbook specification only)
- **M1 headline:** `reply acceptable 37/232 -> target 116/232` (unchanged; suspended under D28/R14)
- **Execution scope:** **`NOT_TRAINING` — source/documentation runbook only (`0` GPU hours, `0` pods, `0` model executions in this PR)**. When the owner releases a bounded qualification lease, this runbook governs **finite GPU fixture qualification (`C-R1..C-R6`), zero-update `K=16` checkpoint-0 ceiling evaluation (`G-Pre`), and zero-update 100-batch `K=16` synchronized counterfactual timing** with **zero `.apply` updates on any corpus row**.
- **Governing authorities:**
  - Pre-registration sheet (`TEST FITNESS: FIT (design)` at `8a4388424`, merged in [#2515](https://github.com/UOR-Foundation/uor-r4/pull/2515) as `5188f2e38`): [`docs/harness/prep/m1-discrete-rung0-admission-redirect-2026-10-15.md`](../../harness/prep/m1-discrete-rung0-admission-redirect-2026-10-15.md)
  - `D-R` source slice (merged in [#2518](https://github.com/UOR-Foundation/uor-r4/pull/2518) as `2fa663b54`): [`docs/labs/admission-rank-hinge-2026-10-15/README.md`](../admission-rank-hinge-2026-10-15/README.md), [`crates/uor-r4-discrete/src/objective.rs`](../../../crates/uor-r4-discrete/src/objective.rs), [`crates/uor-r4-discrete/src/discrete.cu`](../../../crates/uor-r4-discrete/src/discrete.cu), [`crates/uor-r4-discrete/src/device.rs`](../../../crates/uor-r4-discrete/src/device.rs), [`crates/uor-r4-discrete/src/training.rs`](../../../crates/uor-r4-discrete/src/training.rs), [`crates/uor-r4-discrete/src/main.rs`](../../../crates/uor-r4-discrete/src/main.rs)
  - Owner `G-Pre` confirmation: [#2515 comment 6198442894](https://github.com/UOR-Foundation/uor-r4/pull/2515#issuecomment-6198442894)
  - Compute & storage protocol: [`docs/labs/compute.md`](../compute.md), [`scripts/pod/uor-pod`](../../../scripts/pod/uor-pod), `~/.local/share/uor-r4/bin/cloud-store`
- **Hold status:** Global training hold, M2 training/rental hold, and Rung-0 Stage B remain **HELD**. Completing this qualification runbook on a released qualification lease satisfies `G-Code` (`C-R1..C-R6`), `G-Pre`, and the `D-Packet` measured-throughput slot, after which training still awaits the owner's separate personal hold release (`R8`).

---

## 0. Pinned Source, Corpus, and Checkpoint Identities at `2fa663b54`

Before running any GPU command on the qualification pod, the executor must verify the exact source, geometry, and corpus SHA-256 digests below:

| Artifact / Pin | Path / Function | Expected Digest (`SHA-256` / `BLAKE3`) |
|---|---|---|
| `driver_source_sha256()` | `crates/uor-r4-discrete/src/training.rs:145-169` (11 source files) | `3487506d0d6b9197e8b99963dfe92e91b5c3e33e0801d9116bf525cdc9b17fb7` |
| `device::source_sha256()` | `crates/uor-r4-discrete/src/device.rs:648-657` (`discrete.cu`) | `f679f8736500925014ff721ab035f3c284446baa6af0763f27b09bc388763008` |
| `objective.rs` | `crates/uor-r4-discrete/src/objective.rs` | `b813e855be6c4ea460c8711493527d52f1caf1a02bb09b046145f88c7f4ee4d0` |
| `geometry::GEOMETRY_SHA256` | `build.rs` generated `geometry.rs` | `46ddbb76b7fbe6ac6222ea24779fc10055b1d6cea6a8da786895676fd29fe327` |
| `OPERATOR_KAPPA` | `crates/uor-r4-discrete/src/lib.rs:20` | `blake3:a0ea3a10dd10c5f4856b0d881d33d94c3293bad0e68bd07a47918e2bb82c819b` |
| `COMPOSITION_KAPPA` | `crates/uor-r4-discrete/src/lib.rs:22` | `blake3:f2986c8e68dcb30a9cb511a42547179a87b66ef212a87b728765d03e70e640b0` |
| `train/tokens.u16` | `artifacts/corpora/fineweb-edu-v4096-100m-v2/train/tokens.u16` | `8c488f50e0e1ca67a29937be02d4eb751b096757d767710a6ab4fa737df0b86c` |
| `ft/train/tokens.u16` | `artifacts/corpora/fineweb-edu-v4096-100m-v2/ft/train/tokens.u16` | `de8cc71e30dd2f0dd051660870fb2512ed95462335eb7fa18b62796a6aa7484f` |
| `ft/train/response_mask.u8` | `artifacts/corpora/fineweb-edu-v4096-100m-v2/ft/train/response_mask.u8` | `e53fdcfde4a1b983b15b65ac3e24b13f8e8c8c5e3a4be73594a16867f1ff0092` |
| `ft/dev/tokens.u16` | `artifacts/corpora/fineweb-edu-v4096-100m-v2/ft/dev/tokens.u16` | `0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b` |
| `ft/dev/response_mask.u8` | `artifacts/corpora/fineweb-edu-v4096-100m-v2/ft/dev/response_mask.u8` | `a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078` |
| `tokenizer.json` | `artifacts/corpora/fineweb-edu-v4096-100m-v2/tokenizer.json` | `20cd9d96f9d2fe2c1e6e62ff7c5fc4cb810fe57279313fe0660b96597f2685fd` |
| `checkpoint-16404.json` (Q4 occupancy fixture) | `icloud:UOR-R4/results/codex/20261015T060354Z-rung0-stage-a-pilot-seed1.tar` | `4477eb5c2480b0b85cc94036a2e15b0d22b654e25deee9c8717aa6e30f8ef7bf` (`model_snapshot_sha256 = 29ba5517a6b1b6d3c7ce54d593221560b446c5048a0b0db882ad53b5ec75bbbf`, `189` occupied cells) |

---

## 1. `C-R1..C-R6` Fixture Execution on GPU (Analytic/Stored Expected Values & Pass/Fail Criteria, Zero CPU Model Reference)

### 1.1 Policy & Kernel Invariants (D24 / D32)

1. **Zero CPU model reference:** No host function may compute token distance, address distance, cell rank, candidate loss, or row aggregation as a runtime fallback or reference oracle. Every fixture executes through the 11 admitted CUDA `__global__` kernels in [`crates/uor-r4-discrete/src/discrete.cu`](../../../crates/uor-r4-discrete/src/discrete.cu) (`discrete_init`, `discrete_encode`, `discrete_forward`, `discrete_counterfactual`, `discrete_choose`, `discrete_apply`, `discrete_validate`, `discrete_evaluate`, `discrete_evaluation_aggregate`, `discrete_count`, `discrete_count_score`) and checks the GPU output against **hand-derived analytic integer identities or sealed stored values**.
2. **PTX admission gate (`admit_ptx`, `build.rs:25-83`):** At build time under `UOR_NVCC_ARCH=compute_120` (`nvcc V12.8.93`), `admit_ptx` verifies that the compiled PTX defines all 11 kernels and contains zero prohibited tokens (`".f16"`, `".f32"`, `".f64"`, `".bf16"`, `".tf32"`, `"cvt.f"`, `"cvt.rn.f"`, `"cvt.rz.f"`, `"cvt.rm.f"`, `"cvt.rp.f"`, `"ex2."`, `"lg2."`, `"rcp."`, `"sqrt."`, `"rsqrt."`, `"sin."`, `"cos."`, `"tanh"`, `"mma."`, `"wgmma"`).
3. **Zero regression on existing finite suites:** Before running `C-R1..C-R6`, the executor runs the four existing finite GPU suites on the same binary:
   - `discrete-stack qualify` (`11/11` `C1..C11` controls, `0` corpus `.apply` calls),
   - `discrete-stack qualify-flock` (`6/6` `F1..F6` controls),
   - `discrete-stack qualify-relation` (`8/8` `C1..C8` / `10` checks),
   - `discrete-stack qualify-driver` (`8/8` `C-D1..C-D8` controls, capped at the `11` disposable synthetic fixture `.apply` calls enforced by `lib.rs:212-227`).

### 1.2 Analytic E8 Orbit Root Geometry Used by All `C-R1..C-R5` Fixtures

In `geometry::ROOTS` (`GEOMETRY_SHA256 = 46ddbb76b7fbe6ac6222ea24779fc10055b1d6cea6a8da786895676fd29fe327`):

- `root 0 = [ 2,  2,  0,  0,  0,  0,  0,  0]`
- `root 1 = [-2, -2,  0,  0,  0,  0,  0,  0]` (antipode of `root 0`)
- `root 2 = [-2,  2,  0,  0,  0,  0,  0,  0]`
- `root 3 = [ 0,  0, -2,  0, -2,  0,  0,  0]` (`root 4..14` also have `0` in coordinates `0, 1` and two `±2` entries in coordinates `2..7`)
- `root 15 = [ 2, -2,  0,  0,  0,  0,  0,  0]` (antipode of `root 2`)

In `discrete.cu:267-345`:
- `root_cost(x, root)` = $\sum_{j=0}^{7} |x_j - g_{\text{roots}}[\text{root}, j]| / 2$.
  - Relative to `root 0`: `root_cost(root 0, 0) = 0`; `root_cost(root 0, 2) = 4`; `root_cost(root 0, 15) = 4`; `root_cost(root 0, r) = 8` for $r \in \{1, 3..14\}$.
  - Relative to `root 2`: `root_cost(root 2, 2) = 0`; `root_cost(root 2, 0) = 4`; `root_cost(root 2, 1) = 4`; `root_cost(root 2, r) = 8` for $r \in \{3..15\}$ (`13` roots at maximum single-block distance `8`).
  - Relative to `root 3`: `root_cost(root 3, r) = 8` for $r \in \{0, 1, 2, 15\}$.
- `token_cell(table, g, s, token, &first, &last)` (`discrete.cu:286-290`) = $a \cdot 16 + b \in [0, 255]$, where $a = \text{root\_id}(\text{block } 0)$ and $b = \text{root\_id}(\text{block } s[1]-1)$.
- `address_distance(q, g, s, cell)` (`discrete.cu:274-276`) = $\text{root\_cost}(q_{\text{first}}, \lfloor \text{cell}/16 \rfloor) + \text{root\_cost}(q_{\text{last}}, \text{cell} \bmod 16) \in [0, 24]$.
- `cell_precedes(d, c, other_d, other_c)` (`discrete.cu:301-303`) = `(d < other_d) || (d == other_d && c < other_c)`.
- `target_cell_rank` (`discrete.cu:329-334`) = $1 + |\{c \in 0..255 : \text{occupied}[c] \neq 0 \land \text{cell\_precedes}(d(c), c, \text{ad}, \text{target\_cell})\}| \in [1, 256]$.
- `cell_rank_hinge` (`discrete.cu:340`) = $R = \max(0, \text{target\_cell\_rank} - K) \in [0, 255]$ (and $\in [0, 240]$ at $K = 16$).

### 1.3 Fixture-by-Fixture GPU Specifications, Expected Values, and Pass/Fail Criteria

| Control | Governing Sheet & Source | GPU Fixture Construction (Executed via `Device` / `discrete.cu`, Zero CPU Model Reference) | Analytic / Stored Expected Values | Pass / Fail Criteria (`TEST FITNESS: FIT`) |
|---|---|---|---|---|
| **`C-R1`** `rank_hinge_exact_admission_equivalence` | Sheet line 90; `objective.rs:85-91`; `discrete.cu:292-345, 519-550` | **Fixture 1A (5-cell distance + tie sweep across $K = 1..=16$):** `blocks = 4` (`width = 32`), `context = 1`, `vocabulary = 5`, `routes = [0; 4]` (identity transport so query $q$ equals the context token's row).<br>• **Variant 1A-i (mixed distances `[0, 4, 4, 4, 8]`):** query token `0` at `[root 0, root 0, root 0, root 0]` (`cell 0`, $d=0$); token `1` at `[root 0, root 0, root 0, root 2]` (`cell 2`, $d=4$); target token `2` at `[root 0, root 0, root 0, root 15]` (`target_cell = 15`, $\text{ad}=4$); token `3` at `[root 2, root 0, root 0, root 0]` (`cell 32`, $d=4$); token `4` at `[root 15, root 0, root 0, root 15]` (`cell 255`, $d=8$). Occupied `(distance, cell_id)` pairs = `[(0, 0), (4, 2), (4, 15), (4, 32), (8, 255)]`.<br>• **Variant 1A-ii (5-way tie on cells `{0, 1, 15, 16, 255}` from `objective.rs:87`):** tokens `0..4` placed at `cell 0` (`[r0, r3, r3, r0]`), `cell 1` (`[r0, r3, r3, r1]`), `target_cell = 15` (`[r0, r3, r3, r15]`), `cell 16` (`[r1, r3, r3, r0]`), `cell 255` (`[r15, r3, r3, r15]`), plus query token `5` at `[r3, r3, r3, r3]` (`cell 51`, or query formed by `routes` carrying `[r3; 4]`). On the 5 cells `{0, 1, 15, 16, 255}`, `(distance, cell_id)` = `[(16, 0), (16, 1), (16, 15), (16, 16), (16, 255)]`.<br>• **Fixture 1B (256-cell full-occupancy maximum-rank boundary at $K = 16$):** `blocks = 4`, `context = 1`, `vocabulary = 256`, token $c \in 0..256$ placed in cell $c = a \cdot 16 + b$ (`block 0 = root a`, `blocks 1..2 = root 2`, `block 3 = root b`), context token `34` (`2 * 16 + 2`, so $q = [\text{root } 2, \text{root } 2, \text{root } 2, \text{root } 2]$), target token `255` (`target_cell = 255 = [root 15, ..., root 15]`, $\text{ad} = 8 + 8 = 16$), evaluated on GPU at `kept_cells = 16`. | • **Fixtures 1A-i & 1A-ii ($K = 1..=16$):**<br>  - `occupied_cells == 5` (1A-i)<br>  - `target_cell == 15`<br>  - `target_cell_rank == 3` at every $K \in 1..=16$<br>  - `cell_rank_hinge` across $K = 1..=16$: **`[2, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]`** (`objective.rs:88`)<br>  - `target_admitted == (admitted_cells - address_rivals)`: **`[0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]`**<br>  - `weighted_cell_rank_hinge_sum`: `32 * R = [64, 32, 0, ..., 0]`<br>• **Fixture 1B ($K = 16$, 256 occupied cells):**<br>  - `occupied_cells == 256`<br>  - `admitted_cells == 16`, `address_rivals == 16`, `target_admitted == 0`<br>  - `target_cell_rank == 256` (`87` cells at $d \in \{0, 4, 8, 12\}$ + `168` tied cells at $d = 16$ with `cell_id < 255` precede `cell 255`)<br>  - `cell_rank_hinge == 240` (`256 - 16`, `objective.rs:89`)<br>  - `weighted_cell_rank_hinge_sum == 32 * 240 = 7,680` | **PASS** iff on GPU:<br>1. `R == 0` iff `target_cell ∈ admitted[0..K]` across all $K \in 1..=16$;<br>2. `R == target_cell_rank - K > 0` whenever `target_cell ∉ admitted[0..K]`;<br>3. Ties in `address_distance` break strictly by ascending `cell_id`;<br>4. The 256-cell boundary yields exact `target_cell_rank == 256` and `cell_rank_hinge == 240` at $K = 16$. |
| **`C-R2`** `single_count_per_occupied_cell_invariance` | Sheet line 91; `objective.rs:93-98`; `discrete.cu:292-334` | Starting from Fixture 1A-i (`occupied_cells = {0, 2, 15, 32, 255}`, `target_cell = 15`, `target_cell_rank = 3`), vary the token multiplicity $m \in \{1, 2, 4\}$ placed in preceding rival `cell 2` (`[root 0, root 0, root 0, root 2]`, $d = 4 < (4, 15)$) by setting `vocabulary = 4 + m` and assigning tokens `4..(4+m)` to `[root 0, root 0, root 0, root 2]`. Execute GPU `evaluate` at $K = 1$, $K = 2$, and $K = 3$.<br>Then move one of the crowded tokens from `cell 2` into a previously empty preceding cell `cell 1` (`[root 0, root 0, root 0, root 1]`, or under `q = [root 3; 4]` into `cell 3 = [root 0, root 3, root 3, root 3]` with $d = 8 < 16$) and re-run GPU `evaluate`. | • **Crowding rival `cell 2` with $m \in \{1, 2, 4\}$ tokens:**<br>  - `occupied_cells == [5, 5, 5]`<br>  - `target_cell_rank == [3, 3, 3]` (`objective.rs:95`)<br>  - `cell_rank_hinge` at $K = 1$: **`[2, 2, 2]`**; at $K = 2$: **`[1, 1, 1]`**; at $K = 3$: **`[0, 0, 0]`**<br>  - `admitted_token_count` at $K = 3$: **`[3, 4, 6]`** (grows by $+1, +3$ while `target_cell_rank` and `R` remain strictly invariant)<br>• **Single token moved into newly occupied preceding cell:**<br>  - `occupied_cells` increases by `+1` (`5 -> 6`)<br>  - `target_cell_rank` increases by **at most `+1` (`3 -> 4`)** (`objective.rs:96`) | **PASS** iff:<br>1. Adding $m \in \{1, 2, 4\}$ tokens to an already-occupied rival cell produces zero change (`0` delta) in GPU `target_cell_rank` and `cell_rank_hinge`;<br>2. Moving a single token into a newly occupied cell changes `target_cell_rank` by $\Delta r \in \{-1, 0, +1\}$ (and $\Delta R$ by at most $1$). |
| **`C-R3`** `virtual_vs_applied_candidate_identity_on_rank_and_occupancy` | Sheet line 92; `objective.rs:100-104`; `discrete.cu:384-517` | Construct 5 disposable GPU snapshots covering all 5 mutation classes in `objective.rs:100`:<br>1. `create_cell`: moving the selected token's endpoint block `0` or `B-1` into a previously empty cell ($|\text{occ}| \to |\text{occ}| + 1$);<br>2. `empty_cell`: moving the sole token out of an occupied cell into an already-occupied cell ($|\text{occ}| \to |\text{occ}| - 1$);<br>3. `move_target_cell`: `Parameter::VectorBlock { token: target_token, block: 0 or B-1 }`, directly mutating `target_cell` and `ad`;<br>4. `interior_block_only`: `Parameter::VectorBlock { token, block: 1 or 2 }` at `blocks = 4` (`0 < block < B-1`), mutating interior blocks while `token_cell` endpoints `0` and `B-1` are untouched;<br>5. `identity_candidate`: `candidate = 0` (`OPERATOR_TABLE[0] = identity`) and route candidate equal to current `routes[idx]`.<br>For each fixture and each policy (`NearestAddressMarginV1` and `OccupiedCellRankHingeV1`), run `model.propose(&batch, &targets, param)` on GPU to obtain `proposal.candidate_losses()?[0..16]`, and compare against GPU `evaluate` after applying each candidate on an isolated disposable fixture clone. | • Across all `5` mutation classes $\times$ `2` policies $\times$ `16` candidate lanes (`160` candidate evaluations):<br>  - `proposal.candidate_losses()?[c] == post_apply_eval.objective_loss_numerator_sum` (`loss_ta_numerator_sum` under `NearestAddressMarginV1`, `loss_tr_numerator_sum` under `OccupiedCellRankHingeV1`)<br>  - `proposal.baseline_loss() == pre_apply_eval.objective_loss_numerator_sum`<br>  - In class `4` (`interior_block_only`, `block ∈ {1, 2}` at `B = 4`), across all 16 candidates: `occupied_cells`, `target_cell`, `target_cell_rank`, `cell_rank_hinge`, and `address_hinge` are **100% invariant (`delta == 0`)**, while only `token_distance` and `token_hinge` change.<br>  - In class `5` (`identity_candidate`, `c = 0`): `candidate_losses()[0] == baseline_loss()`. | **PASS** iff virtual counterfactual loss in `discrete_counterfactual` equals post-apply `discrete_evaluate` objective loss (`8*T + 32*A` or `8*T + 32*R`) to the integer across all 16 candidates and all 5 mutation classes (`create_cell`, `empty_cell`, `move_target_cell`, `interior_block_only`, `identity_candidate`). |
| **`C-R4`** `absent_rival_and_occupancy_under_k_bounds` | Sheet line 93; `objective.rs:106-112`; `discrete.cu:389-396, 488-503` | • **Fixture 4A (`occupied_cells <= K`):** At $K = 16$, test (i) `M = 1` occupied cell (all `vocabulary = 8` tokens placed in `cell 0` with distinct interior blocks `1, 2`) and (ii) `M = 16` occupied cells (`16` tokens in `cells 0..16`). Run GPU `evaluate` over targets in every occupied cell.<br>• **Fixture 4B (`tr == 0` row skipping & all-ineligible status `8` refusal):**<br>  - *4B-i (mixed batch):* 2-row batch where row `0` has `token_rivals == 0` (only the target token resides in the admitted cell set) and row `1` has `token_rivals >= 1`. Run `model.propose` under both `NearestAddressMarginV1` and `OccupiedCellRankHingeV1` and compare against a 1-row batch containing only row `1`.<br>  - *4B-ii (all-ineligible batch):* 1-row batch where `token_rivals == 0`. Call `model.propose` under both `NearestAddressMarginV1` and `OccupiedCellRankHingeV1`. | • **Fixture 4A (`M = 1` and `M = 16` at $K = 16$):**<br>  - For `M = 1`: `occupied_cells == 1`, `target_cell_rank == 1`, `cell_rank_hinge == 0`, `target_admitted == 1` (`objective.rs:108`).<br>  - For `M = 16`: `occupied_cells == 16`, `target_cell_rank ∈ 1..=16`, `cell_rank_hinge == 0`, `target_admitted == 1` on every row.<br>• **Fixture 4B:**<br>  - *4B-i:* Row `0` (`tr == 0`) contributes `0` to `candidate_losses()[0..16]` under both policies (`candidate_losses(row0 + row1) == candidate_losses(row1)`).<br>  - *4B-ii:* Both policies return `Err(Error::InvalidModel("discrete CUDA counterfactual refused with status 8"))` (`expected_counterfactual_refusal_status: 8`, `objective.rs:109`). | **PASS** iff:<br>1. Whenever `occupied_cells <= K`, `target_cell_rank <= occupied_cells <= K`, `cell_rank_hinge == 0`, and `target_admitted == 1` on every row;<br>2. Rows with `tr == 0` are skipped identically under both `NearestAddressMarginV1` and `OccupiedCellRankHingeV1`, and a batch with zero `tr > 0` rows refuses with CUDA status `8` under both policies. |
| **`C-R5`** `bounded_integer_arithmetic_dual_loss_and_distribution_schema` | Sheet line 94; `objective.rs:114-120, 130-427`; `discrete.cu:566-698` | • **Fixture 5A (Dual-loss, exact-ratio, and histogram/quantile verification on GPU):** Run `evaluate_corpus_Aggregate` (`discrete_evaluate` + `discrete_evaluation_aggregate`) under both `NearestAddressMarginV1` and `OccupiedCellRankHingeV1` on a multi-row GPU fixture spanning $T \in [0, 640]$, $A \in [0, 72]$, $R \in [0, 240]$ at $B = 4, \text{width} = 32, K = 16$.<br>• **Fixture 5B (CUDA status `12` bounds & overflow refusal):** Invoke `discrete_evaluation_aggregate` with malformed row words testing each guard in `discrete.cu:578-680`: (i) `token_hinge = 641` ($> 640$), (ii) `address_hinge = 73` ($> 72$), (iii) `cell_rank_hinge = 256` ($> 255$), (iv) `target_cell_rank = 0` or `257`, (v) `occupied_cells = 0` or `257`, (vi) `admitted_cells = 0` or `17`, (vii) mismatched `loss_ta != 8*T + width*A` or `loss_tr != 8*T + width*R`, and (viii) cross-row `occupied_cells` mismatch. | • **Fixture 5A:**<br>  - `schema == "uor-r4.discrete-objective-evaluation/1"`<br>  - `ROW_WORDS == 14`, `AGGREGATE_WORDS == 1020`, `HISTOGRAM_BINS == [641, 73, 256]`<br>  - `weighted_token_hinge_sum == 8 * token_hinge_sum`<br>  - `weighted_address_hinge_sum == 32 * address_hinge_sum`<br>  - `weighted_cell_rank_hinge_sum == 32 * cell_rank_hinge_sum`<br>  - `loss_ta_numerator_sum == weighted_token_hinge_sum + weighted_address_hinge_sum == total_loss_numerator_sum`<br>  - `loss_tr_numerator_sum == weighted_token_hinge_sum + weighted_cell_rank_hinge_sum`<br>  - `objective_loss_numerator_sum == loss_ta_numerator_sum` (`NearestAddressMarginV1`) or `loss_tr_numerator_sum` (`OccupiedCellRankHingeV1`)<br>  - `address_loss_share_ta == { numerator: weighted_address_hinge_sum, denominator: loss_ta_numerator_sum }`<br>  - `rank_loss_share_tr == { numerator: weighted_cell_rank_hinge_sum, denominator: loss_tr_numerator_sum }`<br>  - `mean_admitted_token_count == { numerator: admitted_token_count_sum, denominator: scored_rows }`<br>  - For each of `token_hinge`, `address_hinge`, `cell_rank_hinge`: $\sum \text{bins} == \text{scored\_rows}$, `zero_rows == bins[0]`, and `min, p25, p50, p75, p90, p99, max` equal exact nearest-rank $\lceil p \cdot N / 100 \rceil$ bin indices.<br>• **Fixture 5B:** Every out-of-bounds/inconsistent input returns `Err(Error::InvalidModel("discrete CUDA evaluation aggregation refused with status 12"))` (`expected_aggregate_overflow_status: 12`, `objective.rs:119`). | **PASS** iff:<br>1. All dual-loss sums, exact ratios, occupancy fields, and `[641, 73, 256]` histogram/quantile fields match exact `i32`/`u64` integer identities on GPU under both policies;<br>2. All 8 out-of-bound / inconsistent row tests are refused by `discrete_evaluation_aggregate` with CUDA status `12`. |
| **`C-R6`** `k16_baseline_guard_and_untrained_ceiling_check` | Sheet line 95; `objective.rs:122-127`; `completion_baseline.rs:70, 567-611` | 1. **Stored baseline guard (`admitted_cells ∈ 1..=16`):** Verify `completion_baseline.rs:70` admits `admitted_cells = 16` on the sealed 36-point `K ∈ {8..16}` Stage-A sweep (`report.json` SHA-256 `cc81ca6f76f3a38ac61ffe1d4665a1442d1e42961f4451691327b91b438f472c`) and refuses `admitted_cells = 0` or `admitted_cells = 17`.<br>2. **Zero-update `G-Pre` GPU corpus check:** Run Section 2 below (`init-corpus` + `eval-corpus` at `accepted_updates == 0` for `seed ∈ {1, 2}`, `K ∈ {8, 16}`). | • `admitted_cells_range == [1, 16]` (`objective.rs:124`)<br>• `g_pre_Checkpoint0_k16_target_admitted_bar == 4,894` (`objective.rs:125`)<br>• At `seed = 1, K = 8, cursor = 0`: `target_admitted_rows == 5,680`, `top1_correct == 17`, `top8_correct == 173`, `token_strict_win_eligible_rows == 105`, `occupied_cells == 16`<br>• At `seed ∈ {1, 2}, K = 16, cursor = 0`: `target_admitted_rows >= 4,894` on both seeds (`G-Pre PASS`) | **PASS** iff:<br>1. `completion_baseline.rs` accepts `1..=16` and rejects `0` and `>= 17`;<br>2. `seed = 1, K = 8` checkpoint `0` reproduces the sealed pilot checkpoint-0 counts to the digit;<br>3. Both `seed = 1` and `seed = 2` at `K = 16` checkpoint `0` achieve `target_admitted_rows >= 4,894` with `0` `.apply` updates. |

---

## 2. `G-Pre` — Untrained `K=16` Checkpoint-0 Zero-Update Ceiling Evaluation (`bar >= 4,894`)

### 2.1 Purpose & Governing Requirements

Per **Fact 4**, **Fact 5**, Amendment 5, and the owner's explicit confirmation ([#2515 comment 6198442894](https://github.com/UOR-Foundation/uor-r4/pull/2515#issuecomment-6198442894)):
1. **Capacity was trained out (`5,680` at `K=8` checkpoint `0` $\to$ `2,180` at cursor `1,640` $\to$ `1,920` at cursor `16,404`), not absent.** Before any `K=16` training lease is requested, `G-Pre` measures the untrained checkpoint-0 `K=16` admission ceiling (`0` updates applied) across **`seed = 1` and `seed = 2` at `K = 8` and `K = 16`** on the 400-document `ft/dev` split (`14,287` eligible target rows).
2. **`G-Pre` release prerequisite bar:** Both `seed = 1` and `seed = 2` at `K = 16` checkpoint `0` must satisfy:
   $$\text{target\_admitted\_rows}(K=16, t=0) \ge 4,894 \quad (\text{out of } 14,287 \text{ eligible rows}).$$
   - **Analytic checkpoint-0 identity in `discrete_init` (`discrete.cu:353-355`):** At initialization, `discrete_init` assigns block roots via `xorshift32(seed ^ ((token * blocks + block) * 0x9e3779b9)) & 15`. Because `blocks = 4`, for every token $v \in 0..4096$, `block 0` uses index $4v$ and `block 3` uses index $4v + 3 = (4v) \oplus 3$, which pairs each first-block root $a \in 0..15$ deterministically with a unique last-block root $b(a) \in 0..15$, populating **at most `16` occupied endpoint cells across all `4,096` tokens at checkpoint `0`** (`occupied_cells == 16`, as already observed in the sealed `seed = 1, K = 8` checkpoint-0 receipt where `occupied_cells = 16` and `5,680 / 14,287` rows fell in the top `8` of `16` occupied cells). Consequently, at `K = 16` checkpoint `0`, all `<= 16` occupied cells are admitted (`admitted_cells == occupied_cells`), yielding `target_cell_rank <= 16`, `cell_rank_hinge_sum == 0`, and **`target_admitted_rows == 14,287 >= 4,894`**.
3. **Re-measuring the `K=16` checkpoint-0 strict-win baseline `u` (`Fact 4`):** Record `token_strict_win_eligible_rows` and `admitted_strict_win_rows` at checkpoint `0` for each `(seed, arm)` (`seed1-k16-a`, `seed1-k16-r`, `seed2-k16-a`, `seed2-k16-r`). This sets each arm's checkpoint-0 baseline rate $u = \text{token\_strict\_win\_eligible\_rows}(t=0) / 14,287$ and the one-sided 99% binomial guard threshold $\lceil 14,287 u + 2.326348 \sqrt{14,287 u (1 - u)} \rceil$ required by `G-Guard`.
4. **Optional `checkpoint-01640.json` early-decay diagnostic (`#2515` comment `6198442894`):** If the preserved seed-1 pilot `checkpoint-01640.json` (`1,640` updates, `K=8` `target_admitted_rows = 2,180`) is staged on the pod, evaluate it at `K = 8` and `K = 16` via `eval-corpus` (`0` updates applied) to record the `t = 1,640` `K=8 -> K=16` admission curve alongside `t = 0` and `t = 4,101..16,404`.

### 2.2 Verbatim `G-Pre` Configuration Files & Commands (`init-corpus` + `eval-corpus`, Zero `.apply`)

`discrete-stack init-corpus` and `discrete-stack eval-corpus` (`crates/uor-r4-discrete/src/main.rs:71-93`) are structurally zero-update commands: `init-corpus` runs only `discrete_init` + `discrete_validate` (`cursor.accepted_updates == 0`) and `eval-corpus` hard-refuses any checkpoint with `checkpoint.cursor.accepted_updates != 0` (`Err(Error::TrainingHeld)` at `main.rs:84-86`).

Run the following script inside `/workspace/uor-r4` via `uor-pod run` (Section 4):

```bash
set -euo pipefail
cd /workspace/uor-r4

export UOR_NVCC_ARCH=compute_120
OUT_ROOT="/workspace/uor-r4/evals/20261015-dr-qualification"
mkdir -p "${OUT_ROOT}/configs"

CORPUS_ROOT="/workspace/uor-r4/artifacts/corpora/fineweb-edu-v4096-100m-v2"
DRIVER_PIN="3487506d0d6b9197e8b99963dfe92e91b5c3e33e0801d9116bf525cdc9b17fb7"

write_cfg() {
  local name="$1"
  local seed="$2"
  local k="$3"
  local policy="$4"
  cat > "${OUT_ROOT}/configs/${name}.json" <<EOF
{
  "schema": "uor-r4.discrete-corpus-run-config/1",
  "shape": {
    "vocabulary": 4096,
    "blocks": 4,
    "context": 192,
    "kept_cells": ${k}
  },
  "read_spec": "route",
  "seed": ${seed},
  "batch_rows": 64,
  "max_batches": 1,
  "checkpoint_every_batches": 1,
  "evaluate_every_batches": 1,
  "schedule_policy": "canonical_vector_then_read_v1",
  "objective_policy": "${policy}",
  "expected_driver_source_sha256": "${DRIVER_PIN}",
  "expected_train_tokens_sha256": "8c488f50e0e1ca67a29937be02d4eb751b096757d767710a6ab4fa737df0b86c",
  "expected_ft_train_tokens_sha256": "de8cc71e30dd2f0dd051660870fb2512ed95462335eb7fa18b62796a6aa7484f",
  "expected_ft_train_mask_sha256": "e53fdcfde4a1b983b15b65ac3e24b13f8e8c8c5e3a4be73594a16867f1ff0092",
  "expected_ft_dev_tokens_sha256": "0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b",
  "expected_ft_dev_mask_sha256": "a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078",
  "expected_tokenizer_sha256": "20cd9d96f9d2fe2c1e6e62ff7c5fc4cb810fe57279313fe0660b96597f2685fd",
  "train_tokens_path": "${CORPUS_ROOT}/train/tokens.u16",
  "ft_train_tokens_path": "${CORPUS_ROOT}/ft/train/tokens.u16",
  "ft_train_mask_path": "${CORPUS_ROOT}/ft/train/response_mask.u8",
  "ft_dev_tokens_path": "${CORPUS_ROOT}/ft/dev/tokens.u16",
  "ft_dev_mask_path": "${CORPUS_ROOT}/ft/dev/response_mask.u8",
  "tokenizer_path": "${CORPUS_ROOT}/tokenizer.json"
}
EOF
}

write_cfg "seed1-k8-a"  1 8  "nearest_address_margin_v1"
write_cfg "seed1-k16-a" 1 16 "nearest_address_margin_v1"
write_cfg "seed1-k16-r" 1 16 "occupied_cell_rank_hinge_v1"
write_cfg "seed2-k8-a"  2 8  "nearest_address_margin_v1"
write_cfg "seed2-k16-a" 2 16 "nearest_address_margin_v1"
write_cfg "seed2-k16-r" 2 16 "occupied_cell_rank_hinge_v1"

BIN="./target/release/discrete-stack"

for run in seed1-k8-a seed1-k16-a seed1-k16-r seed2-k8-a seed2-k16-a seed2-k16-r; do
  "${BIN}" init-corpus "${OUT_ROOT}/configs/${run}.json" "${OUT_ROOT}/g-pre-${run}-init"
  "${BIN}" eval-corpus "${OUT_ROOT}/configs/${run}.json" \
    "${OUT_ROOT}/g-pre-${run}-init/checkpoint-00000.json" \
    "${OUT_ROOT}/g-pre-${run}-eval"
done
```

### 2.3 `G-Pre` Pass/Fail Verification Table

| Run ID | Seed | $K$ | Policy | Expected `accepted_updates` / `updates_applied_by_evaluation` | Expected `occupied_cells` | Expected `target_admitted_rows` (`G-Pre` Bar) | Expected `top1` / `top8` / `strict_wins` |
|---|---|---|---|---|---|---|---|
| `g-pre-seed1-k8-a` | `1` | `8` | `nearest_address_margin_v1` | `0` / `0` | `16` | **`== 5,680`** (exact match to sealed pilot `evaluation-00000.json`) | `top1 == 17`, `top8 == 173`, `token_strict_win_eligible_rows == 105`, `admitted_strict_win_rows == 0`, `address_hinge_sum == 261,824` |
| `g-pre-seed1-k16-a` | `1` | `16` | `nearest_address_margin_v1` | `0` / `0` | `16` | **`>= 4,894`** (analytically `14,287` since `occupied_cells = 16 <= K = 16`) | Record `u_seed1_k16 = token_strict_win_eligible_rows / 14,287`; `cell_rank_hinge_sum == 0` |
| `g-pre-seed1-k16-r` | `1` | `16` | `occupied_cell_rank_hinge_v1` | `0` / `0` | `16` | **`>= 4,894`** (`== target_admitted_rows(seed1-k16-a)`) | Identical to `seed1-k16-a` on `top1`, `top8`, `strict_wins`, `target_admitted_rows`, `T`, `A`, `R`, `loss_ta_numerator_sum`, `loss_tr_numerator_sum`; `objective_loss_numerator_sum == loss_tr_numerator_sum == weighted_token_hinge_sum` |
| `g-pre-seed2-k8-a` | `2` | `8` | `nearest_address_margin_v1` | `0` / `0` | `16` | Record `target_admitted_rows(seed2, K=8, t=0)` | Record `seed 2` `K=8` checkpoint-0 reference |
| `g-pre-seed2-k16-a` | `2` | `16` | `nearest_address_margin_v1` | `0` / `0` | `16` | **`>= 4,894`** (analytically `14,287` since `occupied_cells = 16 <= K = 16`) | Record `u_seed2_k16 = token_strict_win_eligible_rows / 14,287`; `cell_rank_hinge_sum == 0` |
| `g-pre-seed2-k16-r` | `2` | `16` | `occupied_cell_rank_hinge_v1` | `0` / `0` | `16` | **`>= 4,894`** (`== target_admitted_rows(seed2-k16-a)`) | Identical to `seed2-k16-a` on all row/ranking/dual-loss counts; `objective_loss_numerator_sum == loss_tr_numerator_sum` |

- **`G-Pre` Verdict Rule:**
  - **PASS:** `g-pre-seed1-k8-a` matches the sealed pilot checkpoint-0 counts (`17 / 173 / 105 / 0 / 5,680 / 261,824 / 16`), both `seed = 1` and `seed = 2` at `K = 16` satisfy `target_admitted_rows >= 4,894`, and `updates_applied_by_evaluation == 0` on all 6 evaluations.
  - **STOP (`G-Pre FAIL`):** If either `seed = 1` or `seed = 2` at `K = 16` checkpoint `0` has `target_admitted_rows < 4,894`, stop immediately before any `K = 16` training lease and diagnose initialization geometry (`m1-discrete-rung0-admission-redirect-2026-10-15.md` lines 48, 95, 166).

---

## 3. 100-Batch `K=16` Synchronized Timing Benchmark (Filling the `D-Packet` `tokens/s` Slot)

### 3.1 Why a Dedicated `K=16` Dual-Occupancy Benchmark Is Required (`Fact 6`)

1. The existing `discrete-stack measure-cost` subcommand (`crates/uor-r4-discrete/src/measurement.rs:12-17`) is hardcoded to `SHAPE.kept_cells = 8` and times 3 repeats of a single synthetic batch at checkpoint-0 occupancy (`16` occupied cells).
2. In `discrete_counterfactual` (`discrete.cu:453-485`), each candidate lane scans all `256` cells to build `admitted[0..K]` (`O(256 * K)` comparisons when `occupied_cells >= K`) and then scans all `4,096` vocabulary tokens whose cell lies in `admitted[0..K]`. Under `OccupiedCellRankHingeV1`, `discrete_counterfactual` additionally counts occupied cells preceding `target_cell` (`discrete.cu:508-513`).
3. Per **Fact 6** (`m1-discrete-rung0-admission-redirect-2026-10-15.md` lines 49, 102–112, 167), throughput and Stage-B / `K=16` wall-time must be sized from **100 synchronized batches at `K = 16` (`batch_rows = 64`, `context = 192`, `6,400` target rows, `1,228,800` physical context tokens)** and anchored at **Q4 high cell occupancy (`189` occupied cells from `checkpoint-16404.json`)** alongside checkpoint-0 low occupancy (`16` occupied cells), with **zero `.apply` updates**.

### 3.2 Benchmark Protocol, Two-Slice Family-Stratified `80/20` Sampling, and Four Required Timing Cells (`0` `.apply` Calls)

Execute a **two-slice family-stratified `80 + 20 = 100`-batch timing sample** (`Device::propose` + `Device::synchronize()`, **never calling `Device::apply`**) after 1 untimed warmup batch on `1× RTX 5090` (`compute_120`) across the $2 \times 2$ grid of **(Occupancy Regime $\times$ Objective Policy)** at `V4096 / B4 / C192 / K16`.

- **Honest sampling label (`0..80` + `16384..16404` is a two-slice family-stratified sample, NOT 100 consecutive canonical coordinates):**
  - **Slice 1 (`VectorBlock` prefix slice, `80` batches, sample indices `b = 0..80`):** canonical `step_index ∈ 0..80` (`Parameter::VectorBlock { token: (step_index / 4) as u16, block: (step_index % 4) as u8 }`, covering tokens `0..20` across blocks `0..4`).
  - **Slice 2 (`Route` tail slice, `20` batches, sample indices `b = 80..100`):** jumps across the canonical `16,404`-coordinate cycle from `step_index = 79` directly to the route tail segment `step_index ∈ 16384..16404` (`Parameter::Route { stage: ((step_index - 16384) / 4) as u8, block: ((step_index - 16384) % 4) as u8 }`, covering all `20` route coordinates across stages `0..5` and blocks `0..4`).
  - **Per-sample coordinate & corpus target-position pins:** Every timed sample `b ∈ 0..100` must record in `timing-k16-100batch.json` its exact `(sample_index, canonical_step_index, parameter_family, parameter, corpus_batch_index, first_target_position, last_target_position, target_positions_sha256, elapsed_ns)` tuple so that the `80` `VectorBlock` samples and `20` `Route` samples are auditable per sample and reproducible to the exact corpus rows.

| Timing Cell ID | Occupancy Regime | Model State (`epoch = 0`, `accepted_updates = 0`, Zero `.apply`) | Objective Policy | Timed Batches (`batch_rows = 64`, `context = 192`) | Two-Slice Family-Stratified Coordinate Sample (`80` `VectorBlock` + `20` `Route`, Non-Consecutive Across `79 -> 16384`) |
|---|---|---|---|---|---|
| **`T-K16A-OCC16`** | Regime 0 (`occupied_cells = 16`, checkpoint-0 init) | Fresh `seed = 1` checkpoint-0 `Snapshot` (`g-pre-seed1-k16-a-init/checkpoint-00000.json`) | `NearestAddressMarginV1` (`8*T + 32*A`) | `1` warmup + `100` timed (`6,400` rows, `1,228,800` context tokens) | Slice 1: `80` `VectorBlock` (`step_index 0..80`) + Slice 2: `20` `Route` (`step_index 16384..16404`) |
| **`T-K16R-OCC16`** | Regime 0 (`occupied_cells = 16`, checkpoint-0 init) | Fresh `seed = 1` checkpoint-0 `Snapshot` (`g-pre-seed1-k16-r-init/checkpoint-00000.json`) | `OccupiedCellRankHingeV1` (`8*T + 32*R`) | `1` warmup + `100` timed (`6,400` rows, `1,228,800` context tokens) | Slice 1: `80` `VectorBlock` (`step_index 0..80`) + Slice 2: `20` `Route` (`step_index 16384..16404`) |
| **`T-K16A-OCC189`** | Regime Q4 (`occupied_cells = 189`, Q4 sizing anchor per `Fact 6`) | `table` and `routes` copied from sealed `checkpoint-16404.json` (`4477eb5c2480b0b85cc94036a2e15b0d22b654e25deee9c8717aa6e30f8ef7bf`) into a `K = 16, epoch = 0` `Snapshot` | `NearestAddressMarginV1` (`8*T + 32*A`) | `1` warmup + `100` timed (`6,400` rows, `1,228,800` context tokens) | Slice 1: `80` `VectorBlock` (`step_index 0..80`) + Slice 2: `20` `Route` (`step_index 16384..16404`) |
| **`T-K16R-OCC189`** | Regime Q4 (`occupied_cells = 189`, Q4 sizing anchor per `Fact 6`) | `table` and `routes` copied from sealed `checkpoint-16404.json` (`4477eb5c2480b0b85cc94036a2e15b0d22b654e25deee9c8717aa6e30f8ef7bf`) into a `K = 16, epoch = 0` `Snapshot` | `OccupiedCellRankHingeV1` (`8*T + 32*R`) | `1` warmup + `100` timed (`6,400` rows, `1,228,800` context tokens) | Slice 1: `80` `VectorBlock` (`step_index 0..80`) + Slice 2: `20` `Route` (`step_index 16384..16404`) |

*(Note on candidate ordinal evaluation and why `80` `VectorBlock` + `20` `Route` are timed separately: `Device::propose` (`crates/uor-r4-discrete/src/device.rs:285-336`) and `discrete_counterfactual` (`crates/uor-r4-discrete/src/discrete.cu:384-517`, grid `(16, 1, 1)`) evaluate **all `16` candidate ordinals (`candidate ∈ 0..16`, i.e., `CURRENT` + `15` candidate ordinals) for both `Parameter::VectorBlock` (`move_action(g, candidate)` across all `16` E8 action table rows `0..16`) and `Parameter::Route` (`candidate * 4 + (current & 3)` across all `16` route source choices `0..16`)**. Their per-batch GPU costs differ because in `discrete_counterfactual` (`discrete.cu:397-449`), a `VectorBlock` step (`kind == 1`) mutates a single vocabulary token `p_a` — re-running `forward_window` only on context rows containing `p_a` (`w[t] == p_a`) and changing at most one token's cell (`p_b == 0` or `p_b == B - 1`) — whereas a `Route` step (`kind == 2`) mutates tree routing at `(stage = p_a, block = p_b)` and unconditionally re-runs `forward_window` across the `C = 192` window for every eligible row and every non-current route ordinal (`route_val != current_route`). Timing the two-slice `80/20` sample and reporting separately weighted family rates (`ns_per_vector_batch_floor` over `80` batches and `ns_per_route_batch_floor` over `20` batches) gives an exact per-family cost formula for the full `16,384` `VectorBlock` + `20` `Route` = `16,404`-step cycle.)*

### 3.3 Exact Integer Rate Formulas for `D-Packet` (`Zero Float`, Separately Weighted Family Rates + Aggregate)

For each of the four timing cells above, record `elapsed_vector_ns: u128` across the `80` `VectorBlock` batches (`step_index 0..80`), `elapsed_route_ns: u128` across the `20` `Route` batches (`step_index 16384..16404`), and `elapsed_ns = elapsed_vector_ns + elapsed_route_ns` across all `100` timed batches:

$$\begin{aligned}
\text{timed\_batches} &= 80 + 20 = 100, \quad \text{batch\_rows} = 64, \quad \text{context} = 192 \\
\text{target\_rows} &= 100 \times 64 = 6,400 \quad (\text{vector: } 5,120,\ \text{route: } 1,280) \\
\text{context\_tokens} &= 100 \times 64 \times 192 = 1,228,800 \quad (\text{vector: } 983,040,\ \text{route: } 245,760) \\
\text{ns\_per\_vector\_batch\_floor} &= \lfloor \text{elapsed\_vector\_ns} / 80 \rfloor \\
\text{ns\_per\_route\_batch\_floor} &= \lfloor \text{elapsed\_route\_ns} / 20 \rfloor \\
\text{ns\_per\_batch\_floor} &= \lfloor \text{elapsed\_ns} / 100 \rfloor \\
\text{vector\_context\_tokens\_per\_second\_floor} &= \left\lfloor \frac{983,040 \times 1,000,000,000}{\text{elapsed\_vector\_ns}} \right\rfloor \\
\text{route\_context\_tokens\_per\_second\_floor} &= \left\lfloor \frac{245,760 \times 1,000,000,000}{\text{elapsed\_route\_ns}} \right\rfloor \\
\text{sample\_context\_tokens\_per\_second\_floor} &= \left\lfloor \frac{1,228,800 \times 1,000,000,000}{\text{elapsed\_ns}} \right\rfloor \\
\text{sample\_target\_rows\_per\_second\_floor} &= \left\lfloor \frac{6,400 \times 1,000,000,000}{\text{elapsed\_ns}} \right\rfloor \\
\text{projected\_pass\_ns}(16,404\text{ batches}) &= \left\lceil \frac{16,384 \times \text{elapsed\_vector\_ns}}{80} + \frac{20 \times \text{elapsed\_route\_ns}}{20} \right\rceil \\
\text{projected\_pass\_seconds\_ceil}(16,404\text{ batches}) &= \left\lceil \frac{\text{projected\_pass\_ns}(16,404\text{ batches})}{1,000,000,000} \right\rceil \\
\text{cycle\_weighted\_context\_tokens\_per\_second\_floor} &= \left\lfloor \frac{16,404 \times 64 \times 192 \times 1,000,000,000}{\text{projected\_pass\_ns}(16,404\text{ batches})} \right\rfloor
\end{aligned}$$

- **`D-Packet` Slot Rule (`Fact 6`):**
  - Populate the `D-Packet` `tokens/s` slot with **both** the Regime 0 (`occupied_cells = 16`) and Regime Q4 (`occupied_cells = 189`) separately weighted family rates (`vector_context_tokens_per_second_floor`, `route_context_tokens_per_second_floor`, `cycle_weighted_context_tokens_per_second_floor`) and sample rates (`sample_context_tokens_per_second_floor`, `sample_target_rows_per_second_floor`) for `K16-A` and `K16-R`, and use the **slower of Regime 0 and Regime Q4 (`max(projected_pass_seconds_ceil_occ16, projected_pass_seconds_ceil_occ189)`)** plus `20 × eval-corpus` wall time (`5` cursors $\times$ `4` runs) to bound total `GPU-hours` and dollar cost for the `65,616`-batch (`4`-run) contrast.
  - Pre-verify that the initial snapshot SHA-256 on GPU after all `100` timed `propose` batches is **bit-for-bit identical** to the snapshot SHA-256 before batch `0` (`to_snapshot()?.sha256() == initial_snapshot_sha256`, `epoch == 0`).

---

## 4. Pod Discipline — `uor-pod` Lifecycle, Single-GPU Locking, and `cloud-store put` MD5 Round-Trip Before Teardown

### 4.1 Hard Pod & Budget Rules (`docs/labs/compute.md`, `AGENTS.md`)

1. **Authorization gate:** Do **not** run `uor-pod up` or `uor-pod lease` until the owner explicitly releases the qualification lease on `#2028` / `#2029` after exact-head review of `D-R`.
2. **Fleet & spend caps:** $\le 4$ live pods fleet-wide, $\le \$8.00/\text{h}$ fleet-wide (`1× RTX 5090` is $\$0.89/\text{h}$), reuse an existing idle 5090 pod before creating a new one (`uor-pod up` refuses if a free matching GPU already exists).
3. **One job per GPU:** Every GPU command must execute through `scripts/pod/uor-pod run POD --lab <lab> --session <session> --gpu 0 -- <cmd>`, which acquires `flock -n /root/gpu0.lock` on the pod and tees output to `/workspace/uor-r4/jobs/<lab>/<UTC>-<session>-gpu0.log` with a trailing `# exit=<rc> <UTC>` marker.
4. **Lease & heartbeat cadence:** Renew the harness heartbeat every $\le 5$ minutes (`UOR_LAB=<lab> uor-harness heartbeat -m m1`) and renew the pod lease every $\le 30$ minutes (`scripts/pod/uor-pod renew POD --lab <lab> --session <session> --hours 1`). Never cancel a healthy run because a self-set timer expired.
5. **Archive with MD5 round-trip BEFORE `release` / `down`:** Never release or terminate a pod until the sealed output directory has been copied to laptop staging (`~/uor-r4-local/<run-dir>`), verified via `shasum -a 256 -c SHA256SUMS`, and archived via `~/.local/share/uor-r4/bin/cloud-store put <lab> ~/uor-r4-local/<run-dir>` with a verified MD5 round-trip against `icloud:UOR-R4/results/<lab>/<run-dir>.tar`.

### 4.2 Verbatim Step-by-Step Pod Sequence

#### Step 4.2.1 — Pre-Flight Fleet Check & Lease Acquisition (Laptop)

```bash
export UOR_LAB=<lab>
export SESSION="dr-qual-20261015"
export REVIEWED_SHA="<exact-reviewed-main-sha>"

# 1. Record harness presence
/Users/casey.allard/uor-r4/local/harness-target/debug/uor-harness heartbeat -m m1

# 2. Inspect fleet & active leases (must be <= 4 pods, <= $8.00/h)
scripts/pod/uor-pod status

# 3a. If an idle RTX 5090 pod is already RUNNING with a free GPU 0, lease it:
scripts/pod/uor-pod lease <POD_ID> \
  --lab "${UOR_LAB}" \
  --session "${SESSION}" \
  --gpus 0 \
  --purpose "M1 D-R C-R1..C-R6 + G-Pre + 100-batch K16 timing qualification (NOT_TRAINING, 0 apply)" \
  --hours 1 \
  --card "https://github.com/UOR-Foundation/uor-r4/issues/2029"

# 3b. Otherwise, if no free RTX 5090 exists, provision 1x RTX 5090 ($0.89/h):
scripts/pod/uor-pod up \
  --lab "${UOR_LAB}" \
  --session "${SESSION}" \
  --count 1 \
  --gpu 5090 \
  --ref "${REVIEWED_SHA}" \
  --purpose "M1 D-R C-R1..C-R6 + G-Pre + 100-batch K16 timing qualification (NOT_TRAINING, 0 apply)" \
  --hours 1 \
  --card "https://github.com/UOR-Foundation/uor-r4/issues/2029"
```

#### Step 4.2.2 — Pod Bootstrap, Pin Audit, and Release Build (`uor-pod run`, GPU 0)

```bash
scripts/pod/uor-pod run "${POD_ID}" --lab "${UOR_LAB}" --session "${SESSION}" --gpu 0 -- "
  set -euo pipefail
  cd /workspace/uor-r4
  git fetch origin
  git checkout --detach ${REVIEWED_SHA}
  test \"\$(git rev-parse HEAD)\" = \"${REVIEWED_SHA}\"

  # Stage frozen fineweb-edu-v4096-100m-v2 corpus + Q4 checkpoint-16404.json if not yet present
  if [ ! -f artifacts/corpora/fineweb-edu-v4096-100m-v2/ft/dev/tokens.u16 ]; then
    /root/uor-store get corpora/fineweb-edu-v4096-100m-v2.tar.zst /tmp/corpus.tar.zst
    mkdir -p artifacts/corpora
    tar --zstd -xf /tmp/corpus.tar.zst -C artifacts/corpora
  fi

  # Verify nvcc 12.8+ and build release binaries + test harness with PTX admission gate
  nvcc --version
  rustc --version
  export UOR_NVCC_ARCH=compute_120
  cargo test --release --locked -p uor-r4-discrete --no-run
  cargo build --release --locked -p uor-r4-discrete --bin discrete-stack
"
```

#### Step 4.2.3 — Execute Finite GPU Qualification Suites (`C1..C11`, `F1..F6`, `Relation C1..C8`, `C-D1..C-D8`, `C-R1..C-R6`), `G-Pre`, and 100-Batch `K=16` Timing (`uor-pod run`, GPU 0)

```bash
scripts/pod/uor-pod run "${POD_ID}" --lab "${UOR_LAB}" --session "${SESSION}" --gpu 0 -- "
  set -euo pipefail
  cd /workspace/uor-r4
  export UOR_NVCC_ARCH=compute_120
  OUT_DIR=\"/workspace/uor-r4/evals/20261015T000000Z-dr-qualification-${REVIEWED_SHA:0:9}\"
  mkdir -p \"\${OUT_DIR}\"

  # 1. Existing finite GPU suites (zero regression check)
  ./target/release/discrete-stack qualify          > \"\${OUT_DIR}/qualify.json\"
  ./target/release/discrete-stack qualify-flock    > \"\${OUT_DIR}/qualify-flock.json\"
  ./target/release/discrete-stack qualify-relation > \"\${OUT_DIR}/qualify-relation.json\"
  ./target/release/discrete-stack qualify-driver   > \"\${OUT_DIR}/qualify-driver.json\"

  # 2. C-R1..C-R6 GPU fixture suite + host static ratchets
  cargo test --release --locked -p uor-r4-discrete -- --nocapture \
    | tee \"\${OUT_DIR}/cargo-test-cuda-release.txt\"

  # 3. G-Pre zero-update checkpoint-0 ceiling evaluations (seed 1 & seed 2, K=8 & K=16)
  # (Runs the Section 2.2 init-corpus + eval-corpus sequence into \${OUT_DIR}/g-pre-*)

  # 4. 100-batch K=16 synchronized timing benchmark (Regime 0 occ=16 & Regime Q4 occ=189, K16-A & K16-R, 0 apply)
  # (Writes \${OUT_DIR}/timing-k16-100batch.json)

  # 5. Seal output directory with SHA256SUMS
  (
    cd \"\${OUT_DIR}\"
    find . -type f ! -name SHA256SUMS -print0 | sort -z | xargs -0 sha256sum > SHA256SUMS
    sha256sum -c SHA256SUMS
  )
"
```

*(If total wall time approaches 25 minutes, run `scripts/pod/uor-pod renew "${POD_ID}" --lab "${UOR_LAB}" --session "${SESSION}" --hours 1` and `UOR_LAB="${UOR_LAB}" uor-harness heartbeat -m m1` before the 30-minute mark.)*

#### Step 4.2.4 — Laptop Retrieval, `SHA256SUMS` Verification, and `cloud-store put` MD5 Round-Trip (Before Pod Teardown)

```bash
RUN_NAME="20261015T000000Z-dr-qualification-${REVIEWED_SHA:0:9}"
LOCAL_STAGE="${HOME}/uor-r4-local/${RUN_NAME}"
mkdir -p "${HOME}/uor-r4-local"

# 1. Copy sealed result directory from pod to laptop gitignored staging
SSH_INFO="$(scripts/pod/uor-pod ssh "${POD_ID}")"
# Copy via scp using the host/port reported by uor-pod ssh:
scp -P <PORT> -i ~/.ssh/id_ed25519 -r \
  "root@<HOST>:/workspace/uor-r4/evals/${RUN_NAME}" \
  "${LOCAL_STAGE}"

# 2. Verify every file digest on laptop against pod-generated SHA256SUMS
(
  cd "${LOCAL_STAGE}"
  shasum -a 256 -c SHA256SUMS
)

# 3. Archive to shared iCloud store via cloud-store put (runs automatic MD5 download round-trip check)
~/.local/share/uor-r4/bin/cloud-store put "${UOR_LAB}" "${LOCAL_STAGE}"
# Expected output before proceeding:
#   stored 20261015T000000Z-dr-qualification-<sha> (<bytes> bytes, md5 <32-hex-md5>) at icloud:UOR-R4/results/<lab>/20261015T000000Z-dr-qualification-<sha>.tar
# Verify index TSV entry exists:
~/.local/share/uor-r4/bin/cloud-store list "${UOR_LAB}" | grep "${RUN_NAME}"
```

#### Step 4.2.5 — Pod Lease Release & Termination (Strictly After Step 4.2.4 Succeeds)

```bash
# 1. Release GPU lease in state.json
scripts/pod/uor-pod release "${POD_ID}" --lab "${UOR_LAB}" --session "${SESSION}"

# 2. Terminate pod if no other lab holds an active lease on it
scripts/pod/uor-pod down "${POD_ID}" --lab "${UOR_LAB}" --session "${SESSION}"

# 3. Confirm 0 leaked leases / pods
scripts/pod/uor-pod status
```

---

## 5. Explicit `NOT_TRAINING` Scope & Post-Run Zero-Update Verification Checklist

### 5.1 Static Source Guarantees at `2fa663b54`

1. **`crates/uor-r4-discrete/src/lib.rs:189-250` (`apply_call_sites_remain_bounded_to_qualified_paths`):**
   - In `src/main.rs`: **zero** `.apply(` calls (`assert_eq!(count_apply_calls(MAIN_SOURCE), 0)`).
   - In `src/qualification.rs`, `src/flock_qualification.rs`, `src/relation_qualification.rs`, `src/measurement.rs`, `src/objective.rs`, and `src/completion_baseline.rs`: **zero** `.apply(` calls.
   - In `src/driver_qualification.rs`: bounded to $\le 11$ disposable 2-row synthetic fixture `.apply(` calls (`assert!(driver_qualification_apply_calls <= 11)`), which never touch `fineweb-edu-v4096-100m-v2`.
   - In `src/training.rs`: the single corpus `.apply(` call (`training.rs:582`) is gated behind `CorpusUpdateAdmission` (`training.rs:1165`), which requires both `--owner-authorization` (`uor-r4.owner-training-authorization/1` with `authorized: true`) and `--admission` (`uor-r4.corpus-update-admission/1` with `training_hold_released: true`) and otherwise refuses before batch `0` with `Err(Error::TrainingHeld)`.
2. **`eval-corpus` hard-refuses any updated checkpoint (`src/main.rs:84-86`):**
   ```rust
   if checkpoint.cursor.accepted_updates != 0 {
       return Err(Error::TrainingHeld);
   }
   ```

### 5.2 Mandatory 7-Point Post-Run `NOT_TRAINING` Audit Checklist

Before publishing the qualification receipt PR and `D-Packet`, the executor must verify all 7 invariants below across every file in `${LOCAL_STAGE}`:

| # | Audit Invariant | Exact Verification Command / Field Check | Required Result |
|---|---|---|---|
| 1 | **Zero corpus updates accepted** | `jq '.cursor.accepted_updates, .cursor.completed_batches' ${LOCAL_STAGE}/g-pre-*/checkpoint-00000.json` | Every value `== 0` |
| 2 | **Zero updates applied by evaluation** | `jq '.updates_applied_by_evaluation, .evaluation_mutates_model' ${LOCAL_STAGE}/g-pre-*-eval/evaluation-00000.json` | `0` and `false` on all 6 `G-Pre` evaluations |
| 3 | **Model epoch stays `0` across all `G-Pre` and timing runs** | `jq '.model_epoch' ${LOCAL_STAGE}/g-pre-*-eval/evaluation-00000.json` and timing report `pre_epoch, post_epoch` | `0` everywhere |
| 4 | **Pre- vs post-benchmark snapshot digest identity** | Compare `pre_model_snapshot_sha256` and `post_model_snapshot_sha256` across all 4 `100`-batch timing cells (`T-K16A-OCC16`, `T-K16R-OCC16`, `T-K16A-OCC189`, `T-K16R-OCC189`) | `pre_model_snapshot_sha256 == post_model_snapshot_sha256` (`0` bytes mutated across `400` timed `propose` batches) |
| 5 | **Zero training authorization / admission manifests present** | `find "${LOCAL_STAGE}" -name '*owner*.json' -o -name '*admission*.json'` | Empty (`0` files; `run-corpus` is never invoked) |
| 6 | **Hold status preserved in every emitted report** | `jq -r '.hold_status.training, .hold_status.stage_b, .hold_status.m2_rental' ${LOCAL_STAGE}/qualify-driver.json` | `"HELD"`, `"HELD"`, `"HELD"` |
| 7 | **MD5 archive round-trip verified before pod release** | `~/.local/share/uor-r4/bin/cloud-store list "${UOR_LAB}" \| grep "${RUN_NAME}"` | Present in `icloud:UOR-R4/index/${RUN_NAME}.tsv` with matching byte size and MD5 digest |

---

## 6. Verification Executed for This Runbook Record (Host-Only, Zero Compute)

- `CARGO_TARGET_DIR=/Users/casey.allard/uor-r4/local/harness-target cargo test --locked --offline -p uor-r4-discrete --no-default-features`: **37 passed, 0 failed, 1 ignored** (confirms `driver_source_sha256() == 3487506d0d6b9197e8b99963dfe92e91b5c3e33e0801d9116bf525cdc9b17fb7`, `device::source_sha256() == f679f8736500925014ff721ab035f3c284446baa6af0763f27b09bc388763008`, `objective.rs` `C-R1..C-R6` contract tests, and `lib.rs` `.apply(` call-site census).
- `uor-harness ratchet d23`, `uor-harness ratchet d25`, `uor-harness check-formats`, `uor-harness shard-check`, `python3 scripts/check_claim_wording.py`: all pass.
