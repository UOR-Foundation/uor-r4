# ORDER: integer-hamming-read, claude-188 (the artifact carries the read)

Lab: claude. Item: [integer-hamming-read](../../../../roadmap/items/integer-hamming-read.md). Base: `a368b78ef`.

## Goal

This is step 1 of the design's order of units (the item's "What is missing", claude-187): **the artifact.** The `UORLUT01` stack artifact carries claude-146's Hamming read, and a read-free export is unchanged.

**What this unit adds:**
- **`FbsTrunkSnapshot`** gains an optional `hamming`: the heads, the depths, and per read the query, key and value rotations, the margins and the weight rows, as `hamming_read.safetensors` stores them.
- **`uor-r4.lut-stack/6`.** The header gains an optional `hamming` record, beside claude-181's optional `transport_snap`. It holds the heads, depths, kept count and the three fixed-point exponents. A header with it is `/6`, and a header without it is chosen as before.
  - `StackShape` does not change, so its 25 struct literals, `tests/memory_shape.rs` among them, are untouched.
- **The sections:** `hamming.d{depth}.{query_rotation, key_rotation, value_rotation, margin, weight}`, each `i32`.
  - **Rotations** are stored raw, at `2^-24`. Serving normalizes them by the raw gates' exact path.
  - **Margins** are at `2^-16`.
  - **Weights** are at `2^-24`.
- **A parser** (`stack/hamming.rs`) that reads them back, checking every length.
- **The session refuses a `/6` artifact** until a unit serves the read, so a read is never silently dropped.

## Cancel probe (run first)

Run at `a368b78ef`:
- **The item's scope** holds `stack/format.rs`, `fbs_snapshot.rs`, `stack/synthetic.rs` and `stack/tests.rs` (claude-187).
- **No `lut-stack/6` exists.**
- **No open PR touches** these files.

**Go. Class: mechanism.**

## Files in scope

- `crates/uor-r4-integer/src/fbs_snapshot.rs`
- `crates/uor-r4-integer/src/stack/format.rs`
- `crates/uor-r4-integer/src/stack/hamming.rs` (new)
- `crates/uor-r4-integer/src/stack/mod.rs`
- `crates/uor-r4-integer/src/stack/session.rs`
- `crates/uor-r4-integer/src/stack/synthetic.rs`
- `crates/uor-r4-integer/src/stack/tests.rs`
- `crates/uor-r4-float-train/src/fbs_export.rs` (`hamming: None` on its snapshot)
- `docs/loop/tasks/integer-hamming-read/**`
- `docs/research/notebook/m4-serving.md`

## Done when

1. **Read-free exports are byte-identical.** The snapped and both-raw exports of the synthetic trunk have the sha256s pinned at the base, in a test.
2. **A `/6` export round-trips.** It carries every parameter, at its declared exponent, including a zero quaternion. It is refused when:
   - a length is wrong;
   - a depth lies beyond the layers or is out of order;
   - the head count is odd, or does not divide the width into 4-coordinate blocks of at most 64;
   - a value does not fit.
3. **The session refuses a `/6` artifact,** with a clear error.
4. **An old `snapshot.json`,** with no `hamming` key, still parses, and a read-free snapshot serializes with no `hamming` key.
5. **The checks pass:**
   - `cargo test -p uor-r4-integer --features serving-wasm` shows no failure beyond `main`'s missing-input set;
   - `cargo test -p uor-r4-float-train` passes;
   - `audit_zero_matmul_serving.py --stack` is FULL PASS;
   - fmt is clean.
6. **A REPORT and one notebook row.**

## Stop and ask if

Each of these is a stop-and-decide, never a wait:
- **a read-free export's bytes change:** a defect; fix it before anything else.
