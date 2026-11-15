# ORDER: one-model-end-to-end, claude-389 (the native Q31 first-pick read operator)

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407).
Base: `origin/main` at `e5a32f45`, which holds claude-388 (#3997).

## Why this unit

The first step of the 15 Nov architects' path is done. claude-388 showed that the vector read's hard copy, Flock's first pick alone at one fixed gain, reaches `R_ask` 0.965 to 0.984 at both seeds. That was measured on float reference states.

D85 says floating point is reference code only. So the next step is to write the same read as a native Q31 operator. It goes in `uor-r4-native-vector`, the crate that already holds codex's Q31 Hamilton gate, beside its CUDA twin.

The architects' step 2 names three parts:
- one typed operator;
- called by both the native training step and `IntegerStackSession`;
- with a bit-identical replay.

No native token table, training step for reads, or session read exists yet (codex-1 to codex-11 built only the gate). This unit therefore builds the operator and its device twin, and checks them against claude-388's saved states. Attaching the read to `IntegerStackSession` and training its fields natively are the next units, named under "Still open" in the REPORT.

Casey answered "Start the native Q31 read operator unit here?" with "Yes, start it" (15 Nov 2026).

## What is built

**1. `q31_first_pick_read`, in `crates/uor-r4-native-vector/src/read.rs`.** It is integer only, with no float, transcendental or normalization fallback. For one window of `time` rows, each `width` lanes (a multiple of 16), all words Q31:

- **Address.** Each row's stream is cut into `width / 4` quaternion blocks. Each block is rotated by its own query gate and its own key gate, using `q31_hamilton_rotate`, the crate's existing operator. The float read's `q ⊗ u` is the same left product. The rotated lanes are folded to 16 address words: coordinate `c` is the exact i64 sum of lanes `g·16 + c` over the `width / 16` groups. This matches `vector_reads.rs:669-678`.
- **Score.** The score is the L1 distance `Σ_c |q_t[c] − k_s[c]|`, accumulated in u64, over strictly earlier rows `s < t`. The float score's `/16` is monotone, so it is dropped.
- **Choice.** The least distance wins, and a tie goes to the latest `s`, as in the float read. Row 0 reads nothing. The choice is returned as `s + 1`, with 0 meaning none.
- **Payload.** The payload is the token table's row for the token after the pick, `table[ids[s + 1]]`. Each lane is weighted by a Q31 weight and a gain shift `g` in `0..=31`. For each lane: `p = sat((w·e + 2^(30−g)) >> (31 − g))`, then `out = sat(x + p)`. With `g = 0` this is the crate's own rounded Q31 product.
- **Refusals.** The operator refuses words outside the closed domain `-2^31..=2^31`, a width that is not a multiple of 16, `ids` and rows of different lengths, a token id outside the table, and a gain shift above 31.

The operator is a pure function. It returns the picks and the out words, and it changes no state.

**2. Its CUDA twin, in `crates/uor-r4-discrete/src/native_vector.cu`.** The new entry is `native_q31_first_pick_read`, built ahead with nvcc (no NVRTC), admitted in `NATIVE_VECTOR_ENTRIES`, and launched by a `NativeVectorEngine` method next to a `*_reference` twin. The kernel uses integer arithmetic only.

**3. A native replay in the probe** (`recall-payload-probe --native-replay`). At each forward of the float read, it captures exactly the operator's inputs:
- the stream rows `x` at the read;
- the window's token ids;
- the query and key gates, normalized as the float gate does;
- the weight;
- the shared token table.

As comparison-only fields it also captures the float first pick (`latest_best`'s rank 0), each row's float scores and the float out row. **No float pick, rank or score is passed to the native operator.** The replay converts the inputs to Q31 and runs `q31_first_pick_read` on the same windows. The comparison fields are read only after the operator returns.

Conversion (one rule, recorded in the JSON):
- The gates use the fixed Q31 unit scale `2^31` on both sides, rounded to nearest.
- The stream uses one power-of-two scale `2^e_x`, the largest with `max|x| · 2^e_x ≤ 2^31` over the state's captured rows. The query and key addresses share it, since both read the same stream.
- The table uses its own scale `2^e_t`, by the same rule.
- The weight scale is `2^e_w`, with `e_w = 31 + e_x − e_t − g`. The gain shift `g` is the least one in `0..=31` for which the weight fits. That way the operator's `>> (31 − g)` lands the payload on the stream's scale.

This conversion is a diagnostic in reference code (D85). It is not a model export, and nothing it makes is served.

## Native-space five items (SKILL)

- **State schema.** The operator's learned fields are the query gates and key gates (`width / 4` Q31 quaternions each), the per-lane Q31 weights (`width` words) and one gain shift. Its inputs are the Q31 stream words, the token ids and the Q31 token table.
- **Shared operator path.** There is one Rust source definition (`read.rs`) and one CUDA twin, and a bit-identical differential test covers both. A WASM build of the crate is a plain `--target wasm32` check, because the crate is dependency-free.
- **Replay oracle.** Raw words are compared, never rounded values: the picks (`u32`) and the out words (`i64`), CPU against CUDA, on fixed and seeded fixtures.
- **Forward, gradient, update.** Forward only. This unit has no gradient or update for the read's fields; the native update rule is a later unit. So by the SKILL this is an **operator unit, not a training run**.
- **Serialization.** The fields are raw little-endian words with a version byte, like `Q31HamiltonModel::to_bytes`, and a round trip is tested. Attaching them to the lut-stack artifact is the session unit's.

## Sheet (§4.5: one GPU evaluation, the native replay on claude-386's saved states)

- **States.** These are claude-386's saved states on wick, under `/workspace/in/c387/`, with their `model.safetensors` / `vector_read.safetensors` sha256:
  - **Barred**, the 16,384-step states:
    - `try-5/ext-flockg-b2-1/run/run/models/step-016384` (`e39cb058…`, `944c759d…`);
    - `try-6/ext-flockg-b2-2/run/run/models/step-016384` (`13cfba37…`, `bccbdb23…`).
  - **Run and reported, with no bar**, the 4,096-step states:
    - `try-1/arm-flockg-b2-1/run/run/models/step-004096` (`bffad79f…`, `98a1a7c3…`);
    - `try-3/arm-flockg-b2-2/run/run/models/step-004096` (`b00f5adc…`, `1fa08f29…`).
- **Error bound B** (derived for width 256, so 16 fold groups `G`; `LSB = 2^−e_x` in float units):
  - Each rotated lane errs by at most 4 LSB: 1 from the stream's rounding through a unit gate, 2 from the gate's rounding, 0.5 from the product's rounding.
  - So an address coordinate errs by at most `4G = 64` LSB, and `|q − k|` by at most 128 LSB.
  - The L1 sum of 16 coordinates divided by 16, the float read's mean, errs by at most **`B = 8G = 128` LSB**.
  - Two scores each off by up to `B` can swap order only where their float scores are within **`2B`**.
- **Bar.** On the 1,024 held-out query rows of each barred state, both of these must hold:
  1. **Picks.** The native pick equals the float first pick on at least 99% of rows (at least 1,014).
  2. **Out words.** On the rows where the picks agree, every native out word, divided by `2^e_x`, is within `2 LSB + 2^−22 · |float out|` of the float out row, per lane. The `2 LSB` is the Q31 rounding of `x`, `w` and `e` and of the product. The `2^−22` relative term is f32's own rounding of `x + w·e`. The gain is `g` from the rule above, at the trained scale (×1), and the REPORT names it.
- **Noise.** None. Picks are deterministic, so a miss is rounding at a near-tie or a bug. The JSON counts, before any classification:
  - the rows whose float top-1 and top-2 scores are within `2B`, the most mismatches rounding can explain;
  - for each mismatching row, `|float score(float pick) − float score(native pick)|` in LSB.
- **Controls:**
  1. **The replay can fail on the real states.** The replay runs a second time with every key gate replaced by the identity `[2^31, 0, 0, 0]`, and its pick agreement must fall below 99% at both seeds. If it does not, the replay is uninformative, and the outcome reads as D whatever the bar says.
  2. **CPU against CUDA.** On the same converted inputs from one barred state, the operator and its CUDA twin agree bit for bit on picks and out words.
  3. **A 1-LSB perturbation.** In the fixture built near a tie, changing one key-gate word by 1 LSB changes at least one pick or out word. This shows the differential can fail.
  4. **Row 0.** Every window's row 0 returns pick 0, with out words equal to its stream words. The JSON counts these rows.
- **Report lines (no bar).**
  - Native first-right against the float counts, which were 1,000 and 990 in claude-388.
  - The 4,096-step states' agreement and first-right.
  - The out-word check at ×16 (`g + 4`).
- **Seeds.** 2 (claude-386's B2 seeds 1 and 2).
- **Cost.** Minutes on wick, $0. No pod and no training.
- **Reading map** (fixed before any number is read; it applies only when control 1 falls below 99%):
  - **P:** both bar conditions hold at both seeds. The Q31 operator reproduces the float read's choice and its write, and the next unit attaches it to `IntegerStackSession`.
  - **N1 (precision):** a condition fails, but every pick mismatch has `|float score(float pick) − float score(native pick)| ≤ 2B`, and every out-word miss is within 4× the out bound. The fix is a finer scale or a wider accumulator. It is not a mechanism negative.
  - **N2 (definition):** at least one pick mismatch above `2B`, or an out-word miss above 4× its bound. The native definition (rotation, fold, tie rule, gain) differs from the float one, and the next step is to find the difference.
  - **D:** anything else, including control 1 at or above 99%, reported as read.
- **FIT:** a fresh-context non-author review. The first review returned `TEST FITNESS: UNFIT` on 15 Nov with six findings, all folded in above:
  - the replay control on the real states;
  - one gate scale;
  - B derived, with its units;
  - the out-word condition in place of the redundant first-right condition;
  - the states named;
  - row 0.

  The amended sheet goes to a second fresh review before the run.

## Cancel probe

Before the work, I checked whether any native read operator exists on `main`. At base there is none: `uor-r4-native-vector` has only the gate, its rotation and its training step (codex-1 to codex-11).

## Files in scope

- `crates/uor-r4-native-vector/src/{lib.rs,read.rs}` and its tests
- `crates/uor-r4-discrete/src/native_vector.cu`, `src/native_vector_device.rs`, `build.rs` (the entry list only), `tests/native_vector_cuda.rs`
- `crates/uor-r4-float-train/` (the probe's `--native-replay` and its capture of the read's inputs; `Cargo.toml` gains the dependency-free `uor-r4-native-vector`)
- this folder, and one row in `docs/research/notebook/float-training.md`

## Stop list

- No training, and no change to the float read's default forward or to training.
- No change to `IntegerStackSession` or to the artifact format.
- No pod. No owner path.

## Done when

1. `q31_first_pick_read` and its tests pass on the laptop: fixtures for the tie rule, the fold, saturation, the gain and each refusal.
2. The CUDA twin matches the oracle bit for bit on wick (`--features cuda --test native_vector_cuda`), and the 1-LSB control fails as it should.
3. The native replay, with control 1, has run on the four named states: the two barred states and the two reported 4,096-step states. Its JSON is fetched with sha256.
4. The REPORT names the reading map's letter with the numbers, and the notebook row is written.
5. The PR goes through a fresh exact-head review, the queue and `close --verify`.
