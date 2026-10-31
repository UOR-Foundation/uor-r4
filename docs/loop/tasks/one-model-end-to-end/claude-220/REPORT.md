# REPORT: one-model-end-to-end, claude-220 (the point key served in the one model)

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md). Order: [ORDER.md](ORDER.md), written by hand on Casey's word: "screw kimi forge ahead as you were before with your plan for the single serving model".
- **The decisions:** [DECISION.md](DECISION.md), serve the exact f32 table first (APPROVE on five conditions); [DECISION-projection.md](DECISION-projection.md), the probe's time projection corrected (APPROVE).
- **The sheet:** [one-model-end-to-end-served-key-2026-10-31](../../../../harness/prep/one-model-end-to-end-served-key-2026-10-31.md). It was NOT FIT in rounds 1–3 and FIT in round 4. The pre-run review was GO.
- **Results:** [served-key-2026-10-31](../../../../labs/one-model-end-to-end/served-key-2026-10-31/README.md). References #3407.

**These are float research evaluations** (CHARTER §3.5, §3.7), on the GPU. Their tok/s is the GPU float decode's, not the served speed. The integer export of the table is owed, under its own item (the decision's conditions (a) and (d)).

## Tokens per second

- **Decode, on p-w with three seeds sharing one RTX 5090:** greedy 54.2–55.3 tok/s a process, the served arm 52.6–54.1, about 2% slower.
- **The served search:** one a reply, a single query against 212,968 TRAIN opening points.
  - It took 32 ms a search with three processes sharing the GPU (1,172 searches in 37.1–37.6 s a seed), and 5.8 ms alone (the probe on DEV, 19 in 0.11 s).
  - It scans the whole 218 MB table each time.
- **Residency:** the table is 218,931,272 bytes, resident per checkpoint, and 657 MB in the store for three seeds.

## What did not work, first

- **The gain is recall of TRAIN replies, not new wording.**
  - On p-w the served arm gives 665, 619 and 728 verbatim TRAIN replies against greedy's 377, 443 and 380.
  - Its share of new 4-grams falls from 0.73–0.84 to 0.57–0.70.
  - That is the right move on this panel, whose references are 0.647 verbatim TRAIN replies, but it is not generalization. In the protocol's words it is recall of a template, read by its new phrasing.
- **The served key closes about 18% of the gap to a right first token.**
  - A forced reference opening lifts templates on p-w by +230/+219/+238 of 1,202. The served key lifts them by +41/+40/+42.
  - Its openings are right on 28–32% of the 1,172 it names.
- **The search is not cheap under sharing.** It costs 32 ms with three processes on the GPU, against 5.8 ms alone, and it scans 218 MB each time. A served model on a client needs the integer table and a faster search.
- **DEV says nothing new.** The served key opens 19 of its 310 replies, since the exact key fires on the rest. Every DEV pair is within its band.
- **The probe stopped once at my own time rule,** an instrument of cost.
  - The rule took 1,053 s × 18, but 253 s of the probe were one-off overhead, and three seeds sharing the GPU decode at half speed, not a third.
  - A fresh decision review approved the corrected projection: 2.8 hours, 3.6 fully scaled, under the same 5-hour stop.
  - The main run took 2.73–2.78 hours, as projected. It cost about 4 minutes of idle pod.
- **The FIT took four rounds.** Round 1 found the power too low and no rule fixed for a served name differing from the keys file's. Rounds 2 and 3 found two routes by which a harm could read as "underpowered". Each was repaired before any model read p-w, and is pinned by a unit case.

## What worked (the registered outcome: row 2, the one float model serves the point key)

- **The served point key carries the reply at all three seeds on the second fresh panel, p-w (1,200 conversations).**
  - **Templates against greedy:** (50, 9), (44, 4) and (47, 5), net +41/+40/+42 against bands of 15.4, 13.9 and 14.4, with b + c of 59, 48 and 52.
  - **Exact replies:** (48, 9), (44, 4) and (47, 4).
  - **Loops are fewer** (130/167, 132/243, 72/344), and DEV is not lower.
- **The served key is exactly the measured key.**
  - At every split and seed, every served token equals the keys file's name (1,172 on p-w, 385 on p-v, 19 on DEV), with a distance drift of 0.0. The decode's own point at the opening is bit-equal to the batched forward's.
  - The served files equal claude-219's keyed-point files byte for byte on p-v and DEV at every seed.
- **claude-219's result reproduces.** Its p-v and DEV files, 14 checks a seed, reproduce byte for byte from the served model.
- **Power was as argued after FIT round 1:** b + c of 48–59 a seed, against claude-219's 10–19 on p-v.
- **Replies complete more and repeat less.** p-w's complete replies rise from 990/889/929 to 1,048/1,006/1,072, and the repeated 4-gram rate falls (0.529/0.639/0.587 to 0.471/0.554/0.434).
- **So the next unit is the point table's integer export,** under its own roadmap item, with a bar measured on the integer trunk's own points (the decision's conditions (b) and (d)). Then the integer model end to end. A draft plan and item are ready (`local/claude/claude-221-plan.md`, `claude-221-item-draft.md`).

## What changed

- **The served key** (`ServedKey`, `served_opening`, `set_served_key`, `take_served_searches`).
  - In `decode_reply`, at the reply's first step under the greedy rule, where the turn memory's exact first-8 key misses, the opening row's own point from that forward is named by `l1_named` against the table, with no extra forward.
  - Each served sample keeps its token as `keyed` and its least distance as `served_distance`.
- **The table** (`--served-key 1`, `--opening-table PATH`): TRAIN's opening points under the trunk, built in the keys pass, saved to `out/opening-table.safetensors` (`save_opening_table`, never replacing a file), read back to the same values (`load_opening_table`), and served from the copy read back.
- **The served arm** (`samples-{split}-served-memory.json`), through the shared decode helper, with `ScoredFile::search`, the search's count and seconds.
- **The grade** (`--served-grade 1`, `served_grade`):
  - parity (`served_parity`, `ServedParity::holds`, failing past 1% or on no served opening);
  - the served arm read by its own tokens through `keyed_reading`;
  - the decision (`served_outcome`, with `KeyedReading::harmless` split from `carries`, whose meaning for claude-219's grade is unchanged);
  - the protocol's free-generation measures (`generation`).
- **`IdSample` drops `Eq`** for its new f32 field.
- **Also written:** the 1,200-conversation panel p-w (seed 2121) and its census, the two decision cards, the sheet, the lab page, the grade, and one notebook row.
- **The local jobs and watcher** are in `local/claude/claude-220/`:
  - `job.sh`, sha256 `0dd3b6cfec56b38091bf731d52931d4aa55d24a8b4f9fd341a662ae14107d40d`;
  - `job2.sh`, `6e5e5b43705b82113e09a4f825067f7d1f49e2e34b1c8c84f1a3cbba34ddfd95`.

## Measurements

The tables are on the [lab page](../../../../labs/one-model-end-to-end/served-key-2026-10-31/README.md).
- **The run:** one RTX 5090 (`vast-58466776`, $0.563/h), about 3 h 20 min, about $1.90.
- **Built** at `eb762e602` (binary `e2c8057f…5acb`). Later commits change docs, the decision card, and test lines only.
- **Sealed** at `results/claude/one-model-end-to-end-served-key-2026-10-31/run-eb762e60/`: 129 files, `SHA256SUMS` sha256 `482008278ddf2ca1d5acb7f51a3e433091d83a2913b369a818f183273bd4e069`, read back OK. The pod is down.
- **The tables' sha256s:** `31f1fbc1…1317`, `03e8b8ac…c1c9` and `8364bf6a…d54c` at seeds 1–3.
- **The grade:** [grade-p-w.json](../../../../labs/one-model-end-to-end/served-key-2026-10-31/grade-p-w.json), sha256 `6984f5c03af1f86e69493bc6fb0d65ca32df668e289426b2ccd2f1c54a2b5950`.

## Checks run at the head

- **The crate's host tests** (`cargo test --release --locked -p uor-r4-float-train`): all pass, 180 in the library. The CUDA code type-checks with the stub nvcc.
- **Negative gates on the host,** each failing with its rule set aside and passing when restored:
  - the table's never-replaced check;
  - parity's 1% rule;
  - the distance drift;
  - the pooled guard;
  - the guard's harm condition;
  - the thin seed beside a harmed one.
- **On the pod:**
  - the crate's CUDA tests pass, including the served key's own;
  - every input matches its sha256;
  - the probe and the main run reproduce claude-219's files (7 checks, then 14 a seed);
  - the served files equal claude-219's keyed-point files on DEV and p-v.
- **The grade on the sealed run:** exit 0, with no refusal.

## Skills used

None.

## Still open

- **The point table's integer export** (row 2's next unit): a new roadmap item, `point-key-integer`, drafted in `local/claude/claude-221-item-draft.md`.
  - The table is built by the integer engine at the writer query's i16 Q12 (about 109 MB).
  - The search is the existing `-L1` scan with a commonest-at-the-least-distance pass, at `stack_step_fbs`'s first reply step.
  - Its bar: parity against this unit's float served names, the replies, and tok/s on the laptop.
- **A faster search:** the table is scanned whole at every keyed opening. A smaller or indexed table is a later unit's, with its own bar.
- **Generalization:** the key recalls TRAIN replies; the trunk's own reading of new questions (training the opening point to gather paraphrases) is still the route to replies TRAIN never shows.
- **p-w has now been opened once,** by this decision.

## Plain summary

Last unit showed a trick that helps the model start its reply with the right word: when a question is worded in a way the model's memory doesn't recognize, look up which known questions the model's internal state is closest to, and start with the word those questions' replies started with. But that lookup was done in advance, from a file.

This unit built the lookup into the model itself, so it works while the model writes its reply, and tested it on a new, larger set of reworded questions. It works: at all three of the trained models, the model now writes clearly more right replies than it did before. The lookup inside the model gives exactly the same answers as the version tested last time, down to the last bit.

The honest catch is that it helps by recalling replies the model saw in training, which is what this test's answers mostly are, rather than by writing new ones. It also gets only about a fifth of the way to what a perfect first word would give, and the lookup is slow when the GPU is shared.

The next step is to build the same lookup in whole numbers, so the model can be served on an ordinary laptop end to end.
