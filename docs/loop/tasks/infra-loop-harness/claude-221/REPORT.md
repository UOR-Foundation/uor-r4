# REPORT: infra-loop-harness, claude-221 (record the point-key-integer item)

Lab: claude. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md). Order: [ORDER.md](ORDER.md), written by hand: `loop claim` refused on Kimi's #3436 holding `PLAN.md`, a file this order does not touch.

## Tokens per second

No measurement: this unit records an item.

## What did not work, first

- **`loop claim infra-loop-harness` refused:** its `PLAN.md` overlaps Kimi's open docs-status-refresh order (#3436). The ORDER is hand-written, scoped to files that order does not touch.
- **The first item draft's title was 90 characters,** over `roadmap check`'s 80-character limit. It is shortened to "Serve the point key's opening in the integer artifact on the laptop".
- **The fresh review found CHANGES** in the item as first synced:
  - **The only bar could not fail.** It was parity against the float key's names on panels already read, with "near-tie" undefined, and it dropped the decision card's "its own bar on a fresh panel" for form C.
  - **The scope was incomplete:** it missed `stack/tests.rs`, `stack/synthetic.rs`, `fbs-export`, `fbs-toks`, a panel driver and the WASM files. It also listed the float snapshot and exporter files, which condition (b) forbids building the table from.
  - **`needs_gpu: true`** answered a question the item left to the sheet.

  All three are fixed:
  - The deciding bar is form C's own on a fresh panel p-x: integer served against integer greedy, claude-220's `carries` and `harmless`, at seeds 1–3, with rows registered before p-x is read.
  - The exact instrument checks fail the unit.
  - Parity is reported with its near-tie margin fixed in the item (512 Q12 units, the bound on an order rounding alone can flip).
  - The scope lists the files the work needs, and drops the float snapshot files.
- **Round 2 found the no-GPU fix contradicted CHARTER §3.6:** "Panel scoring, qualification and milestone evaluation of an exported artifact remain GPU research compute". So `needs_gpu` is true again, and the runs are split:
  - builds, finite tests, tok/s and resident memory run on the laptop;
  - the export pass over TRAIN, the panel arms and the scoring run on the GPU.
- **Named as missing work:** `uor-r4-integer` has no CUDA path, so an exact integer evaluator on the GPU, bit-exact against the laptop engine, is listed.
- **The owner's question:** whether the bit-exact laptop engine may score the deciding panel instead is a §3.6 call. It goes to the owner's queue once (#2028), and the kernel work does not wait for it.
- **Also from round 2:** the near-tie class is reworded as "within the rounding bound", since the integer points are the integer trunk's own, not rounded float points.

## What worked

- **The item [`point-key-integer`](../../../../roadmap/items/point-key-integer.md) is recorded,** and its issue is #3450. It is claude-220's condition (d), "the export needs its own item", now that claude-220 read row 2 (#3443, `c3d5cb684`).
- **It carries condition (b):** the integer form's bar is measured with the integer trunk's own points, for both the query and the table, never with float points quantized after the fact.
- **Its done-when, in brief:**
  - an integer table built by the integer engine (i16 Q12, about 109 MB), and its search at `stack_step_fbs`'s first reply step, where the exact key misses;
  - exact instrument checks that fail the unit;
  - parity against claude-220's float served names on p-w, p-v and DEV, reported, with its classes fixed in advance;
  - the deciding bar on a fresh panel p-x, at seeds 1–3;
  - tok/s on the laptop.

## What changed

- `docs/roadmap/items/point-key-integer.md` (new, hand-written).
- `docs/roadmap/issues.json`: one line, `point-key-integer` → 3450 (written by `roadmap sync --apply`).
- `docs/roadmap/README.md`: the counts, one row and two dependency edges (written by the sync's render).

## Measurements

- **Roadmap counts at the head:** 116 items OK.
  - By status: ready 18, held 10, decision 1, done 87.
  - By milestone: research 39.
- **The sync plan:** 0 labels, 1 issue to create, 0 to update, 0 to close, 0 milestone checklists.
- **Applied:** created #3450.
- **After each review round,** a sync planned and applied exactly one update, #3450's body. That happened twice. The rendered row's GPU column read "no" after round 1 and "yes" again after round 2.

## Checks run at the head

- `cargo run -q --locked -p xtask -- roadmap check`: 116 items OK.
- `cargo run -q --locked -p xtask -- roadmap sync` (dry): exactly one create, nothing else.
- `cargo run -q --locked -p xtask -- roadmap sync --apply`: created #3450 and wrote both generated files.
- `cargo run -q --locked -p xtask -- roadmap render --check`: current.
- **Nothing else changed:** no owner path, bar, status, other item or code. The rendered page changed only by the new item.

## Skills used

None.

## Still open

- **The item's first unit:** the integer table, its search, and the artifact record. It takes its claim through `loop next` once this merges.
- **Settled in that unit's sheet:** the export pass's cost over TRAIN's openings, the GPU evaluator's bit-exactness check, the fresh panel p-x's seed, and the bar's rows.
- **The GPU evaluator** is built unless the owner allows laptop scoring (#2028). Until then a decision card decides only its crate and files.

## Plain summary

The last unit showed that the float model can look up how to start its reply from its own internal state. The next job is to make the shipped whole-number version of the model do the same. This unit only writes that job down as its own item on the project's roadmap, with what it must build and how it will be checked, and opens its tracking issue.
