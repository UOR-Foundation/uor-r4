# REPORT: integer-nearest-readout, claude-252 (Q5's trace instrument, side T)

Lab: claude. Item: [integer-nearest-readout](../../../../roadmap/items/integer-nearest-readout.md) (#3462). Order: [ORDER.md](ORDER.md). Rule: claude-247's [decision card](../claude-247/DECISION.md). References #3462.

## Tokens per second

No decode speed is claimed, and no served path changes. R's decode speed is still claude-243's: 1,204 tok/s median decode on R s1.
- On the GPU, F's command on T's s1 copy took 325 s with the trace on and 325 s with it off, side by side on one RTX 5090. Both processes ran at once, so this is an instrument's cost, not a serving reading. The trace covered 308 DEV windows, 36,541 steps.
- The copy itself runs on the CPU, about 1 s per seed. Each snapshot and each export took about 1 s.

## What did not work, first

- **Q5 is not read, and nothing here moves done-when 3.** This unit builds T's side of the trace. The replay driver, the near-tie grader and the sheet come next, in that order.
- **No snapshot equals claude-227's, the one R was exported from;** the check holds on the artifact instead.
  - This head's `fbs-snapshot` adds a `readout_nearest` field that claude-227's snapshots lack.
  - The copy's snapshot differs from the as-trained one at every seed (sha256). A `cmp` over ssh at s1 put the first difference at about byte 100, where the rounded floats begin; its output was not saved, so the sha256s are the record.
  - All nine exports (three per seed) still equal R's sealed artifacts. So I-Twin is read on the artifact's sha256, as the card defines it, not on the snapshot.
- **The agreements below are readings, not Q5.** They come from 308 DEV windows outside Q5's set, with nothing forced, over the first scored rows only.
  - By the card, attribution alone accounts for nothing: whether a flip is a near tie or a fault is the replay driver's and the grader's to decide.
  - No row of the set was traced. The positions are F's smoke's (`960f4340…`).
- **The smoke ran F's command with `--samples 1`, as F's did.** `score_rows` writes every rows file before it decodes a sample.
- **The ORDER named `flock_read_memory.rs` for the weights' accessors; none was needed.** The opening and reply weights are `pub(crate)` fields (`flock_read_memory.rs:153`, `:172`), which the copy reads and sets directly. The file is unchanged.
- **I-Twin alone cannot tell a rounded copy from an unrounded one.** The export rounds as it writes, and rounding twice changes nothing, so *orig*, the as-trained step with nothing rounded, exports to R's artifact as well, at every seed. A copy that rounded nothing, or only some groups, would pass the export check too. That T's values are R's rests on I-Twin together with the copy's own read-back, as set out under "What worked".
- **Review round 1 changed the copy and the side after the pod ran.** The pod ran `1590b6011`; `45ad04af1` changes the copy's refusal paths and adds a check to the side:
  - the staging folder is claimed with one atomic `create_dir`, and only a claimed folder is removed;
  - the side checks the rounding itself before it answers `T`;
  - the memory's tie test now tells half away from zero from half to even.
  
  The values the copy writes come from code the round did not touch. The host suite, the CUDA type-check and clippy ran again at this head. The CUDA library suite was not rerun: the round's lines have no CUDA configuration, and the host suite runs them.
- **The suites carry failures this unit did not add.**
  - `cargo fmt --check` already fails on the crate's base. Only this unit's lines were formatted, so the base's drift is unchanged in every file it touches (`in_space_long.rs` 320 lines, `fbs_export.rs` 10, `bin/train-in-space.rs` 18, `bin/fbs-snapshot.rs` 0).
  - Clippy with `-D warnings` finds 7 lints in the float crate, all in code this unit does not touch. None is on an added line.
- **No sheet, FIT or pre-run review was written for the pod run.** It checks this unit's own instrument (done-whens 4 and 5), as claude-251's smoke did, and no claim rests on it. The sheet that reads Q5 comes after the replay driver and the grader.
- **The pod tool's provider adapter had to be rebuilt in this worktree.** claude-251's build went with its worktree, and `uor-pod up` refused until `local/pod-provider-target` held it again (logged in `local/claude/FRICTION.md`).
- **The disk was under the 30 GiB floor (22 GiB free).** This unit's build folders are inside its worktree and go with it.

## What worked

- **The copy: `fbs-snapshot --round-as-exported MODEL_DIR OUT_MODEL_DIR`,** the fifth form beside `--rebind-fbs-backward`. It runs on the CPU and writes an evaluation copy of a step in which every parameter the artifact holds is rounded as `fbs-export --raw-gates --raw-transition` rounds it:
  - `model.safetensors`: `round_tables_as_exported` (grid codes, Q16, Q12);
  - `hamming_read.safetensors`: `round_hamming_as_exported` (rotations 2^−24, margins 2^−16, weights 2^−24);
  - `turn_memory.safetensors` and `turn_memory_reply.safetensors`: the opening and reply weights at 2^−24, by the new `round_memory_as_exported`. The keys and the tables are exact and are copied unchanged;
  - the step's `config.json` and the run's config, byte for byte;
  - last, a marker, `rounded-as-exported.json` (`uor-r4.rounded-as-exported/1`), with each file's source and copy sha256 and its rounding.
- **The copy refuses** an existing output or staging folder, another step's name, a step file it does not know, and whatever the export refuses (a NaN, a value past i32 at its step, a folded run).
  - It claims its staging folder with one atomic `create_dir`, so two copies never share one, and it removes only a folder it claimed.
  - It is written there, reloaded with every check and compared bit for bit with the rounded tensors, then renamed into place.
  - So no refusal leaves anything behind: the tests' refusals after the claim check that their staging folder is gone (gate n2).
- **The side.** `score_rows` writes side `T` when the checkpoint holds the marker, every file it names still has its recorded sha256, and its parameters are their own export rounding: the trunk's tables, and the reads and memory unchanged by re-rounding. It writes side `F` when the checkpoint holds no marker. A copy that fails any of these is refused, including one under a hand-written marker over unrounded files (gate n3). So T's command is F's, unchanged.
  - The rounding check is review round 1's; at the pod's `1590b6011`, the side checked the files against the marker only.
  - The host tests pin the side's lookup and the trace's `on_side`. The pod smoke pins that `score_rows` joins them: its trace reads `T`.
- **I-Twin holds at seeds 1–3,** on one RTX 5090 at `1590b6011`. The path is the copy, then `fbs-snapshot COPY SNAP` with no readouts, then `fbs-export --raw-gates --raw-transition --run-config CONFIG SNAP OUT`. It gives R's sealed artifact byte for byte at each seed: `f86a7856…`, `0a69a009…` and `66ae49de…`.
  - **I-Twin, together with the copy's read-back, makes T's values R's.**
    - I-Twin gives the export of T as R.
    - The copy's own read-back on the pod gives T as its own rounding. At each seed `copy-sN.out` ends: "the copy reads back as rounded, and re-rounding it changes nothing".
    - The export of a copy that is its own rounding holds that copy's values, so T's values are R's.
    - I-Twin alone would not show this (above).
  - **The host tests pin the read-back.** The copy holds every group's rounding bit for bit, every parameter moved, and setting the memory's rounding aside makes the copy refuse itself (gates m1, m2).
  - **Two signs that the copy changed values.** Its snapshots differ from orig's at every seed by sha256 (s1 `3c0f3394…` against `8844a04e…`, s2 `799502b0…` against `e42e3146…`, s3 `5e17e783…` against `bd6492a9…`). (A `cmp` over ssh at s1 found the first difference at byte 100; that output was not saved.) And T's picks differ from F's on 190 rows.
  - **Two controls ran in the same pass,** each also equal to R's artifact at every seed. Had I-Twin failed, they would have located the fault:
    - *rerun*, claude-227's snapshot exported again by this build. It places a fault in the export;
    - *orig*, the as-trained step snapshotted at this head, then exported. It places a fault in the snapshot.
  - **The copy's run config** is byte-identical to the checkpoint's at every seed. `copy-sN.out` gives the copy's sha256 equal to the source's, and the job's `cmp` would have stopped it with exit 5 on a difference (it logs nothing when equal; the job reached `EXIT 0`).
- **T's smoke passes,** outside Q5's set. F's command, unchanged, ran on T's s1 copy and traced DEV s1's 308 windows of R's smoke:
  - **the side:** the trace's side is `T`;
  - **it changes nothing:** the six rows files are equal byte for byte with the trace on and off;
  - **the alignment:** T's windows join F's exactly (same position, `reply_start`, ids, and nearest and memory steps) and R's exactly (same position and `reply_start`; R's ids a prefix of T's; the same nearest and memory steps across R's range). All 308 windows hold.
- **The reading (no bar):** over R's range, T's pick equals R's on 4,380 of 4,382 scored rows, where F's equals R's on 4,298. Over F's whole range, T's equals F's on 10,650 of 10,840.
  - **Row by row** (`cross.jq`), over R's 4,382 rows:

    | F = R | T = R | T = F | Rows |
    |---|---|---|---|
    | yes | yes | yes | 4,297 |
    | no | yes | no | 83 |
    | no | no | no | 1 |
    | yes | no | no | 1 |

  - **In the card's terms:**
    - Of the 84 rows where F and R differ, the structure step (F to T, rounding the parameters) moves 83 to R's pick. On the 84th, all three differ.
    - On one row where F and R agree, T differs from both: the structure step moves the pick away, and the arithmetic step (T to R) brings it back.
  - Which of these flips are near ties is still unread.

## What changed

- **`crates/uor-r4-float-train/src/fbs_export.rs`:**
  - `MEMORY_WEIGHT_EXP` (−24, checked against the integer format's constant) and `round_memory_as_exported`;
  - the copy, `write_rounded_copy` (with `copy_into` after its staging claim), its marker (`RoundedCopy`, `CopiedFile`), `checkpoint_trace_side` and its rounding check `check_rounded_as_exported`;
  - two host tests: `rounding_the_turn_memory_holds_the_exported_weights` and `a_copy_rounded_as_exported_holds_every_rounding_and_is_side_t`. `the_hamming_exponents_are_the_integer_formats` also checks the memory's exponent. Since review round 1, the copy test forges a marker over an unrounded trunk and over an unrounded memory, and the memory test's tie is −2.5 steps.
- **`crates/uor-r4-float-train/src/bin/fbs-snapshot.rs`:** the fifth form, `--round-as-exported`.
- **`crates/uor-r4-float-train/src/in_space_long.rs`:** `FTrace::on_side`, and `score_rows` reading the side from the checkpoint. One host test: `a_float_trace_is_side_f_or_t`.
- **`crates/uor-r4-float-train/src/bin/train-in-space.rs`:** the `--decision-trace` docs name side T.
- **The item's "What is missing"** marks T's side built and names the replay driver next.

## Measurements

The pod run used the binaries `fbs-snapshot` `31666fcc…`, `train-in-space` `79636161…` and `fbs-export` `6dc37d87…`, built at `1590b6011` on the pod.

| What | Compared | Result |
|---|---|---|
| I-Twin | the copy's snapshot, exported raw, against R's sealed `a7r-sN` (sha256) | 3 of 3 equal |
| Control: rerun | claude-227's snapshot `sN`, exported again by this build | 3 of 3 equal |
| Control: orig | this head's snapshot of the as-trained step, exported | 3 of 3 equal |
| Snapshots | orig and the copy's, each against claude-227's; the copy's against orig, by sha256 in `SHA256SUMS` | all differ (a new field; the rounded floats) |
| The copy's run config | sha256 against the checkpoint's in `copy-sN.out`; the job's `cmp`, which stops it on a difference | 3 of 3 equal |
| T's smoke: side | the trace's `side` | `T` |
| T's smoke: on = off | `cmp`, the six rows files, trace on against off | 6 equal |
| Alignment with F | position, `reply_start`, ids, nearest and memory steps, against F's smoke summary (claude-251, round 2) | 308 of 308 windows |
| Alignment with R | position, `reply_start`, ids prefix, nearest and memory steps over R's range, against R's smoke trace (`11e6b7d9…`) | 308 of 308 windows |
| Reading: T's pick = R's | R's range, the steps with a nearest | 4,380 of 4,382 |
| Reading: F's pick = R's | the same rows | 4,298 of 4,382 |
| Reading: T's pick = F's | F's range | 10,650 of 10,840 |
| Trace size | DEV s1, R's 308 windows | 36,541 steps; 621 MB JSON (`b1970581…`), 208 MB gzipped (`ca52fa85…`) |
| Wall time | the two smoke runs side by side on one GPU | traced 325 s, untraced 325 s |
| Pod | 1 × RTX 5090 at $0.563/h | 20 minutes, about $0.19 |

The alignment and the readings come from `local/claude/claude-252/align.jq`. It was first probed with F's summary in T's place, giving 308 of 308 windows and 10,840 of 10,840 picks, F's own count. It was then probed with R's summary in T's place, where `t_vs_f` falls to 69 of 308, so the check can fail.

The evidence is sealed in the HF store:
- **`results/claude/integer-nearest-readout-trace-t-2026-11-02/`:** 164 files under a manifest (sha256 `5a9891c5…`), read back 164 of 164 equal. It holds:
  - the pod's logs and sums, and the export verdicts;
  - the traced and untraced rows, and T's summary;
  - the alignment and cross-table checks with their outputs, and R's summary;
  - the job and its pins;
  - both rounds' negative gates, with their patches and logs;
  - review round 1's host-suite, CUDA-check and clippy logs.
- **The pod's archive:** `vast-archive/vast-58748616/uor-r4/runs/claude-252/`, 1,607 MB, read back 171 files. It holds the three copies, the six snapshots, the nine artifacts, the opening tables and the gzipped trace. The uncompressed trace was removed on the pod first, once `gunzip -c` hashed to its `SHA256SUMS` line.

## Checks run at the head

| Gate | Command | Result |
|---|---|---|
| Host tests (float, `45ad04af1`) | `cargo test --offline --locked -p uor-r4-float-train --no-fail-fast`: the library, its bins and integration targets | 200 passed in the library; every bin and integration target passes |
| CUDA type-check (`45ad04af1`) | `CUDARC_CUDA_VERSION=12080 NVCC=<stub> UOR_NVCC_ARCH=compute_120 cargo check --offline --locked --features cuda --all-targets -p uor-r4-float-train` | exit 0 |
| Clippy, lines this unit adds (`45ad04af1`) | `cargo clippy --no-deps --features cuda --all-targets -p uor-r4-float-train -- -D warnings`, stub nvcc | 0 findings on added lines; 7 in code this unit does not touch, each line's blame older than the branch |
| CUDA library suite (RTX 5090, `1590b6011`) | `cargo test --release --locked -p uor-r4-float-train --features cuda --lib -- --test-threads=1` | 275 passed, 0 failed, 6 ignored; this unit's three tests and claude-251's trace tests pass by name. Not rerun after review round 1 (above) |

### Negative gates

Each mutation sets aside one piece of this unit's own lines with `sed`, is saved as a patch against the head (`1590b6011`), has its test run, and is reversed with `git apply -R`; the tree then matched the head again (`local/claude/claude-252/neg/gates.sh`, its logs and patches beside it). Every test failed at an assertion or a refusal, not a build, and the head then passed all three tests.

| Gate | Piece set aside | Mutation | Test | Result |
|---|---|---|---|---|
| m1 | The reply weight's rounding | the reply table left out of the rounded weights | `rounding_the_turn_memory_holds_the_exported_weights` | FAILED (`fbs_export.rs:1507`) |
| m2 | The copy rounds the memory | `round_memory_as_exported` not called | `a_copy_rounded_as_exported_holds_every_rounding_and_is_side_t` | FAILED: the copy refuses itself at its read-back check |
| m3 | A marked copy is side T | `checkpoint_trace_side` returning `F` | the same | FAILED (`:1636`) |
| m4 | The marker's sha256s | the file check skipped | the same | FAILED: a tampered `model.safetensors` still read as side T (`:1765`) |
| m5 | An unknown step file is refused | the name check removed | the same | FAILED: `transport.json` was refused only later, by the loader's sidecar check, not by the copy's own (`:1704`) |
| m6 | An existing staging folder is refused | only the output checked | the same | FAILED: the copy was written over an existing `v.partial` (`:1703`) |
| m7 | `on_side` sets the side | the side not set | `a_float_trace_is_side_f_or_t` | FAILED (`in_space_long.rs:9576`) |

After review round 1, `local/claude/claude-252/neg/gates2.sh` set aside each of the round's fixes the same way, against `45ad04af1`. Each test failed at an assertion, and the head then passed both tests.

| Gate | Piece set aside | Mutation | Test | Result |
|---|---|---|---|---|
| n1 | The staging folder is claimed | `create_dir_all`, which takes a folder that exists | `a_copy_rounded_as_exported_holds_every_rounding_and_is_side_t` | FAILED: the copy was written over an existing `v.partial` |
| n2 | A refusal after the claim removes the staging folder | the removal skipped | the same | FAILED: `x.partial` was left (`!left("x")`) |
| n3 | The side checks the rounding itself | the check skipped | the same | FAILED: a forged marker over the unrounded trunk read as side T |
| n4 | The memory rounds half away from 0 | half to even | `rounding_the_turn_memory_holds_the_exported_weights` | FAILED: −2.5 steps went to −2, not −3 |

## Skills used

None.

## Still open

- **Next, under the same card: the replay driver and the near-tie grader, then the sheet** with the card's rule verbatim, a fresh FIT and a pre-run review, then the GPU run that reads Q5.
- **The rows the grader starts from.** In R's range, T and R differ on 2 rows and F and T on 85 (83 + 1 + 1 in the table above); over F's whole range, F and T differ on 190. The replay driver forces each first differing decision in forward order, and the grader judges each flip by G₁₀ on the reference side (F for structure, T for arithmetic).
- **What review round 1 found and left (all LOW).**
  - A memory weight whose width is not the trunk's is copied, and refused only at `fbs-export`.
  - The source's sha256s are taken after it was loaded, so a source changed in between would be recorded by its new hash.
  - `run_of` takes the step's grandparent without checking that its parent is `models`, so a nested output inside the source step would be accepted.
  - Nothing is synced to disk before the marker and the rename. A crash would leave a copy the side refuses, not a wrong `T`.
  - Between the second `OUT_RUN` check and the rename, an empty folder made there would still be replaced. That second check is read, not tested: no test can make the folder appear between the copy's two checks.
- **What review round 2 found and left (all LOW).**
  - The staging claim now comes before the step is read, so a refused copy can leave empty parent folders it made for `OUT_RUN` (never anything inside them).
  - The forged-marker test covers the trunk and the memory, not `hamming_read.safetensors`. The side's check reaches the reads through the same re-rounding comparison.
- **Carried from claude-251 and claude-249:** the open items of their "Still open", unchanged.

## Plain summary

To find out why the integer model and the trained model sometimes pick different words, claude-247's rule puts a third model between them: the trained model running on the integer model's own rounded numbers. This unit builds that third model.

It is a copy of the trained checkpoint with every number rounded exactly as the integer export rounds it. As it is written, the copy checks that rounding it again changes nothing. Exported again, it gives the integer model byte for byte at all three seeds. Together, these show the two hold the same numbers, so only the arithmetic differs. The export alone would not show this: the unrounded checkpoint exports to the same bytes. On 308 test windows, the trained model and the integer model disagree on 84 picks. Running the trained model on the rounded copy gives the integer model's pick on 83 of them. That points at the rounding, not at the integer arithmetic. Deciding whether each one is an honest near tie is the next unit's work.
