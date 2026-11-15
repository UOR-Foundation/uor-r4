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

**3. A native replay in the probe** (`recall-payload-probe --native-replay`). It reads the float reference read's inputs at each query row: the stream at the read, the token ids, the normalized query and key gates, the weight and the token table. It converts each tensor to Q31 with one declared power-of-two scale per tensor, then runs `q31_first_pick_read` on the same windows. It reports:
- how often the native pick matches the float first pick;
- how often the native pick's next token is the asked value (native first-right);
- the scales used.

This conversion is a diagnostic in reference code (D85). It is not a model export, and nothing it makes is served.

## Native-space five items (SKILL)

- **State schema.** The operator's learned fields are the query gates and key gates (`width / 4` Q31 quaternions each), the per-lane Q31 weights (`width` words) and one gain shift. Its inputs are the Q31 stream words, the token ids and the Q31 token table.
- **Shared operator path.** There is one Rust source definition (`read.rs`) and one CUDA twin, and a bit-identical differential test covers both. A WASM build of the crate is a plain `--target wasm32` check, because the crate is dependency-free.
- **Replay oracle.** Raw words are compared, never rounded values: the picks (`u32`) and the out words (`i64`), CPU against CUDA, on fixed and seeded fixtures.
- **Forward, gradient, update.** Forward only. This unit has no gradient or update for the read's fields; the native update rule is a later unit. So by the SKILL this is an **operator unit, not a training run**.
- **Serialization.** The fields are raw little-endian words with a version byte, like `Q31HamiltonModel::to_bytes`, and a round trip is tested. Attaching them to the lut-stack artifact is the session unit's.

## Sheet (§4.5: one GPU evaluation, the native replay on claude-386's saved states)

- **Bar.** On the 1,024 held-out query rows of each 16,384-step state (`try-5`, `try-6`), both of these must hold:
  - the native pick equals the float first pick on at least 99% of rows (at least 1,014 rows);
  - native first-right is within 1% of the float first-right count, which was 1,000 and 990 in claude-388 (so at least 990 and at least 980).
- **Noise.** None. Picks are deterministic, so a miss is rounding at near-ties or a bug. The REPORT classifies each mismatching row by its float score gap.
- **Controls:**
  - the CPU operator against the CUDA twin, bit for bit, on the same converted inputs;
  - a 1-LSB perturbation of one key gate must change at least one pick or out word in a fixture where it should, which proves the differential can fail;
  - the read cut (no read) leaves the stream words unchanged.
- **Seeds.** 2 (claude-386's B2 seeds 1 and 2).
- **Cost.** Minutes on wick, $0. No pod and no training.
- **Reading map** (fixed before any number is read):
  - **P:** both conditions hold at both seeds. The Q31 operator reproduces the float read's choice, and the next unit attaches it to `IntegerStackSession`.
  - **N1 (precision):** picks disagree on more than 1% of rows, and the mismatches sit at float score gaps below the stream scale's LSB times 16. The fix is a wider address accumulator or a finer scale. It is not a mechanism negative.
  - **N2 (definition):** mismatches at large score gaps. The native definition differs from the float one (rotation, fold or tie rule), and the next step is to find the difference.
  - **D:** anything else, reported as read.
- **FIT:** a fresh-context non-author review, before the run.

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
3. The native replay has run on the four saved states, and its JSON is fetched with sha256.
4. The REPORT names the reading map's letter with the numbers, and the notebook row is written.
5. The PR goes through a fresh exact-head review, the queue and `close --verify`.
