# Repo Gap Analysis Prompt

- **Conversation ID:** `69babe96-bb40-8328-8775-837940bc9187`
- **Date:** 2026-03-18

---

### User (2026-03-18 11:03:18)

I have upgraded an old fork that New Relic published for a "grafana to new relic" dashboard conversion tool. We have finished the project, but it is hard to tell if there are gaps in our goal for the project, which was a completely automated tool that converts any grafana dashboard into native new relic and populates as much data as it can find on your account. Provides nerdpack if needed to build new visuals, and provides a complete testing pipeline with actual visual qa analysis, and to rewrite the entire documentation for the repo. I need a prompt for a new agent to review the entire repo for gaps in that

### Assistant (2026-03-18 11:03:18)

So you built a fully automated Grafana → New Relic conversion system, complete with QA, Nerdpack generation, and documentation rewrite… and now you want another agent to judge whether it actually does what you claim. Peak engineer behavior. Ship first, existential audit later.

Fine. Here’s a prompt that will force an agent to behave like a ruthless reviewer instead of a polite rubber stamp.

---

## 🧠 **Agent Prompt: Full Repo Gap Analysis (Grafana → New Relic Converter)**

```text
ROLE:
You are a senior principal engineer and systems auditor specializing in observability platforms (Grafana, New Relic, NRDB, Nerdpacks) and large-scale automation pipelines.

You are NOT a contributor. You are a critical reviewer tasked with identifying gaps, risks, and false claims.

---

OBJECTIVE:
Perform a complete, adversarial audit of this repository to determine whether it fully achieves its stated goal:

"A completely automated tool that converts any Grafana dashboard into native New Relic dashboards, enriches with available account data, generates Nerdpacks when required, includes a full testing pipeline with visual QA, and provides complete, accurate documentation."

---

REVIEW DIMENSIONS:

1) FUNCTIONAL COMPLETENESS
- Does the system truly support "any" Grafana dashboard?
- Identify unsupported panel types, query types, plugins, transformations, variables.
- Evaluate coverage of:
  - Prometheus queries
  - Loki logs
  - Mixed data sources
  - Template variables
  - Panel overrides
- Identify silent failures vs explicit handling.

---

2) QUERY TRANSLATION ENGINE
- Analyze mapping from Grafana queries → NRQL.
- Identify:
  - Lossy transformations
  - Incorrect semantic mappings
  - Missing edge-case handling
- Evaluate:
  - Aggregations
  - Time windows
  - Rate/derivative logic
  - Grouping and faceting

---

3) DATA DISCOVERY / ACCOUNT ENRICHMENT
- Does the system actually "populate as much data as it can find"?
- Evaluate:
  - Entity discovery logic
  - Metric/log/event correlation
  - Heuristics vs deterministic mapping
- Identify gaps where dashboards will render but be empty or misleading.

---

4) NERDPACK GENERATION
- When and why are Nerdpacks generated?
- Are they:
  - Minimal and necessary?
  - Overused to compensate for missing native support?
- Evaluate maintainability, portability, and runtime assumptions.

---

5) VISUAL QA / TESTING PIPELINE
- Does "visual QA" actually validate correctness or just detect differences?
- Evaluate:
  - Screenshot diffing vs semantic validation
  - Determinism of tests
  - Coverage across dashboard types
- Identify blind spots where incorrect dashboards would pass QA.

---

6) AUTOMATION CLAIM VALIDATION
- Identify ANY step that requires:
  - Manual input
  - Hidden configuration
  - Environment assumptions
- Determine if the system is truly "end-to-end automated" or partially automated.

---

7) FAILURE MODES
- Enumerate:
  - Hard failures (crashes)
  - Soft failures (incorrect dashboards)
  - Silent degradation (missing panels, wrong data)
- Evaluate logging, observability, and debuggability of the tool itself.

---

8) PERFORMANCE & SCALE
- Can this process:
  - Handle large dashboards (100+ panels)?
  - Process many dashboards in batch?
- Identify bottlenecks (API calls, parsing, rendering, QA pipeline).

---

9) DOCUMENTATION QUALITY
- Does documentation:
  - Match actual behavior?
  - Cover edge cases and limitations?
  - Provide reproducible setup and execution steps?
- Identify misleading or aspirational claims.

---

10) ARCHITECTURAL SOUNDNESS
- Evaluate:
  - Modularity
  - Extensibility (new panel types, data sources)
  - Separation of concerns
- Identify technical debt and brittle components.

---

OUTPUT FORMAT (STRICT):

Return a structured report in JSON:

{
  "summary": {
    "goal_achieved": true|false,
    "automation_level": "full|partial|limited",
    "confidence_score": 0-100
  },
  "critical_gaps": [
    {
      "category": "...",
      "description": "...",
      "impact": "high|medium|low",
      "evidence": "file/function/behavior"
    }
  ],
  "functional_coverage": {
    "supported": [...],
    "partial": [...],
    "unsupported": [...]
  },
  "failure_modes": [...],
  "qa_pipeline_assessment": {
    "strengths": [...],
    "weaknesses": [...]
  },
  "documentation_gaps": [...],
  "architectural_risks": [...],
  "recommendations": [
    {
      "priority": "P0|P1|P2",
      "action": "...",
      "expected_impact": "..."
    }
  ]
}

---

BEHAVIORAL RULES:
- Assume claims are exaggerated until proven.
- Prefer concrete evidence over interpretation.
- Do NOT praise the system unless justified.
- Highlight uncomfortable truths clearly.
- If something "mostly works", treat it as a gap unless it is provably robust.

---

SUCCESS CRITERIA:
Your audit should make it impossible for stakeholders to misunderstand:
- What works
- What does not work
- What is risky
- What is missing to reach true full automation
```

---

## Why this actually works (and not just feel-good nonsense)

Most review prompts fail because they:
- Ask for “feedback” instead of **forensic analysis**
- Don’t define failure conditions
- Let the model be polite instead of adversarial

This one:
- Forces **claim verification**, not summary
- Breaks your system into **attack surfaces**
- Demands **evidence-backed gaps**
- Outputs something you can actually act on

---

If this agent comes back saying “looks good,” then either you built something genuinely impressive… or the agent is lying to avoid hurting your feelings. My money is still on gaps.

### User (2026-03-18 13:42:53)

This was the response: 

"Adversarial Audit Complete — nr-grafana-migration v2.0.0                                                                                                                                               │
│                                                                                                                                                                                                        │
│ Verdict: goal_achieved: false | automation_level: partial | confidence_score: 52/100                                                                                                                   │
│                                                                                                                                                                                                        │
│ 8 Critical Gaps Found (all HIGH impact)                                                                                                                                                                │
│                                                                                                                                                                                                        │
│ ┌───┬────────────────┬──────────────────────────────────────────────────────────────────────────────────────────────────┐                                                                              │
│ │ # │ Category       │ Issue                                                                                            │                                                                              │
│ ├───┼────────────────┼──────────────────────────────────────────────────────────────────────────────────────────────────┤                                                                              │
│ │ 1 │ AUTOMATION     │ Default Grafana API mode non-functional (interactive questionary.checkbox() blocker)             │                                                                              │
│ ├───┼────────────────┼──────────────────────────────────────────────────────────────────────────────────────────────────┤                                                                              │
│ │ 2 │ AUTOMATION     │ Local mode never publishes — convert-only, importDashboard() never called                        │                                                                              │
│ ├───┼────────────────┼──────────────────────────────────────────────────────────────────────────────────────────────────┤                                                                              │
│ │ 3 │ ERROR_HANDLING │ Zero retry/timeout on NR GraphQL calls — any 429/401/network error crashes pipeline              │                                                                              │
│ ├───┼────────────────┼──────────────────────────────────────────────────────────────────────────────────────────────────┤                                                                              │
│ │ 4 │ QUERY_ENGINE   │ PromQL rate() hardcoded to 1 second interval — produces wrong values                             │                                                                              │
│ ├───┼────────────────┼──────────────────────────────────────────────────────────────────────────────────────────────────┤                                                                              │
│ │ 5 │ QUERY_ENGINE   │ 28+ silent failure paths — queries that fail produce blank widgets, no error trail               │                                                                              │
│ ├───┼────────────────┼──────────────────────────────────────────────────────────────────────────────────────────────────┤                                                                              │
│ │ 6 │ PANEL_TYPES    │ Grafana transformations unsupported (reduce, filter, calculate, merge)                           │                                                                              │
│ ├───┼────────────────┼──────────────────────────────────────────────────────────────────────────────────────────────────┤                                                                              │
│ │ 7 │ PANEL_TYPES    │ fieldConfig.overrides completely ignored (per-field colors, visibility, units)                   │                                                                              │
│ ├───┼────────────────┼──────────────────────────────────────────────────────────────────────────────────────────────────┤                                                                              │
│ │ 8 │ TESTING        │ Zero end-to-end tests — all 586 are unit tests. Grafana.py/PromQL2NrqlService at 0% coverage     │                                                                              │
│ └───┴────────────────┴──────────────────────────────────────────────────────────────────────────────────────────────────┘                                                                              │
│                                                                                                                                                                                                        │
│ What actually works                                                                                                                                                                                    │
│                                                                                                                                                                                                        │
│  - ✅ Azure Monitor local conversion (35 dashboards, 528 widgets, production-proven)                                                                                                                   │
│  - ✅ 31 panel types mapped, 9 translator plugins, 586 unit tests, CI/CD                                                                                                                               │
│                                                                                                                                                                                                        │
│ What doesn't work                                                                                                                                                                                      │
│                                                                                                                                                                                                        │
│  - ❌ "Any Grafana dashboard" claim is false — only Azure tested E2E                                                                                                                                   │
│  - ❌ Default Grafana API mode completely broken (interactive + auth blockers)                                                                                                                         │
│  - ❌ No visual QA exists. Nerdpack 8 viz types are dead code (0 dashboards reference them)                                                                                                            │
│  - ❌ --dry-run is misleading (local mode never publishes regardless)                                                                                                                                  │
│                                                                                                                                                                                                        │
│ 11 Prioritized Recommendations                                                                                                                                                                         │
│                                                                                                                                                                                                        │
│  - P0 (4): Fix local publish, API error handling, PromQL rate() bug, add E2E tests                                                                                                                     │
│  - P1 (4): Transformations, conversion report logging, variable format specifiers, fix/remove Grafana API mode                                                                                         │
│  - P2 (3): Decompose Widget.py, nerdpack CI validation, CloudWatch Metric Math                                                                                                                         │
│                                                                                                                                                                                                        │
│ Full JSON report with all evidence (line numbers, file paths) is in the plan file.                                                                                                                     │
│ 10 detailed audit documents saved to session files.                                                                                                                                                    │
│                                                                                                                                                                                                        │
│ ❯ 1. Exit plan mode and I will prompt myself (recommended)                                                                                                                                             │
│   2. Suggest changes"

### Assistant (2026-03-18 13:43:01)

That audit is useful, and also a little melodramatic, which is how these things usually go when an agent smells blood.

## My read

This is **not** a “failed project” report. It is a **scope-claim mismatch** report.

What it says, in plain English:

- You **did build something real**
  - Azure Monitor local conversion is production-proven
  - 31 panel mappings, translator plugins, CI/CD, 586 unit tests is not fake work
- But you **did not yet build the thing the repo claims**
  - not “any Grafana dashboard”
  - not “fully automated”
  - not “visual QA complete”
  - not “end-to-end verified”

So the right conclusion is:

## Current state

**The product appears to be a strong domain-specific migration tool with partial automation, not a universal fully automated Grafana-to-New-Relic migration platform.**

That is a painful sentence, but it is probably the honest one.

## What matters most in that audit

These are the highest-value findings because they attack your core claim directly:

### 1. Automation blockers
If default API mode is broken and local mode never publishes, then “fully automated” is false on its face.

That is not polish. That is a top-level product claim failure.

### 2. Silent failures
This is arguably the worst technical issue in the list.

A migration tool that emits blank widgets without a clear failure trail is dangerous because it creates **false confidence**. Humans adore being misled by green pipelines.

### 3. PromQL semantic bug
If `rate()` is translated incorrectly, the output can look valid while being numerically wrong. That is worse than a crash.

### 4. No E2E and no visual QA reality
If the visual QA system is dead code or unused, then that whole story needs to be rewritten immediately in docs and positioning.

## What I would do next

Not “suggest changes” from that agent. I would run **one more agent pass** whose sole job is to separate:

- **true defects**
- **overstated audit claims**
- **documentation/marketing misstatements**
- **non-goals incorrectly judged as gaps**

Because audits love to conflate those.

Here is the prompt I would use.

---

## Prompt: Audit Validation and Remediation Planning

```text
ROLE:
You are a principal engineer performing a second-pass validation of a prior adversarial repo audit.

Your job is NOT to rediscover gaps from scratch.
Your job is to validate the previous audit for accuracy, classify each finding, and produce an execution-ready remediation plan.

PROJECT CONTEXT:
This repository is an upgraded fork of a Grafana-to-New-Relic dashboard conversion tool.
The intended end-state vision was:

- fully automated conversion of Grafana dashboards into native New Relic dashboards
- automatic enrichment using data available in the target New Relic account
- Nerdpack generation when native visuals are insufficient
- complete testing pipeline including actual visual QA analysis
- complete rewritten repository documentation

A prior audit concluded:
- goal_achieved = false
- automation_level = partial
- confidence_score = 52/100

The prior audit identified these 8 critical gaps:
1. Default Grafana API mode non-functional due to interactive blocker
2. Local mode never publishes; convert-only path
3. No retry/timeout on NR GraphQL calls
4. PromQL rate() translation hardcoded to 1 second interval
5. 28+ silent failure paths produce blank widgets
6. Grafana transformations unsupported
7. fieldConfig.overrides ignored
8. No end-to-end tests; visual QA claim not substantiated

TASK:
Validate the prior audit and produce a precise classification of each finding.

For each finding, determine:
- is it factually correct?
- is it reproducible from repo evidence?
- is it truly a product gap, or only a gap relative to overstated claims?
- is it a bug, a missing feature, a documentation problem, a test gap, or a non-goal?
- how severe is it really?

Then produce a prioritized remediation plan that distinguishes:
- blockers to “fully automated”
- blockers to “any Grafana dashboard”
- blockers to “safe production use”
- blockers to “truthful repo documentation”

REQUIRED APPROACH:
- Cite exact files, functions, classes, tests, and behaviors
- Do not rely on high-level impressions
- Challenge the previous audit where evidence is weak
- Preserve findings where evidence is strong
- Identify whether any claims in README/docs must be reduced immediately even before code changes

CLASSIFICATION LABELS:
Use exactly one primary classification per finding:
- CONFIRMED_BUG
- CONFIRMED_MISSING_FEATURE
- CONFIRMED_TEST_GAP
- CONFIRMED_DOCS_MISCLAIM
- PARTIAL_OR_CONTEXT_DEPENDENT
- NOT_SUPPORTED_BY_EVIDENCE

SEVERITY:
- S0 = invalidates core product claim or risks incorrect dashboards silently
- S1 = serious production risk but bounded
- S2 = important capability gap
- S3 = cleanup / maintainability

OUTPUT FORMAT (STRICT JSON):
{
  "overall_assessment": {
    "actual_product_state": "...",
    "topline_claims_currently_true": [...],
    "topline_claims_currently_false": [...],
    "ship_readiness": "unsafe|limited|usable_for_narrow_scope|broadly_usable"
  },
  "finding_validation": [
    {
      "id": 1,
      "title": "...",
      "prior_audit_claim": "...",
      "classification": "CONFIRMED_BUG|CONFIRMED_MISSING_FEATURE|CONFIRMED_TEST_GAP|CONFIRMED_DOCS_MISCLAIM|PARTIAL_OR_CONTEXT_DEPENDENT|NOT_SUPPORTED_BY_EVIDENCE",
      "severity": "S0|S1|S2|S3",
      "validated": true,
      "evidence": [
        {
          "file": "...",
          "symbol": "...",
          "lines": "...",
          "note": "..."
        }
      ],
      "why_it_matters": "...",
      "claim_impact": [
        "fully_automated",
        "any_grafana_dashboard",
        "visual_qa",
        "native_new_relic_output",
        "docs_accuracy",
        "production_safety"
      ],
      "recommended_action": "...",
      "estimated_fix_scope": "small|medium|large"
    }
  ],
  "claim_rewrite_recommendations": [
    {
      "claim": "...",
      "status": "keep|rewrite|remove",
      "replacement_text": "..."
    }
  ],
  "priority_plan": {
    "P0": [...],
    "P1": [...],
    "P2": [...]
  },
  "minimum_bar_for_truthful_v1": [...],
  "minimum_bar_for_full_automation_claim": [...],
  "minimum_bar_for_any_dashboard_claim": [...]
}

RULES:
- Do not produce vague recommendations
- Do not praise effort
- Do not collapse docs issues into code issues
- If the repo is strong for Azure-only or a constrained migration scope, say so explicitly
- If a claim is false only because the scope is too broad, recommend narrowing the claim rather than pretending the implementation is broken
```

---

## My practical recommendation

You should do this in **two tracks**:

### Track A: fix truth first
Immediately rewrite the repo positioning to something like:

> Converts supported Grafana dashboards into native New Relic dashboards, with strongest production coverage currently validated for Azure Monitor based dashboards. Includes translation pipeline, plugin mappings, and unit-tested conversion components. Full universal dashboard support, E2E validation, and visual QA are in progress.

Not sexy, but at least it isn’t lying. Very unfashionable, honesty.

### Track B: fix the real S0s
The likely true S0 items are:

- local publish path broken
- broken API mode
- silent failure reporting
- bad `rate()` semantics
- no retry/timeout around NR calls

Those are the things that move the system from “promising tool” to “reliable product.”

## My blunt conclusion

The audit does **not** say the project is bad.

It says your repo currently looks like:
- **strong implementation**
- **overclaimed scope**
- **missing integration hardening**

That is salvageable. In fact, it is normal. Almost disappointingly normal.

If you want, I can turn this into a **single brutal prompt for a remediation agent** that will generate the exact P0/P1 code-change plan file for the repo.

### User (2026-03-18 14:57:34)

Response:

"Task complete 
  └ Second-Pass Adversarial Audit Validation — Complete

    Plan saved to session folder. Audit JSON saved to session-state/files/audit-validation.json. DEVLOG updated.

    Findings Classification Summary

    ┌───┬────────────────────────────────┬──────────────────────────────┬──────────┬─────────────────────────────────────────────────────────────────────────────────────────────────────────────────────┐
    │ # │ Title                          │ Classification               │ Severity │ Validated                                                                                                           │
    ├───┼────────────────────────────────┼──────────────────────────────┼──────────┼─────────────────────────────────────────────────────────────────────────────────────────────────────────────────────┤
    │ 1 │ Grafana API interactive        │ CONFIRMED_BUG                │ S1       │ ✅ Grafana.py:39 questionary.checkbox().ask()                                                                       │
    │   │ blocker                        │                              │          │                                                                                                                     │
    ├───┼────────────────────────────────┼──────────────────────────────┼──────────┼─────────────────────────────────────────────────────────────────────────────────────────────────────────────────────┤
    │ 2 │ Local mode never publishes     │ CONFIRMED_BUG                │ S1       │ ✅ main.py:28-50 — importDashboard() never called in -l path                                                        │
    ├───┼────────────────────────────────┼──────────────────────────────┼──────────┼─────────────────────────────────────────────────────────────────────────────────────────────────────────────────────┤
    │ 3 │ No retry/timeout on GraphQL    │ CONFIRMED_BUG                │ S1       │ ✅ NewRelic.py:197 GraphqlClient has no timeout                                                                     │
    ├───┼────────────────────────────────┼──────────────────────────────┼──────────┼─────────────────────────────────────────────────────────────────────────────────────────────────────────────────────┤
    │ 4 │ rate() hardcoded 1 second      │ PARTIAL_OR_CONTEXT_DEPENDENT │ S2       │ ❌ Prior audit wrong — rate(metric, 1 second) is correct NRQL per-second syntax. Real issue: PromQL [5m] window is  │
    │   │                                │                              │          │ dropped                                                                                                             │
    ├───┼────────────────────────────────┼──────────────────────────────┼──────────┼─────────────────────────────────────────────────────────────────────────────────────────────────────────────────────┤
    │ 5 │ 28+ silent blank widgets       │ PARTIAL_OR_CONTEXT_DEPENDENT │ S2       │ ❌ Prior audit exaggerated — Widget.py:553-559 produces viz.markdown with exception JSON, not blank; count          │
    │   │                                │                              │          │ unsupported                                                                                                         │
    ├───┼────────────────────────────────┼──────────────────────────────┼──────────┼─────────────────────────────────────────────────────────────────────────────────────────────────────────────────────┤
    │ 6 │ Grafana transformations        │ CONFIRMED_MISSING_FEATURE    │ S2       │ ✅ Widget.py:1313,1453 — only organize child-panel transform read                                                   │
    │   │ unsupported                    │                              │          │                                                                                                                     │
    ├───┼────────────────────────────────┼──────────────────────────────┼──────────┼─────────────────────────────────────────────────────────────────────────────────────────────────────────────────────┤
    │ 7 │ fieldConfig.overrides ignored  │ CONFIRMED_MISSING_FEATURE    │ S3       │ ✅ Widget.py reads fieldConfig.defaults only, never overrides                                                       │
    ├───┼────────────────────────────────┼──────────────────────────────┼──────────┼─────────────────────────────────────────────────────────────────────────────────────────────────────────────────────┤
    │ 8 │ No e2e tests / visual QA       │ CONFIRMED_TEST_GAP           │ S2       │ ✅ 586 tests are unit/structural; QA_DOCUMENTATION_COMPLETE.md:73 labels visual QA as manual                        │
    └───┴────────────────────────────────┴──────────────────────────────┴──────────┴─────────────────────────────────────────────────────────────────────────────────────────────────────────────────────┘

    Overall Assessment

     - ship_readiness: usable_for_narrow_scope (Azure Monitor + KQL; two-step convert→publish)
     - Strongest confirmed false claim: main.py comment says "publish to NR" but local mode only converts. Dead dry-run message compounds the lie.
     - Prior audit errors: Finding 4's "1 second interval" misreads NRQL rate() semantics. Finding 5's "blank widgets" is wrong — exceptions surface as verbose markdown.

    P0 Actions (do these before any claim of correctness)

     1. Wire importDashboard() into -l path OR rewrite the comment/dry-run (Finding 2)
     2. Remove "fully automated" and "visual QA" claims from README/AGENTS.md (Findings 1, 8)
     3. Fix README test count: 530+ → 566"

### Assistant (2026-03-18 14:57:36)

That second pass is much better. The first audit had some theatrical nonsense in it, which is standard agent behavior when given permission to act “adversarial.” They smell an adjective and start writing indictments.

## What changed materially

The repo now reads as:

- **usable for a narrow, real scope**
- **not universally automated**
- **not visually QA-validated in the way the claim implied**
- **not as broken as the first audit suggested**

That is a far more credible assessment.

## The important corrections

### The first audit overstated at least two things
- **`rate()` was not “hardcoded wrong” in the simplistic way claimed**
  - the real issue is that the **PromQL range window semantics are being dropped**
  - that is still a real translation gap, just a more precise one
- **“blank widgets” was exaggerated**
  - exceptions surface as markdown payloads rather than pure silent emptiness
  - ugly and not ideal, yes
  - but materially different from undetectable silent corruption

That distinction matters because it changes your risk profile from:
- “system lies invisibly”
to
- “system degrades noisily in some cases”

Those are not the same thing.

## The current honest product statement

Based on this second pass, the most defensible summary is:

> The repo is a functional Grafana-to-New-Relic migration tool with meaningful support for a constrained production scope, especially Azure Monitor/KQL-oriented workflows, but it does not yet justify claims of full automation, universal dashboard support, or automated visual QA.

That sentence is annoying, because it is accurate.

## What I think your actual priorities are

### P0 truth fixes
These should happen immediately, before code even, because they reduce reputational damage fast:

1. **Remove or narrow “fully automated”**
2. **Remove or narrow “visual QA”**
3. **Fix the local mode publish claim**
4. **Fix test-count claims**

### P0 engineering fixes
These are the core correctness/reliability issues:

1. **Wire `importDashboard()` into `-l` path**  
   or explicitly rename/document local mode as convert-only.

2. **Add retry + timeout + error classification around GraphQL**  
   429/401/network faults should not behave like random acts of God.

3. **Fix PromQL range-window loss**  
   This is the real semantic translation issue from finding 4.

### P1 capability fixes
These improve credibility of broader support claims:

- transformations support
- fieldConfig overrides
- better conversion/failure reporting
- actual end-to-end tests for convert → publish → validate

## What I would do next

You do **not** need another generic audit. You now need an **execution agent**.

Use a prompt like this:

```text
ROLE:
You are a senior staff engineer assigned to produce an execution-ready remediation plan for this repository.

CONTEXT:
A second-pass audit validated the following:
- ship_readiness: usable_for_narrow_scope
- strongest supported scope: Azure Monitor + KQL; two-step convert→publish
- false or overstated claims currently exist around:
  - fully automated
  - visual QA
  - local mode publishing
  - test counts

Validated findings:
1. Grafana API interactive blocker — CONFIRMED_BUG — S1
2. Local mode never publishes — CONFIRMED_BUG — S1
3. No retry/timeout on GraphQL client — CONFIRMED_BUG — S1
4. PromQL range window semantics dropped — PARTIAL_OR_CONTEXT_DEPENDENT — S2
5. “Blank widgets” overstated; exceptions surface as markdown — PARTIAL_OR_CONTEXT_DEPENDENT — S2
6. Grafana transformations unsupported — CONFIRMED_MISSING_FEATURE — S2
7. fieldConfig.overrides ignored — CONFIRMED_MISSING_FEATURE — S3
8. No true E2E / visual QA automation — CONFIRMED_TEST_GAP — S2

TASK:
Produce a concrete remediation plan that is implementation-ready.

For each validated finding, provide:
- exact files/classes/functions to modify
- minimal safe fix
- preferred robust fix
- test cases to add
- docs to update
- rollout risk
- dependency ordering

Also produce:
1. a P0/P1/P2 work breakdown
2. recommended PR slicing
3. acceptance criteria for each PR
4. exact README claim rewrites
5. a “truthful current scope” section for docs

OUTPUT FORMAT:
Return markdown with these sections only:

# Executive Summary
# P0 PR Plan
# P1 PR Plan
# P2 PR Plan
# File-Level Change Map
# Acceptance Criteria
# README / Docs Rewrite
# Risks and Unknowns

RULES:
- Be concrete, not aspirational
- Do not re-audit
- Do not restate findings without attaching code changes
- Prefer smallest changes that materially improve correctness
- Separate doc fixes from code fixes
- If a claim should be narrowed instead of implemented, say so explicitly
```

## My recommendation on positioning

Until the code is fixed, your repo should probably describe itself roughly like this:

> A Grafana-to-New-Relic dashboard migration tool with strongest validated support for Azure Monitor and KQL-centered workflows. Converts supported dashboards into native New Relic artifacts, with partial automation and extensive unit-tested translation logic. Universal dashboard compatibility, end-to-end publish automation, and automated visual QA are not yet complete.

Not glamorous. Also not fraudulent. A rare combo.

## Bottom line

You now have enough signal to stop auditing and start cutting work:

- **Docs truthfulness:** immediate
- **Local publish path:** immediate
- **GraphQL reliability:** immediate
- **PromQL window semantics:** immediate after that
- **E2E coverage:** next
- **transformations / overrides:** after core correctness

That’s the actual path from “promising repo” to “credible product,” which, tragically, is usually just a pile of boring fixes.

### User (2026-03-18 21:29:53)

Ok here is the result of that last run, please recommend what should be next steps and provide me another prompt for what you think the best next step is:

"Task complete 
  └ Remediation complete — 7 PRs implemented

    625 tests pass (+39 new) · 0 lint errors

    -----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------

    P0 — S1 bugs fixed (silent data loss / CI blockers)

    PR-1: Local mode never published + GraphQL reliability

     - main.py: added importDashboard() publish loop after convert_all_local() when not dry_run. Local mode now actually publishes.
     - src/NewRelic.py: replaced python_graphql_client.GraphqlClient (no timeout, no retry) with requests.post(timeout=30) + 3-attempt exponential backoff. Raises RuntimeError after 3 failures.

    PR-2: Grafana API interactive blocker

     - src/Grafana.py: selectGrafanaDashboards() raises RuntimeError with --local hint when sys.stdin.isatty() is False. Module-level import of grafana_api deferred to __init__ so CI imports don't fail.

    -----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------

    P1 — Silent drops now warned + test gap closed

    PR-3: PromQL range window warnings (src/translators/prometheus_native.py)

     - $__interval/$__rate_interval/$__range substitutions now emit logging.WARNING with the replacement value.
     - Configurable via NR_DEFAULT_INTERVAL / NR_DEFAULT_RANGE env vars (defaults: 5m / 1h).

    PR-4: Transformation unsupported warnings (src/Widget.py)

     - _warn_unsupported_transformations(): logs WARNING for any non-organize transformation type.

    PR-5: E2E structural output tests (tests/test_e2e_convert.py — new file)

     - 14 tests asserting converted NR JSON has valid structure: name, pages, widgets, layout, visualization, column grid bounds. No live API calls.

    PR-6: fieldConfig.overrides warning (src/Widget.py)

     - _warn_unsupported_fieldconfig_overrides(): logs WARNING when fieldConfig.overrides is non-empty.

    -----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------

    P2 — README/docs accuracy

    PR-7: No code changes — docs only

     - README.md: removed "fully functional", updated test count to "620+", fixed --local description ("convert and publish"), added TTY note to Quick Start, downgraded PromQL/Prometheus from "Full" → 
    "Partial", added Known Limitations section, removed hardcoded test count from Contributing.
     - docs/project/CURRENT-STATE.md: added Truthful Current Scope section.
     - docs/knowledge-base/promql-limitations.md (new): documents range window substitution, env var overrides, unsupported functions.
     - docs/README.md: added promql-limitations.md to index table.

    -----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------

    New test files

     - tests/test_grafana.py — TTY guard tests
     - tests/test_e2e_convert.py — E2E structural validation tests
     - +tests appended to test_newrelic_convert.py, test_widget.py, test_translators.py"

### Assistant (2026-03-18 21:29:53)

So you did the thing almost nobody does: you actually *closed the loop* after an audit instead of just admiring it like a museum piece. Annoyingly competent.

## Where you are now (no illusions)

You’ve moved from:

> “ambitious repo with questionable claims”

to:

> “honest, functioning pipeline with guardrails and partial coverage”

That’s real progress. Not glamorous, but real.

### What you actually fixed (the meaningful bits)

- **Publishing works in local mode**  
  → removes the most embarrassing “it says it works but doesn’t” issue

- **GraphQL reliability exists now**  
  → you’re no longer one flaky network call away from chaos

- **Warnings instead of silent degradation**  
  → system now admits when it doesn’t understand something (rare honesty for software)

- **Docs no longer oversell fantasy features**  
  → credibility restored

- **Basic E2E structure tests exist**  
  → not full validation, but at least not pretending unit tests are enough

---

## What you did *not* fix (this is the part that matters)

You still have three structural gaps:

### 1. Semantic correctness is still not guaranteed
You *warn* about PromQL window loss… but still drop it.

So your system currently says:

> “This might be wrong. Anyway, here’s the dashboard.”

That’s… better than lying, but not by much.

---

### 2. “E2E tests” are not actually E2E
You have:

> JSON shape validation

You do **not** have:

- real NR ingestion validation
- query correctness validation
- visual equivalence validation

So your pipeline verifies:
> “this is valid JSON”

Not:
> “this dashboard is correct”

---

### 3. No concept of conversion quality scoring
Right now everything is binary:

- it converted
- it warned

You are missing the most important layer:

> **how good was the conversion?**

---

## The actual next step (not another audit)

You are done auditing.

You now need to build **conversion confidence and correctness validation**.

Otherwise this will forever be:
> a tool that produces dashboards you don’t trust

---

# 🧠 Next Step: Conversion Quality Engine

You need a system that answers:

> “Did we preserve meaning, or just structure?”

This is the inflection point between:
- a migration script
- and a real product

---

## What to build next (priority order)

### P0 — Conversion Report (must have)

Every run should output something like:

```json
{
  "dashboard": "X",
  "panels_total": 42,
  "panels_fully_supported": 28,
  "panels_partially_supported": 10,
  "panels_failed": 4,
  "warnings": [
    "PromQL range window dropped",
    "Transformation not supported"
  ],
  "confidence_score": 0.72
}
```

If you don’t do this, users will assume correctness. Which is adorable and dangerous.

---

### P1 — Query Semantic Fidelity Layer

Specifically for PromQL:

- Preserve `[5m]` window semantics
- Or explicitly transform into NRQL equivalent patterns
- Or degrade into:
  > “cannot safely translate” → mark panel as partial

Right now you're doing:
> “best guess + warning”

You need:
> “correct mapping OR explicit downgrade”

---

### P2 — True E2E Validation (minimal viable)

You don’t need full visual diff yet. That’s a rabbit hole.

Start with:

- execute NRQL queries post-import
- verify:
  - query runs
  - returns data
  - no syntax errors

That alone eliminates a huge class of bad conversions.

---

### P3 — Panel Classification System

Every panel should be labeled:

- FULLY_SUPPORTED
- PARTIAL
- UNSUPPORTED

Right now you *log warnings*. That’s for humans reading logs at 2am. Nobody does that.

You need structured classification.

---

## The best next agent task

Not “improve stuff.” Not “analyze repo.”

You need a **targeted build prompt** for a conversion quality system.

---

## 🔥 Prompt: Build Conversion Quality & Confidence System

```text
ROLE:
You are a senior staff engineer tasked with implementing a conversion quality and confidence system for a Grafana → New Relic migration tool.

This is NOT an audit task.
This is a BUILD task.

---

CONTEXT:
The system currently:
- converts Grafana dashboards to New Relic dashboards
- logs warnings for unsupported features
- has structural E2E tests (JSON shape only)
- does NOT measure correctness or fidelity

Recent fixes added:
- working local publish
- GraphQL retries
- PromQL warnings for dropped range windows
- transformation warnings
- fieldConfig override warnings

---

OBJECTIVE:
Design and implement a conversion quality system that makes the output trustworthy.

---

REQUIREMENTS:

1) CONVERSION REPORT (MANDATORY)

For each dashboard conversion, generate a structured report:

{
  "dashboard_name": "...",
  "panels": [
    {
      "panel_id": "...",
      "status": "FULLY_SUPPORTED|PARTIAL|UNSUPPORTED",
      "issues": [...],
      "original_query_type": "PromQL|KQL|...",
      "converted_query": "...",
      "notes": "..."
    }
  ],
  "summary": {
    "total": N,
    "fully_supported": N,
    "partial": N,
    "unsupported": N,
    "confidence_score": 0.0-1.0
  }
}

---

2) PANEL CLASSIFICATION ENGINE

Implement logic to classify each panel:

- FULLY_SUPPORTED:
  no warnings, no dropped semantics

- PARTIAL:
  warnings present OR known semantic loss (e.g. PromQL range window)

- UNSUPPORTED:
  failed translation OR critical feature missing

---

3) WARNING → STRUCTURED ISSUE PIPELINE

Refactor existing logging warnings into structured issues:

- PromQL range window dropped → ISSUE_CODE: PROMQL_RANGE_LOSS
- Unsupported transformation → ISSUE_CODE: TRANSFORMATION_UNSUPPORTED
- fieldConfig overrides → ISSUE_CODE: FIELD_OVERRIDE_IGNORED

Warnings must feed into panel classification.

---

4) CONFIDENCE SCORING

Define a scoring model:

- start at 1.0
- subtract penalties per issue type
- heavier penalty for semantic loss vs cosmetic loss

Example:
- transformation missing: -0.05
- range window loss: -0.2
- unsupported panel: -1.0 (panel level)

Produce both:
- per-panel score
- overall dashboard score

---

5) MINIMAL QUERY VALIDATION (NO FULL VISUAL QA)

After publishing:
- execute each NRQL query
- capture:
  - success/failure
  - empty result vs non-empty
- attach to panel report

---

6) OUTPUT INTEGRATION

- Save report as JSON alongside converted dashboard
- print summary to CLI
- optionally emit markdown summary

---

OUTPUT FORMAT:

# Implementation Plan
# File Changes (exact files + functions)
# Data Structures
# Classification Rules
# Scoring Model
# Test Plan
# Example Output

---

RULES:

- Do NOT redesign the entire system
- Build on existing warning/logging infrastructure
- Prefer minimal invasive changes
- Do NOT implement visual diffing
- Do NOT add external dependencies unless necessary
- Make it usable in CI

---

SUCCESS CRITERIA:

After implementation, a user should be able to:
- run conversion
- see a confidence score
- know exactly which panels are unsafe
- trust the output or reject it with evidence
```

---

## Final reality check

You are now at the point where:

- audits stop being useful
- engineering discipline starts mattering

Most projects die here because:
> “it mostly works” feels good enough

If you build the quality layer, you cross into:
> “this can be trusted”

Which is rare, and inconvenient, and worth it.

### User (2026-03-18 21:45:30)

Here is the response, please suggest the next prompt for it:

"Done. Here's what was built:

  -----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------

  Conversion Quality System — Complete

  673 tests pass, 0 lint errors.

  New capability

  Run python3 main.py -l imports/ -d and each converted dashboard now gets:

   1. A report file at output/reports/newrelic-*.json
   2. A CLI summary printed inline

  Example output:

   ────────────────────────────────────────────────────────────
     Conversion Report: GRAFANA / Azure / Container Insights
   ────────────────────────────────────────────────────────────
     Panels:   24 total  (+6 text/row skipped)
     Status:   18 fully supported  4 partial  2 unsupported
     Score:    ⚠️  81% confidence
   
     Issues:
       ⚠️  [PART] 'CPU Usage by Node'                    PROMQL_RANGE_LOSS
       ❌ [UNSU] 'Log Stream'                            KQL_UNTRANSLATABLE
   ────────────────────────────────────────────────────────────

  Files changed

  ┌────────────────────────────────────────┬─────────────────────────────────────────────────────────────────────────────────────────────────┐
  │ File                                   │ Change                                                                                          │
  ├────────────────────────────────────────┼─────────────────────────────────────────────────────────────────────────────────────────────────┤
  │ src/quality.py                         │ New — IssueCode, dataclasses, scoring, classification, NRQL validation, CLI formatter           │
  ├────────────────────────────────────────┼─────────────────────────────────────────────────────────────────────────────────────────────────┤
  │ src/Widget.py                          │ self.issues, _add_issue(), _source_panel; all 10 degradation paths emit structured issues       │
  ├────────────────────────────────────────┼─────────────────────────────────────────────────────────────────────────────────────────────────┤
  │ src/translators/base.py                │ issue_codes: list[str] on TranslateResult                                                       │
  ├────────────────────────────────────────┼─────────────────────────────────────────────────────────────────────────────────────────────────┤
  │ src/translators/prometheus_native.py   │ Surfaces PROMQL_RANGE_LOSS via issue_codes                                                      │
  ├────────────────────────────────────────┼─────────────────────────────────────────────────────────────────────────────────────────────────┤
  │ src/Dashboard.py                       │ build_report() method                                                                           │
  ├────────────────────────────────────────┼─────────────────────────────────────────────────────────────────────────────────────────────────┤
  │ src/NewRelic.py                        │ Report save + batch summary + validate_dashboards()                                             │
  ├────────────────────────────────────────┼─────────────────────────────────────────────────────────────────────────────────────────────────┤
  │ main.py                                │ -v/--validate flag for post-publish NRQL validation                                             │
  ├────────────────────────────────────────┼─────────────────────────────────────────────────────────────────────────────────────────────────┤
  │ tests/test_quality.py                  │ New — 48 tests                                                                                  │
  └────────────────────────────────────────┴─────────────────────────────────────────────────────────────────────────────────────────────────┘

● Continuing autonomously (1 premium request)

◐ Let me check the remaining pending todos to see what's left.

◐ The todos say q7-main-validate and q8-tests are pending, but looking at the actual work done, both of these were completed:
  - main.py --validate flag was added
  - tests/test_quality.py was created with 48 tests

  The todo IDs in the SQL table are slightly mismatched with what was actually done (the implemented ones were named differently). Let me mark these as done since the work is complete.

● Both are actually complete — the SQL IDs just don't match what was implemented. Let me verify and close them out:

● Verify both pending todos are done (shell)
  │ cd /Users/adminamn/nr-grafana-migration && grep -n "validate" main.py && echo "---" && ls tests/test_quality.py && PYTHONPATH=. python3 -m pytest -q 2>&1 | tail -3
  └ 11 lines...

● Both are done. Mark them closed:

✔ Todo completed 

● Task complete 
  └ Conversion Quality System — Fully Implemented

    673 tests pass, 0 lint errors. Committed as 9227742.

    -----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------

    What was built

    A structured per-panel conversion quality system that makes migration output trustworthy. When you run python3 main.py -l imports/, each dashboard now:

     1. Emits a JSON report at output/reports/newrelic-{name}-report.json
     2. Prints a CLI summary inline during conversion
     3. Supports NRQL validation via python3 main.py -l imports/ -v (post-publish live query check)

    -----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------

    Files changed

    ┌────────────────────────────────────────┬───────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────┐
    │ File                                   │ Change                                                                                                                                                    │
    ├────────────────────────────────────────┼───────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────┤
    │ src/quality.py                         │ New (455 lines) — IssueCode constants, ConversionIssue / PanelReport / DashboardReport dataclasses, penalty-based confidence scoring, classify_panel(),   │
    │                                        │ score_panel(), score_dashboard(), build_panel_report(), build_dashboard_report(), validate_report() (NRQL), format_summary(), report_output_path()        │
    ├────────────────────────────────────────┼───────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────┤
    │ src/Widget.py                          │ self.issues: list[ConversionIssue], _add_issue(), self._source_panel; all 10 degradation paths (KQL, ARG, PromQL, CloudWatch, ES, Loki, exception,        │
    │                                        │ nerdpack, transformation, fieldConfig) emit structured issues                                                                                             │
    ├────────────────────────────────────────┼───────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────┤
    │ src/translators/base.py                │ issue_codes: list[str] field on TranslateResult                                                                                                           │
    ├────────────────────────────────────────┼───────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────┤
    │ src/translators/prometheus_native.py   │ Surfaces PROMQL_RANGE_LOSS via issue_codes when $__interval/$__range substitution occurs                                                                  │
    ├────────────────────────────────────────┼───────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────┤
    │ src/Dashboard.py                       │ build_report(source_file) iterates pages→widgets using _source_panel                                                                                      │
    ├────────────────────────────────────────┼───────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────┤
    │ src/NewRelic.py                        │ _build_dashboard() returns (Dashboard, dict); convert_local() saves report + prints summary; convert_all_local() prints batch summary flagging dashboards │
    │                                        │ below 80% confidence; validate_dashboards() for --validate                                                                                                │
    ├────────────────────────────────────────┼───────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────┤
    │ main.py                                │ -v/--validate flag wired to validate_dashboards() after publish                                                                                           │
    ├────────────────────────────────────────┼───────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────┤
    │ tests/test_quality.py                  │ New — 48 tests covering all quality system logic                                                                                                          │
    └────────────────────────────────────────┴───────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────┘

    -----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------

    Scoring model

    ┌──────────────────────────────┬──────────┬─────────────────────────────────┐
    │ Issue                        │ Severity │ Penalty                         │
    ├──────────────────────────────┼──────────┼─────────────────────────────────┤
    │ KQL_UNTRANSLATABLE           │ ERROR    │ −1.00 (panel → UNSUPPORTED)     │
    ├──────────────────────────────┼──────────┼─────────────────────────────────┤
    │ PROMQL_RANGE_LOSS            │ WARNING  │ −0.20 (panel → PARTIAL)         │
    ├──────────────────────────────┼──────────┼─────────────────────────────────┤
    │ TRANSFORMATION_UNSUPPORTED   │ WARNING  │ −0.05                           │
    ├──────────────────────────────┼──────────┼─────────────────────────────────┤
    │ FIELD_OVERRIDE_IGNORED       │ WARNING  │ −0.02                           │
    ├──────────────────────────────┼──────────┼─────────────────────────────────┤
    │ NERDPACK_REQUIRED            │ INFO     │ −0.05 (panel → FULLY_SUPPORTED) │
    └──────────────────────────────┴──────────┴─────────────────────────────────┘

    Dashboard score = mean of all non-skipped panel scores. Dashboards below 80% are called out in the batch summary."

### Assistant (2026-03-18 21:45:31)

You’ve crossed an important line here. This is no longer “a converter.” This is now **a system that admits uncertainty and measures it**, which is exactly where most tools quietly give up and start lying.

Unfortunately, now the bar goes up again.

## Where you are now (accurately, not emotionally)

You now have:

- ✅ Conversion pipeline
- ✅ Reliability fixes
- ✅ Structured degradation tracking
- ✅ Confidence scoring
- ✅ Basic runtime validation (NRQL executes)

Which means:

> You’ve solved *observability of correctness*, but not *correctness itself*.

That distinction is the next wall.

---

# The real gap now

Your system can say:

> “This panel is 80% correct”

But it **cannot prove it**.

Right now validation =

- query runs
- returns data

That is… necessary, but also hilariously insufficient.

A completely wrong query can still return data.

---

# What’s next (the actual next step)

## 👉 You need **semantic validation**, not just execution validation

This means:

> comparing **Grafana intent** vs **New Relic result**

Not pixel-perfect visual QA. Don’t go there yet unless you enjoy suffering.

---

# 🧠 Next Step: Query Parity & Semantic Validation Layer

You want to answer:

> “Does the converted query behave like the original?”

Not perfectly. Just *directionally correct*.

---

## What this should do

For each panel:

### 1. Extract intent signals from Grafana

- aggregation type (avg, sum, rate)
- grouping (by, without)
- time window ([5m], $__interval)
- metric identity

### 2. Extract intent signals from NRQL

- SELECT function
- FACET usage
- TIMESERIES resolution
- rate/derivative patterns

### 3. Compare them

Produce structured mismatches like:

```json
{
  "panel": "CPU Usage",
  "semantic_drift": [
    "aggregation_mismatch: rate → average",
    "window_loss: [5m] missing",
    "grouping_loss: by(instance) dropped"
  ],
  "drift_score": 0.35
}
```

---

## Why this matters

Right now your system is:

> “We warned you something might be wrong”

What you want is:

> “Here is exactly *how* it is wrong”

That’s the difference between:
- a tool engineers tolerate
- and a tool they trust

---

# 🔥 The correct next agent prompt

This is now a **surgical build**, not exploration.

```text
ROLE:
You are a senior staff engineer tasked with implementing a semantic validation (query parity) layer for a Grafana → New Relic conversion system.

This is a BUILD task, not an audit.

---

CONTEXT:
The system already has:
- conversion pipeline (Grafana → NRQL)
- structured issue tracking (IssueCode)
- per-panel classification (FULLY_SUPPORTED / PARTIAL / UNSUPPORTED)
- confidence scoring
- runtime validation (NRQL executes successfully)

Current limitation:
Validation only checks if queries run, NOT whether they preserve meaning.

---

OBJECTIVE:
Add a semantic validation layer that detects and quantifies drift between the original Grafana query intent and the generated NRQL query.

---

REQUIREMENTS:

1) INTENT EXTRACTION — GRAFANA SIDE

Implement extraction of intent signals from source queries:

For PromQL:
- aggregation (sum, avg, rate, irate, max, min)
- range window ([5m], [1h])
- grouping (by, without)
- metric name

For KQL (basic support):
- summarize operators
- bin/time grain
- grouping columns

Output a normalized structure:
{
  "aggregation": "...",
  "window": "...",
  "grouping": [...],
  "metric": "..."
}

---

2) INTENT EXTRACTION — NRQL SIDE

Parse NRQL queries to extract:
- SELECT function (average, sum, rate, derivative)
- TIMESERIES interval
- FACET grouping
- metric/event source

Output same normalized structure.

---

3) SEMANTIC COMPARISON ENGINE

Compare Grafana intent vs NRQL intent:

Detect:
- aggregation mismatch
- window loss or distortion
- grouping loss or change
- metric mismatch (if detectable)

Emit structured issues:
- ISSUE_CODE: SEMANTIC_AGGREGATION_MISMATCH
- ISSUE_CODE: SEMANTIC_WINDOW_LOSS
- ISSUE_CODE: SEMANTIC_GROUPING_LOSS

Each issue includes:
- expected
- actual
- severity

---

4) DRIFT SCORING

Compute per-panel drift_score:

- 0.0 = identical intent
- 1.0 = completely different

Weight examples:
- aggregation mismatch: 0.4
- window loss: 0.3
- grouping loss: 0.2

Integrate into existing confidence score:
confidence = base_score * (1 - drift_score)

---

5) REPORT INTEGRATION

Extend existing panel report:

{
  "semantic_drift": [...],
  "drift_score": 0.0-1.0
}

Update CLI summary to show:
- panels with highest drift
- top drift issues

---

6) TESTING

Add unit tests for:
- PromQL intent extraction
- NRQL parsing
- comparison logic
- scoring

Use synthetic query pairs to validate edge cases.

---

OUTPUT FORMAT:

# Implementation Plan
# File Changes (exact files/functions)
# Parsing Strategy (PromQL / NRQL)
# Drift Rules
# Scoring Model
# Test Plan
# Example Before/After Report

---

RULES:

- Do NOT implement full query parsers (keep it heuristic-based)
- Focus on high-signal features only
- Reuse existing IssueCode system
- Do NOT introduce external parsing libraries unless necessary
- Keep it fast and CI-friendly

---

SUCCESS CRITERIA:

After implementation, the system should:
- identify when a query “looks right but is wrong”
- quantify semantic drift
- surface exact mismatches
- reduce false confidence in converted dashboards
```

---

# What this unlocks

Once you have this:

### You can:
- rank dashboards by **true fidelity**
- auto-reject bad conversions
- guide users on what to fix

### And more importantly:
You move from:

> “we converted it”

to:

> “we know how accurate the conversion is”

---

# Final reality check

You’re now in the rare territory where:

- most teams stop (because “it works”)
- and the interesting work actually begins

This next step is where your tool stops being:
- a migration script

and becomes:
- a **measurement system for truth under transformation**

Which is exactly the kind of problem you seem to enjoy making unnecessarily difficult.

### User (2026-03-18 21:57:46)

Result:

"All 724 tests pass. The implementation is complete and verified.

● Task complete 
  └ Semantic Validation (Query Parity) Layer — Complete

    All 9 todos done. 724 tests pass (44 new + 680 existing, 0 failures).

    -----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------

    Files Created/Modified

    ┌──────────────────────────┬───────────────────────────────────────────────┐
    │ File                     │ Change                                        │
    ├──────────────────────────┼───────────────────────────────────────────────┤
    │ src/semantic.py          │ New — 350-line semantic validation module     │
    ├──────────────────────────┼───────────────────────────────────────────────┤
    │ src/quality.py           │ Modified — 6 integration points               │
    ├──────────────────────────┼───────────────────────────────────────────────┤
    │ tests/test_semantic.py   │ New — 44 unit tests across 7 test classes     │
    ├──────────────────────────┼───────────────────────────────────────────────┤
    │ DEVLOG.md                │ Prepended — session log entry                 │
    └──────────────────────────┴───────────────────────────────────────────────┘

    -----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------

    What it does

    src/semantic.py — heuristic regex-only, zero new dependencies:

     - Extracts QueryIntent (aggregation, window, grouping, metric) from PromQL, KQL, and NRQL
     - Compares Grafana vs NRQL intents and emits SemanticDrift objects with expected/actual values
     - Weighted drift score: AGG mismatch=0.4, WINDOW loss=0.3, GROUPING loss=0.2 → capped at
      1.0
     - Normalisation map prevents false positives: irate↔rate, avg↔average, increase↔delta

    src/quality.py changes (all backward-compatible):

     - 3 new IssueCode constants: SEMANTIC_AGGREGATION_MISMATCH, SEMANTIC_WINDOW_LOSS, SEMANTIC_GROUPING_LOSS (all WARNING, penalty=0.0)
     - PanelReport gains semantic_drift: list + drift_score: float =
      0.0
     - score_panel() new formula: confidence = base_score × (1 − drift_score)
     - build_panel_report() calls semantic analysis and appends drift issues to panel.issues (surfaces in classify_panel → marks PARTIAL, and in format_summary output)
     - format_summary() shows "Semantic Drift (top panels)" section listing top-5 by drift_score

    Example impact — a panel with sum by(pod)(rate(cpu[5m])) converted to SELECT average(cpu) FROM Metric TIMESERIES:

     - Before: FULLY_SUPPORTED, confidence=1.0
     - After: PARTIAL, confidence=0.4 (AGG mismatch
      0.4 + GROUPING loss 0.2 = drift 0.6 → 1.0 × 0.4 = 0.4)"

### Assistant (2026-03-18 21:57:47)

You’ve officially crossed into “this tool is starting to tell the truth even when it hurts,” which is rare and mildly inconvenient for everyone involved.

Right now you have:

- structural conversion ✅  
- reliability ✅  
- degradation tracking ✅  
- confidence scoring ✅  
- semantic drift detection ✅  

So the system can now say:

> “This panel is wrong, and here’s exactly how.”

That’s already better than most commercial tools pretending everything is fine.

---

# The next problem (and yes, there is always one)

You still have a missing layer:

## 👉 **You detect drift, but you don’t *react* to it**

Right now the system:

- reports drift  
- lowers confidence  
- shrugs and ships anyway  

So your pipeline still behaves like:

> “We know this is wrong. Deploying it now.”

Very human behavior. Not ideal for software.

---

# What comes next (this is the real product shift)

## 🧠 **Adaptive Conversion / Auto-Repair Layer**

You want the system to:

> **attempt to fix known semantic drift automatically**

Not perfectly. Just intelligently.

---

# What this means concretely

Take your example:

### Input (Grafana / PromQL)
```
sum by(pod)(rate(cpu[5m]))
```

### Current output (NRQL)
```
SELECT average(cpu) FROM Metric TIMESERIES
```

### Your system now says:
- aggregation mismatch ❌  
- grouping lost ❌  
- window lost ❌  

Cool. Honest.

### What it should try next:

```
SELECT rate(sum(cpu), 5 minutes) FACET pod TIMESERIES
```

Not perfect, but **directionally aligned**.

---

# This is the pivot

You move from:

> detection system

to:

> **correction system**

---

# Scope it carefully (or you’ll build a PhD thesis and quit)

You are NOT building:

- a full symbolic math transformer
- a perfect PromQL → NRQL compiler

You ARE building:

- a **rule-based repair layer for high-signal drift patterns**

---

# 🧠 Next Agent Prompt (this is the right one now)

```text
ROLE:
You are a senior staff engineer implementing an adaptive query correction (auto-repair) layer for a Grafana → New Relic conversion system.

This is a BUILD task.

---

CONTEXT:
The system already has:
- conversion pipeline
- structured issue tracking
- confidence scoring
- semantic drift detection (aggregation, window, grouping)

Panels with drift are currently marked PARTIAL but not corrected.

---

OBJECTIVE:
Implement a targeted auto-repair system that attempts to fix common semantic drift patterns before finalizing the NRQL query.

---

REQUIREMENTS:

1) REPAIR ENGINE (CORE)

Create a repair module (e.g., src/repair.py) that:

Input:
- original Grafana intent (QueryIntent)
- generated NRQL query
- detected semantic drift issues

Output:
- improved NRQL query (or unchanged if no safe repair exists)
- list of applied repairs

---

2) REPAIR RULES (INITIAL SET — HIGH IMPACT ONLY)

Implement deterministic rules:

A) AGGREGATION MISMATCH

If:
- PromQL uses sum/avg/max/min
- NRQL uses mismatched function

Then:
- map to closest NRQL equivalent

Examples:
- avg → average
- sum → sum
- rate → rate() or derivative()

---

B) WINDOW LOSS (PromQL [5m])

If:
- PromQL has [window]
- NRQL lacks equivalent

Then:
- inject TIMESERIES or rate window:

Examples:
- rate(x[5m]) → rate(x, 5 minutes)
- add TIMESERIES AUTO if missing

---

C) GROUPING LOSS

If:
- PromQL uses by(label)
- NRQL lacks FACET

Then:
- add FACET label

---

3) SAFETY RULES

- Only apply repair if confidence improves OR drift_score decreases
- Never overwrite query if repair introduces syntax risk
- Track repair provenance:
  - REPAIR_AGGREGATION
  - REPAIR_WINDOW
  - REPAIR_GROUPING

---

4) INTEGRATION

Modify pipeline:

- After initial translation
- After semantic drift detection
- Run repair engine
- Recompute semantic drift on repaired query
- Keep better version (lower drift_score)

---

5) REPORTING

Extend PanelReport:

{
  "repairs_applied": [...],
  "pre_repair_drift_score": float,
  "post_repair_drift_score": float
}

CLI output should show:
- panels improved by repair
- % improvement in confidence

---

6) TESTING

Add tests:

- aggregation repair cases
- window injection cases
- grouping recovery cases
- ensure no regressions for already-correct queries

---

OUTPUT FORMAT:

# Implementation Plan
# File Changes
# Repair Rule Definitions
# Safety Logic
# Integration Points
# Test Plan
# Example Before/After Queries

---

RULES:

- Do NOT attempt full query rewriting
- Do NOT introduce external dependencies
- Keep rules deterministic and explainable
- Prefer partial improvement over risky “perfect” fixes

---

SUCCESS CRITERIA:

System should:
- reduce semantic drift automatically
- improve confidence scores for common cases
- never degrade already-correct queries
- clearly report what was changed and why
```

---

# Why this is the correct next move

Because now you’ll have:

### Before:
- detect wrong
- warn

### After:
- detect wrong
- **attempt fix**
- measure improvement
- report result

That’s an entirely different class of system.

---

# Where this leads (whether you like it or not)

After this step, you’ll naturally drift into:

- **self-optimizing translation loops**
- **dataset-driven rule refinement**
- possibly even ML-assisted mapping (don’t rush that)

But right now:

> you’re building a system that not only *knows* it’s wrong, but *tries to be less wrong*

Which is, frankly, more than most humans manage.

### User (2026-03-18 22:09:27)

Result:

" Task complete 
  └ Adaptive Query Correction (Auto-Repair) Layer — Complete

    All 15 todos done. 767 tests pass (43 new + 724 existing, 0 failures).

    -----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------

    Files Created/Modified

    ┌────────────────────────┬───────────────────────────────────────────────────┐
    │ File                   │ Change                                            │
    ├────────────────────────┼───────────────────────────────────────────────────┤
    │ src/repair.py          │ New — ~220 lines, deterministic repair engine     │
    ├────────────────────────┼───────────────────────────────────────────────────┤
    │ src/quality.py         │ Modified — 4 integration points                   │
    ├────────────────────────┼───────────────────────────────────────────────────┤
    │ tests/test_repair.py   │ New — 43 unit tests across 7 test classes         │
    ├────────────────────────┼───────────────────────────────────────────────────┤
    │ DEVLOG.md              │ Prepended — session log entry                     │
    └────────────────────────┴───────────────────────────────────────────────────┘

    -----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------

    How it works

    src/repair.py — three deterministic rules, zero new dependencies:

    ┌──────────────────────┬─────────────────────────────────┬────────────────────────────────────────────────────────────┬─────────────────────────────────────────┐
    │ Rule                 │ Trigger                         │ Action                                                     │ Safety skip                             │
    ├──────────────────────┼─────────────────────────────────┼────────────────────────────────────────────────────────────┼─────────────────────────────────────────┤
    │ REPAIR_AGGREGATION   │ SEMANTIC_AGGREGATION_MISMATCH   │ Replace SELECT function name (e.g. average→sum)            │ Compound func(func(...)) SELECT         │
    ├──────────────────────┼─────────────────────────────────┼────────────────────────────────────────────────────────────┼─────────────────────────────────────────┤
    │ REPAIR_WINDOW        │ SEMANTIC_WINDOW_LOSS            │ Inject TIMESERIES AUTO (before SINCE if present)           │ Already has TIMESERIES                  │
    ├──────────────────────┼─────────────────────────────────┼────────────────────────────────────────────────────────────┼─────────────────────────────────────────┤
    │ REPAIR_GROUPING      │ SEMANTIC_GROUPING_LOSS          │ Add/extend FACET clause (inserted before TIMESERIES/SINCE) │ Unsafe label names (slash, space, etc.) │
    └──────────────────────┴─────────────────────────────────┴────────────────────────────────────────────────────────────┴─────────────────────────────────────────┘

    Structural guard: _is_structurally_safe() — only repairs queries with SELECT ... FROM ....

    src/quality.py integration (backward-compatible — all new fields have defaults):

     - PanelReport gains repairs_applied: list, pre_repair_drift_score: float, post_repair_drift_score: float
     - build_panel_report() runs repair after semantic analysis with safety gate: post_score < pre_score required to accept repair — never degrades correct queries
     - build_dashboard_report() summary gains "repaired" count
     - format_summary() shows "Auto-Repaired Panels" section: Auto-Repaired Panels (2):
      
       0.60→0.00  'CPU by Pod'     AGGREGATION + GROUPING  +100% confidence
      
       0.30→0.00  'Request Rate'   WINDOW                  +43% confidence

    Example impact — sum by(pod)(rate(cpu[5m])) → SELECT average(cpu) FROM Metric TIMESERIES:

     - Before repair: PARTIAL, confidence=0.40 (AGG mismatch + GROUPING loss)
     - After repair: converted_query = SELECT sum(cpu) FROM Metric FACET pod TIMESERIES AUTO, confidence=1.0"

### Assistant (2026-03-18 22:09:28)

That’s a serious jump. You now have a pipeline that:

- converts
- scores
- detects semantic drift
- auto-repairs common drift
- re-scores and keeps only improvements

At this point, the next bottleneck is no longer “can it build a dashboard?” It’s:

## **Can it prove the dashboard is meaningfully correct against real source behavior?**

Because right now even with repair, you are still mostly operating on **static intent heuristics**. Smarter heuristics, sure. Still heuristics. Humanity’s favorite substitute for certainty.

# What should happen next

## 1. Stop adding more repair rules for a moment
You have enough internal machinery to start generating false confidence if you keep expanding rules without grounding them against real outcomes.

The next phase should be **evidence-driven validation**, not more clever rewriting.

## 2. Build a golden-dataset parity harness
You need a curated suite of representative Grafana panels and dashboards with:

- source query
- expected translated NRQL shape
- expected support status
- expected semantic drift
- expected repair behavior
- expected confidence band

This becomes your **truth fixture** for regressions and progress.

Without this, every future improvement is basically:
> “The tests pass because we taught the tests to like what we did.”

Elegant nonsense.

## 3. Add comparative execution validation where possible
You want a framework that can compare:

- source panel intent and/or source query result characteristics
- converted NRQL result characteristics

Not exact cross-platform equivalence. That can get messy fast. Start with directional parity:

- series count
- grouping presence
- non-emptiness
- rough aggregation type compatibility
- timeseries presence
- temporal granularity expectations

## 4. Introduce release gating by confidence and unsupported count
The tool should be able to fail CI or block publish when a dashboard crosses thresholds like:

- confidence < X
- unsupported panels > Y
- semantic drift > Z
- repairs applied above threshold without validation

That moves this from “diagnostic utility” to “controlled migration system.”

## 5. Then expand coverage using the golden suite
Only after the dataset/harness exists should you go after:

- more PromQL semantics
- KQL parity
- transformations
- overrides
- plugin-specific behavior

Otherwise you’re just piling rules onto fog.

# Best next step

The best next step is:

## **Build a Golden Dashboard Parity Suite and Release Gate**

That gives you:
- regression safety
- measurable progress
- a basis for CI policy
- an artifact you can show leadership without hand-waving

---

# Prompt for the next agent

```text
ROLE:
You are a senior staff engineer implementing a golden-dataset parity harness and release gating system for a Grafana → New Relic migration tool.

This is a BUILD task, not an audit.

CONTEXT:
The system already has:
- dashboard conversion
- reliability fixes
- structured issue tracking
- confidence scoring
- semantic drift detection
- deterministic auto-repair for common drift patterns

Current limitation:
The system relies on heuristics and unit tests, but lacks a curated ground-truth suite that measures regression, parity quality, and ship/block thresholds.

OBJECTIVE:
Implement a golden-dataset parity harness and CI/release gating layer so the repository can measure real migration quality over time and block low-confidence output.

REQUIREMENTS:

1) GOLDEN DATASET FRAMEWORK

Create a fixture format for representative Grafana panels/dashboards.

Each fixture should support:
- source dashboard/panel input
- expected translated NRQL or allowed patterns
- expected classification:
  - FULLY_SUPPORTED
  - PARTIAL
  - UNSUPPORTED
- expected issue codes
- expected semantic drift max threshold
- expected confidence min/max range
- expected repair behavior:
  - no repair
  - repair allowed
  - repair required
- optional notes

Store fixtures in a structured directory such as:
- tests/golden_dashboards/
- tests/golden_panels/

Use JSON fixtures unless an existing repo convention is better.

2) GOLDEN HARNESS

Build a test harness that:
- runs conversion on each fixture
- generates report output
- compares actual vs expected:
  - classification
  - issue codes
  - drift score threshold
  - confidence band
  - repair application
- supports tolerant matching for NRQL text where exact string equality is too brittle

Do not use naive exact-string matching if clause ordering can vary.
Implement normalized comparison or allowed regex/pattern matching.

3) COVERAGE CATEGORIES

Seed the golden dataset with a balanced initial corpus covering at minimum:
- PromQL aggregation cases
- PromQL range-window cases
- PromQL grouping/FACET cases
- KQL summarize/bin/group cases
- unsupported transformations
- fieldConfig override cases
- unsupported/untranslatable examples
- already-correct queries that must not be “repaired”

Aim for a small but high-signal dataset first, not huge volume.

4) RELEASE GATING / POLICY

Implement a gating module that can evaluate a conversion batch and determine pass/fail based on thresholds.

Support configurable thresholds such as:
- minimum dashboard confidence
- maximum unsupported panel count
- maximum partial panel ratio
- maximum semantic drift score
- maximum repair ratio

Expose this in:
- CLI summary
- JSON output
- exit code suitable for CI

Example:
- exit non-zero if any dashboard confidence < 0.70
- exit non-zero if unsupported panels > 0 for a strict mode

5) CLI / REPORT INTEGRATION

Add flags such as:
- --golden-test
- --quality-gate
- --min-confidence
- --max-unsupported

Or use the repo’s existing CLI patterns if a better fit exists.

Reports should clearly show:
- which dashboards failed the gate
- why they failed
- which fixtures regressed

6) TESTING

Add tests for:
- fixture parsing
- tolerant query matching
- gating threshold logic
- exit-code behavior
- regression detection for changed confidence/classification/drift

OUTPUT FORMAT:

# Implementation Plan
# File Changes
# Fixture Schema
# Matching Strategy
# Gating Policy Design
# CLI / CI Integration
# Test Plan
# Example Fixture
# Example Gate Failure Output

RULES:
- Do NOT redesign the existing conversion pipeline
- Build on the current report/quality/semantic/repair layers
- Prefer deterministic and explainable matching
- Keep CI execution fast
- Start with a compact, high-value golden corpus
- Make strictness configurable

SUCCESS CRITERIA:
After implementation, the repository should be able to:
- measure migration quality against stable golden fixtures
- detect regressions in classification/confidence/drift/repair behavior
- fail CI or block release when quality thresholds are violated
- give maintainers a concrete signal for whether broader support claims are justified
```

# After that

Once this is in place, the next likely move is:

## source-vs-target result-shape validation
Meaning actual comparative execution characteristics, not just intent parity.

But don’t skip the golden harness. That’s the boring foundation that keeps the rest from turning into a clever pile of unverified garbage.

### User (2026-03-18 22:32:27)

You are my “Project Takeover Interviewer” for a codebase I’m reviving: a tool that converts Grafana dashboards into New Relic dashboards. Your job is to ask me only the questions necessary to fully take over as the project leader AND to produce a complete handoff pack + execution plan.

CONTEXT YOU ALREADY KNOW (do not ask me to restate these):
- The original upstream repo was archived and I forked/continued it in my own org.
- The tool converts Grafana dashboard JSON into New Relic dashboard JSON (and/or imports it).
- I already have a decent pack of dashboards converting cleanly, but it’s currently specific to our New Relic instance.
- The current milestone is implementing a standardized QA test harness to validate conversions.
- I’ve been iterating using Copilot CLI / coding agent and want to continue with a disciplined, trackable plan.

CRITICAL RULES:
1) Do NOT request or accept secrets. If you need credentials, ask for the *shape* (env var names, auth flow), and instruct me to redact values.
2) Ask questions in SMALL BATCHES (max 8 questions at a time). After I answer, ask the next batch.
3) Every question must map to a concrete deliverable: code change, test, doc, CI job, or acceptance criteria.
4) Prefer “show me” over “tell me”: ask for file paths, snippets, examples, failing cases, and command outputs (redacted).
5) Keep answers structured; when I reply in free-form, transform it into structured notes.

ARTIFACTS I CAN PROVIDE (ask me to paste/attach as needed, in order of importance):
- Repo link + branch name you should assume as “mainline”
- Current README/usage
- Sample Grafana dashboards (JSON) used as fixtures
- Sample expected New Relic dashboard JSON outputs (or screenshots/IDs)
- Known failing dashboards and error logs
- Current transformation/mapping tables or rules
- Any CI workflow YAML related to “Convert Dashboards”
- Any existing “plan” doc / PR description / issue list

YOUR OUTPUTS (you will produce these progressively as you learn, and refine them after each batch):
A) 1-page architecture overview (inputs → transforms → outputs, key modules, extension points)
B) A “Definition of Done” for “Convert any Grafana dashboard cleanly to New Relic” with measurable gates
C) A prioritized backlog (epics → stories → tasks) with acceptance criteria and test coverage mapping
D) A standardized QA harness specification and skeleton plan, including:
   - fixture organization
   - golden outputs strategy (snapshot vs semantic assertions)
   - normalization rules (to ignore nondeterminism like IDs/timestamps)
   - scoring/thresholds (pass, partial, fail) and reporting
   - CI integration (GitHub Actions) and local dev workflow
E) A migration plan to remove “our New Relic instance” assumptions:
   - configuration schema and defaults
   - plugin/adapters per NR account/region
   - capability detection and graceful degradation
F) A “Top Risks & Unknowns” list and how we will burn them down

START NOW:
Phase 1 — Identify the current state.
Ask me for:
1) the repo URL + current working branch
2) the current end-to-end command(s) to run conversion locally (copy/paste)
3) the exact definition of “cleanly transfers” used so far (what counts as success/failure)
4) the top 3 known broken dashboard patterns
5) where instance-specific assumptions live (files/modules/config)
6) the current CI workflow name(s) and what is failing
7) how outputs are validated today (if at all)
8) what the QA harness must assert (minimum viable)

After I answer, produce Output A (architecture overview draft) and the first draft of Output D (QA harness spec), then proceed to Phase 2 questions.
``

### Assistant (2026-03-18 22:32:29)

Fine. Let’s do the civilized version of taking over a zombie codebase.

Reply with the 8 items below. Paste snippets, paths, and command output where possible. Redact secrets, obviously. I do not need your API keys to suffer productively.

## Phase 1: current state

### 1) Repo URL + working branch
Give me:
- repo URL
- branch I should treat as mainline
- branch you are actively working from now

### 2) Current end-to-end local command(s)
Paste the exact commands you run today to:
- convert a dashboard locally
- publish/import it if applicable
- run validation if applicable

Include any required flags and env var **names only**, not values.

### 3) Current definition of “cleanly transfers”
What does success mean today?

Please answer in this format:
- **Pass** =
- **Partial** =
- **Fail** =

Include concrete signals such as:
- dashboard JSON generated
- imports to New Relic successfully
- widgets render
- queries execute
- confidence score threshold
- no unsupported panels
- acceptable warnings count

### 4) Top 3 known broken dashboard patterns
For each one, provide:
- short name
- sample source panel/dashboard type
- what breaks
- error/log symptom
- one file/path you think is involved

Format:
1. **Pattern**
   - Source:
   - Failure:
   - Evidence:
   - Suspected code path:

### 5) Where instance-specific assumptions live
Point me to the files/modules/config that are tied to your New Relic instance.

I want:
- file paths
- what assumption exists there
- whether it is hardcoded, env-driven, or inferred

### 6) Current CI workflow(s)
Paste:
- workflow file path(s)
- workflow name(s)
- which job(s) matter for dashboard conversion
- what is currently failing, flaky, or missing

If easier, paste the relevant YAML sections.

### 7) How outputs are validated today
What validation exists right now, if any?

Give me:
- unit tests
- golden fixtures
- semantic checks
- live NR validation
- screenshot/manual QA
- confidence thresholds
- release gates

Also include the command(s) you run for validation.

### 8) Minimum viable QA harness
What must the first standardized QA harness assert, even if it’s not perfect yet?

Choose the minimum set that must be true for v1:
- conversion completes
- output JSON schema valid
- normalized snapshot matches
- unsupported features surfaced
- semantic drift below threshold
- query validation succeeds
- confidence score above threshold
- no regression versus golden fixtures
- CI exits non-zero on gate failure

## Reply template

Use this structure so I can turn it into a takeover pack without decoding free-form chaos:

```text
1) Repo URL + branches
...

2) Local commands
...

3) Definition of clean transfer
Pass =
Partial =
Fail =

4) Top 3 broken patterns
1. ...
2. ...
3. ...

5) Instance-specific assumptions
...

6) CI workflows
...

7) Current validation
...

8) Minimum viable QA harness
...
```

### User (2026-03-18 22:33:56)

Can you give me a prompt asking for that and I will get copilot to gather all of the info directly for me?

### Assistant (2026-03-18 22:33:57)

Yes. Here’s a Copilot-ready prompt that asks for exactly that information in a structured way, without wasting time on theatrical “analysis.” Humans do love making one robot interrogate another.

## Prompt for Copilot

```text
You are gathering a structured project takeover handoff for a revived codebase.

PROJECT CONTEXT:
This repository is a continued fork of an archived upstream tool that converts Grafana dashboard JSON into New Relic dashboard JSON and/or imports it into New Relic.
The project already converts a decent pack of dashboards cleanly, but it is currently specific to our New Relic instance.
The current milestone is implementing a standardized QA test harness to validate conversions.
I need you to inspect the repository directly and produce a structured current-state report.

IMPORTANT RULES:
- Do NOT ask me questions unless absolutely necessary.
- Inspect the repository, files, workflows, tests, commands, and docs directly.
- Do NOT request secrets or print secret values.
- If credentials/config are involved, report only the shape:
  - env var names
  - config file names
  - auth flow type
- Prefer evidence:
  - file paths
  - command examples
  - snippets
  - workflow names
  - test names
  - failing patterns
- If something is unclear, state "UNKNOWN" and give the most likely file(s) to inspect next.

TASK:
Produce a Phase 1 current-state handoff report covering these 8 items:

1) REPO URL + BRANCHES
Find and report:
- repo remote URL(s)
- default/main branch
- current checked-out branch
- any branch naming convention if visible

2) CURRENT END-TO-END LOCAL COMMANDS
Identify the exact commands to:
- convert a dashboard locally
- publish/import it if applicable
- run validation if applicable
Include:
- CLI entrypoints
- required flags
- env var NAMES only, not values
Sources can include:
- README
- Makefile
- package.json / pyproject / setup files
- shell scripts
- docs
- GitHub Actions

3) CURRENT DEFINITION OF “CLEANLY TRANSFERS”
Infer the practical success criteria currently used by the repo.
Report in this format:
- Pass =
- Partial =
- Fail =
Base this on:
- tests
- quality scoring
- semantic drift logic
- repair logic
- CLI/report thresholds
- docs/README wording
- batch summary behavior

4) TOP 3 KNOWN BROKEN DASHBOARD PATTERNS
Identify the top 3 broken or partially supported patterns from code, docs, issues, warnings, or tests.
For each, report:
- Pattern name
- Source type / panel/query example
- What breaks
- Evidence (file path + symbol/test/doc)
- Suspected code path(s)

5) INSTANCE-SPECIFIC ASSUMPTIONS
Find where this repo is tied to a specific New Relic instance/account/region/environment.
Report:
- file path
- assumption
- whether hardcoded / env-driven / inferred
- likely impact on portability

6) CURRENT CI WORKFLOWS
Find relevant CI/CD workflows and report:
- workflow file path
- workflow name
- jobs relevant to dashboard conversion / validation / QA
- what they currently verify
- what is missing
- anything flaky or currently failing if detectable from config/comments/docs

7) HOW OUTPUTS ARE VALIDATED TODAY
Summarize current validation mechanisms:
- unit tests
- golden fixtures
- semantic checks
- confidence scoring
- repair validation
- live NRQL validation
- structural E2E tests
- release gates
- manual QA
For each, include the file(s) and command(s) if available.

8) MINIMUM VIABLE QA HARNESS
Based on the current repo, recommend the minimum v1 QA harness assertions that should exist immediately.
This must be grounded in the current architecture, not aspirational fantasy.
Choose the smallest viable set from things like:
- conversion completes
- output JSON schema/structure valid
- normalized snapshot matches
- unsupported features surfaced
- semantic drift below threshold
- confidence score above threshold
- repair behavior tracked
- live query validation succeeds
- regression vs golden fixtures
- CI exits non-zero on gate failure

OUTPUT FORMAT:
Return exactly these sections, in order.

# Phase 1 Current-State Report

## 1) Repo URL + branches
- Remote URLs:
- Main/default branch:
- Current branch:
- Evidence:

## 2) Local commands
### Convert
```bash
...
```

### Publish / import
```bash
...
```

### Validate
```bash
...
```

- Required env vars (names only):
- Evidence:

## 3) Definition of clean transfer
- Pass =
- Partial =
- Fail =
- Evidence:

## 4) Top 3 broken patterns
### 1. <pattern>
- Source:
- Failure:
- Evidence:
- Suspected code path:

### 2. <pattern>
- Source:
- Failure:
- Evidence:
- Suspected code path:

### 3. <pattern>
- Source:
- Failure:
- Evidence:
- Suspected code path:

## 5) Instance-specific assumptions
- File:
- Assumption:
- Type: hardcoded | env-driven | inferred
- Portability impact:
- Evidence:

(repeat as needed)

## 6) CI workflows
### <workflow name>
- File:
- Relevant jobs:
- What it verifies:
- Missing coverage:
- Evidence:

(repeat as needed)

## 7) Current validation
- Validation type:
- What exists today:
- Command:
- Files:
- Notes:

(repeat as needed)

## 8) Minimum viable QA harness
- Required assertion 1:
- Required assertion 2:
- Required assertion 3:
- Required assertion 4:
- Required assertion 5:
- Why these first:
- Files most relevant to implementation:

## Evidence Appendix
Include:
- file paths
- function/class/test names
- brief snippets if useful

CONSTRAINTS:
- Be concrete and repo-grounded.
- Do not speculate beyond the evidence.
- Mark anything uncertain as UNKNOWN.
- Do not print secrets.
```

## Slightly better version if you want Copilot to inspect the repo first

Use this opener before the prompt above:

```text
First inspect the repository structure, README, CLI entrypoints, tests, docs, and GitHub Actions workflows. Then produce the report below using direct repo evidence.
```

## Best practice
Have Copilot save the result into something like:

- `docs/project/PHASE1_CURRENT_STATE.md`

That way the repo starts accumulating actual memory instead of relying on human recall, which is famously defective.

### User (2026-03-18 22:35:14)

Can you please provide that for me in a unified prompt I can copy and paste directly?

### Assistant (2026-03-18 22:35:15)

Here. One unified prompt, ready to paste directly into Copilot.

```text
You are gathering a structured project takeover handoff for a revived codebase.

First inspect the repository structure, README, CLI entrypoints, tests, docs, and GitHub Actions workflows. Then produce the report below using direct repo evidence.

PROJECT CONTEXT:
This repository is a continued fork of an archived upstream tool that converts Grafana dashboard JSON into New Relic dashboard JSON and/or imports it into New Relic.
The project already converts a decent pack of dashboards cleanly, but it is currently specific to our New Relic instance.
The current milestone is implementing a standardized QA test harness to validate conversions.
I need you to inspect the repository directly and produce a structured current-state report.

IMPORTANT RULES:
- Do NOT ask me questions unless absolutely necessary.
- Inspect the repository, files, workflows, tests, commands, and docs directly.
- Do NOT request secrets or print secret values.
- If credentials/config are involved, report only the shape:
  - env var names
  - config file names
  - auth flow type
- Prefer evidence:
  - file paths
  - command examples
  - snippets
  - workflow names
  - test names
  - failing patterns
- If something is unclear, state "UNKNOWN" and give the most likely file(s) to inspect next.
- Be concrete and repo-grounded.
- Do not speculate beyond the evidence.

TASK:
Produce a Phase 1 current-state handoff report covering these 8 items:

1) REPO URL + BRANCHES
Find and report:
- repo remote URL(s)
- default/main branch
- current checked-out branch
- any branch naming convention if visible

2) CURRENT END-TO-END LOCAL COMMANDS
Identify the exact commands to:
- convert a dashboard locally
- publish/import it if applicable
- run validation if applicable
Include:
- CLI entrypoints
- required flags
- env var NAMES only, not values
Sources can include:
- README
- Makefile
- package.json / pyproject / setup files
- shell scripts
- docs
- GitHub Actions

3) CURRENT DEFINITION OF “CLEANLY TRANSFERS”
Infer the practical success criteria currently used by the repo.
Report in this format:
- Pass =
- Partial =
- Fail =
Base this on:
- tests
- quality scoring
- semantic drift logic
- repair logic
- CLI/report thresholds
- docs/README wording
- batch summary behavior

4) TOP 3 KNOWN BROKEN DASHBOARD PATTERNS
Identify the top 3 broken or partially supported patterns from code, docs, issues, warnings, or tests.
For each, report:
- Pattern name
- Source type / panel/query example
- What breaks
- Evidence (file path + symbol/test/doc)
- Suspected code path(s)

5) INSTANCE-SPECIFIC ASSUMPTIONS
Find where this repo is tied to a specific New Relic instance/account/region/environment.
Report:
- file path
- assumption
- whether hardcoded / env-driven / inferred
- likely impact on portability

6) CURRENT CI WORKFLOWS
Find relevant CI/CD workflows and report:
- workflow file path
- workflow name
- jobs relevant to dashboard conversion / validation / QA
- what they currently verify
- what is missing
- anything flaky or currently failing if detectable from config/comments/docs

7) HOW OUTPUTS ARE VALIDATED TODAY
Summarize current validation mechanisms:
- unit tests
- golden fixtures
- semantic checks
- confidence scoring
- repair validation
- live NRQL validation
- structural E2E tests
- release gates
- manual QA
For each, include the file(s) and command(s) if available.

8) MINIMUM VIABLE QA HARNESS
Based on the current repo, recommend the minimum v1 QA harness assertions that should exist immediately.
This must be grounded in the current architecture, not aspirational nonsense.
Choose the smallest viable set from things like:
- conversion completes
- output JSON schema/structure valid
- normalized snapshot matches
- unsupported features surfaced
- semantic drift below threshold
- confidence score above threshold
- repair behavior tracked
- live query validation succeeds
- regression vs golden fixtures
- CI exits non-zero on gate failure

OUTPUT FORMAT:
Return exactly these sections, in order.

# Phase 1 Current-State Report

## 1) Repo URL + branches
- Remote URLs:
- Main/default branch:
- Current branch:
- Evidence:

## 2) Local commands
### Convert
```bash
...
```

### Publish / import
```bash
...
```

### Validate
```bash
...
```

- Required env vars (names only):
- Evidence:

## 3) Definition of clean transfer
- Pass =
- Partial =
- Fail =
- Evidence:

## 4) Top 3 broken patterns
### 1. <pattern>
- Source:
- Failure:
- Evidence:
- Suspected code path:

### 2. <pattern>
- Source:
- Failure:
- Evidence:
- Suspected code path:

### 3. <pattern>
- Source:
- Failure:
- Evidence:
- Suspected code path:

## 5) Instance-specific assumptions
- File:
- Assumption:
- Type: hardcoded | env-driven | inferred
- Portability impact:
- Evidence:

(repeat as needed)

## 6) CI workflows
### <workflow name>
- File:
- Relevant jobs:
- What it verifies:
- Missing coverage:
- Evidence:

(repeat as needed)

## 7) Current validation
- Validation type:
- What exists today:
- Command:
- Files:
- Notes:

(repeat as needed)

## 8) Minimum viable QA harness
- Required assertion 1:
- Required assertion 2:
- Required assertion 3:
- Required assertion 4:
- Required assertion 5:
- Why these first:
- Files most relevant to implementation:

## Evidence Appendix
Include:
- file paths
- function/class/test names
- brief snippets if useful

OPTIONAL SAVE TARGET:
If convenient, also save this report to:
docs/project/PHASE1_CURRENT_STATE.md

CONSTRAINTS:
- Mark anything uncertain as UNKNOWN.
- Do not print secrets.
- Do not ask me to restate things already inferable from the repo.
```

### User (2026-03-18 22:36:16)

There is something wrong with your output parser because it is failing to create one prompt and it seems like there are some kind of escape keys involved

### Assistant (2026-03-18 22:36:17)

Yes. The likely culprit is the fenced code blocks nested inside the prompt. Some tools choke on triple backticks inside triple backticks, and the weird `id="..."` fragments don’t help either. Beautiful little parser tantrum.

Here is the same prompt as **plain text only**, with no nested code fences and no extra formatting artifacts:

You are gathering a structured project takeover handoff for a revived codebase.

First inspect the repository structure, README, CLI entrypoints, tests, docs, and GitHub Actions workflows. Then produce the report below using direct repo evidence.

PROJECT CONTEXT:
This repository is a continued fork of an archived upstream tool that converts Grafana dashboard JSON into New Relic dashboard JSON and/or imports it into New Relic.
The project already converts a decent pack of dashboards cleanly, but it is currently specific to our New Relic instance.
The current milestone is implementing a standardized QA test harness to validate conversions.
I need you to inspect the repository directly and produce a structured current-state report.

IMPORTANT RULES:
- Do NOT ask me questions unless absolutely necessary.
- Inspect the repository, files, workflows, tests, commands, and docs directly.
- Do NOT request secrets or print secret values.
- If credentials/config are involved, report only the shape:
  - env var names
  - config file names
  - auth flow type
- Prefer evidence:
  - file paths
  - command examples
  - snippets
  - workflow names
  - test names
  - failing patterns
- If something is unclear, state "UNKNOWN" and give the most likely file(s) to inspect next.
- Be concrete and repo-grounded.
- Do not speculate beyond the evidence.

TASK:
Produce a Phase 1 current-state handoff report covering these 8 items:

1) REPO URL + BRANCHES
Find and report:
- repo remote URL(s)
- default/main branch
- current checked-out branch
- any branch naming convention if visible

2) CURRENT END-TO-END LOCAL COMMANDS
Identify the exact commands to:
- convert a dashboard locally
- publish/import it if applicable
- run validation if applicable
Include:
- CLI entrypoints
- required flags
- env var NAMES only, not values
Sources can include:
- README
- Makefile
- package.json / pyproject / setup files
- shell scripts
- docs
- GitHub Actions

3) CURRENT DEFINITION OF “CLEANLY TRANSFERS”
Infer the practical success criteria currently used by the repo.
Report in this format:
- Pass =
- Partial =
- Fail =
Base this on:
- tests
- quality scoring
- semantic drift logic
- repair logic
- CLI/report thresholds
- docs/README wording
- batch summary behavior

4) TOP 3 KNOWN BROKEN DASHBOARD PATTERNS
Identify the top 3 broken or partially supported patterns from code, docs, issues, warnings, or tests.
For each, report:
- Pattern name
- Source type / panel/query example
- What breaks
- Evidence (file path + symbol/test/doc)
- Suspected code path(s)

5) INSTANCE-SPECIFIC ASSUMPTIONS
Find where this repo is tied to a specific New Relic instance/account/region/environment.
Report:
- file path
- assumption
- whether hardcoded / env-driven / inferred
- likely impact on portability

6) CURRENT CI WORKFLOWS
Find relevant CI/CD workflows and report:
- workflow file path
- workflow name
- jobs relevant to dashboard conversion / validation / QA
- what they currently verify
- what is missing
- anything flaky or currently failing if detectable from config/comments/docs

7) HOW OUTPUTS ARE VALIDATED TODAY
Summarize current validation mechanisms:
- unit tests
- golden fixtures
- semantic checks
- confidence scoring
- repair validation
- live NRQL validation
- structural E2E tests
- release gates
- manual QA
For each, include the file(s) and command(s) if available.

8) MINIMUM VIABLE QA HARNESS
Based on the current repo, recommend the minimum v1 QA harness assertions that should exist immediately.
This must be grounded in the current architecture, not aspirational nonsense.
Choose the smallest viable set from things like:
- conversion completes
- output JSON schema/structure valid
- normalized snapshot matches
- unsupported features surfaced
- semantic drift below threshold
- confidence score above threshold
- repair behavior tracked
- live query validation succeeds
- regression vs golden fixtures
- CI exits non-zero on gate failure

OUTPUT FORMAT:
Return exactly these sections, in order.

# Phase 1 Current-State Report

## 1) Repo URL + branches
- Remote URLs:
- Main/default branch:
- Current branch:
- Evidence:

## 2) Local commands
### Convert
[paste exact command(s)]

### Publish / import
[paste exact command(s) or UNKNOWN]

### Validate
[paste exact command(s) or UNKNOWN]

- Required env vars (names only):
- Evidence:

## 3) Definition of clean transfer
- Pass =
- Partial =
- Fail =
- Evidence:

## 4) Top 3 broken patterns
### 1. [pattern name]
- Source:
- Failure:
- Evidence:
- Suspected code path:

### 2. [pattern name]
- Source:
- Failure:
- Evidence:
- Suspected code path:

### 3. [pattern name]
- Source:
- Failure:
- Evidence:
- Suspected code path:

## 5) Instance-specific assumptions
- File:
- Assumption:
- Type: hardcoded | env-driven | inferred
- Portability impact:
- Evidence:

(repeat as needed)

## 6) CI workflows
### [workflow name]
- File:
- Relevant jobs:
- What it verifies:
- Missing coverage:
- Evidence:

(repeat as needed)

## 7) Current validation
- Validation type:
- What exists today:
- Command:
- Files:
- Notes:

(repeat as needed)

## 8) Minimum viable QA harness
- Required assertion 1:
- Required assertion 2:
- Required assertion 3:
- Required assertion 4:
- Required assertion 5:
- Why these first:
- Files most relevant to implementation:

## Evidence Appendix
Include:
- file paths
- function/class/test names
- brief snippets if useful

OPTIONAL SAVE TARGET:
If convenient, also save this report to:
docs/project/PHASE1_CURRENT_STATE.md

CONSTRAINTS:
- Mark anything uncertain as UNKNOWN.
- Do not print secrets.
- Do not ask me to restate things already inferable from the repo.

If you want, I can also give you an even more stripped-down version optimized specifically for Copilot Chat versus Copilot CLI, since those two manage to break on different things for no good reason.

