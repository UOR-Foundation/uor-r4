# ORDER: object-store-consumer-m3, antigravity-113

Lab: antigravity. Item: [object-store-consumer-m3](../../../../roadmap/items/object-store-consumer-m3.md). Base: `d13846f57`.

## Goal

The item's done-when reads:

> A consumer reads and writes through the object store on the model path, and the durable-memory bars run with recorded results. The M3 bar in [`docs/integration/project-track.md`](../../integration/project-track.md) is: on the selected M1/M2 artifact, `m3-memory` passes provenance, contradiction and correction, abstention on unstated or retracted facts, explicit reset and process restart. Saved and restored sessions must continue identically token-for-token (`IntegerStackSession` + `UR4SESS1`), and non-memory prose must not silently overwrite stored facts (#1718). A numeric threshold for each bar is not yet registered. The bars need a selected M1/M2 model, and M1 is not met and M2 is held, so the on-model run cannot start now. Registering the bars and building the consumer behind finite CPU controls can.

Now that M1 (`#2029`) is met on the selected two-seed 19-table `UORLUT01` (`trunk-joint-pointer-v1-d11`, `uor-r4.lut-stack/3`) artifacts (`s1-trunk-joint-pointer-v1-d11.lut` `04652a3de8e966a23539b04fbd77c7d9872d9f13a541fcb224d48e766e2c147d` and `s2-trunk-joint-pointer-v1-d11.lut` `1b49e8bd823a1021e31de55c767111f9cf43807d3e6d007644e49281935877de`), wire the on-model `m3-memory` evaluator (`m3-durable-memory`) in `crates/uor-r4-object-store` to bind the selected M1 `UORLUT01` artifacts on the GPU via `DeviceScorer`, pre-register (`docs/harness/prep/object-store-consumer-m3-2026-10-26.md` + FIT + D67), run `m3-durable-memory-panel-v1.jsonl` across two CUDA builds (`Build A` and `Build B`) on 1x RTX 5090, archive to `caseyallard/uor-r4-store:results/antigravity/object-store-consumer-m3-2026-10-26/`, and close `#2873` (`status: done`).

## Cancel probe (run first)

Ran `CARGO_TARGET_DIR=local/target ~/.cargo/bin/cargo test --offline --no-fail-fast -p uor-r4-object-store --features stage0-lab` (`133 passed; 0 failed`) and checked `docs/labs/object-store-consumer-m3/` and `docs/research/notebook/m3-durable-memory.md`. The finite CPU controls pass on `origin/main` and acceptance thresholds are owner-registered (`#2028#issuecomment-6356034555`), while the on-model GPU `m3-memory` run on the selected two-seed M1 `UORLUT01` artifacts (`04652a3d…` and `1b49e8bd…`) has not yet been built or executed. Classification: `instrument`. Proceed.

## Files in scope

- `crates/uor-r4-object-store/**`
- `docs/mechanisms/exact-memory.md`
- `docs/mechanisms/object-store.md`
- `docs/labs/object-store-consumer-m3/**`
- `docs/harness/prep/object-store-consumer-m3*`
- `docs/loop/tasks/object-store-consumer-m3/**`
- `docs/research/notebook/m3-durable-memory.md`
- `docs/roadmap/items/object-store-consumer-m3.md`
- `docs/roadmap/README.md`

## Done when

1. `crates/uor-r4-object-store` exposes the on-model M3 durable-memory evaluator (`m3_durable_memory` + `src/bin/m3-durable-memory.rs` under `cuda`), tested with finite host controls (failing test first).
2. `docs/harness/prep/object-store-consumer-m3-2026-10-26.md` passes FIT and D67 pre-run review, and `m3-durable-memory-panel-v1.jsonl` (`582e3879…`) runs on 1x RTX 5090 across both selected M1 `UORLUT01` seeds (`04652a3d…` and `1b49e8bd…`) and two CUDA builds (`Build A` and `Build B`), with all 6 sub-panels and `no-memory` negative controls (`L-010`) archived to `caseyallard/uor-r4-store:results/antigravity/object-store-consumer-m3-2026-10-26/`.
3. Docs and `docs/roadmap/items/object-store-consumer-m3.md` (`status: done`) are updated and `uor-harness loop gates --full` is `11 / 11` GREEN.

## Stop and ask if

- Any `UORLUT01` artifact digest, `ft/dev` Gate R parity (`7,386 / 14,287` or `7,374 / 14,287`), or panel digest (`582e3879…` / `88c27b94…`) mismatches, or any sub-panel misses its registered bar.
