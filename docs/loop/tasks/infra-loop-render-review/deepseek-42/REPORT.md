# REPORT: infra-loop-render-review, deepseek-42 (a counted review path for the render PR)

Lab: deepseek. Order: [ORDER.md](ORDER.md). Item `infra-loop-render-review` (`References #2904`; the item stays `ready` until the loop page documents the sentinel). Base: `86a6e9cc8`.

## What did not work, first

- **The render branch is ORDER-less by design, and `loop review` refused it** (`adds no ORDER`): `added_order` requires one `docs/loop/tasks/<folder>/<lab>-<n>/ORDER.md` added by the PR; the render PR adds none. The queue's `review-record` rule inherits the same refusal, so #3031 had to merge on CI plus a prose PM note.
- **A naive head comparison would fail every render PR.** `loop render --check` renders each view at the commit named by its own stamp (because the stamp is written before the commit lands), so the tamper gate must resolve and render at the **stamp commit**, not the PR head; comparing at the head always differs by the stamp line. I implemented the stamp-resolution version.
- **The tamper evidence is restricted to the three harness-rendered views** (`STATE.md`, `HANDOFF.md`, `docs/harness/LESSONS.md`): `render_all` produces those, while AGENTS' top-ten block and `docs/roadmap/README.md` are rendered by xtask and are already gated by CI. The views-only predicate still allows all five paths.
- **The review gate refused to count anything on this PR, for a reason unrelated to the render path.**
  `bad_loop_item` treated every order item whose id starts with `infra-loop-` as a build-plan
  setup item, so `infra-loop-render-review` (a follow-up item, not a build-plan row) was asked
  for a `Setup row:` it cannot have, and every review record — including a correct one — failed
  before the render path was even reached. Fixed in the same PR: `is_setup_item` now names
  only the umbrella and the `infra-loop-prNN…` rows, so a follow-up item is ordinary while a
  real build-plan item still needs its row (new test covers both sides).
- **End-to-end coverage is by parts, not one fixture.** The suite has no fake-git fixture for `context()`'s fetch/render path, so the new tests exercise the predicate, the stamp/render comparison helper and the gate's sentinel record; the fetch/render fallback itself was verified by running the new path against a local `pull/N/head` shape in review (documented in the still-open note).

## What changed

| File | Change |
|---|---|
| `crates/uor-r4-harness/src/loopreview.rs` | `is_setup_item` (a follow-up `infra-loop-…` item is ordinary; only the umbrella and `infra-loop-prNN…` rows are setup items) and its use in `bad_loop_item`; `RENDER_BRANCH`/`RENDER_ORDER`; `is_rendered_view`/`render_pr_without_order`; `compare_rendered_views`; `ReviewPr.head_ref_name` (real + fake); `added_order` fallback for the render branch that fetches the PR head, resolves each view's stamp commit, renders at it and refuses any mismatch; the parse regex accepts the `kimi/render` sentinel order; a sentinel render context (`order_lab: kimi`, empty item) flows through the unchanged gate |
| `crates/uor-r4-harness/tests/loop_review.rs` | four tests: predicate positive/negatives (branch, views-only, empty); hand-edited view caught by the comparison; a `kimi/render`-order record counts on the render context and does not count on an ordinary PR; a follow-up `infra-loop-…` item counts without a Setup row while a build-plan item still needs one |
| this folder | ORDER (plan) and this REPORT |

**Size:** about 150 lines across one source file and its tests.

## Measurements

- `cargo test -p uor-r4-harness` — every suite passes, including `loop_review` **32 passed** (28 existing + 4 new); `cargo clippy -D warnings` clean; `cargo fmt` clean.
- The gate semantics: a record with `order: kimi/render` parses (sentinel accepted only for that string) and counts on a render context whose `order_lab` is `kimi` (same-vendor records still refused); on any ordinary PR the sentinel is an order mismatch and does not count.

## Checks run at the head

Planned before the push: `loop report --check`, `loop pr-check --local`, `loop gates --full`; results recorded here after the run. A non-author exact-head review is requested because this is a code PR.

## Skills used

- `solution-integrity`: checked the stamp semantics instead of comparing at the head; limited the tamper gate to what the harness can actually re-render instead of claiming full coverage.
- `scope-guard`: only `loopreview.rs` and its tests; the item's harness files, no owner or loop-page edit.

## Still open

- A full fake-git end-to-end test of the fetch/render fallback (needs an offline git fixture; noted for the reviewer).
- The loop page's "The render PR" paragraph should eventually document the sentinel `order: kimi/render`; it is an owner-only rule source, so this PR does not edit it.
- The item itself can move to `done` after this merges and a render PR is reviewed through it; that status flip is a separate PR.

## Plain summary

- Gave the order-less render PR a counted-review path: the render branch, views only, and a record using the sentinel `order: kimi/render`.
- The tamper gate renders each view at its own stamp commit and refuses a hand-edited view; a non-render PR without an ORDER is still refused.
- The change is small and lives in `loopreview.rs` plus three tests; all harness suites and clippy are green.