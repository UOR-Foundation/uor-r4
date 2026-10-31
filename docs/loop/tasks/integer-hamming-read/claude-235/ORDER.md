# ORDER: integer-hamming-read, claude-235

Lab: claude. Item: [integer-hamming-read](../../../../roadmap/items/integer-hamming-read.md) (#3385). Base: `7a1cbcffc`.

## Goal

Run step 5's trace, as the item records it (claude-234, #3466), and the timing Casey folded into it. Casey (31 Oct 2026, in chat): "Trace attention first"; "Yes, fold it in".

claude-233 (#3465) measured the gap on (r-drop) step 16,384. The trunk alone agrees with its float D11 forward at s1 and s3: 5 and 11 of 1,536 sequence positions differ. With the Hamming read it does not: 694 and 823 differ. This unit finds out why, per (position, head, read). Then it fixes the cause if the fix is in scope. Otherwise it records the cause for the next unit.

The item's done-when 2 (the parity bar on claude-146's checkpoints) and done-when 3 (the speed) stay the item's. This unit takes the trace and the timing toward them:
1. **The float capture.** In `flock_read_hamming.rs`, a thread-local trace, off by default, records per (window, read, head):
   - each position's query and key codes, as `pack_signs` packs them;
   - each position's kept positions (real candidates only).
2. **The two arms**, each behind an off-by-default switch:
   - the float tie rule set to the integer's, ascending bit distance then ascending `s`, by the exact key `h·T + s`;
   - the read's parameters rounded as the exporter stores them (rotations and weights at 2^-24, margins at 2^-16).
3. **`fbs-snapshot --hamming-trace`** writes the D11 readout and its trace for one arm, on the readout's fixed sequences (`fbs_export.rs`, `READOUT_SEQUENCES`).
4. **The integer side.** `fbs-toks --hamming-trace` writes the same record from the integer session, on the same windows. `fbs-toks --hamming-trace-compare` compares two traces and gives:
   - the kept-set share per (read, head kind);
   - the judging rule and the causes the item's step 5 sets (tie on the union of both kept sets; sign flip, split into query and key; instrument fault; readout type-head sets downstream of a stream disagreement).
5. **The timing.** `fbs-toks` decode tokens/s against context, with the read and without, on one artifact.
6. **The GPU run.** A prep sheet with a fresh FIT and the D67 pre-run review, then one pod. It covers four arms (d11, d11-tie, d11-rounded, d11-tie-rounded) at s1 and s3, and the export and compare as claude-233 ran them. The integer trace and the comparison run on the laptop, from the stored outputs.

## Cancel probe (run first)

The item's cancel condition (as merged in claude-234) reads: "Another unit already meets this item's done-when on main (the read served at parity with its float forward, and its speed measured), or the float read is withdrawn."
- **Parity on main?** No: claude-233's row R is the latest reading (694 and 823 differing positions).
- **Speed on main?** None measured: the item's step 6 is open, and `m4-serving.md` has no tokens/s row with the read.
- **The float read?** It is not withdrawn: (r-drop) trains with it.
- **No other claim or open PR on the item:** checked at the claim; #3467 is this unit's own draft.

Result: not cancelled. Class: none; nothing found stale.

## Files in scope

The item's scope, as merged:
- `crates/uor-r4-float-train/src/flock_read_hamming.rs`
- `crates/uor-r4-float-train/src/fbs_export.rs`
- `crates/uor-r4-float-train/src/bin/fbs-snapshot.rs`
- `crates/uor-r4-integer/src/fbs_snapshot.rs` (the shared trace record)
- `crates/uor-r4-integer/src/stack/hamming.rs`
- `crates/uor-r4-integer/src/stack/session.rs`
- `crates/uor-r4-integer/src/stack/mod.rs`
- `crates/uor-r4-integer/src/stack/tests.rs`
- `crates/uor-r4-integer/src/stack/synthetic.rs`
- `crates/uor-r4-integer/src/bin/fbs-toks.rs`
- `docs/harness/prep/integer-hamming-read*`
- `docs/loop/tasks/integer-hamming-read/**`
- `docs/research/notebook/m4-serving.md`
- `docs/roadmap/items/integer-hamming-read.md`

## Done when

1. Every switch is off by default, and a test pins that with them off the read's output is bit-identical to main's. The capture, the tie key and the rounding each have a host test. Each test fails with its code set aside.
2. The integer trace and the comparison have host tests on a synthetic artifact. They cover:
   - a planted tie at the 16th place, classed as a tie;
   - a planted code flip, classed as a sign flip;
   - a planted size mismatch, classed as an instrument fault.
3. The sheet `docs/harness/prep/integer-hamming-read-trace-2026-10-31.md` has a fresh FIT and the D67 pre-run review, both posted on #3385.
4. The pod ran, and the outputs are sealed in the store with a SHA256SUMS that was read back. The pod is down.
5. The REPORT gives, at s1 and s3, for each arm:
   - the kept-set share per (read, head kind);
   - the causes;
   - claude-233's differing-position count (`fbs-compare --parity-only`).

   It names which lead closes the gap, if either does, and gives tokens/s against context with the read and without. It has one notebook row in `m4-serving.md`.
6. A fresh exact-head review approves; the PR is merged and verified.

## Stop and ask if

- **A cause lies outside the scope** (for example in the trunk's own export): record it, and carry it to the next unit's ORDER.
- **The capture cannot be made bit-identical when off:** stop the GPU step, and fix it first.
- **The pod or the caps are short:** wait for a slot by `uor-pod status`. Never take another lab's.

(Stop and decide; never stop and wait.)
