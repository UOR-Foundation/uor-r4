# ORDER: m3-memory-turn-discriminator-gate, antigravity-63

Lab: antigravity. Item: [m3-memory-turn-discriminator-gate](../../../../roadmap/items/m3-memory-turn-discriminator-gate.md) (#2855). Base: `14fc2cbe4`.

## Goal

The item's done-when reads:

> The last clause of the M3 bar in `docs/integration/project-track.md:116-120`: non-memory conversational prose does not silently overwrite stored facts. It is measured on the pre-registered panel.

Per the lab decision on `#2856` (`6394013509`) and `claude-102` (`#3201`), `SavedCompiler` is superseded (`SparseSoftmax` is banned by CHARTER §3.2), and the lawful prime-sieve compiler baseline (`claude-94`, `#3188`) scored `false_assert = 0/50`, `false_overwrite = 0/50`, `clean_unresolved = 50/50`, `R1 = 462/462`, `R2 = 0/220`. `claude-102`'s first marker-gate configuration (`#3201`) closed negative (D22) because:
1. `R1` dropped to `406/462 (87.9% < 95%)` due to same-expert nested-span disagreements on 7 correction templates whose training frame carries an extra word (`"now"` or `"really"`) absorbed into the marker reading's value slot;
2. `R2` stayed at `0/220` because leading-prefix cofactors within identical-suffix frames of one relation did not factor training frames into relation cores and cross-relation / circumfix cofactors.

This unit builds and GPU-evaluates the next lawful prime-sieve discriminator-gate configuration derived strictly from `Split::Train` frames (`FrameTable::world_v2()`), resolving same-expert nested spans by maximal frame explanation and factoring training frames into relation cores and training-derived cofactors/templates under an exact-explanation and expert-agreement margin gate.

## Cancel probe (run first)

**Result at `14fc2cbe4` (classed `mechanism`, not `instrument`):**
- Inspected `crates/uor-r4-training/src/sieve_marker_gate.rs`, `crates/uor-r4-training/src/sieve_frame.cu`, `crates/uor-r4-training/src/bin/prose-panel-gate.rs`, and `docs/loop/tasks/m3-memory-turn-discriminator-gate/claude-102/REPORT.md`.
- `cargo test --no-fail-fast -p uor-r4-training --lib sieve_marker_gate` passes (`2/2`), confirming the instrument and host test harness are healthy.
- The two failures in `claude-102` (`R1 = 406/462` from same-expert nested-span refusal in `sieve_frame.cu:182`, and `R2 = 0/220` from restricting `MarkerTable::derive` to prefix-only same-suffix pairs within one relation) are configuration limits of that rule, not instrument bugs.

## Files in scope

- `crates/uor-r4-training/**`
- `docs/compute/**`
- `docs/labs/m3-memory-turn-discriminator-gate/**`

## Done when

1. `crates/uor-r4-training/src/sieve_marker_gate.rs`, `crates/uor-r4-training/src/sieve_frame.cu`, and `crates/uor-r4-training/src/bin/prose-panel-gate.rs` implement the refined training-derived prime-sieve gate and kernel (including all open review items from `claude-102`: nested-span same-expert control, `sig_words > SIEVE_MAX_SIG_WORDS` status write, `gate_frames`/`device_markers` unit tests, and `hole_len == 0` check on `ACT_QUERY`), with pinned table digests and zero squashing/matmul/discrete-float regressions (`d23`, `d25`, `d32`).
2. A one-page pre-registration sheet in `docs/labs/m3-memory-turn-discriminator-gate/` receives a fresh-context non-author `TEST FITNESS: FIT` review and a fresh-context D67 pre-run `GO` review on `#2855` before any pod launch.
3. The GPU run on one leased RTX 5090 executes finite CUDA controls, D29 PTX checks (`sieve-cuda` and `cuda`), and two-build determinism on the frozen prose panel, `R1`, and `R2`, with artifacts uploaded to `caseyallard/uor-r4-store` before `uor-pod down`.
4. If the registered bar (`false_assert = 0/50`, `false_overwrite = 0/50`, `clean_unresolved = 50/50`, `R1 >= 95%`, `R2 > 0/220`) is met, set `m3-memory-turn-discriminator-gate` to `status: done` with `Closes #2855`; if missed, record the configuration negative (D22) with `References #2855`.

## Stop and ask if

- Any derivation reads development phrasings or panel rows instead of `Split::Train` (`FrameTable::world_v2()`): stop and restrict strictly to `Split::Train`.
- Any finite CUDA control or D29 check fails on the pod: classify as instrument fault, repair, and re-verify before measuring.
