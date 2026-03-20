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
