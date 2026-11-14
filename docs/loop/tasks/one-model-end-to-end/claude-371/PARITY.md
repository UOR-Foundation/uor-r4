# PARITY: the native-space parity contract (claude-371)

Lab: claude. The contract checks training-mode simulation against the integer serving runtime on the discrete path ([INVENTORY.md](INVENTORY.md)). One leg runs now: the CPU training program against serving. The other, the CUDA training kernels against serving, is designed here and runs later on `wick`.

## The start

Both sides load the same bytes: one `UORLUT01` `uor-r4.lut-discrete/1` artifact. There are two fixtures, built by the test module's own builders in `crates/uor-r4-cpu-train/src/discrete_route.rs`. Each has vocabulary 3, 2 blocks, context 3 and 2 stages.

- **Route**: routes `[1, 2, 3, 4]`, gates zero.
- **Flock**: K = 2, `active_leaf_v1`, gates zero.

The prompt set is all 39 prefixes of lengths 1 to 3 over the three tokens. The carry starts empty on both sides; the discrete read has no state between prompts.

## The checks (raw words only)

| Check | Words compared | Implementation |
|---|---|---|
| K1 carry | every i32 coordinate of the read vector (blocks × 8 per prefix; 624 words over 39 prefixes) | `export_parity` query-coordinate mismatches |
| K2 memory address | the admitted cell list, in order | `export_parity` admitted-cell mismatches |
| K3 emitted ids | the top-8 list and the top-1 token | `export_parity` top-8 and token mismatches |
| K4 load | the serving loader refuses the bytes (the error must come from the loader, not from a row) | `IntegerDiscreteModel::from_bytes`, `assert_loader_refuses` (new) |
| K5 touched words | the set of (table, index) words in `discrete.table`, `discrete.cells`, `discrete.routes` and `discrete.gates` that a step changed, against the declared set (a Table move on block 0: table words 0–7 and cell 0) | `touched_words` (new) |
| K6 save | the training side loads the bytes and saves them; the saved bytes must equal the bytes loaded | `saved_after_load` (new), via `CpuDiscreteProposalApplier::artifact_bytes` |

## One planted fault per category, and what caught it

Measured by `native_space_parity_contract_catches_one_planted_fault_per_category`. The fault is planted in the serving copy only, except C-5, which is planted in the training step.

| Category | Planted fault | Caught by | Result |
|---|---|---|---|
| C-1 route choices | every single route word changed to each of the other 63 ports (252 faults) | K1, indirectly | **108 seen, 144 missed.** The 144 leave every read word unchanged on all 39 prefixes. No route-port word exists on the serving side to compare. **Failing control.** |
| C-2 complete carry | every flock gate trit changed to each other trit (8 faults) | K1 | **3 seen, 5 missed**: (0, −1), (2, −1) and (2, +1) change the carry; the rest leave every word unchanged. |
| C-3 memory addresses | a stored cell that disagrees with the table | K4 | caught at load (`discrete_d11.rs:1115-1118`). **Negative gate:** with that check set aside, the test fails at C-3. |
| C-4 memory read choices | the flock's kept list | — | **Not plantable.** CUDA writes it to `trace`; serving exposes only the read vector. **Failing control.** |
| C-5 learned-state words touched | a step applied to table block 1 where block 0 was declared | K5 | caught. No read check is involved. |
| C-6 emitted ids and stop | tokens 0 and 1 swapped, rows and cells together (it loads cleanly) | K3 | caught: top-8 and token mismatches. **Stop state: neither side has one. Failing control.** |
| C-8 serialized words | a non-zero gate trit in a Route artifact | the training side's load (K6) | **Serving loads it and reads every word exactly as the clean model; the training side refuses it** ("Route initialization requires zero gates", `discrete_route.rs:153`). The loaders disagree on what a valid artifact is, and serving is the laxer one. **Negative gate:** with the CPU refusal set aside, the test fails at C-8. A non-zero header pad byte is refused by K4. |

The clean start passes K1 to K6 on both fixtures (`native_space_parity_contract_starts_identical_and_stays_identical_after_a_step`). So does one training step: a Table move on block 0 touches only table words 0 to 7, the epoch goes from 0 to 1, and load-then-save gives the same bytes.

## What the misses mean

- **An output-word check cannot certify the route and gate state.** On this fixture, 144 of 252 route faults and 5 of 8 gate faults change no output word on any of the 39 prefixes. A larger panel would see more of them. But a fault that no prompt in the panel exercises is still a different model: it would answer some later prompt differently. The direct check is to compare the learned words themselves (K6 covers this when both sides write bytes), plus the route ports chosen at each stage, as K1b below.
- **The serving loader accepts what training refuses** (C-8). An artifact only serving can load is a model no training run produced. The serving loader should refuse non-zero gates in Route mode, as the CPU loader does.
- **The serving runtime has three internal choices it does not expose:** the route port chosen at each stage, the flock's kept list, and the stop state, which does not exist yet. Until it exposes them, the contract can catch a fault there only through the outputs.

## The CUDA leg (next unit, on `wick`)

- **Build:** `uor-r4-discrete --features cuda` with `UOR_NVCC_ARCH=compute_89` (RTX 4090), on `wick`, in the `uor-r4-claude` container. No rented GPU.
- **Start:** the same two fixture artifacts, loaded into device buffers. `table`, `routes` and `gates` are copied word for word from the artifact sections; the spec words `s[]` come from the header; `s[11]`–`s[15]` are 0.
- **Compare:**
  - `discrete_forward` output words and `trace` against the CPU and serving reads (K1, and C-4 through `trace`);
  - `predictions` against K3;
  - `discrete_counterfactual` loss words for every candidate against `CpuDiscreteProposalScorer::score` on the same rows;
  - the `discrete_choose` proposal words against `CpuDiscreteProposalChooser::choose`;
  - after `discrete_apply`, the device buffers copied back against the CPU applier's `artifact_bytes`, which covers K5 and K6.
- **Plant:** the same C-1 to C-8 faults, plus one per kernel status (stale epoch 3, bad index 4, non-improvement 5, mode mismatch 9). Each must refuse with the same status as the CPU applier.
- **K1b, to add:** a serving-side trace of the port chosen at each stage. This is a read-only accessor beside `route_read`. It is the smallest change that turns C-1 from indirect to direct.
