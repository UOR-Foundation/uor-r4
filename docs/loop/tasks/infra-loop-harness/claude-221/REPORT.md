# REPORT: infra-loop-harness, claude-221 (record the point-key-integer item)

Lab: claude. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md). Order: [ORDER.md](ORDER.md), written by hand: `loop claim` refused on Kimi's #3436 holding `PLAN.md`, a file this order does not touch.

## Tokens per second

No measurement: this unit records an item.

## What did not work, first

- **`loop claim infra-loop-harness` refused:** its `PLAN.md` overlaps Kimi's open docs-status-refresh order (#3436). The ORDER is hand-written, scoped to files that order does not touch.
- **The first item draft's title was 90 characters,** over `roadmap check`'s 80-character limit. It is shortened to "Serve the point key's opening in the integer artifact on the laptop".

## What worked

- **The item [`point-key-integer`](../../../../roadmap/items/point-key-integer.md) is recorded,** and its issue is #3450. It is claude-220's condition (d), "the export needs its own item", now that claude-220 read row 2 (#3443, `c3d5cb684`).
- **It carries condition (b):** the integer form's bar is measured with the integer trunk's own points, for both the query and the table, never with float points quantized after the fact.
- **Its done-when, in brief:**
  - an integer table built by the integer engine (i16 Q12, about 109 MB);
  - its search at `stack_step_fbs`'s first reply step, where the exact key misses;
  - parity against claude-220's float served names on p-w, p-v and DEV, each difference classed;
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
- **Settled in that unit's sheet:**
  - where the table is built and what it costs;
  - whether the parity evaluation runs on the laptop or must run on the GPU (CHARTER §3.5).

## Plain summary

The last unit showed that the float model can look up how to start its reply from its own internal state. The next job is to make the shipped whole-number version of the model do the same. This unit only writes that job down as its own item on the project's roadmap, with what it must build and how it will be checked, and opens its tracking issue.
