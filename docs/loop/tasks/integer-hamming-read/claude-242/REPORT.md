# REPORT: integer-hamming-read, claude-242 (the read's decode speed to a context of 32,768)

Lab: claude. Item: [integer-hamming-read](../../../../roadmap/items/integer-hamming-read.md). Order: [ORDER.md](ORDER.md). Closes #3385.

## Tokens per second

This is a reading, not a bar. It used `fbs-toks ARTIFACT --context-sweep --tokens TOKENS_U16` on claude-240's s1 artifact, re-exported at a context of 32,768 (`69b660bd…`). The prompt was DEV's first 32,704 token ids. It ran on this M1 on AC power, from 11:16:43Z to 11:55:44Z on 1 Nov 2026.

| Prefill | Context end | Decode with the read | Decode without | The read's share of decode | The read per token | Prefill with / without |
|---|---|---|---|---|---|---|
| 128 | 192 | 4,647 (4,577) | 6,093 (5,918) | 23.7% | 51.0 µs | 5,061 / 6,140 |
| 512 | 576 | 3,345 (2,663) | 5,462 (4,520) | 38.7% | 115.8 µs | 3,984 / 5,293 |
| 2,048 | 2,112 | 1,851 (1,388) | 5,462 (3,946) | 66.1% | 357.0 µs | 2,598 / 5,504 |
| 8,192 | 8,256 | 659 (502) | 5,590 (5,019) | 88.1% | 1,337.0 µs | 1,126 / 5,497 |
| 32,704 | 32,768 | **177** (164) | **5,923** (5,552) | **97.0%** | 5,466.8 µs | 348 / 5,865 |

How to read the table:
- Decode is tokens per second over 64 decode steps after each prefill: the median of 20 timed runs, with the p95 in brackets, after one warm-up.
- "With" and "without" are the same artifact; `without_hamming_read()` drops only the read.
- The read's share is (with − without) / with, from the median times. The read per token is (with − without) / 64.

## What did not work, first

- **The read's time grows with the context, and at long contexts it is nearly all of decode.**
  - At 32,768, decode is 177 tok/s with the read and 5,923 without, so the read takes 97% of the time.
  - Without the read, the trunk stays flat at 5,462 to 6,093 tok/s from 192 to 32,768, as a recurrence should.
  - So the item's "cheap to serve by construction" holds for the read's memory (about 2 MiB of codes per token at 32,768), not for its time. 5.47 ms per token over 2 MiB is about 0.38 GB/s, far below this machine's memory bandwidth. The read is limited by its own computation, not by memory.
- **The cost is linear in the context: 157 to 169 ns per earlier position per decode token.** That is about 20 ns per candidate for each of the 8 head-reads (2 reads × 4 heads).
  - **Why, from the code, not from a profiler.** `nearest` (`stack/hamming.rs:281`) builds a new `Vec` of `(popcount, s)` over every earlier position, sorts the whole of it with `sort_unstable`, and keeps 16. It does this for each head-read at every step.
  - The slope does not rise from 576 to 32,768, so the sort's log factor is not visible. How the 20 ns splits between the scan, the key lookup and the sort was not measured.
- **The machine was not idle.**
  - Before: load averages 3.64, 5.12 and 6.61. `kernel_task` was at 91%, with WindowServer, `mediaanalysisd`, Codex and `lean` running.
  - After: 2.82, 3.23 and 3.87, with Kimi Code, Codex and `lean` running.
  - The ORDER does not stop another lab's processes, so they ran beside the sweep. The noise shows in the no-read arm's p95: 4,520 and 3,946 at 576 and 2,112, against medians of 5,462. The medians are the reading.
- **The first I-Reply run read DEV's `tokens.u16` as raw ids.**
  - Its first id, 20,309, is "UO", the start of the file's `UORT` magic. `fbs-toks` refused it: "prompt token 20309 is outside the vocabulary 4096".
  - No number was taken from that run. The prompt is now read past the 64-byte header, and the new sweep parses `UORT` the way `fbs-export` does.
- **The ORDER's I-Reply could not run as first written.** The short artifact cannot run the long sweep, and `--prompt` runs only with the read.
  - So it was amended at the run, before the sweep, into three parts (done-when 3 below). The amendment is in the ORDER.
- **Only one seed was timed: s1's artifact.**
  - The read's work is set by the shape (8 head-reads over `t − 1` candidates), not by the weights, so s2 and s3 should cost the same.
  - That is read from the code, not measured.

## The result

**Done-when 3 holds**, and with it the item's done-whens 1 to 4:
- done-when 1: claude-194, 195 and 200;
- done-when 2: claude-240;
- done-when 3: this unit;
- done-when 4: the notebook rows.

The item is `done`.

The table above is the measurement. To 32,768, the trunk alone decodes at about 5,500 to 6,100 tok/s at every length. With the read, the time per token grows by about 167 ns per earlier position. That puts decode at 4,647 tok/s at 192, 1,851 at 2,112 and 177 at 32,768.

Peak memory at 32,768 was 145,473,536 bytes (`/usr/bin/time -l`), so a session at this length fits on this laptop.

## Instrument checks

- **I-Same (done-when 2): the short re-export is the stored artifact byte for byte.**
  - `fbs-export` at this head, on claude-240's s1 `snapshot-writer.json`, gives `7c82d76b82f50376d0cdd29dc8c2c371d455cc78dd2ad6a7524df9127070f59c`.
  - That equals the stored `artifact.uorlut` from `results/claude/integer-hamming-read-dev-2026-11-01/reading/s1/`, so the exporter's bytes have not moved since claude-240.
- **The long export changes only the context.**
  - The snapshot's diff is one line, `"context":192` to `"context":32768`. The occurrence count was checked as 1 before the `sed`.
  - Both artifacts are 2,156,480 bytes. They differ only in bytes 9 to 8,109, 7,332 bytes in all: the header-length field (`0x1f9b` to `0x1f9d`), the manifest's `shape.context`, and the padding the longer number shifts.
  - Every byte after 8,109, the whole payload, is identical.
- **I-Reply (done-when 3): the long artifact gives the short one's replies.**
  - The tokenless `--context-sweep` (prefills 32 to 128, with and without the read): reply sha256 `d93dc6d2…` on both artifacts.
  - `--prompt` over DEV's first 128 ids, with the read: `82b728a4…` on both.
  - The long sweep's 128-point reply with the read: also `82b728a4…`.
  - So the context field is inert, as the ORDER's method argued: the trunk is `"r"` per layer, with no table sized by the context, and the read's candidates are `1 ≤ s < t` with no window.
- **The read is active at every point.** At all five prefills, the reply with the read differs from the reply without it (45 to 55 distinct ids of 65 with the read).
- **Every timed run gave the same reply.** That is the protocol's own check: `fbs-toks` stops with "timed runs wrote different replies" if any of the 20 differs from the first.
- **The token file:** 42,856 ids at vocabulary 4,096, sha256 `0919e90f…6555b`. It is recorded in the sweep's JSON (`tokens`).

## What changed

`crates/uor-r4-integer/src/bin/fbs-toks.rs`:

- **A new form, `fbs-toks ARTIFACT --context-sweep --tokens TOKENS_U16`.**
  - It runs the `--context-sweep` protocol at prefills 128, 512, 2,048, 8,192 and 32,704 (`LONG_SWEEP_PREFILLS`), over the first 32,704 ids of a `UORT` token file.
  - The report keeps the schema `uor-r4.fbs-toks-context/1`, and adds `context` and `tokens` (the file, its sha256 and its id count).
- **`long_sweep_prompt` reads the file and refuses:**
  - an artifact whose context is under 32,704 + 64;
  - a file that is not `UORT` (shorter than its header, or the wrong magic);
  - a header count that does not match the payload;
  - fewer ids than the largest prefill;
  - an id outside the vocabulary.
- **Without `--tokens`, `--context-sweep` is unchanged:** prefills 32 to 128 and the same report.
- **The test** `the_long_sweep_reads_a_uort_file_and_refuses_what_it_cannot_sweep` checks:
  - the tokenless prefills and the long sweep's context ends (192 to 32,768);
  - the low-byte-first read;
  - each refusal above.

The docs:
- the ORDER, with the cancel probe and the I-Reply amendment;
- this REPORT;
- the item: done-when 3 recorded, status `done`;
- the notebook row in `m4-serving.md`.

## Reviews

- **The in-unit fresh-context review** of the PR at its exact head is recorded on the PR.

## Measurements and cost

- **The sweep:** 2,339 s wall time, 2,233 s user, on this M1 (8 cores, 4 performance, 16 GiB), on AC power. Peak RSS 145,473,536 bytes. Exit 0.
- **No GPU.** The work was host-only: the export, the I-Same and I-Reply checks, and the sweep.
- **Stored** at `caseyallard/uor-r4-store:results/claude/integer-hamming-read-long-context-2026-11-01/`. The folder holds 12 files:
  - `long.uorlut`;
  - the sweep's JSON, stderr, summary, and before and after machine states;
  - the four I-Reply outputs;
  - the DEV128 prompt;
  - `instrument-checks.txt`.

  They were read back into a fresh folder, and all 12 sha256s equal the sealed copy's.

## Checks run at the head

The code last changed at `da9b0cc70`; the commits after it are docs.
- **`cargo test --release --locked -p uor-r4-integer --features serving-wasm --no-fail-fast`:** 614 passed, 35 failed, 4 ignored (`local/claude/claude-242/gates/crate-tests.out`).
  - All 35 panic on a missing local file, a bundle or `tokenizer.json`, at a hard-coded path under `~/uor-r4-investigations/`, which no longer exists.
  - claude-241 recorded the same 35, with 612 passed. None of the 7 files is touched here.
  - `fbs-toks`'s 5 tests pass, the new one among them.
- **`cargo clippy --locked -p uor-r4-integer --features serving-wasm --bin fbs-toks`:** exit 0, with no warning in `fbs-toks.rs`.
  - The library's 51 warnings are older code.
  - Under `--all-targets -D warnings` the crate fails on older files, and on none of this diff.
- **`cargo fmt -p uor-r4-integer -- --check`:** clean.
  - The workspace-wide `cargo fmt --check` reports 9 files in other crates, none of them touched by this branch.
- **`uor-harness loop gates --full`:** 11 of 11 green.
- **`uor-harness loop report --check`:** PASS.

## Skills used

None.

## Still open

- **Select the 16 nearest by counting, not by sorting.**
  - Distances lie in 0 to 64, so a 65-bin count over the candidates finds the threshold distance d\*.
  - One ascending pass over `s` then keeps every `d < d*`, and `d = d*` in ascending `s` until 16 are kept. That is the same set and order the tie rule gives now, with no allocation and no sort.
  - Codes kept per position, rather than looked up through the token each time, would also drop one dependent load per candidate.
  - Both keep the read linear in the context, which the item's design makes it. Neither has been timed.
- **claude-241's snap work:** its remaining 32% of samples is unsplit.
- **Carried minors:**
  - claude-240's three code minors;
  - claude-238's and claude-239's carried items;
  - claude-241's Lows.

## Plain summary

The integer look-back read was timed on this laptop at contexts from 192 to 32,768 tokens, on real text.
- **Without the read,** the model's speed hardly changes with length: about 5,500 to 6,100 tokens a second.
- **With the read,** each new token checks every earlier position, so it slows as the context grows:
  - about 4,600 tokens a second at 192;
  - about 1,900 at 2,000;
  - 177 at 32,768, where the read is 97% of the time.

The long-context copy of the model is the same model: its replies are the same token for token. The slow part is the way the read picks its 16 nearest positions. It sorts every candidate. A counting method gives the same answer without the sort, and it is the next thing to try.
