# ORDER: d11-exporter-float-trunk, claude-141 (a deterministic FBS backward, then a bit-for-bit repeat)

Lab: claude. Item: [d11-exporter-float-trunk](../../../../roadmap/items/d11-exporter-float-trunk.md) (#2820). Base: `3b0c0059e`.

## Goal

**Give the D11 training a backward pass whose result does not depend on GPU thread order. Then show two runs of one binary match bit for bit.**

claude-140 (#3315) found that the same binary, run twice on one device, trains to different models (outcome (a)). The prime suspect is the float `atomicAdd` gradient sums in `spiralcore_fbs_bwd` (`spiralcore_fbs.cu:151`, `:178`). The D11 export arm `OneAtomActive` runs that pinned kernel. The atomics in `spiralcore_fbs_arms.cu` serve only the TwoAtomMix and IdentityTransport arms (`spiralcore_fbs_arms.rs:71-76`).

**The two scatters, and how a gather replaces them.**
- **`d_input`:** contributions collide across outputs within a row.
- **`d_queries`:** contributions collide across rows for one output.
- **Why a gather works:** an output's selected and alternative routes depend only on its queries, never on the row. So each sum can be computed as a gather in a fixed order, with no atomics:
  - `d_input[row]` is summed over outputs in index order;
  - `d_queries[output]` is summed over rows in index order.
- **The math is the same:** the same terms, added in a fixed order.

**The change.**
1. **A new kernel module,** `crates/uor-r4-float-train/src/spiralcore_fbs_det.cu`, built as its own nvcc PTX module through `build.rs`, with no NVRTC. It holds the deterministic backward.
   - The pinned `spiralcore_fbs.cu` is not touched, so `PINNED_SPIRALCORE_FBS_CU_SHA256` and every earlier receipt stay valid.
2. **An opt-in switch,** `--deterministic-bwd` on the `spiralcore-fbs-sheet` training modes, default off.
   - When it is on, the `OneAtomActive`/`FrozenQueries` backward uses the new kernel.
   - The receipt records `deterministic_bwd` and the new module's source and PTX sha256 in its `kernels` block.
3. **Tests.**
   - **Host:** the switch's plumbing and receipt fields.
   - **GPU** (`--features cuda`, which runs in the pod's `bridges` step):
     - the deterministic kernel matches `kernel_controls::spiralcore_fbs_bwd_cpu` within the existing kernel tolerance;
     - it gives bit-identical outputs when run twice;
     - its error codes match the pinned kernel's.
4. **A GPU check, under its own prep sheet with FIT and D67 review.** One pod, one binary built at this branch's head, `--deterministic-bwd`, claude-140's R1/R2 recipe: 1,024 dual steps, seeds 1 and 2, the same pinned inputs.
   - **Pass:** R1 and R2 are bit-identical: every saved model file's sha256, and every evaluation point.
   - **Fail:** another source of nondeterminism remains, for example candle's own CUDA kernels. That is recorded and localised next.

Making the switch the default is a later decision, and is not this unit's.

## Cancel probe (run first)

Cancel if a deterministic FBS backward already exists, or another open order changes these files.

Result (2026-10-27T03:40Z, `origin/main` at `3b0c0059e`):
- `git grep -i deterministic -- crates/uor-r4-float-train` finds only fixture and RNG comments, and no deterministic backward.
- The open PRs are #3319 (kimi, `render`) and this unit's #3320. Neither touches the float-train crate.

**Not cancelled (instrument: none needed).**

## Files in scope

- `crates/uor-r4-float-train/src/spiralcore_fbs_det.cu` (new)
- `crates/uor-r4-float-train/build.rs`
- `crates/uor-r4-float-train/src/spiralcore_fbs.rs`, `geometric_stack*.rs` (`cuda_ops`) and `kernel_controls.rs` (tests)
- `crates/uor-r4-float-train/src/spiralcore_fbs_sheet.rs` and `src/bin/spiralcore-fbs-sheet.rs` (the switch and receipt)
- `docs/harness/prep/d11-exporter-float-trunk-deterministic-bwd-2026-10-27.md`
- `docs/research/notebook/m4-serving.md` (one row)
- this task folder

## Done when

1. **The new kernel and switch build.**
   - `cargo check --features cuda --all-targets -p uor-r4-float-train` passes with a stub nvcc on the laptop.
   - The host test suite passes, as do fmt and clippy.
2. **The GPU tests pass on the pod,** in the `bridges` step: CPU-reference agreement, bit-identical repeat, and error-code parity.
3. **The C1 repeat with `--deterministic-bwd` has its outcome classed** by its prep sheet, under its own FIT and D67 review, sealed in the store.
4. **A REPORT, a notebook row, and a fresh exact-head review.** The required checks pass.

## Stop and ask if

Stop and decide; never wait.
- **The deterministic kernel cannot match the CPU reference** within the existing tolerance. Stop and record it; do not loosen the tolerance.
- **The repeat is not bit-identical.** Record which outputs differ and from which step. The next unit localises the remaining source.
