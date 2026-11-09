# REPORT: one-model-end-to-end, deepseek-264

Lab: deepseek. Base: `402b4091e`. Order: [ORDER.md](ORDER.md). This unit audits real
claude-339 artifacts. It trains nothing, changes no serving arithmetic, moves no bar and
uses no GPU.

## What did not work

The early and final seed-1 artifacts do not yet show the carry selecting the asked value
at the first reply position. At step 16,384 the carried artifact has **0 asked-value
first lines** and **0 complete rows**; the exact `−C` also has 0 asked-value first lines.
The carry changes ranks and continuations, but not in a way that produces the registered
recall behavior so far.

Seed 2's sealed checkpoint, exported independently after its seal, is slightly different
but does not change that diagnosis. Both carried and `−C` have **2 asked-value first
lines**, 0 complete rows and 0 EOS stops. The carry changes 39 generation rows, 172
first-target ranks and 207 paired-first ranks, but neither arm reaches the value reliably.

This is a diagnostic over the published seed-1 points and an independent seed-2 export.
The two-seed outcome remains Claude's registered read; this unit does not substitute for
it.

## What changed

Added `real-artifact-audit` to the one-model instrument. It:

- parses a carried artifact and its supplied `−C`;
- requires identical shapes, depths and ask tables, with the carry only at the readout;
- checks optional published `SHA256SUMS` values for both artifacts;
- reproduces `−C` with `drop_hamming_carry_uorlut01` and requires byte equality;
- binds every panel and rank report to its artifact and expected engine;
- requires the explicit row count and `totals.rows`, then the same settings, rendering and
  non-artifact inputs across all reports;
- compares CPU and integer panel reports after removing only timing fields and engine identity;
- finds generation divergence by case id and token index;
- compares teacher-forced and paired-first reports by case id using the fields the panel
  actually serializes, after checking both rank inventories against the CPU generation
  report;
- reports carry-gain distribution and first-target rank changes.

The command is reusable on the remaining 16,384 cells without code changes.

## Measurements

All three seed-1 L-aC artifact pairs were independently downloaded from the store and
their published `SHA256SUMS` files verified. The final carried artifact at step 16,384 is
`ba221c27…`; its reproduced `−C` is `6e85e4e2…`.

| Step | Generation rows changed by cut | First-target rows changed | Paired-first rows changed | Asked-better carried / `−C` | First-target top-1 carried / `−C` |
|---|---:|---:|---:|---:|---:|
| 8,192 | 157 / 256 | 223 / 256 | 254 / 256 | 126 / 256 vs 125 / 256 | 0 / 256 vs 0 / 256 |
| 12,288 | 53 / 256 | 0 / 256 | 0 / 256 | 127 / 256 vs 127 / 256 | 0 / 256 vs 0 / 256 |
| 16,384 | 87 / 256 | 136 / 256 | 156 / 256 | 128 / 256 vs 127 / 256 | 0 / 256 vs 0 / 256 |

Seed 2 at 16,384 was exported independently from `runs-3/L-aC/s2`'s sealed
`step-016384` snapshot, using the requested raw gates and transition and the sealed run
config. The carried artifact hashes to `852c4721…`, and the exact cut hashes to
`e3fd234c…`. The audit changes 39 / 256 generation rows, 172 / 256 first-target ranks and
207 / 256 paired-first ranks. Asked-better is 128 / 256 on both arms. The first-line count
is 2 asked / 3 other / 0 both for carried and the same for `−C`; both are 0 complete and
0 EOS at 256 cap stops.

At every published seed-1 point:

- CPU generation equals the integer `serve_step` panel report on every non-timing field.
- Every non-carry artifact section and ask table is preserved.
- The supplied `−C` equals an independent reproduction byte for byte.
- The readout carry has 32 rows and 4,096 gains at depth 6.

The cut changes later teacher-forced ranks, but the first-position ranking remains
effectively tied. At 12,288, for example, 165 teacher-forced rows change in the full
serialized rank data while no first-target rank and no paired-first rank changes. At
16,384 the carry moves the paired comparison by one row while both artifacts remain at 0
asked-value first lines and 0 complete replies.

## Checks run at the head

- `real-artifact-audit` tests: **6 passed**, including same-engine rejection,
  malformed-engine rejection, wrong-artifact rejection, partial-inventory rejection,
  cross-arm metadata rejection and missing-rank-section rejection.
- Existing `served-reply-panel` suite with `cpu-parity`: **29 passed**.
- Real artifact audits after the review repairs: **3 / 3 passed** at steps 8,192, 12,288
  and 16,384, with both published hashes supplied and verified.
- Seed 2's independently exported final pair: **1 / 1 passed**, with report identity and
  panel metadata checked. Its hashes are diagnostic until Claude publishes the official
  artifact.
- CPU panel versus stored integer report: **6 / 6 comparisons passed** across carried and
  `−C`.
- Published store `SHA256SUMS`: **3 / 3 pairs verified**.
- Strict Clippy for both binaries and changed-file rustfmt: passed.
- `uor-harness loop gates --full`: **11 / 11 green**.
- `uor-harness check`: **362 rules, 0 errors, 327 warnings**. The warnings are
  repository-wide claim, stale-branch, preparation and milestone-record findings; this
  unit adds no error.

## Skills used

No Codex skill was loaded. The audit used the project's Rust crates, stored artifacts,
panel inputs, pinned tokenizer and existing panel scorer.

## Still open

1. Reconcile seed 2's independent export with Claude's official published artifact when it
   arrives; audit both L-a guards once their reports are available.
2. Apply the registered two-seed outcome map in claude-339; this unit supplies diagnostics.
3. Explain why the carry changes later ranks but not the first asked-value choice.
4. Re-run the same receipt against a direct `uor-chat` loopback transcript if the final
   product-path probe needs a second wrapper beyond `IntegerSession::serve_step`.

## Plain summary

The serving and control chain is clean on real artifacts: CPU and integer production
paths agree, and the carry cut is byte-exact. The geometric carry is doing something to
later ranks, but it is not yet carrying the requested value into the first reply token.
The remaining task is therefore model/query diagnosis, not export or serving repair.
