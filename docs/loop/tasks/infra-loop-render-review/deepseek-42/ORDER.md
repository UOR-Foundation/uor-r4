# ORDER: infra-loop-render-review, deepseek-42 (a counted review path for the kimi/render PR)

Lab: deepseek. Item: [infra-loop-render-review](../../../../roadmap/items/infra-loop-render-review.md) (#2904's new item). Base: `86a6e9cc8`. Item PR (partial: `References #2904`; the item stays `ready` until the render path is implemented).

## Goal

Give the steady-state render PR (branch `kimi/render`, loop page step 14; `docs/loop/README.md:91`) a counted-review path, so it stops merging on green CI plus a prose PM comment. The item allows either half: `loop review` accepts the render branch without an ORDER, or the queue exempts it — behind the same two gates: `loop render --check` and the views-only rule. The planted fault must turn the check red: a hand-edited `STATE.md` on `kimi/render` fails review.

## Cancel probe (run first)

Class **instrument** (harness code + tests; no GPU). At the base: `kimi/render` has no ORDER (branch exists by design?), so `loop review --check` on #3031 failed `adds no ORDER`; `loop queue` refused it; there is no render-branch special case in `loopreview.rs`/`loopprcheck.rs`. Task stands.

## Files in scope

- `crates/uor-r4-harness/src/loopreview.rs` (branch special case, views-only predicate, tamper comparison)
- `crates/uor-r4-harness/tests/loop_review*.rs` (positive/negative/tamper tests)
- `docs/roadmap/items/*.md` (the item's own status only, if needed)
- this task folder

## Design (small, per the item's how-to-start)

1. `ReviewPr` gains `head_ref_name` (`headRefName`), filled by `RealReviewGh::pr` and the fake.
2. `pub fn render_pr_without_order(head_ref, files) -> bool`: true iff the branch is exactly `kimi/render` and every changed path is a rendered view (`STATE.md`, `HANDOFF.md`, `docs/harness/LESSONS.md`, plus `AGENTS.md`/`docs/roadmap/README.md` which are CI-rendered).
3. `context()`: try `added_order`; when it returns `Missing` and the render predicate holds, fetch the PR head (`git fetch origin pull/<n>/head`), resolve `AtCommit::resolve(root, "FETCH_HEAD")`, and compare `looprender::render_all` expectations for STATE/HANDOFF/LESSONS against the PR's texts (`gh.file_at_pr`). Any difference, or a fetch/render failure, refuses (**the tamper gate**). With clean texts, synthesize a render context: sentinel order path `kimi/render`, empty item, no setup row.
4. `gate`/`refusal`: a record on the render sentinel is accepted when the PR is the render branch and the sentinel matches; the ordinary ORDER/loop-item rules are skipped only for that sentinel. `loop queue` then counts the record through the existing `review-record` rule (no queue change).
5. Tests (pure where possible): the predicate (render branch + views-only vs. a code path, vs. a non-render branch); the tamper comparison (identical → pass; one hand-edited byte → fail naming the view); a gate test that a render-branch record with `order: kimi/render` counts while a non-render record without a path still fails.

## Done when

- A `uor-review v1` record with `order: kimi/render` counts on a `kimi/render` PR whose changed paths are only rendered views and whose STATE/HANDOFF/LESSONS match `loop render` at the PR head; no ORDER is needed.
- A non-render PR without an ORDER is still refused; a hand-edited view on `kimi/render` is refused by the tamper gate (planted fault).
- `cargo test -p uor-r4-harness` (targeted review tests + the existing suite), `loop report --check`, `loop pr-check --local`, `loop gates --full` pass at the head; a non-author exact-head review is requested.

## Stop and ask if

- The render comparison would need a new dependency or network at test time (tests must stay offline; fake the gh reads, use the local git object store only).
- Changing `pr.head_ref` plumbing would touch files outside the scope above (it does not: only `loopreview.rs` and its tests are expected).