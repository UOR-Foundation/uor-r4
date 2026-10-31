# Prep sheet: the point key served in the integer artifact (point-key-integer run, claude-229), 2026-10-31

- Item: [point-key-integer](../../roadmap/items/point-key-integer.md) (#3450). ORDER: [claude-229](../../loop/tasks/point-key-integer/claude-229/ORDER.md).
- Run: all on the laptop's bit-exact integer engine (Apple M-series), under the owner's 31 Oct allowance for this item's scoring ("If the laptop bit-exact will allow us to proceed right now I think we should do that"). No GPU is leased.
  - The only GPU step, (r-drop)'s snapshots, ran in claude-227 (store `results/claude/integer-retention-rates-parity-2026-10-31/out/full/`).

## 1. Question

**Does the served point key carry the reply in the integer artifact, as form C's own bar reads it on a fresh panel?** (DECISION.md, claude-220: "Its own bar on a fresh panel".)

claude-220's float model carried at all three seeds on p-w. The integer form names an opening by the integer trunk's own points (condition (b)), so its names may differ from the float key's by design. Its bar is read on its own tokens.

## 2. Arms

The model is (r-drop) step 16,384 at seeds 1–3, the checkpoints claude-217 to claude-220 scored, exported per seed:
- **G, integer greedy:** `/7`, `fbs-export SNAPSHOT A7`. It has the turn memory and the Hamming read, and serves the retained recurrence (claude-226; `rec_gates = "geometric-retention"`).
- **S, integer served:** `/8`, `fbs-export --point-key A7 TRAIN/tokens.u16 TRAIN/response_mask.u8 A8`. It is `/7` plus TRAIN's point table, which names an opening where the exact key misses (claude-222).

Both decode each prompt in a fresh session, with the memory on and the reply start marked at its last token, then greedy (`fbs-toks A --replies PROMPTS --out FILE --reply-tokens 64`, over `panel_reply`).
- **The prompts:** `train-in-space --panel-prompts 1 --samples 1536 --reply-tokens 64`, `score_rows`' own windows and prompts.
  - On DEV, p-w and p-v, each seed's prompts carry claude-220's float served names, for parity.
  - On p-x, the prompts carry none: no float arm served it.
- **The cap:** the integer session cannot slide its window, so a reply ends where prompt and reply fill the 192-token context, as well as at BOS or 64 tokens. G and S are capped identically, so the bar stays a fair paired test. The capped count is reported for each split, arm and seed.

## 3. Bar and decision table (done-when 4)

**S is read against G** by its own tokens, paired on the same prompts, at each seed, on **p-x**. In each pair, b counts S right and G wrong, c the reverse, and the band is 2·√(b + c). S **carries at a seed** when all four hold (`IntegerGrade::carries`, claude-220's `carries`):
- **(a) Templates:** value-masked template matches, b − c > band, with b + c ≥ 10.
- **(b) Exact replies:** b − c ≥ −band.
- **(c) Loops:** b − c ≤ band, with b counting S looping where G does not.
- **(d) DEV:** S's templates, b − c ≥ −band.

| Row | Reading on p-x | Decides | Next |
|---|---|---|---|
| 0 | **An instrument fault** (§4), at any split and seed | Nothing is counted. | Repair in a new unit, then rerun. |
| 1 | S carries at **all three** seeds | Form C carries in the integer artifact. Done-when 4 is met, and the item is done once tok/s (done-when 5) is reported. | The one model end to end with the point key, served on a client (one-model-end-to-end). |
| 2 | S carries at **one or two** seeds | Reported under its row, with each seed's (a)–(d) and b + c. No panel is redrawn (the item's rule). | A decision card: pool the seeds under a fresh sheet, or a larger panel, by whether the failing seeds are thin (b + c < 10) or harmed ((b), (c) or (d) failing). |
| 3 | S carries at **no** seed | Form C does not carry in the integer form on this fresh panel. | A decision card on the next form, with the float result on p-w beside it. |

**Reported, deciding nothing:**
- **Parity against the float key (done-when 3)** on DEV, p-w and p-v at each seed, in the item's fixed classes (`integer_parity`):
  - agree;
  - within the rounding bound (the float name's row within 512 Q12 units of the integer least distance);
  - trunk drift.

  Beside it, the replies' agreement on the same prompts.
- **The other splits' `carries`:** p-w and p-v.
- **The free-generation measures** for both arms on every split (`generation`).
- **The capped counts.**
- **Tok/s and the resident memory** (§6, step 8).

## 4. Instrument checks (each must hold; any failure is row 0)

The item's done-when 2, built by claude-228:
1. **`I-Reference-Scan`:** at every served opening, the search's name and least distance equal the reference scan's, with the same query and table (`check_served_point` in `panel_reply`). A mismatch stops the decode. Each decode prints `reference_checked`.
2. **`I-Read-Back`:** `/8`'s table reads back from its bytes as built (`check_point_key_read_back` in `fbs-export --point-key`), or the export refuses.
3. **`I-No-BOS`:** no served name is BOS (`ServedPointKey::name` refuses it at the step).

And these:

4. **`I-Retained`:**
   - each snapshot carries `"retain_rates":true` once;
   - each `/7` and `/8` header's `rec_gates` is `geometric-retention`;
   - the snapshots' sha256 equal the store's (`dcbfb69b…`, `d0522d02…`, `9744d385…`).
5. **`I-Artifact-Identity`** (claude-228's review): each artifact's sha256, as `fbs-export` prints it, equals the sha256 of the file every later step loads. It is checked before the decodes and again before the timing.
6. **`I-Rows`:** each `/8` table holds 212,968 rows, TRAIN's opening count (the census), and no other.
7. **`I-Grade`:** `--integer-grade` refuses none of its checks:
   - a served reply that does not open with its keyed token;
   - an unkeyed reply that differs from greedy's;
   - float names that are not the seed's served file's;
   - a missing arm, seed or split.
8. **`I-Prompts`:** `--panel-prompts` refuses no float served name that is not its prompt's, and its DEV, p-w and p-v prompt counts equal claude-224's check (310, 1,202 and 393).

## 5. Base artifact and data split

- **Base:** (r-drop) step 16,384, seeds 1–3, through `fbs-snapshot` in claude-227 (memory and Hamming read carried). The run config has `retain_rates: true`, `hamming_read: 4`, `turn_memory: 8` and `turn_memory_reply: true`.
- **Data split:**
  - **TRAIN:** `w2-inputs/2026-10-25/driver-data/mw-balp/train`, 7,512,797 tokens with 212,968 openings. It builds the point table and nothing else.
  - **DEV** (`ft/dev`) carries bar (d).
  - **p-x decides.** It is a fresh held-out-phrasing panel at seed 2131, which no panel had used (the seeds used are 2051 to 2121 in steps of 10):

    `heldout-panel out=p-x cell=dev,dev conversations=1200 seed=2131 protocol=2`

    - **The files:** `tokens.u16` sha256 `5f996480628c13b974af728a903a80066466da0f5b38f9cb04edab62fd2c8c6d` and `response_mask.u8` `904d7e073a67eb8d085cbbc176449818082dd3403fae2e5b76901ed656ca8dc6`, 149,800 tokens. A second draw is byte-identical.
    - **Stored** at `inputs/claude/heldout-panel-p-x-2026-10-31/` with `SHA256SUMS`, and read back OK.
    - **The census** ([census-p-x-2026-10-31.json](../../labs/point-key-integer/census-p-x-2026-10-31.json)), a count with no model, counts 4,212 openings:
      - 0.000 of its prompts are in TRAIN, exactly or as templates at K = 1,024 (0.045 at K = 256);
      - 0.627 of its replies are verbatim TRAIN replies;
      - the exact key fires on 703 openings.

      DEV's and Fresh V2's rows reproduce p-w's census exactly. Fresh V2 was counted only, from its preservation archive under its pins (`0b7e754c…`, `a1cda529…`), as claude-202's census did.
    - **No model has read p-x.**
  - **p-w and p-v** carry parity against claude-220's float served files, and are reported.

## 6. Steps

All paths are on the laptop. `R` is the run folder in the unit's worktree (`local/claude/run/`), and `D` is the data root.
1. **The snapshots:** `uor-store get` the three, then check `I-Retained`.
2. **`/7`:** `fbs-export snapshot-sN.json R/a7-sN.uorlut` for N = 1–3, recording each printed sha256 (`I-Artifact-Identity`) and header (`I-Retained`).
3. **`/8`:** first a timing on a slice of TRAIN, to project the pass. Then `fbs-export --point-key R/a7-sN.uorlut D/mw-balp/train/tokens.u16 D/mw-balp/train/response_mask.u8 R/a8-sN.uorlut`, with the seeds in parallel (`I-Read-Back`, `I-Rows`).
4. **The prompts:** `train-in-space --data-root D --panel-prompts 1 --out R/prompts --panel p-w=… --panel p-v=… --panel p-x=… --runs <claude-220 out> --samples 1536 --reply-tokens 64` (`I-Prompts`).
5. **The decodes:** 24 runs (2 arms × 4 splits × 3 seeds) of `fbs-toks R/a{7,8}-sN.uorlut --replies R/prompts/prompts-{split}[-sN].json --out R/r-drop-sN/integer-{greedy,served}-{split}.json --reply-tokens 64`.
   - DEV, p-w and p-v use each seed's own prompts, so both arms read the same ones.
   - Each run's wall time is recorded (`I-Reference-Scan`).
6. **The grade:** `train-in-space --data-root D --integer-grade 1 --runs R --keys-runs <claude-220 out> --panel p-w=… --panel p-v=… --panel p-x=…` into `R/grade.json` (`I-Grade`). The row is read from it.
7. **Seal:** `R` without the artifacts but with their sums, plus the three `/8` artifacts, to `results/claude/point-key-integer-run-2026-10-31/` with `SHA256SUMS`, read back.
8. **Tok/s, first in the REPORT:**
   - **Decode against context:** `fbs-toks R/a7-sN.uorlut` and `fbs-toks R/a8-sN.uorlut --stages`, the standard protocol (a 128-token prefill and 64 decode steps, 1 warmup and 20 timed runs).
   - **The search counted:** that protocol never marks a reply start, so the point search does not run in it. The search is measured from step 5, as each split's S wall time against G's over its keyed openings.
   - **Resident memory:** `/usr/bin/time -l`'s maximum resident set for one decode of each arm.

## 7. Minimum test power

- **claude-220's float key on p-w,** a panel of the same size and draw, read templates (50, 9), (44, 4) and (47, 5): b + c of 59, 48 and 52.
- **p-x holds 4,212 openings,** against p-w's 4,340, and about 1,200 prompts.
- **If the integer form's effect is near the float's,** each seed's b + c is about 50 against the floor of 10, and net about +40 against a band of about 14. So each seed carries with high probability.
- **The integer names differ by design,** and the cap shortens long prompts' replies. Either can shrink the effect, so rows 2 and 3 are registered, and each seed's b + c is reported.
- **The instruments have no power question:** they check every served opening, every table and every file.

## 8. Cost and stop rule

- **Cost:** the laptop only, no money.
  - At the measured 4,000–4,800 tok/s of this trunk family, the decodes are about 2.8M steps over the 6 arm-seeds, roughly 12 minutes, plus one table search per keyed opening.
  - The TRAIN pass is about one pass of TRAIN's windows per seed: the slice timing in step 3 projects it, three seeds in parallel.
  - The disk needs about 3 × 115 MB for `/8` and 3 × 60 MB of snapshots; 30 GiB is free.
- **Stop rule:**
  - Any instrument check that fails stops the run as row 0 (§4), never counted (CHARTER §4.1).
  - A slice that projects the TRAIN pass past 4 hours with the seeds in parallel stops it, and the code is fixed first.
  - The decodes and the grade never read p-x before every check of steps 1–4 holds.

## 9. Non-author TEST FITNESS review

TEST FITNESS: PENDING

## 10. Pre-run review

PENDING
