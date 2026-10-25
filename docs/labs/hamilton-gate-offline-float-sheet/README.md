# Lab Record: `hamilton-gate-offline-float-sheet` (`#2824`)

- **Date:** 2026-10-25
- **Lab:** `antigravity` (`antigravity-79`)
- **Issue:** `#2824` (`hamilton-gate-offline-float-sheet`)
- **Pre-registration sheet:** [`docs/harness/prep/hamilton-gate-offline-float-sheet-2026-10-25.md`](../../harness/prep/hamilton-gate-offline-float-sheet-2026-10-25.md)
  - Non-author `TEST FITNESS: FIT`, `D68 DECISION REVIEW: APPROVE`, and `D67 PRE-RUN REVIEW: APPROVE` by fresh-context reviewer `f9fe6674-84f8-4c79-895b-d07cfbe8c4b3` ([`#2824` comment `6400747936`](https://github.com/UOR-Foundation/uor-r4/issues/2824#issuecomment-6400747936))
- **GPU Pod:** `vast-57698805` (`1 x NVIDIA GeForce RTX 5090`, 32,607 MiB VRAM, driver `595.84`, `nvcc 12.8.93`, `sm_120`, Czechia CZ, `$0.908/h`, leased and torn down after archiving; total spend `~$0.26`)
- **Source commit built on pod:** `8c920d3070e4147df875ff942c9b00e3191cecd3`
- **Binary SHA-256 (byte-identical across two independent clean `nvcc` builds):**
  - `bin/hamilton-gate-sheet-build-a`: `b9cbd600e601515820caf909c48b09c83b9d7ef2a52a161678a49960eee7e15c`
  - `bin/hamilton-gate-sheet-build-b`: `b9cbd600e601515820caf909c48b09c83b9d7ef2a52a161678a49960eee7e15c`
- **Committed receipts & HF store bundle:**
  - [`receipt.json`](./receipt.json) (Build A): `ac82937195856ef136b98c57158254337fcd76fdcae30a7603de9d78696c432f`
  - [`receipt-build-b.json`](./receipt-build-b.json) (Build B): `033b87db772f78505e75aebe69942aa11a709a2110bb59ae064482b443eac81a`
  - [`synthetic-probe.json`](./synthetic-probe.json): `881a1a5f75f988096f15c298dccae6d8d61966c627c8574ca72a33ae7a11f1c2`
  - [`throughput-probe.json`](./throughput-probe.json): `087d0541b1afae062f62cb1f7ce71f846c8caae2d22a7fadeb44791f8efae36d`
  - HF store: `caseyallard/uor-r4-store:results/antigravity/hamilton-gate-offline-float-sheet-2026-10-25/`

Labels used below: **proof** means derived from definitions or code, **measurement** means a recorded run or test, **hypothesis** means an untested idea. See [`docs/formal_vocabulary.md`](../../formal_vocabulary.md).

---

## 1. Summary of Outcomes (Negatives First)

1. **Language fit on integer DEV metrics (`G-Hamilton-Fit`, measurement):**
   - **Lead metric (`tokens/s`):** On `1 x RTX 5090` (`sm_120`, `context = 192`), the 120-icosian + Q12 `i32` integer-exported DEV evaluator runs at **`14,205.9–14,317.8 tok/s`** for `Hamilton_Active` and **`14,277.4–14,290.6 tok/s`** for `Hamilton_IcosianHard` (`~5.75 s` across all `428` causal windows / `82,176` evaluated tokens / `14,287` scored targets), **`125,027.8–125,674.0 tok/s`** for `IdentityGate_Control`, and **`88,186.4–90,367.6 tok/s`** for `RandomGate_Null`. Continuous `f32` DEV evaluation runs at **`26,103.5–26,917.1 tok/s`** (`Hamilton_Active`) and **`10,031.2–10,041.8 tok/s`** (`Hamilton_IcosianHard`). Training throughput (`batch = 4`, `context = 192`, `256` steps = `196,608` tokens) is **`12,942.0–12,954.9 tok/s`** (`Hamilton_Active`, `15.18 s`), **`4,478.8–4,484.9 tok/s`** (`Hamilton_IcosianHard`, `43.84 s`), **`19,242.4–19,416.2 tok/s`** (`IdentityGate_Control`, `10.13 s`), and **`15,990.8–16,191.9 tok/s`** (`RandomGate_Null`, `12.14 s`).
   - **Beat against `RandomGate_Null` and `TrainUnigram_CountTable` (`G-Beats-Random = PASS`):** On the `14,287` eligible DEV response targets (`ft/dev/tokens.u16`), `Hamilton_Active` and `Hamilton_IcosianHard` both beat `RandomGate_Null` (`24–28 / 14,287 = 0.17%–0.20%` top-1) by **`+1,010` to `+1,402` rows** on both seeds across both builds (`G-Beats-Random = true`). In Build A ([`receipt.json`](./receipt.json)), both arms also beat the response-masked training unigram count table (`TrainUnigram_CountTable`, `1,226 / 14,287 = 8.581%` top-1) in integer export on **both seeds**: `Hamilton_Active` scores **`1,270 / 14,287` (`8.889%`)** on seed 1 and **`1,298 / 14,287` (`9.085%`)** on seed 2 (`1,582` and `1,560` in float), while `Hamilton_IcosianHard` scores **`1,430 / 14,287` (`10.009%`)** on seed 1 and **`1,253 / 14,287` (`8.770%`)** on seed 2 (`1,419` and `1,231` in float). In Build B ([`receipt-build-b.json`](./receipt-build-b.json)), seed 1 beats unigram (`1,242` and `1,304` int top-1) while seed 2 lands at `1,057` (`Hamilton_Active` int; `1,295` float) and `1,034` (`Hamilton_IcosianHard` int).
   - **Negative against `IdentityGate_Control` (`G-Fit-Unigram = FAIL`):** At the 256-step micro-rung budget, `IdentityGate_Control` ($q_b \equiv (1,0,0,0)$, passing `up` un-rotated) reaches **`3,908 / 14,287` (`27.354%`)** on seed 1 and **`3,041 / 14,287` (`21.285%`)** on seed 2 in Build A (`3,673` and `3,678` in Build B), outscoring `Hamilton_Active` and `Hamilton_IcosianHard` by `1,743–2,644` rows (`g_hamilton_fit_pass_unigram = false`). Because `Hamilton_Active` and `Hamilton_IcosianHard` initialize `mlp.gate` and `rec.gate` from standard normal weights (`393,216` extra active parameters), their step-0 rotations are nearly uniform on $S^3$ and must first un-scramble the `up` branch before aligning with the downstream `SpiralcoreFbs` projections. Per `CHARTER.md` §4.1, this negative closes only the 256-step un-preconditioned `Gate::Hamilton` micro-rung configuration, not the Hamilton gate mechanism.
2. **Float-to-integer export gap (`G-Hamilton-ExportGap = PASS`, measurement):**
   - All arms and seeds pass `G-Hamilton-ExportGap` ($\Delta_{\text{export,top1}} \le 398$ rows and $\Delta_{\text{export,top8}} \le 748$ rows).
   - Crucially, **training with the hard 120-icosian forward pass (`Hamilton_IcosianHard`) reduces the step-256 float-to-integer export gap by an order of magnitude** relative to post-hoc icosian snapping of continuous `Hamilton_Active`:
     - `Hamilton_Active` step-256 export gap: $\Delta_{\text{top1}} = 312$ rows (`2.184 pp`, seed 1) and $262$ rows (`1.834 pp`, seed 2) in Build A (`80` and `238` rows in Build B), with float-vs-integer row-wise top-1 prediction agreement of `17.35%–20.23%`.
     - `Hamilton_IcosianHard` step-256 export gap: **$\Delta_{\text{top1}} = 11$ rows (`0.077 pp`, seed 1)** and **$22$ rows (`0.154 pp`, seed 2)** in Build A (`23` and `2` rows in Build B), $\Delta_{\text{top8}} = 3$ rows (`0.021 pp`), $\Delta M_{\text{tok}} \le 2.06 \times 10^{-6}$, with float-vs-integer row-wise top-1 prediction agreement of **`97.59%–97.84%`** (`13,942–13,979 / 14,287` rows identical; the remaining `~2.2%` difference comes solely from Q12 fixed-point rounding in `NativeVectorWriter` at the output head).
3. **Whether the gate suppressed signal (`G-Hamilton-Suppression`, measurement + proof):**
   - **Verdict:** `NO_INTRINSIC_AMPLITUDE_SUPPRESSION_DIRECTIONAL_MODULATION_ONLY` (`gate_suppressed_intrinsic_amplitude = false`).
   - **Intrinsic per-block amplitude suppression (`Supp_Hamilton`, measurement):** Across **`78,888,960` observed 4-blocks per DEV evaluation** (`428` windows $\times$ `192` tokens $\times$ `6` layers $\times$ `(96 + 64)` blocks) at steps 0 and 256 across all seeds and builds, **zero blocks (`0 / 78,888,960 = 0.0%`)** ever enter $\rho(g_b, u_b) = \|\text{out}_b\|_2 / \|u_b\|_2 \le 0.5$, $\|g_b\|_2 \le 10^{-3}$, or the $\|g_b\|_2 \le 5.7735 \times 10^{-7}$ guard core. Even under `weight_decay = 0.1`, `min_g_norm` remains in `0.00918–0.04279`, `median_g_norm` in `1.508–2.326`, `min_amplitude_ratio_rho` in `0.99999971–0.99999988` (`mean_rho = 1.00000000`), and `max_backward_scale_inv_r` in `23.37–108.94` (five orders of magnitude below the $\sim 10^6$ guard-core blow-up proved in `#2879`).
   - **Downstream projection cancellation (`mlp.down` and `rec.out` under `QuaternionWeights::SpiralcoreFbs`, measurement + proof):** Across **`986,112` observed downstream projection rows** per DEV evaluation, the downstream gain ratio $\gamma_{\text{down}} = \|P_{\text{down}}(q \otimes u)\|_2 / \|P_{\text{down}}(u)\|_2$ has `mean = 1.00000000`, `min = 0.99999977`, `max = 1.00000023`, and **`frac_rows_downstream_suppressed_lt_0_8 = 0.0` (`0 / 986,112` rows)**. This is a structural consequence (**proof**) of pairing `Gate::Hamilton` with `QuaternionWeights::SpiralcoreFbs`: `spiralcore_fbs_fwd` computes each 8D output atom by selecting a single 8D input atom (two 4-blocks $u_{2a}, u_{2a+1}$), applying an $O(8)$ signed permutation (`Cl(0,6)` monomial action), and scaling by `w_scale[o] / 2`. Because 4-block unit-quaternion rotation preserves $\|q_{2a}\otimes u_{2a}\|_2^2 + \|q_{2a+1}\otimes u_{2a+1}\|_2^2 = \|u_{2a}\|_2^2 + \|u_{2a+1}\|_2^2$ and the `Cl(0,6)` signed permutation is orthogonal on $\mathbb{R}^8$, `SpiralcoreFbs` preserves the L2 norm of every 8D atom regardless of the Hamilton gate's quaternion rotation! Thus on the `SpiralcoreFbs` stack, `Gate::Hamilton` suppresses signal **neither** inside the 4-block ($\rho = 1$) **nor** inside the downstream projection ($\gamma_{\text{down}} = 1$); it acts purely by rotating carrier directions prior to `SpiralcoreFbs` signed-permutation routing and residual addition $x + P_{\text{down}}(q \otimes u)$.

---

## 2. Instrument Checks (`I-1..4`) and Probe Ladder

All four pre-registered instrument checks passed (`4/4 PASS`) on both Build A and Build B before any mechanism conclusion was drawn:

| Instrument Check | Requirement | Measured Value (Build A & Build B) | Verdict |
|---|---|---|---|
| `I-Finite-CUDA` | Every forward, backward, optimizer, and DEV eval step executes on CUDA with `0` non-finite values, `0` CUDA errors, and step-0 $\|\partial L / \partial g\|_2 > 0$ on `Hamilton_Active` and `Hamilton_IcosianHard` | `20,412` CUDA error checks per arm-seed (`163,296` per build), `0` errors; step-0 $\|\partial g\|_2 = 0.283000$ (seed 1) / $0.282855$ (seed 2) on `Hamilton_Active` and $0.228077$ (seed 1) / $0.286125$ (seed 2) on `Hamilton_IcosianHard` | **PASS** |
| `I-DEV-Denominator` | `ft/dev/tokens.u16` (`82,535` tokens, `400` documents) + `response_mask.u8` yields exactly `14,287` eligible targets across `428` document-bounded causal windows (`context = 192`), and `TrainUnigram_CountTable` over `mw-balp/train` (`2,425,925` eligible targets) scores `1,226 / 14,287` (`8.581%`) top-1 | `400` documents, `428` windows, `14,287` scored rows on every evaluation; `TrainUnigram_CountTable` `top1_token_id = 13`, `dev_top1_hits = 1,226` (`8.5812%`), `dev_top8_hits = 4,060` (`28.4174%`) | **PASS** |
| `I-Icosian-Closure` | The 120-root `H4_UNIT_ICOSIAN_COEFFICIENTS_Q1` table (`717c064c4baae2ed54c61626eb63c876c37d0ce894d1a1c317bdddc672f6648a`) has unit norm (`< 1e-6`), closes under Hamilton product (`120 x 120` within `1e-5`), and recovers all `120 / 120` indices under integer `signed_h4_code_i32` | `root_count = 120`, `max_unit_norm_error = 0.0`, `max_closure_distance = 1.110223e-16`, `self_recovery_exact_count = 120 / 120` | **PASS** |
| `I-Q12-Int-Parity` | GPU Q12 `NativeVectorWriter` exported head (`round(x * 4096)` + integer `encode_from_q12` + `score_candidate`) matches `pure_i32_q12_writer_oracle` (`0` float ops) with `0` mismatches on `>= 256` sampled rows | `517` rows checked per integer DEV evaluation (`12` evaluations $\times$ `517` = `6,204` checked rows per build), **`0` mismatches** (`100.0%` exact agreement); GPU vs `i32` `signed_h4_code_i32` icosian snap agreement **`1.000000`** (`100.0%`) on all `Hamilton_Active` and `Hamilton_IcosianHard` evaluations | **PASS** |

### Probe Ladder ([`synthetic-probe.json`](./synthetic-probe.json), [`throughput-probe.json`](./throughput-probe.json))

- **Probe (a) (`synthetic-probe`):** 4-step synthetic cycle (`[0..7]` repeated over `32` scored positions) on `1 x RTX 5090`. All 4 arms decreased margin loss cleanly (`Hamilton_Active` $2.6901 \to 1.9810$, $\Delta = +0.7091$; `Hamilton_IcosianHard` $2.6987 \to 2.0252$, $\Delta = +0.6736$; `IdentityGate_Control` $2.5877 \to 1.7186$, $\Delta = +0.8691$; `RandomGate_Null` $2.8388 \to 2.2663$, $\Delta = +0.5725$), with `q12_oracle_mismatches = 0` on all 4 arms.
- **Probe (b) (`throughput-probe`):** 16-step timing + full `14,287`-row DEV evaluation on `Hamilton_Active` and `Hamilton_IcosianHard`. Measured `0.946 s` (`Hamilton_Active` 16-step train), `2.739 s` (`Hamilton_IcosianHard` 16-step train), `3.050 s` (float DEV eval), `5.747 s` (integer export DEV eval), projecting `62.12 s` per 256-step arm (`< 3,600 s` cap; `under_one_hour_per_arm = true`). Actual wall time for the entire 8-run `full` sheet was `~5.5 minutes` per build.

---

## 3. Language Fit and Export Gap Across All 4 Arms & 2 Seeds

### 3.1 Build A ([`receipt.json`](./receipt.json), SHA-256 `ac82937195856ef136b98c57158254337fcd76fdcae30a7603de9d78696c432f`)

| Arm | Seed | Train `tok/s` | Int Eval `tok/s` | Float Eval `tok/s` | Train Loss (`0 -> 256`, $\Delta$) | Float DEV `top-1` (`%`) | Int Export DEV `top-1` (`%`) | Float DEV `top-8` (`%`) | Int Export DEV `top-8` (`%`) | Int Admitted (`%`) | Int $M_{\text{tok}}$ | Export Gap $\Delta_{\text{top1}}$ (rows / pp) | Float-Int Row Agreement |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| `Hamilton_Active` | 1 | `12,942.0` | `14,317.8` | `26,887.8` | `2.8033 -> 2.1610` (`+0.6423`) | `1,582` (`11.073%`) | **`1,270` (`8.889%`)** | `2,340` (`16.379%`) | `2,008` (`14.055%`) | `2,615` (`18.303%`) | `0.992759` | `312` (`2.184 pp`) | `2,543 / 14,287` (`17.80%`) |
| `Hamilton_Active` | 2 | `12,954.9` | `14,205.9` | `26,917.1` | `2.7332 -> 2.0601` (`+0.6731`) | `1,560` (`10.919%`) | **`1,298` (`9.085%`)** | `2,500` (`17.498%`) | `2,118` (`14.825%`) | `2,728` (`19.094%`) | `0.990442` | `262` (`1.834 pp`) | `2,890 / 14,287` (`20.23%`) |
| `Hamilton_IcosianHard` | 1 | `4,479.0` | `14,280.6` | `10,031.2` | `2.8031 -> 2.1743` (`+0.6288`) | `1,419` (`9.932%`) | **`1,430` (`10.009%`)** | `2,276` (`15.931%`) | `2,279` (`15.952%`) | `3,117` (`21.817%`) | `0.993865` | **`11` (`0.077 pp`)** | **`13,949 / 14,287` (`97.63%`)** |
| `Hamilton_IcosianHard` | 2 | `4,484.9` | `14,279.1` | `10,041.8` | `2.7537 -> 2.2054` (`+0.5483`) | `1,231` (`8.616%`) | **`1,253` (`8.770%`)** | `1,551` (`10.856%`) | `1,554` (`10.877%`) | `1,884` (`13.187%`) | `0.994907` | **`22` (`0.154 pp`)** | **`13,942 / 14,287` (`97.59%`)** |
| `IdentityGate_Control` | 1 | `19,242.4` | `125,674.0` | `26,876.9` | `2.8062 -> 1.9657` (`+0.8405`) | `3,912` (`27.382%`) | `3,908` (`27.354%`) | `5,926` (`41.478%`) | `5,923` (`41.457%`) | `6,197` (`43.375%`) | `0.970921` | `4` (`0.028 pp`) | `14,252 / 14,287` (`99.76%`) |
| `IdentityGate_Control` | 2 | `19,416.2` | `125,287.8` | `26,838.2` | `2.7764 -> 1.9748` (`+0.8016`) | `3,044` (`21.306%`) | `3,041` (`21.285%`) | `4,585` (`32.092%`) | `4,577` (`32.036%`) | `4,681` (`32.764%`) | `0.961787` | `3` (`0.021 pp`) | `14,250 / 14,287` (`99.74%`) |
| `RandomGate_Null` | 1 | `15,990.8` | `88,546.7` | `24,474.3` | `2.8329 -> 2.4328` (`+0.4000`) | `28` (`0.196%`) | `28` (`0.196%`) | `129` (`0.903%`) | `129` (`0.903%`) | `434` (`3.038%`) | `1.001781` | `0` (`0.000 pp`) | `14,230 / 14,287` (`99.60%`) |
| `RandomGate_Null` | 2 | `16,191.9` | `88,186.4` | `24,847.2` | `2.8297 -> 2.4125` (`+0.4172`) | `26` (`0.182%`) | `26` (`0.182%`) | `134` (`0.938%`) | `134` (`0.938%`) | `462` (`3.234%`) | `1.001658` | `0` (`0.000 pp`) | `14,229 / 14,287` (`99.59%`) |
| `TrainUnigram_CountTable` | — | — | — | — | — | — | `1,226` (`8.581%`) | — | `4,060` (`28.417%`) | — | — | — | — |

### 3.2 Build B Replication ([`receipt-build-b.json`](./receipt-build-b.json), SHA-256 `033b87db772f78505e75aebe69942aa11a709a2110bb59ae064482b443eac81a`)

Step-0 forward and backward losses and norms are bit-identical between Build A and Build B across all 8 runs (`2.803302049636841`, `2.8030929565429688`, `2.806154727935791`, `2.832854747772217` on seed 1; `2.7332184314727783`, `2.7537314891815186`, `2.7764201164245605`, `2.829723834991455` on seed 2). Over 256 optimization steps, thread-scheduling order in GPU `atomicAdd` inside `spiralcore_fbs_bwd` introduces small trajectory variance (`<= 241` DEV top-1 rows, inside the `398`-row K16 two-seed noise band) while preserving every gate verdict (`I-1..4 = true`, `G-Beats-Random = true`, `G-Hamilton-ExportGap = true`, `G-Fit-Unigram = false`, `gate_suppressed_intrinsic_amplitude = false`):

| Arm | Seed | Train `tok/s` | Int Eval `tok/s` | Train Loss (`0 -> 256`, $\Delta$) | Float DEV `top-1` (`%`) | Int Export DEV `top-1` (`%`) | Int Export DEV `top-8` (`%`) | Int Admitted (`%`) | Export Gap $\Delta_{\text{top1}}$ (rows / pp) | Float-Int Row Agreement |
|---|---|---|---|---|---|---|---|---|---|---|
| `Hamilton_Active` | 1 | `12,953.5` | `14,297.2` | `2.8033 -> 2.1387` (`+0.6646`) | `1,322` (`9.253%`) | `1,242` (`8.693%`) | `1,852` (`12.963%`) | `2,390` (`16.728%`) | `80` (`0.560 pp`) | `2,479 / 14,287` (`17.35%`) |
| `Hamilton_Active` | 2 | `12,943.3` | `14,272.0` | `2.7332 -> 2.1107` (`+0.6225`) | `1,295` (`9.064%`) | `1,057` (`7.398%`) | `1,488` (`10.415%`) | `1,831` (`12.816%`) | `238` (`1.666 pp`) | `2,780 / 14,287` (`19.46%`) |
| `Hamilton_IcosianHard` | 1 | `4,480.8` | `14,287.2` | `2.8031 -> 2.1369` (`+0.6662`) | `1,281` (`8.966%`) | `1,304` (`9.127%`) | `2,273` (`15.910%`) | `3,104` (`21.726%`) | **`23` (`0.161 pp`)** | **`13,979 / 14,287` (`97.84%`)** |
| `Hamilton_IcosianHard` | 2 | `4,478.8` | `14,290.6` | `2.7537 -> 2.1499` (`+0.6038`) | `1,036` (`7.251%`) | `1,034` (`7.237%`) | `1,275` (`8.924%`) | `1,661` (`11.626%`) | **`2` (`0.014 pp`)** | **`13,962 / 14,287` (`97.73%`)** |
| `IdentityGate_Control` | 1 | `19,318.9` | `125,383.8` | `2.8062 -> 1.9810` (`+0.8251`) | `3,670` (`25.688%`) | `3,673` (`25.709%`) | `5,387` (`37.706%`) | `5,717` (`40.015%`) | `3` (`0.021 pp`) | `14,244 / 14,287` (`99.70%`) |
| `IdentityGate_Control` | 2 | `19,301.1` | `125,027.8` | `2.7764 -> 1.9611` (`+0.8153`) | `3,678` (`25.744%`) | `3,678` (`25.744%`) | `5,873` (`41.107%`) | `5,984` (`41.884%`) | `0` (`0.000 pp`) | `14,248 / 14,287` (`99.73%`) |
| `RandomGate_Null` | 1 | `16,021.3` | `88,979.2` | `2.8329 -> 2.4295` (`+0.4033`) | `25` (`0.175%`) | `25` (`0.175%`) | `129` (`0.903%`) | `424` (`2.968%`) | `0` (`0.000 pp`) | `14,234 / 14,287` (`99.63%`) |
| `RandomGate_Null` | 2 | `16,044.9` | `90,367.6` | `2.8297 -> 2.4460` (`+0.3837`) | `24` (`0.168%`) | `24` (`0.168%`) | `126` (`0.882%`) | `469` (`3.283%`) | `0` (`0.000 pp`) | `14,239 / 14,287` (`99.66%`) |

### 3.3 Paired McNemar Discordance (`(b, c)` on `14,287` Integer DEV Rows, Build A)

| Comparison (`Arm A` vs `Arm B`) | Seed | `c` (`A` hit, `B` miss) | `b` (`A` miss, `B` hit) | Net Advantage (`c - b` rows) |
|---|---|---|---|---|
| `Hamilton_Active` vs `RandomGate_Null` | 1 | `1,264` | `22` | **`+1,242`** |
| `Hamilton_Active` vs `RandomGate_Null` | 2 | `1,294` | `22` | **`+1,272`** |
| `Hamilton_IcosianHard` vs `RandomGate_Null` | 1 | `1,428` | `26` | **`+1,402`** |
| `Hamilton_IcosianHard` vs `RandomGate_Null` | 2 | `1,248` | `21` | **`+1,227`** |
| `Hamilton_Active` vs `TrainUnigram_CountTable` | 1 | `208` | `164` | **`+44`** |
| `Hamilton_Active` vs `TrainUnigram_CountTable` | 2 | `262` | `190` | **`+72`** |
| `Hamilton_Active` vs `Hamilton_IcosianHard` | 1 | `456` | `616` | `-160` |
| `Hamilton_Active` vs `Hamilton_IcosianHard` | 2 | `316` | `271` | `+45` |
| `Hamilton_Active` vs `IdentityGate_Control` | 1 | `69` | `2,707` | `-2,638` |
| `Hamilton_Active` vs `IdentityGate_Control` | 2 | `256` | `1,999` | `-1,743` |
| `Hamilton_IcosianHard` vs `IdentityGate_Control` | 1 | `182` | `2,660` | `-2,478` |
| `Hamilton_IcosianHard` vs `IdentityGate_Control` | 2 | `192` | `1,980` | `-1,788` |

---

## 4. Signal Suppression Telemetry (`G-Hamilton-Suppression`, `78,888,960` Blocks & `986,112` Rows per Eval)

Every DEV evaluation at steps 0 and 256 recorded both (i) intrinsic per-block amplitude ratio $\rho(g_b, u_b) = \|\text{out}_b\|_2 / \|u_b\|_2$ across all `78,888,960` 4-blocks (`6` layers $\times$ `(96 + 64)` blocks $\times$ `428` windows $\times$ `192` positions) and (ii) downstream projection norm ratio $\gamma_{\text{down}} = \|P_{\text{down}}(q \otimes u)\|_2 / \|P_{\text{down}}(u)\|_2$ across all `986,112` `(window, pos, layer)` projection rows (`mlp.down` and `rec.out`):

| Arm | Seed | Step | $\min \|g_b\|_2$ | `p01` $\|g_b\|_2$ | Median $\|g_b\|_2$ | $\min \rho(g_b, u_b)$ | Blocks $\rho \le 0.5$ | Blocks $\|g_b\|_2 \le 10^{-3}$ | Max $1/r_b$ | Active Icosian Roots | Mean Icosian $\cos$ | GPU-vs-`i32` Snap Match | Mean $\gamma_{\text{down}}$ (`min..max`) | Rows $\gamma_{\text{down}} < 0.8$ |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| `Hamilton_Active` | 1 | 0 | `0.02165` | `0.5698` | `1.7717` | `0.99999971` | `0` (`0.0%`) | `0` (`0.0%`) | `46.19` | `120 / 120` | `0.9647` | `1.000000` | `1.000000` (`0.99999983..1.00000017`) | `0` (`0.0%`) |
| `Hamilton_Active` | 1 | 256 | `0.04279` | `0.5257` | `1.5082` | `0.99999973` | `0` (`0.0%`) | `0` (`0.0%`) | `23.37` | `120 / 120` | `0.9649` | `1.000000` | `1.000000` (`0.99999978..1.00000022`) | `0` (`0.0%`) |
| `Hamilton_Active` | 2 | 0 | `0.03324` | `0.6050` | `1.8289` | `0.99999975` | `0` (`0.0%`) | `0` (`0.0%`) | `30.08` | `120 / 120` | `0.9637` | `1.000000` | `1.000000` (`0.99999983..1.00000017`) | `0` (`0.0%`) |
| `Hamilton_Active` | 2 | 256 | `0.01903` | `0.4567` | `2.1214` | `0.99999975` | `0` (`0.0%`) | `0` (`0.0%`) | `52.55` | `120 / 120` | `0.9658` | `1.000000` | `1.000000` (`0.99999977..1.00000023`) | `0` (`0.0%`) |
| `Hamilton_IcosianHard` | 1 | 0 | `0.02165` | `0.5698` | `1.7717` | `0.99999988` | `0` (`0.0%`) | `0` (`0.0%`) | `46.19` | `120 / 120` | `0.9656` | `1.000000` | `1.000000` (`0.99999983..1.00000016`) | `0` (`0.0%`) |
| `Hamilton_IcosianHard` | 1 | 256 | `0.03651` | `0.5064` | `1.5917` | `0.99999988` | `0` (`0.0%`) | `0` (`0.0%`) | `27.39` | `120 / 120` | `0.9613` | `1.000000` | `1.000000` (`0.99999983..1.00000019`) | `0` (`0.0%`) |
| `Hamilton_IcosianHard` | 2 | 0 | `0.02458` | `0.6671` | `1.8441` | `0.99999988` | `0` (`0.0%`) | `0` (`0.0%`) | `40.68` | `120 / 120` | `0.9662` | `1.000000` | `1.000000` (`0.99999984..1.00000016`) | `0` (`0.0%`) |
| `Hamilton_IcosianHard` | 2 | 256 | `0.03285` | `0.5224` | `2.3262` | `0.99999988` | `0` (`0.0%`) | `0` (`0.0%`) | `30.44` | `120 / 120` | `0.9623` | `1.000000` | `1.000000` (`0.99999983..1.00000020`) | `0` (`0.0%`) |

---

## 5. Reproducibility Command

```bash
export UOR_NVCC_ARCH=compute_120
cargo run --release --locked -p uor-r4-float-train --features cuda --bin hamilton-gate-sheet -- \
  --mode full \
  --data-root /workspace/w2-inputs/w2-inputs/2026-10-25/driver-data \
  --out docs/labs/hamilton-gate-offline-float-sheet/receipt.json
```
