# Decisions Log

## 2026-03-05 (import snapshot)
- Prioritize speed + automation above new features.
- Baseline (kmeans, lambda=0) is the yardstick.
- Treat time-pressure as "maybe later" until iteration is cheap.

Add new entries below.

## 2026-03-06 (research increment INC-0035 Slice A)
- Added live Poincare-ball global-alignment diagnostics to:
  - `hyperbolic_router_so8.py`
  - `tasks/router_proxy_eval.py`
  - `tools/summarize.py`
  - `tools/proxy_sweep.py`
- Added direct invariant tests:
  - rotation preserves alignment
  - global scaling breaks alignment
- Ran diagnostic screen:
  - `configs/proxy_transfer_inc0035_alignment_diag_screen.json`
  - analysis: `results/analysis/inc0035_alignment_diag_screen.json`
  - gate note: `docs/governance/gates/gate_20260306_030909.md`
- Mean diagnostic result:
  - `HOPF_PHI2_BAND`: `0.003910130`, `37.623s`, `align_pair_mae=0.103762`, `align_pair_corr=0.840958`
  - `PHASE_K25_C035`: `0.003923775`, `35.415s`, `align_pair_mae=0.231601`, `align_pair_corr=0.678286`
  - `HOPF_K25_BASE`: `0.003934246`, `36.150s`, `align_pair_mae=0.118315`, `align_pair_corr=0.833911`
  - `R0`: `0.003946221`, `36.584s`, `align_pair_mae=0.147991`, `align_pair_corr=0.799078`
- Decision:
  - keep the alignment metric as a permanent part of the experiment contract
  - do not change route leadership from this fast diagnostic screen
  - treat `HOPF_PHI2_BAND` as the best-aligned widened geometry candidate on this slice
  - move next to a low-rank shell-anchor pilot inside `INC-0035`

## 2026-03-06 (research increment INC-0035 Slice B)
- Implemented `phase4d_hopf_ball`:
  - same Hopf angular allocator
  - shells anchored to original-ball geodesic radius
- Ran shell-anchor screen:
  - `configs/proxy_transfer_inc0035_shell_anchor_screen.json`
  - analysis: `results/analysis/inc0035_shell_anchor_screen.json`
  - gate note: `docs/governance/gates/gate_20260306_032618.md`
- Mean result:
  - `HOPF_PHI2_BAND`: `0.003910130`, `39.743s`, `shell_pmax=0.5583`, `align_pair_mae=0.103762`
  - `HOPF_K25_BALL`: `0.003923518`, `39.581s`, `shell_pmax=0.7988`, `align_pair_mae=0.157295`
  - `PHASE_K25_C035`: `0.003923775`, `40.472s`, `shell_pmax=0.5754`, `align_pair_mae=0.231601`
  - `HOPF_K25_BASE`: `0.003934246`, `37.032s`, `shell_pmax=0.5275`, `align_pair_mae=0.118315`
  - `R0`: `0.003946221`, `30.975s`, collapsed
- Decision:
  - kill naive shell anchoring as the primary global-alignment fix
  - keep `HOPF_K25_BALL` only as a negative control
  - move next to a chart-isometry / shared-route-coordinate pilot
  - treat shell-only repair as mathematically insufficient in the current architecture

## 2026-03-06 (research increment INC-0036)
- Implemented `phase4d_hopf_iso`:
  - routing uses the learned rotation only
  - learned chart scale is ignored for shells and sectors
- Added direct invariance tests:
  - `apply_chart_isometric` preserves Poincare alignment under chart scaling
  - `phase4d_hopf_iso` matches `phase4d_hopf` when the chart is already rotation-only
- Ran chart-isometry screen:
  - `configs/proxy_transfer_inc0036_chart_iso_screen.json`
  - analysis: `results/analysis/inc0036_chart_iso_screen.json`
  - gate note: `docs/governance/gates/gate_20260306_074531.md`
- Mean result:
  - `HOPF_PHI2_BAND`: `0.003910130`, `30.702s`, `pair_mae=0.103762`, health pass
  - `PHASE_K25_C035`: `0.003923775`, `29.103s`, `pair_mae=0.231601`, health pass
  - `HOPF_K25_ISO`: `0.003926004`, `42.004s`, `pair_mae=0.000000`, health fail on runtime only
  - `HOPF_K25_BASE`: `0.003934246`, `33.220s`, `pair_mae=0.118315`, health pass
  - `R0`: `0.003946221`, `31.958s`, collapsed
- Decision:
  - keep `phase4d_hopf_iso` as a positive geometry diagnostic and a negative operational result
  - do not promote pure isometry as a route lead
  - keep `HOPF_K25_BASE` as the routed-quality lead
  - keep `HOPF_PHI2_BAND` as the widened Hopf geometry candidate
  - move next to an isometric-band route rather than another shell-only or pure-isometry variant

## 2026-03-06 (research increment INC-0037)
- Implemented `phase4d_hopf_fib_band_iso`:
  - banded Hopf widening on a rotation-only route coordinate
- Ran screen:
  - `configs/proxy_transfer_inc0037_isometric_band_screen.json`
  - analysis: `results/analysis/inc0037_isometric_band_screen.json`
  - gate note: `docs/governance/gates/gate_20260306_075923.md`
- Mean result:
  - `HOPF_PHI2_BAND`: `0.003910130`, `30.184s`, `pair_mae=0.103762`, health pass
  - `HOPF_PHI2_BAND_ISO`: `0.003924123`, `43.879s`, `pair_mae=0.000000`, runtime fail
  - `HOPF_K25_ISO`: `0.003926004`, `37.821s`, `pair_mae=0.000000`, runtime fail
  - `HOPF_K25_BASE`: `0.003934246`, `29.686s`, `pair_mae=0.118315`, health pass
  - `R0`: `0.003946221`, `28.349s`, collapsed
- Decision:
  - exact alignment and widened capacity can coexist
  - the runtime penalty is still too large
  - move next to bounded isometry, not more pure-isometry variants

## 2026-03-06 (research increment INC-0038)
- Implemented `phase4d_hopf_fib_band_bound` plus `route_scale_lambda`.
- Ran screen:
  - `configs/proxy_transfer_inc0038_bounded_band_screen.json`
  - analysis: `results/analysis/inc0038_bounded_band_screen.json`
  - gate note: `docs/governance/gates/gate_20260306_082106.md`
- Mean result:
  - `HOPF_PHI2_BAND`: `0.003910130`, `35.837s`, `pair_mae=0.103762`, health pass
  - `HOPF_PHI2_BAND_B075`: `0.003914296`, `46.314s`, `pair_mae=0.077111`, runtime fail
  - `HOPF_PHI2_BAND_B025`: `0.003918418`, `56.210s`, `pair_mae=0.004458`, runtime fail
  - `HOPF_PHI2_BAND_ISO`: `0.003924123`, `42.842s`, `pair_mae=0.000000`, runtime fail
  - `HOPF_PHI2_BAND_B050`: `0.003925862`, `46.515s`, `pair_mae=0.055238`, runtime fail
  - `HOPF_K25_BASE`: `0.003934246`, `33.363s`, `pair_mae=0.118315`, health pass
  - `R0`: `0.003946221`, `32.968s`, collapsed
- Decision:
  - bounded isometry behaves like a clean alignment/runtime interpolation
  - no bounded point passed the operational gate
  - keep the bounded family only as diagnostic evidence
  - move next to route/memory coordinate separation

## 2026-03-06 (research increment INC-0039)
- Added `memory_coord_mode={route_chart,full_chart}` so routing keys can stay geometry-aligned while memory/prototypes optionally use the full chart coordinate.
- Ran screen:
  - `configs/proxy_transfer_inc0039_route_memory_screen.json`
  - analysis: `results/analysis/inc0039_route_memory_screen.json`
  - gate note: `docs/governance/gates/gate_20260306_084204.md`
- Mean result:
  - `HOPF_PHI2_BAND`: `0.003910130`, `45.122s`, `pair_mae=0.103762`, health pass
  - `DUAL_B050`: `0.003915606`, `60.174s`, `pair_mae=0.055238`, runtime fail
  - `DUAL_B025`: `0.003919752`, `56.867s`, `pair_mae=0.004458`, runtime fail
  - `DUAL_B075`: `0.003925511`, `58.174s`, runtime fail
  - `HOPF_K25_BASE`: `0.003934246`, `41.476s`, health pass
  - `R0`: `0.003946221`, `45.127s`, collapsed
- Decision:
  - route/memory separation improved geometry, but not enough operationally
  - keep `HOPF_K25_BASE` and `HOPF_PHI2_BAND` as the active Hopf frontier
  - stop opening new geometry branches for one step
  - move next to a strict frontier confirm

## 2026-03-06 (control CTRL-0003)
- Ran strict 4-seed frontier confirm:
  - `configs/proxy_transfer_ctrl0003_hopf_frontier_confirm.json`
  - analysis: `results/analysis/ctrl0003_hopf_frontier_confirm.json`
  - gate note: `docs/governance/gates/gate_20260306_085323.md`
- 4-seed means:
  - `HOPF_K25_BASE`: `0.003937984`, `44.838s`, `shells=2.25`, `sectors=4.0`, health pass
  - `HOPF_PHI2_BAND`: `0.003921230`, `51.541s`, `shells=2.25`, `sectors=9.25`, runtime fail
  - `R0`: `0.003946853`, `42.409s`, shell-collapse health fail
- Timing read:
  - `HOPF_K25_BASE`: `chart_opt=39.959s`, `training_ema=4.143s`
  - `HOPF_PHI2_BAND`: `chart_opt=40.658s`, `training_ema=10.133s`
  - `R0`: `chart_opt=40.634s`, `training_ema=0.856s`
- Decision:
  - promote `HOPF_K25_BASE` as the current healthiest routed branch
  - keep `HOPF_PHI2_BAND` as the widened-quality candidate only
  - keep `R0` as the transfer control baseline, but explicitly note it fails the route-health standard
  - move next to cost decomposition rather than another geometry branch

## 2026-03-06 (research increment INC-0040 screen)
- Added explicit cost reporting tool:
  - `tools/cost_report.py`
  - report artifact: `docs/reports/HOPF_COST_DECOMPOSITION.md`
- Ran cost-only screen:
  - `configs/proxy_transfer_inc0040_cost_screen.json`
  - analysis: `results/analysis/inc0040_cost_screen.json`
  - gate note: `docs/governance/gates/gate_20260306_091429.md`
- 2-seed means:
  - `HOPF_K25_BASE_IT60_P4`: `0.003925725`, `20.532s`, `chart_opt=11.214s`, health pass
  - `HOPF_PHI2_BAND_IT60_P4`: `0.003937058`, `18.286s`, `chart_opt=11.559s`, health pass
  - `HOPF_PHI2_BAND`: `0.003910130`, `52.211s`, health pass
  - `R0`: `0.003946221`, `46.039s`, shell-collapse health fail
- Decision:
  - chart schedule was the dominant runtime lever
  - both reduced-schedule routes are confirm-worthy
  - send only the reduced pure Hopf and reduced widened Hopf variants to 4-seed confirm, with `R0` and the old widened reference as anchors

## 2026-03-06 (research increment INC-0040 confirm)
- Ran 4-seed cost confirm:
  - `configs/proxy_transfer_inc0040_cost_confirm.json`
  - analysis: `results/analysis/inc0040_cost_confirm.json`
  - gate note: `docs/governance/gates/gate_20260306_092503.md`
- 4-seed means:
  - `HOPF_K25_BASE_IT60_P4`: `0.003919349`, `19.905s`, `chart_opt=12.421s`, `training_ema=6.520s`, health pass
  - `HOPF_PHI2_BAND_IT60_P4`: `0.003928139`, `18.270s`, `chart_opt=10.387s`, `training_ema=7.202s`, health pass
  - `HOPF_PHI2_BAND`: `0.003921230`, `58.684s`, runtime fail
  - `R0`: `0.003946853`, `44.240s`, shell-collapse health fail
- Decision:
  - promote `HOPF_K25_BASE_IT60_P4` as the operational routed lead
  - promote `HOPF_PHI2_BAND_IT60_P4` as the widened efficient lead
  - demote the old full-schedule `HOPF_PHI2_BAND` reference to historical comparison status
  - make larger-subset cost-frontier confirmation the next live branch

## 2026-03-06 (research increment INC-0041)
- Ran 4-seed larger-subset cost confirm:
  - `configs/proxy_transfer_inc0041_cost_large_subset.json`
  - analysis: `results/analysis/inc0041_cost_large_subset.json`
  - gate note: `docs/governance/gates/gate_20260306_093641.md`
- 4-seed means:
  - `HOPF_K25_BASE_IT60_P4`: `0.003895705`, `37.090s`, `chart_opt=19.655s`, `training_ema=16.140s`, runtime fail
  - `HOPF_PHI2_BAND_IT60_P4`: `0.003904061`, `47.604s`, `chart_opt=25.087s`, `training_ema=21.057s`, runtime fail
  - `R0`: `0.003913707`, `27.271s`, shell-collapse health fail
- Decision:
  - the smaller-subset cost rescue does not hold as an operational runtime win under larger load
  - keep `HOPF_K25_BASE_IT60_P4` as the best large-subset quality candidate
  - keep `HOPF_PHI2_BAND_IT60_P4` as the widened large-subset candidate, but behind reduced pure Hopf
  - move next to large-subset EMA/chart pressure rather than reopening geometry

## 2026-03-06 (research increment INC-0042)
- Ran large-subset timing decomposition:
  - `configs/proxy_transfer_inc0042_timing_diag.json`
  - analysis: `results/analysis/inc0042_timing_diag.json`
  - gate note: `docs/governance/gates/gate_20260306_094708.md`
- Timing read:
  - `HOPF_K25_BASE_IT60_P4`: `chart_opt=20.011s`, `training_route=14.913s`, `training_update=0.120s`
  - `HOPF_PHI2_BAND_IT60_P4`: `chart_opt=20.573s`, `training_route=11.798s`, `training_update=0.117s`
  - `R0`: `chart_opt=28.073s`, `training_route=1.768s`, `training_update=0.086s`
- Decision:
  - the large-subset EMA problem was almost entirely per-step training rerouting
  - post-growth knobs are not the right next lever
  - promote static training-route reuse as the next live systems branch

## 2026-03-06 (research increment INC-0043 screen)
- Ran static training-route screen:
  - `configs/proxy_transfer_inc0043_train_route_static_screen.json`
  - analysis: `results/analysis/inc0043_train_route_static_screen.json`
  - gate note: `docs/governance/gates/gate_20260306_095825.md`
- 2-seed screen means:
  - `HOPF_K25_BASE_IT60_P4`: `0.003896506`, `26.597s`, runtime fail
  - `HOPF_K25_BASE_IT60_P4_STATIC`: `0.003897352`, `17.783s`, health pass
  - `HOPF_PHI2_BAND_IT60_P4_STATIC`: `0.003896989`, `18.263s`, health pass
  - `R0`: `0.003912808`, `19.094s`, shell-collapse fail
- Decision:
  - static training-route reuse is a live frontier branch
  - promote only the static routed variants to 4-seed confirm, with dynamic Hopf and `R0` as controls

## 2026-03-06 (research increment INC-0043 confirm)
- Ran 4-seed static training-route confirm:
  - `configs/proxy_transfer_inc0043_train_route_static_confirm.json`
  - analysis: `results/analysis/inc0043_train_route_static_confirm.json`
  - gate note: `docs/governance/gates/gate_20260306_100530.md`
- 4-seed means:
  - `HOPF_K25_BASE_IT60_P4`: `0.003895705`, `32.034s`, runtime fail vs `R0`
  - `HOPF_K25_BASE_IT60_P4_STATIC`: `0.003899506`, `19.798s`, health pass
  - `HOPF_PHI2_BAND_IT60_P4_STATIC`: `0.003902306`, `19.602s`, health pass
  - `R0`: `0.003913707`, `22.520s`, shell-collapse fail
- Timing read:
  - `HOPF_K25_BASE_IT60_P4_STATIC`: `chart_opt=18.385s`, `training_route=0.003s`, `training_update=0.094s`
  - `HOPF_PHI2_BAND_IT60_P4_STATIC`: `chart_opt=18.240s`, `training_route=0.003s`, `training_update=0.067s`
- Decision:
  - promote `HOPF_PHI2_BAND_IT60_P4_STATIC` as the operational routed lead
  - promote `HOPF_K25_BASE_IT60_P4_STATIC` as the quality-balanced routed lead
  - hold geometry fixed for the next step
  - move next to static-frontier chart pressure

## 2026-03-06 (research increment INC-0044 screen)
- Ran static-frontier chart-pressure screen:
  - `configs/proxy_transfer_inc0044_static_chart_pressure_screen.json`
  - analysis: `results/analysis/inc0044_static_chart_pressure_screen.json`
  - gate note: `docs/governance/gates/gate_20260306_101427.md`
- 2-seed screen means:
  - `HOPF_PHI2_BAND_IT48_P3_STATIC`: `0.003902583`, `10.298s`, health pass
  - `HOPF_K25_BASE_IT48_P3_STATIC`: `0.003908684`, `13.028s`, health pass
  - `HOPF_K25_BASE_IT60_P4_STATIC`: `0.003897352`, `16.934s`, health pass
  - `HOPF_PHI2_BAND_IT60_P4_STATIC`: `0.003896989`, `20.816s`, runtime fail vs cheap `R0`
  - `R0`: `0.003920084`, `15.524s`, shell-collapse fail
- Decision:
  - cheaper chart pressure is a live lever on the widened static branch
  - promote `HOPF_PHI2_BAND_IT48_P3_STATIC` to 4-seed confirm
  - keep `HOPF_K25_BASE_IT60_P4_STATIC` and `HOPF_PHI2_BAND_IT60_P4_STATIC` as controls

## 2026-03-06 (research increment INC-0044 confirm)
- Ran 4-seed static-frontier chart-pressure confirm:
  - `configs/proxy_transfer_inc0044_static_chart_pressure_confirm.json`
  - analysis: `results/analysis/inc0044_static_chart_pressure_confirm.json`
  - gate note: `docs/governance/gates/gate_20260306_102058.md`
- 4-seed means:
  - `HOPF_PHI2_BAND_IT48_P3_STATIC`: `0.003901257`, `17.217s`, health pass
  - `HOPF_K25_BASE_IT60_P4_STATIC`: `0.003899506`, `24.155s`, runtime fail vs cheap `R0`
  - `HOPF_PHI2_BAND_IT60_P4_STATIC`: `0.003902306`, `20.963s`, runtime fail vs cheap `R0`
  - `R0`: `0.003922779`, `16.183s`, shell-collapse fail
- Timing read:
  - `HOPF_PHI2_BAND_IT48_P3_STATIC`: `chart_opt=15.728s`, `training_ema=0.090s`
  - `R0`: `chart_opt=11.373s`, `training_ema=3.234s`
- Decision:
  - promote `HOPF_PHI2_BAND_IT48_P3_STATIC` as the current strict-gate routed lead under the cheaper common schedule
  - do not claim an absolute runtime win vs cheap `R0`
  - move next to one more chart-floor step before reopening geometry

## 2026-03-06 (research increment INC-0045 screen)
- Ran static routed chart-floor screen:
  - `configs/proxy_transfer_inc0045_static_chart_floor_screen.json`
  - analysis: `results/analysis/inc0045_static_chart_floor_screen.json`
  - gate note: `docs/governance/gates/gate_20260306_103538.md`
- 2-seed screen means:
  - `HOPF_K25_BASE_IT40_P2_STATIC`: `0.003902717`, `5.725s`, health pass
  - `HOPF_PHI2_BAND_IT40_P2_STATIC`: `0.003904835`, `6.488s`, health pass
  - `R0`: `0.003916428`, `8.152s`, shell-collapse fail
- Decision:
  - one more chart-floor step is live
  - promote both `IT40_P2_STATIC` routed branches to 4-seed confirm

## 2026-03-06 (research increment INC-0045 confirm)
- Ran 4-seed static routed chart-floor confirm:
  - `configs/proxy_transfer_inc0045_static_chart_floor_confirm.json`
  - analysis: `results/analysis/inc0045_static_chart_floor_confirm.json`
  - gate note: `docs/governance/gates/gate_20260306_103811.md`
- 4-seed means:
  - `HOPF_K25_BASE_IT40_P2_STATIC`: `0.003895098`, `6.800s`, health pass
  - `HOPF_PHI2_BAND_IT40_P2_STATIC`: `0.003903409`, `7.176s`, health pass
  - `R0`: `0.003911417`, `8.334s`, shell-collapse fail
- Timing read:
  - `HOPF_K25_BASE_IT40_P2_STATIC`: `chart_opt=5.527s`, `training_ema=0.078s`
  - `HOPF_PHI2_BAND_IT40_P2_STATIC`: `chart_opt=6.045s`, `training_ema=0.081s`
  - `R0`: `chart_opt=5.129s`, `training_ema=1.686s`
- Decision:
  - promote `HOPF_K25_BASE_IT40_P2_STATIC` as the operational routed lead
  - promote `HOPF_PHI2_BAND_IT40_P2_STATIC` as the widened cheap routed lead
  - move next to scale robustness

## 2026-03-06 (research increment INC-0046 screen)
- Ran static routed scale-robustness screen:
  - `configs/proxy_transfer_inc0046_static_scale_robustness_screen.json`
  - analysis: `results/analysis/inc0046_static_scale_robustness_screen.json`
  - gate note: `docs/governance/gates/gate_20260306_104728.md`
- 2-seed screen means:
  - `HOPF_K25_BASE_IT40_P2_STATIC`: `0.003886145`, `12.201s`, health pass
  - `HOPF_PHI2_BAND_IT40_P2_STATIC`: `0.003905894`, `11.685s`, health pass
  - `R0`: `0.003891917`, `15.917s`, shell-collapse fail
- Decision:
  - the cheap routed win survived the next larger subset step
  - promote both routed branches to 4-seed confirm to resolve quality-vs-runtime leadership

## 2026-03-06 (research increment INC-0046 confirm)
- Ran 4-seed static routed scale-robustness confirm:
  - `configs/proxy_transfer_inc0046_static_scale_robustness_confirm.json`
  - analysis: `results/analysis/inc0046_static_scale_robustness_confirm.json`
  - gate note: `docs/governance/gates/gate_20260306_105119.md`
- 4-seed means:
  - `HOPF_K25_BASE_IT40_P2_STATIC`: `0.003884370`, `11.035s`, health pass
  - `HOPF_PHI2_BAND_IT40_P2_STATIC`: `0.003900404`, `10.186s`, health pass
  - `R0`: `0.003892404`, `18.872s`, shell-collapse fail
- Timing read:
  - `HOPF_K25_BASE_IT40_P2_STATIC`: `chart_opt=9.170s`, `training_ema=0.138s`
  - `HOPF_PHI2_BAND_IT40_P2_STATIC`: `chart_opt=8.629s`, `training_ema=0.110s`
  - `R0`: `chart_opt=10.716s`, `training_ema=5.628s`
- Decision:
  - keep `HOPF_K25_BASE_IT40_P2_STATIC` as the operational routed lead
  - keep `HOPF_PHI2_BAND_IT40_P2_STATIC` as the hardware-efficiency routed lead
  - move next to near-full-proxy scale

## 2026-03-06 (research increment INC-0034)
- Implemented `phase4d_hopf_blend` with:
  - `hopf_blend_lambda`
  - `hopf_blend_chi_weight`
  - `hopf_blend_shell_weight`
- First screen attempt exposed a proxy-evaluator diagnostics bug for the new mode.
- Fixed the evaluator, reran tests, and reran the full 2-seed screen.
- Final 2-seed screen means:
  - `HOPF_K25_BASE`: `0.003888756`, `62.885s`, `sectors=4.0`, `chi_bin_pmax=0.7834`
  - `HOPF_PHI2_BAND`: `0.003897103`, `62.094s`, `sectors=10.5`, `chi_bin_pmax=0.9418`
  - `HOPF_BLEND_L110_C15_S05`: `0.003911125`, `59.899s`, `sectors=8.0`, `chi_bin_pmax=0.7628`
  - `HOPF_BLEND_L080_C10_S05`: `0.003914592`, `68.800s`, `sectors=9.0`, `chi_bin_pmax=0.7720`
  - `PHASE_K25_C035`: `0.003909488`, `58.010s`
  - `R0`: `0.003911258`, `45.506s`
- Decision:
  - keep `HOPF_K25_BASE` as the routed-quality lead
  - keep `HOPF_PHI2_BAND` as the widened Hopf geometry candidate
  - kill `phase4d_hopf_blend` as the active next branch
  - move the next primary branch to stronger Poincare-ball global alignment

## 2026-03-05 (program implementation kickoff)
- Locked structured run output contract: final line must be `__JSON_SUMMARY__ {...}` schema v1.0.
- Added runtime controls: `fast_dev`, cache knobs, and chart early-stop to collapse iteration latency.
- Added staged sweep and gate-note pipeline as default orchestration path.
- Added real-task priority scaffolding (WikiText-2 proxy + dense baseline comparator) for transfer checks.
- Added route extensions (`phase4d`, `complex2`) behind explicit matrix and kill criteria.

## 2026-03-05 (research increment INC-0001)
- Ran non-fast-dev phase4d validation sweep (`configs/route_sweep_phase4d_validation.json`) with stage policy 1/2/4 seeds.
- Finalize outcome: `R0(kmeans)` outperformed `R5(phase4d)` on mean post-growth MSE (`0.829354` vs `0.837396`).
- `R5` looked better in screen/confirm means, indicating seed-sensitive instability rather than a dead branch.
- Decision: keep `R5` as active branch, but baseline stays `R0` until phase4d dimension sensitivity is tested.
- Next increment queued: phase4d dimension study (`0,1,2,3` vs alternative 4D projections).

## 2026-03-05 (research increment INC-0002)
- Ran fast staged phase4d dimension sweep (`configs/route_sweep_phase4d_dims_fast.json`) over `0,1,2,3`, `0,2,4,6`, `1,3,5,7`.
- Result: `phase4_dims=0,2,4,6` produced the best phase4 finalize mean (`0.936811`) and beat both `R0` (`0.947021`) and other phase4 variants in this fast profile.
- Decision: adopt `0,2,4,6` as the active phase4d candidate dimension set.
- Next increment queued: non-fast-dev validation of `R0` vs `R5B` (`phase4_dims=0,2,4,6`) with stricter runtime-quality comparison.

## 2026-03-05 (research increment INC-0003)
- Ran non-fast-dev R0 vs R5B (`configs/route_sweep_inc0003_r0_vs_r5b.json`) with cache disabled.
- Finalize means (2 seeds): `R0=0.840253`, `R5B=0.819587` post-growth MSE.
- Runtime means (2 seeds): `R0=29.663s`, `R5B=24.350s` total.
- Decision: `R5B` is the provisional leader (better quality and runtime), pending 4-seed non-fast finalize confirmation before full promotion.

## 2026-03-05 (research increment INC-0004)
- Ran 4-seed non-fast finalize confirmation (`configs/route_sweep_inc0004_r0_vs_r5b_finalize4.json`) for R0 vs R5B.
- Finalize means: `R0=0.829354`, `R5B=0.825124` post-growth MSE.
- Finalize runtime means: `R0=32.591s`, `R5B=29.478s`.
- Decision: `R5B` becomes current best-known route; keep R0 as control baseline.

## 2026-03-05 (research increment INC-0005)
- Ran R5B time-pressure ablation (`configs/route_sweep_inc0005_r5b_timepressure.json`) with `lambda in {0.0,0.25,0.5,0.8,1.2}`.
- Baseline `lambda=0.0` outperformed all positive lambda settings on post-growth MSE.
- Decision: keep `time_pressure_lambda=0.0`; positive time pressure remains regressive in current regime.

## 2026-03-05 (research leadership hardening)
- Added a lead-research workflow so future increments are selected by mechanism and direction, not just by available flags.
- Split research fleet responsibilities across lead research, geometry theory, experimental validation, LM transfer, skeptical review, and systems/performance.
- Locked the active direction around `R5B` while explicitly queueing robustness, transfer, and hybrid-routing questions as the next thoughtful branches.

## 2026-03-05 (research increment INC-0006)
- Ran larger-`N` robustness validation (`configs/route_sweep_inc0006_r5b_robustness.json`) for `R0` vs `R5B`.
- Finalize means: `R0=0.835950`, `R5B=0.807154`.
- Finalize runtime means: `R0=69.786s`, `R5B=53.679s`.
- Decision: `R5B` remains the best-known route and is now materially more credible as a real direction.

## 2026-03-05 (research increment INC-0007)
- Built `tasks/router_proxy_eval.py` to run the geometry router directly on LM proxy tensors.
- First PTB proxy smoke (`train=2000`, `test=1000`, `seed=0`) slightly favored `R5B` over `R0` on both post-growth MSE and total runtime.
- Transfer caution: `R5B` collapsed the proxy into only `2` buckets with `pmax_after=0.875`, so the transfer result is promising but not yet healthy.
- Decision: continue transfer work, but require multi-seed larger-subset confirmation before using proxy evidence as a promotion argument.

## 2026-03-05 (research increment INC-0008)
- Ran multi-seed larger-subset PTB proxy transfer (`configs/proxy_transfer_inc0008.json`) through `tools/proxy_sweep.py`.
- Mean proxy results:
  - `R0`: `test_mse_after=0.0039450`, `total_sec=26.112`, `buckets=8.0`, `pmax_after=0.205`
  - `R5B`: `test_mse_after=0.0038858`, `total_sec=23.237`, `buckets=2.0`, `pmax_after=0.877`
- Decision: `R5B` transfer is now repeatably better than `R0` on this proxy, but the mechanism is still unhealthy because traffic collapses into two dominant buckets.
- Direction shift: prioritize transfer stabilization over more generic synthetic flag search.
- Interpretation guardrail: PTB proxy is currently a relative router-comparison harness, not evidence that the whole routed system is already cheaper than the dense baseline in absolute runtime.

## 2026-03-05 (research increment INC-0009)
- Ran fixed-`phase4d` transfer stabilization sweeps (`configs/proxy_transfer_inc0009_screen.json` and `configs/proxy_transfer_inc0009_confirm.json`).
- Result: raising `K` widened active sectors from `2` to `4`, but `chart_beta`, `delta_r`, and extra growth budget did not solve the collapse.
- Best fixed candidate `R5B_K25` still failed the transfer-health gate with `pmax_after=0.675`.
- Decision: fixed `phase4d` stabilization is insufficient. Open an adaptive time-expanded branch.

## 2026-03-05 (research increment INC-0010)
- Added `phase4d_adaptive` plus time-expanded widening controls and formalized the mechanism in `docs/research/ADAPTIVE_PHASE4D_SPEC.md`.
- Ran adaptive transfer confirm (`configs/proxy_transfer_inc0010_adaptive_confirm.json`) against `R0`, `R5B_ref`, and `R5B_K25`.
- `R5A_K25_M3` mean:
  - `test_mse_after=0.0039009`
  - `total_sec=32.664`
  - `buckets=11.0`
  - `pmax_after=0.598`
- Decision: promote `R5A_K25_M3` as the current stabilized proxy-transfer candidate.
- Remaining bottleneck: shell diversity stayed at `1.0`, so the next branch must target radial/time shell activation rather than more sector-only widening.

## 2026-03-05 (research increment INC-0011)
- Added divergence-aware shell geometry to the adaptive phase route:
  - `adaptive_shell_growth`
  - `adaptive_shell_balance`
  - `adaptive_converge_lambda`
- Added shell-aware proxy diagnostics and health gates:
  - `eval_shells`
  - `shell_pmax`
  - shell multiplier / shell drive summaries
- Ran shell screen (`configs/proxy_transfer_inc0011_shell_screen.json`).
- Screen result:
  - `SG08` did not activate shells
  - `SG12` activated shells cleanly
  - `SG16_SB10` activated shells aggressively
- Ran shell confirm (`configs/proxy_transfer_inc0011_shell_confirm.json`).
- Confirm means:
  - `R0`: `test_mse_after=0.0039450`, `total_sec=34.208`, `eval_shells=1.0`
  - `R5A_REF`: `0.0039009`, `33.887`, `eval_shells=1.0`
  - `R5A_SG12`: `0.0039451`, `33.330`, `eval_shells=4.5`, `shell_pmax=0.442`
  - `R5A_SG16_SB10`: `0.0039636`, `35.384`, `eval_shells=40.5`, `shell_pmax=0.190`
- Decision:
  - promote `R5A_SG12` as the new shell-active proxy-transfer lead
  - keep `R5A_REF` as the best raw-MSE collapsed reference
  - keep `R5A_SG16_SB10` as an exploratory over-dispersed branch, not the lead
- Direction shift:
  - shell activation is now proven
  - the next branch is controlled convergence and hysteresis, not more brute-force widening

## 2026-03-05 (research increment INC-0012)
- Replaced the shell convergence law with a target-and-overflow controller:
  - `adaptive_converge_target`
  - `adaptive_converge_hysteresis`
- Added overflow/convergence diagnostics and tightened transfer health with `max_unseen_rate`.
- Ran convergence-control screen (`configs/proxy_transfer_inc0012_convergence_screen.json`).
- Screen result:
  - `R5A_SG12_C10` improved over `R5A_SG12_REF`
  - `R5A_SG16_C10` pulled the over-dispersed branch back into the healthy regime
  - `R5A_SG16_C10_D35` showed that coarser `delta_r` materially improves the strong-divergence branch
  - `R5A_SG16_C15` collapsed back to one shell and was killed
- Ran convergence-control confirm (`configs/proxy_transfer_inc0012_convergence_confirm.json`).
- Confirm means:
  - `R0`: `0.0039450`, `32.666s`, `eval_shells=1.0`
  - `R5A_SG12_REF`: `0.0039451`, `32.291s`, `eval_shells=4.5`, `shell_pmax=0.442`
  - `R5A_SG12_C10`: `0.0039362`, `31.231s`, `eval_shells=2.5`, `shell_pmax=0.542`
  - `R5A_SG16_C10`: `0.0039431`, `28.815s`, `eval_shells=3.5`, `shell_pmax=0.639`
  - `R5A_SG16_C10_D35`: `0.0039278`, `28.430s`, `eval_shells=2.0`, `shell_pmax=0.792`
- Decision:
  - promote `R5A_SG16_C10_D35` as the current stabilized proxy-transfer lead
  - keep `R5A_SG12_C10` as the lower-concentration comparison branch
  - interpret `delta_r` as part of the convergence law, not just a shell indexing constant
- Direction shift:
  - the next problem is not whether convergence helps
  - the next problem is mapping the shell-control phase diagram and replacing hand-tuned radial quantization with a more local merge rule

## 2026-03-05 (research increment INC-0013)
- Ran shell-control phase diagram sweeps:
  - `configs/proxy_transfer_inc0013_phase_diagram_screen.json`
  - `configs/proxy_transfer_inc0013_phase_diagram_confirm.json`
- Mean-gate confirm initially favored `PD_SG18` over the previous lead `PD_CENTER` (`R5A_SG16_C10_D35`) because it recovered runtime while preserving similar mean MSE.
- Research review found a governance flaw:
  - the transfer-health gate only checked route means
  - `PD_SG18` and `PD_CENTER` both crossed the shell concentration wall on `seed1` (`shell_pmax=0.908`)
  - `PD_C12` also slipped through on mean stats despite `seed0` collapsing to one shell
- Hardened the sweep tool by adding `enforce_seed_health` to multi-seed transfer health review.
- Re-scored the completed confirm batch under strict seed review:
  - `PD_D30`: `0.0039431`, `29.851s`, `eval_shells=3.5`, `shell_pmax=0.639`, pass
  - `PD_T080`: `0.0039496`, `31.186s`, `eval_shells=3.0`, `shell_pmax=0.733`, pass
  - `PD_C06`: `0.0039538`, `27.952s`, `eval_shells=9.0`, `shell_pmax=0.400`, pass
  - `PD_CENTER` / `PD_SG18`: fail strict review due seed-wise shell concentration
  - `PD_D40`: stable collapse
- Decision:
  - promote `R5A_SG16_C10_D30` (`PD_D30`) as the provisional strict-health transfer lead
  - demote the `D35` ridge from promoted-lead status until it can pass seed-wise review
  - require seed-wise route-health for future multi-seed transfer promotions
- Direction shift:
  - the phase diagram is real
  - the new question is whether the low-`delta_r` healthy band is robust or simply a different boundary effect

## 2026-03-05 (research increment INC-0014)
- Ran larger-subset strict-health robustness:
  - `configs/proxy_transfer_inc0014_strict_robustness.json`
  - `max_train=5000`, `max_eval=2500`, `seeds=0,1,2,3`, `enforce_seed_health=true`
- Result:
  - `R0` kept the best raw proxy MSE (`0.0039079`) but remained fully collapsed and very slow (`75.653s`)
  - `R5A_SG16_C10_D30`: `0.0039185`, `50.461s`, `eval_shells=3.0`, `shell_pmax=0.579`, strict-health pass
  - `R5A_SG18_C10_D35`: `0.0039183`, `50.966s`, `eval_shells=3.0`, `shell_pmax=0.658`, strict-health pass
  - `R5A_SG16_C10_T080_D35` failed strict review on `seed1` and `seed2`
  - `R5A_SG16_C06_D35`: `0.0039326`, `52.589s`, `eval_shells=7.0`, `shell_pmax=0.482`, strict-health pass
- Decision:
  - keep `R0` as the raw-MSE transfer baseline
  - promote `R5A_SG16_C10_D30` as the hardware-efficiency transfer lead
  - keep `R5A_SG18_C10_D35` as the nearest challenger
  - demote `R5A_SG16_C10_T080_D35`
- Governance correction:
  - recommendation logic now promotes healthy faster routes that stay within the configured MSE tolerance, instead of requiring a raw-MSE win
- Direction shift:
  - transfer evaluation now has a real Pareto split:
    - `R0` = raw quality baseline
    - `R5A_SG16_C10_D30` = hardware-efficiency lead
  - the next task is not another broad sweep
  - the next task is explaining the `D30` vs `SG18` ridge mathematically

## 2026-03-05 (research increment INC-0015)
- Ran a narrow larger-subset ridge sweep:
  - `configs/proxy_transfer_inc0015_ridge_discrimination.json`
  - `delta_r in {2.8,3.0,3.2,3.5}`
  - `adaptive_shell_growth in {1.6,1.8}`
  - `seeds=0,1`
- Result:
  - paired routes at the same `delta_r` were effectively identical on MSE, shell count, shell concentration, unseen-rate, and `adaptive_shell_mult_mean`
  - `adaptive_shell_mult_mean ≈ 2.4596` across the healthy ridge
  - `D32` failed strict review regardless of `shell_growth`
- Decision:
  - treat `shell_growth` as non-discriminative in the current capped regime
  - collapse the live search space to `delta_r`
  - do not change the route lead from this increment alone
- Direction shift:
  - the next branch should target the controller cap law itself
  - this is the first point where a `phi`-structured ratio law becomes more plausible than more shell-growth tuning

## 2026-03-05 (research increment INC-0016)
- Ran 4-seed delta-only confirm:
  - `configs/proxy_transfer_inc0016_delta_confirm.json`
  - `D28_SG18`, `D30_SG18`, `D35_SG18`
- Result:
  - `D30_SG18`: `0.00391514`, `49.334s`, `shells=3.0`, `shell_pmax=0.579`
  - `D28_SG18`: `0.00391578`, `54.294s`, `shells=2.0`, `shell_pmax=0.580`
  - `D35_SG18`: `0.00391829`, `49.549s`, `shells=3.0`, `shell_pmax=0.658`
- Decision:
  - kill `D28` as a lead-replacement candidate
  - keep `D30` as the best current radial law
  - keep `D35` only as a trailing comparison branch
- Systems caution:
  - this batch did not reproduce the large runtime gap vs `R0` from `INC-0014`
  - use within-batch comparisons and route-health first until sweep order / host-load bias is reduced
- Direction shift:
  - the route problem is now focused:
    - `delta_r` matters
    - `shell_growth` does not, under the current cap
    - the next meaningful branch is a new cap/merge law

## 2026-03-05 (research increment INC-0017)
- Implemented a new adaptive shell controller mode:
  - `adaptive_converge_mode=fixed|phi_ratio`
  - `phi_ratio` keeps `pi` in the divergence field and uses a `phi`-scaled ratio pressure for shell convergence
- Ran controller screen:
  - `configs/proxy_transfer_inc0017_phi_ratio_screen.json`
  - compared `D30_FIXED_SG16` vs `D30_PHI_L100`, `D30_PHI_L120`, `D30_PHI_L140`
- Screen result:
  - `D30_PHI_L100`: `0.0039290`, `27.523s`, health pass
  - `D30_PHI_L120`: `0.0039430`, `27.217s`, health pass
  - `D30_PHI_L140`: failed seed-wise shell concentration
- Ran 4-seed larger-subset confirm:
  - `configs/proxy_transfer_inc0017_phi_ratio_confirm.json`
- Confirm result:
  - `R0`: `0.0039079`, `41.442s`, collapsed baseline
  - `D30_FIXED_SG16`: `0.0039185`, `42.805s`, `shells=3.0`, `shell_pmax=0.579`, pass
  - `D30_PHI_L120`: `0.0039144`, `44.637s`, `shells=3.25`, `shell_pmax=0.555`, pass
  - `D30_PHI_L100`: `0.0039323`, `45.349s`, `shells=7.75`, `shell_pmax=0.486`, pass
- Decision:
  - keep `D30_FIXED_SG16` as the transfer hardware-efficiency route lead
  - track `D30_PHI_L120` as the healthiest quality `phi`-controller branch
  - kill `D30_PHI_L140`
  - demote `D30_PHI_L100` to a slower over-dispersed comparison branch
- Direction shift:
  - `phi_ratio` is a real mechanism branch; the controller axis is live again
  - but the first healthy `phi` branch is quality-first rather than hardware-first
  - the next meaningful branch is not more `lambda` tuning on fixed `delta_r`
  - the next meaningful branch is radial retuning under the live `phi` controller

## 2026-03-05 (research increment INC-0018)
- Ran `phi` radial retune screen:
  - `configs/proxy_transfer_inc0018_phi_delta_screen.json`
  - compared `R0`, `D30_FIXED_SG16`, `PHI_D30_L120`, `PHI_D32_L120`, `PHI_D35_L120`
- Screen result:
  - `PHI_D30_L120`: `0.0039430`, `25.750s`, health pass
  - `PHI_D32_L120`: `0.0039378`, `28.177s`, health pass
  - `PHI_D35_L120`: `0.0039305`, `36.180s`, failed runtime gate
- Ran 4-seed larger-subset confirm:
  - `configs/proxy_transfer_inc0018_phi_delta_confirm.json`
- Confirm result:
  - `R0`: `0.0039079`, `45.985s`, collapsed baseline
  - `D30_FIXED_SG16`: `0.0039185`, `47.476s`, `shells=3.0`, `shell_pmax=0.579`, pass
  - `PHI_D30_L120`: `0.0039144`, `50.308s`, `shells=3.25`, `shell_pmax=0.555`, pass
  - `PHI_D32_L120`: `0.0039371`, `40.801s`, `shells=6.0`, `shell_pmax=0.543`, pass
- Decision:
  - promote `PHI_D32_L120` as the routed hardware-efficiency transfer lead
  - retain `PHI_D30_L120` as the routed quality-first `phi` branch
  - demote `D30_FIXED_SG16` to fixed-controller comparator
  - kill `PHI_D35_L120` as a lead-replacement candidate on runtime
- Direction shift:
  - `phi_ratio` is no longer only a quality-first controller branch
  - retuning `delta_r` moved the routed hardware-efficiency optimum from fixed `D30` to `phi` `D32`
  - the next high-value branch is hybrid local zoom, not more blind continuous retuning
  - discrete `phi` step-ladder control is now a conditional stabilization branch, not the primary next move

## 2026-03-05 (research increment INC-0019)
- Implemented `sector_mode=phase4d_complex_local`:
  - coarse `phase4d_adaptive` routing
  - local complex refinement using discrete root-of-unity / imaginary-field rotation
  - composed sector ids `coarse_sector * local_k + local_sector`
- Ran seed-major screen:
  - `configs/proxy_transfer_inc0019_hybrid_screen.json`
- Screen result:
  - `PHI_D32_L120`: `0.0039378`, `29.284s`, health pass
  - `HYB_L4_R4`: `0.0039577`, `31.961s`, failed on unseen-route exposure
  - `HYB_L9_R4`: `0.0039784`, `34.356s`, failed on unseen-route exposure and runtime
- Decision:
  - do not promote the hybrid branch to confirm
  - keep `phase4d_complex_local` as a mechanism candidate only
- Direction shift:
  - local complex zoom is not blocked by lack of capacity; it is blocked by missing local convergence / merge control
  - the next useful hybrid branch is not larger `local_k`; it is local convergence rescue

## 2026-03-05 (control CTRL-0001)
- Ran seed-major larger-subset control:
  - `configs/proxy_transfer_ctrl0001_seedmajor_lead.json`
- Control result:
  - `R0`: `0.0039079`, `44.125s`, collapsed baseline
  - `D30_FIXED_SG16`: `0.0039185`, `44.432s`, health pass
  - `PHI_D32_L120`: `0.0039371`, `43.990s`, health pass
- Decision:
  - retain `PHI_D32_L120` as the routed hardware-efficiency transfer lead
  - narrow the claim: the runtime edge is controlled but small
- Direction shift:
  - the lead survives route-order control
  - future reporting should not describe the current routed lead as a decisive throughput win

## 2026-03-05 (research increment INC-0020 screen)
- Implemented hybrid local-convergence rescue:
  - added local controller parameters for `phase4d_complex_local`
  - changed local activation from absolute-scale overflow to ratio-based pressure
  - added `hybrid_local_min_k` to enforce a stable minimum local split
- Ran seed-major screen:
  - `configs/proxy_transfer_inc0020_hybrid_rescue_screen.json`
- Screen result:
  - `HYB4_M2_T010_C005`: `0.0039365`, `29.562s`, health pass
  - `PHI_D32_L120`: `0.0039378`, `29.962s`, health pass
  - `R0`: `0.0039450`, `29.572s`, collapsed baseline
- Decision:
  - promote `HYB4_M2_T010_C005` to 4-seed larger-subset confirm
  - keep `HYB4_M2_T005_C005` as the slightly more open comparator
- Direction shift:
  - the hybrid branch is no longer blocked by unseen-route explosion alone
  - the relevant next question becomes quality-vs-runtime, not viability-vs-collapse

## 2026-03-05 (research increment INC-0020 confirm)
- Ran seed-major 4-seed larger-subset confirm:
  - `configs/proxy_transfer_inc0020_hybrid_rescue_confirm.json`
- Confirm result:
  - `R0`: `0.0039079`, `43.329s`, collapsed baseline
  - `HYB4_M2_T010_C005`: `0.0039203`, `46.116s`, health pass
  - `HYB4_M2_T005_C005`: `0.0039231`, `46.062s`, health pass
  - `PHI_D32_L120`: `0.0039371`, `45.038s`, health pass
- Decision:
  - promote `HYB4_M2_T010_C005` as the healthiest routed-quality branch
  - retain `PHI_D32_L120` as the fastest healthy routed branch
  - do not claim a routed hardware-efficiency win from this confirm because no healthy route beats `R0` on runtime
- Direction shift:
  - hybrid local zoom is now a real quality branch rather than a blocked mechanism branch
  - the next high-value runtime branch is again the controller law, not more hybrid capacity

## 2026-03-05 (research increment INC-0021 screen)
- Implemented a discrete shell controller:
  - `adaptive_converge_mode=phi_ladder`
  - shell overflow is quantized in additive `log(phi)` steps before convergence is applied
- Ran seed-major larger-subset screen:
  - `configs/proxy_transfer_inc0021_phi_ladder_screen.json`
- Screen result:
  - `R0`: `0.0039113`, `42.542s`, collapsed baseline
  - `PHI_D32_L120`: `0.0039543`, `68.881s`, runtime gate fail
  - `LADDER_D32_L045`: `0.0039383`, `50.493s`, runtime gate fail
  - `LADDER_D32_L055`: `0.0039335`, `50.592s`, runtime gate fail
  - `LADDER_D32_L065`: `0.0039419`, `47.407s`, health pass
- Decision:
  - close `INC-0021` at the screen stage
  - do not spend a 4-seed confirm on `LADDER_D32_L065` yet because no healthy route beat `R0` on runtime
  - track `LADDER_D32_L065` as the healthiest routed controller candidate
  - put `PHI_D32_L120` under review rather than treating it as the current operational lead
- Direction shift:
  - `phi` is stronger as a discrete controller constant than as a continuous post-threshold slope
  - `log(phi)` is the right additive unit for shell hysteresis / split-merge ladders
  - the remaining mismatch is likely shell metric, not controller family
  - next branch: keep `pi` in the angular manifold, keep hyperbolic time expansion in the radial field, and replace linear shell indexing with `phi`-spaced log shells

## 2026-03-05 (research increment INC-0022)
- Implemented `shell_mode=phi_log` so shell indexing now follows the same multiplicative family as the discrete `phi` controller.
- Screen:
  - `PHILOG_D32_L065` looked strongest on the narrow slice, but failed screen on shell concentration.
  - `PHILOG_D36_L065` beat the linear-ladder comparator on mean runtime and mean quality while staying healthy.
- 4-seed larger-subset confirm:
  - `R0`: `0.003907888`, `47.751s`, collapsed baseline
  - `LINEAR_D32_L065`: `0.003924776`, `52.032s`, healthy comparator
  - `PHILOG_D36_L065`: `0.003901309`, `50.566s`, healthy
- Decision:
  - promote the branch narratively as `PHI_PHI_PHI v1` (artifact: `PHILOG_D36_L065`)
  - keep `R0` as the transfer control baseline and absolute runtime baseline
  - keep `PHI_PHI_PHI v1` as the current transfer quality lead
  - demote `LINEAR_D32_L065` to shell-metric comparator status
  - kill `PHILOG_D32_L065` as a promotion candidate because concentration remained too high
- Direction shift:
  - the shell metric is no longer the main unknown; it works
  - the next live research problem is budget compression inside the `PHI_PHI_PHI` family so the quality/health gain can become a hardware-efficiency gain

## 2026-03-05 (research increment INC-0023)
- Tested simple angular budget compression inside the `PHI_PHI_PHI` family.
- Screen (`configs/proxy_transfer_inc0023_phi3_budget_screen.json`):
  - `PHI3_K20_D36_L065` beat `R0` and `PHI3_K25_D36_L065` on the 2-seed screen while staying healthy.
  - `PHI3_K16_D36_L065` failed on seed-wise shell concentration.
  - `PHI3_K16_B2_D36_L065` stayed healthy but was slower and weaker.
- 4-seed confirm (`configs/proxy_transfer_inc0023_phi3_budget_confirm.json`):
  - `R0`: `0.003907888`, `53.154s`
  - `PHI3_K25_D36_L065`: `0.003901309`, `48.472s`
  - `PHI3_K20_D36_L065`: `0.003893818`, `50.990s`
- Decision:
  - do not promote `K20` over `K25`
  - keep `PHI_PHI_PHI v1` / `PHI3_K25_D36_L065` as the stabilized proxy-transfer candidate
  - treat `K20` as a screen-only compression candidate until it reproduces under stronger control
- Direction shift:
  - the family appears to need a minimum coarse angular budget
  - the next immediate task is fairness control on the runtime claim
  - after that, phase-coupled shells are the next geometry branch if the current family still looks incomplete

## 2026-03-05 (control CTRL-0002)
- Ran a stricter fairness audit on the coarse `PHI_PHI_PHI` family:
  - `configs/proxy_transfer_ctrl0002_phi3_vs_r0_seedmajor.json`
  - intentionally ordered `PHI3_K25_D36_L065` before `R0` in a seed-major batch
- Control result:
  - `R0`: `0.003907888`, `44.916s`, collapsed baseline
  - `PHI3_K25_D36_L065`: `0.003901309`, `52.077s`, healthy on route structure
  - failure reason: `runtime_ratio_vs_r0=1.159 > 1.150`
- Decision:
  - keep `PHI_PHI_PHI v1` as the transfer quality/health lead
  - remove the runtime-win claim from the current family
  - keep `R0` as the operational runtime preference until a new branch clears fairness control
- Direction shift:
  - the next live geometry question is phase-coupled / phase-shifted shells
  - do not spend another fairness control batch until a new branch plausibly improves the runtime story

## 2026-03-05 (research increment INC-0024)
- Implemented `shell_mode=phi_phase` with signed shell-boundary shifts from phase pressure.
- Screen (`configs/proxy_transfer_inc0024_phase_shell_screen.json`):
  - `PHASE_K25_C035`: `0.003912563`, `54.184s`, health pass
  - `PHASE_K25_C020`: `0.003916593`, `53.645s`, health pass
  - `PHI3_K25_D36_L065`: `0.003921162`, `68.920s`, runtime gate fail
  - `R0`: `0.003911258`, `52.145s`, collapsed baseline
- Confirm (`configs/proxy_transfer_inc0024_phase_shell_confirm.json`):
  - `R0`: `0.003907888`, `46.405s`, collapsed baseline
  - `PHASE_K25_C035`: `0.003916993`, `53.423s`, failed only on `runtime_ratio_vs_r0=1.151`
  - `PHI3_K25_D36_L065`: `0.003917867`, `57.203s`, failed by a wider runtime margin
- Decision:
  - promote `PHASE_K25_C035` over the coarse `PHI_PHI_PHI` family reference inside the routed family
  - keep `R0` as the operational transfer baseline
  - do not claim a routed runtime win yet
- Direction shift:
  - shell phase matters
  - the next geometry branch should make the shell-phase law sparser or more discrete rather than more continuous

## 2026-03-05 (deep math review)
- Completed:
  - `docs/research/MATH_REVIEW_H4_GEOMETRY_20260305.md`
- Main conclusion:
  - the current route likely still misses the true `H^4` shell-sector scaling law
  - current paired-phase routing is probably seeing a real `S^3` / Hopf structure, but using only a heuristic substitute for its angular measure
  - continuous shell-phase coupling is real, but likely only a local correction on top of the wrong global scaling law
- Decision:
  - promote `INC-0026` (`H4`-Hopf geodesic pilot) to the primary next branch
  - demote `INC-0025` sparse / quantized shell-phase laws to fallback status
  - postpone pure cost decomposition until after the deeper geometry branch is tested

## 2026-03-05 (research increment INC-0026)
- Implemented Slice A diagnostics for the adaptive 4D route:
  - `chi`
  - `chi_entropy`
  - `r_alpha`
  - capped Hopf shell-capacity estimate
  - Hopf/current pair-bin gap metrics
- Diagnostic sweep (`configs/proxy_transfer_inc0026_hopf_diag.json`):
  - `PHASE_K25_C035`: `chi_mean=0.3271`, `hopf_shell_capacity~=9.0005`, current `k1,k2~=4.3,4.3`
  - `PHI3_K25_D36_L065`: `chi_mean=0.3325`, `hopf_shell_capacity~=9.0008`, current `k1,k2~=4.4,4.3`
  - interpretation: the current routed family is stably over-allocating angular capacity relative to the capped `H^4` shell-capacity law
- Implemented `sector_mode=phase4d_hopf`.
- Pilot screen (`configs/proxy_transfer_inc0026_hopf_pilot_screen.json`):
  - `HOPF_K25_BASE`: `0.003888756`, `75.630s`, `sectors=4.0`, `shell_pmax=0.652`, runtime gate fail
  - `PHASE_K25_C035`: `0.003909488`, `72.920s`, runtime gate fail
  - `PHI3_K25_D36_L065`: `0.003917124`, `64.673s`, runtime gate fail
  - `R0`: `0.003911258`, `52.468s`, collapsed baseline
- Decision:
  - keep the `H4` branch alive because pure Hopf shell-capacity coupling improved quality
  - do not promote pure `phase4d_hopf`; it compresses too hard and gets slower
  - re-rank the next branch to explicit `chi` representation or blended shell capacity
  - add phi/Fibonacci lattice routing as the geometric fallback if the `chi` branch still fails

## 2026-03-05 (research increment INC-0028)
- Implemented `phase4d_hopf_chi` with measure-aware `chi` binning via `u_chi = sin^2(chi)`.
- First screen attempt was invalidated by a proxy-evaluator bug:
  - `tasks/router_proxy_eval.py` passed `hopf_chi_bins` into `phase4d_adaptive_components()`
  - routed branches crashed before emitting `__JSON_SUMMARY__`
- Fixed the evaluator bug and reran the screen:
  - `HOPF_K25_BASE`: `0.003888756`, `57.498s`, `sectors=4.0`
  - `HOPF_CHI3_K25`: `0.003902545`, `59.118s`, `sectors=11.0`
  - `HOPF_CHI2_K25`: `0.003929591`, `63.581s`, `sectors=8.0`
  - `PHASE_K25_C035`: `0.003909488`, `56.723s`
  - `PHI3_K25_D36_L065`: `0.003917124`, `56.528s`
- Decision:
  - explicit `chi` reopened angular capacity but did not beat pure Hopf
  - keep `HOPF_K25_BASE` alive and promote it to 4-seed confirm
  - kill the first standalone `chi`-axis branch as the immediate lead path
  - promote the phi/Fibonacci lattice branch to next-live status if pure Hopf confirm still misses runtime

## 2026-03-06 (research increment INC-0030)
- Ran 4-seed larger-subset pure Hopf confirm:
  - `configs/proxy_transfer_inc0030_hopf_confirm.json`
- Confirm means:
  - `HOPF_K25_BASE`: `0.003896580`, `63.244s`, `sectors=4.0`, health pass
  - `PHASE_K25_C035`: `0.003904390`, `60.888s`, health pass
  - `PHI3_K25_D36_L065`: `0.003916927`, `63.745s`, health pass
  - `R0`: `0.003907888`, `57.833s`, shell-collapse health fail
- Decision:
  - promote `HOPF_K25_BASE` to routed-quality lead status
  - keep `PHASE_K25_C035` as the widened routed-family comparator
  - do not promote any routed runtime lead from this confirm
  - make `INC-0029` (phi/Fibonacci lattice) the default next geometry branch

## 2026-03-06 (research increment INC-0029)
- Implemented first `phase4d_hopf_fib` lattice branch.
- Screen result:
  - `HOPF_FIB_K25`: matched Hopf quality exactly (`0.003888756`) but became much slower (`104.563s`)
  - `adaptive_chi_bins_used=1.0`
  - effective sectors remained `4.0`
- Reading:
  - the branch did not falsify the phi/Fibonacci direction
  - it falsified the first allocator law
  - under `K=25` and `min_pair_bins=3`, the greedy Fibonacci fit collapsed back to the same effective Hopf pair
- Decision:
  - kill the first greedy Fibonacci allocator
  - keep the phi/Fibonacci program alive
  - promote `INC-0031` recurrence-constrained rung forcing as the next branch

## 2026-03-06 (research increment INC-0031)
- Implemented `phase4d_hopf_fib_rung` as a recurrence-constrained `phi^2` widening branch.
- Screen result:
  - `HOPF_K25_BASE`: `0.003888756`, `81.824s`, `sectors=4.0`
  - `HOPF_FIB_K25`: `0.003888756`, `120.002s`, `sectors=4.0`
  - `HOPF_PHI2_K25`: `0.003902407`, `115.481s`, `sectors=10.5`, `buckets=20.0`, `chi_bins=2.0`
  - `PHASE_K25_C035`: `0.003909488`, `71.515s`, `sectors=11.5`
  - `R0`: `0.003911258`, `59.113s`, shell-collapse health fail
- Reading:
  - recurrence-constrained rung forcing is a real geometry branch; it widened Hopf cleanly and reproducibly across both seeds
  - the first successful `phi^2` law is too global and too expensive in its current form
  - `chi` occupancy remains too concentrated (`chi_bin_pmax ~= 0.94`), so the extra lattice capacity is not being used efficiently yet
- Decision:
  - keep `HOPF_PHI2_K25` as a geometry candidate, not an operational lead
  - keep `HOPF_K25_BASE` as the routed-quality lead
  - promote sparse / gated `phi^2` widening as the next branch
  - do not spend a confirm on global ungated rung forcing

## 2026-03-06 (research increment INC-0032)
- Ran threshold-gated `phi^2` widening screen:
  - `configs/proxy_transfer_inc0032_phi2_gated_screen.json`
- Result:
  - `HOPF_K25_BASE`: `0.003888756`, `61.261s`
  - `HOPF_PHI2_K25`: `0.003902407`, `104.365s`
  - `HOPF_PHI2_G062`: `0.003905332`, `96.498s`
  - `HOPF_PHI2_G085`: `0.003894006`, `97.079s`
  - `PHASE_K25_C035`: `0.003909488`, `58.525s`
  - `R0`: `0.003911258`, `46.961s`
- Reading:
  - threshold gating reduced `phi^2` cost modestly but not enough
  - both gated variants remained about `2x` slower than `R0`
  - neither gated variant beat pure Hopf on quality
  - `chi` concentration remained severe; the strict gate made it worse
- Decision:
  - kill per-point threshold gating as the main rescue for the `phi^2` lattice
  - keep `HOPF_K25_BASE` as routed-quality lead
  - keep `PHASE_K25_C035` as widened routed-family comparator
  - keep the `phi^2` family alive only as geometry evidence
  - promote a banded shared-state lattice as the next branch

## 2026-03-06 (research increment INC-0033)
- Ran banded shared-state `phi^2` lattice screen:
  - `configs/proxy_transfer_inc0033_phi2_band_screen.json`
- Result:
  - `HOPF_K25_BASE`: `0.003888756`, `62.389s`, `sectors=4.0`, `shells=3.0`
  - `HOPF_PHI2_BAND`: `0.003897103`, `65.725s`, `sectors=10.5`, `shells=2.0`
  - `HOPF_PHI2_K25`: `0.003902407`, `116.411s`, `sectors=10.5`, `shells=2.5`
  - `PHASE_K25_C035`: `0.003909488`, `69.983s`, `sectors=11.5`, `shells=3.0`
  - `R0`: `0.003911258`, `52.127s`, shell-collapse health fail
- Reading:
  - banded shared states preserved the widened Hopf signal
  - runtime improved dramatically relative to ungated `phi^2`
  - the branch still failed the operational screen because runtime remained above the configured bar vs `R0`
  - `chi` concentration remained severe
- Decision:
  - keep `HOPF_K25_BASE` as routed-quality lead
  - promote `HOPF_PHI2_BAND` over `HOPF_PHI2_K25` as the widened Hopf geometry candidate
  - kill banded `phi^2` as an operational rescue at the screen stage
  - promote blended Hopf-capacity control as the next live branch

## 2026-03-06 (research increment INC-0048)
- Implemented the first translated retrieval harness:
  - `tasks/router_retrieval_eval.py`
  - `tools/proxy_sweep.py` generalized to task-level sweeps
  - `tools/summarize.py` extended with retrieval fields
- Ran translation screen:
  - `configs/proxy_transfer_inc0048_retrieval_translation_screen.json`
  - analysis: `results/analysis/inc0048_retrieval_translation_screen.json`
  - gate note: `docs/governance/gates/gate_20260306_111959.md`
- Reading:
  - translated routed retrieval preserved candidate-pruning signal
  - dense exact retrieval remained operationally dominant on single-batch total wall-clock
  - `probe_buckets=1` was the right systems rescue branch
- Decision:
  - keep geometry fixed
  - promote retrieval cost rescue as the next branch

## 2026-03-06 (research increment INC-0049)
- Implemented grouped same-bucket routed retrieval for `probe_buckets=1`.
- Added explicit offline/online timing decomposition to translated retrieval runs.
- Ran retrieval cost-rescue screen:
  - `configs/proxy_transfer_inc0049_retrieval_cost_rescue_screen.json`
  - analysis: `results/analysis/inc0049_retrieval_cost_rescue_screen.json`
  - gate note: `docs/governance/gates/gate_20260306_113201.md`
- Mean result:
  - `DENSE`: `mse=0.004318726`, `total=1.332s`, `offline=0.000s`, `online=0.879s`, `cand_frac=1.0`
  - `HOPF_RET_P1`: `0.004325216`, `10.687s`, `offline=9.694s`, `online=0.401s`, `cand_frac=0.3488`
  - `HOPF_PHI2_RET_P1`: `0.004326332`, `8.525s`, `offline=7.664s`, `online=0.299s`, `cand_frac=0.3415`
- Reading:
  - vectorized same-bucket retrieval materially narrowed the routed cost
  - the routed branch now wins on online/query-time cost
  - single-batch total still loses because offline chart/index build dominates
- Decision:
  - keep translated retrieval alive
  - do not promote on single-batch total wall-clock
  - move next to repeated-query amortization analysis

## 2026-03-06 (research increment INC-0051)
- Added repeated-query amortization metrics to the translated retrieval harness:
  - `query_repeats`
  - `retrieval_online_total_per_repeat_sec`
  - `retrieval_total_amortized_per_repeat_sec`
- Ran amortization screen:
  - `configs/proxy_transfer_inc0051_retrieval_amortization_screen.json`
  - analysis: `results/analysis/inc0051_retrieval_amortization_screen.json`
  - gate note: `docs/governance/gates/gate_20260306_114654.md`
- Reading:
  - `HOPF_RET_P1_Q24` is the first routed translated branch to beat matched dense on amortized per-repeat cost
  - `HOPF_PHI2_RET_P1` did not cash in its stronger pruning on the translated task
  - single-batch total still favors dense because offline chart/index build dominates
- Decision:
  - keep plain Hopf as the live translated retrieval branch
  - demote widened Hopf on the translated retrieval task
  - move next to a narrow 4-seed crossover confirm at `Q24/Q32`

## 2026-03-06 (research increment INC-0052)
- Ran amortization confirm:
  - `configs/proxy_transfer_inc0052_retrieval_amortization_confirm.json`
  - analysis: `results/analysis/inc0052_retrieval_amortization_confirm.json`
  - gate note: `docs/governance/gates/gate_20260306_115931.md`
- Mean result:
  - `DENSE_Q24`: `mse=0.004321788`, `amortized_per_repeat=0.5051s`
  - `HOPF_RET_P1_Q24`: `mse=0.004324992`, `amortized_per_repeat=0.5938s`
  - `DENSE_Q32`: `mse=0.004321788`, `amortized_per_repeat=0.5586s`
  - `HOPF_RET_P1_Q32`: `mse=0.004324992`, `amortized_per_repeat=0.6544s`
- Reading:
  - the narrow screen-stage crossover was not stable
  - translated routed retrieval remains a positive transfer signal, but not a promoted operational path
- Decision:
  - kill translated retrieval promotion for now
  - reopen the next deep geometry branch
  - keep the translated retrieval harness as an evaluation target for future geometry families
