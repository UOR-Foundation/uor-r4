# REPORT: one-model-end-to-end, claude-211 (the protocol's training run)

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md). Order: [ORDER.md](ORDER.md). Sheet: [one-model-end-to-end-training-2026-10-29](../../../../harness/prep/one-model-end-to-end-training-2026-10-29.md), FIT and pre-run PASS after three rounds. Protocol: [one-model-end-to-end-protocol-2026-10-29](../../../../harness/prep/one-model-end-to-end-protocol-2026-10-29.md).

## Tokens per second

- **Training:** 33,925–34,326 context tok/s for (r), (r-drop), (r0), (r-ident) and (r-frozen); 29,793–29,996 for (r-spin); 27,229–27,365 for (r-flock). That is one RTX 5090 at width 256, about 1,470–1,850 s per 16,384-step run.
- **Decode tok/s of the shipped recipe:** not measured here. Its integer serving path is not built yet (Still open).

## What did not work, first

- **Decision 4's registered rule shipped greedy, and greedy is degenerate.**
  - The Flock rank rule failed at seed 1: 404 complete against 433, band 17. It held at seeds 2 and 3.
  - Greedy's replies loop: repeated-4-gram rate 0.49–0.50 on the 512-reply decode panel, and 0.56–0.64 on the 256 P-dev replies, against the references' 0.0. Examples: "small small small…" and ",,,,,".
  - Casey, 30 Oct: "do not rule flock out, improve it instead". So the end-of-turn Flock rule was built and run (below).
- **The verdict is "mixed" under either rule.**
  - **Neither verdict holds.** For "memorized", (r-drop) is not ahead of (r0) on K at all three seeds (inside at seed 2: d 548, band 552), and v − v_ref ≤ tol at every seed. For "generalizes":
  - **(r-drop) is not ahead of (r0) on U or N-u.** (r0) is close on N-u, but unstable across seeds: 10,272, 11,413 and 9,446. Its spread of 2,047 sets the band.
  - **Under greedy:** 186–206 of 256 P-dev replies complete, they loop, and v is 0.35–0.48 against v_ref 0.566.
  - **Under the end-of-turn Flock rule:** 255–256 of 256 complete, and the repeat rate is 0.001–0.002. But only 0.004–0.012 of the replies are verbatim TRAIN replies, against the references' 0.566.
  - **Reading the replies:** some are apt ("Noted!" after "Okay, I'll remember all of that"). Many are word salad ("Gran name tasty tellwa…"). The 16-rank harmonic draw lands below rank 0 about 70% of the time, so content tokens drift.
- **Decisions 2 and 3: no decision.** (r-flock), (r-spin), (r-ident) and (r-frozen) each have a three-seed cell inside the band against (r), on U and on N-u (single seeds can lie outside it: (r-flock) and (r-frozen) on U at seed 2). That is underpowered at this scale, never a negative (L-007).
- **Two instrument faults, both repaired and not counted:**
  - **I-C's frozen check** first loaded tensors through candle onto the CPU, which the project's CPU guard refuses (D24/D29). It now compares safetensors bytes (`5081b8066`).
  - **Seed 2's first seal upload** hit a transient Hugging Face `502`. The pod was kept up, and the retry sealed it.

## What changed

- **I-D** (`008b40413`): the controls (T), the memory's own tables, and (S), the longest suffix over TRAIN, via `--controls 1`.
- **I-E's GPU half** (`e4ebdcfdd`): the Flock rank rule carries its K, and a CUDA test checks that K = 1 decodes as greedy on a model.
- **I-G** (`1b15c4e49`): `--grade 1`, with the strata, the three-seed cells and the five decisions. It has one test per rule, and each of ten negative gates fails as it should.
- **The Flock rank rule that keeps the end of turn** (`54ca497d5`): `DecodeRule::FLOCK_END`, `--flock-end 1`. A nearest `<|eos|>` is taken; every other token is drawn as before.
- **I-C** (`5081b8066`): the byte-level frozen check, with a test.
- **The job, watcher, seal and finish scripts** (`local/claude/claude-211/`). EXIT reports every arm's step, and a pod is taken down only after its seal is checked.

## Measurements

**Instruments, every pod:**
- I-E, I-A (16,417 P-dev rows), I-B at P = 0 and P = 1000, and I-C under `one-atom` all pass.
- I-H0 passes: (r) twice gives the same bytes, so training is reproducible on these GPUs.
- I-C under `frozen`: 36 routing tensors are byte-identical between steps 0 and 256, while 38 others moved.
- I-F's full count: 13,061 later rows, the decoder's key equal to the census's on 13,061 of them, and the reply table naming 9,655.
- The strata are exactly 10,849 / 3,438 / 15,143 / 1,274.
- **I-H, the reproduction:** (r) and (r0) at seeds 1 and 2 give DEV 11,513 / 11,361 / 10,079 / 11,171. These equal claude-194's readings exactly, across builds.

**The five decisions** ([grade-registered.json](../../../../labs/one-model-end-to-end/training-2026-10-30/grade-registered.json), [grade-flock-end.json](../../../../labs/one-model-end-to-end/training-2026-10-30/grade-flock-end.json)):

| Decision | Reading | Numbers, at seeds 1 / 2 / 3 |
|---|---|---|
| 1. Base recipe | **(r-drop) ships**: ahead of (r) on N-u and on U | N-u: 11,412 / 11,315 / 11,395 against (r)'s 4,304 / 3,914 / 4,172 (band 293). U: d = 340 / 494 / 354 (band 154) |
| 2. Add-ons | (r-flock), (r-spin): no decision | inside the band against (r) on U and N-u |
| 3. Transport | no decision | (r-ident), (r-frozen) inside on N-u (bands 285 and 377) |
| 4. Decode rule, registered | greedy | Flock rank: 404 / 482 / 459 complete against greedy's 433 / 440 / 432 |
| 4. Decode rule, end of turn kept | **Flock rank ships** | 508 / 510 / 510 complete against 433 / 440 / 432. Repeat rate 0.002 / 0.001 / 0.002 against 0.50 / 0.49 / 0.49 |
| 5. Verdict | mixed: neither verdict holds | "generalizes": not ahead of (r0) on U or N-u, with (r0)'s spread at 2,047. "memorized": not ahead of (r0) on K at all three seeds (inside at seed 2, d 548 against band 552), and v − v_ref ≤ tol at every seed. Under Flock with the end kept, v is 0.004–0.012 against v_ref 0.566 |

- **How the two decode rules were graded:**
  - `grade-flock-end.json` is `--grade 1 --flock-end 1`. It reads the end-of-turn rule's own files (`samples-*-flock-end-memory.json`) and records `"flock_rule": "keeps the end of turn"`.
  - `grade-registered.json` records `"registered"`.
  - The end-of-turn rule was added after the first grade, at Casey's direction. So its reading is a second decode rule beside the protocol's, not the protocol's own decision 4.
- **That `decode_reply` passes `keep_end` and the end of turn to `flock_pick`** is shown by the run, not by a unit test. On the same checkpoints, its replies differ from the registered rule's, and 508–510 of 512 complete against 404–482.
- **How (S) counts:** every transition in TRAIN, prompt tokens included, ignoring the reply mask. It matches suffixes by 64-bit hashes, so a collision is possible, though unlikely.
- **Against (S), the strongest lawful recall control:** (r-drop) is ahead on U and on N-u at every seed, by about 1,300–1,400 rows.
- **On K, the keyed rows,** (r-drop)'s picks equal the tables', 9,423 at every seed.

**The seal:** `results/claude/one-model-end-to-end-2026-10-29/training-460fe46e/s{1,2,3}/`, 748 files each, with a `SHA256SUMS`, and every fetched file matched.

**Cost:** about $7.65 for the three training pods (about 4.5 h each at $0.563/h, up to each seal). Four pods ran at the peak, inside the caps.

## Checks run at the head

- **The crate's host tests** (`cargo test --locked -p uor-r4-float-train --lib`): 157 passed.
- **The CUDA tests on each pod** at `460fe46e9`: 210 passed, `the_flock_rank_rule_at_k1_decodes_as_greedy_on_a_model` among them.
- **The ten negative gates of I-G's rules, and the gate for `flock_pick`:** each fails as it should.
- **`--grade 1`** on the 21 sealed runs, with the frozen probe: exit 0.

## Skills used

None.

## Still open

- **Flock rank's content draw.**
  - **The problem:** the end-of-turn rule fixed completion and loops, but the 16-wide harmonic draw puts unrelated tokens in replies.
  - **Next:** a geometric narrowing that draws only among candidates inside the nearest token's cell, so a near tie gets the draw and a far one never does. Measure it on a new panel.
- **(r0)'s instability:** 9,446–11,413 on N-u across seeds. Its spread sets the verdict's band; more seeds, or a lower-variance (r0), would make the verdict decidable.
- **Serving the shipped recipe in integers:** (r-drop) through the Hamming read's exporter, `turn-memory-integer` units 2 and 3, and the nearest readout. Then its decode tok/s.
- **P-dev has now been opened once,** as registered. Any later decision needs a new panel.

## Plain summary

The training run trained all 21 models cleanly, and every instrument check passed. It reproduced the earlier run exactly. The recipe that ships is memory dropout, (r-drop). On prompts phrased in new ways it gets 11,300–11,400 of 15,143 rows right, where the always-on memory gets about 4,000, and it beats the strongest memorize-only baseline at every seed.

Greedy decoding loops. Keeping the end of turn in Flock rank makes replies complete (99%) with almost no loops, but its content is often word salad. Narrowing the draw is the next step.
