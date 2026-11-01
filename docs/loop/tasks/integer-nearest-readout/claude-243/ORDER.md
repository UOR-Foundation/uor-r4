# ORDER: integer-nearest-readout, claude-243

Lab: claude. Item: [integer-nearest-readout](../../../../roadmap/items/integer-nearest-readout.md). Base: `abbfb0001`.

## Goal

The item's done-when reads:

> 1. **The artifact carries its readout,** with a negative gate. An artifact without it decodes byte for byte as at the base, and an engine without it refuses a nearest artifact.
> 2. **The nearest step** serves `stack_whole_table_nearest`'s rule under the field, with a test against a reference scan and a negative gate.
>    - The browser path (`wasm_serve`'s step) gives the same token as the session's nearest on one artifact, with a test.
>    - The serving audit (`audit_zero_matmul_serving.py --stack`) and the WASM build pass.
> 3. **End-to-end token parity passes its bar, under a prep sheet with a fresh FIT and a pre-run review.**
>    - Integer greedy of (r-drop) step 16,384, memory on, against claude-220's stored float greedy on DEV and p-w at seeds 1–3.
>    - It is reported as first-token agreement and whole-reply agreement. The bar is registered in the sheet against the exporter's known rounding residual (claude-227: 0.33% of sequence positions).
>    - **The item is done only when the bar passes.** On a miss, the sheet's row is recorded, the item stays `ready`, and the next unit locates the gap.
>    - Integer decodes run on the laptop: product serving (CHARTER §3.6), under the owner's 31 Oct allowance for this chain. The float side is stored GPU output.
> 4. **Tokens per second,** with the nearest scan counted (4,096 rows a token), and the resident memory.
> 5. **One notebook row, and a REPORT with the negatives first.**

Done-whens 1 and 2 are met by claude-231. This unit takes done-when 3 again, and done-when 4 with it.

claude-232's parity missed every cell. The units since have found two differences between its two sides. Neither is a rounding residual:

- **The tie order.** claude-220's float greedy was built at `eb762e602`. There, the read keeps its 16 positions by one GPU `arg_sort`, and the read's own doc says "Ties may come out in another order".
  - The integer read keeps ascending distance, then ascending `s`.
  - The float read has taken that rule by default since claude-239 (`flock_read_hamming.rs`, `TIE` starts on).
  - On (r-drop) step 16,384's fixed sequences, the tie order alone moves the D11 readout's top-1 at 174 and 227 of 1,536 positions (claude-238). On claude-146's checkpoints on DEV, it moves 10–14% of next-word choices (claude-240).
  - So no exact port of the read can meet a 3% bar against claude-220's stored replies.
- **The snap.** `fbs-export` snaps the trunk's gates and transition to the 120 icosians by default. claude-232's artifacts are snapped; the trained float forward is not.
  - claude-227's trunk parity held against the D11 forward, which snaps too (d = 5 of 1,536).
  - claude-233's diagnostic, which was not registered: the snapped query sits a median 46% (in L1) from the trained one.
  - `fbs-export --raw-gates --raw-transition` writes them unsnapped. The engine normalizes them at serving (claude-181).

This unit measures the parity with both sides on the same rules:
- the integer artifact exported raw;
- the float reference decoded again at this unit's head, with the tie rule on.

## The decision this needs

Done-when 3 names claude-220's stored replies as the reference, and claude-232 read it on the default export, which snaps. Changing either side is a change to a bar's scope, so it needs a decision card ([DECISION.md](DECISION.md)), approved by this session after a fresh decision review that reads the whole project. It is logged on #3462 for Casey, who can reverse it (CHARTER §5.1).

The card proposes:
- the bar's reference becomes F′, the same decode as claude-220's arm G at this unit's head; claude-220's F stays as a reading;
- the bar's integer side becomes R, the raw export; claude-232's snapped S stays as a reading, the snap's control;
- "done" applies to the raw artifact, and the item records its recipe as the served one.

## Method

- **The arms:**
  - **R, integer, raw.**
    - `fbs-export --raw-gates --raw-transition --run-config CONFIG SNAPSHOT` runs on claude-227's three full snapshots of (r-drop) step 16,384: `results/claude/integer-retention-rates-parity-2026-10-31/out/full/`, sha256 `dcbfb69b…`, `d0522d02…` and `9744d385…`. The configs are claude-232's.
    - Each artifact is decoded by `fbs-toks ARTIFACT --replies PROMPTS --out FILE --reply-tokens 64` on claude-232's prompt files, on the laptop.
  - **S, integer, snapped:** claude-232's sealed integer replies (`results/claude/integer-nearest-readout-parity-2026-10-31/`), read as stored.
  - **F′, float greedy at this unit's head, on the GPU.**
    - It runs claude-220's command, `train-in-space --score-rows CKPT` with claude-220's flags, on the same checkpoints, prompts and panels.
    - Only the "greedy, memory on" arm (G) is read.
    - At this head the read's tie rule is on.
  - **F, float greedy as stored:** claude-220's G files, read as claude-232 read them.
- **The bar** is claude-232's, unchanged: per cell, first-token disagreement ≤ ⌊0.03 n⌋, so 9 of 310 on DEV and 36 of 1,202 on p-w. It is read on **R against F′** in all six cells (3 seeds × DEV and p-w), with claude-232's registered grader.
- **The readings, with no bar:**
  - **S against F′:** the snap's share once both sides keep ties the same way.
  - **R against F and S against F:** S against F must give claude-232's numbers again; that is an instrument check.
  - **F′ against F:** the code's change since `eb762e602` on G's path. By the code, that is the tie rule alone; the reading does not credit the tie rule, since `train-in-space` has no switch to test it (the card lists the commits).
  - **The whole-reply agreement of each pair.**
- **Instrument checks:**
  - **I-Repeat:** seed 3 is decoded twice on the pod at this head, by a second process beside the main three. Its G files on DEV (`samples-memory.json`) and on p-w (`samples-p-w-greedy-memory.json`) must equal the main run's byte for byte. That is one DEV cell and the worst p-w cell. GPU greedy must be repeatable to serve as a reference.
  - **I-Flags:** every flag of claude-220's command parses at this head with the same meaning. Checked before the pod: `git diff eb762e602 <head> -- crates/uor-r4-float-train/src/bin/train-in-space.rs` removes `--opening-table` (which claude-220's command never passed) and adds two modes, and changes no parse line of the flags the command passes.
  - **I-Float-Inputs** and **I-Prompts,** as in claude-232.
  - **I-Raw:** each R artifact's shape must carry `head = native_vector_nearest`, `rec_gates = geometric-retention`, and the raw gate and transition flags. Its parse must give the `/7` schema.
- **Speed (done-when 4):** `fbs-toks` decode and prefill tok/s on R s1, with the nearest scan counted, and the peak resident memory, on this M1 on AC power.
- **Compute:** one Vast RTX 5090, inside the caps, under a prep sheet with a fresh FIT and a pre-run review (CHARTER §4.5).
  - claude-220's three seeds took 9,840 to 9,995 s side by side on one 5090, at about 93% GPU use and 11,354 MiB at the peak.
  - A fourth process, I-Repeat's, puts the run near 3 h 40 min, about $2.30 with setup. The sheet gives the estimate.

## Cancel probe (run first)

The unit is cancelled if any check fails:
- **(a) The parity is still owed.** The item is `ready`, and no row on main measures integer greedy against a float reference with the same tie rule.
- **(b) The tie difference is real.** claude-220's reference build `eb762e602` keeps its 16 by an unruled `arg_sort`, and main's float read starts with the tie rule on.
- **(c) The raw export exists.** `fbs-export` takes `--raw-gates` and `--raw-transition` together with `--run-config`.

**Result (1 Nov): all three pass; the unit goes ahead.**
- (a) The item is `ready` on main (`abbfb0001`). The m4-serving rows for this item are claude-232's MISS and claude-233's localization; both use claude-220's stored F.
- (b) At `eb762e602`, `flock_read_hamming.rs:714` is `.arg_sort_last_dim(true)`, under the doc line "Ties may come out in another order" (`:689`). At main, `:74` reads `static TIE: … Cell::new(true)`. `train-in-space` has no tie switch, so its decode at main runs with the rule on.
- (c) `fbs-export [--raw-gates] [--raw-transition] [--run-config CONFIG_JSON] SNAPSHOT OUT`, with the flags combining (`fbs-export.rs:9` and `:109–118`, at the base). Whether a real `/7` snapshot exports and parses raw is checked again by I-Raw.

## Files in scope

- `crates/uor-r4-integer/src/fbs_snapshot.rs`
- `crates/uor-r4-integer/src/stack/format.rs`
- `crates/uor-r4-integer/src/stack/session.rs`
- `crates/uor-r4-integer/src/stack/kernels.rs`
- `crates/uor-r4-integer/src/stack/tests.rs`
- `crates/uor-r4-integer/src/stack/synthetic.rs`
- `crates/uor-r4-integer/src/bin/fbs-export.rs`
- `crates/uor-r4-integer/src/bin/fbs-toks.rs`
- `crates/uor-r4-integer/src/wasm_serve.rs`
- `crates/uor-r4-integer/src/wasm_bindgen_exports.rs`
- `crates/uor-r4-float-train/src/fbs_export.rs`
- `crates/uor-r4-float-train/src/bin/fbs-snapshot.rs`
- `docs/roadmap/items/integer-nearest-readout.md`
- `docs/harness/prep/integer-nearest-readout*`
- `docs/labs/integer-nearest-readout/**`
- `docs/loop/tasks/integer-nearest-readout/**`
- `docs/research/notebook/m4-serving.md`

## Done when

1. **The decision card** is approved after a fresh decision review, and logged on #3462.
2. **The prep sheet** registers the arms, the bar, the instrument checks, the decision rows and the cost. It has a fresh FIT and a pre-run review, each recorded on #3462.
3. **F′ runs on the GPU** at the sheet's head, with I-Repeat. Its outputs are stored and read back.
4. **R is exported and decoded on the laptop,** with I-Raw. The grader reads every pair above, and the sheet's row is recorded.
5. **The speed and resident memory** of R s1 (done-when 4).
6. **The record:**
   - the item's done-when 3 and 4 are recorded; the item is `done` only if the bar passes in every cell;
   - on a pass, the item names the raw export's recipe as the served artifact's, and records making it the default as a follow-up;
   - one notebook row;
   - a REPORT, negatives first.

## Stop and ask if

The lab answers these itself and does not wait (CHARTER §5.1):

- **I-Repeat fails.** Then GPU greedy is not repeatable, and F′ cannot be a reference. Record row 0, an instrument fault, and stop the run. The next unit makes the decode repeatable.
- **S against F does not give claude-232's numbers.** Then the grader or the inputs moved. Record row 0.
- **`fbs-export` refuses the raw flags with the turn memory or the nearest head, or the R artifact does not parse.** That is a code gap inside this item's scope. The unit fixes it with a test and a negative gate before any decode, and the REPORT says so.
- **The pod cost passes $3.** Extend and note it; a healthy run is never cancelled for a self-set budget.

The lab makes every other call itself and records it in the REPORT (CHARTER §5.1).
