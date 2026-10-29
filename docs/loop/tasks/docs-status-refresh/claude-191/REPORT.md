# REPORT: docs-status-refresh, claude-191 (every status page, picture and tracker to 29 Oct 2026)

Lab: claude. Order: [ORDER.md](ORDER.md). Closes #3380.

The owner's direction (29 Oct, PLAN.md): a full documentation update, with "these latest decisions from last night and the planned path forward", efficiently and without sacrificing correctness. This is a documentation unit: no model and no run.

**Speed.** No serving or training change.

## What did not work, first

- **A maker fits about three pages.** A subagent starts near 140k tokens of context.
  - Of the first four makers, three handed back part-way: front door 3 of 10 pages, model 1 of 6, figures nearly done.
  - Each handback listed exactly what was left, and smaller follow-up makers took those lists.
- **Suborders bound to the wrong maker.** The guard binds a starting subagent to the lowest unclaimed numbered suborder, and a finished maker frees its file. That raced twice:
  - one maker took the figures' order and could write nothing;
  - a crate maker's order was moved aside while it ran, and it was refused every write.

  The fix was to copy each order to `orders/<agent id>.md` right after the maker started; the guard checks that name first. No write went outside a maker's files.
- **One write outside scope sits uncommitted.** `cargo fmt -p xtask` reformatted one test in `xtask/src/figures_charts.rs`, which is outside this item's scope.
  - The guard refuses to restore an out-of-scope file, so it stays in the worktree, never added, and goes when the worktree is removed.
  - `main`'s `figures_charts.rs` is not rustfmt-clean as committed.
- **`cargo clippy -p xtask -D warnings` fails on `main`'s code, not this unit's:** `book_stage.rs:342` (`map_or`) and `docs_relink.rs:249` (`repeat().take()`). Not fixed here, since both files are outside scope.
- **The built site has 2 broken links, both from before this unit.** They come from `docs/research/notebook/m4-serving.md`, antigravity-96's row (#3287), which links a `docs/harness/prep/` page the book does not stage. The notebook is outside scope and is left alone.

## What changed

**Trackers**
- **`cargo xtask roadmap sync --apply`,** from this tree, which equals `main` outside the refresh's pages.
  - It created 15 issues, #3380–#3394. They cover every item opened since 27 Oct and this one.
  - It closed 9 of them as `done`: #3381–#3384 and #3386–#3390.
  - It updated 7 issue bodies whose item text had changed: #2820, #2837, #2841, #2858, #2861, #2873 and #2904. Of these, #2873's labels follow its item (`+help wanted`, `−status:held`).
  - It updated 3 milestone checklists: #2029, #2031 and #2032.
- **`docs/roadmap/issues.json`** now maps all 110 items. `docs/roadmap/README.md` is re-rendered.
- **This item** is set `done`.

**Pictures**
- **The generated status figures** (`xtask/src/figures.rs`, `DATE` "29 October 2026"):
  - architecture, both layouts: the FBS trunk, one part per card;
  - milestones;
  - the training and serving pipelines;
  - the geometry stack;
  - the titles of the compliant path and the operators.

  The unused `St::Frozen` status key was removed. No figure drew it after the milestones change, and clippy flags dead code.
- **The hand-drawn figures:**
  - `model/pipeline.svg`, `model/integer-export.svg` and `model/discrete-learning-loop.svg`;
  - `start/roadmap.svg` (the order of work), `start/hero-how-it-thinks.svg` and `start/transformer-vs-geometry.svg`;
  - `mechanisms-family.svg`. Its tags now match the board: Hamilton gate and SpiralCore/FBS are ACTIVE, Ternary/XOR and Spin-native weights READY. The legend reads 12 / 4 / 11 over its 27 nodes; it draws Hopf as one node, which the board splits in two.

**Pages, each with a dated "29 Oct 2026" status where the page tracks status**
- **Front door:** `README.md`, `docs/README.md`, and `docs/start/*`:
  - what UOR-R4 is, the transformer comparison, where we are, where we are going, the FAQ;
  - glossary entries for the Hamming read, exact turn memory, whole-table nearest, `lut-stack/5` and `/6`, D80/D81, route + exact memory and the fixed-order backward;
  - the layout and the FILEMAP rows for `uor-r4-float-train`.
- **The model:** `docs/model/architecture.md`, `training.md`, `serving-and-export.md` and `pipeline-map.md`.
  - The map's stages 4, 6, 9 and 10, gaps 1 and 5, and the order of work are brought up to claude-189.
  - "What changed" gains rows for claude-161, 179, 182, 185/186 and 188/189.
- **Mechanisms:**
  - the board `docs/mechanisms/README.md`, with a composition section;
  - `flock-read`, `ternary-xor` (the Hamming read), `hamilton-gate`, `spiralcore-fbs` and `admission`.
- **Research:**
  - `docs/research/README.md`, `findings.md`, `open-questions.md` (two questions answered, six added) and `history.md` (a 27–29 Oct entry);
  - the research lines `README.md`, `m1-float-rung0.md`, `m4-serving.md` and `m2-exact-memory.md`.
- **Products and geometry:**
  - `docs/products/README.md`, `language-model.md`, `web-client.md` and `desktop-client.md`;
  - `docs/geometry/spiralcore.md` and `vsa-and-hamming.md`.
- **Crates:**
  - a new `crates/uor-r4-float-train/README.md`, for the FBS trunk's crate, which had none;
  - a status section in `crates/uor-r4-integer/README.md`;
  - one line in `crates/uor-r4-discrete/README.md`.

**The facts sheet.** Every maker checked its pages against one sheet of 29 Oct facts, each naming its unit (`local/claude/docs-refresh/FACTS.md`, kept off the tree). The sheet's rules:
- use only facts it lists;
- leave a claim it does not cover;
- change no bar or milestone status;
- keep dated history;
- check every new link.

## Decisions logged for the owner (each reversible)

- **The board moves Hamilton gate and SpiralCore/FBS from READY to ACTIVE,** and ACTIVE now reads "on a current model path: the discrete path or the float-trained FBS trunk".
  - Why: READY is "built and tested, but not yet trained into a model", and both are trained and served in the FBS trunk. The map's "Two model paths" is the basis.
  - This is the top session's decision review under AGENTS.md. To reverse it, restore the board's two rows and its ACTIVE definition.
- **Dated, not deleted:** `training.md`'s "Training is held" and "No float sheet is registered" now read "As of 19 October". Pages that said training waits on an owner release now say runs go ahead on the lab's own pre-run review (AGENTS.md, CHARTER §5.1).
- **`St::Frozen` removed** from the figure generator's status keys (above).

## Measurements

- **Issues:** 15 created, 9 closed, 7 updated and 3 checklists changed, as the dry run planned (`roadmap sync plan: 0 labels to create, 15 issues to create, 7 to update, 9 to close, 3 milestone checklists to change`).
- **Links** (`cargo xtask links`):
  - the whole tree: 23,058 checked and 1,198 broken;
  - the pages this unit changed: 0 broken. Each broken link's source was matched against the changed files.
- **The site:**
  - `cargo xtask book-stage`: 104 pages and 70 figure files;
  - `mdbook build`: 0 errors;
  - `cargo xtask book-check`: 2 broken, both from before this unit (above).
- **Figures:**
  - `cargo xtask figures`: all 8 generated figures current;
  - `figures-charts`: all 4 current;
  - `figures-math`: all 5 current;
  - `xmllint --noout`: every hand-edited SVG passes.
- **Agents:** 15 makers and the reviews below. That compares with about 150 for the 9 Oct cleanup.

## Checks run at the head

__CHECKS__

## Skills used

- **holding-the-goal.** One facts sheet bounded every maker, so no page could state a fact the sheet did not source. A maker that ran out of room handed its exact remainder to the next one, rather than starting over.

## Still open

- **Pages outside this item's scope:**
  - `docs/mechanisms/exact-memory.md` and `object-store.md`, held by antigravity-113's open order. `exact-memory.md` needs a note on claude-161's exact turn memory.
  - the research notebooks;
  - `STATE.md` and `HANDOFF.md`, which `loop render` writes.
- **Not covered by the sheet, left as is:** the older open-claims tables in `open-questions.md`, the discrete research lines' "next steps" (W2, A8), and the K16 figures on product pages.
- **A sentence for each:** the RMS norm has no mechanism page, and the energy line has no new measurement.

## Plain summary

The project's front page, website and trackers now say where things actually stand on 29 October: what the model is made of, what was learned and decided over the last two days, and what comes next. The status pictures are redrawn. Every planned piece of work now has its own issue: fifteen were missing, and nine of those were finished already.

Each page was checked against a single list of facts, each tied to the work that produced it. Nothing was claimed that the project hasn't shown.
