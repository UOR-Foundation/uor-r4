# REPORT: integer-hamming-read, claude-237 (the copy-code host check)

Lab: claude. Item: [integer-hamming-read](../../../../roadmap/items/integer-hamming-read.md). Order: [ORDER.md](ORDER.md). References #3385.

## Tokens per second

This unit measured no speed: it is a host check of codes, and it changed nothing in the served path. The item's last figures are claude-235's, decoding on this M1 under load: 3,812–4,109 tok/s with the read and 4,467–4,733 without, at contexts 96–192.

## What did not work, first

- **The float D11 trace is the wrong reference for the read.** The GPU's D11 forward snaps the read's rotations to the 120 icosians, but training never snapped them, and the artifact serves them unsnapped by design. So claude-233's d with the read (694 and 823 of 1,536) and claude-235's kept-set shares measured the integer port against a snap the model never trained with. The parity bar (done-when 2) is still unmeasured against a valid reference.
- **The fix is not made here.** The snap happens in a CUDA-only hook, so a failing test for the fix and the measurement of its effect both need a GPU. The ORDER's stop rule covers this case: the stage is the GPU forward, and a GPU re-run needs a prep sheet and its FIT. The decision review is below, and the next step is written into the item.
- **The snap is pinned only on the laptop's side, not yet on the GPU's.** The CPU cannot run the trainer's snap: `icosian_snap_cuda_tensor` is CUDA-only, and the trainer's CPU model compute refuses with "D24/D29: CPU model compute is unavailable". So a float-train test pins the 120 roots bit for bit against `hamilton_sheet::h4_unit_roots_f32`, and a `#[cfg(feature = "cuda")]` test compares the port's snap with the GPU's own. That CUDA test type-checks with a stub nvcc, but has never run. It is owed to the next pod.
- **The first placement did not build.** The ORDER first put the check in `fbs-snapshot`. Under D29 the float trainer reaches only the integer crate's native modules, and the served `CopyHeads` sit behind `serving-wasm`. The check moved to `fbs-toks` in the integer crate's serving build (commit `c859e14b1`, and the ORDER's done-when 1).
- **Two things the check cannot tell.**
  - **A trace's own consistency is a weak check here.** As claude-235 found, the readout's sequences never repeat a token: their ids step by 11, so 1,536 positions give 3,072 distinct (token, copy head) pairs, the 3,072 `pairs` each output counts. "0 inconsistent" therefore only says that each token's recorded query code equals its own recorded key code. It cannot test determinism across positions.
  - **A trace from another checkpoint whose ids and depths fit is not refused.** The check would report 0 equal codes. It records every input's sha256, so a reader can match the trace to its artifact.

## The stage, named

All four runs agree. Each one is one of claude-233's artifacts (h1 or h3) against claude-235's sealed traces at the same seed, in the d11 and d11-rounded arms. Each count is out of 3,072 copy query codes (1,536 tokens × 2 copy heads) at each depth.

| Run | Depth | Float trace = control (a) | Float trace = snapped control (a′) | Mean bits from (a) |
|---|---|---|---|---|
| h1 / s1, d11 | 2 | 172 | **3,072** | 3.22 |
| h1 / s1, d11 | 6 | 10 | **3,072** | 5.18 |
| h3 / s3, d11 | 2 | 328 | **3,072** | 2.19 |
| h3 / s3, d11 | 6 | 15 | **3,072** | 4.62 |

- **The keys agree in the same way.** Of 3,056 key codes, the float trace equals (a) at 172 and 10 (h1) and at 327 and 15 (h3), and equals (a′) at 3,056 in every run. There are 16 fewer keys than queries because the last position of each of the 8 sequences is no position's earlier one.
- **The d11-rounded arm gives the same counts as d11, in every cell.** This is claude-235's finding again: rounding the read's parameters as exported changes no code.
- **The four stages the ORDER names are cleared, and the cause is a fifth:**
  - **The integer arithmetic is not the cause.** (a) equals (b), the integer session's `CopyHeads` codes, on 8,192 of 8,192 codes (4,096 entries × 2 heads) at both depths of both artifacts, with 0 differing bits.
  - **The float arithmetic is not the cause.** The port behind (a) is pinned bit for bit to the trainer's own `hamilton_gate_fwd_cpu` and `pack_signs` (a float-train test).
  - **The record is not the cause.** The integer trace equals its own session's codes: 3,072 of 3,072 queries and 3,056 of 3,056 keys in every run. A stack test pins the served codes to `CopyHeads::code`.
  - **The GPU forward's inputs are not the cause, at the resolution a snap leaves.** (a′) is computed from the artifact's own table and rotations, and it reproduces the GPU's codes bit for bit. A different table would flip signs near 0 even after a snap. A rotation that differed only slightly would not, because the snap hides it.
  - **The GPU forward's arithmetic is the stage,** a fifth one, which the ORDER did not list. In D11 mode, `with_d11_mode` sets `HamiltonArm::HamiltonIcosianHard` through `with_d11_snaps(D11Snaps::ALL)`. With that arm, `hamilton_sheet::try_hamilton_gate_fwd_cuda` (`hamilton_sheet.rs:1014`, called from `geometric_stack_native_cuda.rs:96` on every CUDA Hamilton gate) replaces each gate with `icosian_snap_cuda_tensor`. The read's `rotate` (`flock_read_hamming.rs:525`) is a `hamilton_gate`, so it reaches the hook.
    - **Measured:** the key rotation (lines 589 and 598) is snapped. (a′) snaps only that rotation, and it reproduces the trace.
    - **Read from the code, not measured:** the type heads' query rotation (590) and value rotation (623), and a step rotation when there is one (608), go through the same `rotate`, so they are snapped too.
- **The differing bits are not rounding.** Binned by the control's value at each differing bit (|v| below 10⁻⁶, 10⁻⁵, 10⁻⁴, 10⁻³, 10⁻², 10⁻¹, and at or above it), they sit mostly between 10⁻³ and 10⁻¹. Some are at 0.1 or more: 273 and 695 at h1's depths 2 and 6, 57 and 476 at h3's. A grid's rounding cannot flip a value that far from 0, but a different rotation can.
- **Where the other two sides stand.**
  - **The checkpoints' trainer does not snap.** The checkpoints came from `train-in-space` (`in_space_long.rs`): their run config carries `turn_memory_dropout`, a field only that trainer has. It trains under `with_gate_mode(HamiltonArm::HamiltonActive, …)` with export mode off. With that arm, the hook's match passes the gate through (`_ => gate_tensor.clone()`). The D11 training path (`with_d11_training`) is used only by `spiralcore_fbs_sheet.rs`.
  - **The artifact does not snap.** The item's design (claude-187) reads "Rotations are stored raw, and normalized at serving", by the integer `stack_rotation`.

  So the float D11 reference is the side that differs from both.

## The decision, and its review

The ORDER's stop rule reads: "The stage is the float trainer or the GPU forward's own inputs. Then the fix changes the float model or the export, outside this scope. The lab decides by a decision review and writes the next step. A GPU re-run needs a prep sheet and its FIT." The stage found is the GPU forward's arithmetic, which the rule does not name, so it is applied here by analogy. The fix sits in a file this ORDER scopes (`flock_read_hamming.rs`), but it still needs a GPU, both to fail a test and to measure the effect.

- **The options:**
  - **(A) Keep the read out of the D11 gate hook in the float forward** (chosen, next unit). The float reference then computes the read as training did and as the artifact serves it. The next unit's ORDER picks the form and scopes the files it needs, with three constraints the decision review set:
    - **Not a scoped `with_gate_mode`.** That function (`hamilton_sheet.rs:1978`) restores only the export and telemetry flags. It leaves the arm at `HamiltonActive` and resets `fwd_call_idx` to 0, so layers 3 to 6 would run unsnapped after the stream read. The form must save and restore the whole hook state, as `with_d11_snaps` does, or route the read's `rotate` around the hook.
    - **A trunk check.** With the fix in, the trunk-only figures (5 and 11 of 1,536 positions differing, claude-233) must stay as they are.
    - **The Export readout mode.** `FbsReadoutMode::Export` runs under `with_d11_export_gates`, whose `icosian_export_mode` snaps every gate whatever the arm (the hook's `(_, true)` branch). The fix either keeps the read out of that path too, or the ORDER states that the Export mode is not a parity reference.
  - **(B) Snap the read in the integer port and the export as well.** Rejected. It would serve a read the model never trained with, against claude-187's design. D11 does not require a snap: CHARTER §3.6 asks for integer-only serving, and the engine already serves raw gates through `stack_rotation` (`lut-stack/5`'s `raw_gates`).
  - **(C″) Train the read under the snap.** Not taken here. It is a new configuration with its own experiment, not a repair of the reference.
  - **(C) Make fix (A) in this unit.** Rejected. Its failing test needs CUDA, and the ORDER's "fix follows here" clause asks for its effect measured on the laptop against the sealed float traces, which the fix makes obsolete. A GPU re-trace needs its own prep sheet, FIT and pre-run review.
- **This unit's part.** The probe that found the snap became a committed control (a′) in the check. So the check names this stage itself, and will show the fix's effect on the next trace: (a) should then equal the float trace.
- **The review:** a fresh-context decision reviewer, recorded on #3385 as `[claude] owner log:` (below, under Review).

## What changed

- `crates/uor-r4-integer/src/fbs_snapshot.rs`, the native half, with no serving dependency:
  - `hamilton_gate_f32` and `pack_signs_f32`, an f32 port of the trainer's CPU forward and its sign packing;
  - `icosian_roots_f32`, the 120 unit icosians in the trainer's order, and `snap_icosian_f32`, the nearest root lane by lane (the first root on a tie; root 1 for an all-zero lane, as the GPU's kernel does);
  - `float_copy_codes` and `copy_code_reads`, which compute (a) and (a′) per vocabulary entry, head and depth;
  - `copy_code_check` and `check_copy_trace`, the comparisons, with schema `uor-r4.copy-codes/1`. They refuse a trace whose ids, depths, heads, code counts or code widths do not fit the artifact, or whose side does not match;
  - their tests: a planted bit, a planted snapped bit, twelve refusals, and the snap's nearest-root test.
- `crates/uor-r4-integer/src/stack/hamming.rs` and `stack/mod.rs`: `served_copy_codes_uorlut01` and `ServedCopyCodes`, the session's own copy codes with its table, for (b).
- `crates/uor-r4-integer/src/stack/tests.rs`: `the_copy_code_check_holds_the_session_trace_to_its_served_codes`. The served codes must equal `CopyHeads::code` at every entry and head, and must differ by depth, head, entry and neighbouring position, so a wrong index or an unshifted key cannot pass. The integer trace must equal them, with 0 inconsistent.
- `crates/uor-r4-integer/src/bin/fbs-toks.rs`: `fbs-toks ARTIFACT --copy-codes FLOAT_TRACE INTEGER_TRACE OUT_JSON`. It refuses an existing output file and swapped traces, and prints one summary line.
- `crates/uor-r4-float-train/src/flock_read_hamming.rs` (tests only):
  - the port against the trainer's `hamilton_gate_fwd_cpu` and `pack_signs`, bit for bit;
  - the roots against `h4_unit_roots_f32`, bit for bit;
  - the CUDA-only snap test against `icosian_snap_cuda_tensor`. It has a 0.0 lane and a −0.0 lane, and four lanes midway between the identity and one of its nearest icosians. Those are ties in exact arithmetic, so either tied root is accepted there.
- The ORDER, amended on 1 Nov: the check's placement (D29), and (a′) in done-when 1.
- `out/`: the four outputs and their `SHA256SUMS`.
- The item's step 5 records this result and the next step.

## Measurements

| Output | sha256 | Artifact | Float trace | Integer trace |
|---|---|---|---|---|
| `copy-codes-h1-s1-d11.json` | `61183f95…ef45` | `28b01bba…9054` | `2ef1584a…82cc` (d11) | `a8dbc19c…bdfc` |
| `copy-codes-h1-s1-d11-rounded.json` | `193de6b7…fc7e` | `28b01bba…9054` | `07195625…29db` (d11-rounded) | `a8dbc19c…bdfc` |
| `copy-codes-h3-s3-d11.json` | `99c56b73…3083` | `1d1d2dc8…c566` | `7b8a42f2…3f1d` (d11) | `cca428db…d8cc` |
| `copy-codes-h3-s3-d11-rounded.json` | `472823b4…fec7` | `1d1d2dc8…c566` | `f17f2842…44a3` (d11-rounded) | `cca428db…d8cc` |

Every output holds the full sha256 of each input, and the paths it read. The artifacts are claude-233's (`results/claude/integer-nearest-readout-locate-2026-10-31/out/h{1,3}/artifact.uorlut` in the store). The float traces are claude-235's (`results/claude/integer-hamming-read-trace-2026-10-31/out/s{1,3}/{d11,d11-rounded}/trace.json`), and the integer traces are claude-235's `laptop/reading/trace-integer-s{1,3}.json`.

**Value bins of the bits where the float trace differs from (a)**, over queries (bins: |v| < 10⁻⁶, 10⁻⁵, 10⁻⁴, 10⁻³, 10⁻², 10⁻¹, ≥ 10⁻¹):

| Run, depth | Bins |
|---|---|
| h1 d2 | 0, 6, 35, 560, 3,998, 5,008, 273 |
| h1 d6 | 0, 6, 53, 591, 4,987, 9,592, 695 |
| h3 d2 | 1, 5, 66, 593, 3,695, 2,311, 57 |
| h3 d6 | 0, 4, 55, 676, 4,959, 8,025, 476 |

## Checks run at the head

- **At `f8af79d75`** (review round 1's code; the commits after it change only docs and the outputs):
  - `cargo test --release -p uor-r4-integer --features serving-wasm --lib --bins`: the lib 402 passed, 1 ignored; `fbs-toks` 4, `fbs-compare` 1, `uor-chat` 3.
  - `cargo test --release -p uor-r4-float-train --lib`: 194 passed.
  - The CUDA type-check, with a stub nvcc that writes an empty entry per kernel (for the vendored candle kernels, the names in `entry_allowlist.rs`): `CUDARC_CUDA_VERSION=12080 NVCC=<stub> UOR_NVCC_ARCH=compute_120 cargo check --offline -p uor-r4-float-train --features cuda --lib --tests`, exit 0.
  - **Three negative gates.** Each sets its lines aside, runs the test, then restores the file (`git checkout`); the copy-code tests are green again after (26 passed):
    - **the width refusal (round 1) set aside:** the refusal test panics with "index out of bounds: the len is 20 but the index is 20". That is the failure the review predicted;
    - **the counts of (a) against (b) set aside:** the planted test fails, `(5, 0, 0)` against `(5, 4, 1)`;
    - **the snapped count set aside:** it fails, `0` against `4` ("entry 1 at positions 1 and 6, and position 3").
  - `cargo fmt -p uor-r4-integer -- --check`: clean. The float crate is not rustfmt-clean on main. This unit's lines in `flock_read_hamming.rs` are formatted, and the file's other 20 diffs are main's.
  - `cargo clippy`, on the integer crate with `serving-wasm --lib --tests --bins` and on the float crate with `--lib --tests`: no warning in this unit's lines. The 7 in `stack/tests.rs` are on main's lines.
- **At `9df9701d9`** (before round 1, whose change the narrow tests above cover):
  - **The integer crate's whole suite,** `--features serving-wasm --no-fail-fast`: the lib, the bins and the doc tests pass. 35 tests in 7 integration files fail, every one on a model file this laptop does not have (`NotFound` for a quaternion or Householder bundle, or for `tokenizer.json`). This branch touches no file in `crates/uor-r4-integer/tests/`.
  - `uor-harness loop gates --quick` and `--full`: green.
  - **The rerun.** `fbs-toks --copy-codes` on h3 / s3 d11, written to a new path, reproduces `copy-codes-h3-s3-d11.json` byte for byte (`99c56b73…`). Run again to the same path, it refuses ("File exists"). With the traces swapped, it refuses ("the float-d11 trace was written by the integer side").
- `uor-harness loop report --check` and `uor-harness loop pr-check --local`.

## Review

**Round 1, two fresh reviewers at `9df9701d9`,** given only the ORDER, the item and their slice of the diff:
- **Slice A, the check's code (REQUEST CHANGES):**
  - **Major: a trace code wider than its head was not refused.** Its bits then read other heads' or entries' values, and past the last entry the check panicked. **Fixed:** the reads' codes and both traces' codes are refused when wider than a head. Two refusal cases pin it, with the negative gate above.
  - **Minor: the snap's doc claimed the GPU's rule outright.** The GPU sums the four products in its own order, and its arg-max does not promise the first of tied roots. **Fixed:** the doc says so, and the CUDA test now has exact-tie lanes.
  - **Minor: nothing ties a trace to its artifact.** Kept as a recorded limit (above): every output records each input's sha256.
  - **Minor: the key convention (the key at `s` is the token at `s − 1`) is assumed by the planted test.** The sealed runs settle it: the integer trace's keys equal the session's codes on 3,056 of 3,056 in every run, which a wrong convention could not give.
- **Slice B, the tests, the CLI and the served codes (APPROVE):** it confirmed `served_copy_codes_uorlut01` builds the session's own `CopyHeads`, from the same image, table and constructor.
  - **Minor: one copy head and one read in the planted fixture,** so a head-major index or a wrong read index would pass it. **Fixed:** the session test asserts that the codes differ by depth, head and entry.
  - **Minor: the key shift is pinned only if neighbouring codes differ.** **Fixed:** the session test asserts that they do.
  - **Minor: the CUDA test had no tie lane and has not run.** **Fixed:** a −0.0 lane and four tie lanes. That it has not run is recorded above.

**Round 2, a fresh reviewer at `f8af79d75`,** given slice A's findings and the round-1 fix diff (APPROVE):
- all five round-1 fixes hold: the width refusals, the snap doc, the session test's depth, head, entry and neighbour asserts, and the CUDA test's tie lanes;
- **Minor: the CUDA test's doc comment still described its old lanes.** **Fixed:** it names the spread, 0.0, −0.0 and exact-tie lanes.

**A claims check, a fresh reader at `90d7a4f86`,** given the REPORT, the notebook row, the item's step and the outputs (CORRECTIONS NEEDED, all applied):
- the Measurements table's sha suffixes and the h3 prefixes were wrong. They now come from the outputs and `SHA256SUMS`, and the d11-rounded traces are named;
- the plain summary said the integer version is right. It now says the copy codes are, and that the parity is still unmeasured;
- the snap of the query, value and step rotations was stated as measured. Only the key rotation's is; the others are read from the code;
- "training does not snap" is narrowed to the checkpoints' trainer, `train-in-space`, identified by its run config;
- the GPU arithmetic is named a fifth stage, with the ORDER's stop rule applied by analogy;
- the distinct-token count now follows claude-235: 3,072 distinct (token, copy head) pairs.

**The decision review, a fresh reader of the whole project at `90d7a4f86`** (PASS for (A), with three corrections, all applied in "The decision, and its review" and in the item's step 5): not a scoped `with_gate_mode`, a trunk-only regression check, and the Export readout mode.

## Skills used

None.

## Still open

- **The next unit: keep the read out of the D11 gate hook** (option A, under the decision review's three constraints), with a CUDA test that fails without it. Then re-trace on the GPU under a prep sheet, its FIT and its pre-run review. It runs `fbs-toks --copy-codes` on the new trace: (a) should then equal the float trace at every code. Then it reads the kept-set share and top-1 against the registered bar (done-when 2).
- **The CUDA-only snap test** (`the_copy_code_snap_is_the_gpus_hooks_own`) runs on that pod.
- **claude-235's two CUDA tests for the tie switch** are still owed to the next pod run.
- **The timing at long contexts** (done-when 3): the sweep still stops at 192.
- **The item stays `ready`;** done-when 2 and 3 remain open.

## Plain summary

The integer and float versions of the model's look-back read disagreed, and this unit found where. The GPU's float reference rounds the read's rotations to the nearest of 120 fixed symmetric rotations. Training never did that, and the integer version doesn't either. A laptop check that applies the same rounding reproduces the GPU's codes exactly, on every token, on both seeds. Without the rounding, the laptop's float codes and the integer codes agree exactly. So the integer copy codes are right, and the GPU reference was the wrong one to compare against. This covers the copy heads' codes only. The type heads, the kept sets and the decoded tokens are still to be compared, against a fixed reference. The next step is to fix the reference on the GPU and measure the parity again.
