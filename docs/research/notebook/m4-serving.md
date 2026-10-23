# Notebook: integer serving (M4)

This notebook records finite serving controls and served-model results under D11: integer arithmetic and lookup tables, with no floating-point arithmetic or multiply instruction in the declared kernel. No trained-model quality result is established, and the language base (M1) is not yet met.

## Results

Sources: [`archive/catalog.jsonl`](../../../archive/catalog.jsonl). Verdict words are defined on the [format page](./README.md#the-five-verdict-words).

| Date | What ran | Number | Bar | Verdict | Artifact |
|---|---|---|---|---|---|
| 2026-10-23 | A toy `uor-r4.lut-discrete/1` through `uor-chat --discrete`, twice in fresh processes against direct prefix evaluation; existing native and WebAssembly instruction audits | 3/3 CLI tests; 8/8 generated ids matched in each process; 59 + 44 + 55 native ranges and 7/7 WASM roots with zero forbidden instructions; trained quality and export gap NOT_RUN | kimi-31 ORDER fixed exact-id/refusal controls and zero forbidden instructions in the named audit roots | positive (finite CLI and existing audit scopes only; no trained quality or discrete-root audit) | [host record](../../labs/m4-export-uorlut01-cli/README.md) |

## Earlier records

The audit catalog holds 42 records for this line. Between 12 October and 19 October 2026 it has one record, and that record is tooling, not a result:

- 2026-10-18, [wasm-serving-runtime](../../labs/wasm-serving-runtime-2026-10-18/README.md): an outside contributor's browser and Node WebAssembly integer serving runtime was merged after hardening. Its multiply-free audit passes, but the SIMD kernels are not integrated and the demo cannot be run from a clean clone. This is a build record, so it has no row.

Serving results before 12 October (for example the d11-serving record of 10 October) predate this notebook. Find them in [`archive/CATALOG.md`](../../../archive/CATALOG.md) under line `m4-serving`, and read the record before relying on its verdict.

## What a quality row will need

A quality row needs a number measured through the exported integer artifact, a bar fixed beforehand, and the export gap: the difference between the offline float candidate and the integer artifact on the same evaluator (`CHARTER.md`, section 3.7). Serving must be integer on the user's device (section 3.6). The M1 closing bar also asks for tokens per second to be reported with the integer top-1 result (`CHARTER.md`, section 4.8).

## Go deeper

- [Serving and export](../../model/serving-and-export.md) and the [lookup-serving mechanism](../../mechanisms/lut-serving.md).
- [Format and verdict words](./README.md).
- [`CHARTER.md`](../../../CHARTER.md), sections 3.6, 3.7 and 4.8.
