# REPORT: CPU counterfactual parameter groups, deepseek-2

Lab: deepseek. Item: [cpu-shared-vector-train](../../../../roadmap/items/cpu-shared-vector-train.md) (#3505). Base: `728cf3f6c`.

## What did not work

- **Single-coordinate screening was neutral on the first synthetic fixture.** Up to 1,024 one-coordinate candidates over 128 causal samples accepted no normalization, bias, decay or route move. This remains configuration evidence for using coordinated proposals; it is not a mechanism negative and no acceptance rule was weakened.
- **Random admitted samples were also neutral at a wider block.** A K=64 probe over the first 64 admitted samples found no accepted move in any group. The positive controls below therefore construct a target from the query produced by a known legal candidate, then require that same candidate to pass the trainer's ordinary full-forward decision path.
- **The first exact-head review caught an acceptance-policy defect.** The initial implementation let an equal-margin rank improvement count as better. That contradicted the ORDER and BVG-K's strict loss rule; the repaired implementation rejects every equal-margin move and uses coordinate ordinal only to break ties among equal best losses.
- **This is a finite component result.** The model is a width-64, one-layer, 4,096-token synthetic fixture. It is not a corpus-training rate, saved-model parity result, language result or replacement for GPU qualification.
- **Candidate scoring is serial.** The benchmark measures the exact implementation, including artifact parse and full-forward reconstruction per candidate. It is a correctness ceiling, not a tuned CPU training rate.
- **One group trains at a time.** Cross-kind bundles combining normalization, bias, decay and route coordinates remain outside this unit.

## What changed

- Added `CpuCounterfactualTrainer`, implementing the discrete BVG-K decision shape:
  - a deterministic schedule selects at most 64 distinct table entries;
  - each coordinate scores its legal positive and negative delta lanes against the unchanged artifact;
  - only coordinates with a strictly lower exact `margin_loss` and preserved target admission enter the eligible set; equal-margin rank changes are refused;
  - the lowest exact margin is the best single move, with the lower coordinate ordinal breaking ties;
  - the complete eligible block is committed only when its exact joint margin is strictly below the best single margin, otherwise the best member is committed.
- Added exact table patching for:
  - per-layer `rec_norm_gain`, `mlp_norm_gain` and final `final_norm_gain` grid codes;
  - per-layer `conv_bias` and `gate_bias`;
  - per-layer `decay_offset`;
  - all six per-layer FBS route tables.
- Normalization proposals remain inside the artifact's zero-or-nonzero-grid-code predicate. Route frame moves change only bits 16..21; route atom moves change only the low 16 bits and must remain inside each map's input-block count.
- Every single-member or joint proposal patches only named table spans, reparses the complete `UORLUT01` model, constructs a fresh changed forward, and evaluates the same sample. Hamming keys and all other derived state therefore cannot remain stale.
- `CounterfactualStep` records sample, target, coordinate count, candidate-evaluation count, eligible count, the complete accepted member list, best member, joint-versus-single decision, and before/after loss, admission, top-1, top-8 and rank.
- Checkpoint version 2 stores group, margin, delta, block size, seed, clocks, sample order, sample fingerprint and complete artifact bytes. Resume reproduces subsequent steps and checkpoint bytes.
- Added `cpu-counterfactual-bench` for candidate-evaluation throughput and peak RSS.

## Measurements

- `cargo test --release --locked -p uor-r4-cpu-train`: 54 passed, 0 failed.
- Constructed exact fixtures accepted a step for normalization, bias, decay and route. At least one accepted bundle contained more than one coordinate and exercised the joint-loss branch.
- Acceptance controls cover strict margin improvement, equal-margin rejection, full-forward admission loss, non-improving candidates for every group, malformed table spans, illegal grid codes and overflowing moves, no-op margins, route frame `0 -> 63` wrapping, route atom field isolation and bounds, deterministic block bounds, and checkpoint/resume.
- Release benchmark on the Ryzen 9 9950X, eight fresh K=16 steps per group:

| Group | Candidate evaluations | Elapsed | Evaluations/s | Accepted steps | Peak RSS |
|---|---:|---:|---:|---:|---:|
| Normalization | 256 | 329.866 ms | 776.073 | 0 | 7,336 KiB |
| Bias | 256 | 337.299 ms | 758.970 | 0 | 7,336 KiB |
| Decay | 256 | 329.771 ms | 776.297 | 0 | 7,336 KiB |
| Route | 480 | 607.556 ms | 790.051 | 0 | 7,336 KiB |

The zero accepted steps belong to the benchmark's unconstructed admitted sample. They measure evaluation cost only; the separate constructed fixture tests prove each legal update path can be accepted.

## Checks run at the head

- `cargo test --release --locked -p uor-r4-cpu-train`: PASS, 54 tests.
- `cargo clippy --locked -p uor-r4-cpu-train --all-targets --no-deps -- -D warnings`: PASS.
- `cargo fmt --package uor-r4-cpu-train -- --check`: PASS.
- `cargo run --locked --release -p uor-r4-cpu-train --bin cpu-counterfactual-bench`: PASS; readings above.
- D23, D25, D32 and the full loop gates are recorded after they run at the final exact head.

## Skills used

No repository skill was invoked. A read-only prior parameter-patch audit checked the UORLUT01 spans and legal table fields; a fresh adversarial exact-head reviewer checks the committed BVG-K implementation independently.

## Still open

- Corpus-level and multi-sample admission preservation.
- Exact saved-artifact and query-panel parity; the private trained artifact required by #3502/#3501 remains unavailable without store credentials.
- Hamming rotations, convolution taps and cross-kind candidates.
- Parallel candidate scoring and a pre-registered CPU corpus pilot.
- GPU qualification and language outcomes.

## Plain summary

The CPU shared-vector trainer can now propose, rerun and accept exact integer counterfactual updates to normalization gains, convolution and gate biases, decay offsets and every FBS route table. It follows the project's verified K-coordinate learning rule: coordinated moves must beat the best single exact move, and target admission cannot disappear. The unit is verified only on synthetic finite fixtures; useful learning still requires a corpus pilot and GPU-qualified evidence.
