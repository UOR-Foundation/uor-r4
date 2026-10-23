# ORDER: loop-render-review-hardening, deepseek-46

Lab: deepseek. Item: [loop-render-review-hardening](../../../../roadmap/items/loop-render-review-hardening.md). Base: `0af044c7f`.

## Goal

The item's done-when reads:

> A `context()` fake-git fixture fails when the render check is removed and when `stamp == base` is disabled, and covers the cross-repo, fallback-removed, AGENTS-outside-block and hand-edited-block cases; **`ReviewContext::of` refuses a cross-repository PR named `kimi/render`**; the documented merge-main-forward refresh passes without hand-special-casing (either `loop render` defaults the stamp to the merge base on `kimi/render`, or the loop-page wording requires `--at <merge-base>`); `docs/roadmap/README.md` is either gated by an xtask render comparison at its stamp or explicitly exempted with a recorded reason; and the loop-page wording change is either made by the owner or carded. The planted faults turn the new fixtures red: **a no-op render check must fail the suite, and a fork PR on `kimi/render` must not pass the queue mirror.**

## Cancel probe (run first)

To fill in.

## Files in scope

- `crates/uor-r4-harness/src/loopreview.rs`
- `crates/uor-r4-harness/src/looprender.rs`
- `crates/uor-r4-harness/tests/loop_review*.rs`
- `docs/roadmap/items/*.md`

## Done when

To fill in.

## Stop and ask if

To fill in.
