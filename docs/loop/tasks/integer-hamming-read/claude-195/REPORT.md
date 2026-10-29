# REPORT: integer-hamming-read, claude-195 (the type heads served)

Lab: claude. Order: [ORDER.md](ORDER.md). Refs #3385. The item is not done: this unit is step 3 of its six, and steps 4 to 6 remain.

**Speed.** Not measured. The read is not yet in a trained artifact (the exporter, step 4), and decode tokens/s with and without the read is step 6.

## What did not work, first

- **The first tests could not fail on the type mean's divisor** (review round 1, tests, blocking).
  - The spec takes the mean over the type heads that found a candidate. Under the synthetic margins no candidate counted, and under the wide margins both heads found one at nearly every row. The hook tests share `TypeHeads::mean`.
  - So dividing by every head passed. Two seeds now mix one wide and one narrow type-head margin, and the test asserts more than 40 rows where exactly one head found a candidate. The wrong divisor now fails (gate 7).
- **Nothing pinned the payload's rounding** (review round 1, tests). The `2^-16` bound on the mean is loose enough to pass a truncated payload.
  - Each cached payload is now checked against the f64 rotation at half a grid step, plus `2^-10` of a step for the unit quaternion's own Q31 quantization. At ±4 inputs that error is below `2^-11`.
  - The first run of this check found a payload 0.50003 steps off: the quantization, not a fault. A truncated payload now fails (gate 8).
- **The ORDER's first bound for the type mean could not hold.** It asked for `2^-20` of the input's scale. Each payload is rounded half away from zero back to the input's grid, so a mean can be off by up to half a step, `2^-17` at the stream's Q16. The bound was amended to one grid step, `2^-16`, before any code ran.
- **A saved session cannot carry the type heads.** Their per-position payloads are not in `SerializedStackSession`, and its byte codec (`session_codec.rs`) is outside the item's scope. So a restore with type heads is refused by name, instead of continuing differently. A read with copy heads alone keeps only the tokens, and still restores.
- **No small synthetic shape reaches the payload cache's 4 GiB bound.** So the bound is a function of its own, tested at its limit and one past it. Its call at load has no negative gate.
- **One negative gate is caught only by the independent kernel test,** not by the hook tests: setting aside the value rotation.
  - The hook references share `TypeHeads::payload`, `queries` and `mean` with the code, as claude-194's shared `CopyHeads::mean`. So they pin the hooks' placement, the cache write and the combine, not the kernel's arithmetic.
  - The kernel test against f64 pins that arithmetic.

## What changed

- **`crates/uor-r4-integer/src/stack/hamming.rs`:**
  - **`TypeHeads`**, the float read's heads `h ≥ heads / 2`:
    - **keys:** the code of `x_{s−1}`'s writer-table point through `key_rotation`, built once per vocabulary entry, as the copy heads' keys;
    - **`queries(point)`:** the code of the hook's input through `query_rotation`, on each type head's coordinates;
    - **`payload(point, out)`:** the full-width input through `value_rotation`, rounded half away from zero back to the input's exponent;
    - **`choose`:** the copy heads' choice, against the type keys;
    - **`mean`:** the copy mean's form over the payloads. Each head's `Σ w_s · v_s / max(1, #{w_s > 0})`, then the mean over the heads that found a candidate, at the payloads' exponent minus 31.
  - **Shared helpers.** The copy heads' code builder and choice become `units`, `head_code`, `vocab_codes` and `nearest`, so the type heads run the same audited paths. `nearest` keeps `popcount_swar`.
  - **`ServedHamming`:**
    - builds a read's type heads when any of its type weights is not zero;
    - keeps `w_y`;
    - checks the payload cache (`check_payload_cache`, at most the read cache's 4 GiB);
    - gives the session its caches (`payload_caches`).
  - **The stream hook** forms `x · 2^55 + w_c · c · 2^4 + w_y · y` exactly at `2^-71` and rounds it once to Q16 (`stream_sum`).
  - **The readout hook** adds `w_y · y`, rounded first to the copy move's `2^-67`, so the sum stays inside an i128 for any `n`. Then it rounds once to `2^-24` (`readout_sum`).
  - Before either hook's type mean, the input's payload is written at row `t`, for the rows after it.
- **`crates/uor-r4-integer/src/stack/session.rs`:**
  - the session holds `hamming_payloads`, one `context × width` cache per read with type heads;
  - both step bodies pass the caches to the hooks;
  - `restore_state` refuses a session of a read with type heads;
  - the load passes the context to `ServedHamming::new`.
- **`crates/uor-r4-integer/src/stack/mod.rs`:** exports `TypeHeads`.
- **`crates/uor-r4-integer/src/stack/tests.rs`:**
  - six new tests;
  - claude-194's helpers take a type-weight scale, and its copy-only tests pass 0;
  - claude-188's round-trip and claude-194's refusal tests now expect a non-zero type weight to be served;
  - claude-194's tie test takes the new signatures.

## Measurements

All on the laptop, on synthetic trunks; no trained checkpoint.

- **The kernel against f64** (`type_heads_match_the_f64_reference`). A 256-row, 256-wide table and a 4-head read (2 type heads of 64 coordinates), 7 seeds × 48 rows of random ±4 points at Q16. The margins:
  - two seeds use the synthetic ones (1 to 9 bits of 64), where candidates almost never count;
  - three use wide ones (33 to 39 bits);
  - two mix one wide type head (37 bits) with one synthetic.

  Results:
  - every vocabulary key code equal (256 × 2 per seed);
  - every row's two query codes equal;
  - every cached payload within half a grid step plus `2^-10` of the f64 rotation;
  - every kept set equal with its distances;
  - every type mean within `2^-16` of the f64 mean, with the same rows found and not found;
  - candidates found at more than 100 rows, and exactly one type head found at more than 40.
- **The stream read with type heads** (`the_stream_read_adds_both_kinds_of_head`, read at depth 1):
  - At all 40 rows, the residual after layer 1 equals `round_half_away(x · 2^55 + w_c · c · 2^4 + w_y · y, 55)`. Here `x` is the read-free residual, and `y` the type mean over the payloads of the read-free residuals so far. A layer before the read cannot see it, so these are exactly the read's inputs.
  - This holds with both kinds of head and with type heads alone (copy weights 0).
  - In each case the read moved more than 10 rows, and a type head found candidates at more than 10.
- **The readout move with type heads** (`the_readout_read_adds_both_kinds_of_head`, read at depth 3, the layer count):
  - at all 40 rows, the final norm output equals `round_half_away(n · 2^43 + [c] w_c · (c − n · 2^19) + round_half_away(w_y · y, 12), 43)` on the read-free `n`;
  - the Q12 query is that point converted;
  - this holds with both kinds of head and with type heads alone. In each case the read moves more than 10 queries, and a type head finds candidates at more than 10 rows;
  - the reference copies the design's pre-rounding of the type term to `2^-67`, kept deliberately so that the sum stays inside an i128 (ORDER, made exact in review round 1).
- **Weight 0 with type heads present** (claude-194's `a_read_at_weight_zero_serves_as_the_read_free_trunk`, now with both weight rows scaled by 0): all 40 steps equal the read-free trunk's. At `w_y = 0` the type heads are not built.
- **The type term's single rounding** (`the_type_term_joins_the_single_rounding`). In the stream, `3 − 0.5 → 3` and `−3 + 0.5 → −3`. A copy and a type term of 0.25 each give `3.5 → 4`, where rounding each term apart would give 3. On the readout, a type term moves `n` with no copy match.
- **The restore refusal** (`a_session_with_type_heads_is_not_restored`):
  - a session with type heads, saved after 8 tokens, is refused by `restore_state` with its exact text;
  - the same trunk with copy heads alone restores, and continues bit for bit for the remaining 32 tokens.
- **The cache bound** (`the_payload_cache_is_bounded`):
  - exactly 4 GiB passes, at `(1, 2^16, 2^13)` and `(2, 2^16, 2^12)`;
  - one column more, a third read, and an overflowing product are each refused by the exact text.
- **The timed steps** (claude-194's test, now with both kinds of head): `measure_fbs_step_stages` and `measure_fbs_step_parts` equal `step_native` at all 40 rows, with the same saved state at the end.

**Negative gates, all at `9ac99efbc`.** Nothing in the code changed after it.
- **What was run:** each set-aside built, its tests failed, and the tree was restored with `git checkout` of the file (`local/claude/neg/ng*.out`).
  - The logs hold the test output only. The short head and the clean `git status` were printed by each command in the session, not into the logs.
  - Every log is timestamped after `9ac99efbc`'s commit, and the restored tree passes all 23 (`neg/restore.out`).
- **Earlier runs:** the runs at `7b449fbee`, before review round 1, are kept in `neg/pre-round1/`.
- **Scope:** the failures are counted within the 23 read tests, or, for gate 5, the 4 `m5_full_path_cost_controls` tests.

| Set aside | Failed |
|---|---|
| 1. the payload write in the hooks | the stream and readout type-head tests |
| 2. the value rotation (payload = the input) | the f64 kernel test only, of the 23: the hook tests share `payload` |
| 3. the stream hook's type mean | the stream type-head test |
| 4. the readout hook's type mean | the readout type-head test |
| 5. `popcount_swar` in the shared `nearest` (`count_ones`) | both `m5_full_path_cost_controls` serving tests |
| 6. the restore refusal | the restore test |
| 7. the type mean divided by every type head, not by those that found one | the f64 kernel test, at seed 5, a mixed-margin seed |
| 8. the payload truncated (`>> 31`) instead of rounded | the f64 kernel test, on the payload check |

## Checks run at the head

At `9ac99efbc`, the code head. The later commits change only this folder and the notebook. Logs are under `local/claude/` in the worktree, and are copied to the lab's folder at merge.
- **`cargo test --locked -p uor-r4-integer --features serving-wasm --no-fail-fast`**, with `~/.cargo/bin` first on PATH: 572 passed, 35 failed, 4 ignored (`full-integer-tests-2.out`).
  - Every failure panics on a missing local input, as on `main` (`failures-195.txt`):
    - 32 a bundle: 13 "load real quaternion bundle", 12 "load bundle", 2 "load real householder_pair bundle", and 1 each of "load quaternion bundle", "Quaternion bundle path must exist", "Quaternion bundle must exist", "Householder bundle path must exist" and "Householder bundle must exist";
    - 3 the tokenizer.
  - 572 is claude-194's 566 and this unit's 6 tests.
  - The `m5_full_path_cost_controls` step verifier passes inside the same run.
- **The 23 read tests:** the Hamming, copy-head and type-head tests all pass.
- **The serving audit** (`audit_zero_matmul_serving.py --stack` on the release `uor-r4-stack`, `audit-195.out`): FULL PASS. 0 forbidden instructions across 44 symbol ranges and 109 reachable functions, up from claude-194's 106.
  - `TypeHeads::new`, `mean`, `choose` and `payload` are symbols of the audited binary (`nm-195.out`). `queries` is inlined; only its `collect` closure's instantiation is a symbol.
  - That the audit walks the type mean is shown by gate 5's verifier output, where the closure of `nearest` fails for both kinds of head.
- **The GPU evaluator's host tests** (`docs/labs/d11-exporter-float-trunk/instrument`, `--target aarch64-apple-darwin`, `instrument-host.out`, at `18c2b945c`; nothing it reads has changed since): 5 passed, 1 ignored (it needs a trained artifact).
- **`cargo clippy -p uor-r4-integer --all-targets`** (`clippy-195.out`): no warning in `hamming.rs`, `session.rs` or the new tests. Its three `tests.rs` warnings, at lines 864, 1794 and 3672, are older tests on `main` (`15c474a2d`, `9556c1bd3`).
- **`uor-harness loop gates --full`:** 11 of 11 green at `18c2b945c`, and re-run at the head before the review record.

## Skills used

- None was invoked this unit. The ORDER's bound was amended before the code ran, with its reason.

## Still open

- **Steps 4 to 6 of the design:**
  - the exporter reading `hamming_read.safetensors`. It must map the float's readout depth 6 to the layer count, or refuse another depth (claude-194's review);
  - the float parity on claude-146's checkpoints;
  - the speed with and without the read.
- **Restoring a session with type heads.** It needs the payload caches in `SerializedStackSession` and its byte codec (`session_codec.rs`, outside this item's scope), or a restore that replays the saved tokens.

## Plain summary

The integer runtime now runs the second half of the memory lookup too: the heads that look back by meaning, matching the current state against earlier positions and bringing back a rotated copy of what was there. Their numbers match a floating-point reference of the trained read's steps, and the forbidden-instruction audits still pass.
