# REPORT: infra-loop-harness, claude-225 (record the integer-retention-rates item)

Lab: claude. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md). Order: [ORDER.md](ORDER.md), written by hand: `loop claim` refuses while Kimi's #3436 holds `PLAN.md`.

## Tokens per second

No measurement: this unit records an item.

## What did not work, first

- **`point-key-integer`'s run cannot start yet.** Planning it, I found that the integer artifact does not serve the recurrence that (r-drop) trained with.
  - **The float side:** with `retain_rates` on (since 27 Oct; (r-drop)'s run config has `"retain_rates": true`), a lane carries 1 − 2^−(k+1).
  - **The integer side:** its geometric recurrence serves only λ = 2^−(k+1), and the exporter carries no flag.
  - So the run's integer model would not be the trained trunk.
  - This was found by reading the code, so it cost no compute.
- **The sync also rewrote #3440's body,** DeepSeek's `m7-studio-live-model-sources`. Its item file changed on main in `d2f0354a9` without a sync, and claude-223 left it unapplied.
  - This ORDER decided it in advance: the sync writes only text generated from files already on main, so no lab's content changes.
  - Leaving it would block every later sync.

## What worked

- **The item [`integer-retention-rates`](../../../../roadmap/items/integer-retention-rates.md) is recorded,** as issue #3456. Its done-when:
  - the snapshot and the artifact carry the reading, refused by engines without it;
  - the retained recurrence in integers (λ_i = 1 − 2^−(i+1) at Q31, keep from a precomputed isqrt table), tested against an f64 reference;
  - trunk parity on the GPU with `fbs-compare` on (r-drop) s1, under a sheet;
  - the serving audit and the WASM build.
- **`point-key-integer` now depends on it,** and its run step says why.
- **The rate score's other flags are settled for (r-drop)** (the review's finding). `fold_scores` and `reflect_floor` also change the score before selection, and both are off in its run config. So the item serves `retain_rates` alone, and its snapshot refuses a config with either on.
- **The review checked the core claim independently:** CLAIM TRUE. The integer geometric recurrence's λ and keep depend only on the rate index, and no path applies retention.
- **DeepSeek's Studio is unaffected.** It serves the M1 discrete trunk and `fbs-dual-c131`, both trained before the retention switch (their owner log on #2028).

## What changed

- `docs/roadmap/items/integer-retention-rates.md` (new).
- `docs/roadmap/items/point-key-integer.md`: `depends_on`, and the run step.
- `docs/roadmap/issues.json`: one line, `integer-retention-rates` → 3456.
- `docs/roadmap/README.md`: re-rendered.

## Measurements

- `roadmap check`: 117 items OK.
- **The sync:** created #3456, and updated #3440's and #3450's bodies. Nothing else.

## Checks run at the head

- `roadmap check`, and `render --check` (current).
- **Nothing else changed:** no owner file, bar, status, other item or code.

## Skills used

None.

## Still open

- **The item's build** (claude-226), and its parity run under a sheet.
- **Then `point-key-integer`'s run.**

## Plain summary

While planning the test of the whole-number model, I found that it would run a slightly different version of one internal step than the model was trained with. The trained model keeps a long memory in that step, and the shipped engine keeps a short one. Testing it as it is would measure the wrong model. This unit writes the fix down as its own job, ahead of the test.
