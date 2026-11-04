# Prep sheet: scoring passes side by side on one GPU (one-model-end-to-end, P3(a), claude-277, 4 Nov 2026)

Lab: claude. Item: [one-model-end-to-end](../../roadmap/items/one-model-end-to-end.md) (#3407). This is step P3(a) of Casey's 4 November plan: run a seed's scoring passes side by side on one GPU instead of one after another.

TEST FITNESS: PENDING - a fresh non-author adversarial review of this sheet is under way.

**What this run measures.** In P2b (claude-276), each seed scored its checkpoints in three passes, one after another, on one RTX 5090:
- 2,624 s for (L) at 2,048, then 148 s for (L) at 192 and 138 s for (S) at 192;
- each pass peaked at 5,296 MiB of 32 GiB;
- seed 1's GPU trace shows a kernel running in 99% of samples over the 2,048 pass, but in only 74% on average over (S)'s pass at 192, with a quarter of its samples under 50%.

This run re-scores P2b's sealed seed-1 checkpoints, unchanged, with the same commands at the same pin. It times the passes serial against side by side, and checks that every scored file is byte-identical either way. No training runs, and no recall or model reading is taken. The object is the scoring job's schedule, not the model.

## Question

- At 192, do two scoring passes run side by side on one GPU finish sooner than the same two in turn, with every scored file byte-identical?
- How does that scale to four passes at once?
- At 2,048, does a second identical pass beside the first cost about nothing, or about a whole pass? This says whether the 2,048 forward leaves the GPU room. That decides whether P3(c), memory on and off in one forward, can help at 2,048.

## Base artifact

- **The pin** is `e1ad88afc5a7f43f5e7714d9ab70b8937aa6bdde`, the commit P2b scored at. `train-in-space` is built there with `--locked --features cuda`; the pod checks the commit is that one and clean, and records the binary's SHA-256.
- **The checkpoints** are P2b's seed 1, under `results/claude/one-model-end-to-end-long-context-2026-11-04/s1/`:
  - (L) `L/models/step-008192/`;
  - (S) `S/models/step-021845/`.

  The seal file `SHA256SUMS.files` there has SHA-256 `ca0b8aeda4e23a6f793df8b503534a48ff090724632dc384c66a0b4b27424b23`. The pod checks the ten checkpoint files against its lines.
- **The outputs** go under `results/claude/one-model-end-to-end-side-by-side-2026-11-04/`.

## Arms

Every pass runs P2b's scoring command, unchanged, on GPU 0: `--score-rows CKPT --out DIR --samples 16 --reply-tokens 32 --retain-rates 1 --seed 1 --panel p-long=PLONG --panel-queries p-long=PLONG/queries.json`. The passes at 192 add `--context 192 --panel p-v=PV`; the passes at 2,048 add `--context 2048`. Each pass writes to its own new folder.

- **A (serial):** (L) at 192, then (S) at 192. Run twice: A1 and A2.
- **B (side by side):** (L) at 192 and (S) at 192, started together. Run twice: B1 and B2.
- **C (four at once):** (L) at 192, (S) at 192, (L) at 192 and (S) at 192, started together. Run once.
- **D (2,048 pair):** (L) at 2,048, twice, started together. Run once.

The order on the pod is A1, B1, A2, B2, C, D. Interleaving A and B spreads any drift in the host over both. D runs last, so a long D cannot hold up the readings at 192.

## Readings

- **G (the gate): byte-identity.** For every pass, take the SHA-256 of every file under its `--out` folder. Each pass at 192 must equal A1's pass of the same checkpoint. D's two passes must equal each other.
  - Reported beside it, not part of the gate: A1 and D against P2b's sealed files, which come from another 5090.
- **R1 (deciding): the pair's wall time at 192.** For each of A1, A2, B1 and B2, the seconds from the phase's first start to its last end, from the job's millisecond stamps. The speedup is mean(A) / mean(B). Both repeats are shown.
- **R2: four at once.** C's wall time against 2 × mean(A), the serial time of its four passes. Reported only.
- **R3: room at 2,048.** r = D's wall time / 2,624 s.
  - 2,624 s is P2b's seed-1 (L) pass at 2,048: the same pin, the same checkpoint, on another 5090. The seed-2 pass took 2,622 s.
  - Beside it: each D pass's decode tok/s against P2b's 15.2.
- **R4 (tok/s first):** every pass's decode tok/s for each queries file, from the `decode` pair (tokens, seconds) that `--score-rows` prints, in every phase. Also every phase's GPU memory peak and its utilization trace.

## Bar

- **Positive:**
  - G holds in every pass;
  - both B repeats finish faster than both A repeats;
  - mean(A) / mean(B) ≥ 1.2.

  From here on, the job scripts on this line (P4 to P6) run a seed's scoring passes side by side, at most as wide as R2 shows still gains.
- **Negative:** G holds, but B does not beat A by that margin. The passes stay serial. This closes this schedule on this pod type only.
- **Unread (instrument):** G fails anywhere. No speed reading counts. The REPORT then classifies the failure:
  - if A1 and A2 differ too, the scorer itself is not deterministic on one pod;
  - otherwise, running passes side by side changes a scored file.

  The schedule is not adopted until this is repaired.
- **Inconclusive:** the repeats disagree; one B is faster than an A and the other is not.

**R3 sets P3's next unit, whatever R1 reads:**
- r ≤ 1.3: the 2,048 forward leaves the GPU room. P3(c) is next: memory on and off in one forward. Its rows are the same prompt at the same length, so they batch with no padding.
- r ≥ 1.7: the 2,048 pass fills the GPU. P3(c) is then for the passes at 192 only, and P3's next unit is chosen in its own ORDER.
- Between the two: P3(c) is still next, with R3 named as uncertain.

Whatever the outcome:
- no model, recall or served reading rests on this run;
- the REPORT gives every number.

**Reversal.** Casey can say "hold": no further pod launches. The outputs stay sealed under the results root.

## Instrument checks

- **I-0:** the CUDA library suite at the pin, `cargo test --locked --features cuda -p uor-r4-float-train --lib --no-fail-fast -- --test-threads=1`. It passes when at least one test ran, none failed, and P2b's nine named tests are each `ok`.
- **I-1 (the inputs):**
  - `sha256sum -c SHA256SUMS` passes in each input folder;
  - every SHA-256 named in the Data split equals its file's;
  - each of the ten checkpoint files matches its line in P2b's seal.
- **I-2 (the passes):** every pass exits 0, and its `scored.jsonl` holds a `decode` pair for each p-long queries file it decodes. A pass that runs out of memory fails here.
- **I-3 (the gate can read):** A1 and A2 are byte-identical, pass for pass. If they are not, G cannot tell running side by side apart from the scorer's own noise.

## Data split

The scoring inputs are P2b's, fetched from the store and checked by their SHA-256s, as in P2b's job:
- **Driver data,** `w2-inputs/2026-10-25/driver-data/`:
  - `tokenizer.json` `d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89`;
  - `mw-balp/train/tokens.u16` `3dee858f84b750f507ceb3dc3f393711741950954735efaaf61bf704c017a4c8`;
  - `mw-balp/train/response_mask.u8` `b5e7352d67d6883a833155a5fd397a03f6d4f1b2649e3d4ec3e2ea7d95b949e3`;
  - `ft/dev/tokens.u16` `0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b`;
  - `ft/dev/response_mask.u8` `a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078`.
- **p-v,** `inputs/claude/heldout-panel-p-v-2026-10-30/p-v/`:
  - `tokens.u16` `3488eef5bba82c3d3b97633fbd92f568c237636e6bb44d9decbdb8886d4196f0`;
  - `response_mask.u8` `c170fd0ac9568f98fa3e2ab5f22af0fe31b94ea14bbbf802daf446e20d5ca023`.
- **p-long,** `results/claude/one-model-end-to-end-long-context-2026-11-04/inputs/p-long/`:
  - `tokens.u16` `8b2529b1a1385112ef41a42eca50dc982dc474f3b40e8948b8887b6b4158ab36`;
  - `response_mask.u8` `b17ff53fc37dfe2910b1ffca820538ca5473d1b0b707da3f06eb83c25c58fed0`;
  - `queries.json` `cf5c30b96e98f7770314f96be5114cf5eeb21e3f9657aeb84b4c1d6168ca2390`;
  - `user_turns.json` `4d21bc632073c6384e0193960c862ddbb326ecedf4ed27b6af30811319207800`.

Nothing is held out and nothing is graded for recall, so no split is spent.

## Steps

1. On one RTX 5090 pod, fetch the pin, build, record the binary's SHA-256, and run I-0.
2. Fetch the inputs and the two checkpoints, and run I-1.
3. Run A1, B1, A2, B2, C, then D. Each phase starts only when the one before has ended. Record I-2 after each.
4. Seal every output with `SHA256SUMS.files`, put it in the store, read it back, and only then take the pod down.
5. On the laptop: check I-3 and G, then compute R1 to R4 and apply the Bar.

## Minimum test power

Each arm runs twice. "Both B repeats faster than both A repeats" happens by chance in 1 of the 6 orderings of four equal times. The 1.2× floor keeps a gain inside timing noise from counting. P2b's two pods timed the same pass within 2 s of each other at 2,048 and 10 s at 192. That noise is far under the 1.2× floor at 192, about 57 s on a 286 s pair.

## Cost

The pod takes about:
- 10 minutes to fetch and build, and 10 for I-0;
- 5 minutes to fetch the inputs and checkpoints;
- 10 minutes for A1 and A2, at about 286 s each;
- 5 to 10 minutes for B1 and B2;
- 5 to 10 minutes for C;
- 45 to 90 minutes for D;
- 15 minutes to seal and read back.

That is about 1.8 to 2.7 hours. Verified 5090 offers ran from $0.975 an hour at this sheet's writing, against $0.563 for P2b's pods, so it costs about $1–2.7. `uor-pod status` is read at launch, inside 4 pods and $8 an hour. The watcher's wall limit is 4 hours (`MAXSEC=14400`), its spend limit $4, and the lease is taken for 4 hours and renewed. A budget or time stop wakes the session to extend a healthy run, never to cancel it.

## Stop rule

- **A failed instrument check** (I-0 to I-2) ends the job at once; it seals what it wrote. The fault is repaired, and the run is repeated at the repaired job.
- **D running past the wall limit** is extended, never cancelled. If the pod dies during D, the readings at 192 stand, R3 is unread, and D is run again alone.
- **A pod that fails to start** is replaced by one more, once, inside the caps.
