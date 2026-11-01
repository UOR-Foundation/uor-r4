# ORDER: integer-hamming-read, claude-242

Lab: claude. Item: [integer-hamming-read](../../../../roadmap/items/integer-hamming-read.md). Base: `5bd5f9896`.

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

This unit takes done-when 3, the last one open. Done-when 1 is met by claude-194, 195 and 200. Done-when 2 is met by claude-240 (`50d3958cd`, row P: parity on DEV at all three seeds).

claude-235's `--context-sweep` is the start. It took done-when 3 early and stopped at the readout window: prefills of 32 to 128, plus 64 decode steps, so contexts 96 to 192. It ran on battery under load, and its REPORT says that a longer sweep, on mains power and an idle machine, is still owed. The item claims about 2 MiB of read per token at a context of 32,768. This unit measures decode at that length.

## Method

- **The artifact.** The snapshot is claude-240's seed 1, `snapshot-writer.json`, from the store at `results/claude/integer-hamming-read-dev-2026-11-01/reading/s1/`. Its stored export, `artifact.uorlut`, has sha256 `7c82d76b82f50376d0cdd29dc8c2c371d455cc78dd2ad6a7524df9127070f59c`. It is exported twice with `fbs-export` at this unit's head:
  - **short:** as stored, at context 192;
  - **long:** with only the snapshot's `context` field raised to 32,768.

  The edit is one `sed` of the single `"context":192`, with its occurrence count checked first. Nothing else in the snapshot changes.
- **Why the long export serves the same model.**
  - The FBS trunk's shape is `"r"` per layer: recurrence only, with no read layer, so there is no `age` or other table sized by the context.
  - The read's candidates are `1 ≤ s < t`, with no window.
  - So the context is a buffer bound. Two checks hold it to that: I-Same and I-Reply below.
- **The sweep.** A new form, `fbs-toks ARTIFACT --context-sweep --tokens TOKENS_U16`:
  - prefills 128, 512, 2,048, 8,192 and 32,704, each followed by the 64 decode steps, so the contexts end at 192, 576, 2,112, 8,256 and 32,768;
  - the same protocol as `--context-sweep`: one warm-up and 20 timed runs, every run's reply the same, with the read and without it (`without_hamming_read`) on the same artifact;
  - the prompt is the token file's first 32,704 ids;
  - it refuses an odd-length file, a file shorter than the largest prefill, an id outside the vocabulary, and an artifact whose context is under 32,768;
  - without `--tokens`, `--context-sweep` is unchanged.
- **The tokens.** DEV's `tokens.u16`, from the store's `w2-inputs/2026-10-25/driver-data/ft/dev/`: 85,776 bytes, so 42,888 ids. It is real text, so the read's matches are the language's, not a repeated pattern's.
- **The machine.** This M1 on AC power, with no build or timing of this lab's running. Load averages and the top CPU users are recorded before and after the sweep. Another lab's process is never stopped; if the machine is busy, the sweep waits for a quiet time, and the REPORT says when it ran.

## Cancel probe (run first)

The unit is cancelled if either check fails:
- **(a) The measurement is still owed.** No row on main measures the read's decode past a context of 192.
- **(b) The read is still served.** The read is not withdrawn, and the served read is at parity.

**Result (1 Nov): both pass; the unit goes ahead.**
- (a) `docs/research/notebook/m4-serving.md` rows 11 (claude-241) and 12 (claude-240) both stop at contexts 96 to 192. Since claude-235, no commit to `fbs-toks.rs` or `stack/hamming.rs` adds a longer sweep: `git log 62e3cfed4..HEAD` on both lists only claude-237.
- (b) claude-240 merged parity on DEV as `50d3958cd`, and the item is `ready`.

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

1. **The sweep exists and is tested.** `fbs-toks --context-sweep --tokens` runs the protocol at the five prefills above. Tests cover the token file's reading and each refusal. The tokenless form keeps its prefills and schema.
2. **I-Same:** the short re-export equals the stored `artifact.uorlut` byte for byte. If the exporter's bytes moved since claude-240's build, the stored artifact is the short arm, and the REPORT names the change.
3. **I-Reply:** the long artifact gives the short artifact's reply at a prefill of 128, with the read and without it, over the same 128 DEV ids.
4. **The numbers.** At each context end (192 to 32,768), with the read and without it, the REPORT gives:
   - decode tok/s, as the median and p95 of the 20 runs;
   - the read's share of decode time;
   - the machine's state.
5. **The record:**
   - the item's done-when 3 is recorded, and its status is set: done if done-whens 1 to 4 hold on main;
   - one row in `docs/research/notebook/m4-serving.md`;
   - a REPORT, negatives first.

## Stop and ask if

The lab answers these itself and does not wait (CHARTER §5.1):

- **I-Reply fails.** Then the context field is not inert. That is an instrument finding, not a speed result. The sweep then moves to a synthetic artifact at claude-146's shape (`stack/synthetic.rs`, `FbsSynth.context`), and the REPORT says so.
- **A session at 32,768 does not fit in memory on this laptop.** Then the largest prefill drops to the largest that fits, and the REPORT says why.

The lab makes every other call itself and records it in the REPORT (CHARTER §5.1).
