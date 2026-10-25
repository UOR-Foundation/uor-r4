# REPORT: infra-loop-harness, claude-93

Lab: claude. Order: [ORDER.md](ORDER.md).
References #2904. References #3179.

## What did not work, first

- **The claude-88 scope put the tool in the crate, which blocked the claim.** It listed five globs under `crates/uor-r4-discrete`, which overlap the `crates/uor-r4-discrete/**` of Antigravity's open #3180 (A8 qualification). So `loop claim` check (d) refused the item, and `loop next` offered an M2 item instead. That would have been harmless while the item gated nothing. It gates the M1 path once claude-91 and claude-92 make Stage T depend on it.
- **The sync was not run in this unit.** `cargo xtask roadmap sync` would also refresh #2833 and #2835, which claude-90 changed on `main`. That is outside this ORDER, so the sync runs from `main` after the merge, where its plan is only `main`'s state.

## What changed

Size: 1 item file (+8/−2) and this task folder (ORDER 51, REPORT 40 lines). The rendered roadmap page is unchanged by this edit (`render --check` current). No code.

- **Scope:** `docs/roadmap/items/m1-w2-code-utilization.md` now has `[docs/harness/prep/m1-w2-code-utilization*, docs/labs/m1-w2-code-utilization/**, docs/loop/tasks/m1-w2-code-utilization/**]`, with no crate glob.
- **"How to start":** it now names `docs/labs/m2-fresh-heldout-panel/instrument/Cargo.toml` as the pattern, and marks `w2_capacity.rs` and `w2-assemble.rs` as read-only references.
- **"Rules that apply":** a new rule. The tool is the standalone crate `docs/labs/m1-w2-code-utilization/instrument/`, with a detached `[workspace]`, its own `Cargo.lock` and a read-only path dependency on `uor-r4-discrete` (built without `cuda`). A needed change in `uor-r4-discrete` is a separate item, and the exact-integer rule applies to the instrument too.
- **Unchanged:** the bar, status, dependencies, done-when and sheet.

## Measurements

None. This is a docs unit.

## Checks run at the head

Recorded in the PR's review record. The review is also the decision review (D68), because this is a scope change.

## Skills used

- holding-the-goal: the diagnostic now gates Stage T, so the scope that let the claim guard hold it back was the limiter.
- scope-guard: the item's tool moves out of a crate that other M1 orders hold, instead of widening around the guard.

## Still open

- After the merge: `roadmap sync --apply` from `main` (#3179's body; #2833 and #2835 follow `main`), then `loop next` for claude.
- The implementation unit, claude-94: build the instrument, run it on the stored `run-b.tar`, and record the bands.

## Plain summary

The diagnostic's tool was going to live inside the model crate that another lab is working in right now, so the loop would not let anyone start it. The diagnostic only reads saved data, so its tool can live in its own small crate beside the lab notes and use the model crate without changing it. This PR moves it there, so the work can start now, alongside the other lab's.
