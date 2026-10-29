# ORDER: integer-hamming-read, claude-187 (the artifact carries the read; the copy heads, served)

Lab: claude. Item: [integer-hamming-read](../../../../roadmap/items/integer-hamming-read.md). Base: `98a2f3b52`.

## Goal

The item's done-when 1 asks the integer runtime to run the whole of claude-146's Hamming read under D11's rules.

**This unit is the first slice, on the host only:**
1. **The `UORLUT01` stack artifact can carry the read:** both reads (after layer 2, and on the readout), each with its heads' key, query and value rotations, margins and weights.
   - It carries them under a new schema, `uor-r4.lut-stack/6`.
   - An artifact without the read stays `/4` or `/5`, byte for byte.
2. **The integer session serves the copy heads of both reads:**
   - the keys and queries from the table's points through the key rotation, as sign codes packed in `u64`, built once per vocabulary entry;
   - the match by `popcount(q XOR k)`;
   - the 16 nearest earlier positions `1 ≤ s < t`, with the float read's tie rule;
   - the weights `max(0, m − d)/m`;
   - the payload, the table point of `x_s`;
   - the stream combine after layer 2, and the readout move `n + w_c ⊙ (c − n)`.

   **The type heads are carried but not served yet.** The session refuses an artifact whose type-head weights are not all zero, so a copy-only serve cannot silently differ from float.
3. **Finite host controls:**
   - a synthetic FBS trunk with a synthetic read, against an f64 reference of the same quantized arithmetic;
   - a recall control, where a repeated token is found at distance 0.

**Later units:**
- the type heads;
- the exporter reading `hamming_read.safetensors`;
- the float-side parity on claude-146's checkpoints. That is GPU work: the trunk runs only on CUDA, and needs a prep sheet.
- speed.

## Cancel probe (run first)

Run at `98a2f3b52`:
- **The integer runtime has no Hamming read:** no `stack/hamming.rs`, and no Hamming term in `stack/`.
- **The float read is on `main`:** `crates/uor-r4-float-train/src/flock_read_hamming.rs`, 1,115 lines.
- **Its checkpoints are stored:** `results/claude/hamming-read-2026-10-27/lang-6d8bac25/hamming-s{1,2}/models/step-*/`, with `model.safetensors` and `hamming_read.safetensors` at each step.
- **No open PR touches the item's scope.**

**Go. Class: mechanism.**

## Files in scope

- `crates/uor-r4-integer/src/stack/hamming.rs` (new)
- `crates/uor-r4-integer/src/stack/mod.rs`
- `crates/uor-r4-integer/src/stack/session.rs`
- `crates/uor-r4-integer/src/stack/format.rs`, if the read's section needs the container
- `crates/uor-r4-integer/src/stack/tests.rs`
- `crates/uor-r4-integer/src/stack/synthetic.rs`
- `docs/loop/tasks/integer-hamming-read/**`
- `docs/research/notebook/m4-serving.md`

## Done when

1. **The artifact carries the read** (`lut-stack/6`) and round-trips it. `/4` and `/5` artifacts are byte-identical to `main`'s exports, and load as before.
2. **The session serves both reads' copy heads,** and refuses non-zero type-head weights.
3. **On the synthetic control,** the integer read's kept sets and decoded ids match an f64 reference of the same quantized arithmetic. The bar is fixed in this ORDER before the test runs: every kept set equal, and every decoded id equal. **On the recall control,** every repeated token's earlier copy is kept at distance 0.
4. **`audit_zero_matmul_serving.py --stack`** is FULL PASS. The `uor-r4-integer` tests show no failure beyond `main`'s missing-input set.
5. **A REPORT and one notebook row.**

## Stop and ask if

Each of these is a stop-and-decide, never a wait:
- **the float read's semantics are ambiguous** (the tie rule, or the mean over heads): take the float code's behaviour and quote it;
- **the read needs a float or transcendental instruction in the served path:** it cannot ship in that form, so find the integer form.
