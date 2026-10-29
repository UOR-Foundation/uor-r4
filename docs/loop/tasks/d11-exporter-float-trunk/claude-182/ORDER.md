# ORDER: d11-exporter-float-trunk, claude-182

Lab: claude. Item: [d11-exporter-float-trunk](../../../../roadmap/items/d11-exporter-float-trunk.md). Base: `ce20de296`.

## Goal

The item's done-when reads:

> A float-trunk snapshot exports to D11 with the audits green and the export gap measured inside the band. The trunk is a SpiralCore/FBS recurrence trunk with the Hamilton gate and the native vector writer head, as Stage 3 trains it. The synthesis (Stage 4) adds tokens per second, reported for decode and for context separately.

**This unit's step toward it** (owner direction 29 Oct, "fix the exporter"):
- claude-181 built the unsnapped D11 contract on the host (`raw_gates`, `raw_transition`, `uor-r4.lut-stack/5`).
- The export gap is panel scoring of an exported artifact, so it runs on the GPU (CHARTER §3.5, §3.6). The GPU evaluator (`docs/labs/d11-exporter-float-trunk/instrument`) has the snapped forms only.
- **This unit:**
  1. gives the evaluator's core (`fbs_core.h`, shared by the CUDA kernel and the host twin) the unsnapped forms, with host-twin parity against the Rust session on this Mac;
  2. requalifies the evaluator bit-exact on the GPU;
  3. measures the export gap of the unsnapped artifacts on claude-180's eight models, under a FIT'd prep sheet and a D67 pre-run review.

## Cancel probe (run first)

- Does the evaluator serve an unsnapped artifact?
- Does an open PR touch the instrument?

**Result (29 Oct): go. Class: mechanism.**
- **The evaluator snaps every site.** `fbs_core.h` snaps at three call sites: the transition (`:489`), the output gate (`:505`) and the feed-forward gate (`:524`), all through `fbs_snap_rotation`. No `raw_gates` or `raw_transition` appears in the instrument.
- **No open PR touches** `docs/labs/d11-exporter-float-trunk/instrument/` (checked across every open PR's files).

## Files in scope

- `docs/labs/d11-exporter-float-trunk/instrument/**`: the core, the Rust model image, and the tests.
- `crates/uor-r4-float-train/**`, if the gap mode needs the unsnapped D11 panel.
- `docs/harness/prep/d11-exporter-float-trunk-unsnapped-gap-2026-10-29.md`
- `docs/loop/tasks/d11-exporter-float-trunk/claude-182/**`
- `docs/research/notebook/m4-serving.md`

## Done when

- **The core** serves both unsnapped forms, and the host twin equals the Rust session bit for bit in all four combinations, on the synthetic controls and over sequences.
- **On the GPU,** the evaluator equals the host twin and the session on the controls and on trained exports, in all four combinations.
- **The sheet** is FIT and its D67 review passed.
- **The run:** one RTX 5090 run gives each model's export gap (F against the artifact) for the unsnapped forms the sheet names, sealed and read back.
- **Records:** a REPORT with the gap verdict per the sheet, and one notebook row.

## Stop and ask if

Each of these is a stop-and-decide, never a wait:
- the host twin and the session differ: fix the core before any GPU run;
- the GPU evaluator differs from its host twin: that is an instrument fault, so repair it and requalify;
- the measurement needs a training run: that is not this unit.
