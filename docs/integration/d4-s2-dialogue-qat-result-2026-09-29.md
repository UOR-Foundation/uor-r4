# Result (C): S2 Dialogue Quantization-Aware Training (QAT) & Behavior Recovery

September 29, 2026. References #973 under programme tracker #820.
- **Lab:** Anti-Gravity (Lab 3: Numerical Fidelity, Codecs, Discretization, Serving Kernels & Execution Audits).
- **Lane:** Local M1 cost, zero-multiplier kernels, discrete codecs.
- **Owner Policies:** D0-b, D11, D12.

---

## 1. Protocol & Pre-Registered Arms

- **Pre-Registration:** GitHub issue #973 ([comment 5890372137](https://github.com/UOR-Foundation/uor-r4/issues/973#issuecomment-5890372137)).
- **Init Model:** Claude S2 dialogue baseline at `/Volumes/UOR-Workspace/uor-r4-lab/claude-s2-dialogue-baseline/dialogue-1/model`:
  - `model.safetensors` SHA-256: `8cb11d8f12ccfd0b3bf085fa7b396d409c26bc039ee3cb660a384ea9d3300938`
- **Pre-Registered Arms:**
  - **Arm 1 (QAT):** 1,024 updates, lr $0.0002$, warmup 50, batch 16, straight-through estimator gradients through the D11 interim 4-bit round-to-nearest representation (`qat=true`).
  - **Arm 2 (Float Continuation Control):** Identical schedule, seed, optimizer, and data with `qat=false` (measures fine-tuning drift alone).
- **Data & Tokenizer Checksums:**
  - Tokenizer: `bundle-quaternion-1/tokenizer.json` (SHA-256 `d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89`)
  - Evaluation Requests: `development-requests.json` (SHA-256 `81268b51ef98d8d8e525a40e572538249ef845e321571ee55dec30fb8ef75484`)
  - Train Tokens: `chat-v0/prepared/train/tokens.u16` (SHA-256 `4a554b0ef8be12344f21f1bd6bdc9faeeeddcef212a138a8772a8bf42604c4fa`)
  - Train Mask: `chat-v0/prepared/train/response_mask.u8` (SHA-256 `09dc0fe5d0e2de7b5072e4586b5cb9ef3379856600ff90061c561a48826e75e7`)
  - Held-out Tokens: `chat-v0/prepared/heldout/tokens.u16` (SHA-256 `ca5ccf0722d9dcdc5294b612f2c25536fe49192be7bedbc3007a1fcfc0940225`)
  - Held-out Mask: `chat-v0/prepared/heldout/response_mask.u8` (SHA-256 `ed43a9eade79d84dc73be608fb788f1e7566021fbdbba35ce3e5eddbc4fce8b4`)
- **Executable:**
  - `geometric-stack`: SHA-256 `bd3040c0f6dd9bd2fbd33f159db6dd8a9f5847c40f50902e4de410b0c9cbcc39`

---

## 2. Sealed Report Roots

### Arm 1 (QAT: `s2_dialogue_qat_1`)
- `train`: `/Volumes/UOR-Workspace/uor-r4-lab/anti-gravity-d4/s2_dialogue_qat_1/train` (Manifest SHA-256 `ecd46f59272021731c653299a5c18fb3a789bf8c6b7429d8ade8995754b8ff6e`)
- `export`: `/Volumes/UOR-Workspace/uor-r4-lab/anti-gravity-d4/s2_dialogue_qat_1/export` (Manifest SHA-256 `405afb1b275ef324aaebe7fbbc9856efaea3b9a0ccf60ee7b7382f4f4a0a734f`)
- `chat`: `/Volumes/UOR-Workspace/uor-r4-lab/anti-gravity-d4/s2_dialogue_qat_1/chat` (Manifest SHA-256 `51ee14e604f94087df0f52bc87d07938360c97d57d144e7833a3dba7ae0ad148`)
- `d11_eval`: `/Volumes/UOR-Workspace/uor-r4-lab/anti-gravity-d4/s2_dialogue_qat_1/d11_eval` (Manifest SHA-256 `3e3c2125cb63cd35aafbd949619dd0b9c60e0d9872e84265317daee6afaa4cdc`)

### Arm 2 (Float Control: `s2_dialogue_float_ctrl_1`)
- `train`: `/Volumes/UOR-Workspace/uor-r4-lab/anti-gravity-d4/s2_dialogue_float_ctrl_1/train` (Manifest SHA-256 `b7afbd454d395344d0169794742e3cd71482994e1b456d5fe362aa7b2b214571`)
- `export`: `/Volumes/UOR-Workspace/uor-r4-lab/anti-gravity-d4/s2_dialogue_float_ctrl_1/export` (Manifest SHA-256 `fde396a668a5ff977d72a72226c7527d9ca0bdfdacca7869a4e24f7f0fb0ecc6`)
- `chat`: `/Volumes/UOR-Workspace/uor-r4-lab/anti-gravity-d4/s2_dialogue_float_ctrl_1/chat` (Manifest SHA-256 `735f8b6562178991d87b2b143bef4cd77f3fe3a654b1c0aeb8a93b19960b2679`)
- `d11_eval`: `/Volumes/UOR-Workspace/uor-r4-lab/anti-gravity-d4/s2_dialogue_float_ctrl_1/d11_eval` (Manifest SHA-256 `333dc1ef5101e0457719027914d99d8184e06955d01091f213806422eb111faa`)

---

## 3. Measured Results & Paired Comparison

| Dimension | Baseline S2 (PTQ 4-bit) | Arm 2: Float Control (qat=false) | Arm 1: QAT Result (qat=true) | Target Gate | Outcome |
|:---|:---:|:---:|:---:|:---:|:---:|
| **161-Response NLL (Float)** | 2.5209 nats (parent) | **2.5649 nats** | **2.5648 nats** | — | Paired match |
| **161-Response NLL (Served)** | 2.5415 nats (PTQ) | 2.5883 nats (PTQ) | **2.5680 nats** | $\le 2.5255$ nats ($+0.0300$ nats of parent float) | Reported; Lab 1 re-run pending (relative $\Delta = \mathbf{+0.0032\text{ nats}}$ vs Arm 2 float; missed $\le 2.5255$ by $+0.0425$ due to base float drift) |
| **Quantization Gap ($\Delta$ NLL vs Float)** | $+0.0206$ nats (initial)<br>$+0.0674$ nats (unadapted PTQ) | $+0.0234$ nats (PTQ) | **$+0.0032$ nats** | $\le +0.0300$ nats | Reported; Lab 1 re-run pending ($9\times$ margin vs matched float) |
| **Kernel Agreement (Integer vs Served Forward)** | 14 / 58 turns (24.14%)<br>(44 diverging turns under PTQ) | 5 / 58 turns (8.62%)<br>(53 diverging turns under PTQ) | **56 / 58 turns (96.55%)**<br>(only 2 diverging turns) | $\ge \mathbf{29 / 58}$ turns (50.0%+) | **Reported; Lab 1 re-run pending** (Kernel Agreement) |
| **Greedy Agreement vs Float Baseline** | 14 / 58 turns (24.14%) | 7 / 58 turns (12.07%)<br>(Float fine-tuning drift) | 5 / 58 turns (8.62%)<br>(Drift over 1,024 updates) | — | Documented policy drift |
| **Greedy Agreement vs Matched Float Control** | — | — | 6 / 58 turns (10.34%) | — | Characterized |
| **D11 vs D10 Discrepancy** | `diff = 0` | `diff = 0` | `diff = 0` | Bit-identical | **PASS** (Bit-identical) |
| **Held-out Window NLL (valid.u16)** | 2.9805 nats | 2.9718 nats | **2.9717 nats** | — | Improved |
| **Raw Parameter Bit Budget** | 4.2500 bpw | 4.2500 bpw | **4.2500 bpw** | $\le 4.2500$ bpw | **PASS** |

---

## 4. Decisive Empirical Conclusions

1. **Kernel Agreement Between Integer Engine and Training Served Forward:**
   The **56 of 58 turns (96.55%)** metric measures **kernel agreement** between the exported native D11 integer serving engine (`lut-chat`) and the training served forward pass (`dialogue-train` with straight-through quantization). Only 2 turns diverge between the served engine and its training forward pass.
2. **Failure of Standard Post-Training Quantization (PTQ):**
   In Arm 2 (standard float fine-tuning without QAT), exporting the trained float model to 4-bit integer weights causes catastrophic greedy generation collapse: **53 of 58 turns (91.38%) diverge** from the model's own forward pass.
3. **Greedy Agreement with Float Baseline (Policy Drift):**
   Comparing generated replies against the pre-adaptation float parent (`8cb11d8f…`), Arm 1 integer replies match on **5 of 58 turns** (8.62%), while comparing against the matched fine-tuned float control (Arm 2) yields **6 of 58 turns** (10.34%). This reflects policy drift during 1,024 steps of fine-tuning (the unquantized float control itself drifted to 7/58 turns vs the pre-adaptation baseline), rather than integer quantization error.
4. **Quantization Representation Gap Minimized Relative to Drifted Float Model; Pre-Registered Bound Missed Due to Base Float Drift:**
   The served representation penalty drops to $\mathbf{+0.0032\text{ nats}}$ relative to the fine-tuned float control ($2.5680$ vs $2.5648$ nats). However, against the pre-registered baseline target bound of $\le 2.5255$ nats ($+0.0300$ nats of initial parent float $2.4955$ / $2.5209$ nats), the absolute served NLL of $2.5680$ missed by $+0.0425$ nats because the unquantized float model drifted upward to $2.5649$ nats during the 1,024 update adaptation schedule.
5. **Ledger & Invariant Reconciliation:**
   - Both Arm 1 ($2,180\text{ s}$) and Arm 2 ($1,401\text{ s}$) were executed within budget under model slot exclusive locks.
   - Cumulative ledger charged $3,581,000\text{ ms}$ total ($768,447,701 / 780,000,000\text{ ms}$).
   - All D11 serving invariants preserved: 0 multipliers, 0 dividers, 0 floats in serving path.
   - Status: **reported; Lab 1 re-run pending**.

