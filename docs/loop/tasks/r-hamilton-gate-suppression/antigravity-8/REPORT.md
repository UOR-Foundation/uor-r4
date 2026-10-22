# REPORT: r-hamilton-gate-suppression, antigravity-8

Lab: antigravity. Item: [r-hamilton-gate-suppression](../../../../roadmap/items/r-hamilton-gate-suppression.md) (closes #2879).

## Negative results and falsifications

1. **Per-block amplitude suppression by a norm-preserving Hamilton gate is falsified by proof (`Supp_Hamilton`, `Guarantee`, Status: `Structural`):**
   - Under the registered definition `Supp_Hamilton` (posted on [`#2879`](https://github.com/UOR-Foundation/uor-r4/issues/2879#issuecomment-6336354313) before testing: achieving per-4-block amplitude ratio $\rho(g, u) := \|\text{out}(g, u)\|_2 / \|u\|_2 \le \tau = 0.5$ with bounded backward Jacobian norm $\|\partial q / \partial g\|_2 \le 10$), exact quaternion norm multiplicativity gives $\rho(g, u) = \|q\|_2 = \|g\|_2 / \sqrt{\|g\|_2^2 + 10^{-12}}$ and the exact coupling identity $\rho^2 + 10^{-12} / r^2 = 1$.
   - Therefore $\rho(g, u) \le 0.5$ holds if and only if $\|g\|_2 \le 10^{-6} / \sqrt{3} \approx 5.7735027 \times 10^{-7}$, where the backward Jacobian norm on $q^\perp$ is $1/r = 10^6 \sqrt{1 - \rho^2} \ge \frac{\sqrt{3}}{2} \times 10^6 \approx 8.660254 \times 10^5$ (and the radial derivative is $10^6 (1 - \rho^2)^{3/2} \ge 6.4951905 \times 10^5$, reaching $10^6$ at $g = 0$). In `f32`, for all $\|g\|_2 \ge 10^{-2}$, $\|g\|_2^2 + 10^{-12} = \|g\|_2^2$ bit-for-bit so $\rho(g, u) = 1.0\text{f32}$ exact; across the decade sweep $\|g\|_2 \in [10^{-9}, 10^3]$, the smallest ratio reached is $9.9999947 \times 10^{-4}$ at $\|g\|_2 = 10^{-9}$ ($0.0$ at $\|g\|_2 = 0$).
   - Per D22 (CHARTER §4.1), this negative closes the **un-scaled unit-norm configuration** as an amplitude-shutoff gate, never the Hamilton rotation mechanism itself (which remains a pure 4D directional rotor and can zero a single scalar readout projection $\langle w, q \otimes u \rangle = 0$ while preserving full 4-block norm $\|q \otimes u\|_2 = \|u\|_2$).
2. **Deliberate break:**
   - In `hamilton_gate_block_bwd` (`crates/uor-r4-training/src/geometric_stack_native.rs`), replacing `/ r` with `* r` caused `hamilton_gate_suppression_closed_form_and_backward_hazard_proof` to fail immediately with `|d_gate| 0.000001 != 1/r 1000000`. Restoring `/ r` restored green execution.

## Changes delivered

- **Design note (Step 2):** Posted on [`#2879`](https://github.com/UOR-Foundation/uor-r4/issues/2879#issuecomment-6336354313) before writing any test, registering `Supp_Hamilton`, YES/NO criteria, the analytic proof of $\rho(g, u)$ and $1/r$, test form `(a)`, and the notebook-page question for the maintainer.
- **Finite host test (Step 4, Form (a)):** Added `hamilton_product_4`, `hamilton_gate_scalar_ratio`, `hamilton_gate_block_fwd`, `hamilton_gate_block_bwd`, and `hamilton_gate_suppression_closed_form_and_backward_hazard_proof` in [`crates/uor-r4-training/src/geometric_stack_native.rs`](../../../../crates/uor-r4-training/src/geometric_stack_native.rs) (`mod tests`), with zero matmul and zero softmax.
- **Documentation & records (Steps 5–6):**
  - Updated [`docs/mechanisms/hamilton-gate.md`](../../../mechanisms/hamilton-gate.md) ("How it works", "What has been tried", "What has never been tested", and "Code" — pointing to the compiled `geometric_stack_native.rs` and noting that `geometric_stack.rs` and `tests/cuda_stack_ops_parity.rs` are inactive/disabled).
  - Updated [`docs/geometry/quaternions-and-rotation.md`](../../../geometry/quaternions-and-rotation.md) and [`docs/formal_vocabulary.md`](../../../formal_vocabulary.md) (`Supp_Hamilton`, **Definition** and **Guarantee**, Status: **Structural**).
  - Wrote [`docs/labs/r-hamilton-gate-suppression/README.md`](../../../labs/r-hamilton-gate-suppression/README.md) with the research notebook row (and asked on `#2879` which `docs/research/notebook/` page to use or open, since `docs/research/notebook/**` and `docs/research/open-questions.md` are outside `r-hamilton-gate-suppression.md`'s frontmatter `scope:`).
  - Marked [`docs/roadmap/items/r-hamilton-gate-suppression.md`](../../../roadmap/items/r-hamilton-gate-suppression.md) `status: done`.

## Verification

- `CARGO_TARGET_DIR=/Users/casey.allard/uor-r4/local/antigravity/cargo-target ~/.cargo/bin/cargo test -p uor-r4-training --lib hamilton_gate_suppression_closed_form_and_backward_hazard_proof -- --nocapture` — 1 passed, 0 failed (`smallest ratio reached = 9.9999947e-4 (at |g|=1e-9); at |g|=0 ratio = 0.0`).
- `uor-harness loop gates --full` — all 6 gates green, bound in `docs/loop/tasks/r-hamilton-gate-suppression/antigravity-8/receipts.jsonl`.
