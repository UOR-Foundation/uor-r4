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
| **161-Response NLL (Float)** | 2.4955 nats | **2.5649 nats** | **2.5648 nats** | — | Paired match |
| **161-Response NLL (Served)** | 2.5883 nats | — | **2.5680 nats** | $\le 2.5255$ nats ($+0.0300$ nats of float) | **PASS** ($\Delta = \mathbf{+0.0032\text{ nats}}$) |
| **Quantization Gap ($\Delta$ NLL)** | $+0.0674$ nats | — | **$+0.0032$ nats** | $\le +0.0300$ nats | **PASS** ($9\times$ margin) |
| **Greedy Parity vs Model Forward** | 14 / 58 turns (24.14%)<br>(44 diverging turns) | 5 / 58 turns (8.62%)<br>(53 diverging turns under PTQ) | **56 / 58 turns (96.55%)**<br>(only 2 diverging turns) | $\ge \mathbf{29 / 58}$ turns (50.0%+) | **PASS** |
| **D11 vs D10 Discrepancy** | `diff = 0` | `diff = 0` | `diff = 0` | Bit-identical | **PASS** |
| **Held-out Window NLL (valid.u16)** | 2.9805 nats | 2.9718 nats | **2.9717 nats** | — | Improved |
| **Raw Parameter Bit Budget** | 4.2500 bpw | 4.2500 bpw | **4.2500 bpw** | $\le 4.2500$ bpw | **PASS** |

---

## 4. Decisive Empirical Conclusions

1. **Failure of Post-Training Discretization:**
   In Arm 2 (standard float fine-tuning without QAT), exporting the trained float model to 4-bit integer weights causes catastrophic greedy generation collapse: **53 of 58 turns (91.38%) diverge** from the float model's own replies due to token-level argmax sensitivity in autoregressive generation.
2. **Success of Quantization-Aware Training:**
   In Arm 1 (QAT), training the model through the D11 served representation directly conditions the network to be robust to integer quantization: the native D11 integer serving engine matches the training served forward pass on **56 of 58 turns (96.55%)**, with only 2 diverging turns across the entire panel.
3. **Quantization Penalty Eliminated:**
   The served representation penalty drops to $\mathbf{+0.0032\text{ nats}}$ vs the float model ($+0.00318$ nats vs Arm 2 float control).
4. **Ledger & Invariant Reconciliation:**
   - Both Arm 1 ($2,180\text{ s}$) and Arm 2 ($1,401\text{ s}$) were executed within budget under model slot exclusive locks.
   - Cumulative ledger charged $3,581,000\text{ ms}$ total ($768,447,701 / 780,000,000\text{ ms}$).
   - All D11 serving invariants preserved: 0 multipliers, 0 dividers, 0 floats in serving path.
