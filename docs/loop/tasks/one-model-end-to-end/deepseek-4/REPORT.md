# REPORT: one-model-end-to-end, served graph wiring

Lab: deepseek. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md).
Base: `2f8f665c5`. Order: [ORDER.md](ORDER.md).

## What did not work, first

- **A direct CPU tensor forward could not serve as an integration control.** The
  first probe called the served Q12 path on a CPU tensor and was refused by
  D24/D29 before arithmetic. The accepted test checks the complete bridge state
  on CPU; exact tensor forward/backward remains a CUDA control.
- **No CUDA device is available on this workstation.** The integrated finite
  loss/gradient test is updated but was `NOT_RUN`. No training run, ranking
  comparison, export-gap reading or capability claim is made.
- **Checkpoint evaluation previously escaped a failed round trip.** The old
  checkpoint closure restored rounded parameters only after successful
  evaluation. The new wrapper restores them after both success and error.

## What changed

- `run_long` now keeps `with_served_surrogate_training` active around the real
  loss forward and its backward when `--train-as-served 1` is set. The ordinary
  Hamilton gate mode remains the off path.
- The geometric MLP applies the tested dynamic i16 activation quantizer between
  the Hamilton product and `mlp_down` in served mode.
- Scored training queries, DEV nearest picks, panel loss and generated replies
  use the Q12 query bridge while the served graph is active. Writer-mode
  generation also applies that query before the native writer.
- Checkpoint DEV evaluation, panel loss, nearest/writer readings and sample
  generation all run through the same Q16/i16/D11/Q12 graph before the rounded
  checkpoint is saved.
- Served mode now refuses `--turn-memory-dropout` and
  `--reply-read-train`, whose train-only row selections have no serving
  counterpart.
- Updated the CLI and configuration documentation to describe the actual graph
  rather than parameter rounding alone.

## Measurements

No model run was possible on the available CPU backend. The host-visible
integration readings are:

- served graph closure: D11 bridge, i16 activation bridge and Q16 residual
  rounding all enabled together, then all restored off;
- ordinary graph closure: all three remain off;
- all nine existing served-mode refusal controls pass, including new memory
  dropout and reply-cut cases;
- full float library suite: 208 passed, 0 failed, 0 ignored.

The CUDA-gated three-step served test now runs the full surrogate graph rather
than rounded parameters over a plain float forward. It remains the required
finite-loss/gradient and export-invariance check when a GPU is available.

## Checks run at the head

- `cargo test -p uor-r4-float-train --release
  the_trainer_selects_and_restores_the_full_served_graph`: passed.
- `cargo test -p uor-r4-float-train --release
  training_as_served_refuses_what_the_export_does_not_carry`: passed.
- Full `uor-r4-float-train --release --lib` suite in a fresh isolated target:
  208 passed, 0 failed, 0 ignored.
- Crate-local strict Clippy still reports the same pre-existing findings in
  `fbs_export.rs`, `flock_read_memory.rs`, `flock_read_register.rs` and
  `in_space_long.rs`; this change adds no reported finding.
- The inserted source regions have no rustfmt diff. File-wide rustfmt still
  reports pre-existing formatting in unrelated lines. `git diff --check` and
  `uor-harness loop report --check --order` passed.
- Loop report/pr checks and exact-head review are recorded in the PR review
  record.

## Skills used

No repository skill was invoked. The unit reuses the existing served
surrogate primitive and gate-hook state model.

## Still open

- Run the CUDA finite integration test and matched four-arm training
  comparison on GPU hardware.
- Establish forward parity between this shadow and the exact integer/audited
  artifact on the selected trained checkpoint.
- Formalize served semantics for memory dropout and reply-row cuts before
  allowing them in this mode.
- The completed CPU training paths remain separate exact-integer trainers under
  D24/D29; this unit does not claim the float training loop runs on CPU.

## Plain summary

The trainer already had exact quantization blocks and a switch that rounded
parameters to the values an exported integer model would contain. This unit
connects those blocks to the real training graph: the hidden stream is held on
the Q16 grid, MLP activations take the same dynamic i16 form as serving, gates
and transitions use the discrete D11 choices, and the scored query is rounded
to Q12. The same mode is used for checkpoint scoring and generation, so a saved
checkpoint is not selected under a different graph. Training-only memory masks
are refused because serving cannot reproduce them. The local CPU verifies state
and refusal behavior, while the actual float training computation still
requires CUDA.
