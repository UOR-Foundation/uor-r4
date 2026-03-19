# Local Observer

This directory contains a single static observer page for inspecting MUDBench
report exports locally.

The observer consumes the existing `reports export --dir <path>` JSON viewmodel.
It does not run a server, mutate artifacts, or fetch benchmark data on its own.

## Generate Saved Artifacts

First save one or more suite reports. The observer's replay inspector depends on
the sibling `*.replay.json` files written by the `suite --output-file ...`
workflow.

Example baseline artifact:

```bash
python -m src.cli.main suite --suite tiny --output json --output-file /tmp/tiny-suite.json
```

Example mixed external comparison artifact:

```bash
python -m src.cli.main suite \
  --suite tiny \
  --baseline-agent agent-a \
  --agent-command "python examples/agents/deterministic_rule_agent.py" \
  --agent-label deterministic-wrapper \
  --external-agent-actor agent-b \
  --output json \
  --output-file /tmp/tiny-suite-mixed.json
```

That workflow writes:

- `...report.json`
- `...report.json.manifest.json`
- `...report.json.replay.json`

## Generate Export JSON

Write the export viewmodel to a file:

```bash
python -m src.cli.main reports export --dir /path/to/saved-artifacts --output json > observer-export.json
```

The saved-artifacts directory should contain the `*.manifest.json` files created
by the `suite --output-file ...` workflow, plus their sibling `*.replay.json`
files.

## Open The Observer

Open [index.html](/Users/adminamn/MUDBench/tools/observer/index.html) directly in a browser.

Because the page is intentionally static and dependency-free, use one of these
manual workflows:

1. Click `Choose Export JSON` and select the exported `observer-export.json` file.
2. Or paste the JSON into the text area and click `Load From Text`.
3. Optionally narrow the loaded export with the observer filter controls.

## What It Displays

The page renders the existing export viewmodel only. It shows:

- leaderboard summary
- identity rollup summaries when the export includes `identity_rollups`
- artifact/history entries
- scenario coverage
- actor coverage
- external agent label coverage
- lightweight identity filters for built-in actors, external labels, external
  profiles, and shared-run vs non-shared comparison artifacts
- arena summaries for true shared-run comparison artifacts, including compact
  matchup leaders and replay entrypoints
- comparison artifact match summaries from the export's `history` and `replay_drilldowns`
- replay drilldown entries derived from the export's `replay_drilldowns`
- one selected saved run in event-by-event form, including event types, final
  state summary, and score summary

If the JSON does not match the current export shape, the page shows a validation
error instead of attempting to guess.

## Replay Inspection Workflow

The replay viewer is still fully static and `file://` compatible. It does not
load manifest or replay files directly from disk.

Use this exact workflow:

1. Generate saved suite artifacts with `suite --output-file ...`.
2. Generate `observer-export.json` with `reports export --dir ...`.
3. Open [index.html](/Users/adminamn/MUDBench/tools/observer/index.html).
4. Load the exported JSON file or paste its contents.
5. Use the replay inspector dropdowns to choose one replay drilldown artifact
   and one run within it.

The page consumes the exported `replay_drilldowns` data already embedded in the
viewmodel. No extra backend or file fetch is required.

## Match Summary Workflow

The match-summary view is also fully static and consumes only the loaded export
JSON.

When the export contains `suite_comparison` artifacts, the page shows:

- scenario-by-scenario baseline and candidate identities
- per-scenario score difference
- aggregate comparison difference from the saved history score summary
- event-type summaries derived from the replay drilldown runs
- a direct `Inspect Replay` action that focuses the replay inspector on the
  chosen scenario run

If a comparison artifact contains true shared-run replay context, the page uses
that directly.

If a comparison artifact only contains paired baseline/candidate runs, the page
derives the head-to-head scenario row by matching runs with the same
`scenario_id`. In that case, the replay inspector focuses one saved source run
at a time and the page makes no claim that the event stream is from a shared
execution.

## Arena Summary Workflow

The arena-summary view is a compact shared-run view over the same loaded export
JSON.

Use this workflow:

1. Generate a shared-run comparison artifact with `suite --output-file ...`.
2. Generate `observer-export.json` with `reports export --dir ...`.
3. Open [index.html](/Users/adminamn/MUDBench/tools/observer/index.html).
4. Load the exported JSON file or paste its contents.
5. Choose a `Shared-Run Matchup` in the arena summary section.

For each shared-run comparison artifact, the page shows:

- matchup identity, including external profile labels when present
- per-scenario score differences
- aggregate summary difference
- a derived leader indication when the score difference is available
- event-type summaries from the replay drilldown
- an `Inspect Replay` entrypoint that focuses the existing replay inspector on
  the selected scenario run

If the loaded export contains only paired baseline/candidate comparisons, the
arena-summary section degrades gracefully and directs you to the existing
Match Summary view instead.

## Filter Workflow

The observer also includes lightweight static filters. These controls do not
rewrite the loaded export JSON or mutate any saved artifacts. They only narrow
the rendered view in the browser.

Available filters:

- `Built-In Actor`
- `External Label`
- `External Profile`
- `Comparison View` (`All artifacts`, `Shared-run comparisons`,
  `Non-shared comparisons`)

These filters affect at minimum:

- identity rollup rows
- history entries
- leaderboard rows
- arena-summary visibility
- match-summary visibility
- replay drilldown selectors for the currently visible artifact set

When identity fields are absent from the loaded export, the page leaves the
relevant filter empty and continues to render the remaining views normally.

If the loaded export does not yet include `identity_rollups`, the observer
shows a small explanatory message instead of guessing a rollup model locally.

## Identity Rollup Workflow

When the loaded export includes `identity_rollups`, the observer renders a
profile-centric summary panel that shows:

- `identity_type`
- `identity_value`
- scenario coverage count
- artifact/run counts
- score total and score average per artifact when present
- comparison presence indicators
- shared-run arena indicators when present

This panel is read-only and uses the same active observer filters as the rest
of the page.
