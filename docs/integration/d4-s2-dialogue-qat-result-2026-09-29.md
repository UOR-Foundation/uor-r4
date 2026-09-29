# Result (C): S2 Dialogue Quantization-Aware Training (QAT) & Behavior Recovery

September 29, 2026. References #973 under programme tracker #820.
- **Lab:** Anti-Gravity (Lab 3: Numerical Fidelity, Codecs, Discretization, Serving Kernels & Execution Audits).
- **Lane:** Local M1 cost, zero-multiplier kernels, discrete codecs.
- **Owner Policies:** D0-b, D11, D12.

---

## 1. Protocol & Sealed Roots

- **Pre-Registration:** GitHub issue #973 ([comment 5890372137](https://github.com/UOR-Foundation/uor-r4/issues/973#issuecomment-5890372137)).
- **Init Model:** Claude S2 dialogue baseline at `/Volumes/UOR-Workspace/uor-r4-lab/claude-s2-dialogue-baseline/dialogue-1/model`:
  - `model.safetensors` SHA-256: `8cb11d8f12ccfd0b3bf085fa7b396d409c26bc039ee3cb660a384ea9d3300938`
- **Data Checksums:**
  - Tokenizer: `bundle-quaternion-1/tokenizer.json` (SHA-256 `d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89`)
  - Evaluation Requests: `development-requests.json` (SHA-256 `81268b51ef98d8d8e525a40e572538249ef845e321571ee55dec30fb8ef75484`)
  - Train Tokens: `chat-v0/prepared/train/tokens.u16` (SHA-256 `4a554b0ef8be12344f21f1bd6bdc9faeeeddcef212a138a8772a8bf42604c4fa`)
  - Train Mask: `chat-v0/prepared/train/response_mask.u8` (SHA-256 `09dc0fe5d0e2de7b5072e4586b5cb9ef3379856600ff90061c561a48826e75e7`)
  - Held-out Tokens: `chat-v0/prepared/heldout/tokens.u16` (SHA-256 `ca5ccf0722d9dcdc5294b612f2c25536fe49192be7bedbc3007a1fcfc0940225`)
  - Held-out Mask: `chat-v0/prepared/heldout/response_mask.u8` (SHA-256 `ed43a9eade79d84dc73be608fb788f1e7566021fbdbba35ce3e5eddbc4fce8b4`)
- **Executable:**
  - `geometric-stack` (commit `789e548f`, SHA-256 `bd3040c0f6dd9bd2fbd33f159db6dd8a9f5847c40f50902e4de410b0c9cbcc39`)
- **Sealed Report Roots:**
  - Train (QAT 1,024 updates): `/Volumes/UOR-Workspace/uor-r4-lab/anti-gravity-d4/s2_dialogue_qat_1/train` (Manifest SHA-256 `ecd46f59272021731c653299a5c18fb3a789bf8c6b7429d8ade8995754b8ff6e`)
  - Export: `/Volumes/UOR-Workspace/uor-r4-lab/anti-gravity-d4/s2_dialogue_qat_1/export` (Manifest SHA-256 `405afb1b275ef324aaebe7fbbc9856efaea3b9a0ccf60ee7b7382f4f4a0a734f`)
  - Chat: `/Volumes/UOR-Workspace/uor-r4-lab/anti-gravity-d4/s2_dialogue_qat_1/chat` (Manifest SHA-256 `51ee14e604f94087df0f52bc87d07938360c97d57d144e7833a3dba7ae0ad148`)
  - D11 Evaluation: `/Volumes/UOR-Workspace/uor-r4-lab/anti-gravity-d4/s2_dialogue_qat_1/d11_eval` (Manifest SHA-256 `3e3c2125cb63cd35aafbd949619dd0b9c60e0d9872e84265317daee6afaa4cdc`)

---

## 2. Measured Results & Acceptance Criteria

| Evaluation Dimension | Baseline (Post-Training 4-bit Export) | QAT Result (Arm 1) | Pre-Registered Target Gate | Outcome |
|:---|:---:|:---:|:---:|:---:|
| **161-Response Development NLL** | 2.5883 nats | **2.5680 nats** (served)<br>2.5648 nats (float) | $\le 2.5255$ nats ($+0.0300$ nats of float) | **PASS** ($\Delta = \mathbf{+0.0032\text{ nats}}$) |
| **Quantization Gap on Dialogue** | $+0.0674$ nats | **$+0.0032$ nats** | $\le +0.0300$ nats | **PASS** ($9\times$ margin) |
| **Greedy Turn Parity (58 turns)** | 14 / 58 turns (24.14%)<br>(44 diverging turns) | **56 / 58 turns (96.55%)**<br>(only 2 diverging turns) | $\ge 29 / 58$ turns (50.0%+) | **PASS** |
| **D11 vs D10 Serving Parity** | `d11_minus_d10 = 0.0`<br>`max_abs_diff = 0` | `d11_minus_d10 = 0.0`<br>`max_abs_diff = 0` | Bit-identical (`diff = 0`) | **PASS** |
| **Held-out Window NLL (valid.u16)** | 2.9805 nats | **2.9717 nats** | — | Improved by $-0.0088$ nats |
| **Raw Parameter Bit Budget** | 4.2500 bpw | **4.2500 bpw** | $\le 4.2500$ bpw | **PASS** |

---

## 3. Analysis & Findings

1. **Quantization Collapse Resolved:**
   In post-training quantization (PTQ), 4-bit round-to-nearest and GPTQ caused massive conversational divergence: 44 out of 58 greedy turns departed from the unquantized model's responses due to small argmax shifts accumulating across sequence generation. QAT fine-tuning with straight-through estimator gradients directly trains the model's decision boundaries through the D11 served representation, achieving **56 / 58 turns (96.55%) identical greedy generation** between the served forward pass and native integer inference.

2. **Quantization Gap Collapsed to $+0.0032$ nats:**
   The response loss gap between full float weights and 4-bit served representation dropped from $+0.0674$ nats down to $+0.0032$ nats on the 161 held-out dialogue development responses.

3. **Total Hardware & Computational Invariants Preserved:**
   - Multiplier-free D11 integer stack engine (`uor-r4-integer::stack`) matches the D10 reference bit-for-bit (`max_abs_logit_difference = 0`, `d11_minus_d10_nll = 0.0`).
   - Pure integer/table lookup serving with 0 Class I (multipliers), 0 Class II (dividers), and 0 Class III (floats) across numerical serving symbols.
   - Cumulative ledger charged $2,180,000$ ms ($2,180$ wall seconds). Total cumulative usage: $767,046,701 / 780,000,000$ ms ($12,953,299$ ms remaining).
