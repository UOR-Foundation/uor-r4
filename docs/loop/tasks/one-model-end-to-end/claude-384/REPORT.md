# REPORT: one-model-end-to-end, claude-384 (V3: serve the pair rows as integer rows)

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Order: [ORDER.md](ORDER.md). PR #3983. Follows claude-370 (PR #3961), claude-373 (PR #3964), claude-374 (PR #3965) and claude-376 (V0, PR #3970). This is stage V3 of the 14 Nov roadmap.

The order asked for claude-370's pair rows (`--pair-rows 2`: 187,413 rows of 256, one per unordered pair of different tokens seen in at least 2 TRAIN user turns, addressed by the product of the two tokens' primes) to reach the integer artifact as i16 rows at 2^-12, and for the served stack to read them. Then it asked whether claude-370's float gain survives serving.

**The bar reads "Served":** over P14 + P23 (2,218 rows), served exact replies go from 423 (S-b) to 524 (S-pair) on seed 2 and from 409 to 517 on seed 3. G_s is 4.71 points against a line of 2.3, and each seed's gain is above zero. claude-370 measured 4.60 points in float on the same panels (431 to 527, 413 to 521): the served gain is the float gain. The replay instrument bar passed first: the served picks equal the trainer's plan on 4,436 of 4,436 prompts, as raw row numbers and raw addresses.

This is a reading on panels and in one nine-turn conversation. It is not a milestone claim, and the row values are an F32 to Q12 export gap, not D85's native path.

## What did not work, first

1. **The facts the rows were first aimed at are still wrong.** Fact and number rows (188) exact: 43 and 46 with the rows, 33 and 27 without. Capital: 0 and 1 of 34 exact with the rows, 1 and 0 without. Adding: 0 of 21 in every artifact. What moved is legs (11 and 11 of 29, against 4 and 8), colour and day after; claude-370 saw the same in float.
2. **The served conversation is not better on values.** In claude-356's nine-turn script, with pair experts off, S-pair answers "What is druvalk?" (told 73) with "That's 66." on seed 2 and "That's 666" (a short cycle) on seed 3; S-b answers "Your cake 18." and "That's 65.". Seed 3's S-pair then breaks: the cycled reply stays in the conversation, and the next two replies run into a made-up user line ("That's 66 / User: O spellucuc remember remember remember"). S-b seed 3 does not break this way. Every artifact answers "What is my name?" (never told) with "You're welcome!".
3. **The model leans on the rows, served as in float.** Served with the rows dropped (S-cut, claude-370's I-8), exact falls to 322 and 316: about 100 below the plain `b` models, and replies that did not end fall from 2,058 and 2,038 to 1,817 and 1,593. A served artifact whose rows are lost or damaged is worse than one trained without them.
4. **The artifact is six times larger.** 116,050,880 bytes with the rows against 19,345,344 without: the rows add 96.7 MB (187,413 × 256 × 2 bytes of values and 187,413 u32 addresses). That is the cost the next step (compression: rows shared by the pair's E8 address, or a higher MIN) has to cut. Compression did not run in this unit.
5. **Speed under load is not settled by the panel reads.** The load average was 13 to 35 during the reads (system media analysis, indexing and browsers, not this unit), and seed 2's two S-pair reads ran at 177 and 336 served tokens a second where every other read ran at 548 to 1,123. The re-timed pairs below are the speed reading.
6. **The row values do not meet D85's default path.** They were trained in F32 by claude-370 and rounded once to Q12. The rounding is at most half a step (1.221e-4) and the served replies agree with the float replies about as often with the rows as without them (below), but this is historical float/export evidence: the values are not trained in the served representation.

## What changed

- **One shared operator for the picks** (`crates/uor-r4-integer/src/fbs_pair_rows.rs`, new, declared in both integer crate roots). The prime address of a token pair, the dictionary (ascending u32 addresses, checked), the marks (user marker and assistant header ids), the turn scan, `open_picks` (the rows of the open turn's pairs, sorted and unique, or none when no turn is open or the turn has no rows), the streaming `PairTracker` the served session feeds token by token, the Q12 rounding (`quantize_q12`, refusing any value outside i16) and the plain-data `PairRowsImage` with its rows file (magic, sizes, order and digest checked). The float trainer's `pair_rows.rs` now calls this module for every pick and every address; it has no picks of its own.
- **Snapshot** (`fbs-snapshot`). A checkpoint with pair rows is no longer refused. Weighed or boosted rows (claude-373, claude-374) and rows beside address rows are still refused. The rows are rounded to Q12 into `{out}.pair-rows`, with a census line and a note in the snapshot.
- **Artifact** (`fbs-export --pair-rows ROWS`). The rows file is added as a UORLUT01 table under `lut-stack/12` (`add_pair_rows_uorlut01`), so a reader from before refuses the artifact by schema instead of serving it without its rows. The flag needs the snapshot's note and refuses an address table beside it.
- **Served read** (`stack/pair_rows.rs`, `stack/session.rs`). The model loads the rows from the artifact; the session carries a `PairTracker`, reset and restored with the rest of its state. At layer 0, after the Hamilton block and any address rows, before the Hamming read, the bag (the sum of the picked i16 rows, shifted by 4 into the i32 residual at 2^-16) is added at every step whose turn is open with picks. Both the plain and the timed fbs steps do this.
- **Replay instrument.** `pair-picks` (float crate, new) writes the trainer's plan for each panel prompt from the checkpoint's own dictionary. `raw-id-replies --pair-rows 0|1` names each reply's picks and addresses, counts reply steps whose picks moved, and drops the rows with 0.
- **Lockfile.** The instrument's `Cargo.lock` gains `uor-r4-native-vector`, which `main` gave the integer crate without refreshing it.

## Measurements

**Models and snapshots.**
- claude-370's four step-16,384 checkpoints on `wick` (`/workspace/out/c370/try-1/{pair-s2,pair-s3,b-s2,b-s3}`).
- Snapshotted on `wick`'s RTX 4090 by `fbs-snapshot` built at `bd66ff21d3b8b2db0f82e65a9b54859f55d11f2b` (binary `e7e99bb3…6d74`), an export step beside the V1 session's run. Census:
  - pair-s2: largest |value| 2.246155, largest rounding 1.221e-4, 344,040 zeros of 47,977,728; rows file `5d35efc6…2e37`;
  - pair-s3: largest |value| 2.150860, largest rounding 1.221e-4, 342,295 zeros; rows file `210d6cbb…a8a6`.
- Exported on the laptop with `fbs-export --legacy-artifact --raw-gates --raw-transition --run-config <snap>/config.json --context 8256`, the pair models with `--pair-rows`. SHA-256 and bytes:
  - pair-s2 `8f804dcc9f8504958308818a0940d4591aeb76a0ac88e9bf0f6313df77580db6`, 116,050,880;
  - pair-s3 `5f6e95f5c35cc736ba06b0ac231f0dc2f64a961bf79ad4e80e078ca506dbfd7e`, 116,050,880;
  - b-s2 `6ee0efe59c1276715140a4ebf2a10e8efe0813b41a8fb8e2be817e66e9bb2469`, 19,345,344;
  - b-s3 `793a30244e1b53eb63e2b8bbd6a085ff56f1c855a0d553fbb87ae275ad2992e9`, 19,345,344.

**Panels.** P14 (1,108 rows) and P23 (1,110 rows), claude-374's `samples-p-greedy-memory.json` (`33bdd07a…178d`, `ddccde6f…7d99`), identity hashes `6944401c8b70fac2` and `b40ce94f0ce85fff`, checked by `rows-384.sh` before scoring. Every row served greedy, 48 tokens, `--mark-reply 1`, turn memory as exported, no asks table, no serve-again, no pair experts, `--time-steps 1`, one process at a time.

**The replay (instrument bar, passed).** `pair-picks` on `wick` against `raw-id-replies` on the laptop, prompt by prompt:

| run | panel | prompts | rows agree | addresses agree | with picks |
|---|---|---|---|---|---|
| pair-s2 | P14 | 1,108 | 1,108 | 1,108 | 1,108 |
| pair-s2 | P23 | 1,110 | 1,110 | 1,110 | 1,110 |
| pair-s3 | P14 | 1,108 | 1,108 | 1,108 | 1,108 |
| pair-s3 | P23 | 1,110 | 1,110 | 1,110 | 1,110 |

In every served read the picks held from a reply's first id to its end (`picks_moved` 0).

**The export gap.** Served replies equal to claude-370's float scoring-pass replies, id for id (the float reply cut after its first end mark):

| model | P14 | P23 | of 2,218 |
|---|---|---|---|
| pair-s2 | 961 | 959 | 1,920 |
| pair-s3 | 968 | 988 | 1,956 |
| b-s2 | 943 | 953 | 1,896 |
| b-s3 | 907 | 927 | 1,834 |

The artifacts with rows agree with float at least as often as the ones without (87–88% against 83–85%). The Q12 rounding of the rows adds no gap that this count can see beyond the trunk's own export gap.

**The number.** Exact is `read-374.sh`'s rule; fact and number rows are `subst-374.sh`'s, run as `subst-384.sh` with only the run names changed.

| model | setting | exact of 2,218 | ended | fact and number rows, exact / answer word, of 188 |
|---|---|---|---|---|
| pair-s2 | S-pair | 524 | 2,058 | 43 / 85 |
| b-s2 | S-b | 423 | 1,960 | 33 / 69 |
| pair-s2 | S-cut | 322 | 1,817 | 23 / 49 |
| pair-s3 | S-pair | 517 | 2,038 | 46 / 86 |
| b-s3 | S-b | 409 | 2,048 | 27 / 56 |
| pair-s3 | S-cut | 316 | 1,593 | 20 / 43 |

Gains: +101 (s2) and +108 (s3); G_s = 104.5 / 2,218 = **4.71 points**, each seed above zero: **Served**. Float (claude-370): +96 and +108, 4.60 points.

Rows won and lost, S-pair against S-b of the same seed: seed 2 won 66 + 67 and lost 18 + 14 (P14 + P23); seed 3 won 78 + 64, lost 13 + 21. Most won rows are "Say exactly this", "Copy these words" and "Please repeat" rows said back in full, as in float.

**Fact and number rows by kind** (P14 + P23, exact / ended with the answer word):

| kind | of | pair-s2 | pair-s3 | b-s2 | b-s3 | cut-s2 | cut-s3 |
|---|---|---|---|---|---|---|---|
| capital | 34 | 0/5 | 1/4 | 1/6 | 0/2 | 0/0 | 0/2 |
| colour | 19 | 8/13 | 8/13 | 5/9 | 4/8 | 4/8 | 5/9 |
| day after | 28 | 3/9 | 5/12 | 3/9 | 0/1 | 0/1 | 0/1 |
| legs | 29 | 11/27 | 11/27 | 4/16 | 8/19 | 4/14 | 0/6 |
| opposite | 6 | 5/5 | 5/5 | 5/5 | 0/0 | 0/0 | 0/0 |
| sound | 28 | 6/11 | 6/12 | 6/12 | 6/13 | 6/14 | 6/13 |
| add | 21 | 0/1 | 0/0 | 0/0 | 0/1 | 0/1 | 0/1 |
| bigger | 23 | 10/14 | 10/13 | 9/12 | 9/12 | 9/11 | 9/11 |

**The conversation** (`uor-chat --stack ART --tokenizer <claude-370's> --script claude-356/chat-script.json`, pair experts off, turn memory as exported; `uor-chat` `4e19b04c…6603`):

| turn | user | pair-s2 | b-s2 | pair-s3 | b-s3 |
|---|---|---|---|---|---|
| 1 | Keep these in mind: druvalk is 73, mibtorn is felsharp, quonzet is 18. | Got it. | Got it. | Got it. | Got it. |
| 2 | What is mibtorn? | Mibtorn is felsharp. | Mibtorn?? | Mibtorn is felsharp. | Mibtorn is felsharp. |
| 3 | What does a cow say? | A cow says moo. | A cow says moo. | A cow says moo. | A cow says moo. |
| 4 | Say exactly this: trelvon dask pimmerholt zane | trelvon dask pimmerholt zane | the line, then "Assistant: trelvon dask pimmerholt zane" six more times to the token limit | trelvon dask pimmerholt zane | trelvon dask pim |
| 5 | What is druvalk? | That's 66. | Your cake 18. | That's 666 (short cycle) | That's 65. |
| 6 | I'm tired today. | I'm tired today. | I'm tired today. | That's 66 / User: O spellucuc remember remember remember (short cycle) | I'm tired today. |
| 7 | Tell me what quonzet was. | It's 18. | It's 18. | That's 66 / User: Tell me what quonzet is 18. | It's 18. |
| 8 | What is two plus three? | 18. | That's 18. | 18. | Hibtorn is felsharp. |
| 9 | What is my name? | You're welcome! | You're welcome! | You're welcome! | You're welcome! |

The rows' gain shows in the conversation where the panel shows it: the repeat turn is said back exactly by both S-pair seeds and by neither S-b seed. Values and arithmetic are wrong in all four.

**Speed** (`step_time` from the receipts; 124,113 prompt ids on P14 and 126,946 on P23 a run, 15,165 to 23,928 reply tokens; load average at the start of each read):

| model | setting | P14 prompt ids/s | P14 served tok/s | P23 prompt ids/s | P23 served tok/s | load |
|---|---|---|---|---|---|---|
| pair-s2 | S-pair | 564 | 177 | 970 | 336 | 34.7, 30.0 |
| b-s2 | S-b | 1,589 | 575 | 1,901 | 698 | 25.1, 21.3 |
| pair-s2 | S-cut | 2,009 | 668 | 2,633 | 898 | 14.0, 14.4 |
| pair-s3 | S-pair | 2,868 | 1,085 | 2,497 | 904 | 14.8, 13.1 |
| b-s3 | S-b | 2,547 | 947 | 2,874 | 1,123 | 11.8, 15.5 |
| pair-s3 | S-cut | 2,137 | 708 | 1,516 | 548 | 12.6, 17.8 |

**Re-timed: seed 2, P14, S-pair and S-b alternated twice, after the reads** (every re-timed reply equals the read's, 1,108 of 1,108 in each run):

| run | prompt ids/s | served tok/s | load |
|---|---|---|---|
| S-pair, 1st | 1,805 | 674 | 23.7 |
| S-b, 1st | 2,553 | 880 | 19.0 |
| S-pair, 2nd | 2,121 | 764 | 14.3 |
| S-b, 2nd | 1,669 | 542 | 17.6 |

At this machine's load (the laptop was never below 11) the runs of one setting differ by more than the settings do, in both directions. This unit does not settle what the rows cost a second; it does show the reads are reproducible id for id. The cost per reply step is one gather and sum of the picked rows (a reply's picks are fixed for the whole reply), against claude-376's 1,450 to 1,600 served tokens a second at a load of about 5.

## Checks run at the head

Run on the laptop at the tree merged with `origin/main` (`cb91a1baa`), after the reads (`local/claude/claude-384/gates-head.sh`, log beside it):

| Check | Result |
|---|---|
| `cargo test -p uor-r4-integer --features serving-wasm --lib -- address pair_rows` | 34 passed |
| `cargo test -p uor-r4-integer --lib -- pair_rows address` (the default build) | 21 passed |
| `cargo test -p uor-r4-integer --features serving-wasm --bin fbs-export` | 6 passed |
| `cargo test -p uor-r4-float-train --lib -- pair_rows fbs_export` | 40 passed |
| `cargo check -p uor-r4-float-train --bin fbs-snapshot --bin pair-picks` | builds |
| `cargo check -p uor-r4-training -p uor-r4-cpu-train` (the integer crate's dependents) | builds |
| `cargo test --release --locked --manifest-path docs/labs/one-model-end-to-end/instrument/Cargo.toml --features integer-reader --bin raw-id-replies` | 19 passed |
| `rustfmt --edition 2021 --check` on the four new files | clean (`pair-picks.rs` formatted at this step; no behaviour change) |
| Negative gates, run when the served read was built (`bd66ff21d`): the layer-0 bag call taken out of the session; then the pair tracker's reset taken out of `restore_state` | each fails a `stack/tests/pair_rows.rs` test (the served-step test; the same-length restore in the picks test); restored, all pass |
| The replay (above) | 4,436 of 4,436 |
| Panel identity hashes | P14 `6944401c8b70fac2`, P23 `b40ce94f0ce85fff`, claude-374's |
| Every read's exit | 0 (12 of 12, and 4 of 4 re-timed) |

Binaries that served and exported, built at `44a07a527eec6fe12e3f3d3af5b3ea62b0ead58f` (the merge with `main` changed nothing in the integer crate or the instrument they build from): `raw-id-replies` `874139a95c2993cb57927838cdfbcfc436a2834f612413988ca3c5f2b50086a6`, `fbs-export` `5af2cc461ec60b4548843681fc1b13df9bbeb68dda113db1dbb870a097c16ed7`, `uor-chat` `4e19b04cbd7e59026ce8dc62d1fd5503fdaa24735b32a8d44d992f453d566603`. On `wick`, at `bd66ff21d`: `fbs-snapshot` `e7e99bb3…6d74`, `pair-picks` `95fcb98b…c568c343`.

## Skills used

None. The kit is `local/claude/claude-384/`: `snap-384.sh` (the `wick` job), `export-384.sh`, `serve-384.sh`, `replay-384.sh`, `rows-384.sh` with `subst-384.sh` (claude-374's, run names changed), `retime-384.sh` and `chat-384.sh`.

## Still open

- **Compression, the next unit.** The rows are 96.7 MB of a 116 MB artifact. Rows shared by the pair's E8 address, or a higher MIN than 2 turns, would cut it; each needs its own float training and the same served bar.
- **The values in the served representation.** The rows are trained in F32 and rounded once; a D85 unit would train them in Q12 in the shared space.
- **Conversations that break after a cycle.** Seed 3's S-pair keeps a cycled reply in the conversation and breaks the next two turns; the panel reads each row from a clean window and cannot see this.
- **The packet-parity bin.** `cargo check --tests` on the float crate fails at base on the `packet-parity` bin, which needs `uor_r4_cpu_train`; the checks above build `--lib` and the named bins instead. Not this unit's to fix.

## Plain summary

The model claude-370 trained in float learns a table with one row for each pair of words that appears together in users' messages. When the user writes, the rows for every pair of their words are added into the model's thinking. That table made the float model about 4.6 points better at giving exactly the expected reply. This unit puts the table into the small integer file that runs on the laptop, so the laptop model reads it too, and checks that the laptop picks exactly the same rows as the trainer did for every one of 4,436 test prompts. On the laptop the model gets 524 and 517 of 2,218 replies exactly right with the table, against 423 and 409 without it: the same gain as in float. Most of the gain is the model repeating the user's words back correctly. Facts like capitals and arithmetic did not improve, and the file is six times larger, which is what the next step has to fix.
