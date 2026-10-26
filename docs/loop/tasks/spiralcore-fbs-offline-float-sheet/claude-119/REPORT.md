# REPORT: spiralcore-fbs-offline-float-sheet, claude-119

Lab: claude. Order: [ORDER.md](ORDER.md). Sheet: [`spiralcore-fbs-offline-float-sheet-2026-10-25.md`](../../../../harness/prep/spiralcore-fbs-offline-float-sheet-2026-10-25.md) (antigravity-81, FIT in #3233). Record: [lab record](../../../../labs/spiralcore-fbs-offline-float-sheet/README.md).

Closes #2891.

## What did not work, first

- **`G-Fbs-Sheet-Fit` failed on both seeds, in both builds.**
  - **Top-1 against frozen routing.** Learned SpiralCore/FBS routing loses to frozen random routing (`Fbs_FrozenQueries_Null`) on paired float DEV top-1 in all 8 comparisons:

    | Arm | A s1 | A s2 | B s1 | B s2 |
    |---|---:|---:|---:|---:|
    | `Fbs_1Atom_Active` | −3,993 | −1,851 | −4,920 | −2,267 |
    | `Fbs_2Atom_Mix` | −4,334 | −3,000 | −5,098 | −2,685 |

  - **Loss against frozen routing.**
    - On seed 1, `Fbs_1Atom_Active` has lower DEV loss than the frozen null, but only inside the noise band: 0.0557 against a band of 0.0686 in A, and 0.0423 against 0.0597 in B.
    - On seed 2 it has higher loss.
  - **Loss across arms.** The identity-transport control has the lowest float DEV loss in all four seed-build cells (1.8030–1.8290).
  - **Verdict.** A negative for this configuration only: 6 layers, 1,024 steps, `StackAdamW` at the Hamilton sheet's rates (CHARTER §4.1). SpiralCore/FBS stays ACTIVE in its integer form and READY in float.
- **`G-Fbs-ExportGap` failed in both builds, on its top-1 part.**
  - **The loss part passes.** The one-atom arm's integer-minus-float DEV loss gap is 0.0080–0.0264, inside the band.
  - **The top-1 part fails in build A (proof).** The top-1 gaps are 184 and 481 rows. The hit counts allow F_I of at most 117 and 166, since b + c cannot exceed float hits plus integer hits.
  - **Build B records a FAIL, but which seed failed can't be recomputed.** Its net gaps are 25 and 79 rows. The receipt does not store the export gap's b and c. That is a receipt gap, not a gate gap.
- **Trained-query arms do not reproduce between two byte-identical builds.**
  - Both builds use the same binary, `bb88f774…`, and the same seeds, yet the results hashes differ (`d6b2d993…` against `ee0fd4aa…`).
  - Trained-query top-1 moves by up to 2,212 rows. For example, the identity control on seed 2 scores 2,533 against 321. The frozen null moves by at most 280.
  - The one-atom-versus-identity top-1 comparison changes sign between builds, so this run gives no transport verdict on top-1.
  - **Hypothesis.** Atomically accumulated adjoints flip hard root choices. Step 0 is identical in both builds.
- **Late collapse on seed 1 (measurement; the cause is a hypothesis).** The trained-query arms' top-1 and admitted-target hits peak at step 512 and fall by step 1,024 while loss falls. For example, the identity control goes from 1,548 to 152 top-1, and from 2,118 to 256 admitted. Antigravity's recurrence sheet showed the same pattern.
- **`G-Fbs-Transfer` was not reached,** because the fit and export bars failed. Separately, the sealed `fresh_v2` draw is not in the `w2-inputs/2026-10-25/driver-data/` store bundle. The runner records a missing draw as blocked, never as a negative.
- **Attempt 1 stopped at the GPU tests (instrument fault, repaired).** The frozen-arm test demanded bit equality of an input adjoint that is summed with device atomics, and the values differed only in the last bit. The forward was bit-identical, and the other three arm tests passed. `ca057242` compares with the 1e-5 relative tolerance the two-path test already used. Attempt 1's logs are in the store under `attempt1/`.
- **The notebook row is staged, not appended.**
  - The installed `~/.cargo/bin/uor-harness` was built at 2026-10-25T01:10Z, before #3194. Its guard therefore refused `docs/research/notebook/float-training.md`, which docs/loop/README.md lists as always allowed.
  - The row is in [NOTEBOOK-ROW.md](NOTEBOOK-ROW.md), ready to append.
  - It is logged in `local/claude/FRICTION.md`, with one note for Casey on #2028.

## What changed

- **Code (about 1,580 lines, `crates/uor-r4-float-train`):**
  - **Arm kernels.** `src/spiralcore_fbs_arms.cu` and `src/spiralcore_fbs_arms_prelude.cu`, built as their own nvcc PTX module (`spiralcore_fbs_arms_f32.ptx`, `build.rs`). They provide identity transport, and two paths combined with a constant 1/√2.
  - **Arm hook.** `src/spiralcore_fbs_arms.rs`: the arm hook and the projection op. The frozen arm uses the standard kernel with `dq = 0`.
  - **Stack changes.** `StackModel::linear` dispatches through the hook. `widen_fbs_queries_two_path` builds the two-path queries.
  - **Runner.** `src/spiralcore_fbs_sheet.rs` and the bin `spiralcore-fbs-sheet`, with modes `synthetic-probe`, `throughput-probe` and `full`.
  - **Gate wrapper.** `hamilton_sheet::with_gate_mode`.
- **The D68 conditions held.**
  - The three §5-pinned files are byte-identical, and were re-hashed on the pod.
  - The receipt records the arm kernel, prelude and PTX sha256.
  - There are no new dependencies and no softmax, squashing or matmul. The kernels are built ahead of time by nvcc.
- **Tests (about 330 lines).**
  - Four `#[ignore]`d CUDA tests pin each arm against a reference or an exact invariant: identity selection, √2 × standard on equal paths, frozen credit 0 with the standard input adjoint, and adjoint-transpose on every arm.
  - The host tests cover arm names and codes, the per-thread hook, the kernel inventory, F_I, the schedule, the pins and synthetic batches.
- **Docs:**
  - the lab record and four receipts;
  - the staged notebook row;
  - a dated row and updated open questions on `docs/mechanisms/spiralcore-fbs.md`.
- **Generated:**
  - receipts of about 3,690 JSON lines;
  - the roadmap item set to `status: done`, and `docs/roadmap/README.md` re-rendered.
- **Size against estimate.** The ORDER gave no line estimate. It anticipated that the build might fill one PR. It did not, and the run landed in this unit.

## Measurements

All of these come from one RTX 5090: GPU 1 of `vast-57710022`, `sm_120`, driver 595.80, nvcc 12.8.93. They were produced by `/workspace/fbs-job.sh ca0572423f472c9f2b01d60a4f50cb17261281e9` (local copy `local/claude/fbs-job.sh`, sha256 `f2231b40…`). That job runs, in order:
1. the pins;
2. the store inputs;
3. build A, then `cargo test --release --locked -p uor-r4-float-train --features cuda --lib -- --include-ignored spiralcore_fbs`;
4. `spiralcore-fbs-sheet --mode synthetic-probe`, then `--mode throughput-probe`, then `--mode full` on build A;
5. a clean build B, then `--mode full` again.

| Measurement | Value |
|---|---|
| Training (batch 4 × context 192, 1,024 steps) | 20,643–23,929 tok/s |
| Float DEV evaluation (14,287 targets, 82,176 tokens) | 110,864–134,857 tok/s |
| Integer-export-mode DEV evaluation | 13,815–14,531 tok/s |
| Greedy decode (batch 1, 128-token prefix, 64 tokens; full recompute per token, a lower bound) | 109.1–139.5 tok/s |
| Float DEV loss, step 0 → step 1,024 | 2.6487–2.8450 → 1.8030–1.9250, every arm |
| Route mobility (trained-query arms) | 5.46%–7.95%; last-step churn ≤ 0.0004 |
| Noise band (largest two-seed loss difference; the frozen null in both builds) | 0.0686 (A), 0.0597 (B) |
| GPU tests matching `spiralcore_fbs` | 18 of 18 pass |
| `I-Export-Exactness` | 48 of 48 lawful blocks; 0 of 3,072 integer mismatches |
| Q12 `i32` writer oracle | 0 mismatches on 517 rows × 32 evaluations per build |
| GPU time | about 34 min on one leased GPU of antigravity's pod; about $0.62 |

The receipts are [`receipt.json`](../../../../labs/spiralcore-fbs-offline-float-sheet/receipt.json) (`17c5563b74f3ccd9400b5a05ae042f844d2f0e0af2f0b6ee17acc84e95edb789`) and [`receipt-build-b.json`](../../../../labs/spiralcore-fbs-offline-float-sheet/receipt-build-b.json) (`d3cf226db091006a79226fdeb5529ae11206391998328e482dd02a0c56e18443`). The store copy is `caseyallard/uor-r4-store:results/claude/spiralcore-fbs-offline-float-sheet-2026-10-25/`, with `SHA256SUMS` `17939779656db70ae40409cf06ea64a705ffd0ebe94d6822747b3dc83c759320`. In that folder, `attempt2/job.log` differs from its listed sum only by the closing `DONE` line, which was written after the sums.

**Open choices, labelled:**
- The integer-export mode is the Hamilton sheet's D11-compatible evaluation. It snaps the Hamilton gates to the 120 icosians, and rounds the writer table and final hidden state to Q12. The rest of the stack stays float, and the two-atom arm's 1/√2 is applied in float.
- `I-Export-Exactness` covers the one-atom forward against the pinned crate's integer signed-axis table, not the discrete crate's kernel.
- The Hamilton sheet's learning rates, weight decay and clip are reused.
- Training uses 1,024 batches.
- Route mobility is measured as the change in each query block's nearest orbit root.
- The paired fit comparisons use float predictions.

## Checks run at the head

Filled in by the exact-head review record: `loop report --check`, `loop pr-check --local`, the host tests, the stub-nvcc `--features cuda` clippy, and CI.

## Skills used

- **holding-the-goal.** The frozen-arm test failure and the stale guard were each read before acting. The first was classed an instrument fault, and the second as the guard working from an old binary. Neither was routed around.

## Still open

- **Append the staged notebook row** once the shared harness binary is current (FRICTION line, #2028 note).
- **Make the FBS backward deterministic** (an ordered reduction in place of atomics) before any further float comparison of trained routes. This is a hypothesis, not yet tested.
- **Test whether the margin-hinge loss and the writer's admission come apart late in training.** This is a hypothesis.
- **Store the export gap's b and c in the receipt,** so a reader can recompute the top-1 export bar per seed.
- **Add `fresh_v2` to the `w2-inputs` store bundle** before any sheet that reaches its transfer gate.

## Plain summary

SpiralCore/FBS is the project's replacement for matrix multiplication in a layer's projections. Each output block picks one 8-number input block by walking a small tree of fixed geometric points, then flips and permutes its entries. This unit trained a small language model that uses it, in ordinary floating point and offline, as a pre-registered test allows. It compared four versions:
- the standard one;
- one that mixes two picked blocks;
- one that picks but never flips or permutes;
- one whose picking is frozen at random.

Every version learned: its error on held-out text fell. Training ran at about 21,000–24,000 tokens per second on one gaming-class GPU. But learning where to pick did not beat the frozen random picks on the main accuracy count, and rounding the model to integers moved that count too much.

There is a deeper problem. Two identical builds of the same program gave noticeably different results whenever the picks were being learned. That fits tiny rounding differences in the GPU's order of addition flipping picks. So the next step is to make that addition order fixed before comparing learned picks again. This closes this one training setup, not the idea.
