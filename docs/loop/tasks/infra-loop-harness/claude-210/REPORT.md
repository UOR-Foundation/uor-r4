# REPORT: infra-loop-harness, claude-210 (#3416 in the issue map; a loop-guard change withdrawn)

Lab: claude. Order: [ORDER.md](ORDER.md). Setup row: 2. References #2028.

**Result: the roadmap's issue map gains `m1-heldout-phrasing-transfer` → #3416.** The loop-guard change this unit also carried is **withdrawn**. The guard is left exactly as it is on `main`.

**Speed.** None measured.

## What did not work, first

- **The guard change opened real holes, so it does not land.** It was made at the owner's request ("change the harness to allow this"): heredoc bodies as data, variables the command sets, and owner paths opened by a recorded approval.
  - **What the review found:** 13 commands that write `CHARTER.md` with no owner approval, which the old guard refused:
    - a conditional or subshell reassignment fooling the variable tracking;
    - a `<<` inside a comment or arithmetic hiding real commands;
    - a quoted delimiter ending the body late;
    - shells and runners (`"bash"`, `\bash`, `$SHELL`, `csh`, `git apply`) that the body check missed.
  - **What was done instead:** the probes were written as refusal tests, and 13 of 15 were allowed. Doing it safely needs a real shell lexer, and the owner then asked for the tooling to stop: "stop with the fucking tooling and go back to the actual project". So `loopguard.rs` and its tests are restored to `origin/main`, and nothing in the guard changes.
- **The roadmap sync ran in the wrong place.** `roadmap sync --apply` was run from the shared main checkout and wrote its two files there. R10 refuses a lab's restore. The same two lines land here. The main checkout's copies equal them; Casey can restore them with `git -C ~/uor-r4 checkout -- docs/roadmap`.
- **The harness suite is red on `main`, from before this unit.** `preprules::tests::the_real_sheets_pass_the_shape_rule` fails on two of claude's prep sheets (claude-205 and claude-206), which lack the sheet rule's headings. The fix is a heading rename in those sheets, inside `one-model-end-to-end`'s scope. It rides with that item's next unit.

## What changed

- **`docs/roadmap/issues.json`:** `"m1-heldout-phrasing-transfer": 3416`, as the sync wrote it.
- **`docs/roadmap/README.md`:** that item's issue in its row.
- **This task folder.**

## Measurements

None.

## Checks run at the head

- `cargo xtask roadmap check`: 112 items OK.
- `loop report --check`, `loop pr-check --local` and `loop gates --full`: run before review.

## What it unlocks

`loop claim` and `loop close` find #3416 for `m1-heldout-phrasing-transfer`.

## Skills used

None was invoked this unit.

## Still open

- **The two sheets' headings,** with `one-model-end-to-end`'s next unit.
- **Casey's restore** of the main checkout's two roadmap files.
- **The guard's three refusals are unchanged.** A fix needs a real shell lexer, which is not scheduled.

## Plain summary

This lands one line that links a new milestone task to its issue. An attempt to loosen the command safety checker was reviewed, found to open real holes, and dropped. The checker stays as it was, and the work goes back to the model.
