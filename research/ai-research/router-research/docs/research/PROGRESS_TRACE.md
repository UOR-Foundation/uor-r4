# Progress Trace

## 2026-03-06

### Resume after `INC-0044`
1. Read `INC_0045_static_chart_floor.md` and `HANDOFF_CURRENT.md`.
2. Verified the queued next branch was still `INC-0045`.
3. Checked the existing resume/update protocol in `docs/research/UPDATE_PROTOCOL.md`.

### Compaction-resilience work
4. Added this progress trace file as an explicit step-by-step session record.
5. Planned to keep this file updated before and after long-running sweeps and after any branch decision.

### `INC-0045` staging
6. Marked `INC_0045_static_chart_floor.md` as the active next branch in current docs.
7. Defined the branch target:
   - `HOPF_PHI2_BAND_IT48_P3_STATIC`
   - `HOPF_PHI2_BAND_IT40_P2_STATIC`
   - `HOPF_K25_BASE_IT40_P2_STATIC`
   - `R0`
8. Kept `train_route_mode=final_static` fixed for routed branches.
9. Kept the larger-subset proxy workload fixed.
10. Chose chart-pressure only as the live axis.

### Pending next action
11. Create `configs/proxy_transfer_inc0045_static_chart_floor_screen.json`.
12. Run the 2-seed screen.
13. If the screen is live, promote a single cheaper branch to 4-seed confirm.

### `INC-0045` screen launch
14. Created the screen config for the chart-floor test.
15. Next command: `python tools/proxy_sweep.py --config configs/proxy_transfer_inc0045_static_chart_floor_screen.json`.

### `INC-0045` screen result
16. Screen analysis landed at `results/analysis/inc0045_static_chart_floor_screen.json`.
17. `HOPF_K25_BASE_IT40_P2_STATIC` beat cheap `R0` on both quality and runtime and passed the health gate.
18. `HOPF_PHI2_BAND_IT40_P2_STATIC` also passed and beat cheap `R0` on both quality and runtime.
19. Promotion decision: confirm `HOPF_K25_BASE_IT40_P2_STATIC` and `HOPF_PHI2_BAND_IT40_P2_STATIC`, with `R0` as control.

### `INC-0045` confirm result
20. Confirm analysis landed at `results/analysis/inc0045_static_chart_floor_confirm.json`.
21. `HOPF_K25_BASE_IT40_P2_STATIC` beat cheap `R0` on both quality and runtime across 4 seeds and passed the health gate.
22. `HOPF_PHI2_BAND_IT40_P2_STATIC` also beat cheap `R0` on both quality and runtime across 4 seeds and passed the health gate.
23. Promotion decision: `HOPF_K25_BASE_IT40_P2_STATIC` becomes the operational routed lead.
24. Next queued branch: `INC-0046` static scale robustness.

### `INC-0046` staging
25. Marked `INC_0046_static_scale_robustness.md` as in progress.
26. Chose the next larger subset step at `max_train=8000`, `max_eval=4000`.
27. Kept the cheap chart schedule fixed at `IT40_P2` and kept geometry fixed.
28. Next command: `python tools/proxy_sweep.py --config configs/proxy_transfer_inc0046_static_scale_robustness_screen.json`.

### `INC-0046` screen result
29. Screen analysis landed at `results/analysis/inc0046_static_scale_robustness_screen.json`.
30. `HOPF_K25_BASE_IT40_P2_STATIC` stayed ahead of `R0` on both quality and runtime at the larger subset step.
31. `HOPF_PHI2_BAND_IT40_P2_STATIC` also stayed ahead of `R0` and was the faster routed branch.
32. Promotion decision: confirm both routed branches against `R0` to resolve quality-vs-runtime leadership.

### `INC-0046` confirm result
33. Confirm analysis landed at `results/analysis/inc0046_static_scale_robustness_confirm.json`.
34. `HOPF_K25_BASE_IT40_P2_STATIC` stayed ahead of `R0` on both quality and runtime across 4 seeds at the larger subset step.
35. `HOPF_PHI2_BAND_IT40_P2_STATIC` also stayed ahead of `R0` and remained the faster routed branch within quality tolerance.
36. Promotion decision: keep the cheap routed frontier and move next to `INC-0047` near-full-proxy scale.

### `INC-0047` staging
37. Marked `INC_0047_near_full_proxy_scale.md` as in progress.
38. Chose the near-full-proxy step at `max_train=12000`, `max_eval=6000`.
39. Kept the cheap `IT40_P2` chart schedule and static training-route reuse fixed.
40. Next command: `python tools/proxy_sweep.py --config configs/proxy_transfer_inc0047_near_full_proxy_scale_screen.json`.

### `INC-0047` screen result
41. Screen analysis landed at `results/analysis/inc0047_near_full_proxy_scale_screen.json`.
42. `HOPF_K25_BASE_IT40_P2_STATIC` stayed ahead of `R0` on both quality and runtime near full proxy scale.
43. `HOPF_PHI2_BAND_IT40_P2_STATIC` also stayed ahead of `R0` and remained the faster routed branch.
44. Promotion decision: confirm both routed branches against `R0` at near-full-proxy scale.

### `INC-0047` confirm result
45. Confirm analysis landed at `results/analysis/inc0047_near_full_proxy_scale_confirm.json`.
46. `HOPF_K25_BASE_IT40_P2_STATIC` beat `R0` on both quality and runtime across 4 seeds near full proxy scale.
47. `HOPF_PHI2_BAND_IT40_P2_STATIC` also beat `R0` on both quality and runtime across 4 seeds near full proxy scale.
48. Promotion decision: keep the cheap routed frontier and move next to `INC-0048` integration translation.

### `INC-0048` translation staging
49. Read `HANDOFF_CURRENT.md`, `CURRENT_DIRECTION.md`, and `INC_0048_integration_translation.md`.
50. Chose the first translation target as routed token-memory retrieval preselection, not expert preselection or another regression harness.
51. Reused the existing JSON-summary contract instead of opening a second parser pipeline.
52. Added `tasks/router_retrieval_eval.py` as the new translation harness.
53. Extended `tools/proxy_sweep.py` so config can pick `task_script` and `baseline_route_id`.
54. Extended `tools/summarize.py` with retrieval-specific columns.
55. Added a non-`R0` baseline test to `tests/test_proxy_sweep.py`.
56. Ran compile and sweep-unit smoke; both passed.
57. Ran direct smoke commands for:
   - `smoke_dense_retrieval`
   - `smoke_hopf_retrieval`
58. Dense smoke returned a valid JSON summary.
59. Routed Hopf smoke returned a valid JSON summary.
60. Wrote `docs/research/INTEGRATION_TRANSLATION_PLAN.md`.
61. Wrote `configs/proxy_transfer_inc0048_retrieval_translation_screen.json`.
62. Updated `INC_0048_integration_translation.md` from queued to in progress.

### Pending next action
63. Run `python tools/proxy_sweep.py --config configs/proxy_transfer_inc0048_retrieval_translation_screen.json`.
64. If the screen is live, promote at most one routed retrieval branch to confirm.

### `INC-0048` screen result
65. Screen analysis landed at `results/analysis/inc0048_retrieval_translation_screen.json`.
66. Dense exact retrieval stayed operationally dominant on both MSE and wall-clock.
67. The translated routed branches still showed real candidate pruning:
   - `HOPF_RET_P1`: `cand_frac≈0.349`
   - `HOPF_PHI2_RET_P1`: `cand_frac≈0.341`
68. Multi-bucket probing improved quality slightly but made runtime worse.
69. Decision: keep translation alive, but move next to retrieval cost rescue instead of confirm.
70. Queued `INC_0049_retrieval_cost_rescue.md`.

### Dynamic-state geometry note
71. Recorded the new geometry hypothesis that the missing object may be 8D position-plus-flow state rather than another static shell law.
72. Queued `INC_0050_dynamic_h4_state.md` as a math branch to revisit only after the current retrieval systems rescue path.
73. Refined the geometry note: keep `H^4 x H^4` live as a stronger alternative to `H^4 + T_xH^4`, not just as another way to say “8 variables”.

### `INC-0049` staging
74. Read `INC_0049_retrieval_cost_rescue.md` and kept geometry fixed.
75. Implemented grouped same-bucket retrieval for `probe_buckets=1` in `tasks/router_retrieval_eval.py`.
76. Added offline/online timing decomposition fields for translated retrieval runs.
77. Updated `tools/summarize.py` and `tools/proxy_sweep.py` to carry the new retrieval timing fields.
78. Added grouped retrieval coverage to `tests/test_router_retrieval_eval.py`.
79. Wrote `configs/proxy_transfer_inc0049_retrieval_cost_rescue_screen.json`.
80. Updated `INC_0049_retrieval_cost_rescue.md` from queued to in progress.

### Pending next action
81. Run `python tools/proxy_sweep.py --config configs/proxy_transfer_inc0049_retrieval_cost_rescue_screen.json`.
82. Decide whether vectorized same-bucket retrieval rescues the translated path enough to justify a confirm.

### `INC-0049` screen result
83. Screen analysis landed at `results/analysis/inc0049_retrieval_cost_rescue_screen.json`.
84. `HOPF_RET_P1` and `HOPF_PHI2_RET_P1` kept the same pruning signal while vectorized same-bucket retrieval cut routed online cost sharply.
85. `HOPF_PHI2_RET_P1` became the translated routed online-speed lead:
   - `offline=7.664s`
   - `online=0.299s`
   - `cand_frac=0.3415`
86. `DENSE` kept the single-batch wall-clock lead because it pays no offline route/index build:
   - `offline=0.000s`
   - `online=0.879s`
   - `total=1.332s`
87. Decision: keep translated retrieval alive under amortization; do not promote on single-batch total wall-clock.
88. Queued `INC_0051_retrieval_amortization.md`.

### `INC-0051` staging
89. Updated direction, handoff, and live worklog to make amortization the next active question.
90. Next implementation target: add repeated-query evaluation over one offline routed index.
91. Planned next config: `configs/proxy_transfer_inc0051_retrieval_amortization_screen.json`.

### `INC-0051` run launch
92. Created `configs/proxy_transfer_inc0051_retrieval_amortization_screen.json`.
93. Next command: `python tools/proxy_sweep.py --config configs/proxy_transfer_inc0051_retrieval_amortization_screen.json`.

### `INC-0051` run result
94. Ran `python tools/proxy_sweep.py --config configs/proxy_transfer_inc0051_retrieval_amortization_screen.json`.
95. Screen analysis landed at `results/analysis/inc0051_retrieval_amortization_screen.json`.
96. `HOPF_RET_P1_Q24` became the first translated routed branch to beat matched dense on amortized per-repeat cost:
   - `HOPF_RET_P1_Q24`: `0.5399s`
   - `DENSE_Q24`: `0.5545s`
97. `HOPF_PHI2_RET_P1_Q24` stayed slower than dense on amortized cost despite slightly stronger pruning.
98. Decision: keep plain Hopf as the live translated retrieval branch; demote widened Hopf for this task.
99. Queued `INC_0052_retrieval_amortization_confirm.md`.

### `INC-0052` staging
100. Next target is a 4-seed confirm around the narrow amortized crossover band.
101. Planned compare set:
   - `DENSE_Q24`
   - `DENSE_Q32`
   - `HOPF_RET_P1_Q24`
   - `HOPF_RET_P1_Q32`
102. Created `configs/proxy_transfer_inc0052_retrieval_amortization_confirm.json`.
103. Ran `python tools/proxy_sweep.py --config configs/proxy_transfer_inc0052_retrieval_amortization_confirm.json`.
104. Confirm analysis landed at `results/analysis/inc0052_retrieval_amortization_confirm.json`.
105. The `Q24/Q32` crossover did not survive 4 seeds:
   - `DENSE_Q24`: `0.5051s`
   - `HOPF_RET_P1_Q24`: `0.5938s`
   - `DENSE_Q32`: `0.5586s`
   - `HOPF_RET_P1_Q32`: `0.6544s`
106. Decision: close the translated systems branch without promotion and return next to the dynamic geometry branch.

### `INC-0052` run launch
102. Created `configs/proxy_transfer_inc0052_retrieval_amortization_confirm.json`.
103. Next command: `python tools/proxy_sweep.py --config configs/proxy_transfer_inc0052_retrieval_amortization_confirm.json`.

### `INC-0050` reopening
107. Verified git branch and clean state on `codex/RR-050-dynamic-h4-state`.
108. Read `INC_0050_dynamic_h4_state.md`, `CURRENT_DIRECTION.md`, `HANDOFF_CURRENT.md`, and the prior geometry reviews.
109. Confirmed the dynamic branch should be executed as a new ordered-sequence diagnostic, not by reusing the randomized proxy evaluator.
110. Wrote `docs/research/MATH_REVIEW_DYNAMIC_H4_STATE_20260306.md`.
111. Created `docs/research/LEARNED_KNOWLEDGE.md` as a durable mathematical knowledge file.
112. Added `tasks/dynamic_h4_state_eval.py` for Slice A:
   - static `H^4`
   - tangent surrogate `H^4 + T_xH^4`
   - product surrogate `H^4 x H^4`
113. Extended summary/export support for dynamic metrics in `tools/summarize.py` and `tools/proxy_sweep.py`.
114. Added `tests/test_dynamic_h4_state_eval.py`.
115. Ran targeted validation:
   - `py_compile` passed
   - `python -m unittest tests.test_dynamic_h4_state_eval tests.test_proxy_sweep -v` passed
116. Updated `INC_0050_dynamic_h4_state.md`, `CURRENT_DIRECTION.md`, `HANDOFF_CURRENT.md`, and `LIVE_WORKLOG.md` to reflect Slice A in progress.
117. Pending next action: create `configs/proxy_transfer_inc0050_dynamic_h4_screen.json` and run the 2-seed screen.
118. Created `configs/proxy_transfer_inc0050_dynamic_h4_screen.json`.
119. Ran `python tools/proxy_sweep.py --config configs/proxy_transfer_inc0050_dynamic_h4_screen.json`.
120. Screen analysis landed at `results/analysis/inc0050_dynamic_h4_screen.json`.
121. Screen result:
   - `TXH4_W050` beat static `H^4` on proxy MSE and runtime
   - product `H^4 x H^4` branches improved top-1 but not main MSE
122. Promotion decision:
   - confirm `TXH4_W050`
   - keep one restrained product comparator (`H4XH4_W025`)
123. Created `configs/proxy_transfer_inc0050_dynamic_h4_confirm.json`.
124. Ran `python tools/proxy_sweep.py --config configs/proxy_transfer_inc0050_dynamic_h4_confirm.json`.
125. Confirm analysis landed at `results/analysis/inc0050_dynamic_h4_confirm.json`.
126. Confirm result:
   - `STATIC_H4`: `0.004314443`, `top1=0.02758`, `8.569s`
   - `TXH4_W050`: `0.004303599`, `top1=0.03200`, `8.458s`
   - `H4XH4_W025`: `0.004305430`, `top1=0.03767`, `8.454s`
127. Decision:
   - tangent surrogate `H^4 + T_xH^4` becomes the primary dynamic implementation path
   - product `H^4 x H^4` stays alive as a secondary top-1 / retrieval field branch
128. Added next queued increments:
   - `INC_0054_tangent_flow_route_law.md`
   - `INC_0055_product_h4x4_retrieval_field.md`
129. Added agent handoff packet:
   - `docs/agents/packets/PACKET_DYNAMIC_STATE_AGENT.md`
130. Switched to branch `codex/RR-054-tangent-flow-route-law`.
131. Read `INC_0054_tangent_flow_route_law.md`, `CURRENT_DIRECTION.md`, and `PACKET_DYNAMIC_STATE_AGENT.md`.
132. Implemented `candidate_mode=global_knn|static_bucket_knn` in `tasks/dynamic_h4_state_eval.py`.
133. Added same-bucket dynamic retrieval metrics:
    - `retrieval_candidate_count_mean`
    - `retrieval_candidate_fraction_mean`
    - `retrieval_probe_bucket_mean`
    - `retrieval_bucket_fallback_rate`
134. Added RR-054 test coverage in `tests/test_dynamic_h4_state_eval.py`.
135. Added config `configs/proxy_transfer_inc0054_tangent_flow_route_law_screen.json`.
136. First RR-054 screen failed because `STATIC_GLOBAL` referenced `train_pos` instead of `pos_tr`; baseline logs were invalid.
137. Fixed the evaluator bug in `tasks/dynamic_h4_state_eval.py`.
138. Re-ran targeted validation:
    - `py_compile` passed
    - `python -m unittest tests.test_dynamic_h4_state_eval -v` passed
139. Re-ran the full RR-054 screen successfully.
140. RR-054 corrected result:
    - static Hopf bucket keys cut candidate fraction to about `0.34`
    - bucket fallback stayed `0.0`
    - `STATIC_GLOBAL` kept best MSE
    - `TXH4_BUCKET_W050` became the best bucketed runtime/MSE branch
    - `H4XH4_BUCKET_W025` kept the best bucketed top-1
141. Decision:
    - close RR-054 without confirm
    - promote RR-055 as the next dynamic branch
    - carry forward the hypothesis that the second `H^4` may store route keys in a discrete complex / imaginary field
142. Switched to branch `codex/RR-055-product-h4x4-retrieval-field` and fast-forwarded it to RR-054.
143. Added `route_key_mode=hopf_bucket|hopf_plus_complex` to `tasks/dynamic_h4_state_eval.py`.
144. Added discrete complex secondary-key support:
    - `complex_key_roots`
    - `complex_key_radius_bins`
    - `retrieval_secondary_key_count`
145. Added product branch packet:
146. Switched to branch `codex/RR-056-product-complex-translation` and completed translated complex-key confirm.
147. Closed RR-056 as positive and moved next to RR-057 hierarchical backfill.
148. Switched to branch `codex/RR-057-product-complex-backfill`.
149. Implemented bounded coarse backfill in `tasks/router_retrieval_eval.py`.
150. Added `test_complex_backfill_can_recover_coarse_neighbor` in `tests/test_router_retrieval_eval.py`.
151. First RR-057 screen showed broad backfill was pathologically slow because coarse extras were recomputed per query.
152. Optimized RR-057 by precomputing coarse extra pools per composite key.
153. Observed that broad `BF4/BF8` remained materially heavier than exact complex addressing even after that fix.
154. Started selective RR-057 work:
    - `complex_backfill_mode=always|small_bucket|low_margin`
    - `complex_backfill_max_exact`
    - `complex_backfill_margin_threshold`
155. Extended translated metrics with:
    - `retrieval_backfill_trigger_rate`
    - `retrieval_backfill_extra_candidates_mean`
156. Updated retrieval tests, sweep summaries, and summary CSV support for the new metrics.
157. Validated the interrupted retrieval path fix with compile plus targeted unit tests.
158. Launched a mixed selective screen including small-bucket and low-margin routes.
159. Read live seed-0 summaries during the run and identified low-margin over-trigger as the failure mode.
160. Killed the mixed screen once `M002` showed pathological amortized cost and `M005` was entering the same path.
161. Created the narrowed small-bucket-only screen config:
    - `configs/proxy_transfer_inc0057_product_complex_backfill_smallbucket_screen.json`
162. Ran the narrowed 2-seed screen to completion.
163. Result:
    - `SB1/SB2` preserved the complex-key pruning gain
    - triggers stayed near zero
    - top-1 did not improve at all
164. Decision:
    - close RR-057 negative
    - do not spend more time on candidate-expansion backfill
    - queue RR-058 exact-bucket rerank as the next translated recall branch
165. Committed RR-057 closure on `codex/RR-057-product-complex-backfill`:
    - commit `501ade0`
166. Created and switched to branch `codex/RR-058-product-complex-rerank`.
167. Implemented no-expansion rerank controls in `tasks/router_retrieval_eval.py`:
    - `complex_rerank_mode`
    - `complex_rerank_lambda`
168. Added a unit test proving the complex-plane rerank can flip ordering without changing candidate count.
169. Created `configs/proxy_transfer_inc0058_product_complex_rerank_screen.json`.
170. Ran the RR-058 2-seed screen.
171. RR-058 result:
    - candidate fraction stayed fixed as intended
    - small lambdas were neutral or slightly worse
    - large lambda improved top-1 only marginally and at unacceptable cost
172. Decision:
    - close RR-058 negative
    - stop local translated patching for now
    - queue RR-059 coupled `H^4 x H^4` polar flow as the next branch
    - `docs/agents/packets/PACKET_PRODUCT_H4X4_AGENT.md`
146. Added RR-055 screen config:
    - `configs/proxy_transfer_inc0055_product_h4x4_retrieval_field_screen.json`
147. Ran RR-055 2-seed screen.
148. Screen reading:
    - complex key reduced candidate fraction from about `0.341` to `0.280`
    - fallback stayed near zero
    - runtime improved materially
    - quality regressed slightly
149. Promoted `H4XH4_CPX13_W025` to 4-seed confirm against `H4XH4_BUCKET_W025`.
150. Added RR-055 confirm config:
    - `configs/proxy_transfer_inc0055_product_h4x4_retrieval_field_confirm.json`
151. Ran RR-055 confirm.
152. Confirm result:
    - `H4XH4_BUCKET_W025`: quality/top-1 reference
    - `H4XH4_CPX13_W025`: discrete-key efficiency lead
153. Decision:
    - close RR-055 as a positive product retrieval-field branch
    - promote translation/integration of the complex key law as the next slice
154. Committed RR-055 on branch `codex/RR-055-product-h4x4-retrieval-field`:
    - commit `2cc10f7`
155. Created and checked out `codex/RR-056-product-complex-translation`.
156. Extended `tasks/router_retrieval_eval.py` with translated complex route-key support:
    - `route_key_mode=hopf_bucket|hopf_plus_complex`
    - `complex_key_roots`
    - `complex_key_radius_bins`
157. Generalized translated retrieval keys from 2-tuples to generic tuple keys.
158. Added translated complex-key helpers:
    - `complex_key_ids`
    - `augment_route_keys_with_complex`
159. Added translated retrieval metrics:
    - `retrieval_bucket_fallback_rate`
    - `retrieval_secondary_key_count`
160. Added translated unit coverage in `tests/test_router_retrieval_eval.py`.
161. Ran targeted verification:
    - `py_compile`
    - `python -m unittest tests.test_router_retrieval_eval tests.test_proxy_sweep -v`
162. Added RR-056 screen config:
    - `configs/proxy_transfer_inc0056_product_complex_translation_screen.json`
163. Ran RR-056 2-seed screen.
164. Screen reading:
    - `HOPF_RET_CPX_P1_Q24` cut candidate fraction from `0.3488` to `0.2075`
    - online cost improved from `0.2935s` to `0.2698s` per repeat
    - fallback stayed at `0.0000`
    - MSE improved slightly versus plain Hopf translated retrieval
165. Promoted `HOPF_RET_CPX_P1_Q24` to 4-seed confirm.
166. Added RR-056 confirm config:
    - `configs/proxy_transfer_inc0056_product_complex_translation_confirm.json`
167. Ran RR-056 4-seed confirm.
168. Confirm result:
    - `HOPF_RET_CPX_P1_Q24` beat plain Hopf translated retrieval on candidate fraction, online cost, amortized cost, and proxy MSE
    - dense exact still kept a small top-1 edge
169. Decision:
    - close RR-056 as a positive translated complex-key branch
    - queue hierarchical complex backfill as RR-057 to repair top-1 without giving back pruning
170. Committed RR-056 on branch `codex/RR-056-product-complex-translation`:
    - commit `2a4c6e6`
171. Created and checked out `codex/RR-057-product-complex-backfill`.
172. Implemented `complex_backfill_items` in `tasks/router_retrieval_eval.py`.
173. Added unit coverage for translated hierarchical backfill in `tests/test_router_retrieval_eval.py`.
174. Ran targeted RR-057 verification:
    - `py_compile`
    - `python -m unittest tests.test_router_retrieval_eval -v`
175. Added RR-057 screen config:
    - `configs/proxy_transfer_inc0057_product_complex_backfill_screen.json`
176. First RR-057 screen attempt exposed a pathologically slow naive backfill path.
177. Root cause:
    - coarse extra pools were being recomputed per query.
178. Killed the first RR-057 screen and optimized backfill by precomputing coarse extra pools per composite key.
179. Added optimized rerun config:
    - `configs/proxy_transfer_inc0057_product_complex_backfill_screen_v2.json`
180. Reran RR-057 screen under v2.
181. Partial reading from live observation:
    - `BF4` remained materially heavier than exact complex addressing even after the optimization.
182. Decision for current checkpoint:
    - keep RR-057 open
    - treat selective or cached backfill as the likely next refinement, not broad fixed-size coarse augmentation
