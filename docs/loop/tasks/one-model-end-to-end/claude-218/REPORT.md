# REPORT: one-model-end-to-end, claude-218 (a memory key a reworded question can reach)

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md). Order: [ORDER.md](ORDER.md).
- **The redirect:** from the rollout hinge, by the lab's decision review, logged on #3407 (6526603691).
- **The sheet:** [one-model-end-to-end-memory-key-2026-10-30](../../../../harness/prep/one-model-end-to-end-memory-key-2026-10-30.md). It was NOT FIT in rounds 1–3 and FIT in round 4. Its amendment, the Hadamard sign key (Casey: "ok add that trick"), was FIT in round 5. The pre-run reviews were GO.
- **Results:** [memory-key-2026-10-30](../../../../labs/one-model-end-to-end/memory-key-2026-10-30/README.md). References #3407.

## Tokens per second

- **Decode:** 52–60 tok/s per process on one RTX 5090, with three seeds sharing it (p-u: greedy 59.9–60.0, forced 58.8–59.9, memory off 52.0–60.4). This is not the served speed.
- **Each trunk key's search** takes about 7 ms an opening (10.2 s for p-u's 1,466) against TRAIN's 212,968 opening points (218 MB as f32). That is the two-pass instrument's time, an upper bound on a served key's. The sign keys need 32 bytes an opening.

## What did not work, first

- **The memory's exact key misses new wording.** On p-u it fires on 248 of 1,466 openings. On p-s it fires on 17.0% and is right on 19.3% of those (step 0; on DEV, 97.4% and 50.3%).
- **Order-free word keys do not help.** The prompt's distinct ids fire on 0.4% of p-s, and the first 8 as a set behave like `first-8`.
- **The cheapest key that passes is not the best one.** The sign key ships by the registered rule, but the point key gains about 3.7 times as much (424 against 116 over the three seeds). The next unit must weigh the point key's size (1 KB an opening, against 32 bytes) against its gain.
- **The Hadamard sign key's gain varies by seed.** At its chosen level its new-stratum net is +38, +31 and 0, and its DEV backoff +5, 0 and +5. It passes, but it does not beat the plain sign key at the chosen levels.
- **Nothing here is a decode.** The keys are read teacher-forced at the opening; the trained, served key and its free-generation effect are the next unit's.

## What worked (the registered outcome: row 3, the sign key ships)

- **On p-u's 1,218 missed rows against the trunk alone, the sign key (level 2) gains +46, +37 and +33.**
  - Each is beyond its band (16.5, 18.2 and 16.4), with b + c at least 10.
  - Where it fires, its hits are 88, 80 and 88 against the trunk's 42, 43 and 55.
  - **The new stratum** (448 rows whose reference is not a verbatim TRAIN reply) nets +39, +30 and +28.
  - **DEV's backoff** nets +8, +4 and +6.
  - So the next unit trains and serves the sign key, with an opening bar on another fresh panel.
- **All three keys pass row 3.**
  - **The point key** (level 10, always firing) lifts missed-row openings from 12.9–18.0% to 27.3–28.1%.
  - **The Hadamard sign key** (level 1) gains +46, +33 and +26.
- **Row 1 does not hold: a right opening matters.** Forcing only the reply's first token lifts exact replies on p-u from 4, 0 and 0 of 256 to 54, 49 and 52 (paired 50/0, 49/0, 52/0), and template matches by 52, 49 and 54.
- **Row 2 does not hold.** The oracle U against the exact key reads 889/5 at every seed.
- **Casey's trick improves the sign code.** Read with every row firing, the Hadamard sign key gains +50, +72 and +30 against the plain sign key's +10, +45 and +2.

## What changed

- **Step 0:** `--opening-bound 1 --panel NAME=DIR`.
  - It scores DEV and each panel, counts each key's fire rate and its precision (`fired_hits`), and adds the order-free keys `set` and `set-first-8` and the oracle `reply-template`.
  - It runs the anchor check on every split (`opening_bound`, `bound`, `reply_at`, `template_keyed`, `PromptKey`).
- **The probe:** `--score-rows … --opening-keys 1`. It builds, with the memory off:
  - **`opening_points`:** TRAIN's opening points under the trunk.
  - **`l1_named`:** the commonest target among the TRAIN openings at the least L1 distance, ties to the smaller id, in two passes on the device with no dot product.
  - **`sign_code`**, and **`hadamard_sign_code`** = sign(H·S·x), with `rotation_signs` and `walsh_hadamard`: 8 add-and-subtract stages.
  - **`keys-{split}.json`** (`OpeningKeys`).
  - **A forced-opening arm:** each reply's first token is its reference's, through a thread-local in `decode_reply`.
- **The grade:** `--key-grade 1 [--keys-runs DIR] --panel p-s=DIR [--panel p-u=DIR --key-levels sign=L,hsign=L,point=L]`.
  - It holds the sheet's decision table (`key_outcome`), the per-seed decile levels chosen on p-s (`level_tau`, `choose_level`, `level_given`), the strata (`key_strata`), row 2's arm (`oracle_reading`), and row 1's forced reading.
  - It refuses as instrument faults: the panels' token and mask sha256s, the anchor check, keys at the split's openings, the memory moving a missed row, the forced arm's opening token and its 256 replies, and the follow-up keys equal to the main run's (`same_trunk_keys`).
  - With no levels it never opens p-u.
- **Also written:** the panel p-u (seed 2101) and its census, the sheet and its amendment, the lab page, the levels file and the grade, and one notebook row.
- **The local job and watchers** are in `local/claude/claude-218/`:
  - `job.sh`, sha256 `485fe7fcb4d497bac902b0421986b4146176ad96eea37a75cf133151ca32e661`;
  - `job2.sh`, `fb08f05156ea778ecbb77cb8059caf163ca253234c6cce3b2042e479fea6a092`.

## Measurements

The tables are on the [lab page](../../../../labs/one-model-end-to-end/memory-key-2026-10-30/README.md).
- **The run:** one RTX 5090 (`vast-58426412`, $0.563/h), about 70 minutes, about $0.66.
- **The main run:** built at `4cc00df2a` (binary `20d37cbe…7fc4f`).
  - Sealed at `results/claude/one-model-end-to-end-memory-key-2026-10-30/run-4cc00df2/`: 115 files, `SHA256SUMS` sha256 `d9efe9d31d4f79f91818cfe3ee06173f0840ad949419e362e774755be4bf25bd`.
- **The follow-up pass:** built at `d5b6f49f7` (binary `4121465f…d814c`).
  - Sealed at `followup-d5b6f49f/` inside it: 83 files, `587b0b235de768cb1b3a0cdd080fcc3bb5b2fd6a04b04c975a302b484b6cc05b`.
- Both were matched on the pod and the laptop file for file, and read back OK. The pod is down.
- **The levels** (p-s, committed at `dfa703c36` before p-u was read): sign 2, hsign 1, point 10. Their net gains on p-s's missed rows, summed over the seeds, were 77, 105 and 429.

## Checks run at the head

- **The crate's host tests** (`cargo test --release --locked -p uor-r4-float-train --lib`): 176 pass. The CUDA code type-checks with the stub nvcc.
- **On the pod:**
  - the CUDA tests pass;
  - two negative gates fail on their assertions after a real recompile (ties to the larger id; the fixed signs set aside);
  - every input matches its sha256;
  - the 15 registered files reproduce byte for byte;
  - the follow-up pass's 18 rows files equal the main run's.
- **Negative gates on the host,** each failing with its condition set aside and passing when restored:
  - step 0's precision count and order-free sort;
  - in the grade: the backoff mask, the level tie, row 4's band, the new stratum's band, row 1's floor and its one-sidedness, row 2's two-sidedness and its floor, row 3's strict edge and its floor, row 4's floor, row 2's arm tie, the level check and the anchor refusal;
  - for hsign: the key order and the fixed signs.
- **The grade on the sealed runs:** exit 0 for both the levels and the decision. It checked every field of the follow-up pass's point and sign keys against the main run's.

## Skills used

None.

## Still open

- **The next unit (row 3):** train and serve the sign key, with an opening bar on another fresh panel.
  - Before it is registered, the point key's 3.7 times larger gain should be weighed: the sheet's "cheapest that passes" was written before any key was read.
  - The Hadamard rotation applies there too: in the integer export's quantization, and as the sign key's rotation.
- **Free generation:** whether a right opening from the key carries the reply (the forced arm says it should: about a fifth of replies exact).
- **The rollout hinge** stays deferred, not negative.
- **p-u has now been opened once,** by this decision.

## Plain summary

The model's memory looks up a question by its exact first eight words. So when a question is reworded, it almost never finds the answer it already knows, and the first word of the reply comes out wrong.

This unit tried looking the question up by where the model's own internal state lands instead. On a fresh set of reworded questions, that finds the right first word noticeably more often. The cheapest version, which looks at just the signs of that state, passes every registered test, so it is what we build next. The fullest version, which uses the whole state, does about three to four times better again.

Getting the first word right matters a lot. When we handed the model only the correct first word, it wrote the exact right reply about a fifth of the time, against about once in two hundred now.

Casey's Hadamard trick made the sign-based version better when it is used everywhere.
