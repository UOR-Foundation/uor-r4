# REPORT: integer-hamming-read, claude-239 (the read's tie rule on by default)

Lab: claude. Item: [integer-hamming-read](../../../../roadmap/items/integer-hamming-read.md). Order: [ORDER.md](ORDER.md). References #3385.

## Tokens per second

No new reading. This unit changes the float read that the trainer and the float references run. The served integer artifact, `fbs-toks` and the integer session are untouched, so serving speed stays at claude-238's reading. On this M1 under load, decode runs at 3,068–4,085 tok/s with the read and 4,306–4,857 tok/s without, at contexts 96–192.

## What did not work, first

- **No parity number moved.** The default changes the float side only, and no trace or bar was re-run. Parity on claude-146's checkpoints and DEV (done-when 2) is still open, and so is the long-context timing (done-when 3).
- **The set-asides trip A's order check, not a check on sets alone.** In all three set-asides, A fails at row 6 of a window. That row has 5 candidates, all kept, and only their order differs: `[1, 4, 5, 2, 3]` against the rule's `[1, 4, 2, 5, 3]`. So the set-asides show that the read keeps the rule's order only with the default and its wiring in place.
  - The sets differ too, and parity counts sets. A's own off run shows it: 307 of its 512 rows keep a different set with the rule off. Every off set is just as near: its sorted bit distances equal the rule's, which implies that every such row ties across the 16th place. 332 rows do, counted on the rule-on codes, which equal the off run's.
  - No set-aside shows a set-only check failing by itself. The order check is the stronger one, since equal order implies equal sets.
- **`.round()` is still not pinned.** B's codes now carry last-bit noise (`1 − ε/2`, `1`, `1 + ε` times ±1), and the rule keeps its order through it. But removing only `.round()` would leave every key's order unchanged:
  - the noise moves a key `h · time + s` by far less than ½ at these sizes, while keys differ by at least 1;
  - so no ordering test can see the rounding at the windows the reads run (40 and 64 in these tests, 192 in training). It keeps each key an exact integer, and only matters once the noise moves a key by ½.
- **Checkpoints trained before this unit trained with the rule off.** From this unit on, a new training run keeps ties by (bits, `s`). claude-146's and claude-233's checkpoints were trained under the GPU sort's own order: claude-146's `choose_sorted` (`b349e65ca`) is one arg-sort with no tie rule, and its runs trained at `df1d7892`. So their float forward at the new default is not the forward they trained with. claude-238 measured the gap on (r-drop): the two readouts differ in top-1 at 174 and 227 of 1,536 positions.
  - `fbs-snapshot`'s `d11` arm turns the rule off, the order those checkpoints trained with. This unit does not show that `d11` reproduces their training forward exactly. The first form's D11 readout now follows the rule on any checkpoint with reads.
  - The parity reference for done-when 2 is the float with the rule on, as the item's step 5 chose.

## What changed

- `crates/uor-r4-float-train/src/flock_read_hamming.rs`:
  - **The default** (`97e7e7e5d`): `TIE` starts on, so every thread breaks the read's ties by ascending bit distance, then ascending `s`, unless `set_hamming_tie_rule(false)` turns it off. The read's codes are ±1 forward up to their last bit (`p + (bits − p).detach()`), and the selection is detached. So only which of the equally near candidates are kept, and their order, changes. No gradient flows through the choice itself, but a run's forward and its gradients change wherever a tie crosses the 16th place. `tie_keys_exact` still refuses a head whose keys would pass 2^24.
  - **The docs** of `set_hamming_tie_rule`, `choose_sorted` and the read's switch comment say the new default.
  - **The CPU test** is now `the_trace_starts_off_and_the_tie_rule_on`: on a new thread the trace is off and the rule on, and the rule turns off and back on.
  - **CUDA test A** (new), `a_read_breaks_its_ties_by_the_integer_sessions_rule_by_default`. It runs a real `read()`: 4 heads of 64, 2 windows of 64, 3 tokens repeating, so every key code is one of three. The trace is on, at the default.
    - Every traced kept set must equal, in order, the nearest 16 by (bits, `s`), computed on the host from the traced codes.
    - The same read with the rule off must give the same codes and keep sets just as near.
    - The test counts the rows that tie across the 16th place (more than 100 required) and the off sets that differ from the rule's (more than 0 required), so it cannot pass on a plant with no tie to break. It prints both counts (`0e0fb51ac`).
  - **CUDA test B** (`the_tie_rule_keeps_the_integer_sessions_order`): its planted codes carry last-bit noise.
  - The comment in `recording_the_trace_leaves_the_read_unchanged` says the rule starts on.
- `crates/uor-r4-float-train/src/bin/fbs-snapshot.rs`: the trace arms' docs. `d11` is now the explicit off arm, the GPU sort's own order (the first form's D11 readout before this unit). `d11-tie` is the first form's D11 readout since this unit. The arms' switches are unchanged.
- **The item's step 5** records this result and the next step. **The notebook row** is in `m4-serving.md`.

## Measurements

**The pod job** ran on vast-58634303 (`out/negatives.txt`, `out/counts.txt`, `out/tests.out`). Each set-aside was applied, the crate rebuilt (`Compiling uor-r4-float-train` in each step's log), A and B run alone, and the source restored to the pinned head:

| Set-aside | Its lines | A | B |
|---|---|---|---|
| `default-off.patch` | `TIE` starts `false` again | FAILED (row 6, order) | ok |
| `wire-off.patch` | the read's `let tie = false;` | FAILED (row 6, order) | ok |
| `branch-out.patch` | `choose_sorted`'s tie branch removed | FAILED (row 6, order) | FAILED (row 8, order) |
| none (restored, rebuilt) | | ok | ok |

- **A's counts at the restored head:** 332 of 512 rows tie across the 16th place. With the rule off, 307 kept sets differ from the rule's as sets, and every off set is as near.
- **The read's and the exporter's tests** (`flock_read_hamming::`, `fbs_export::`, run with `--features cuda --include-ignored`) at the restored head: 44 passed, 0 failed. That is claude-238's 43 plus A. A, B, T1, T2, `the_copy_code_snap_is_the_gpu_hooks_own` and `recording_the_trace_leaves_the_read_unchanged` each ran and passed.
- **The CPU negative, on the laptop:**
  - with `default-off.patch` applied and the crate rebuilt, `the_trace_starts_off_and_the_tie_rule_on` FAILED at its first assert;
  - restored and rebuilt, the module's 10 CPU tests passed.
- **Cost:** the job ran from 08:02:33Z to 08:03:40Z (`out/job.log`). The pod's time and rate are from the terminal, not the stored evidence: one RTX 5090, bootstrap ready at 08:01:50Z, deleted about 08:06Z, at $0.563/h (the cheapest listed 5090 offer). That is about 6 minutes and about $0.06.
- **Stored** at `caseyallard/uor-r4-store:results/claude/integer-hamming-read-tie-default-2026-11-01/out/`, 20 files. Its `SHA256SUMS` is `fef6667d589518835883bd69cfcffcb129a094faa6942201d5bfb973e22566a2`; the laptop tar is `2bb9244d…`.

## Checks run at the head

The code last changed at `0e0fb51ac`; the commits after it are docs.

- **On the pod, at `0e0fb51ac`:** the three set-asides, A alone with its counts, then the 44 `flock_read_hamming::` and `fbs_export::` tests with `--features cuda` (`out/job.log`).
- **On the laptop, at `0e0fb51ac`:**
  - `cargo test --locked -p uor-r4-float-train --no-fail-fast`: 216 passed, 0 failed (194 in the library);
  - `uor-harness loop gates --quick`, then `--full`: 11 gates, all GREEN (`local/claude/gates/1793520242-55928`);
  - the CUDA code type-checks with the stub nvcc (`cargo check --features cuda --all-targets`): 0 errors;
  - `cargo clippy -p uor-r4-float-train --lib --tests`, with and without `cuda`: no warning in either changed file;
  - `rustfmt --check` on `flock_read_hamming.rs`: 20 diffs, the same count as main's copy, none in this unit's lines. `fbs-snapshot.rs` has none.
- `uor-harness loop report --check` and `uor-harness loop pr-check --local`.

## Skills used

None.

## Still open

- **Parity on claude-146's checkpoints and on DEV,** done-when 2's named reference, with the default on. It needs a prep sheet, its FIT and a pre-run review.
- **The timing at long contexts** (done-when 3): claude-238's sweep stops at 192.
- **`.round()` has no test** that fails without it, for the reason above.
- **A's `broken > 0` leans on candle's CUDA sort not being stable.** A stable sort upstream would keep equal keys in ascending `s`, as the rule does, and A would then fail with no defect in the read.
- **Carried from claude-238:**
  - the guard's restore on a panic;
  - the read's unguarded backward;
  - T1's export leg, which has no negative of its own.
- **The item stays `ready`;** done-when 2 and 3 remain open.

## Plain summary

The model's look-back read keeps the 16 nearest earlier positions. When several positions are equally near, something has to decide which ones make the cut. The integer version always took the earliest. The float version used to let the GPU's sort decide, in an order nobody fixed. On one checkpoint, claude-238 found that this alone moved 11–15% of the float model's next-word choices.

By default, wherever the float version runs, in training and in the references, it now uses the integer version's rule. Two GPU tests check it, and each fails when its piece of the change is taken out. No accuracy number changes yet. The next step re-measures parity on the named checkpoints and DEV with the rule in place.
