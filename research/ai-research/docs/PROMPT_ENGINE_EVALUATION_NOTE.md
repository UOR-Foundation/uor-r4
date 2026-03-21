# Prompt Engine Evaluation Note

## 2026-03-20 Live Comparison Update

- Earlier audit work found that routed prompt engines were acting like shallow sidecars: routed prompts still included every routed layer plus the full `Observation anchor`, so routing mostly changed ordering and metadata rather than real context selection.
- The `routed_context_selection_v1` change made routed selection real. Routed modes now keep the top routed layer in full, keep the next routed layer as a summary, omit lower-priority layers entirely, and replace the full `Observation anchor` with a reduced observation summary.
- Live `compare-playable-slices` reruns used `openai-chat-completions` with `gpt-4.1-mini` and `--direct-provider-comparison-timeout-seconds 15`.
- The current comparison CLI still accepts only one angular variant per invocation, so two live comparison artifacts were required:
  - `angular-hopf-base` artifact totals: `baseline=0.60`, `angular-canonical::angular-hopf-base=0.65`, `legacy-router-backed::legacy-phase4d_hopf_transport=0.70625`
  - `angular-hopf-trans` artifact totals: `baseline=0.80`, `angular-canonical::angular-hopf-trans=0.775`, `legacy-router-backed::legacy-phase4d_hopf_transport=0.70625`
- Current leading routed variant in the richer live comparison artifacts is `angular-canonical::angular-hopf-trans`.
- The richer comparison path now shows routed context selection materially affecting results, but it does not yet show a stable overall routed win over baseline across both live comparison artifacts.

## 2026-03-20 Single-Artifact Live Comparison Update

- `compare-playable-slices` now supports multiple angular variants in one invocation, so one live artifact can include:
  - `baseline`
  - `angular-canonical::angular-hopf-base`
  - `angular-canonical::angular-hopf-trans`
  - `legacy-router-backed::legacy-phase4d_hopf_transport`
- Single-artifact live totals on `openai-chat-completions` with `gpt-4.1-mini` and `--direct-provider-comparison-timeout-seconds 15` were:
  - `baseline=0.65`
  - `angular-canonical::angular-hopf-base=0.65`
  - `angular-canonical::angular-hopf-trans=0.675`
  - `legacy-router-backed::legacy-phase4d_hopf_transport=0.70625`
- In the cleaner shared-baseline artifact, `legacy-router-backed::legacy-phase4d_hopf_transport` is the strongest routed variant overall.
- `angular-canonical::angular-hopf-trans` still beats `angular-canonical::angular-hopf-base` in the single artifact, but it does not beat legacy transport overall in that same artifact.

## 2026-03-20 Guarded-Relic Cross-Model Replicates Update

- Repeated guarded-relic subset captures on stronger models used `openai-chat-completions` with `gpt-4.1` and `gpt-5-chat-latest`.
- On `gpt-4.1`, the earlier one-sample “strong separation” result did not hold up as a stable pattern across three replicates:
  - aggregate-score ties occurred in `2/3` replicates
  - replay equality occurred in `1/3` replicates
  - no replicate showed the earlier stalled `use relic-key` loop or a stable sentinel-path attractor
  - `angular-hopf-trans` did not consistently outperform `angular-hopf-base`
- On `gpt-5-chat-latest`, the ceiling-style collapse did hold up across three replicates:
  - aggregate-score ties occurred in `3/3` replicates
  - replay equality occurred in `3/3` replicates
  - both variants followed the same successful sentinel path in all three replicates
- Current cross-model interpretation:
  - `gpt-4.1` is not a stable router-sensitive benchmark model for this slice
  - `gpt-5-chat-latest` currently behaves like a stable router-insensitive ceiling model on guarded-relic

## 2026-03-20 Hazard-Route Cross-Model Replicates Update

- Repeated hazard-route subset captures on `openai-chat-completions` with `gpt-4.1-mini`, `gpt-4.1`, and `gpt-5-chat-latest` produced a different cross-model pattern from guarded-relic.
- On `gpt-4.1-mini`, hazard-route was strongly router-sensitive across all three replicates:
  - `angular-hopf-base` beat `angular-hopf-trans` in `3/3` replicates
  - `angular-hopf-base` consistently fell into a `bridge_kit_route`
  - `angular-hopf-trans` consistently fell into a `raider_attack_loop`
- On both `gpt-4.1` and `gpt-5-chat-latest`, hazard-route collapsed to the same `all_look_loop` replay for both variants in `3/3` replicates.
- Current hazard-route interpretation:
  - hazard-style routing is more model-sensitive than guarded-relic
  - `gpt-4.1-mini` is currently the clearest discriminator for angular variant differences on hazard-route
  - stronger models currently collapse variant differences on hazard-route rather than sharpening them
