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
- **The disagreement starts at the first read, in the copy heads, and covers nearly every position.** A copy head's codes depend only on the token's table point and the read's key rotation, never on the trunk (the item's step 5). Yet at depth 2, s1, only 172 of 3,072 copy-head query codes are bit-identical between the float D11 forward and the integer session (328 at s3).
- **The cause is not located.** This unit ruled out the leads and narrowed the rest (below). It did not find the stage where the two sides' copy codes part. So nothing is fixed, and the item's parity bar (done-when 2) stays unmet.
- **An instrument fault on the laptop, repaired before any number was read.** The first `read.sh` stopped at the check of claude-233's two artifacts: `test "$(wc -l < artifacts.sums)" = 2` fails on macOS, whose `wc` pads the count (`"       2"`). The same check passed on the pod (GNU `wc`). It became an arithmetic test, and the rerun read every input against its pins. The failed attempt is kept (`laptop/reading-attempt1/`).
- **A correction to my own first reading.** I first took "a mean of 2–3 bits differ" as ruling out any convention fault. That holds for a bit order or a complement, which would leave every kept set equal. It does not hold for a mirrored product (left against right). Near the identity, where the trained rotations start, the two differ only in small cross terms, which would also flip a few bits. So I compared the products term by term (below). They are the same.
- **One check could not tell anything apart.** The readout's fixed sequences never repeat a token: the ids step by 11, so 1,536 positions give 3,072 distinct (copy head, token) pairs. So "one code per token on each side" holds trivially, and says nothing about determinism.

## What changed

- `crates/uor-r4-integer/src/fbs_snapshot.rs`: the comparison row's `query_bits` (commit `3d4a880f0`); the trace record, the comparison and its rule are the unit's earlier commits.
- `crates/uor-r4-integer/src/stack/tests.rs`: the causes test pins `query_bits` on the planted tie and the planted one-bit flip. With the counting line set aside it fails ("no query differs"); restored, it passes.
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

**Diagnostic, not registered: how far apart the query codes are.** `query_bits` was added to the comparison after the run (commit `3d4a880f0`), and computed from the stored traces. Its counts match the registered comparison's exactly. The mean bit distance between the two sides' query codes:

| Read, heads | s1 mean (identical, max) | s3 mean (identical, max) |
|---|---|---|
| d2 copy | 3.21 bits (172 of 3,072, 12) | 2.19 bits (328, 8) |
| d2 type | 5.47 (9, 13) | 5.79 (9, 14) |
| d6 copy | 5.18 (10, 13) | 4.62 (15, 12) |

Codes that share nothing would sit near 32 bits apart. These sit a few bits apart, at nearly every position. The type heads' extra 2–3 bits fit the trunk's own small differences added on top.

### The instrument checks (the sheet's §4): all hold

- **`I-Data`:** every checkpoint file passed the run's sums, and all 12 of claude-233's files passed the sheet's pins, on the pod. On the laptop, the pod's 107 output files and claude-233's two artifacts passed again.
- **`I-GPU-Tests`:** `flock_read_hamming::` and `fbs_export::` passed on the pod at `e10bae9d6`, with both named tests ok.
- **`I-Repro`:** at both seeds, the d11 arm's `readout-d11.json` and its `compare.json` are claude-233's byte for byte (`6b3d9d7d…`, `ef617640…`; `bf34b819…`, `0d9484bf…`; `out/i-repro.txt`). So the capture and the switches are bit-identical when off, end to end.
- **`I-Trace-Shape`:** every comparison read 8 windows, at the depths 2 and 6, with 4 heads keeping 16, and refused nothing.
- **`I-Faults`:** 0 instrument faults in all 8 comparisons, and 0 ties in the tie arms. The plain arms count 0 ties too: their codes differ before any tie could.
- **`I-Finite`:** every tool exited 0, and the job ended with EXIT 0.

### What the trace rules out, and what is left

- **Ruled out:**
  - **the parameter grid:** each rounded arm's 12,288 query codes and 12,288 key codes are its unrounded twin's, at both seeds. Its readout's bytes differ (the rounded weights move the read's output), but its d does not;
  - **the tie order:** the tie arms move at most 7 sets per row, and no set reaches a tie;
  - **a bit-order, complement or head-order convention:** any of these would leave the kept sets equal;
  - **the product's form:** the CUDA `hamilton_gate_fwd` (`cuda_stack.cu`), the CPU control (`kernel_controls.rs`) and the integer `stack_hamilton` (`stack/kernels.rs`) compute the same left product `q ⊗ u`, term for term. Each normalizes `g/|g|`: the float adds 1e-12 under the root, the integer 0;
  - **activation storage:** the default CUDA build keeps activations in f32;
  - **the table's grid:** the D11 readout rounds the embedding to Q12 before the read (`round_tables_as_exported`, called in `fbs-snapshot --hamming-trace` before `fbs_float_readout`).
- **Left:** on paper both sides compute the same function of the token, yet their copy codes differ by 2–3 bits per word on about 90% of positions. By estimate, not measured: f32 rounding (about 1e-7 relative) flips a sign only for a rotated coordinate that close to zero, which on Q12 points should give well under one bit per word. So the inputs each side actually feeds the copy code differ (the table rows or the rotation units), or the arithmetic does in a way not yet seen.

### The sheet's decision: row F

Rows 0, T, Q and B do not hold, so row F applies: no lead alone closes the gap.

Its next step goes by the most common cause in d11-tie-rounded's judged differing sets. That cause is a query-and-key flip, at the copy heads as much as at the type heads, from the first read on. Copy-head codes depend only on the table and the read's rotations. So this is the sheet's second case: **"the rotation arithmetic, or a fault. The next unit tells the two apart."** Training with D11 in the loop (the first case) is not next: it would treat the trunk, and the trunk is not where the copy codes part.

**The next unit, a laptop host check, needs no GPU.** For each vocabulary entry and copy head, compute the code three ways:
1. the CPU float control (`hamilton_gate_fwd_cpu` and `pack_signs`) on the checkpoint's rounded key rotation and the Q12 table;
2. the integer `vocab_codes` on the artifact;
3. the GPU trace's code.

If 1 matches 2, the GPU read was handed other inputs. If 1 matches 3, the artifact's table or rotation is not the checkpoint's. Either way the stage is named, and the fix is in this item's scope.

### Cost and records

- **The pod:** one RTX 5090 (vast-58612576) at $0.563/h, up about 11 minutes (03:46–03:57Z, 1 Nov), so about $0.10. The job itself took 4 minutes. The lease was released, and the pod was deleted once its results were in the store. No pod is running.
- **The store:** `results/claude/integer-hamming-read-trace-2026-10-31/`:
  - `out/`, the pod's sealed output, whose `SHA256SUMS` is `b197c896…02c19`, read back 107 of 107 OK;
  - `bin/`, the two binaries, matching `out/binaries.sha256`;
  - `watch.sh`, `watch.log`, and a top `SHA256SUMS`;
  - `laptop/`, the laptop half (`reading/`, `reading-bits/`, the failed `reading-attempt1/` and `read.sh`), whose `SHA256SUMS` is `ea844ff1…7ac`.
- **The FIT** (#3385, 6552958496) and **the pre-run review**, GO (#3385, 6553001528).

## Checks run at the head

- **On the pod, at `e10bae9d6`:** the CUDA tests `flock_read_hamming::` and `fbs_export::` (the job's `tests.out`), then the eight arms.
- **On the laptop:**
  - `cargo test --release -p uor-r4-integer --features serving-wasm --lib`;
  - the default `--lib`;
  - `--bin fbs-toks`;
  - `cargo test --release -p uor-r4-float-train --lib`.
- **The negative gate for `query_bits`:** with its counting line set aside, `the_trace_comparison_classes_ties_flips_and_instrument_faults` fails ("no query differs"); restored, it passes.
- `cargo fmt -p uor-r4-integer -- --check`: clean.
- `uor-harness loop report --check` and `uor-harness loop pr-check --local`.
- `cargo run -p xtask -- roadmap render`: this unit's edit renders no change. The render's two other rows (the titles of `m1-close-saved-model` and `docs-status-refresh`) are the drift claude-234 recorded, and are left to their items.

## Skills used

None.

## Still open

- **The host check that names the stage** (the item's next step): every vocabulary entry's copy codes, three ways (the CPU float control on the checkpoint's rounded rotation and Q12 table; the integer `vocab_codes` on the artifact; the GPU trace). It needs no GPU. The fix follows in the item's scope, then the parity bar (done-when 2).
- **The timing at long contexts:** the sweep stops at the readout window, 192. The item's done-when 3 asks for tokens/s against context; a longer sweep, on mains power and an idle machine, is still owed.
- **The item stays `ready`;** done-when 2 and 3 remain open.

## Plain summary

The model's look-back read picks different earlier positions in the integer version than in the float one. This unit traced it on the GPU and on the laptop. Neither suspected cause, the order among ties or the rounding of the read's settings, explains it. The difference starts at the very first read, in the heads whose codes depend on the token alone. The two versions' codes are a few bits apart at almost every position, which should not happen when the same formula is applied to the same inputs. So the inputs differ somewhere, and a laptop check, token by token, will say where. On this laptop the read costs about 15% of decode speed: about 3,900 tokens per second with it, 4,700 without.
