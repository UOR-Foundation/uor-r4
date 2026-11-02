# REPORT: public M1 discrete writer parity, deepseek-2

Lab: deepseek. Item: [cpu-discrete-writer-core](../../../../roadmap/items/cpu-discrete-writer-core.md) (#3502). Base: `707e5025d`.

## What did not work

- **The first private-store premise was unavailable, not false.** The private artifact cannot be read without credentials. The public M1 release supplies the same seed-1 artifact bytes, so this unit binds that public copy and does not claim private-store access.
- **The apparent `summarize` double-count was not a source defect.** An overlapping inspection range displayed one line twice; the committed function increments once. No code change was made for that false alarm.
- **The GPU comparison is aggregate.** Every row is checked against `IntegerDiscreteModel`; the published GPU observation supplies the full-panel admitted, top-1 and top-8 counts, not a public row dump.
- **This is not training or language evidence.** It is an exact writer component parity result on a trained artifact and public DEV panel.

## What changed

- Added `cpu-public-discrete-parity`.
- Bound the public M1 seed-1 artifact, driver/model tokenizers, DEV manifest, token stream and mask by SHA-256.
- Added exact `discrete.table` extraction from the 19-table artifact without editing serving source.
- Constructed the independent `CpuWriterCore` from the trained 4,096 × 32 codebook.
- Checked every eligible DEV row against `IntegerDiscreteModel` for query vector, admitted-cell order, top-8 and prediction.
- Compared CPU aggregate target admission, top-1 and top-8 with the published M1 seed-1 GPU observation.
- Recorded rows/s, elapsed time, stable checksum and peak RSS.

## Measurements

Public inputs:

- artifact `04652a3de8e966a23539b04fbd77c7d9872d9f13a541fcb224d48e766e2c147d`, 86,536,132 B;
- DEV tokens `0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b`, 42,856 tokens;
- DEV mask `a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078`, 14,287 eligible targets;
- driver tokenizer `d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89`;
- model tokenizer `a9fb13f70c69ffe5afed40c8251a8e702adf0b00732d35ccff9457a3309ee377`;
- DEV manifest `ab269c9818a713ef41f9aff0fdd8995386d5eb0ebaf8a98636f7766e99f83eab`.

Exact parity:

| Reading | Result |
|---|---:|
| Rows | 14,287 |
| Query-vector mismatches | 0 |
| Admitted-cell-order mismatches | 0 |
| Top-8 mismatches | 0 |
| Prediction mismatches | 0 |
| Target admitted | 9,012 |
| Top-1 | 7,002 |
| Top-8 | 7,485 |
| Stable checksum | `cc84145641b5d7a2` |

Release benchmark on the Ryzen 9 9950X: 952.864 ms, 14,993.740 rows/s, one thread, 772,988 KiB peak RSS.

## Checks run at the head

- `cargo run --locked --release -p uor-r4-cpu-train --bin cpu-public-discrete-parity -- <fixture-dir>`: PASS; readings above.
- `cargo test --release --locked -p uor-r4-cpu-train`: recorded at the final head.
- `cargo clippy --locked -p uor-r4-cpu-train --all-targets --no-deps -- -D warnings`: recorded at the final head.
- `cargo fmt --package uor-r4-cpu-train -- --check`: recorded at the final head.
- D23/D25/D32, local PR checks, full loop gates and exact-head review: recorded at the final head.

## Skills used

No repository skill was invoked. The public release card, committed M1 evidence and current CPU writer source were reconciled directly.

## Still open

- Port discrete Route/Flock/Relation reads, proposals, BVG-K, apply and resume in #3501.
- Build a pre-registered CPU training pilot after the remaining component gates.

## Plain summary

The independent CPU writer now reproduces the public M1 trained artifact's discrete readout exactly on all 14,287 DEV rows and matches the published GPU aggregate counts. The trained-artifact blocker for #3502 is closed; training remains the next separate unit.
