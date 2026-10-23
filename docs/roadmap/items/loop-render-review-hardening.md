---
id: loop-render-review-hardening
title: Harden the render-PR review path
milestone: infra
area: tooling
status: ready
open_to: lab
needs_gpu: false
depends_on: [infra-loop-render-review]
scope: [crates/uor-r4-harness/src/loopreview.rs, crates/uor-r4-harness/src/loopprcheck.rs, crates/uor-r4-harness/tests/loop_review*.rs, docs/roadmap/items/*.md]
---

## Why it matters

The render-PR review path (#3035) closes the last ORDER-less merge hole, but two reviews recorded the gaps left open on purpose: the gate has no end-to-end test that fails when its render check is removed, the xtask-rendered `docs/roadmap/README.md` is still outside the no-ORDER path, and the loop page's "The render PR" paragraph does not yet name the sentinel `order: kimi/render`.

## What is missing

1. **No fake-git `context()` fixture pins the gate.** Replacing the `verify_render_views` call with a no-op, or disabling the `stamp != base` check, still leaves every `loop_review` test green (each run was demonstrated on #3035). A fixture should fail in both cases, and also for a cross-repo PR, a removed `ReviewContext::of` fallback, an AGENTS edit outside the block, and a hand-edited top-ten block.
2. **`docs/roadmap/README.md` is excluded from the no-ORDER path** until it has its own tamper comparison (an xtask render equality check at the stamp, like the AGENTS block gate).
3. **The loop page wording** ("The render PR") is an owner-only rule source: it should say the branch is `kimi/render`, the views are the four covered paths, and the record uses `order: kimi/render`; the PM/owner must make that edit.

## Done when

A `context()` fake-git fixture fails when the render check is removed and when `stamp == base` is disabled, and covers the cross-repo, fallback-removed, AGENTS-outside-block and hand-edited-block cases; `docs/roadmap/README.md` is either gated by an xtask render comparison at its stamp or explicitly exempted with a recorded reason; and the loop-page wording change is either made by the owner or carded. The planted fault turns the new fixture red: **a no-op render check must fail the suite.**

## How to start

1. Read `crates/uor-r4-harness/src/loopreview.rs` (`context`, `verify_render_views`, `top_ten_markers`) and the fake-gh format (`REVIEW_GH_FAKE_ENV`) in the same file.
2. Build the fixture as a test that creates a temporary git repository with a merge base, a render commit and a PR head, plus a fake gh file; run `context()` through it.
3. For the roadmap README, reuse the xtask render path at the stamp commit or record why it stays out.

## Rules that apply

- Delivery path (CHARTER §5.5): PR, exact-head review from another vendor's lab, merge queue, verify on fresh `main`.
- Rust first (CHARTER §3.11); tests stay offline (no network in the fixture).

## Sources

- [`docs/roadmap/items/infra-loop-render-review.md`](infra-loop-render-review.md) (#3035 `6e9d6db41`, #3038 `885df8f3e`)
- reviews on #3035 (Claude 06:26Z, Kimi 05:24Z) and the follow-ups carded on #2028 comment 6358961404
