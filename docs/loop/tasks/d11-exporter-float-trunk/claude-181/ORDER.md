# ORDER: d11-exporter-float-trunk, claude-181

Lab: claude. Item: [d11-exporter-float-trunk](../../../../roadmap/items/d11-exporter-float-trunk.md). Base: `e25be3bd5`.

## Goal

The item's done-when reads:

> A float-trunk snapshot exports to D11 with the audits green and the export gap measured inside the band. The trunk is a SpiralCore/FBS recurrence trunk with the Hamilton gate and the native vector writer head, as Stage 3 trains it. The synthesis (Stage 4) adds tokens per second, reported for decode and for context separately.

**This unit's step toward it** (owner direction 29 Oct, "fix the exporter"):
- claude-180 read Both off: no single change brings the D11 forward inside the export bar on every model. On six of eight models the Hamilton gate snap carries most of the snaps' gap, and one model needs the transition unsnapped too.
- **This unit builds the unsnapped D11 contract on the host:**
  - the artifact states whether its Hamilton gates, and separately its transition, are snapped to the 120 icosians;
  - an unsnapped gate is served as the Q16 integer quaternion the gathers already produce, multiplied by `stack_hamilton` (D80's multiply);
  - an unsnapped transition uses the runtime's existing `stack_rotation` (integer square root and D81's divide).
- Finite CPU parity controls hold the integer session against the f64 reference in each of the four combinations. Snapped artifacts serve bit-identically to before.
- No GPU here. The new artifact's export gap needs the GPU evaluator updated, and is the next unit.

## Cancel probe (run first)

- Does the runtime already serve an unsnapped FBS gate?
- Does an open PR touch the files below?

**Result (29 Oct): go. Class: mechanism (it is the contract change claude-180's reading names).**
- **No unsnapped FBS gate exists.** `validate` (`stack/format.rs:225`, `:233`) refuses any gate other than `"geometric"` and `"hamilton"`. `Container::parse` requires `transport_snap` for an FBS shape. The session snaps both gates (`session.rs:2835`, `:3339`). The transition's unsnapped branch (`stack_rotation`, `session.rs:3447`) exists but cannot be reached by an FBS artifact.
- **No open PR touches** `crates/uor-r4-integer/src/stack/`, `fbs_snapshot.rs`, `fbs_export.rs` or the GPU instrument (checked across every open PR's files).

## Files in scope

- `crates/uor-r4-integer/src/stack/format.rs`, `session.rs`, `tests.rs`: the shape vocabulary, the header, the reader, the step, the f64 reference, and the parity tests.
- `crates/uor-r4-integer/src/fbs_snapshot.rs` and `crates/uor-r4-integer/src/bin/fbs-export.rs`: the export choice.
- `crates/uor-r4-float-train/src/fbs_export.rs`: passing the choice through, if needed.
- `docs/loop/tasks/d11-exporter-float-trunk/claude-181/**`
- `docs/research/notebook/m4-serving.md`

## Done when

- **The format** states each snap choice, refuses an inconsistent header, and round-trips. Snapped artifacts parse as before.
- **The session** serves both unsnapped forms with integer arithmetic only (multiply, shift, divide, integer square root), and no float.
- **Parity:** the integer session equals the f64 reference, within the tolerance the existing snapped tests use, on the synthetic controls and a stored trained snapshot, in all four combinations.
- **No regression:** the existing snapped tests pass unchanged, and a snapped export of claude-126's seed-1 snapshot serves the same tokens as before.
- **Audits:** the zero-multiply/instruction audit of the `stack` module is green under D80 and D81.
- **Speed:** decode tokens/s on this laptop (`fbs-toks`) for claude-126's seed-1 snapshot, exported snapped and unsnapped.
- **Records:** a REPORT and one notebook row. The item is not closed by this unit.

## Stop and ask if

Each of these is a stop-and-decide, never a wait:
- a snapped artifact's output changes: that is a regression, so stop and fix it;
- an unsnapped product overflows its integer width: bound it and record the bound;
- the change needs the CUDA instrument or a GPU: that is the next unit.
