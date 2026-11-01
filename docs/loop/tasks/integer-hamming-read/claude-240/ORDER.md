# ORDER: integer-hamming-read, claude-240

Lab: claude. Item: [integer-hamming-read](../../../../roadmap/items/integer-hamming-read.md). Base: `cef022733`.

## Goal

The item's done-when reads:

> 1. The integer runtime runs the whole read as claude-146's REPORT describes it, all four heads of both reads, under D11's rules (CHARTER §3.6: integer only, no float or transcendental instruction; since D80 and D81 the hardware integer multiply and divide are allowed once [`integer-mul-div-serving`](integer-mul-div-serving.md)'s audits allow them, and until then products go through D11's tables):
>    - **copy heads:** keys and queries from the table's points through the key rotation. These depend only on the token, so their codes can be built once per vocabulary entry;
>    - **type heads:** queries from the residual through the query rotation, and payloads from the residual at `s` through the value rotation. These rotate the state at run time, so each product goes through D11's tables;
>    - the codes packed into 64-bit words; the match by `popcount(q XOR k)`; the 16 nearest earlier positions `1 ≤ s < t`; the weights `max(0, m − d)/m`; the stream read after layer 2 and the move on the readout, each product through the tables.
> 2. **Parity,** against the float forward of the same stored checkpoint (claude-146's language runs, `caseyallard/uor-r4-store:results/claude/hamming-read-2026-10-27/`), on DEV. No D11 tolerance exists for this read yet, so the unit's prep sheet pre-registers the bar for two numbers, and the REPORT gives both:
>    - the share of (position, head, read) kept sets equal to float's. One sign flip in an integer rotation changes a code, so this is not assumed to be exact;
>    - top-1 agreement between the integer and float decoded ids.
>    
>    The float reference is computed on the GPU (CHARTER §3.5); the integer side runs on the laptop, as serving does.
> 3. **Speed on this laptop:** `fbs-toks` decode tokens/s against context, with the read and without, on the same artifact.
> 4. One notebook row; the result on main.

## This unit

claude-238 (#3478) put the float reference on the read as trained. With one tie rule on both sides, (r-drop) step 16,384 met both bars at both seeds. claude-239 (#3479, `cef022733`) made that tie rule the float read's default. This unit measures **done-when 2 as the item states it: claude-146's language checkpoints, on DEV.**

- **The checkpoints:** claude-146's merged form, `results/claude/hamming-read-2026-10-27/lang-df1d7892/hamming-s1` and `hamming-s2`, and `lang-s3-df1d7892/hamming-s3`, at step 16,384.
  - Seed 1's run config reads `objective: nearest`, `retain_rates: true`, `hamming_read: 4`, `fold_scores: false`, `reflect_floor: false` (fetched 1 Nov).
  - The older `lang-6d8bac25` form is out of scope.
- **DEV:** `w2-inputs/2026-10-25/driver-data/ft/dev/`.
  - It is loaded with its pins (`hamilton_sheet::load_pinned_uort_split`) and cut into the trainer's windows (`build_document_windows` at `SHEET_CONTEXT`, 192), as `in_space_long.rs` cuts it.
  - The sample is every 8th window, claude-126's `PARITY_WINDOW_STRIDE` rule (`spiralcore_fbs_sheet.rs`).
- **The code gap, in this scope:**
  - `fbs-snapshot`'s readouts and its `--hamming-trace` form read only `fbs_float_readout`'s fixed 8 × 192 sequences;
  - the trace refuses any count other than 8 windows.

  The change adds `--dev DATA_ROOT` to both forms. It reads the DEV sample through `fbs_float_readout_on`, and the trace checks the sample's own count. `fbs-compare` (not in scope) needs the plain, export and D11 readouts to hold the same sequences, so the snapshot form writes all three on the sample.
- **The arms:**
  - `d11-tie`, the float read's default since claude-239, is the parity arm;
  - `d11`, the rule off, is reported as the gap that the default closes;
  - the fixed sequences under `d11-tie` bridge to claude-238's numbers.

## Cancel probe (run first)

- **Does an open PR touch these files?** No. No open PR changes `flock_read_hamming.rs`, `fbs-snapshot.rs`, `fbs_export.rs`, `fbs-toks.rs`, `stack/hamming.rs` or `fbs_snapshot.rs` (`gh pr list --state open`, each PR's files, 1 Nov).
- **Is done-when 2 already met?** No. Parity has been measured on (r-drop) only (claude-238), on the fixed sequences, never on claude-146's checkpoints or on DEV.
- **Does a DEV path already exist in these tools?** No.
  - `fbs-snapshot.rs` calls only `fbs_float_readout`, and its trace checks `windows.len() != READOUT_SEQUENCES`.
  - The integer tools (`fbs-toks --hamming-trace`, `compare_hamming_traces`, `fbs-compare`) take their sequences from the readout and require only equal counts.

Class: **instrument** (the parity reference's inputs). The unit goes ahead.

## Files in scope

- `crates/uor-r4-integer/src/stack/hamming.rs`
- `crates/uor-r4-integer/src/stack/mod.rs`
- `crates/uor-r4-integer/src/stack/session.rs`
- `crates/uor-r4-integer/src/stack/format.rs`
- `crates/uor-r4-integer/src/fbs_snapshot.rs`
- `crates/uor-r4-integer/src/stack/synthetic.rs`
- `crates/uor-r4-integer/src/stack/tests.rs`
- `crates/uor-r4-integer/src/bin/fbs-toks.rs`
- `crates/uor-r4-integer/src/bin/fbs-export.rs`
- `crates/uor-r4-float-train/src/fbs_export.rs`
- `crates/uor-r4-float-train/src/bin/fbs-snapshot.rs`
- `crates/uor-r4-float-train/src/flock_read_hamming.rs`
- `docs/harness/prep/integer-hamming-read*`
- `docs/loop/tasks/integer-hamming-read/**`
- `docs/research/notebook/m4-serving.md`
- `docs/roadmap/items/integer-hamming-read.md`

## Done when

1. **The DEV option is in, with a test that fails without it.** `fbs-snapshot --dev DATA_ROOT` works in both forms:
   - DEV is loaded with its pins, cut into the trainer's windows and sampled every 8th window;
   - the readouts are taken on that sample;
   - the trace checks the sample's count.

   A CPU test pins the sample rule, and fails with the rule set aside. Checks:
   - the crate's tests pass;
   - `uor-harness loop gates --full` is green;
   - the CUDA build type-checks with the stub nvcc.
2. **A prep sheet** (`docs/harness/prep/integer-hamming-read-dev-2026-11-01.md`) registers the bars, with a fresh FIT and a fresh pre-run review, each posted on #3385.
3. **One pod job** produces each seed's outputs on DEV: the snapshot, the plain, export and D11 readouts, and the `d11-tie` and `d11` traces. It also produces the fixed-sequence `d11-tie` trace.
   - The outputs are sealed and stored before release and down.
   - The laptop computes the integer traces, the comparisons, `fbs-compare --parity-only` and the copy codes from the stored files.
4. **The REPORT** gives both numbers per seed and arm, and the sheet's row. Also:
   - a notebook row;
   - the item's step 5 records the result.
   - If the sheet's parity row holds, done-when 2 is met. The item's status follows done-when 3's state, which the REPORT states.
   - The item's two older lines are reworded (round-2 nits carried from claude-239): "d11, the forward as trained", and the quoted "every float reference".

## Stop and ask if

Each of these means stop and decide in a decision review, never stop and wait:
- DEV's pins or window count differ from the trainer's (14,287 targets, 428 windows);
- a claude-146 checkpoint fails a pin, or `fbs-snapshot` refuses it;
- the sample rule's test passes with the rule set aside;
- the throughput check projects the job past 2 hours;
- any instrument check in the sheet fails. That is row 0: repair, never count.
