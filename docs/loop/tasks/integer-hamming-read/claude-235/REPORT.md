# REPORT: integer-hamming-read, claude-235 (the Hamming read's trace, and its timing)

Lab: claude. Item: [integer-hamming-read](../../../../roadmap/items/integer-hamming-read.md). Order: [ORDER.md](ORDER.md). Sheet: [integer-hamming-read-trace-2026-10-31](../../../harness/prep/integer-hamming-read-trace-2026-10-31.md). References #3385.

## Tokens per second

`fbs-toks --context-sweep` on claude-233's h1 artifact (`28b01bba…`), on this laptop: an Apple M1, 8 cores, 16 GiB. It ran on battery power at a load average of 3.5–3.7 (`sweep-load.txt`), so read these as this machine's numbers under load, not its best. Each point decodes 64 tokens after its prefill; the figures are medians.

| Context (prefill + 64) | Decode, with the read | Decode, without | The read's cost | Prefill, with / without |
|---|---|---|---|---|
| 96 | 4,109 tok/s | 4,733 tok/s | 13% | 4,327 / 4,750 |
| 128 | 3,812 | 4,467 | 15% | 3,995 / 4,469 |
| 160 | 3,895 | 4,712 | 17% | 4,127 / 4,728 |
| 192 | 3,898 | 4,694 | 17% | 4,114 / 4,675 |

The read takes 13–17% of decode time at these lengths. 192 is the readout window, so the sweep says nothing about long contexts. This is the item's done-when 3 taken early (Casey, 31 Oct 2026: "Yes, fold it in"); it does not depend on parity.

## What did not work, first

- **Neither lead closes the gap, or moves it** (the sheet's row F). claude-233's d with the read, at s1 and s3:

  | Arm | s1 d | s3 d |
  |---|---|---|
  | d11 | 694 | 823 |
  | d11-tie | 696 | 816 |
  | d11-rounded | 694 | 823 |
  | d11-tie-rounded | 696 | 816 |

  The bar is d ≤ 46 of 1,536. The test (b) for a move is d(d11) − d(arm) > 2√(d(d11) + d(arm)): at s3, 7 against 81; at s1, the tie arm is 2 worse. So neither the tie order nor the parameter grid is a cause, even in part.
- **The disagreement is already there at the first read, in the copy heads as much as the type heads, and covers nearly every position.** By design (the item's step 5), a copy head's codes depend only on the token's table point and the read's key rotation, never on the trunk; this run could not test that (below). Yet at depth 2, s1, only 172 of 3,072 copy-head query codes are bit-identical between the float D11 forward and the integer session (328 at s3). At depth 2 the copy and type heads keep about the same share of equal sets (10.7% and 9.8% at s1).
- **The cause is not located.** This unit ruled out the leads and narrowed the rest (below). It did not find the stage where the two sides' copy codes part. So nothing is fixed, and the item's parity bar (done-when 2) stays unmet.
- **An instrument fault on the laptop, repaired before any number was read.** The first `read.sh` stopped at the check of claude-233's two artifacts: `test "$(wc -l < artifacts.sums)" = 2` fails on macOS, whose `wc` pads the count (`"       2"`). The same check passed on the pod (GNU `wc`). It became an arithmetic test, and the rerun read every input against its pins. The failed attempt is kept (`laptop/reading-attempt1/`).
- **A correction to my own first reading.** I first took "a mean of 2–3 bits differ" as ruling out any convention fault. That holds for a bit order or a complement, which would leave every kept set equal. It does not hold for a mirrored product (left against right). Near the identity, where the trained rotations start, the two differ only in small cross terms, which would also flip a few bits. So I compared the products term by term (below). They are the same.
- **One check could not tell anything apart.** The readout's fixed sequences never repeat a token: the ids step by 11, so 1,536 positions give 3,072 distinct (copy head, token) pairs. So "one code per token on each side" holds trivially, and says nothing about determinism, nor whether a copy code really is a function of the token alone.
- **The review found two gaps in my own instrument, both now closed** (round 1, below). The comparison never checked that a side's kept set is the nearest by that side's own recorded codes, so a float capture recording other codes than it chose with would have passed as sign flips (the L-007 pattern). And the rounded arms refused only the type spin, not every value the export refuses. Rerun with the new check, the eight comparisons are unchanged, with 0 faults.

## What changed

- `crates/uor-r4-integer/src/fbs_snapshot.rs`: the comparison row's `query_bits` (commit `3d4a880f0`); the trace record, the comparison and its rule are the unit's earlier commits.
- `crates/uor-r4-integer/src/stack/tests.rs`: the causes test pins `query_bits` on the planted tie and the planted query flip. With the counting line set aside it fails ("no query differs"); restored, it passes.
- **Review round 1** (commit `95aa2836b`):
  - **The own-codes check:** `kept_set` refuses a set that leaves out a candidate nearer the query than one it keeps, by that side's own codes, as an instrument fault. With it, a difference on the same codes is always a tie, so the old off-place branch is gone.
  - **The causes test:** it plants a query flip, key flips and a query-and-key flip, each with its sets re-chosen by the new codes, and a set its codes do not choose (the query complemented), which must be a fault.
  - **The rounding refusals:** `round_hamming_as_exported` refuses, before any value changes, a value that is not finite, one past `i32` at its step, and a margin that rounds to 0 or below, as the export and the integer read do.
  - **The rounding test:** it no longer checks against the code under test. Every value must be on its step and within half a step. Half-step values on every table must round away from 0. A value past 1 must stay where it is on a `2^-24` table, and round at a margin's `2^-16`. Each refusal must leave every value as it was.
  - **`fbs-snapshot --hamming-trace`:** it refuses an existing or shared output path before the GPU runs, and prints its usage on a wrong argument count.
  - **Docs:** the misplaced doc comment is back on `saved_hamming_reads_snapshot_as_they_load`, the session's trace switch says it is not for timing, and the rounding's doc says why every rounded value is an exact `f32`.
- **Review round 2** (the re-review approved; its two minor points are fixed here):
  - **One output, two spellings:** `--hamming-trace` compares its two outputs by where they would be written (the folder resolved, plus the name), so `./t.json` and `t.json` are refused as one file, not by their text. The bin's first test pins it.
  - **One read per depth:** `round_hamming_as_exported` refuses a read count other than one per depth, where it used to round the reads its depths reached and skip the rest silently.
- The unit's earlier commits: the float trace, the tie rule and the rounding arm; `fbs-snapshot --hamming-trace`; the integer trace, the comparison and the context sweep; the prep sheet, its FIT and its pre-run review.
- The item's step 5 records this result and the next check.

## Measurements

**The kept-set share** (equal ÷ judged, per read and head kind; d11 arm). 288 of each row's judged sets are positions t ≤ 17. There every earlier position is kept, so those sets match by construction. The second figure leaves them out.

| Read, heads | s1 | s3 |
|---|---|---|
| d2 copy | 329 / 3,072 (10.7%); 41 / 2,784 past t = 17 | 331 / 3,072 (10.8%); 43 / 2,784 |
| d2 type | 300 / 3,072 (9.8%); 12 / 2,784 | 298 / 3,072 (9.7%); 10 / 2,784 |
| d6 copy | 319 / 3,072 (10.4%); 31 / 2,784 | 314 / 3,072 (10.2%); 26 / 2,784 |
| d6 type | 288 / 288 judged; 2,784 downstream | 288 / 288 judged; 2,784 downstream |

The readout's type-head sets are judged only where the stream read agrees up to their position, which is only at t ≤ 17. The tie arms move these counts by at most 7 sets per row; the rounded arms not at all.

**Top-1** (the nearest kept member, by each side's own codes): s1 d2 copy 1,183 / 3,040 (38.9%), d2 type 800 (26.3%), d6 copy 877 (28.8%), d6 type 132 / 256; s3 1,385 (45.6%), 718 (23.6%), 918 (30.2%), 114 / 256.

**The causes** of the judged sets that differ (d11; d11-tie-rounded, the arm row F reads, differs by at most 7 per row):

| Read, heads | s1: key only / query and key | s3: key only / query and key |
|---|---|---|
| d2 copy | 150 / 2,593 | 285 / 2,456 |
| d2 type | 9 / 2,763 | 8 / 2,766 |
| d6 copy | 10 / 2,743 | 14 / 2,744 |

Query flips alone: 0 everywhere. Ties: 0. Faults: 0.

**Diagnostic, not registered: how far apart the query codes are.** `query_bits` was added to the comparison after the run (commit `3d4a880f0`), and computed from the stored traces. Its counts match the registered comparison's exactly. It counts every judged position, the empty sets at rows 0 and 1 included. The mean bit distance between the two sides' query codes:

| Read, heads | s1 mean (identical, max) | s3 mean (identical, max) |
|---|---|---|
| d2 copy | 3.22 bits (172 of 3,072, 12) | 2.19 bits (328, 8) |
| d2 type | 5.48 (9, 13) | 5.79 (9, 14) |
| d6 copy | 5.18 (10, 13) | 4.62 (15, 12) |

Codes that share nothing would sit near 32 bits apart. These sit a few bits apart, at nearly every position: 94.4% (s1) and 89.3% (s3) of d2's copy query codes differ, and 99.5–99.7% of d6's. The d6 copy codes sit about 2 bits farther apart than d2's, as far as d2's type codes, though no trunk enters a copy code by design. The two depths have their own key rotations, so the host check takes both depths.

### The instrument checks (the sheet's §4): all hold

- **`I-Data`:** every checkpoint file passed the run's sums, and all 12 of claude-233's files passed the sheet's pins, on the pod. On the laptop, the pod's 107 output files and claude-233's two artifacts passed again.
- **`I-GPU-Tests`:** `flock_read_hamming::` and `fbs_export::` passed on the pod at `e10bae9d6`, with both named tests ok.
- **`I-Repro`:** at both seeds, the d11 arm's `readout-d11.json` and its `compare.json` are claude-233's byte for byte (`6b3d9d7d…`, `ef617640…`; `bf34b819…`, `0d9484bf…`; `out/i-repro.txt`). So the capture and the switches are bit-identical when off, end to end.
- **`I-Trace-Shape`:** every comparison read 8 windows, at the depths 2 and 6, with 4 heads keeping 16, and refused nothing.
- **`I-Faults`:** 0 instrument faults in all 8 comparisons, and 0 ties in the tie arms. The plain arms count 0 ties too: their codes differ before any tie could.
- **`I-Finite`:** every tool exited 0, and the job ended with EXIT 0.
- **Own codes (added in review round 1):** on both sides, every kept set is the nearest by that side's own recorded codes. The eight comparisons were rerun from the sealed traces with the check that makes anything else an instrument fault, and every count is unchanged, with 0 faults (`reading-own/`, whose `SHA256SUMS` is `3c8e5177…92b2`). So each capture recorded the codes it chose with: the flips are in the codes, not in the record.
- **The tie switch took effect:** I-Repro makes the off path bit-identical, so the tie arms' moved sets (s3 d 823 → 816, s1 694 → 696) show the switch acted. No host test runs the read with it on (Still open).

### What the trace rules out, and what is left

- **Ruled out by measurement:**
  - **the parameter grid:** each rounded arm's 12,288 query codes and 12,288 key codes are its unrounded twin's, at both seeds. Its readout's bytes differ (the rounded weights move the read's output), but its d does not;
  - **the tie order:** the tie arms move at most 7 sets per row, and no set reaches a tie;
  - **a bit-order or complement convention:** either would leave the kept sets equal, and they differ;
  - **a capture that records other codes than it chose with:** the own-codes check finds none.
- **Ruled out by reading the code only** (the weakest kind, since the codes do differ):
  - **the product's form:** the CUDA `hamilton_gate_fwd` (`cuda_stack.cu`), the CPU control (`kernel_controls.rs`) and the integer `stack_hamilton` (`stack/kernels.rs`) compute the same left product `q ⊗ u`, term for term. Each normalizes `g/|g|`: the float adds 1e-12 under the root, the integer 0;
  - **activation storage:** the default CUDA build keeps activations in f32;
  - **the table's grid:** the D11 readout rounds the embedding to Q12 before the read (`round_tables_as_exported`, called in `fbs-snapshot --hamming-trace` before `fbs_float_readout`).
- **Left:** on paper (by design, not tested here) both sides compute the same function of the token. Yet 89–94% of d2's copy query codes differ, by 2.2–3.2 bits on average (d6's by 4.6–5.2). By estimate, not measured: f32 rounding (about 1e-7 relative) flips a sign only for a rotated coordinate that close to zero, which on Q12 points should give well under one bit per word. So the inputs each side actually feeds the copy code differ (the table rows or the rotation units), or the arithmetic does in a way not yet seen.

### The sheet's decision: row F

Rows 0, T, Q and B do not hold, so row F applies: no lead alone closes the gap.

Its next step goes by the most common cause in d11-tie-rounded's judged differing sets. That cause is a query-and-key flip, at the copy heads as much as at the type heads, from the first read on. Both of the sheet's triggers hold. The type heads' differing sets are almost all query-and-key flips (s1 d2 type 2,761, more than d2 copy's 2,587), which is the first case's trigger. Counted by kind, copy leads only because it has two judged rows (d2 and d6) to type's one.

I take the sheet's second case, as the lab's call: **"the rotation arithmetic, or a fault. The next unit tells the two apart."** By design (not tested here), copy-head codes depend only on the table and the read's rotations, so a copy-head flip cannot come from the trunk. Training with D11 in the loop (the first case) treats the trunk, so it cannot remove the copy-head flips. A fix to the copy codes' inputs may also shrink the type heads' share, since those heads read the same table and rotations.

**The next unit, a laptop host check, needs no GPU.** For each vocabulary entry and copy head, at both depths, compute the code three ways:
1. the CPU float control (`hamilton_gate_fwd_cpu` and `pack_signs`) on the checkpoint's rounded key rotation and the Q12 table;
2. the integer `vocab_codes` on the artifact;
3. the GPU trace's code.

If 1 matches 2, the GPU read was handed other inputs. If 1 matches 3, the artifact's table or rotation is not the checkpoint's. Either way the stage is named, and the fix is in this item's scope.

### Cost and records

- **The pod:** one RTX 5090 (vast-58612576) at $0.563/h, up about 11 minutes, so about $0.10. The job ran 03:50:21–03:54:20Z on 1 Nov (`job.log`, `gpu.csv`), and the watcher saw it finish at 03:54:43Z (`watch.log`). The pod's start time is not in the saved records. The lease was released, and the pod was deleted once its results were in the store. No pod is running.
- **The store:** `results/claude/integer-hamming-read-trace-2026-10-31/`:
  - `out/`, the pod's sealed output, whose `SHA256SUMS` is `b197c896…02c19`, read back 107 of 107 OK;
  - `bin/`, the two binaries, matching `out/binaries.sha256`;
  - `watch.sh`, `watch.log`, and a top `SHA256SUMS`;
  - `laptop/`, the laptop half (`reading/`, `reading-bits/`, the failed `reading-attempt1/` and `read.sh`), whose `SHA256SUMS` is `ea844ff1…7ac`;
  - `reading-own/`, the eight comparisons rerun with the own-codes check, whose `SHA256SUMS` is `3c8e5177…92b2`. `reading/read.log` took its last line ("done") after its folder's sums were taken. Without that line it matches them, and the rerun checks only the two integer traces it reads from there.
- **The FIT** (#3385, 6552958496) and **the pre-run review**, GO (#3385, 6553001528).

## Checks run at the head

- **On the pod, at `e10bae9d6`:** the CUDA tests `flock_read_hamming::` and `fbs_export::` (the job's `tests.out`), then the eight arms.
- **On the laptop, at `95aa2836b`** (the commit after it only formats this unit's lines and edits docs; the narrow tests passed again after it):
  - `cargo test --release -p uor-r4-integer --features serving-wasm --lib`: 397 passed, 1 ignored;
  - the default `--lib`: 56 passed;
  - `--bin fbs-toks`: 4 passed;
  - `cargo test --release -p uor-r4-float-train --lib`: 192 passed.
- **The negative gate for `query_bits`:** with its counting line set aside, `the_trace_comparison_classes_ties_flips_and_instrument_faults` fails ("no query differs"); restored, it passes.
- **Review round 1, at `95aa2836b`:**
  - **The own-codes gate:** with the round's `fbs_snapshot.rs` change set aside (`git diff HEAD~1`), the causes test fails at the planted set its codes do not choose (`causes(&unchosen)`); restored, it passes.
  - **The rounding gate:** with the round's `round_hamming_as_exported` change set aside, `rounding_the_hamming_reads_holds_the_exported_values` fails at the first refusal (`expect_err`); restored, it passes.
  - **The rerun:** the eight comparisons from the sealed traces with the new check, all identical to the registered ones, with 0 faults.
  - **`cargo clippy`:** `-p uor-r4-float-train --all-targets`, and `-p uor-r4-integer --features serving-wasm --lib --tests --bins`, with no error and no warning in the changed files.
- **Review round 2, at the final head:**
  - **The count gate:** with the new count check set aside, `rounding_the_hamming_reads_holds_the_exported_values` fails at the short set (`expect_err`, "a read short"); restored, it passes.
  - `cargo test --release -p uor-r4-float-train --bin fbs-snapshot`: 1 passed (`two_spellings_of_one_output_are_one_place`). No set-aside run was made for it: the test calls the helper directly, and a text comparison of `./trace.json` with `trace.json` is unequal.
  - `cargo clippy -p uor-r4-float-train --all-targets`: no new warning. The one in `fbs_export.rs` (`manual_is_multiple_of`, line 384) is claude-200's line, on main.
- `cargo fmt -p uor-r4-integer -- --check`: clean. The float crate is not rustfmt-clean on main. `fbs_export.rs` has the same 10 diffs as at the merge base with `origin/main` (`7a1cbcffc`), and this unit's lines in it are formatted; `bin/fbs-snapshot.rs` is clean.
- `uor-harness loop report --check` and `uor-harness loop pr-check --local`.
- **CI, `harness rules and tests`: red, and not from this branch.** It has been red on every branch that ran it since 31 Oct (`kimi/docs-current-status`, `claude/locate-gap`). On main's own files it fails:
  - `the_real_repo_passes_the_ported_rules`: three DeepSeek REPORTs document `pr-check --local ->`;
  - `the_real_repo_agrees_with_its_own_prompts`: "evaluation integrity" is missing from the milestone registry;
  - four `prgates` tests fail there too.

  This branch touches neither the harness crate nor those files. The job runs here only because the unit adds a prep sheet under `docs/harness/prep/`.
- `cargo run -p xtask -- roadmap render`: this unit's edit renders no change. The render's two other rows (the titles of `m1-close-saved-model` and `docs-status-refresh`) are the drift claude-234 recorded, and are left to their items.

## Skills used

None.

## Still open

- **The host check that names the stage** (the item's next step): every vocabulary entry's copy codes, three ways (the CPU float control on the checkpoint's rounded rotation and Q12 table; the integer `vocab_codes` on the artifact; the GPU trace). It needs no GPU. The fix follows in the item's scope, then the parity bar (done-when 2).
- **Two CUDA tests for the tie switch, owed to the next pod run** (review round 1, the float slice):
  - no test runs `read()` with the tie rule on. The run's moved sets show the switch acted, but a broken wiring would leave the tie arm silently equal to d11. A CUDA test that sets `set_hamming_tie_rule(true)` on planted ties and checks the traced order closes that.
  - `the_tie_rule_keeps_the_integer_sessions_order` has had no set-aside check. With its tie branch removed it may still pass, if the CUDA sort happens to keep ties in order on its 40 columns. Its codes carry no last-bit noise either, so `.round()` is not pinned; building them as `p + (bits − p)` would pin it.
- **The timing at long contexts:** the sweep stops at the readout window, 192. The item's done-when 3 asks for tokens/s against context; a longer sweep, on mains power and an idle machine, is still owed.
- **The red `harness rules and tests` job** is main's, in the harness and in other labs' files. Both are outside this unit's scope, so it is left to their owners.
- **The item stays `ready`;** done-when 2 and 3 remain open.

## Plain summary

The model's look-back read picks different earlier positions in the integer version than in the float one. This unit traced it on the GPU and on the laptop. Neither suspected cause, the order among ties or the rounding of the read's settings, explains it. The difference is already there at the very first read, in the heads whose codes, by design, depend on the token alone. The two versions' codes are a few bits apart at almost every position, which should not happen when the same formula is applied to the same inputs. So the inputs differ somewhere, and a laptop check, token by token, will say where. On this laptop the read costs about 15% of decode speed: about 3,900 tokens per second with it, 4,700 without.
