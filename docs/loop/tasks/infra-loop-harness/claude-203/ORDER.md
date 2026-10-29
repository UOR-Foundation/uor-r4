# ORDER: infra-loop-harness, claude-203 (the scope a held-out-phrasing panel needs)

Lab: claude. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md), the umbrella that edits roadmap items. Base: `443a980e5`.

Setup row: 2 (item-file maintenance: a scope addition to a gap item).

## Goal

A scope decision for [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). The guard enforces an item's scope as it stands on `main`, so the scope must merge before the code that needs it (claude-197's finding).

**Why the item needs it.** The census (claude-202, `443a980e5`) found that neither DEV nor Fresh V2 holds new templates: 0.750 and 0.777 of their replies are verbatim TRAIN replies. The item's next unit needs a panel whose phrasings TRAIN never shows.
- **The generator already splits phrasings.** M-world v2's module doc says `dev_phrasing × dev_value` is its development split, and `train_phrasing × dev_value` its pure-retrieval cell (`crates/uor-r4-training/src/milestone_world_v2.rs`, "Revision 2.1 … Cross cells").
- **But nothing writes those cells as a corpus.** `m-world corpus world=v2` writes TRAIN conversations only. `corpus_v2` in `crates/uor-r4-training/examples/m-world.rs` takes no split or cell.
- **The float crate cannot generate them itself.** It is the D37 scanned module "with zero dependency on uor-r4-training" (its `Cargo.toml` description).

**The decision.** Add one file to the item's `scope:`, `crates/uor-r4-training/examples/m-world.rs`, so the next unit can give `corpus world=v2` a `cell=` option (for example `cell=dev_phrasing,dev_value`). That option writes a held-out-phrasing panel as a UORT split, which `train-in-space --census` can then count. The library (`milestone_world_v2.rs`) is not added, because its `conversation_in(rng, cell)` already takes a cell.

## Cancel probe (run first)

- **Is the file already in the item's scope?** No. At `443a980e5`, `one-model-end-to-end`'s `scope:` has no path under `crates/uor-r4-training/`.
- **Does a writer for dev cells already exist?** No. `corpus_v2` has no `split`, `cell` or `phrasing` argument (`git show origin/main:crates/uor-r4-training/examples/m-world.rs`, lines 1130–1215).
- **Classed:** instrument. Nothing cancels the unit.

## Files in scope

- `docs/roadmap/items/one-model-end-to-end.md`
- `docs/roadmap/issues.json`
- `docs/roadmap/README.md`
- `docs/loop/tasks/infra-loop-harness/claude-203/**`

## Done when

1. **The item's `scope:`** lists `crates/uor-r4-training/examples/m-world.rs`, and nothing else is added. Its "What is missing" names the held-out-phrasing panel and the writer it needs.
2. **`cargo xtask roadmap check`** passes, and the render and `roadmap sync --apply` are current.
3. **A fresh decision review passes.** It reads the item, the census REPORT, the generator's cell code, the example, CHARTER §5.1, `owner-paths.json`, and the other items' scopes and open claims. The owner log goes on #3407.
4. **`loop report --check` and `loop pr-check --local`** pass.

## Stop and ask if

Stop and decide (never wait) if:
- the review finds the writer also needs a library change: then add that file too, named;
- another open ORDER holds `m-world.rs`.
