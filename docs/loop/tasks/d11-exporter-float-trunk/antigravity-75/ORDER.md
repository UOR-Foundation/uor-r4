# ORDER: d11-exporter-float-trunk, antigravity-75

- **Lab:** antigravity
- **Item:** `d11-exporter-float-trunk` ([`docs/roadmap/items/d11-exporter-float-trunk.md`](../../../roadmap/items/d11-exporter-float-trunk.md), Issue #2820)
- **Milestone:** M4
- **Base:** `9e7f7a0fdd40b03dc5aa63d0588c605a025d246d`
- **Estimate:** 350 lines (code + tests + docs)
- **Scope:**
  - `crates/uor-r4-integer/**`
  - `docs/mechanisms/lut-serving.md`

## Goal

Build the D11 integer exporter and reader/evaluator support for a D37 offline float trunk (`RecGates::Geometric` dyadic recurrence with quaternion transport and `Gate::Hamilton` with 120-element icosian snap) in `crates/uor-r4-integer/src/stack/`, verify finite CPU controls and the native zero-multiply audit (`scripts/audit_zero_matmul_serving.py --stack`), record the result in `docs/mechanisms/lut-serving.md` and `docs/research/notebook/m4-serving.md`, and complete the non-author `TEST FITNESS: FIT` and D68 decision reviews for the Stage 4 float-trunk export-gap evaluation (setting `status: held` until a Stage 3 float trunk is trained).

## Cancel probe

Run first before any code or doc edit:
1. Verify `uor-harness loop claim d11-exporter-float-trunk --lab-topic d11-exporter-float` refusal is solely the `infra-loop-harness` (`PR #3169`, `codex-1`) umbrella overlap on `docs/mechanisms/lut-serving.md`, and verify via `gh pr view 3169 --json files,isDraft,updatedAt` that PR #3169 (`codex-1`, idle draft since `2026-10-25T01:07:47Z`) modifies only `PLAN.md`, `docs/integration/DECISIONS.md`, and `docs/loop/tasks/infra-loop-harness/codex-1/**` (`0` overlapping files with `d11-exporter-float-trunk`). Claimed under the `run-antigravity.txt` Part 3 Step 2 umbrella-lock exception (`instrument`).
2. Verify all three dependencies of `d11-exporter-float-trunk` (`float-training-module` #2823, `m1-d11-exporter-discrete` #2838, `owner-decision-float-path-scope` #2875) are `status: done` on `origin/main` (`9e7f7a0fdd40b03dc5aa63d0588c605a025d246d`), while no trained Stage 3 float-trunk checkpoint exists yet (`#2890`, `#2824`, `#2891` wait on `r-secant-surrogate-descent` `#2886`). Classify the absence of a trained Stage 3 float trunk for the GPU DEV export-gap run as `instrument` (upstream Stage 3 training dependency, not a mechanism negative).

## Done when

1. A design note and one-page export-gap pre-registration are posted on Issue #2820 covering: (a) snapshot format handed to the exporter, (b) dyadic recurrence (`RATE_TABLE` / `KEEP_TABLE` at `i = min(floor(16 * clamp(g + d, 0, 1)), 15)`) with quaternion transport as shifts, adds, and table lookups, (c) 120-element icosian snap (`stack_snap_select` / `stack_snap_rotation`) for the Hamilton gate (`Gate::Hamilton`), (d) absence of softmax, squashing, and dense GEMM, and (e) finite CPU control specification and DEV two-seed export-gap band (`398` top-1, `748` top-8, `930` admission).
2. `crates/uor-r4-integer/src/stack/` exports and evaluates a geometric float-trunk snapshot (`RecGates::Geometric` + `Gate::Hamilton` with icosian snap and softmax-free flock rank read) in `UORLUT01` (`uor-r4.lut-stack/3`), with finite CPU controls comparing the float forward and exported integer forward across every input of a small vocabulary (`cargo test -p uor-r4-integer --features serving-wasm --no-fail-fast`) and `python3 scripts/audit_zero_matmul_serving.py <rlib> --stack` passing with zero `mul`/`div`/`float` instructions.
3. Fresh-context non-author `TEST FITNESS: FIT` and D68 decision reviews pass and are logged on #2820 (`[antigravity] owner log:`), `docs/mechanisms/lut-serving.md` and `docs/research/notebook/m4-serving.md` are updated, `docs/roadmap/items/d11-exporter-float-trunk.md` is set to `status: held` pending Stage 3 float-trunk training, and `uor-harness loop pr-check --local` and `uor-harness loop gates --full` pass.

## Stop and ask (stop and decide)

- Never edit `crates/uor-r4-training/src/stack_export.rs` or re-enable it in `crates/uor-r4-training`.
- Never introduce softmax, squashing, dense GEMM, or multiplier/divider/float instructions on audited `stack_` symbols.
