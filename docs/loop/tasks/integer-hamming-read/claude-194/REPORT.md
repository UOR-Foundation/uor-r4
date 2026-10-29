# REPORT: integer-hamming-read, claude-194 (the copy heads served in the FBS session)

Lab: claude. Order: [ORDER.md](ORDER.md). Refs #3385. The item is not done: this unit finishes step 2 of its six, and steps 3 to 6 remain.

**Speed.** Not measured. The read is not yet in a trained artifact (the exporter, step 4), and decode tokens/s with and without the read is step 6. The timed step runs the read inside the block it follows, so `fbs-toks`' stage list is unchanged.

## What did not work, first

- **claude-189's kernel was not clean for serving.** Its audit run passed because nothing on the served path called the kernel. Once the hooks put it there, each of the two disassembly audits found SIMD in it:
  - **The serving audit** (`audit_zero_matmul_serving.py --stack`) found 4 `fmov` in `CopyHeads::mean` (`local/claude/audit-194-before-blackbox.out`, kept on the laptop). LLVM loaded the table points eight at a time into a NEON register and moved them out to multiply. Each point now goes through `black_box`, as the kernels' own loads do.
  - **The step verifier** (`verify_step_objdump_zero_mul_zero_float`, run by `m5_full_path_cost_controls`) found `fmov d0, x8` and `stur s0` in `choose`'s closure. `u64::count_ones` lowers to the NEON `cnt` on arm64. `choose` now uses `bitcode`'s `popcount_swar`, which exists for exactly this reason. The Python audit had passed that closure, a generic `core::iter` symbol, so the full crate suite is what caught it.
- **A latent panic in the kernel.** The negative gate that pushes the token after the layers made `CopyHeads::choose` index `tokens[0]` on an empty slice. The served path always pushes first, so serving could not hit it, but the kernel now keeps nothing for no tokens, pinned in a test.
- **The first build rounded the product, not the sum** (review round 1, code, Low).
  - The float rounds the sum, `q16(x + w·c)`, and rounding half away from zero is not translation-invariant. So `x + round(w·c)` differed from it at an exact half tie: `3 − 0.5` gave 2, not 3.
  - Both hooks now round their sum once (`stream_sum`, `readout_sum`).
  - The trunk-level tests could not tell the two apart, since their reference applied the same formula and no tie occurs. So a direct tie test (`the_hooks_round_the_sum_not_the_product`) pins it.
- **The first negative gates ran before three code commits** (review round 1, claims check).
  - Gate 3 then failed by the latent panic above, not by an assertion.
  - All nine gates were re-run at `da0650313`. The earlier outputs are kept apart in `local/claude/neg/pre-guard/`.
- **Done-when 2's third bullet was amended before the code.** It first said "the changed steps are exactly those where a copy head found a candidate".
  - That holds for the readout read only. A stream read changes the later layers' recurrence states, so a later row with no candidate can still differ.
  - The amended bullet checks the residual at the hook for the stream read, and "no candidate, no change" for the readout only.

## What changed

- **`crates/uor-r4-integer/src/stack/hamming.rs`: `ServedHamming`**, built at model load from the `/6` image and the writer table (the float trunk's `embedding.weight`). Each read keeps its `CopyHeads` and `w_c`, weight row 0.
  - **`stream(depth, tokens, table, x)`:** after the layer whose count is the read's depth (`1 ≤ δ < layers`), `x + w_c ⊙ c`. The product is at `2^-24 · 2^-43`. Each sum is formed exactly there and rounded once, half away from zero, to the residual's `2^-16`, as the float trunk's `q16(read.apply(...))` (`geometric_stack_native.rs:1148`) rounds its output. No candidate adds nothing, as the float's `sums / max(found, 1)` is 0 there.
  - **`readout(tokens, table, n)`:** on the final norm's output at `NORM_FINE_EXP`, after the final gain and before the Q12 query, `n + w_c ⊙ (c − n)` where a copy head found a candidate. The sum is formed exactly and rounded once to `2^-24`. It is not rounded to Q16, as the float's line 1187 is not.
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
- **`crates/uor-r4-integer/src/stack/tests.rs`:** six tests and their helpers, mostly on a synthetic trunk of claude-127's non-square shape (width 64, 3 layers, 4 heads of 16 coordinates) over 40 positions with exact repeats, with `synthetic_hamming`'s reads, type weights zeroed. claude-188's round-trip test now matches the type-head refusal's text.

## Measurements

All on the laptop, on synthetic trunks; no trained checkpoint.

- **Off at weight 0** (`a_read_at_weight_zero_serves_as_the_read_free_trunk`): reads at depth 1 and on the readout, with `w_c = 0`.
  - All 40 steps equal the read-free export's: the native token, its kept cells and the Q12 query, the ORDER's "bit for bit".
  - Meanwhile each read's copy heads found candidates at more than 10 rows.
  - The view returns `None`.
  - The copy heads keep nothing for an empty token list.
- **The stream read** (`the_stream_read_adds_the_rounded_copy_to_the_residual`; read at depth 1 only):
  - at all 40 rows, the residual after layer 1 equals `round_half_away(x · 2^51 + w_c · c, 51)` on the read-free trunk's residual `x` there. A layer before the read cannot see it, so the read-free run gives the read's input exactly;
  - rows 0 and 1, which have no earlier position `1 ≤ s < t`, are asserted to find no candidate and add nothing;
  - the read moved more than 10 residuals and changed more than 10 queries.
- **The readout move** (`the_readout_read_moves_the_norm_output_toward_the_copy`; read at depth 3, the layer count):
  - at all 40 rows, the final norm output equals `round_half_away(n · 2^43 + w_c · (c − n · 2^19), 43)` on the read-free `n`, and the Q12 query is that point converted;
  - every row without a candidate keeps the read-free query. There are at least two such rows, and rows 0 and 1 are asserted among them;
  - more than 10 queries changed.
- **The tie** (`the_hooks_round_the_sum_not_the_product`): at exact half ties each hook rounds its sum (`3 − 0.5 → 3`, `−3 + 0.5 → −3`), and at `w_c = 1` the readout lands on the copy exactly.
- **The refusals** (`the_session_refuses_what_it_does_not_serve_by_name`), each by its exact text through `IntegerStackModel::parse`: one non-zero type weight (on the second read); a read on a dense stack; depths `[0, 3]` and `[1, 4]`.
- **The timed steps** (`the_timed_fbs_steps_serve_the_read`): with both reads, `measure_fbs_step_stages` and `measure_fbs_step_parts` equal `step_native` at all 40 rows, with the same saved state at the end. The read changed more than 10 steps against the read-free trunk.
- **What the references share with the code.** Each hook test takes its copy mean `c` from `CopyHeads::mean`, and the readout test converts to Q12 with `to_exp_i32`.
  - So these tests catch a wrong hook placement, token window, weight row, shift or rounding.
  - They do not catch a bug inside the copy mean itself. That is claude-189's `copy_heads_match_the_f64_reference`, against an f64 reference.

**Negative gates, all at `da0650313`.** Each set-aside built, its tests failed, and the tree was restored from the commit (`local/claude/neg/ng*.out`):

| Set aside | Failed |
|---|---|
| 1. the stream hook in `stack_step_fbs` | the stream test, the timed test |
| 2. the readout hook in `stack_step_fbs` | the readout test, the timed test |
| 3. the token kept before the layers (pushed after them, as before) | the stream, readout and timed tests, by assertion at rows 2 to 4 (the weight-0 test rightly passes: at weight 0 the window cannot matter) |
| 4. rounding (a truncating shift) | the stream, readout and tie tests |
| 5. the timed step's hooks | the timed test |
| 6. the type-weight refusal | the refusal test, claude-188's round-trip test |
| 7. rounding the sum (the product rounded instead) | the tie test only: the trunk never hits a tie |
| 8. `popcount_swar` (`count_ones` back) | both `m5_full_path_cost_controls` verifier tests, on `fmov d0, x8` |
| 9. `black_box` on the points | the serving audit: 4 forbidden instructions in `CopyHeads::mean` |

## Checks run at the head

At `da0650313`, the code head. The later commits change only this folder and the notebook. Logs are under `local/claude/` in the worktree.
- **`cargo test --locked -p uor-r4-integer --features serving-wasm --no-fail-fast`**, with `~/.cargo/bin` first on PATH: 566 passed, 35 failed, 4 ignored (`full-integer-tests-3.out`).
  - Every failure panics on a missing local input, as on `main` at claude-189: 32 a bundle, 3 the tokenizer.
  - 566 is claude-189's 560 and this unit's 6 tests.
  - **Without that PATH,** `wasm-pack` runs Homebrew's `cargo` ("unknown command 'build'"). The wasm parity test then fails as an instrument fault, as it did on the first run here (`full-integer-tests-2.out`). This is recorded in the friction log.
- **The 17 Hamming and copy-head tests:** all pass, claude-189's 3 included.
- **The serving audit** (`audit_zero_matmul_serving.py --stack` on the release `uor-r4-stack`, `audit-194.out`): FULL PASS. 0 forbidden instructions across 44 symbol ranges and 106 reachable functions.
- **The step verifier** (`m5_full_path_cost_controls`, in the full run): 4 of 4 pass.
- **The GPU evaluator's host tests** (`docs/labs/d11-exporter-float-trunk/instrument`, `--target aarch64-apple-darwin`, `instrument-host.out`): 5 passed, 1 ignored (it needs a trained artifact).
- **`cargo clippy -p uor-r4-integer --all-targets`** (`clippy-194.out`): none of its warnings is in `hamming.rs`, `session.rs` or the new tests.
- **`uor-harness loop gates --full`:** 11 of 11 green, at `7b6e65ec4`. It runs no crate code, and is re-run at the PR head before review.

## Skills used

- None was invoked this unit. The done-when was fixed in the ORDER before the code ran, and amended once, also before the code (above), in the spirit of holding-the-goal.

## Still open

- **Steps 3 to 6 of the design:**
  - the type heads (queries from the residual, payloads through the value rotation, `w_y`);
  - the exporter reading `hamming_read.safetensors` (`fbs_export.rs` still writes `hamming: None`);
  - the float parity on claude-146's checkpoints (GPU, under a prep sheet);
  - the speed with and without the read.
- **The readout's depth, for the exporter (review round 1, code, Low).** The two sides name the readout differently:
  - the float's readout is the read at the fixed `HAMMING_READOUT_DEPTH = 6` (`flock_read_hamming.rs:62–64`), whatever the layer count;
  - the integer's readout is the read at the layer count;
  - they agree on claude-146's 6-layer trunks.

  Step 4 must refuse a trunk whose layer count is not 6, or map depth 6 to the layer count.
- **Outside this scope, noted:**
  - the GPU evaluator's "not an FBS stack" names the wrong cause for a `/6` artifact;
  - the Python audit does not walk `core::iter` closure instantiations that the Rust step verifier does.

## Plain summary

The integer runtime now uses the part of the memory lookup that finds an earlier copy of the current word and brings back what followed it. It runs at the same two places in the model as the float trainer does, and gives exactly the numbers the formulas say. Two kinds of instruction the serving rules forbid were removed from the lookup on the way.
