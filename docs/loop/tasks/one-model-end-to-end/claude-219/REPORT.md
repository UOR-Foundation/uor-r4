# REPORT: one-model-end-to-end, claude-219 (the keyed opening in free generation)

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md). Order: [ORDER.md](ORDER.md), written by hand on Casey's word: "screw kimi forge ahead as you were before with your plan for the single serving model".
- **The sheet:** [one-model-end-to-end-keyed-opening-2026-10-30](../../../../harness/prep/one-model-end-to-end-keyed-opening-2026-10-30.md). It was NOT FIT in round 1 and FIT in round 2. The pre-run review was GO.
- **Results:** [keyed-opening-2026-10-30](../../../../labs/one-model-end-to-end/keyed-opening-2026-10-30/README.md). References #3407.

## Tokens per second

- **Decode:** 55–57 tok/s per process on one RTX 5090, with three seeds sharing it. On p-v greedy runs at 55.3–56.4 and keyed-point at 54.9–56.2: the keyed opening costs no decode speed. This is not the served speed.
- **Each trunk key's search** takes about 6.8 ms an opening (9.3–9.8 s for p-v's 1,402) against TRAIN's 212,968 opening points (218 MB as f32). That is the two-pass instrument's time, an upper bound on a served key's. The point key's table is about 1 KB an opening; the sign keys' 32 bytes.

## What did not work, first

- **The sign key that claude-218 shipped does not show a free-generation gain at this panel's size.**
  - Its template pairs on p-v are (6, 0), (1, 1) and (2, 0): b + c under 10 at every seed, too thin to read.
  - It fires on 154–165 of 393 openings and names the right first token on 35–53 of them.
  - claude-218's teacher-forced gain (+46/+37/+33 openings) does not reach whole replies here.
- **The Hadamard sign key is thin too:** (4, 2), (3, 0) and (6, 1).
- **The point key closes only part of the gap.**
  - A right first token (the forced arm) lifts templates by +71/+66/+73 of 393. Keyed-point lifts them by +13/+8/+12, 12–18% of that.
  - Its openings are right on 105–119 of the 385 it gives (27–31%).
- **Seed 2's template reading is at the floor:** b + c is exactly 10, net 8 against a band of 6.3.
- **Part of keyed-point's fall in loops is short replies, not better ones.**
  - 49–58 of its openings a seed are " Noted." and an end of turn, a common TRAIN opening.
  - 332–348 of its replies end within 64 tokens, against greedy's 291–316.
  - So the loop counts (35/47/22 against 60/85/113) are read only as bar (c), "not higher", never as a gain.
- **DEV says little.** The keys open only 12–19 of its 310 replies, because the exact key fires on most DEV openings. Every DEV pair is within its band.
- **My pod job stopped once at its own check, an instrument fault.** It compared the keys files byte for byte, but they record the search's wall-clock milliseconds.
  - 18 of 21 registered files had reproduced. The 3 keys files equal claude-218's at every seed without the three timing fields, checked with jq and with the job's sed.
  - `job2.sh` continued on the same pod, from the same probe, binary and inputs, and the sheet records the amendment, made before any result was read.
  - It cost no extra pod time beyond the probe. The lesson is in FRICTION.md.

## What worked (the registered outcome: row 2, the point key ships)

- **Served as the reply's first token where the memory's exact key misses, the point key carries the reply at all three seeds on the fresh panel p-v.**
  - **Templates:** (16, 3), (9, 1) and (14, 2), net +13/+8/+12 against bands of 8.7, 6.3 and 8.0, with b + c at least 10.
  - **Exact replies:** (12, 3), (9, 1) and (14, 2).
  - **Loops** are not higher (fewer, see above), and **DEV** is not lower.
- **So the next unit trains and serves the point key's opening in the one model and scores it end to end** (#3407, step 4). Its decision review weighs the point key's table (218 MB as f32 for TRAIN's openings) against the gain. Integer export of the table, and a smaller code that keeps the gain, are that unit's to test.
- **The forced arm lifts at every seed** (row 1 does not hold): exact (66, 0), (63, 0) and (70, 0); templates (71, 0), (66, 0) and (73, 0).
- **The prediction held.** The power check on claude-218's p-u files, registered before the pod, predicted keyed-point exact gains of +11/+13/+12 at 256 prompts, and sign and hsign thin. On p-v at 393 prompts: +9/+8/+12, with sign and hsign thin.
- **The reply identity held in every cell.** No keyed token was refused, and every keyed reply is the one its opening fixes:
  - no keyed token: greedy's reply;
  - the reference's first token: the forced arm's reply;
  - greedy's own first token: greedy's reply.

  So the observed exact pairs equal the predicted ones.

## What changed

- **`--score-rows … --keyed-decode sign=T,hsign=T,point=T`**, which needs `--opening-keys 1`:
  - the opening keys are taken before the decodes (memory off) and written after them, as before;
  - for each key, a greedy arm with the memory on gives the reply's first token as the key's name where the served memory's exact first-8 key misses the opening and the key's distance is at most τ (`keyed_opening`, `keyed_openings`, `OpeningKeys::by_key`);
  - each forced token is kept as the sample's `keyed`;
  - the forced arm and the keyed arms share one decode helper.
- **`--keyed-grade 1 --runs DIR [--keys-runs DIR] --panel p-v=DIR|p-u=DIR`** (`keyed_grade`):
  - the registered τ (`KEYED_TAUS`), the panels' sha256s (`KEYED_PANELS`);
  - the reply identity (`keyed_reply`) and its refusals (`keyed_reading`);
  - the bar (`KeyedReading::carries`, `thin`) and the table (`keyed_outcome`; row 1 at any seed, after FIT round 1);
  - a BOS-named key is a refusal.
- **Also written:** the panel p-v (seed 2111) and its census, the sheet, the power check, the lab page, the grade, and one notebook row.
- **The local job and watcher** are in `local/claude/claude-219/`:
  - `job.sh`, sha256 `fb2244abcea9bb29498a199e19fbf9ac8da4453187f7f246c08310212efe6bb4`;
  - `job2.sh`, `73f363d23b8b065a3e66285d7baec0510de4c7845bd3642f4685ca85b10bec16`.

## Measurements

The tables are on the [lab page](../../../../labs/one-model-end-to-end/keyed-opening-2026-10-30/README.md).
- **The run:** one RTX 5090 (`vast-58445674`, $0.563/h), about 2 h 15 min, about $1.30.
- **Built** at `da4a6212f` (binary `951e2694…98bf`).
- **Sealed** at `results/claude/one-model-end-to-end-keyed-opening-2026-10-30/run-da4a6212/`: 147 files, `SHA256SUMS` sha256 `730026ee5380cbcb7cb3eea01eb4529af277961028327d42209124bd4079b04d`, read back OK. The pod is down.
- **The grade:** [grade-p-v.json](../../../../labs/one-model-end-to-end/keyed-opening-2026-10-30/grade-p-v.json), sha256 `044fe4aa1a7021fd415ebae2a90759c0fc274fc79ac8539c7fecbfbaaf6d8037`.

## Checks run

- **The crate's host tests** (`cargo test --release --locked -p uor-r4-float-train`): all pass, 178 in the library. The CUDA code type-checks with the stub nvcc.
- **Negative gates on the host,** each failing with its fix set aside and passing when restored:
  - the keyed opening's miss rule;
  - the served memory's lookup;
  - row 1 at any seed.
- **On the pod:**
  - the crate's CUDA tests pass, including claude-218's named GPU tests and this unit's two;
  - every input matches its sha256;
  - the probe's 21 files reproduce claude-218's at all three seeds (the 18 decode files byte for byte, the 3 keys files without their timing fields);
  - the main run's DEV rows and keys equal the probe's.
- **The grade on the sealed run:** exit 0, with no refusal.
