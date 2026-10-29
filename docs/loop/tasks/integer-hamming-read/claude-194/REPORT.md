# REPORT: integer-hamming-read, claude-194 (the copy heads served in the FBS session)

Lab: claude. Order: [ORDER.md](ORDER.md). Refs #3385. The item is not done: this unit finishes step 2 of its six, and steps 3 to 6 remain.

**Speed.** Not measured. The read is not yet in a trained artifact (the exporter, step 4), and decode tokens/s with and without the read is step 6. The timed step runs the read inside the block it follows, so `fbs-toks`' stage list is unchanged.

## What did not work, first

- **claude-189's kernel was not clean for serving.** Its audit run passed because nothing on the served path called the kernel. Once the hooks put it there, each of the two disassembly audits found SIMD in it:
  - **The serving audit** (`audit_zero_matmul_serving.py --stack`) found 4 `fmov` in `CopyHeads::mean` (`local/claude/audit-194-before-blackbox.out`, kept on the laptop). LLVM loaded the table points eight at a time into a NEON register and moved them out to multiply. Each point now goes through `black_box`, as the kernels' own loads do.
  - **The step verifier** (`verify_step_objdump_zero_mul_zero_float`, run by `m5_full_path_cost_controls`) found `fmov d0, x8` and `stur s0` in `choose`'s closure. `u64::count_ones` lowers to the NEON `cnt` on arm64. `choose` now uses `bitcode`'s `popcount_swar`, which exists for exactly this reason. The Python audit had passed that closure, a generic `core::iter` symbol, so the full crate suite is what caught it.
- **A latent panic in the kernel.** The negative gate that pushes the token after the layers made `CopyHeads::choose` index `tokens[0]` on an empty slice. The served path always pushes first, so serving could not hit it, but the kernel now keeps nothing for no tokens, pinned in a test.
- **Done-when 2's third bullet was amended before the code.** It first said "the changed steps are exactly those where a copy head found a candidate". That holds for the readout read only. A stream read changes the later layers' recurrence states, so a later row with no candidate can still differ. The amended bullet checks the residual at the hook for the stream read, and "no candidate, no change" for the readout only.

## What changed

- **`crates/uor-r4-integer/src/stack/hamming.rs`: `ServedHamming`**, built at model load from the `/6` image and the writer table (the float trunk's `embedding.weight`). Each read keeps its `CopyHeads` and `w_c`, weight row 0.
  - **`stream(depth, tokens, table, x)`:** after the layer whose count is the read's depth (`1 ≤ δ < layers`), `x + w_c ⊙ c`. Each product, at `2^-24 · 2^-43`, is rounded half away from zero to the residual's `2^-16`, as the float trunk's `q16(read.apply(...))` (`geometric_stack_native.rs:1148`) rounds its output. No candidate adds nothing, as the float's `sums / max(found, 1)` is 0 there.
  - **`readout(tokens, table, n)`:** on the final norm's output at `NORM_FINE_EXP`, after the final gain and before the Q12 query, `n + w_c ⊙ (c − n)` where a copy head found a candidate. The difference is formed exactly at `2^-43`, and the move is rounded once to `2^-24`. It is not rounded to Q16, as the float's line 1187 is not.
  - **The refusal:** a read with any non-zero type-head weight (weight row 1) is refused at load, by `"a Hamming read's type heads are not served yet: its type-head weights (weight row 1) must be zero"`. The format already refuses a read on a dense stack and a depth of 0 or past the layers (claude-188), so the session needed no second check for those.
  - **The kernel's two scalar fixes** (above), and the empty-`tokens` guard in `choose`.
  - A `#[cfg(test)]` layer trace (`LAYER_TRACE`), off until a test turns it on, and absent from every non-test build.
- **`crates/uor-r4-integer/src/stack/session.rs`:**
  - `FbsModel` gains `hamming: Option<ServedHamming>`, built in `fbs_model`.
  - `IntegerStackModel::parse` no longer refuses every `/6` artifact.
  - **`stack_step_fbs`** keeps the token before the layers run, so the read at row `t` sees `x_0..=x_t`. It calls `stream` after each layer and `readout` after the final gain.
  - **`stack_step_fbs_timed`** does the same, inside the blocks it times, so the stage list is unchanged.
  - **`fbs_image()` returns `None` for a trunk with a read.** The view does not carry the read, and an evaluator of the view alone would drop it. The GPU evaluator instrument then stops with its own "not an FBS stack".
  - A `#[cfg(test)]` accessor for the last step's final norm output.
- **`crates/uor-r4-integer/src/stack/tests.rs`:** five tests and their helpers, on a synthetic trunk of claude-127's non-square shape (width 64, 3 layers, 4 heads of 16 coordinates) over 40 positions with exact repeats, with `synthetic_hamming`'s reads, type weights zeroed. claude-188's round-trip test now matches the type-head refusal's text.

## Measurements

All on the laptop, on synthetic trunks; no trained checkpoint.

- **Off at weight 0** (`a_read_at_weight_zero_serves_as_the_read_free_trunk`): with reads at depth 1 and on the readout and `w_c = 0`, all 40 steps equal the read-free export's, bit for bit: the native token, its kept cells and the Q12 query. Meanwhile the copy heads found candidates at more than 10 rows. The view returns `None`.
- **The stream read** (`the_stream_read_adds_the_rounded_copy_to_the_residual`; read at depth 1 only): at all 40 rows, the residual after layer 1 equals the read-free trunk's residual there plus `round_half_away(w_c · c, 51)`. A layer before the read cannot see it, so the read-free run gives the read's input exactly. The read moved more than 10 residuals and changed more than 10 queries.
- **The readout move** (`the_readout_read_moves_the_norm_output_toward_the_copy`; read at depth 3, the layer count):
  - at all 40 rows, the final norm output equals `n + round_half_away(w_c · (c − n · 2^19), 43)` on the read-free `n`, and the Q12 query is that point converted;
  - every row without a candidate (at least rows 0 and 1) keeps the read-free query;
  - more than 10 queries changed.
- **The refusals** (`the_session_refuses_what_it_does_not_serve_by_name`), each by its exact text: one non-zero type weight; a read on a dense stack; depths `[0, 3]` and `[1, 4]`.
- **The timed steps** (`the_timed_fbs_steps_serve_the_read`): with both reads, `measure_fbs_step_stages` and `measure_fbs_step_parts` equal `step_native` at all 40 rows, with the same saved state at the end. The read changed more than 10 steps against the read-free trunk.

**Negative gates.** Each set-aside built and failed its tests; restored, all pass (`local/claude/neg/*.out`):

| Set aside | Failed |
|---|---|
| the stream hook in `stack_step_fbs` | the stream test, the timed test |
| the readout hook in `stack_step_fbs` | the readout test, the timed test |
| the token kept before the layers (pushed after them, as before) | the stream, readout, weight-0 and timed tests |
| rounding (a truncating shift) | the stream and readout tests |
| the timed step's hooks | the timed test |
| the type-weight refusal | the refusal test, claude-188's round-trip test |
| `popcount_swar` (`count_ones`, the first full run) | both `m5_full_path_cost_controls` verifier tests |
| `black_box` on the points (the first audit run) | the serving audit: 4 forbidden instructions |

## Checks run at the head

At `7b6e65ec4`, the code head. The later commits change only this folder and the notebook.
- **`cargo test --locked -p uor-r4-integer --features serving-wasm --no-fail-fast`:** 564 passed, 36 failed, 4 ignored.
  - 35 of the failures panic on a missing local input, as on `main` at claude-189: 32 a bundle, 3 the tokenizer.
  - The 36th, `library_cli_and_browser_produce_bit_identical_tokens_and_snapshots_across_frozen_panel`, failed because `wasm-pack` ran Homebrew's `cargo` from PATH ("unknown command 'build'"). It is an instrument fault, recorded in the friction log. Re-run with `~/.cargo/bin` first on PATH, its file passes, 2 of 2.
  - So 565 pass, against claude-189's 560: this unit's 5 tests.
- **The 16 Hamming and copy-head tests:** all pass, claude-189's 3 included.
- **The serving audit** (`audit_zero_matmul_serving.py --stack` on the release `uor-r4-stack`): FULL PASS. 0 forbidden instructions across 44 symbol ranges and 106 reachable functions.
- **The step verifier** (`m5_full_path_cost_controls`): 4 of 4 pass.
- **The GPU evaluator's host tests** (`docs/labs/d11-exporter-float-trunk/instrument`, `--target aarch64-apple-darwin`): 5 passed, 1 ignored (it needs a trained artifact).
- **`cargo clippy -p uor-r4-integer --all-targets`:** no warning in the new code.
- **`uor-harness loop gates --full`:** 11 of 11 green.

## Skills used

- None was invoked this unit. The done-when was fixed in the ORDER before the code ran, and amended once, also before the code (above), in the spirit of holding-the-goal.

## Still open

- **Steps 3 to 6 of the design:**
  - the type heads (queries from the residual, payloads through the value rotation, `w_y`);
  - the exporter reading `hamming_read.safetensors` (`fbs_export.rs` still writes `hamming: None`);
  - the float parity on claude-146's checkpoints (GPU, under a prep sheet);
  - the speed with and without the read.
- **Outside this scope, noted:**
  - the GPU evaluator's "not an FBS stack" names the wrong cause for a `/6` artifact;
  - the Python audit does not walk `core::iter` closure instantiations that the Rust step verifier does.

## Plain summary

The integer runtime now uses the part of the memory lookup that finds an earlier copy of the current word and brings back what followed it. It runs at the same two places in the model as the float trainer does, and gives exactly the numbers the formulas say. Two kinds of instruction the serving rules forbid were removed from the lookup on the way.
