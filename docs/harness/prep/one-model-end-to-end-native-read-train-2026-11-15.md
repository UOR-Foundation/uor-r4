# Prep sheet: the first native training of the read's gates (one-model-end-to-end, D86 step 2, 15 Nov 2026)

Written by claude-393. The run is the next unit, after this sheet's fresh non-author FIT and that unit's own pre-run review (§5.10). Nothing here runs before both.

## The question

D86 step 2 has one native Q31 read with its operator on host and device:
- the read and its device twin (claude-389);
- serving (claude-390);
- the gate update `Q31FirstPickRead::train_pick_step` (claude-391), with its CUDA twin (claude-392, split by window in claude-393).

No gate of the native read has yet been trained on real states. This run asks one thing: **starting from identity gates, does the native integer rule teach the read to pick the asked fact on held-out queries, on the served states of a trunk whose float read already picks it?** If it does, does the hard copy at the ×16 gain then answer right?

The trunk stays the float trunk: only the read's gates train, in Q31, on the GPU. Training and serving use the same operator and the same state, which is the D85 native default for the read. The trunk itself is D86 step 6's.

## Steps: the arms

**Task.** claude-386's two-class recall, B2 form, unchanged:
- `--recall 4 --recall-query 1 --recall-header 4`;
- 4 queries a window, each scored at `H4`;
- far buckets 1 (33–96) and 2 (97–180).

At a query row, the one right pick is the asked relation's position `p − 1`, where `p` is the asked value's position. So the row's target is `p` (the rule's `s + 1` form), and every other row's target is 0. These are the same rows and the same definition as claude-386's `chose_target`.

**States** (on wick, as claude-389 used them):
- **ext-s1, ext-s2:** claude-386's Flock-g read, trunk and read trained to 16,384 steps. The float read's first pick is right on 1,000 and 990 of 1,024 held-out queries (claude-388).
- **4k-s1, 4k-s2:** the same arm at 4,096 steps. The float first pick is right on 418 and 474 of 1,024.

Each state is converted to Q31 by claude-389's rule. That gives the stream at the read, the table, and the weights at gain shift `g = 4` (×16). The weights and the gain are fixed in this run; only the gates train.

**The rule.**
- `train_pick_step` on the device (`NativeVectorEngine::read_train_pick_step`), with margin 0 (the rule's own minimum of 1 against a later rival still applies) and the strict-drop accept.
- A batch is 64 windows, in index order. 2,048 train windows make 32 batches an epoch; 16 epochs make **512 steps**.
- Step shift grid `{4, 8, 12}` for each arm. The shift whose final **train** loss is lowest is the arm's reading. All three are reported. The held-out split never chooses.

| Arm | States | Start gates | Targets | Runs |
|---|---|---|---|---|
| **A-ext** | ext-s1, ext-s2 | identity | asked fact | 2 states × 3 shifts |
| **A-4k** | 4k-s1, 4k-s2 | identity | asked fact | 2 × 3 |
| **C0** (untrained) | all four | identity | none | step 0 of each A run |
| **C1** (float reference) | all four | claude-389's converted float gates | none | one evaluation each |
| **C2** (random targets) | ext-s1, ext-s2 | identity | a seeded uniform earlier row at each query row | at A-ext's chosen shift |
| **C3** (warm start, descriptive) | ext-s1, ext-s2 | C1's gates | asked fact | at A-ext's chosen shift |
| **Replay** | ext-s1 | identity | asked fact | A-ext s1 again at its chosen shift |

**Readings, at steps 0, 64, 128, 256, 384 and 512, on the 256 held-out windows:**
- per far bucket, `chose_target` and `chose_other`: the native first pick lands on the asked relation's position, or on the other fact's;
- the same two counts on the train split;
- the train loss, and the accepted-step count;
- at step 512 only, `R_ask` and `R_oth`: the hard copy at ×16 with the native picks.

**Built in the run unit, before any arm** (none of it exists yet; the pre-run review checks each):
1. **TRAIN dumps.** The probe's native-replay pass (`recall-payload-probe --first-only --native-replay --native-dump`) gains a split option. It writes the same `Q31ReadDump` for train windows `0..2048` of the run's own seed, and beside each dump a targets file: little-endian `u32`, one per row, with `ask_value_pos` at each query's `scored_offset` and 0 elsewhere. The other fact's targets are written the same way, for `chose_other`.
2. **A native trainer** (`uor-r4-discrete` example, Rust). It reads the dumps and targets, starts from identity gates or from a saved read, runs the device step, writes each reading as JSON lines and saves the final read with `Q31FirstPickRead::to_bytes`.
3. **The injected-pick decode.** The probe gains `--native-picks FILE`. The hard-copy pass copies the payload of the native read's pick, not the float read's, at ×16. The decode is then the trunk's own, `final_norm` and the nearest table point, which is float diagnostic code (D85) that reads a natively chosen state.

## Bar (pre-registered), at step 512, held-out, each far bucket (512 rows), both ext states

A-ext passes when all five hold:
1. `chose_target >= 0.90` (461 of 512 or more).
2. `chose_target >= C0's + 0.10` on the same rows.
3. `R_ask >= 0.90` and `R_ask >= R_oth + 0.25`, from the native picks at ×16.
4. **C2 control:** C2's `chose_target` is at most C0's + 0.05. If random targets also raise the asked pick, the gain is not the targets' (outcome 6).
5. **Replay:** the replay arm's final read is byte-identical to A-ext s1's. The host rule (`read_train_pick_step_reference`) gives the same `Q31ReadTrainStep` as the device on A-ext s1's first two steps.

**A-4k (learning beyond the float read), descriptive with one pre-registered reading.** Per bucket, the paired difference of native `chose_target` against C1's float count over the same rows. "Beyond the float read" means at least 0.10 above C1 in both buckets at both states.

**C3** is descriptive: does the native rule keep, raise or lower C1's pick count from a warm start?

## Outcome map

0. **Instrument faults.** Any fault means repair and rerun, before any reading is called.
   - C1's native picks differ from claude-389's on any held-out row (claude-389: 1,024 of 1,024 agree with the float first pick).
   - C1's injected-pick `R_ask` differs from claude-388's ×16 counts (0.984 / 0.969 at ext-s1, 0.969 / 0.965 at ext-s2).
   - A train target is refused.
   - The replay is not byte-identical, or host and device differ on a step.
1. **Pass:** A-ext meets the bar at both states. The read's gates are trained natively. Next:
   - the weights' and gain's own update (claude-391 Still open 3), so that ×16 is trained rather than fixed;
   - then the natively trained read in claude-390's serving path, judged by replies (D86 step 6).
2. **The stream already carries the choice:** C0's `chose_target >= 0.80` in a bucket at an ext state. Bar 2 then fails there, and the ext arm cannot show learning. The A-4k reading decides whether the rule learns. Reported as such, not as a pass.
3. **Partial learning:** `chose_target` rises by at least 0.10 over C0, but stays under 0.90. If it is still rising by more than 0.02 a bucket between steps 384 and 512, extend that arm to 2,048 steps under this sheet. Otherwise it is a configuration result; next is a margin above 0 and a step-size schedule (claude-391 Still open 4), under an addendum.
4. **No learning:** `chose_target` within 0.05 of C0 at every shift, with at least one accepted step. A configuration negative for this margin, shift grid and target set (§4.1), not a mechanism close. Next: margin and the per-batch credit shift (Still open 5).
5. **The rule never accepts:** no accepted step in the first 64 at any shift. First classify the instrument (L-007): run claude-391's fixture through the trainer. A step that also fails there is a trainer fault (outcome 0). Otherwise report every shift's step-0 loss and candidate loss.
6. **Not the targets:** C2 breaks bar 4. Any A-ext pass is withheld, and the REPORT gives both arms' counts.
7. **Picks right but answers wrong:** bars 1 and 2 pass but bar 3 misses. The injected decode is checked against claude-388's first-right decode rate (1,000 of 1,000 and 990 of 990). If that holds, the miss is the trunk's decode at these picks. Reported with the per-bucket counts.
8. **States split:** A-ext passes at one ext state only. Reported as a split, with both states' readings; no pass.
9. **Overfit:** train `chose_target >= 0.90` while held-out is more than 0.10 lower. Reported; next is more train windows under an addendum.

## Data split (generated, none from the store)

- Train windows are `recall_query_window(seed, i, 192, 4)` for `i` in `0..2048`, where `seed` is the state's own run seed. These are windows the trunk itself trained on.
- Held-out windows are claude-386's 256, from `seed ^ RECALL_EVAL_SEED`, which no training ever reads.
- C2's random targets come from a fixed 393 seed and are written in the targets file.

## Base artifact

claude-386's states on wick under `/workspace/in/c387/`, with the `model.safetensors` / `vector_read.safetensors` sha256 recorded in claude-389's ORDER:
- ext-s1: `try-5/ext-flockg-b2-1/run/run/models/step-016384` (`e39cb058…` / `944c759d…`);
- ext-s2: `try-6/ext-flockg-b2-2/run/run/models/step-016384` (`13cfba37…` / `bccbdb23…`);
- 4k-s1: `try-1/arm-flockg-b2-1/run/run/models/step-004096` (`bffad79f…` / `98a1a7c3…`);
- 4k-s2: `try-3/arm-flockg-b2-2/run/run/models/step-004096` (`b00f5adc…` / `1fa08f29…`).

The run unit checks all eight against claude-389's full sha256 before any dump.

## Minimum test power

- Each far bucket holds 512 held-out queries. Near 0.9, one arm's standard deviation is about 0.013, so bar 2's 0.10 over C0 is more than 7 standard deviations.
- C0 and A run on the same rows, so the difference is paired.
- Bar 4's 0.05 is about 4 standard deviations near 0.1.
- The two states are the seeds. The rule has no other randomness: batches go in index order, and every sum is exact.

## Cost

All of it runs on wick (Casey's RTX 4090), so no rental.
- **Steps.** claude-393's timing test puts one device step at this batch (width 256, 64 × 192 rows) at about 17 ms. With the host's copies, one 512-step run is under a minute, and the 17 training runs and 4 evaluations take under 30 minutes.
- **Dumps.** One TRAIN dump per state is about 0.8 GB, so 3.2 GB in all; wick had 43 GB free on 15 Nov. Building them takes the float forward over 2,048 windows a state, minutes each.
- **Total:** about an hour of wick, with the D41 pilot's `llama-server` left alone.

## Results and stop rule

- The readings, the saved reads, the targets files' sha256 and the dumps' sha256 go to `results/claude/one-model-end-to-end-native-read-train/` in the HF store, with a `SHA256SUMS`, uploaded from the laptop with `uor-store`. The dumps themselves stay on wick.
- The job checks outcome 0 (C1 agreement, injected-pick `R_ask`, host against device, replay) before the arms. It stops at any fault.
- No arm stops on its loss: every run takes its 512 steps.
- A completion watcher is armed on every job (L-004).

TEST FITNESS: PENDING (claude-393: a fresh non-author FIT follows in this unit)
