# REPORT — public research guide and issue reconciliation

Lab: codex. Item: [docs-status-refresh](../../../../roadmap/items/docs-status-refresh.md). References #3380.

## What did not work

The published Studio catalog offered a synthetic fixture. A model-only URL import verified the M1 stack while leaving its text tokenizer unbound. The public guide now distinguishes artifact availability, deployment and accepted-model parity. A separate Studio change pairs the immutable public files and exercises the rendered predictor. Local mdBook 0.4 produced a different page count from the publication toolchain. The pinned Mermaid plugin still warns about mdBook 0.5.0 versus 0.5.4; #3972 tracks that warning.

## What changed

The README and book entry points lead with the research question, the M1 weights and measured baselines, the D87 comparison and the separate LexLean formalization. The editorial illustration is AI-generated and depicts a physical geometric model. The M1 numbers and diagrams retain their original scope. The cross-window and vector-read descriptions now link the merged outcome.

Reviewed all 52 issues open when the unit started. The roadmap now registers previously standalone defects against their existing IDs, preserves the pilot chronology, and identifies the frozen-reference experiment (#4063) and shared interfaces (#4064). The parent, seven milestone bodies and compute board preserve the earlier descriptions as dated history. #3801 closed on delivered instrumentation. #2846, #3450, #3462 and #3594 closed as not planned, with their historical evidence and revival conditions retained. The M1 milestone closed after its final untriggered fallback was moved. Source, render and live tracker have distinct verification steps.

## Measurements

The issue census counted 52 originally open issues and found 156 source-backed issue mappings after registration. The historical pilot text contains 842 lines and 56,170 preserved bytes.

## Checks run at the head

- 156 roadmap items validated and mapped. Roadmap render is current. Before publication, the reviewed sync preview proposed 47 existing issue edits, six child-checklist updates and zero new issues or automatic closures.
- Pinned mdBook 0.5.4 and mdbook-mermaid 0.17.1 built the book. The 241 xtask tests passed. The built link check counted 107 HTML pages and zero broken links. The claim-wording gate passed.
- Compared the preserved CPU pilot text with its source, checked the new local links, and ran the branch diff whitespace check. No research execution or scientific bar changed in this unit.

## Skills used

The parallel audit skill separated issue reconciliation, PR review and live Studio checks. The image-generation skill supplied the editorial illustration.

## Plain summary

The research guide now gives a reproducible M1 example and a dated account of the next experiment. The roadmap records what was delivered, superseded or still requires a measurement.

## Still open

Publication uses the protected branch, followed by the existing docs-site workflow and a live URL check. The current M1 release and the synthetic demo are separately identified in the Studio change. Milestone verification remains with the original registered experiments.
