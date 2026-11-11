# ORDER: one-model-end-to-end, claude-351

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Base: `999cdc087e67c58c8de0a6ce8dac2333f744a7b3`.

## Goal

**Build the place read (arm p) as stage 1 routed it, with arming rule D3: in the float trainer and in the integer serve, with the export, float against integer parity, gradient checks and a one-page sheet. No pod and no training in this unit.**

**Why now.** [claude-350](../claude-350/REPORT.md) counted what the build must contain and put one choice to the owner. On 11 Nov (UTC), in chat, the lab asked "Build stage 2 of the place read, and with which arming rule?" and the owner answered **"Build with D3"**. That approves this build, with D3. It does not approve a pod: stage 3 runs only when the owner says run. The hold of 10 Nov stands for every other arm (PR #3913 stays a draft).

## What is built

Each row is stage 1's routing ([claude-350's ORDER](../claude-350/ORDER.md), "The arm" and "What stage 1 decides").

| Part | As routed | New here |
|---|---|---|
| Find | Two ask heads. Each asks with two halves: the previous token's half plainly, the current token's half with the head's learned row (C3w). | The two-half question for ask heads. |
| One position | Each head reports its nearest position, the later on ties. A row's find is the later of the two heads' positions. | The rule for ask heads; today they report a mean of 16. |
| Mask | The find sees user-turn positions only, an exact mask from the role markers. | The mask. |
| Arm (D3) | A row's find sets the register only if the row's token fires: one learned gain per question token, firing when it is above zero. The last firing row of the turn wins. | The gain and its rule. |
| Hold | One place register per conversation, an integer. The next user turn clears it. | The register. |
| Open | A reply row opens the read when its token's gain is above zero and the register is set: one learned gain per reply token. | The gain. |
| Read | At an open row the register offers the shared table's point of the token at its place, through the copy read's readout mix. | The source. |
| Step | At an open row, when the reply's token equals the token at the place, the place moves on by one and the next row is open. A miss closes that row only. | The step. |
| Stop (B1) | After a step, the read ends where the token at the place starts a new word or is punctuation, or the turn ends. The register then clears. Nothing is learned. | The stop. |
| Label | Places from TRAIN's oracle twin, as stage 1 read them. | A places file the trainer reads. |
| Window | 256 tokens in training and in serving. | The window. |

## Decisions the build fixes

Stage 1 left these to the build. Each is stated here before any code, and none changes a bar.

- **One register, one program.** The hold, the arm, the open, the step and the stop are one integer program. The trainer runs it teacher-forced over the window's tokens and the serve runs it over the emitted tokens. No gradient passes through it.
- **When a head finds.** A head finds when its nearest user-turn position is inside the head's margin, the test the serve's `LatestNearest` rule already makes. Else the head finds nothing at that row.
- **Two heads, one pull.** Each row of a labelled question pulls one head toward the key of the labelled place: the head whose question is nearer that key. Only the question rows learn, as in the pull on `main`.
- **Both gains are counts.** Each gain is a signed count per token that saturates, read by its sign, as stage 1's majority rules were. The arm count goes up at a question row whose find is the labelled place, and down at a row whose find is another place or stands before a reply with no label. The open count goes up at the value-start row when the register is at the labelled place, and down at any other reply row while the register is set; rows the step opens are left out. No gradient reaches either.
- **The source replaces, it does not add.** At an open row the register's token is the mix's source in place of the copy heads' mean. The mix's weight `w_c` is the only part of the read the training loss moves.
- **One table for roles and the stop.** A class per token (starts a user turn, starts a reply, ends a turn, stops a read) is built from the tokenizer at training and written to the artifact, so the serve reads roles and the stop from the token stream alone.
- **Off is main.** With the arm off, every output and every artifact is byte-identical to `main`.

## What a pass is

| # | Gate | Check |
|---|---|---|
| G1 | The register program reproduces stage 1. Driven by the census's own finds, it gives D3's line: the true place on 426 of the 436 reached, a wrong place on 30, set on 2 general replies and 5 abstentions; opened right on 425 of 426; first open on 413 of 562. | a test on the panel of claude-349, on the laptop |
| G2 | Float against integer: on a small model built with no GPU, the trainer's forward and the serve give the same register, the same open rows and the same read at every row. | a parity test |
| G3 | Gradients: a finite-difference check of every new differentiable part (the two-half question's row, the pull aimed at the labelled place, the open gain, the mix with the new source). | tests |
| G4 | Off is main: the crate's suites pass unchanged, and an artifact written with the arm off equals one written from `main`. | tests, and a byte compare |
| G5 | The export round trip: a model with the arm on is written, read back and served, with G2 holding on the read-back artifact. | a test |
| G6 | `uor-harness ratchet d23`, `d25`, `d32`: no additions. | the three commands |
| G7 | A one-page sheet for stage 3 with a FIT from a reader who is not its author. | the sheet and the reader's verdict |

## Amendments made during the build (11 Nov, claude's own decision review; logged for Casey, who can reverse them)

Each is a fact the build met, the change it forces, and why no bar moves.

1. **The one program lives in `crates/uor-r4-integer/src/fbs_snapshot.rs`.** The integer crate has two builds that share no engine code: the serving build (`--features serving-wasm`, the `stack` engine) and the default build the float trainer links. `fbs_snapshot.rs` is the one module both compile, and it is in the item's `scope:` line. So the register (`PlaceState`), the turns (`PlaceTurns`), the pick (`place_pick`) and the artifact's record (`PlaceMarks`, `PlaceImage`) are there, and `stack` re-exports them. This is "One register, one program" made literal: the trainer and the serve call the same compiled functions.
2. **"One table for roles and the stop" is a record and one table.** Role markers are several tokens long ("User:" after a newline), so a class per token cannot name a turn. The artifact's header carries the marker token runs (`PlaceMarks`: begin, end, separator, the three role markers) and one section, `hamming.d{depth}.place`, carries three bits per token: stops a read, arms the register, opens the read. The serve still reads everything from the token stream alone.
3. **G3 has no open-gain check.** The decision "Both gains are counts" makes the arm gain and the open gain saturating counts with no gradient. G3's list named "the open gain" in error. G3 checks the three differentiable parts: the two-half question's row, the pull aimed at the labelled place, and the mix with the register's source.
4. **G1 is read on the dynamic program, and it is exact.** The census counts each reply row against the register as the reply's start left it; the program's register also steps and stops. Run token by token over the panel with the census's own finds, the program gives D3's line exactly (426, 30, 2, 5; 425 of 426; 413 of 562), no reply's register had left the true place at its value-start row, and all 425 opened values were read whole, none short and none long. The output with the program's lines removed hashes to claude-350's `census.out`.
5. **G2 and G3 split at the GPU.** The vendored candle CPU backend refuses every tensor operation (D24, D29), so the trainer's forward and its gradients cannot run on the laptop, and this unit has no pod. The repo's standing form for such checks is a `#[cfg(feature = "cuda")]` test run on a pod.
   - **G2a, on the laptop:** the register, the open rows and the offered token come from the same compiled program in the trainer and the serve, so they agree when its inputs agree. The tests are: the packed codes the trainer hands the program equal the codes of the exported artifact for every token and head; and the integer mix at an open row equals the float form `n + w(c − n)` within half a step.
   - **G2b and G3, on the pod:** the trainer's forward against the served artifact row by row, and the three finite-difference checks, are written here as `cuda` tests and compile under the stub. They run as stage 3's first step, before any training, and a failure there stops stage 3. **This unit does not run them and does not claim them.**
6. **Files in scope gain `crates/uor-r4-integer/src/fbs_snapshot.rs`**, already in the item's `scope:` line.

## Not established by this unit

- Nothing is trained. No gate shows that a trained model takes the register's token, that the question rows learn their targets, or that the gains settle. Only stage 3 can.
- G1 drives the register with the census's exact finds, not a model's. It checks the program, not the find.
- The trainer's forward and its gradients are not run here (amendment 5): G2b and G3 are written and compiled, and run on the pod as stage 3's first step.
- The 126 recall replies the find cannot reach stay out of reach.

## Cancel probe (run first)

- **What would make this unit unnecessary:** a read on `main` that holds a position across rows.
- **Result (at claim):** none. `git grep -n -i -E "place[_ ]register|place[_ ]read" HEAD -- crates/uor-r4-float-train/src crates/uor-r4-integer/src/stack` gives lines in `turn_census.rs` only, stage 1's counts.

## Claim

Written by hand, as claude-347's to claude-350's were: claude-346 (PR #3913, a draft under the hold) holds the item for `uor-harness loop claim`.

## Files in scope

- `docs/loop/tasks/one-model-end-to-end/claude-351/**`
- `crates/uor-r4-float-train/src/flock_read_hamming.rs`, `geometric_stack_native.rs`, `in_space_long.rs`, `fbs_export.rs`, `turn_census.rs`, `bin/train-in-space.rs`
- `crates/uor-r4-integer/src/stack/hamming.rs`, `mod.rs`, `session.rs`, `format.rs`, `synthetic.rs`, `tests.rs`, and `generation.rs`
- `crates/uor-r4-integer/src/fbs_snapshot.rs` (amendment 1)
- `docs/research/notebook/**` (one row a result)
- A new source file only if the item's `scope:` line gains its path in the same commit.

## Inputs, all on the laptop

- The panel and TRAIN of claude-349 under `local/claude/claude-349/`, and TRAIN's twin and the count output of claude-350 under `local/claude/claude-350/` (`census.out`, SHA-256 `90ecd6881ae50f0af9dfc70c69530ef626bd55de329b7b7430b2becddf7b3bae`). No model reads them in this unit.
