# ORDER: turn-memory-integer, claude-197

Lab: claude. Item: [turn-memory-integer](../../../../roadmap/items/turn-memory-integer.md). Base: `48cc5fe83`.

## Goal

The item's done-when reads:

> 1. **Export and session, for both tables.**
>    - The exporter writes each table (keys and values as integer ids), each weight (at the exporter's fixed point, rounded as it rounds), and the memory's key window (192).
>    - The session takes the reply's start from its caller, keys as "The keys" above, and applies the opening move at a reply's first step and the reply move at each later one.
>    - Each key is checked against a test-side reference of the float rule above. The reference rows include:
>      - a reply cut by the window;
>      - a prompt with an internal `<|eos|>`;
>      - a session whose `context` is not 192.
>    - Both disassembly audits pass with the moves reached. The memory is off for an artifact without one.
>    - A session restore with a memory is refused until its turn state is saved.
> 2. **Parity,** on DEV, against the float forward of claude-194's sealed (r) checkpoints at step 16,384, both seeds.
>    - **Location:** `caseyallard/uor-r4-store:results/claude/turn-memory-reply-2026-10-29/s{1,2}-fc66ed74/r-s{1,2}/models/step-016384/`, five files each: `config.json`, `model.safetensors`, `hamming_read.safetensors`, `turn_memory.safetensors` and `turn_memory_reply.safetensors`.
>    - **Before use,** the sheet checks them against each seed's `SHA256SUMS`.
>    - The share of reply rows (openings and later rows) whose integer and float decoded ids agree.
>    - The integer `T`, `O`, `C` and `R` counts beside float's.
>    - The float reference runs on the GPU (CHARTER §3.5). The integer side runs on the laptop.
>    - The bar is pre-registered in a sheet with a fresh FIT.
> 3. **Cost on the M1:** `fbs-toks --stages` with the memory, timing its lookup and move beside the writer and the trunk, plus the artifact's added bytes.
> 4. One notebook row and a REPORT.

## Cancel probe (run first)

To fill in.

## Files in scope

- `crates/uor-r4-integer/src/stack/memory.rs`
- `crates/uor-r4-integer/src/stack/mod.rs`
- `crates/uor-r4-integer/src/stack/session.rs`
- `crates/uor-r4-integer/src/stack/format.rs`
- `crates/uor-r4-integer/src/stack/tests.rs`
- `crates/uor-r4-integer/src/bin/fbs-export.rs`
- `crates/uor-r4-integer/src/bin/fbs-toks.rs`
- `crates/uor-r4-float-train/src/fbs_export.rs`
- `crates/uor-r4-float-train/src/flock_read_memory.rs`
- `crates/uor-r4-float-train/src/in_space_long.rs`
- `crates/uor-r4-float-train/src/bin/train-in-space.rs`
- `docs/harness/prep/turn-memory-integer*`
- `docs/loop/tasks/turn-memory-integer/**`
- `docs/research/notebook/m4-serving.md`
- `docs/roadmap/items/turn-memory-integer.md`

## Done when

To fill in.

## Stop and ask if

To fill in.
