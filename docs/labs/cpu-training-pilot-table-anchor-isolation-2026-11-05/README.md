# CPU table-anchor isolation

## Question

Which part of deepseek-183's table-wide port anchor caused its decoding loss:
the recurrent input rows, the writer scoring rows, or their interaction?

## Identities

- Lab and item: DeepSeek, `cpu-training-pilot` (#3506).
- D84 substrate: CPU development research; GPU qualification remains required.
- Code base: `1c5209f8e`; implementation
  `1f02345e69abc4c6fcc7bf906b63f900df6ab025`; FIT and D67
  `a5e472c363f848be7ff4b5b462a6b5391478afeb`.
- Release driver:
  `target/deepseek/table-anchor-isolation/release/release/cpu-table-anchor-isolation`,
  SHA-256
  `ab39c515f15856245d5681dc2230f1fa184a959224739d0933dcf10a7ec58e84`.
- Raw artifacts: seed 1
  `a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd`,
  seed 2
  `da56fe96b14a4a0bf383996873f10f3b149f63ebcc22d14e3dfaa260a8db51d1`.
- Anchored artifacts: seed 1
  `88c6d1e3a40db3d23a72a0435060147306e98bf9b7ad46d7c42e4099e1ad5a03`,
  seed 2
  `275fcde2c67153d6ea0140da067e3151616d0bbd6ed3e40439e6a69618fb3930`.
- DEV panel: 58 conversations and 616 response rows; tokens
  `add1f395…1e442`, mask `da1ae9ee…3c01`, manifest `ce489b38…3949`.
- Host: AMD Ryzen 9 9950X, 16 physical cores / 32 threads, 16 Rayon workers.
- Sealed outputs:
  `/home/callard/projects/codex-playground/uor-r4/local/deepseek/table-anchor-isolation/2026-11-05/seed-{1,2}`.

The complete command, stop rule and pre-run review are in the
[pre-registration](../../harness/prep/cpu-training-pilot-table-anchor-isolation-2026-11-05.md).

## What ran

The driver replayed every registered response prefix four times from reset:
raw input with raw writer, anchored input with raw writer, raw input with
anchored writer, and both anchored. It reconstructed the anchored table from
the raw table and parsed SpiralCore roots, required exact equality with the
published anchored artifact, and reran every arm in-process.

Raw/raw matched the integer serving session exactly. The combined arm matched
a forward parsed from the anchored artifact exactly. Every query, token and
kept-cell decision was deterministic. Hamming and turn memory were refused.
The hybrid table pairs remain in-memory diagnostics because `UORLUT01` stores
one shared table.

Both complete runs took under 36 seconds. Seed 1 used 47,980 KiB peak RSS and
seed 2 used 47,872 KiB.

## Measured result

| Seed | Arm | Admitted | Top-1 | Top-8 | Port distance |
|---|---|---:|---:|---:|---:|
| 1 | raw/raw | 245 | 181 | 236 | 29,882,444 |
| 1 | input-only | 120 | 55 | 89 | 25,493,709 |
| 1 | writer-only | 245 | 195 | 233 | 29,882,444 |
| 1 | combined | 120 | 55 | 86 | 25,493,709 |
| 2 | raw/raw | 171 | 136 | 164 | 23,870,298 |
| 2 | input-only | 29 | 9 | 20 | 27,634,674 |
| 2 | writer-only | 171 | 137 | 162 | 23,870,298 |
| 2 | combined | 29 | 10 | 20 | 27,634,674 |

On both seeds, input-only reproduced combined admission exactly and all but
three top-8 rows on seed 1 and all top-8 rows on seed 2. Writer-only preserved
raw admission, kept cells and port distance exactly. Its top-1 changed by
`+14/-3` on seed 1 and `+1/-2` on seed 2. The only non-zero interaction was
`-14` top-1 on seed 1 and `+2` top-8 on seed 2.

On seed 2 the anchored input moved target queries farther from the roots:
port distance rose from 23,870,298 to 27,634,674. The root projection therefore
does not preserve the learned input code, and lower distance on seed 1 is not a
sufficient proxy for better decisions.

## Decision

**COMPLETE (instrument).** The observed decoding collapse is attributable to
the recurrent input table change, not direct writer-row mismatch. Writer-only
table anchoring is comparatively neutral on this panel.

This is not a trained-model result, an exportable representation or a GPU
qualification. It does not reject root-addressed writer scoring. It shows that
overwriting the shared recurrent input rows is the wrong way to add that
addressing.

## Limitation and next step

The panel was previously exposed, and the hybrid table pairs cannot be emitted
by the current one-table artifact. The next design must preserve learned token
input identity while representing writer address rows separately, or train
both roles with an explicit input-code preservation objective.
