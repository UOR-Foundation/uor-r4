# REPORT: one-model-end-to-end, claude-371 (native-space inventory and first parity contract)

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Order: [ORDER.md](ORDER.md). Inventory: [INVENTORY.md](INVENTORY.md). Contract: [PARITY.md](PARITY.md). PR #3962.

The order asked for an inventory of every state field and operator a served native-space model carries, and one executable contract comparing training-mode and serving words, with one planted fault per category. Both exist. The answer is mixed.

- **Representation.** The discrete SpiralCore/FBS path already keeps one integer representation from the first training step to serving: i32 orbit-root table, u32 route ports, i32 gate trits, u64 epoch. Export copies these words; it does not quantize them.
- **Answering.** That path cannot answer a turn.
- **Checking.** An output-word check misses most route faults.

No training run, no GPU, no capability or speed claim.

## What did not work, first

1. **The native path cannot answer.**
   - No generation loop and no stop state on either side. `uor-chat --discrete` is one-shot `--prompt-ids` only and refuses text chat (`uor-chat.rs:960-970`); WASM has `predict` only.
   - No turn memory, no copy store, no E8/H4/Hopf operator.
   - Everything that makes the talking model talk is on the stack path. That path's served words are a 4-bit quantization of an F32 master: the export gap the hand-off rules out.
2. **Most route faults pass every read check.** Over every single route-word fault (4 words × 63 other ports), the carry check sees 108 and misses 144 on all 39 prefixes of the Route fixture. Flock gate faults: 3 of 8 seen. Serving exposes no route-port word to compare, so this is a failing control of the contract, not of the model.
3. **Two categories cannot be planted at all.**
   - The flock's kept list: CUDA writes it to `trace`, serving does not expose it.
   - Stop state: it does not exist.
4. **No CUDA leg.** No test runs the CUDA kernels against the CPU or against serving on the same bytes. The CPU tests copy CUDA's rules by hand (`apply_status_order_and_noop_match_cuda`, `discrete_route.rs:5886`). Serving's tests compare against hand-copied CUDA witnesses (`discrete_d11.rs:1920`).
5. **Two edge rules differ between CUDA and serving.**
   - `root_cost` subtracts in signed `int` on CUDA (silent wrap) and with `unsigned_abs` in Rust (a debug panic).
   - `snap` starts its best root at ABSENT on CUDA and at 0 in Rust.
   - Both are unreachable while coordinates stay in range, and no check pins the range.
6. **Side finding, at base, not caused by this unit.** Two stack-path CPU-to-serving tests fail on `675f5eb43`: `fbs::tests::ask_heads_at_both_hooks_match_serving_without_carry` and `fbs::tests::readout_ask_carry_matches_serving_over_a_multi_row_window` (`fbs.rs:2295`, "row 2: complete query"). They fail with this unit's change set aside, so they are not this change. Filed as a note, not fixed here.
7. **Sources the hand-off named are absent.** D85, a native-space skill, `native-space.json`, a native-space roadmap item and the Codex branch are not on `main`, GitHub or this laptop. The rule was taken from the hand-off text.

## What changed

- `docs/loop/tasks/one-model-end-to-end/claude-371/INVENTORY.md`: the card. It has two path tables and the shared constants. Seven areas (token to vector; carry and transport; snapping and root selection; memory addresses and reads; writer and emission; serialization; update and optimizer state) each list domain, width, lifetime, CUDA, serving, CPU, WASM, evidence and gap, with file and line. Backend identities follow.
- `docs/loop/tasks/one-model-end-to-end/claude-371/PARITY.md`: the start bytes, checks K1 to K6, the eight categories with their planted faults and results, and the CUDA leg's design for `wick`.
- `crates/uor-r4-cpu-train/src/discrete_route.rs`, test module only (+233 lines): two tests and six helpers (`table_words`, `every_prefix_sample`, `contract_reading`, `touched_words`, `saved_after_load`, `table_move`). No non-test line changed.
- `docs/research/notebook/cpu-training.md`: one row. This is the D84 CPU line, because this leg is CPU work.

## Measurements

| Check | Route fixture | Flock fixture |
|---|---|---|
| Clean start, K1 to K6 | exact (39 rows, 624 carry words) | exact |
| After one Table step on block 0 | exact; touched = table words 0–7 only; epoch 0 → 1; load-then-save byte-identical | exact; same |

| Category | Planted fault | Caught by | Result |
|---|---|---|---|
| C-1 route choices | 252 single-word route faults | K1, indirectly | 108 seen, 144 missed |
| C-2 carry | 8 gate-trit faults (Flock) | K1 | 3 seen: (0,−1), (2,−1), (2,+1) |
| C-3 memory address | stored cell ≠ table | K4 loader | caught |
| C-4 memory read choice | — | — | not plantable |
| C-5 touched words | step on block 1, block 0 declared | K5 | caught |
| C-6 emitted ids | tokens 0 and 1 swapped with their cells | K3 | caught |
| C-6 stop | — | — | not plantable |
| C-8 serialized words | unread gate trit (Route) | K6 only | caught by K6; K1 to K3 pass |
| C-8 serialized words | non-zero header pad byte | K4 loader | caught |

## Checks run at the head

| Check | Result |
|---|---|
| `cargo test -p uor-r4-cpu-train --lib native_space_parity_contract` (laptop, debug) | 2 passed |
| Negative gate: the serving loader's cell check (`discrete_d11.rs:1115`) set aside, the contract test run, then restored | failed at "C-3", as it should; restored and re-run green |
| `cargo test -p uor-r4-cpu-train --lib` (full, 538 s) | 506 passed, 2 failed, 5 ignored. The two failures are the `fbs::tests` above; they also fail at base with this change set aside. |
| `rustfmt --edition 2021 --check` on `discrete_route.rs` | clean; the formatter's only change was inside the new block |
| `cargo clippy -p uor-r4-cpu-train --tests` | no finding in the new lines |

## Skills used

None.

## Still open

- **The smallest next training dependency** is the CUDA leg of this contract on `wick`'s RTX 4090: `discrete_counterfactual` loss words, `discrete_choose` proposals and the buffers after `discrete_apply`, against the CPU scorer, chooser and applier on the same bytes. Until it passes, a CUDA training run's updates are not known to be the ones the CPU and serving compute.
- **K1b:** a read-only serving trace of the route port chosen at each stage, beside `route_read`. It turns C-1 from 144 misses into a direct check. A trace of the flock's kept list does the same for C-4.
- **The shared typed-operator layer** (the hand-off's step after acceptance). Its first transition should be one move, `discrete_apply` with a Table kind: one source definition of `action`, `root_cost`, `snap` and `checked_scaled_add` used by CUDA, CPU and serving, with the range check that closes item 5.
- **Answering on the native path** needs a generation loop with stop state, and memory, in this representation. No current code does it.

## Plain summary

The project has two ways of serving a model. The one that talks is trained in floating point and squeezed into integers afterwards, which this track rules out. The other one, the "discrete" path, is integers all the way: what training learns is exactly what gets served. But it can only predict one next token. It cannot hold a conversation yet.

This unit wrote down every number that path carries and checked that the training side and the serving side agree on them. They agree, and several kinds of planted damage are caught. One kind mostly is not: of 252 ways to damage a routing word, 144 produce no visible difference on the test prompts, because serving never shows its routing choices. Making it show them is a small next step. Running the same check against the GPU kernels on `wick` is the next one.
