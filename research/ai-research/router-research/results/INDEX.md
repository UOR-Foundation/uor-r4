# Results Index

Use this file to map each run batch to parsed summaries and decision records.

## Batch Template
- Batch ID:
- Config:
- Logs:
- Parsed files:
- Summary row range:
- Decision note:

## Batch B001
- Batch ID: `B001_smoke_single`
- Config: ad hoc fast-dev smoke command (`run_tag=smoke_single`)
- Logs: `results/raw/smoke_single.log`
- Parsed files: `results/parsed/smoke_single.json`
- Summary row range: `results/summary.csv` row with `log_file=smoke_single.log`
- Decision note: n/a

## Batch B002
- Batch ID: `B002_sweep_20260305_123420`
- Config: `configs/route_sweep.yaml`
- Logs: `results/raw/sweep_*.log`
- Parsed files: `results/parsed/sweep_*.json`
- Summary row range: rows with `log_file` prefix `sweep_` in `results/summary.csv`
- Decision note: `docs/governance/gates/gate_20260305_123420.md`

## Batch B003
- Batch ID: `B003_wikitext2_proxy_baseline`
- Config: `tools/prepare_wikitext2.py` + `tasks/wikitext2_proxy.py` + `tasks/dense_baseline.py`
- Logs: `results/raw/dense_baseline_wikitext2_proxy.json`
- Parsed files: n/a (non-router artifact)
- Summary row range: n/a
- Decision note: captured in `docs/reports/REAL_TASK_COMPARISON.md`

## Batch B004
- Batch ID: `B004_phase4d_validation_20260305_125915`
- Config: `configs/route_sweep_phase4d_validation.json`
- Logs: `results/raw/sweep_R0_*_20260305_125*.log`, `results/raw/sweep_R5_*_20260305_125*.log`
- Parsed files: matching `results/parsed/*.json`
- Summary row range: `results/summary.csv` where `run_tag` contains `_20260305_125`
- Decision note: `docs/governance/gates/gate_20260305_125915.md`

## Batch B005
- Batch ID: `B005_lm_proxy_ptb_baseline`
- Config: `tools/prepare_wikitext2.py --dataset auto` (resolved to PTB) + proxy + dense baseline
- Logs: `results/raw/dense_baseline_lm_proxy.json`
- Parsed files: n/a (non-router artifact)
- Summary row range: n/a
- Decision note: reflected in `docs/reports/REAL_TASK_COMPARISON.md`

## Batch B006
- Batch ID: `B006_phase4d_validation_non_fast`
- Config: `configs/route_sweep_phase4d_validation.json`
- Logs: `results/raw/sweep_R0_*_20260305_125*.log`, `results/raw/sweep_R5_*_20260305_125*.log`
- Parsed files: matching `results/parsed/*.json`
- Summary row range: rows where `run_tag` contains `_20260305_125`
- Decision note: `docs/governance/gates/gate_20260305_125915.md`

## Batch B007
- Batch ID: `B007_phase4d_dims_fast`
- Config: `configs/route_sweep_phase4d_dims_fast.json`
- Logs: `results/raw/sweep_R0_*_20260305_130*.log`, `results/raw/sweep_R5A_*_20260305_130*.log`, `results/raw/sweep_R5B_*_20260305_130*.log`, `results/raw/sweep_R5C_*_20260305_130*.log`
- Parsed files: matching `results/parsed/*.json`
- Summary row range: rows where `run_tag` contains `_20260305_130`
- Decision note: `docs/governance/gates/gate_20260305_130139.md`

## Batch B008
- Batch ID: `B008_inc0003_r0_vs_r5b_non_fast`
- Config: `configs/route_sweep_inc0003_r0_vs_r5b.json`
- Logs: `results/raw/sweep_R0_*_20260305_1303-1306.log`, `results/raw/sweep_R5B_*_20260305_1306-1308.log`
- Parsed files: matching `results/parsed/*.json`
- Summary row range: rows where `run_tag` contains `_20260305_1303`..`_20260305_1308` for routes `R0` and `R5B`
- Decision note: `docs/governance/gates/gate_20260305_130833.md`

## Batch B009
- Batch ID: `B009_inc0004_r0_vs_r5b_finalize4`
- Config: `configs/route_sweep_inc0004_r0_vs_r5b_finalize4.json`
- Logs: `results/raw/sweep_R0_*_20260305_131*.log`, `results/raw/sweep_R5B_*_20260305_131*.log`
- Parsed files: matching `results/parsed/*.json`
- Summary row range: rows where `run_tag` contains `_20260305_131` for routes `R0` and `R5B`
- Decision note: `docs/governance/gates/gate_20260305_131933.md`

## Batch B010
- Batch ID: `B010_inc0005_r5b_timepressure`
- Config: `configs/route_sweep_inc0005_r5b_timepressure.json`
- Logs: `results/raw/sweep_R5B_L*_finalize_seed*_{20260305_1324..1330}.log`
- Parsed files: matching `results/parsed/*.json`
- Summary row range: rows where `run_tag` contains `R5B_L` and `_20260305_13`
- Decision note: `docs/governance/gates/gate_20260305_133115.md`

## Batch B011
- Batch ID: `B011_inc0006_r5b_robustness_large_n`
- Config: `configs/route_sweep_inc0006_r5b_robustness.json`
- Logs: `results/raw/sweep_R0_*_20260305_1348..1355.log`, `results/raw/sweep_R5B_*_20260305_1356..1401.log`
- Parsed files: matching `results/parsed/*.json`
- Summary row range: rows where `run_tag` contains `_20260305_134`..`_20260305_140`
- Decision note: `docs/governance/gates/gate_20260305_140216.md`

## Batch B012
- Batch ID: `B012_inc0007_lm_proxy_smoke`
- Config: `tasks/router_proxy_eval.py` with `run_tag=proxy_cmp_r0` and `run_tag=proxy_cmp_r5b`
- Logs: `results/raw/proxy_cmp_r0.log`, `results/raw/proxy_cmp_r5b.log`
- Parsed files: `results/parsed/proxy_cmp_r0.json`, `results/parsed/proxy_cmp_r5b.json`
- Summary row range: rows with `run_tag in {proxy_cmp_r0, proxy_cmp_r5b}`
- Decision note: reflected in `docs/reports/REAL_TASK_COMPARISON.md`

## Batch B013
- Batch ID: `B013_inc0008_proxy_transfer_multiseed`
- Config: `configs/proxy_transfer_inc0008.json`
- Logs: `results/raw/inc0008_proxy_transfer_multiseed_R0_seed*.log`, `results/raw/inc0008_proxy_transfer_multiseed_R5B_seed*.log`
- Parsed files: matching `results/parsed/inc0008_proxy_transfer_multiseed_*.json`
- Analysis: `results/analysis/inc0008_proxy_transfer_multiseed.json`
- Summary row range: rows where `run_tag` contains `inc0008_proxy_transfer_multiseed`
- Decision note: `docs/governance/gates/gate_20260305_141648.md`

## Batch B014
- Batch ID: `B014_inc0009_proxy_stabilization_screen`
- Config: `configs/proxy_transfer_inc0009_screen.json`
- Logs: `results/raw/inc0009_proxy_stabilization_screen_*.log`
- Parsed files: matching `results/parsed/inc0009_proxy_stabilization_screen_*.json`
- Analysis: `results/analysis/inc0009_proxy_stabilization_screen.json`
- Summary row range: rows where `run_tag` contains `inc0009_proxy_stabilization_screen`
- Decision note: `docs/governance/gates/gate_20260305_144909.md`

## Batch B015
- Batch ID: `B015_inc0009_proxy_stabilization_confirm`
- Config: `configs/proxy_transfer_inc0009_confirm.json`
- Logs: `results/raw/inc0009_proxy_stabilization_confirm_*.log`
- Parsed files: matching `results/parsed/inc0009_proxy_stabilization_confirm_*.json`
- Analysis: `results/analysis/inc0009_proxy_stabilization_confirm.json`
- Summary row range: rows where `run_tag` contains `inc0009_proxy_stabilization_confirm`
- Decision note: `docs/governance/gates/gate_20260305_145544.md`

## Batch B016
- Batch ID: `B016_inc0010_adaptive_phase4d_confirm`
- Config: `configs/proxy_transfer_inc0010_adaptive_confirm.json`
- Logs: `results/raw/inc0010_adaptive_phase4d_confirm_*.log`
- Parsed files: matching `results/parsed/inc0010_adaptive_phase4d_confirm_*.json`
- Analysis: `results/analysis/inc0010_adaptive_phase4d_confirm.json`
- Summary row range: rows where `run_tag` contains `inc0010_adaptive_phase4d_confirm`
- Decision note: `docs/governance/gates/gate_20260305_151230.md`

## Batch B017
- Batch ID: `B017_inc0011_shell_activation_screen`
- Config: `configs/proxy_transfer_inc0011_shell_screen.json`
- Logs: `results/raw/inc0011_shell_activation_screen_*.log`
- Parsed files: matching `results/parsed/inc0011_shell_activation_screen_*.json`
- Analysis: `results/analysis/inc0011_shell_activation_screen.json`
- Summary row range: rows where `run_tag` contains `inc0011_shell_activation_screen`
- Decision note: `docs/governance/gates/gate_20260305_153422.md`

## Batch B018
- Batch ID: `B018_inc0011_shell_activation_confirm`
- Config: `configs/proxy_transfer_inc0011_shell_confirm.json`
- Logs: `results/raw/inc0011_shell_activation_confirm_*.log`
- Parsed files: matching `results/parsed/inc0011_shell_activation_confirm_*.json`
- Analysis: `results/analysis/inc0011_shell_activation_confirm.json`
- Summary row range: rows where `run_tag` contains `inc0011_shell_activation_confirm`
- Decision note: `docs/governance/gates/gate_20260305_153937.md`

## Batch B019
- Batch ID: `B019_inc0012_convergence_screen`
- Config: `configs/proxy_transfer_inc0012_convergence_screen.json`
- Logs: `results/raw/inc0012_convergence_screen_*.log`
- Parsed files: matching `results/parsed/inc0012_convergence_screen_*.json`
- Analysis: `results/analysis/inc0012_convergence_screen.json`
- Summary row range: rows where `run_tag` contains `inc0012_convergence_screen`
- Decision note: `docs/governance/gates/gate_20260305_160251.md`

## Batch B020
- Batch ID: `B020_inc0012_convergence_confirm`
- Config: `configs/proxy_transfer_inc0012_convergence_confirm.json`
- Logs: `results/raw/inc0012_convergence_confirm_*.log`
- Parsed files: matching `results/parsed/inc0012_convergence_confirm_*.json`
- Analysis: `results/analysis/inc0012_convergence_confirm.json`
- Summary row range: rows where `run_tag` contains `inc0012_convergence_confirm`
- Decision note: `docs/governance/gates/gate_20260305_160951.md`

## Batch B021
- Batch ID: `B021_inc0013_phase_diagram_screen`
- Config: `configs/proxy_transfer_inc0013_phase_diagram_screen.json`
- Logs: `results/raw/inc0013_phase_diagram_screen_*.log`
- Parsed files: matching `results/parsed/inc0013_phase_diagram_screen_*.json`
- Analysis: `results/analysis/inc0013_phase_diagram_screen.json`
- Summary row range: rows where `run_tag` contains `inc0013_phase_diagram_screen`
- Decision note: `docs/governance/gates/gate_20260305_163649.md`

## Batch B022
- Batch ID: `B022_inc0013_phase_diagram_confirm`
- Config: `configs/proxy_transfer_inc0013_phase_diagram_confirm.json`
- Logs: `results/raw/inc0013_phase_diagram_confirm_*.log`
- Parsed files: matching `results/parsed/inc0013_phase_diagram_confirm_*.json`
- Analysis: `results/analysis/inc0013_phase_diagram_confirm.json`
- Summary row range: rows where `run_tag` contains `inc0013_phase_diagram_confirm`
- Decision note: `docs/governance/gates/gate_20260305_164628.md`

## Batch B023
- Batch ID: `B023_inc0013_phase_diagram_confirm_strict_review`
- Config: strict seed-wise review of `results/analysis/inc0013_phase_diagram_confirm.json`
- Logs: reused `B022` logs
- Parsed files: reused `B022` parsed JSON
- Analysis: `results/analysis/inc0013_phase_diagram_confirm_strict.json`
- Summary row range: reused `B022` rows
- Decision note: `docs/governance/gates/gate_20260305_164628_strict.md`

## Batch B024
- Batch ID: `B024_inc0014_strict_robustness`
- Config: `configs/proxy_transfer_inc0014_strict_robustness.json`
- Logs: `results/raw/inc0014_strict_robustness_*.log`
- Parsed files: matching `results/parsed/inc0014_strict_robustness_*.json`
- Analysis: `results/analysis/inc0014_strict_robustness.json`
- Summary row range: rows where `run_tag` contains `inc0014_strict_robustness`
- Decision note: `docs/governance/gates/gate_20260305_171526.md`

## Batch B025
- Batch ID: `B025_inc0015_ridge_discrimination`
- Config: `configs/proxy_transfer_inc0015_ridge_discrimination.json`
- Logs: `results/raw/inc0015_ridge_discrimination_*.log`
- Parsed files: matching `results/parsed/inc0015_ridge_discrimination_*.json`
- Analysis: `results/analysis/inc0015_ridge_discrimination.json`
- Summary row range: rows where `run_tag` contains `inc0015_ridge_discrimination`
- Decision note: `docs/governance/gates/gate_20260305_174043.md`

## Batch B026
- Batch ID: `B026_inc0016_delta_confirm`
- Config: `configs/proxy_transfer_inc0016_delta_confirm.json`
- Logs: `results/raw/inc0016_delta_confirm_*.log`
- Parsed files: matching `results/parsed/inc0016_delta_confirm_*.json`
- Analysis: `results/analysis/inc0016_delta_confirm.json`
- Summary row range: rows where `run_tag` contains `inc0016_delta_confirm`
- Decision note: `docs/governance/gates/gate_20260305_175537.md`

## Batch B027
- Batch ID: `B027_inc0017_phi_ratio_screen`
- Config: `configs/proxy_transfer_inc0017_phi_ratio_screen.json`
- Logs: `results/raw/inc0017_phi_ratio_screen_*.log`
- Parsed files: matching `results/parsed/inc0017_phi_ratio_screen_*.json`
- Analysis: `results/analysis/inc0017_phi_ratio_screen.json`
- Summary row range: rows where `run_tag` contains `inc0017_phi_ratio_screen`
- Decision note: `docs/governance/gates/gate_20260305_181815.md`

## Batch B028
- Batch ID: `B028_inc0017_phi_ratio_confirm`
- Config: `configs/proxy_transfer_inc0017_phi_ratio_confirm.json`
- Logs: `results/raw/inc0017_phi_ratio_confirm_*.log`
- Parsed files: matching `results/parsed/inc0017_phi_ratio_confirm_*.json`
- Analysis: `results/analysis/inc0017_phi_ratio_confirm.json`
- Summary row range: rows where `run_tag` contains `inc0017_phi_ratio_confirm`
- Decision note: `docs/governance/gates/gate_20260305_183037.md`

## Batch B029
- Batch ID: `B029_inc0018_phi_delta_screen`
- Config: `configs/proxy_transfer_inc0018_phi_delta_screen.json`
- Logs: `results/raw/inc0018_phi_delta_screen_*.log`
- Parsed files: matching `results/parsed/inc0018_phi_delta_screen_*.json`
- Analysis: `results/analysis/inc0018_phi_delta_screen.json`
- Summary row range: rows where `run_tag` contains `inc0018_phi_delta_screen`
- Decision note: `docs/governance/gates/gate_20260305_184239.md`

## Batch B030
- Batch ID: `B030_inc0021_phi_ladder_screen`
- Config: `configs/proxy_transfer_inc0021_phi_ladder_screen.json`
- Logs: `results/raw/inc0021_phi_ladder_screen_*.log`
- Parsed files: matching `results/parsed/inc0021_phi_ladder_screen_*.json`
- Analysis: `results/analysis/inc0021_phi_ladder_screen.json`
- Summary row range: rows where `run_tag` contains `inc0021_phi_ladder_screen`
- Decision note: `docs/governance/gates/gate_20260305_202833.md`

## Batch B030
- Batch ID: `B030_inc0018_phi_delta_confirm`
- Config: `configs/proxy_transfer_inc0018_phi_delta_confirm.json`
- Logs: `results/raw/inc0018_phi_delta_confirm_*.log`
- Parsed files: matching `results/parsed/inc0018_phi_delta_confirm_*.json`
- Analysis: `results/analysis/inc0018_phi_delta_confirm.json`
- Summary row range: rows where `run_tag` contains `inc0018_phi_delta_confirm`
- Decision note: `docs/governance/gates/gate_20260305_185546.md`

## Batch B031
- Batch ID: `B031_inc0019_hybrid_screen`
- Config: `configs/proxy_transfer_inc0019_hybrid_screen.json`
- Logs: `results/raw/inc0019_hybrid_screen_*.log`
- Parsed files: matching `results/parsed/inc0019_hybrid_screen_*.json`
- Analysis: `results/analysis/inc0019_hybrid_screen.json`
- Summary row range: rows where `run_tag` contains `inc0019_hybrid_screen`
- Decision note: `docs/governance/gates/gate_20260305_191832.md`

## Batch B032
- Batch ID: `B032_ctrl0001_seedmajor_lead`
- Config: `configs/proxy_transfer_ctrl0001_seedmajor_lead.json`
- Logs: `results/raw/ctrl0001_seedmajor_lead_*.log`
- Parsed files: matching `results/parsed/ctrl0001_seedmajor_lead_*.json`
- Analysis: `results/analysis/ctrl0001_seedmajor_lead.json`
- Summary row range: rows where `run_tag` contains `ctrl0001_seedmajor_lead`
- Decision note: `docs/governance/gates/gate_20260305_192810.md`

## Batch B033
- Batch ID: `B033_inc0020_hybrid_rescue_screen`
- Config: `configs/proxy_transfer_inc0020_hybrid_rescue_screen.json`
- Logs: `results/raw/inc0020_hybrid_rescue_screen_*.log`
- Parsed files: matching `results/parsed/inc0020_hybrid_rescue_screen_*.json`
- Analysis: `results/analysis/inc0020_hybrid_rescue_screen.json`
- Summary row range: rows where `run_tag` contains `inc0020_hybrid_rescue_screen`
- Decision note: `docs/governance/gates/gate_20260305_195327.md`

## Batch B034
- Batch ID: `B034_inc0020_hybrid_rescue_confirm`
- Config: `configs/proxy_transfer_inc0020_hybrid_rescue_confirm.json`
- Logs: `results/raw/inc0020_hybrid_rescue_confirm_*.log`
- Parsed files: matching `results/parsed/inc0020_hybrid_rescue_confirm_*.json`
- Analysis: `results/analysis/inc0020_hybrid_rescue_confirm.json`
- Summary row range: rows where `run_tag` contains `inc0020_hybrid_rescue_confirm`
- Decision note: `docs/governance/gates/gate_20260305_200621.md`

## Batch B035
- Batch ID: `B035_inc0021_phi_ladder_screen`
- Config: `configs/proxy_transfer_inc0021_phi_ladder_screen.json`
- Logs: `results/raw/inc0021_phi_ladder_screen_*.log`
- Parsed files: matching `results/parsed/inc0021_phi_ladder_screen_*.json`
- Analysis: `results/analysis/inc0021_phi_ladder_screen.json`
- Summary row range: rows where `run_tag` contains `inc0021_phi_ladder_screen`
- Decision note: `docs/governance/gates/gate_20260305_202833.md`

## Batch B036
- Batch ID: `B036_inc0022_phi_log_screen`
- Config: `configs/proxy_transfer_inc0022_phi_log_screen.json`
- Logs: `results/raw/inc0022_phi_log_screen_*.log`
- Parsed files: matching `results/parsed/inc0022_phi_log_screen_*.json`
- Analysis: `results/analysis/inc0022_phi_log_screen.json`
- Summary row range: rows where `run_tag` contains `inc0022_phi_log_screen`
- Decision note: `docs/governance/gates/gate_20260305_205512.md`

## Batch B037
- Batch ID: `B037_inc0022_phi_log_confirm`
- Config: `configs/proxy_transfer_inc0022_phi_log_confirm.json`
- Logs: `results/raw/inc0022_phi_log_confirm_*.log`
- Parsed files: matching `results/parsed/inc0022_phi_log_confirm_*.json`
- Analysis: `results/analysis/inc0022_phi_log_confirm.json`
- Summary row range: rows where `run_tag` contains `inc0022_phi_log_confirm`
- Decision note: `docs/governance/gates/gate_20260305_210615.md`

## Batch B038
- Batch ID: `B038_inc0023_phi3_budget_screen`
- Config: `configs/proxy_transfer_inc0023_phi3_budget_screen.json`
- Logs: `results/raw/inc0023_phi3_budget_screen_*.log`
- Parsed files: matching `results/parsed/inc0023_phi3_budget_screen_*.json`
- Analysis: `results/analysis/inc0023_phi3_budget_screen.json`
- Summary row range: rows where `run_tag` contains `inc0023_phi3_budget_screen`
- Decision note: `docs/governance/gates/gate_20260305_212430.md`

## Batch B039
- Batch ID: `B039_inc0023_phi3_budget_confirm`
- Config: `configs/proxy_transfer_inc0023_phi3_budget_confirm.json`
- Logs: `results/raw/inc0023_phi3_budget_confirm_*.log`
- Parsed files: matching `results/parsed/inc0023_phi3_budget_confirm_*.json`
- Analysis: `results/analysis/inc0023_phi3_budget_confirm.json`
- Summary row range: rows where `run_tag` contains `inc0023_phi3_budget_confirm`
- Decision note: `docs/governance/gates/gate_20260305_213556.md`

## Batch B040
- Batch ID: `B040_ctrl0002_phi3_vs_r0_seedmajor`
- Config: `configs/proxy_transfer_ctrl0002_phi3_vs_r0_seedmajor.json`
- Logs: `results/raw/ctrl0002_phi3_vs_r0_seedmajor_*.log`
- Parsed files: matching `results/parsed/ctrl0002_phi3_vs_r0_seedmajor_*.json`
- Analysis: `results/analysis/ctrl0002_phi3_vs_r0_seedmajor.json`
- Summary row range: rows where `run_tag` contains `ctrl0002_phi3_vs_r0_seedmajor`
- Decision note: `docs/governance/gates/gate_20260305_214935.md`

## Batch B041
- Batch ID: `B041_inc0024_phase_shell_screen`
- Config: `configs/proxy_transfer_inc0024_phase_shell_screen.json`
- Logs: `results/raw/inc0024_phase_shell_screen_*.log`
- Parsed files: matching `results/parsed/inc0024_phase_shell_screen_*.json`
- Analysis: `results/analysis/inc0024_phase_shell_screen.json`
- Summary row range: rows where `run_tag` contains `inc0024_phase_shell_screen`
- Decision note: `docs/governance/gates/gate_20260305_221026.md`

## Batch B042
- Batch ID: `B042_inc0024_phase_shell_confirm`
- Config: `configs/proxy_transfer_inc0024_phase_shell_confirm.json`
- Logs: `results/raw/inc0024_phase_shell_confirm_*.log`
- Parsed files: matching `results/parsed/inc0024_phase_shell_confirm_*.json`
- Analysis: `results/analysis/inc0024_phase_shell_confirm.json`
- Summary row range: rows where `run_tag` contains `inc0024_phase_shell_confirm`
- Decision note: `docs/governance/gates/gate_20260305_222202.md`

## Batch B043
- Batch ID: `B043_inc0026_hopf_diag`
- Config: `configs/proxy_transfer_inc0026_hopf_diag.json`
- Logs: `results/raw/inc0026_hopf_diag_*.log`
- Parsed files: matching `results/parsed/inc0026_hopf_diag_*.json`
- Analysis: `results/analysis/inc0026_hopf_diag.json`
- Summary row range: rows where `run_tag` contains `inc0026_hopf_diag`
- Decision note: `docs/governance/gates/gate_20260305_230403.md`

## Batch B044
- Batch ID: `B044_inc0026_hopf_pilot_screen`
- Config: `configs/proxy_transfer_inc0026_hopf_pilot_screen.json`
- Logs: `results/raw/inc0026_hopf_pilot_screen_*.log`
- Parsed files: matching `results/parsed/inc0026_hopf_pilot_screen_*.json`
- Analysis: `results/analysis/inc0026_hopf_pilot_screen.json`
- Summary row range: rows where `run_tag` contains `inc0026_hopf_pilot_screen`
- Decision note: `docs/governance/gates/gate_20260305_231636.md`

## Batch B045
- Batch ID: `B045_inc0028_hopf_chi_screen`
- Config: `configs/proxy_transfer_inc0028_hopf_chi_screen.json`
- Logs: `results/raw/inc0028_hopf_chi_screen_*.log`
- Parsed files: matching `results/parsed/inc0028_hopf_chi_screen_*.json`
- Analysis: `results/analysis/inc0028_hopf_chi_screen.json`
- Summary row range: rows where `run_tag` contains `inc0028_hopf_chi_screen`
- Decision note: `docs/governance/gates/gate_20260305_235825.md`
- Note: ignore the earlier invalid first attempt at `gate_20260305_234515.md`; routed runs crashed before summary emission due an evaluator bug.

## Batch B046
- Batch ID: `B046_inc0030_hopf_confirm`
- Config: `configs/proxy_transfer_inc0030_hopf_confirm.json`
- Logs: `results/raw/inc0030_hopf_confirm_*.log`
- Parsed files: matching `results/parsed/inc0030_hopf_confirm_*.json`
- Analysis: `results/analysis/inc0030_hopf_confirm.json`
- Summary row range: rows where `run_tag` contains `inc0030_hopf_confirm`
- Decision note: `docs/governance/gates/gate_20260306_001608.md`

## Batch B047
- Batch ID: `B047_inc0029_fib_screen`
- Config: `configs/proxy_transfer_inc0029_fib_screen.json`
- Logs: `results/raw/inc0029_fib_screen_*.log`
- Parsed files: matching `results/parsed/inc0029_fib_screen_*.json`
- Analysis: `results/analysis/inc0029_fib_screen.json`
- Summary row range: rows where `run_tag` contains `inc0029_fib_screen`
- Decision note: `docs/governance/gates/gate_20260306_004144.md`

## Batch B048
- Batch ID: `B048_inc0031_phi2_rung_screen`
- Config: `configs/proxy_transfer_inc0031_phi2_rung_screen.json`
- Logs: `results/raw/inc0031_phi2_rung_screen_*.log`
- Parsed files: matching `results/parsed/inc0031_phi2_rung_screen_*.json`
- Analysis: `results/analysis/inc0031_phi2_rung_screen.json`
- Summary row range: rows where `run_tag` contains `inc0031_phi2_rung_screen`
- Decision note: `docs/governance/gates/gate_20260306_010724.md`

## Batch B049
- Batch ID: `B049_inc0032_phi2_gated_screen`
- Config: `configs/proxy_transfer_inc0032_phi2_gated_screen.json`
- Logs: `results/raw/inc0032_phi2_gated_screen_*.log`
- Parsed files: matching `results/parsed/inc0032_phi2_gated_screen_*.json`
- Analysis: `results/analysis/inc0032_phi2_gated_screen.json`
- Summary row range: rows where `run_tag` contains `inc0032_phi2_gated_screen`
- Decision note: `docs/governance/gates/gate_20260306_014339.md`

## Batch B050
- Batch ID: `B050_inc0033_phi2_band_screen`
- Config: `configs/proxy_transfer_inc0033_phi2_band_screen.json`
- Logs: `results/raw/inc0033_phi2_band_screen_*.log`
- Parsed files: matching `results/parsed/inc0033_phi2_band_screen_*.json`
- Analysis: `results/analysis/inc0033_phi2_band_screen.json`
- Summary row range: rows where `run_tag` contains `inc0033_phi2_band_screen`
- Decision note: `docs/governance/gates/gate_20260306_021036.md`

## Batch B051
- Batch ID: `B051_inc0034_blended_hopf_screen`
- Config: `configs/proxy_transfer_inc0034_blended_hopf_screen.json`
- Logs: `results/raw/inc0034_blended_hopf_screen_*.log`
- Parsed files: matching `results/parsed/inc0034_blended_hopf_screen_*.json`
- Analysis: `results/analysis/inc0034_blended_hopf_screen.json`
- Summary row range: rows where `run_tag` contains `inc0034_blended_hopf_screen`
- Decision note: `docs/governance/gates/gate_20260306_024928.md`

## Batch B052
- Batch ID: `B052_inc0035_alignment_diag_screen`
- Config: `configs/proxy_transfer_inc0035_alignment_diag_screen.json`
- Logs: `results/raw/inc0035_alignment_diag_screen_*.log`
- Parsed files: matching `results/parsed/inc0035_alignment_diag_screen_*.json`
- Analysis: `results/analysis/inc0035_alignment_diag_screen.json`
- Summary row range: rows where `run_tag` contains `inc0035_alignment_diag_screen`
- Decision note: `docs/governance/gates/gate_20260306_030909.md`

## Batch B053
- Batch ID: `B053_inc0035_shell_anchor_screen`
- Config: `configs/proxy_transfer_inc0035_shell_anchor_screen.json`
- Logs: `results/raw/inc0035_shell_anchor_screen_*.log`
- Parsed files: matching `results/parsed/inc0035_shell_anchor_screen_*.json`
- Analysis: `results/analysis/inc0035_shell_anchor_screen.json`
- Summary row range: rows where `run_tag` contains `inc0035_shell_anchor_screen`
- Decision note: `docs/governance/gates/gate_20260306_032618.md`

## Batch B054
- Batch ID: `B054_inc0036_chart_iso_screen`
- Config: `configs/proxy_transfer_inc0036_chart_iso_screen.json`
- Logs: `results/raw/inc0036_chart_iso_screen_*.log`
- Parsed files: matching `results/parsed/inc0036_chart_iso_screen_*.json`
- Analysis: `results/analysis/inc0036_chart_iso_screen.json`
- Summary row range: rows where `run_tag` contains `inc0036_chart_iso_screen`
- Decision note: `docs/governance/gates/gate_20260306_074531.md`

## Batch B055
- Batch ID: `B055_inc0037_isometric_band_screen`
- Config: `configs/proxy_transfer_inc0037_isometric_band_screen.json`
- Logs: `results/raw/inc0037_isometric_band_screen_*.log`
- Parsed files: matching `results/parsed/inc0037_isometric_band_screen_*.json`
- Analysis: `results/analysis/inc0037_isometric_band_screen.json`
- Summary row range: rows where `run_tag` contains `inc0037_isometric_band_screen`
- Decision note: `docs/governance/gates/gate_20260306_075923.md`

## Batch B056
- Batch ID: `B056_inc0038_bounded_band_screen`
- Config: `configs/proxy_transfer_inc0038_bounded_band_screen.json`
- Logs: `results/raw/inc0038_bounded_band_screen_*.log`
- Parsed files: matching `results/parsed/inc0038_bounded_band_screen_*.json`
- Analysis: `results/analysis/inc0038_bounded_band_screen.json`
- Summary row range: rows where `run_tag` contains `inc0038_bounded_band_screen`
- Decision note: `docs/governance/gates/gate_20260306_082106.md`

## Batch B057
- Batch ID: `B057_inc0039_route_memory_screen`
- Config: `configs/proxy_transfer_inc0039_route_memory_screen.json`
- Logs: `results/raw/inc0039_route_memory_screen_*.log`
- Parsed files: matching `results/parsed/inc0039_route_memory_screen_*.json`
- Analysis: `results/analysis/inc0039_route_memory_screen.json`
- Summary row range: rows where `run_tag` contains `inc0039_route_memory_screen`
- Decision note: `docs/governance/gates/gate_20260306_084204.md`

## Batch B058
- Batch ID: `B058_ctrl0003_hopf_frontier_confirm`
- Config: `configs/proxy_transfer_ctrl0003_hopf_frontier_confirm.json`
- Logs: `results/raw/ctrl0003_hopf_frontier_confirm_*.log`
- Parsed files: matching `results/parsed/ctrl0003_hopf_frontier_confirm_*.json`
- Analysis: `results/analysis/ctrl0003_hopf_frontier_confirm.json`
- Summary row range: rows where `run_tag` contains `ctrl0003_hopf_frontier_confirm`
- Decision note: `docs/governance/gates/gate_20260306_085323.md`

## Batch B059
- Batch ID: `B059_inc0040_cost_screen`
- Config: `configs/proxy_transfer_inc0040_cost_screen.json`
- Logs: `results/raw/inc0040_cost_screen_*.log`
- Parsed files: matching `results/parsed/inc0040_cost_screen_*.json`
- Analysis: `results/analysis/inc0040_cost_screen.json`
- Summary row range: rows where `run_tag` contains `inc0040_cost_screen`
- Decision note: `docs/governance/gates/gate_20260306_091429.md`

## Batch B060
- Batch ID: `B060_inc0040_cost_confirm`
- Config: `configs/proxy_transfer_inc0040_cost_confirm.json`
- Logs: `results/raw/inc0040_cost_confirm_*.log`
- Parsed files: matching `results/parsed/inc0040_cost_confirm_*.json`
- Analysis: `results/analysis/inc0040_cost_confirm.json`
- Summary row range: rows where `run_tag` contains `inc0040_cost_confirm`
- Decision note: `docs/governance/gates/gate_20260306_092503.md`

## Batch B061
- Batch ID: `B061_inc0041_cost_large_subset`
- Config: `configs/proxy_transfer_inc0041_cost_large_subset.json`
- Logs: `results/raw/inc0041_cost_large_subset_*.log`
- Parsed files: matching `results/parsed/inc0041_cost_large_subset_*.json`
- Analysis: `results/analysis/inc0041_cost_large_subset.json`
- Summary row range: rows where `run_tag` contains `inc0041_cost_large_subset`
- Decision note: `docs/governance/gates/gate_20260306_093641.md`

## Batch B050
- Batch ID: `B050_inc0042_timing_diag`
- Config: `configs/proxy_transfer_inc0042_timing_diag.json`
- Logs: `results/raw/inc0042_timing_diag_*.log`
- Parsed files: matching `results/parsed/inc0042_timing_diag_*.json`
- Analysis: `results/analysis/inc0042_timing_diag.json`
- Summary row range: rows where `run_tag` contains `inc0042_timing_diag`
- Decision note: `docs/governance/gates/gate_20260306_094708.md`

## Batch B051
- Batch ID: `B051_inc0043_train_route_static_screen`
- Config: `configs/proxy_transfer_inc0043_train_route_static_screen.json`
- Logs: `results/raw/inc0043_train_route_static_screen_*.log`
- Parsed files: matching `results/parsed/inc0043_train_route_static_screen_*.json`
- Analysis: `results/analysis/inc0043_train_route_static_screen.json`
- Summary row range: rows where `run_tag` contains `inc0043_train_route_static_screen`
- Decision note: `docs/governance/gates/gate_20260306_095825.md`

## Batch B052
- Batch ID: `B052_inc0043_train_route_static_confirm`
- Config: `configs/proxy_transfer_inc0043_train_route_static_confirm.json`
- Logs: `results/raw/inc0043_train_route_static_confirm_*.log`
- Parsed files: matching `results/parsed/inc0043_train_route_static_confirm_*.json`
- Analysis: `results/analysis/inc0043_train_route_static_confirm.json`
- Summary row range: rows where `run_tag` contains `inc0043_train_route_static_confirm`
- Decision note: `docs/governance/gates/gate_20260306_100530.md`

## Batch B053
- Batch ID: `B053_inc0044_static_chart_pressure_screen`
- Config: `configs/proxy_transfer_inc0044_static_chart_pressure_screen.json`
- Logs: `results/raw/inc0044_static_chart_pressure_screen_*.log`
- Parsed files: matching `results/parsed/inc0044_static_chart_pressure_screen_*.json`
- Analysis: `results/analysis/inc0044_static_chart_pressure_screen.json`
- Summary row range: rows where `run_tag` contains `inc0044_static_chart_pressure_screen`
- Decision note: `docs/governance/gates/gate_20260306_101427.md`

## Batch B054
- Batch ID: `B054_inc0044_static_chart_pressure_confirm`
- Config: `configs/proxy_transfer_inc0044_static_chart_pressure_confirm.json`
- Logs: `results/raw/inc0044_static_chart_pressure_confirm_*.log`
- Parsed files: matching `results/parsed/inc0044_static_chart_pressure_confirm_*.json`
- Analysis: `results/analysis/inc0044_static_chart_pressure_confirm.json`
- Summary row range: rows where `run_tag` contains `inc0044_static_chart_pressure_confirm`
- Decision note: `docs/governance/gates/gate_20260306_102058.md`

## Batch B055
- Batch ID: `B055_inc0045_static_chart_floor_screen`
- Config: `configs/proxy_transfer_inc0045_static_chart_floor_screen.json`
- Logs: `results/raw/inc0045_static_chart_floor_screen_*.log`
- Parsed files: matching `results/parsed/inc0045_static_chart_floor_screen_*.json`
- Analysis: `results/analysis/inc0045_static_chart_floor_screen.json`
- Summary row range: rows where `run_tag` contains `inc0045_static_chart_floor_screen`
- Decision note: `docs/governance/gates/gate_20260306_103538.md`

## Batch B056
- Batch ID: `B056_inc0045_static_chart_floor_confirm`
- Config: `configs/proxy_transfer_inc0045_static_chart_floor_confirm.json`
- Logs: `results/raw/inc0045_static_chart_floor_confirm_*.log`
- Parsed files: matching `results/parsed/inc0045_static_chart_floor_confirm_*.json`
- Analysis: `results/analysis/inc0045_static_chart_floor_confirm.json`
- Summary row range: rows where `run_tag` contains `inc0045_static_chart_floor_confirm`
- Decision note: `docs/governance/gates/gate_20260306_103811.md`

## Batch B057
- Batch ID: `B057_inc0046_static_scale_robustness_screen`
- Config: `configs/proxy_transfer_inc0046_static_scale_robustness_screen.json`
- Logs: `results/raw/inc0046_static_scale_robustness_screen_*.log`
- Parsed files: matching `results/parsed/inc0046_static_scale_robustness_screen_*.json`
- Analysis: `results/analysis/inc0046_static_scale_robustness_screen.json`
- Summary row range: rows where `run_tag` contains `inc0046_static_scale_robustness_screen`
- Decision note: `docs/governance/gates/gate_20260306_104728.md`

## Batch B058
- Batch ID: `B058_inc0046_static_scale_robustness_confirm`
- Config: `configs/proxy_transfer_inc0046_static_scale_robustness_confirm.json`
- Logs: `results/raw/inc0046_static_scale_robustness_confirm_*.log`
- Parsed files: matching `results/parsed/inc0046_static_scale_robustness_confirm_*.json`
- Analysis: `results/analysis/inc0046_static_scale_robustness_confirm.json`
- Summary row range: rows where `run_tag` contains `inc0046_static_scale_robustness_confirm`
- Decision note: `docs/governance/gates/gate_20260306_105119.md`

## Batch B059
- Batch ID: `B059_inc0047_near_full_proxy_scale_screen`
- Config: `configs/proxy_transfer_inc0047_near_full_proxy_scale_screen.json`
- Logs: `results/raw/inc0047_near_full_proxy_scale_screen_*.log`
- Parsed files: matching `results/parsed/inc0047_near_full_proxy_scale_screen_*.json`
- Analysis: `results/analysis/inc0047_near_full_proxy_scale_screen.json`
- Summary row range: rows where `run_tag` contains `inc0047_near_full_proxy_scale_screen`
- Decision note: `docs/governance/gates/gate_20260306_105627.md`

## Batch B060
- Batch ID: `B060_inc0047_near_full_proxy_scale_confirm`
- Config: `configs/proxy_transfer_inc0047_near_full_proxy_scale_confirm.json`
- Logs: `results/raw/inc0047_near_full_proxy_scale_confirm_*.log`
- Parsed files: matching `results/parsed/inc0047_near_full_proxy_scale_confirm_*.json`
- Analysis: `results/analysis/inc0047_near_full_proxy_scale_confirm.json`
- Summary row range: rows where `run_tag` contains `inc0047_near_full_proxy_scale_confirm`
- Decision note: `docs/governance/gates/gate_20260306_110140.md`

## Batch B063
- Batch ID: `B063_inc0048_retrieval_translation_screen`
- Config: `configs/proxy_transfer_inc0048_retrieval_translation_screen.json`
- Logs: `results/raw/inc0048_retrieval_translation_screen_*.log`
- Parsed files: matching `results/parsed/inc0048_retrieval_translation_screen_*.json`
- Analysis: `results/analysis/inc0048_retrieval_translation_screen.json`
- Summary row range: rows where `run_tag` contains `inc0048_retrieval_translation_screen`
- Decision note: `docs/governance/gates/gate_20260306_111959.md`

## Batch B064
- Batch ID: `B064_inc0049_retrieval_cost_rescue_screen`
- Config: `configs/proxy_transfer_inc0049_retrieval_cost_rescue_screen.json`
- Logs: `results/raw/inc0049_retrieval_cost_rescue_screen_*.log`
- Parsed files: matching `results/parsed/inc0049_retrieval_cost_rescue_screen_*.json`
- Analysis: `results/analysis/inc0049_retrieval_cost_rescue_screen.json`
- Summary row range: rows where `run_tag` contains `inc0049_retrieval_cost_rescue_screen`
- Decision note: `docs/governance/gates/gate_20260306_113201.md`

## Batch B065
- Batch ID: `B065_inc0051_retrieval_amortization_screen`
- Config: `configs/proxy_transfer_inc0051_retrieval_amortization_screen.json`
- Logs: `results/raw/inc0051_retrieval_amortization_screen_*.log`
- Parsed files: matching `results/parsed/inc0051_retrieval_amortization_screen_*.json`
- Analysis: `results/analysis/inc0051_retrieval_amortization_screen.json`
- Summary row range: rows where `run_tag` contains `inc0051_retrieval_amortization_screen`
- Decision note: `docs/governance/gates/gate_20260306_114654.md`

## Batch B066
- Batch ID: `B066_inc0052_retrieval_amortization_confirm`
- Config: `configs/proxy_transfer_inc0052_retrieval_amortization_confirm.json`
- Logs: `results/raw/inc0052_retrieval_amortization_confirm_*.log`
- Parsed files: matching `results/parsed/inc0052_retrieval_amortization_confirm_*.json`
- Analysis: `results/analysis/inc0052_retrieval_amortization_confirm.json`
- Summary row range: rows where `run_tag` contains `inc0052_retrieval_amortization_confirm`
- Decision note: `docs/governance/gates/gate_20260306_115931.md`

## Batch B096
- Batch ID: `B096_inc0050_dynamic_h4_screen`
- Config: `configs/proxy_transfer_inc0050_dynamic_h4_screen.json`
- Logs: `results/raw/inc0050_dynamic_h4_screen_*.log`
- Parsed files: matching `results/parsed/inc0050_dynamic_h4_screen_*.json`
- Analysis: `results/analysis/inc0050_dynamic_h4_screen.json`
- Summary row range: rows where `run_tag` contains `inc0050_dynamic_h4_screen`
- Decision note: `docs/governance/gates/gate_20260306_122447.md`

## Batch B097
- Batch ID: `B097_inc0050_dynamic_h4_confirm`
- Config: `configs/proxy_transfer_inc0050_dynamic_h4_confirm.json`
- Logs: `results/raw/inc0050_dynamic_h4_confirm_*.log`
- Parsed files: matching `results/parsed/inc0050_dynamic_h4_confirm_*.json`
- Analysis: `results/analysis/inc0050_dynamic_h4_confirm.json`
- Summary row range: rows where `run_tag` contains `inc0050_dynamic_h4_confirm`
- Decision note: `docs/governance/gates/gate_20260306_122733.md`

## Batch B098
- Batch ID: `B098_inc0054_tangent_flow_route_law_screen_invalid`
- Config: `configs/proxy_transfer_inc0054_tangent_flow_route_law_screen.json`
- Logs: `results/raw/inc0054_tangent_flow_route_law_screen_*.log`
- Analysis: `results/analysis/inc0054_tangent_flow_route_law_screen.json`
- Decision note: `docs/governance/gates/gate_20260306_124108.md`
- Note: first screen attempt was invalid because `STATIC_GLOBAL` failed to emit a summary after an evaluator bug.

## Batch B099
- Batch ID: `B099_inc0054_tangent_flow_route_law_screen`
- Config: `configs/proxy_transfer_inc0054_tangent_flow_route_law_screen.json`
- Logs: `results/raw/inc0054_tangent_flow_route_law_screen_*.log`
- Parsed files: matching `results/parsed/inc0054_tangent_flow_route_law_screen_*.json`
- Analysis: `results/analysis/inc0054_tangent_flow_route_law_screen.json`
- Summary row range: rows where `run_tag` contains `inc0054_tangent_flow_route_law_screen`
- Decision note: `docs/governance/gates/gate_20260306_124322.md`

## Batch B100
- Batch ID: `B100_inc0055_product_h4x4_retrieval_field_screen`
- Config: `configs/proxy_transfer_inc0055_product_h4x4_retrieval_field_screen.json`
- Logs: `results/raw/inc0055_product_h4x4_retrieval_field_screen_*.log`
- Parsed files: matching `results/parsed/inc0055_product_h4x4_retrieval_field_screen_*.json`
- Analysis: `results/analysis/inc0055_product_h4x4_retrieval_field_screen.json`
- Summary row range: rows where `run_tag` contains `inc0055_product_h4x4_retrieval_field_screen`
- Decision note: `docs/governance/gates/gate_20260306_125229.md`

## Batch B101
- Batch ID: `B101_inc0055_product_h4x4_retrieval_field_confirm`
- Config: `configs/proxy_transfer_inc0055_product_h4x4_retrieval_field_confirm.json`
- Logs: `results/raw/inc0055_product_h4x4_retrieval_field_confirm_*.log`
- Parsed files: matching `results/parsed/inc0055_product_h4x4_retrieval_field_confirm_*.json`
- Analysis: `results/analysis/inc0055_product_h4x4_retrieval_field_confirm.json`
- Summary row range: rows where `run_tag` contains `inc0055_product_h4x4_retrieval_field_confirm`
- Decision note: `docs/governance/gates/gate_20260306_125455.md`

## Batch B102
- Batch ID: `B102_inc0056_product_complex_translation_screen`
- Config: `configs/proxy_transfer_inc0056_product_complex_translation_screen.json`
- Logs: `results/raw/inc0056_product_complex_translation_screen_*.log`
- Parsed files: matching `results/parsed/inc0056_product_complex_translation_screen_*.json`
- Analysis: `results/analysis/inc0056_product_complex_translation_screen.json`
- Summary row range: rows where `run_tag` contains `inc0056_product_complex_translation_screen`
- Decision note: `docs/governance/gates/gate_20260306_131055.md`

## Batch B103
- Batch ID: `B103_inc0056_product_complex_translation_confirm`
- Config: `configs/proxy_transfer_inc0056_product_complex_translation_confirm.json`
- Logs: `results/raw/inc0056_product_complex_translation_confirm_*.log`
- Parsed files: matching `results/parsed/inc0056_product_complex_translation_confirm_*.json`
- Analysis: `results/analysis/inc0056_product_complex_translation_confirm.json`
- Summary row range: rows where `run_tag` contains `inc0056_product_complex_translation_confirm`
- Decision note: `docs/governance/gates/gate_20260306_131507.md`

## Batch B104
- Batch ID: `B104_inc0057_product_complex_backfill_smallbucket_screen`
- Config: `configs/proxy_transfer_inc0057_product_complex_backfill_smallbucket_screen.json`
- Logs: `results/raw/inc0057_product_complex_backfill_smallbucket_screen_*.log`
- Parsed files: matching `results/parsed/inc0057_product_complex_backfill_smallbucket_screen_*.json`
- Analysis: `results/analysis/inc0057_product_complex_backfill_smallbucket_screen.json`
- Summary row range: rows where `run_tag` contains `inc0057_product_complex_backfill_smallbucket_screen`
- Decision note: `docs/governance/gates/gate_20260306_135217.md`
- Note: the earlier mixed selective screen was stopped after low-margin backfill showed pathological trigger frequency and cost; its partial logs remain in `results/raw/` for forensics only.

## Batch B105
- Batch ID: `B105_inc0058_product_complex_rerank_screen`
- Config: `configs/proxy_transfer_inc0058_product_complex_rerank_screen.json`
- Logs: `results/raw/inc0058_product_complex_rerank_screen_*.log`
- Parsed files: matching `results/parsed/inc0058_product_complex_rerank_screen_*.json`
- Analysis: `results/analysis/inc0058_product_complex_rerank_screen.json`
- Summary row range: rows where `run_tag` contains `inc0058_product_complex_rerank_screen`
- Decision note: `docs/governance/gates/gate_20260306_140424.md`
