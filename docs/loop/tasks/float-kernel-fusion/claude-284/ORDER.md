# ORDER: float-kernel-fusion, claude-284

Lab: claude. Item: [float-kernel-fusion](../../../../roadmap/items/float-kernel-fusion.md). Base: `bff50c993`.

## Goal

The item's done-when reads:

> 1. **The kernel list is generated.**
>    - `build.rs` scans each `MODULES` entry's combined `sources` (`build.rs:26-54`) for `extern "C" __global__ void <name>` on every build, CPU builds included. It writes one list per PTX module, which `lib.rs` includes. The arms module is built from three files, `spiralcore_fbs_arms_prelude.cu`, `spiralcore_fbs.cu` and `spiralcore_fbs_arms.cu` (`:44-53`). So its list is the 4 FBS kernels plus `spiralcore_fbs_arm_fwd` and `spiralcore_fbs_arm_bwd`.
>    - The dispatch refuses a name not in its own module's list. The arms kernels, launched through `launch_prebuilt` (`geometric_stack_native_cuda.rs:798-805`), must pass that check, not fail it.
>    - No literal kernel count is left in code, tests or their doc comments: `lib.rs:3-4, :33-35`, `tests/no_training_crate_dep.rs:3-4`, and the messages at `:83` and `:104`.
>    - The test re-scans the sources itself and compares with the generated lists. The banned-token scan stays.
> 2. **Fused `activation_quantize16`,** behind a flag, equal bit for bit to the op chain it replaces. The flag is a thread-local in `hamilton_sheet.rs`, where the served wrappers switch to the kernel, set by a `train-in-space` option. It is equal:
>    - forward and backward, on random rows that include zeros, −0, saturating rows, subnormals and exact .5 ties;
>    - inside one training step at P2b's recipe: the loss and every gradient.
>    The order of the bridge's ops (`free = x · inside`, `d = q − free`, `out = free + d`) and nvcc's `--fmad` choice are part of the gate.
> 3. **Fused `q16_residual`,** the same way.
> 4. **The cost,** on an RTX 5090 at P2b's recipe: kernel launches and seconds per training step, chain against fused, by the trainer clock (and `nsys`, if the pod has it). Also the seconds per cached decode step at 2,048, set against claude-279's estimate of 8–9 ms.
> 5. The flag defaults on only when 2 and 3 hold. One notebook row and a REPORT.

## Cancel probe (run first)

To fill in.

## Files in scope

- `crates/uor-r4-float-train/build.rs`
- `crates/uor-r4-float-train/src/lib.rs`
- `crates/uor-r4-float-train/src/cuda_stack.cu`
- `crates/uor-r4-float-train/src/cuda_stack_kernels.rs`
- `crates/uor-r4-float-train/src/fused_quantize*.rs`
- `crates/uor-r4-float-train/src/hamilton_sheet.rs`
- `crates/uor-r4-float-train/src/bin/train-in-space.rs`
- `crates/uor-r4-float-train/tests/no_training_crate_dep.rs`
- `crates/uor-r4-float-train/tests/twenty_four_kernel_controls.rs`
- `crates/uor-r4-float-train/tests/fused_*.rs`
- `crates/uor-r4-float-train/Cargo.toml`
- `docs/harness/prep/float-kernel-fusion*`
- `docs/loop/tasks/float-kernel-fusion/**`
- `docs/research/notebook/float-training.md`
- `docs/roadmap/items/float-kernel-fusion.md`

## Done when

To fill in.

## Stop and ask if

To fill in.
