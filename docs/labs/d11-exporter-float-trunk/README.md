# d11-exporter-float-trunk: the FBS artifact evaluator

This is the instrument of [d11-exporter-float-trunk](../../roadmap/items/d11-exporter-float-trunk.md), built in claude-128 (#2820). It is a GPU evaluator of a `UORLUT01` SpiralCore/FBS stack artifact. It exists so that the exported artifact itself, not an emulation of it, can be scored on the GPU panel (CHARTER §3.6).

- **`instrument/src/fbs_core.h`** is an integer-only port of the Rust session's FBS step (`stack_step_fbs`, its kernels and the native vector writer in `crates/uor-r4-integer/src/stack/`). It is compiled two ways:
  - by nvcc into the evaluator;
  - by the host C++ compiler into a twin that only qualifies the port. The twin never scores a panel.
- **The artifact's integers** come from `uor-r4-integer`'s `IntegerStackModel::fbs_image`. There is no second parser.
- **The crate is standalone,** outside the workspace (its own `[workspace]`), so CUDA and `unsafe` stay out of the serving crate.

## Build and test on the Mac

```bash
cd docs/labs/d11-exporter-float-trunk/instrument
CARGO_TARGET_DIR=<worktree>/local/target-instr ~/.cargo/bin/cargo test --target aarch64-apple-darwin
```

`tests/parity.rs` holds the host twin equal to `step_native`, bit for bit, at every position. It runs every single token of the three synthetic FBS controls (`uor_r4_integer::stack::synthetic`), and sequences that carry state for up to 64 positions.

The CUDA evaluator (the `cuda` feature) and its CLI come next in claude-128.
