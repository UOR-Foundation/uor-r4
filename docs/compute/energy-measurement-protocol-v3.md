# Energy and Full-Path Laptop Measurement Protocol (Version 3)

**Status:** Draft (`PROPOSED, NOT REGISTERED` for M5 acceptance until owner registration on [`#2862`](https://github.com/UOR-Foundation/uor-r4/issues/2862) / [`#2033`](https://github.com/UOR-Foundation/uor-r4/issues/2033))
**Supersedes:** [`archive/docs/compute/energy-measurement-protocol-v2.md`](../../archive/docs/compute/energy-measurement-protocol-v2.md)
**Item:** [`docs/roadmap/items/m5-laptop-full-path-measurement.md`](../roadmap/items/m5-laptop-full-path-measurement.md) ([`#2862`](https://github.com/UOR-Foundation/uor-r4/issues/2862) / [`#2033`](https://github.com/UOR-Foundation/uor-r4/issues/2033))
**Evidence folder:** [`docs/labs/m5-laptop-full-path-measurement/README.md`](../labs/m5-laptop-full-path-measurement/README.md)

---

## 1. Scope and Purpose

This protocol governs laptop full-path cost and energy measurement for Milestone M5 ([`#2033`](https://github.com/UOR-Foundation/uor-r4/issues/2033), [`docs/roadmap/items/m5-laptop-full-path-measurement.md`](../roadmap/items/m5-laptop-full-path-measurement.md)) on the native integer stack runtime ([`crates/uor-r4-integer/src/stack/session.rs`](../../crates/uor-r4-integer/src/stack/session.rs) and [`crates/uor-r4-integer/src/bin/uor-r4-stack.rs`](../../crates/uor-r4-integer/src/bin/uor-r4-stack.rs)).

Protocol v3 closes the two audit defects documented in Protocol v2 §4 on [`archive/ops/measurement_report_rnd.json`](../../archive/ops/measurement_report_rnd.json) and adds the four M5 full-path cost instruments required by [`docs/roadmap/items/m5-laptop-full-path-measurement.md`](../roadmap/items/m5-laptop-full-path-measurement.md):

1. **Power/throughput arithmetic consistency:** In [`archive/ops/measurement_report_rnd.json`](../../archive/ops/measurement_report_rnd.json), `power.work_rate_w = 40.5 W` disagrees with `energy_per_token.gross_j_per_tok * throughput.tokens_per_s = 0.003172 * 17,190 = 54.52668 W` by `25.72%` because `work_rate_w` was averaged across wall-clock time including process startup/teardown outside the stepping window while `tokens_per_s` was computed over the stepping window alone. Under Protocol v3, both `work_rate_w` and `tokens_per_s` are integrated over the identical window `[T_start, T_end]`, and the validator in [`crates/uor-r4-integer/tests/m5_full_path_cost_controls.rs`](../../crates/uor-r4-integer/tests/m5_full_path_cost_controls.rs) rejects any receipt where `|work_rate_w - gross_j_per_tok * tokens_per_s| / (gross_j_per_tok * tokens_per_s) > 0.02`.
2. **Small-sample Student-$t$ confidence intervals:** In [`archive/ops/measurement_report_rnd.json`](../../archive/ops/measurement_report_rnd.json), `gross_mj_per_tok.ci95 = 0.037 mJ` on $n = 3$ samples (`[3.135, 3.172, 3.199]` mJ, sample SD $s = 0.032130$ mJ) used the normal approximation $z_{0.975} = 1.96$ ($1.96 \cdot s / \sqrt{3} = 0.036358 \approx 0.037$ mJ), understating the true two-sided 95% confidence half-width under Student's $t_{0.975, 2} = 4.302653$ ($4.302653 \cdot s / \sqrt{3} = 0.079815 \approx 0.080$ mJ) by `53.64%`. Under Protocol v3, all sample confidence intervals use Student's $t_{0.975, n-1}$ with $n - 1$ degrees of freedom (`student_t_975` and `sample_mean_sd_t_ci95` in [`crates/uor-r4-integer/tests/m5_full_path_cost_controls.rs`](../../crates/uor-r4-integer/tests/m5_full_path_cost_controls.rs)).
3. **Exact per-phase parameter and active-ratio accounting:** [`IntegerStackModel::parameter_breakdown`](../../crates/uor-r4-integer/src/stack/session.rs) computes the exact 4-bit matrix weights, packed weight+scale bytes, and learned integer/grid-code scalars across all four serving phases (`ingestion`, `routing`, `memory_lookup`, `emission`) directly from the loaded artifact, replacing the hardcoded `7,238,304` string in `uor-r4-stack cost`.
4. **Per-phase latency breakdown outside the audited `step` call graph:** [`IntegerStackSession::measure_step_phases`](../../crates/uor-r4-integer/src/stack/session.rs) measures nanosecond durations across `ingestion`, `routing`, `memory_lookup`, and `emission` while keeping [`IntegerStackSession::step`](../../crates/uor-r4-integer/src/stack/session.rs) and `stack_step` untouched so `python3 scripts/audit_zero_matmul_serving.py target/release/uor-r4-stack --stack` continues to pass all `44 / 44` mandatory symbols and `0` call-graph violations. `uor-r4-stack cost` verifies `phase_instrumentation_logits_match == true` (`logits_sha256` bit-identical between the unphased `step` pass and the phased `measure_step_phases` pass).
5. **Kernel high-water-mark memory footprint and process isolation:** `uor-r4-stack cost` records the kernel high-water-mark physical footprint (`vmmap -summary <pid>` `Physical footprint (peak)` on macOS; `/proc/self/status` `VmHWM` on Linux) alongside multi-stage `ps -o rss=` checkpoints (`initial_rss_mb`, `post_load_rss_mb`, `post_session_alloc_rss_mb`, `post_eval_rss_mb`, `max_sampled_rss_mb`) and verifies `external_inference_daemons_detected == []` (`ollama`, `llama-server`, `llama-cli`, `vllm`, `lmstudio`, `mlx_lm`).

---

## 2. Four-Phase Decomposition of a Served Step

Every served token in [`IntegerStackSession::step`](../../crates/uor-r4-integer/src/stack/session.rs) executes four sequential phases:

| Phase ID | JSON Key | Operations in [`crates/uor-r4-integer/src/stack/session.rs`](../../crates/uor-r4-integer/src/stack/session.rs) | Active 4-Bit Matrix Weights Read per Token | Total 4-Bit Matrix Weights Stored | Learned Integer / Grid-Code Scalars |
|---|---|---|---|---|---|
| Phase 1 | `ingestion` | Token/context bounds check + `stack_dequant_row(&model.embed, token, &mut b.x)` | `d` (`1` row of `embed`) | `vocab * d` | `0` |
| Phase 2 | `routing` | Each `r` layer: pre-norm + `stack_recurrence` (`rec_in`, `rec_gate`, causal conv, quaternion transport, `rec_out`) + residual + pre-norm + MLP (`gate`, `up`, `stack_swiglu`, `down`) + residual | Per `r` layer: `2*d*d + gate_rows*d + d*d + 3*mlp*d` (`100%` of `r`-layer 4-bit weights) | Same as active (`100%` dense per `r` layer) | Per `r` layer: `4*d` (`conv_taps`) + `d` (`conv_bias`) + `gate_rows` (`gate_bias`) + `lanes` (`decay_rate`) |
| Phase 3 | `memory_lookup` | Each `a` layer: pre-norm + `stack_read` (`query`, `key`, `value`, `null`, Dot/Lorentz/L2/binary/rank read, `out`) + residual + pre-norm + MLP (`gate`, `up`, `stack_swiglu`, `down`) + residual | Per `a` layer: `4*d*d + heads*d + 3*mlp*d` (`100%` of `a`-layer 4-bit weights) | Same as active (`100%` dense per `a` layer) | Per `a` layer: `heads` (`null_bias`) + `heads * context` (`age`) + (`2 * heads` if scaled: `beta`, `offset`) + (`heads * side^2` if `read_learned`) |
| Phase 4 | `emission` | Final `stack_rms_norm` + `stack_map_pairs(&model.head, ...)` + optional `copy_scale_q16` boost + optional `stack_pointer` + cursor advance | `vocab * d` (+ `(2 * p.dim + 1) * d` if pointer head) | Same as active (`100%` dense on `head` and optional pointer head) | `0` (+ `1 + usize::from(p.lorentz)` if pointer head) |

### Reference Parameter Counts

- **On `mk-synthetic-stack` (`seed = 5`, `vocab = 40, width = 32, heads = 2, mlp = 32, pattern = "ra", context = 64, rotation = true, lorentz = true`):**
  - `ingestion`: `32` active / `1,280` total 4-bit weights (`17 B` active / `680 B` total), `0` scalars.
  - `routing` (`l0` `r`): `7,424` active / `7,424` total 4-bit weights (`3,944 B`), `208` scalars (`560 B`).
  - `memory_lookup` (`l1` `a`): `7,232` active / `7,232` total 4-bit weights (`3,842 B`), `134` scalars (`532 B`).
  - `emission` (`head`): `1,280` active / `1,280` total 4-bit weights (`680 B`), `0` scalars.
  - **Totals:** `weights_per_token = 15,968`, `total_4bit_weights = 17,216` (`weights_read_over_total_4bit_weights = 15,968 / 17,216 = 0.92750929`), `total_learned_scalars = 342`, `total_learned_parameters = 17,558`, `active_learned_parameters_per_token = 16,310` (`active_over_total_learned_parameters = 16,310 / 17,558 = 0.92892129`).
- **On the 7.5M D11 stack shape (`vocab = 1024, width = 256, heads = 4, mlp = 688, pattern = "rrarra", context = 256`):**
  - `ingestion`: `256` active / `262,144` total 4-bit weights (`136 B` active / `139,264 B` total).
  - `routing` + `memory_lookup` + `emission`: `7,238,048` active / `7,238,048` total 4-bit weights.
  - **Totals:** `weights_per_token = 7,238,304`, `total_4bit_weights = 7,500,192` (`weights_read_over_total_4bit_weights = 7,238,304 / 7,500,192 = 0.96508249`).

---

## 3. Hardware Energy Measurement Procedure (Protocol v3)

### 3.1 Supported Power Samplers

1. **Apple Silicon (`arm64` macOS, e.g., Apple M1):**
   - Rootless sampler: `macmon pipe -s 100` (`scripts/energy_per_token.py --sampler macmon`), reading `all_power` (`cpu_power + gpu_power + ane_power`) and `ram_power` (`sys_power` fallback) at `10 Hz` (`100 ms` interval).
   - Privileged fallback: `sudo powermetrics -i 100 --samplers cpu_power,gpu_power,ane_power` (`scripts/energy_per_token.py --sampler powermetrics`).
2. **Linux x86\_64 / NVIDIA GPU:**
   - CPU package + DRAM energy counters via Linux `perf stat -e power/energy-pkg/,power/energy-ram/` or RAPL `/sys/class/powercap/intel-rapl*/energy_uj`.
   - GPU board power via `nvidia-smi --query-gpu=timestamp,power.draw --format=csv -lms 100` (`scripts/energy_per_token.py --sampler nvidia-smi`).

### 3.2 Idle Baseline, Stepping Window, and Trapezoidal Integration

1. **Quiescent pre-flight:** Verify `external_inference_daemons_detected == []` and record `sysctl -n vm.loadavg` before and after the run.
2. **Idle baseline ($P_{\text{idle}}$):** Sample undisturbed system power for $T_{\text{idle}} \ge 5.0\text{ s}$ immediately before the workload starts:
   $$P_{\text{idle}} = \frac{1}{K_{\text{idle}}} \sum_{k=1}^{K_{\text{idle}}} P(t_k)$$
3. **Synchronized stepping window ($[T_{\text{start}, i}, T_{\text{end}, i}]$):** For each trial $i \in \{1, \dots, n\}$ ($n \ge 3$; $n \ge 5$ recommended for sealing a milestone receipt) of $N_{\text{tok}}$ stepped tokens, integrate gross energy over the exact stepping window using trapezoidal integration:
   $$E_{\text{gross}, i} = \sum_{m=1}^{M_i - 1} \frac{P(t_m) + P(t_{m+1})}{2} (t_{m+1} - t_m), \qquad \Delta t_i = T_{\text{end}, i} - T_{\text{start}, i}$$
   $$\bar{P}_{\text{work}, i} = \frac{E_{\text{gross}, i}}{\Delta t_i}, \qquad R_i = \frac{N_{\text{tok}}}{\Delta t_i}, \qquad e_{\text{gross}, i} = \frac{E_{\text{gross}, i}}{N_{\text{tok}}} = \frac{\bar{P}_{\text{work}, i}}{R_i}$$
   $$E_{\text{net}, i} = \max\!\left(0,\, E_{\text{gross}, i} - P_{\text{idle}} \cdot \Delta t_i\right), \qquad e_{\text{net}, i} = \frac{E_{\text{net}, i}}{N_{\text{tok}}}$$

### 3.3 Mandatory Statistical and Consistency Checks

For $n \ge 2$ trials with per-trial gross or net energy per token $x_1, \dots, x_n$ (in mJ/tok):

$$\bar{x} = \frac{1}{n} \sum_{i=1}^{n} x_i, \qquad s = \sqrt{\frac{1}{n - 1} \sum_{i=1}^{n} (x_i - \bar{x})^2}, \qquad \text{CI}_{95} = t_{0.975,\, n - 1} \cdot \frac{s}{\sqrt{n}}$$

where $t_{0.975, 2} = 4.302653$, $t_{0.975, 4} = 2.776445$, $t_{0.975, 9} = 2.262157$.

Every Protocol v3 receipt must pass `validate_protocol_v3_energy_receipt` ([`crates/uor-r4-integer/tests/m5_full_path_cost_controls.rs`](../../crates/uor-r4-integer/tests/m5_full_path_cost_controls.rs)):
1. **Power/throughput identity:**
   $$\frac{\left|\bar{P}_{\text{work}} - \bar{e}_{\text{gross, J/tok}} \cdot \bar{R}_{\text{tok/s}}\right|}{\bar{e}_{\text{gross, J/tok}} \cdot \bar{R}_{\text{tok/s}}} \le 0.02$$
2. **Student-$t$ CI95 identity:** Reported `ci95` for both `gross_mj_per_tok` and `net_mj_per_tok` matches $t_{0.975, n-1} \cdot s / \sqrt{n}$ within `2.0%`.
3. **Required full-path fields:** `/memory/parameter_breakdown`, `/memory/kernel_peak_footprint_bytes`, `/step_latency/phase_latency_us`, `/tokens/logits_sha256`, and `/machine/external_inference_daemons_detected` are all present.
