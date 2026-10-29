# Prep sheet: the export gap of the unsnapped D11 artifacts (d11-exporter-float-trunk, claude-182), 2026-10-29

Lab: claude. Item: `d11-exporter-float-trunk` (#2820). ORDER: `docs/loop/tasks/d11-exporter-float-trunk/claude-182/ORDER.md`. Owner direction of 29 Oct: "fix the exporter".

TEST FITNESS: PENDING

## 1. Question

**What came before.**
- **claude-180** read Both off: on the D11 forward in float, no single change brings every model inside the export bar.
- **The snaps-off arm** (`rounding_only`) was inside on both seeds of claude-131's dual recipe, with nets 3 and −9.
- **claude-181** built that change as an integer contract (`raw_gates`, `raw_transition`, `uor-r4.lut-stack/5`), with host parity.
- **This unit's code** gives the GPU evaluator the same forms. The host twin equals the Rust session bit for bit, in all four snap choices.
  - On the synthetic controls: `tests/parity.rs`, `host_twin_equals_the_session_in_every_snap_choice`.
  - On claude-126 seed 1's four exports: `tests/trained.rs`, with no difference at 4,096 single tokens and 10,368 DEV-window positions each. Its output is in this unit's REPORT.

**The question:** is the exported unsnapped artifact (A) inside the export gap's bar against its own float trunk (F)? And for which training configuration does that hold on both seeds?

The item's done-when needs the export gap "measured inside the band". The pre-named configuration of section 3, passing on both seeds, is that measurement.

**That configuration was chosen from claude-180's readings, on these same saved models and DEV rows.** So a close here is a DEV-scope result informed by that selection, and is reported as one.

## 2. Base artifacts and arms

**The models: the eight saved models of claude-180's sheet**, with the same `model.safetensors` pins and training receipts (`docs/harness/prep/d11-exporter-float-trunk-snap-ablation-2026-10-29.md`, section 2).
- **The six older models load only with B_old,** this head with claude-179's `fix.patch` reversed. That is claude-180's section 10, reviewed GO. The two bfixed-c179 models load with the head binary.
- **The snapshots and their store sums:**

| Model | `snapshot.json` sha256 |
|---|---|
| plain-c126 s1 / s2 (`…/d11gap/save/seed-s/`) | `a24ea8cf7af43733589ab51388bd0921a70a418e653fcb2e4677852dc69e4545` / `74b97fa3df9270784a8a98c22c88cd78423f6e8984a99104d70b1c4ef9dacd5a` |
| d11-c130 s1 / s2 (`…/d11t/save/seed-s/`) | `7a28779bfa63dff18ddacc0f293649d2ffa4d105347aa501865ea83064a22f01` / `0fa16c0a8ca8426f6dbec852aee9dbcfed9f1cbcb045757611b59d2adb873a32` |
| dual-c131 s1 / s2 (`…/dual/save/seed-s/`) | `5ac62adc0161e1f6d8d7c987b312fe8664d66a4a43bd7985f72c289d0199d160` / `39e8ed82a42da00cb7fa2cb4f9658a109c63bb45ad8dbe7c01909bede15ccbce` |
| bfixed-c179 s1 / s2 (`…/fix-arms/n1/save/seed-s/`) | `04f95fafd5f88d277af2df0a7f53e828673abfffe04112d9a81a9e85909af7f0` / `170b357a4d8b8a07440f76fb2d0ea06fd70e60aefe381b3ffbc3b6a29637e3e5` |

- **The DEV windows:** `windows.json`, `083c1be628f871ff1502e7d05f84cc3f55dda2ac461614a62df9167bcd08893e` (`…/dual/windows.json`). These are the same 428 windows the earlier artifact runs read.

**The arms.** Each model's snapshot is exported on the pod by this head's `fbs-export` four times, and each artifact is scored the same way:

| Arm | `fbs-export` flags | Role |
|---|---|---|
| `snapped` | none | control: the `lut-stack/4` artifact |
| `raw_gates` | `--raw-gates` | the gates unsnapped |
| `raw_transition` | `--raw-transition` | the transition unsnapped |
| `raw_both` | `--raw-gates --raw-transition` | the full unsnapped contract |

**The scoring of each artifact:**
- `d11-fbs-eval run ARTIFACT windows.json OUT --verify`: the GPU evaluator, held at every position against the host twin and the Rust session.
- Then `spiralcore-fbs-sheet --mode d11-artifact-gap`, with `--train-receipt`, against the saved model's own plain-float panel F, on the 14,287-row `ft/dev` panel.
- B_old scores the six older models, and the head binary scores the two bfixed models.

## 3. Bar and decision table

**The bar** (Stage 4b's export gap, `export_gap_within`). An artifact is **inside** when both halves hold:
- |b − c| ≤ F_I = max(24, ⌈2√(b + c)⌉), with b and c the paired discordant rows between F and A;
- |L_A − L_F| ≤ Δ = `0.045679637781173676`.

**A configuration passes under an arm** when both of its seeds' artifacts are counted (section 4) and inside. The configurations are the four training recipes: plain-c126, d11-c130, dual-c131 and bfixed-c179.

**The one configuration that can close the item is named here, before the run: dual-c131 under `raw_both`.**
- In claude-180's float emulation it was the configuration whose snaps-off forward was inside on both seeds (nets 3 and −9, F_I 24).
- `raw_both` is the full unsnapped contract.
- Naming one of the 8 configuration-and-arm pairs in advance avoids choosing the winner after the run.
- A pass anywhere else is a reading, and closes nothing without a confirming run of its own.

**The item's reading.** The first row that holds applies:

| Reading | When | What it decides |
|---|---|---|
| **No reading** | fewer than 6 models are counted, or `I-Qualify` fails | An instrument repair, then this run again. |
| **Close** | dual-c131 passes under `raw_both` | **The done-when's export gap is measured inside the band.** This unit then runs the close checks below, and closes #2820 at DEV scope only if all of them pass. |
| **Pass elsewhere** | any other configuration-and-arm pair passes under `raw_both` or `raw_gates`, dual-c131 under `raw_gates` included | **The next unit: a confirming run.** Its own sheet names that pair before it runs, and it trains **new seeds** of that configuration. Never the eight models read here: re-scoring them would reproduce this run bit for bit. The item closes only on that confirmation. |
| **No reading** | fewer than 7 models are counted | An instrument repair first: "Narrowed" and "No change" need at least 7 models. |
| **Narrowed** | either of two things holds, over the counted models: some `raw_both` or `raw_gates` artifact is inside (one of the 16 is enough); or, for `raw_both` or for `raw_gates`, the sum of \|b − c\| is below the snapped arm's sum | **The next unit trains a configuration against the unsnapped D11 forward:** dual training with D in the unsnapped mode, then this measurement. |
| **No change** | otherwise | **The next unit: wider codes** for the query and the tables (claude-180's rounding readings), then this measurement. |

**The close checks**, run on the laptop after the pod, at the head. Each is a static check, a host test, or client-device serving (CHARTER §3.6).
1. **The serving audit:** `python3 scripts/audit_zero_matmul_serving.py <bin> --stack`, on the binary that `cargo build --release --locked -p uor-r4-integer --features serving-wasm --bin uor-r4-stack` builds. That is the `lut-stack/5` runtime, and the binary is what claude-177 and claude-181 audited. It must read FULL PASS.
2. **The WASM audit:** `python3 scripts/audit_zero_matmul_wasm.py <wasm> --serving --call-graph --build-commit <head>`. The module comes from `cargo build --release --locked --target wasm32-unknown-unknown -p uor-r4-integer --features serving-wasm --lib` with `RUSTFLAGS="-C target-feature=-bulk-memory"`. It must read 7 of 7 clean. Without that flag, 0xFC bulk-memory instructions read as unclassified, on `main` too.
3. **The integer serving tests:** `cargo test --locked -p uor-r4-integer --features serving-wasm --no-fail-fast`, with `~/.cargo/bin` first on PATH. It must show no failure beyond the missing-local-input set that `main` has, which claude-181 recorded as 35 tests, all panicking on a missing bundle or `tokenizer.json`.
4. **Speed:** `fbs-toks` decode and context tokens/s for both dual-c131 `raw_both` artifacts. This is the synthesis's tokens per second. Speed has no bar; only a failed `fbs-toks` run counts as red.

**On a red close check,** the item stays open: the check is repaired and run again, and the close waits on it.

**What a close implies against kimi-46's outcome 2:**
- **The top-1 floors:** a pass puts A within F_I of F on top-1. dual-c131's F is 3,845 and 3,128 (claude-180's `abl3/summary.json`, `float_top1`).
  - F_I is at most 204: even if all 10,368 DEV positions disagreed, ⌈2√10,368⌉ = 204.
  - So a pass puts A at 3,641 or more on seed 1 and 2,924 or more on seed 2. That clears kimi-46's floors of 1,907 and 2,294 on both seeds.
  - Those floors were set on claude-129's plain-c126 artifacts.
- **D, the counterfactual:** here D is claude-180's float emulation of the same contract, `rounding_only`. It is already inside on both dual-c131 seeds (3 and −9, F_I 24). So the close is not kimi-46's "cancellation" case, where A passes but D fails.

**"DEV scope"** is the scope kimi-46's sheet registered for this item's close ("close #2820 at DEV scope only"). The fresh split is not read.

`raw_transition` alone is a reading. A close is a result for dual-c131's two saved models under `raw_both` only; nothing here closes or opens a mechanism (§4.1).

**Readings, with no bar:**
- each artifact's top-1, b, c, net, F_I and |ΔL|;
- each artifact against its float D11 emulation (claude-180's arm of the same snaps);
- tokens/s of the passing configuration's artifacts.

## 4. Instrument checks (each must be green before a model counts)

1. **`I-GPU-Tests`.** `cargo test --release --locked -p uor-r4-float-train --features cuda --lib` passes on the pod at the head.
2. **`I-Qualify`.** `d11-fbs-eval qualify` (schema `qualify/2`) passes. That covers every synthetic control in all four snap choices, single tokens and sequences, with GPU = host twin = Rust session bit for bit. A failure stops the run.
3. **`I-Fetch`.** Every fetched file verifies against its folder's `SHA256SUMS`, plus the DEV pins and the four receipt pins of claude-180's section 2.
4. **`I-Snapshot`.** Each `snapshot.json` equals its pin in section 2.
5. **`I-Export-Snapped`.** For the six older models, the snapped export reproduces the artifact claude-129 to claude-131 scored:
   - plain-c126: `314c3909…fdb305` and `d5b43596…dcde`;
   - d11-c130: `4255d39b…6eca` and `b466501e…f1cd`;
   - dual-c131: `0ae7dcd4…493a` and `8394a4d3…b599`.
   These are claude-132's sheet, section 5.
6. **`I-Verify`.** Each `run --verify` is equal at all 82,176 positions (428 × 192), in both checks.
7. **`I-Inputs`, `I-Save-Load`, `I-DEV-Denominator`**, as the gap mode checks them:
   - the evaluator read these windows and this artifact;
   - the model pin holds;
   - F reproduces `preds-float.json` and the receipt's loss within 1e-6;
   - there are 14,287 rows.
8. **`I-Snapped-Recorded`.** For the six older models, the snapped artifact's net and F_I equal claude-136's F against A: 81 (60), 349 (70), 278 (69), −332 (59), 353 (73) and 231 (70).
   - The two bfixed-c179 models have no earlier artifact. Their snapped arm has no tie to an earlier run, which is a known gap.
8a. **`I-Arm-Header`.** Each artifact's header carries its arm's schema and flags:
   - `snapped`: `lut-stack/4`, both flags false, with a transport snap;
   - `raw_gates`: `/5`, `raw_gates` only, with a snap;
   - `raw_transition`: `/5`, `raw_transition` only, with no snap;
   - `raw_both`: `/5`, both flags, with no snap.
8b. **`I-Arms-Distinct`.** A model's four artifacts have four different sha256s.
8c. **`I-Same-Tables`.** A model's four artifacts have byte-identical data sections, so they differ in the header only. That makes the gap mode's artifact loss valid for every arm: its own Q12 queries through the same exported head tables. On claude-126 seed 1's four local exports, the four data sections are identical.
9. **`I-Negative`.** One gap run with a wrong `--artifact-sha256` gives `i_inputs: false`.
10. **`I-Finite`.**
    - Every job-level step exits 0.
    - A model's own failed step leaves that model uncounted.
    - Each step's stdout, stderr and exit code are saved.

## 5. Data split

- DEV only: `w2-inputs/2026-10-25/driver-data/ft/dev/`. `tokens.u16` is `0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b` and `response_mask.u8` is `a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078`.
- That is 428 windows and 14,287 scored rows.
- No training data is read, and neither is `fresh_v2`.

## 6. Steps

1. Lease one RTX 5090 with `uor-pod up` at the pushed head.
2. **Build:**
   - `spiralcore-fbs-sheet` at the head (B_head), and B_old from a copy with `fix.patch` reversed (exactly three files apart, with the old `spiralcore_fbs.cu` pin);
   - `fbs-export` (`uor-r4-integer`, `serving-wasm`);
   - `d11-fbs-eval` (`--features cuda`).
   Each is copied out with its sha256.
3. Run `I-GPU-Tests`, then `I-Qualify`.
4. Fetch the inputs and check them (`I-Fetch`, `I-Snapshot`).
5. **For each model:** export the four artifacts (`I-Export-Snapped`). For each artifact, run `d11-fbs-eval run … --verify`, then `d11-artifact-gap` with the model's binary.
6. Run `I-Negative`.
7. A `jq` summary applies section 3.
8. Seal to `results/claude/d11-unsnapped-gap-2026-10-29/` with a `SHA256SUMS`, read it back, then `release` and `down`.

**The job and summary programs** are local files, run in a fresh folder (a non-empty one is refused, with a marker outside it) and copied into the seal, as in claude-180.
- `job.sh` template: `0c84aebbddb1f2f769e97fbaa5c118c72407c0b4c8894fa1b0a0b5ac4208a608`. The launched copy differs only in its `SHA=` line.
- `summary.jq`: `b8827c588d2e46e96381cf75c479400735ba82cac81bfdc07150458cb6f0c8e3`. The job checks this pin.
- `fix.patch`, for B_old: claude-179's `local/claude/claude-179/fix.patch`, at `a2728e80a980c02b034e056199d45f8dc119cf1cf87aaa67434697f9216a9d55`. The job checks this pin too.
- **The summary reads one record per artifact:** its sha256, its header (schema, flags, snap), the sha256 of its data sections, the stored pin, and the gap receipt's readings. It applies `I-Arm-Header`, `I-Arms-Distinct`, `I-Same-Tables`, `I-Export-Snapped` and `I-Snapped-Recorded` to decide which models are counted, then section 3's table.

## 7. Minimum test power

- **Scale:** 32 artifacts, being 8 models in 4 configurations of 2 seeds, times 4 arms. Each is paired with F on 14,287 rows.
- **Determinism:** the evaluator is deterministic and verified at every position, so one pass decides each artifact.
- **Pass needs both seeds** of a configuration.
- **What claude-180's float emulation leads us to expect:**
  - The snaps-off forward was inside on both dual-c131 seeds (3 and −9 against F_I 24), so a Pass is reachable there.
  - Elsewhere it missed by 7 to 28 rows on three models, and by more on three others.
  - The integer contract adds its own rounding (claude-181: median query drift of 6 LSB), so the integer result can differ from the emulation in either direction.

## 8. Cost and stop rule

**Cost:**
- one RTX 5090 on Vast, about $0.6/h, on a 2 h lease;
- builds of about 6 minutes, with two `spiralcore-fbs-sheet` builds and the evaluator's `nvcc` build;
- tests and qualification of about 5 minutes;
- 32 evaluator runs at about 15 s of GPU each (claude-128's figure for 428 windows), and 32 gap runs at about 1 minute each.
- **In all, about 1 hour, about $0.6.**

**Stop rule:**
- **The whole job stops,** and saves, on a failed build, `I-GPU-Tests`, `I-Qualify`, `I-Fetch`, `I-Snapshot`, `I-Negative` or summary step.
- **A model's own failed step** leaves that model uncounted, and the run goes on.
- Classify every failure as instrument or mechanism before calling it anything.
- Renew the lease rather than cancel a healthy run. Save and read back before `down`.

## 9. Non-author TEST FITNESS review

**Round 1 (fresh non-author reviewer, at `5fc87fbbb`): NOT FIT. All nine findings are fixed.**
1. **High: "Narrowed" was ambiguous.** It is now defined per arm, over the counted models, with `raw_both` or `raw_gates` only, and one inside artifact enough.
2. **High: a Pass picked the best of 8.** The one closing configuration is named before the run (dual-c131 under `raw_both`). Any other pass is "Pass elsewhere", a confirming run.
3. **High: the close step did not name its audits.** The close checks are listed with their commands and targets, and a red one blocks the close.
4. **Medium: "DEV scope" was untied.** It is tied to kimi-46's sheet.
5. **Medium: nothing checked that the export flags took effect.** `I-Arm-Header` and `I-Arms-Distinct` now do.
6. **Medium: the artifact loss assumed the same head tables.** `I-Same-Tables` checks it, and the local exports confirm it.
7. **Medium: missing models could make a reading.** "Narrowed" and "No change" need 7 counted models.
8. **Low: the bfixed snapped arm had no tie.** It is named as a known gap.
9. **Low: the trained parity claim was unsourced.** The tests are named.

**Round 1 confirmed:** the six artifact pins match claude-132's table; the B_old reuse is sound; the cost is inside the caps.

**Round 2 (fresh non-author reviewer, at `7cdc61a54`): NOT FIT. All six findings are fixed.**
1. **High: "Pass elsewhere" could close on re-scoring the same models.** Its confirming run now trains new seeds of the named pair.
2. **Medium: the table and `summary.jq` disagreed on dual-c131 under `raw_gates`.** The row now says "any other configuration-and-arm pair, dual-c131 under `raw_gates` included", as `summary.jq` computes it.
3. **Medium: the tie to kimi-46 borrowed the phrase without its conditions.** The close checks gain the integer serving tests. The sheet states what a close implies for kimi-46's top-1 floors and for D.
4. **Low: section 1 contradicted section 3.** It now names the pre-named configuration, and notes that the choice came from these same models.
5. **Low: the close checks were underspecified.** They now give the build commands, `--build-commit`, no bar on speed, and what a red check means.
6. **Low: `fix.patch` was unnamed in the sheet.** Section 6 names its source and pin.

**Round 2 confirmed:**
- the program pins and the 12 stored pins;
- the gap fields `record` copies;
- the header readers against the writer;
- `summary.jq`'s order;
- the pre-named close's justification;
- the audit usage lines.

The D67 pre-run review is still to come.
