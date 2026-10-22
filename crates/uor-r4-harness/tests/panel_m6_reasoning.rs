//! Integration tests for the frozen `m6-reasoning` acceptance panel (`m6-reasoning-panel-freeze`,
//! #2866 / #2034).

use std::collections::BTreeSet;

use uor_r4_harness::panels::m6_reasoning::{
    evaluate_count_table_baseline_noise, generate_frozen_m6_panel, generate_m6_panel,
    grade_code_task_program, grade_m6_panel, grade_m6_row, M6OracleDiagnostic, M6RowSpec,
    SandboxError, SandboxOp, SandboxProgram, ABSTAIN_UNSATISFIABLE_TOKEN, M6_PANEL_FROZEN_SEED_V1,
    M6_PANEL_ROWS_PER_FAMILY, M6_PANEL_TOTAL_ROWS,
};

/// 1. Same seed gives the exact same bytes and SHA-256 content hashes; `verify_zero_overlap` passes.
#[test]
fn panel_seed_determinism_and_sha256_seal() {
    let p1 = generate_frozen_m6_panel();
    let p2 = generate_m6_panel(M6_PANEL_FROZEN_SEED_V1);
    assert_eq!(p1, p2);
    assert_eq!(p1.rows.len(), M6_PANEL_TOTAL_ROWS);
    p1.verify_zero_overlap().expect("zero overlap check");

    let unscored_1 = p1.to_canonical_jsonl_bytes(false);
    let unscored_2 = p2.to_canonical_jsonl_bytes(false);
    assert_eq!(unscored_1, unscored_2);
    assert_eq!(unscored_1.len(), 79_215);

    let sealed_1 = p1.to_canonical_jsonl_bytes(true);
    let sealed_2 = p2.to_canonical_jsonl_bytes(true);
    assert_eq!(sealed_1, sealed_2);
    assert_eq!(sealed_1.len(), 206_422);

    let unscored_sha = p1.unscored_sha256();
    let sealed_sha = p1.sealed_oracle_sha256();
    assert_eq!(
        unscored_sha,
        "ac2034eda9b8ade5b6c760461a2c40c167955289a7f4a53c5ad1081894ab3edc"
    );
    assert_eq!(
        sealed_sha,
        "d1e5e4eb295d06d711b90e79805e6e35a84caa47666baeed6e202e203d407203"
    );
}

/// 2. Overlap exclusion: zero overlap with `m6_exact_arithmetic.rs` / `m6_ablation.rs` CPU
/// controls and zero prompt or canonical-target overlap across independent seeds.
#[test]
fn panel_overlap_exclusion_and_corpus_disjointness() {
    let frozen = generate_frozen_m6_panel();
    frozen
        .verify_zero_overlap()
        .expect("frozen panel passes overlap exclusion");

    let alt_seed = generate_m6_panel(M6_PANEL_FROZEN_SEED_V1 ^ 0xDEAD_BEEF_CAFE_BABE);
    alt_seed
        .verify_zero_overlap()
        .expect("alternate seed panel passes overlap exclusion");

    let frozen_prompts: BTreeSet<&str> = frozen.rows.iter().map(|r| r.prompt.as_str()).collect();
    let alt_prompts: BTreeSet<&str> = alt_seed.rows.iter().map(|r| r.prompt.as_str()).collect();
    assert_eq!(
        frozen_prompts.intersection(&alt_prompts).count(),
        0,
        "independent seeds must have zero prompt overlap"
    );

    // Also verify that every CodeTasks row generates valid standalone Rust test source.
    for row in &frozen.rows {
        if let M6RowSpec::CodeTasks(spec) = &row.spec {
            let rust_src = spec
                .canonical_program
                .to_rust_source("solve", &spec.test_vectors);
            assert!(rust_src.contains("pub fn solve(x: i64) -> i64"));
            assert!(rust_src.contains("#[test]\nfn verify_vectors()"));
        }
    }
}

/// 3. Each family oracle (`Compositions`, `AlteredConstraints`, `CodeTasks`) accepts known-correct
/// answers (`240 / 240`) and rejects known-wrong answers (`0 / 240`), including intermediate-state
/// errors, default-habit capture errors, false abstentions, and sandbox step-cap/overflow traps.
#[test]
fn family_oracles_accept_correct_and_reject_wrong_answers() {
    let panel = generate_frozen_m6_panel();

    // Known-correct answers: 240 / 240 pass.
    let canonical_answers: Vec<String> = panel.rows.iter().map(|r| r.canonical_answer()).collect();
    let pass_card = grade_m6_panel(&panel, &canonical_answers);
    assert_eq!(pass_card.overall_passed, M6_PANEL_TOTAL_ROWS);
    assert_eq!(pass_card.compositions_passed, M6_PANEL_ROWS_PER_FAMILY);
    assert_eq!(
        pass_card.altered_constraints_passed,
        M6_PANEL_ROWS_PER_FAMILY
    );
    assert_eq!(pass_card.altered_constraints_satisfiable_passed, 60);
    assert_eq!(pass_card.altered_constraints_abstention_passed, 20);
    assert_eq!(pass_card.habit_capture_errors, 0);
    assert_eq!(pass_card.code_tasks_passed, M6_PANEL_ROWS_PER_FAMILY);

    // Also verify `grade_code_task_program` passes all 80 canonical sandbox programs and rejects
    // perturbed programs (off-by-one return, division by zero, and step-limit exceeded).
    for row in &panel.rows {
        if let M6RowSpec::CodeTasks(spec) = &row.spec {
            let ok_grade = grade_code_task_program(&row.row_id, spec, &spec.canonical_program);
            assert!(ok_grade.passed);

            // Program returning constant 0 must fail.
            let zero_prog = SandboxProgram {
                ops: vec![
                    SandboxOp::LoadImm { dst: 1, val: 0 },
                    SandboxOp::Return { src: 1 },
                ],
            };
            let zero_grade = grade_code_task_program(&row.row_id, spec, &zero_prog);
            assert!(!zero_grade.passed);

            // Program dividing by zero must fail with CodeSandboxError(DivisionByZero).
            let div_zero_prog = SandboxProgram {
                ops: vec![
                    SandboxOp::LoadImm { dst: 1, val: 0 },
                    SandboxOp::Div {
                        dst: 2,
                        lhs: 0,
                        rhs: 1,
                    },
                    SandboxOp::Return { src: 2 },
                ],
            };
            let div_grade = grade_code_task_program(&row.row_id, spec, &div_zero_prog);
            assert!(!div_grade.passed);
            assert_eq!(
                div_grade.diagnostic,
                M6OracleDiagnostic::CodeSandboxError(SandboxError::DivisionByZero)
            );

            // Program exceeding MAX_SANDBOX_STEPS (300 instructions) must fail with StepLimitExceeded.
            let mut long_ops = vec![SandboxOp::Copy { dst: 1, src: 0 }; 260];
            long_ops.push(SandboxOp::Return { src: 1 });
            let long_prog = SandboxProgram { ops: long_ops };
            let long_grade = grade_code_task_program(&row.row_id, spec, &long_prog);
            assert!(!long_grade.passed);
            assert!(matches!(
                long_grade.diagnostic,
                M6OracleDiagnostic::CodeSandboxError(SandboxError::StepLimitExceeded { .. })
            ));
        }
    }

    // Known-wrong answers across all 240 rows:
    // - Compositions: right final value but wrong intermediate state (`INT + 1`), AND right
    //   intermediate state but wrong final value (`OUT + 1`).
    // - AlteredConstraints: `default_habit_answer` (must be rejected and flagged as
    //   `HabitCaptureError` on the 60 satisfiable rows and `FailedToAbstainOnUnsatisfiableRow` on
    //   the 20 unsatisfiable rows).
    // - CodeTasks: perturbing V3 by +1 (`V3 + 1`).
    let mut wrong_answers = Vec::with_capacity(panel.rows.len());
    for row in &panel.rows {
        match &row.spec {
            M6RowSpec::Compositions(c) => {
                // Corrupt intermediate state while keeping final output exact: must be rejected!
                let bad_int = format!(
                    "INT={};OUT={}",
                    c.expected_intermediate + 1,
                    c.expected_final
                );
                let g_int = grade_m6_row(row, &bad_int);
                assert!(!g_int.passed);
                assert!(matches!(
                    g_int.diagnostic,
                    M6OracleDiagnostic::IntermediateStateMismatch { .. }
                ));

                // Corrupt final output while keeping intermediate state exact: must be rejected!
                let bad_out = format!(
                    "INT={};OUT={}",
                    c.expected_intermediate,
                    c.expected_final + 1
                );
                wrong_answers.push(bad_out);
            }
            M6RowSpec::AlteredConstraints(a) => {
                // Supply the un-altered default habit answer: must be rejected!
                wrong_answers.push(format!("VAL={}", a.default_habit_answer()));
            }
            M6RowSpec::CodeTasks(k) => {
                let (v0, v1, v2, v3) = (
                    k.test_vectors[0].1,
                    k.test_vectors[1].1,
                    k.test_vectors[2].1,
                    k.test_vectors[3].1 + 1,
                );
                wrong_answers.push(format!("V0={v0};V1={v1};V2={v2};V3={v3}"));
            }
        }
    }

    let fail_card = grade_m6_panel(&panel, &wrong_answers);
    assert_eq!(fail_card.overall_passed, 0);
    assert_eq!(fail_card.compositions_passed, 0);
    assert_eq!(fail_card.altered_constraints_passed, 0);
    assert_eq!(fail_card.habit_capture_errors, 60);
    assert_eq!(fail_card.code_tasks_passed, 0);

    // Also verify that emitting ABSTAIN:UNSATISFIABLE on all 240 rows scores only 20/240 (the 20
    // unsatisfiable rows) and fails all 60 satisfiable AlteredConstraints rows with
    // FalseAbstentionOnSatisfiableRow.
    let all_abstain = vec![ABSTAIN_UNSATISFIABLE_TOKEN.to_string(); panel.rows.len()];
    let abstain_card = grade_m6_panel(&panel, &all_abstain);
    assert_eq!(abstain_card.overall_passed, 20);
    assert_eq!(abstain_card.compositions_passed, 0);
    assert_eq!(abstain_card.altered_constraints_passed, 20);
    assert_eq!(abstain_card.altered_constraints_satisfiable_passed, 0);
    assert_eq!(abstain_card.altered_constraints_abstention_passed, 20);
    assert_eq!(abstain_card.code_tasks_passed, 0);
}

/// 4. Lawful non-neural count-table baseline noise measurement (CHARTER §4.4).
#[test]
fn lawful_count_table_baseline_and_noise_measurement() {
    let panel = generate_frozen_m6_panel();
    let report = evaluate_count_table_baseline_noise(&panel);

    assert_eq!(report.num_seeds, 8);
    assert_eq!(report.numeric_only_scores, vec![2, 1, 1, 2, 0, 0, 0, 0]);
    assert_eq!(
        report.context_table_scores,
        vec![12, 16, 17, 18, 14, 17, 12, 18]
    );
    assert_eq!(
        report.family_modal_ceiling, 20,
        "family-conditioned modal count table ceiling is the 20 ABSTAIN:UNSATISFIABLE rows"
    );
    assert_eq!(report.context_min_score, 12);
    assert_eq!(report.context_max_score, 18);
    assert_eq!(report.measured_noise_half_band_rows, 3);
}

/// 5. Registry integration: `docs/harness/panels/m6-reasoning.json` is frozen, passes `panel_frozen`
/// and `panel_grader_permitted`, and its 4 bars score `PASS`, `FAIL`, and `AT NOISE` via
/// `uor_r4_harness::panels::classify`.
#[test]
fn m6_reasoning_json_registry_is_frozen_and_scorable() {
    let repo = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("repo root")
        .to_path_buf();
    let paths = uor_r4_harness::StatePaths::resolve(&repo, None, None);
    let regs = uor_r4_harness::Registries::load(&paths).expect("registries");
    let panel = uor_r4_harness::panels::find_panel(&regs.panels, "m6-reasoning")
        .expect("m6-reasoning panel in registry");

    assert!(panel.frozen, "m6-reasoning must be frozen");
    assert!(
        uor_r4_harness::preprules::panel_grader_permitted(&paths).is_empty(),
        "m6-reasoning grader must pass D28 panel_grader_permitted"
    );
    let frozen_findings = uor_r4_harness::preprules::panel_frozen(&paths, &regs);
    assert!(
        !frozen_findings
            .iter()
            .any(|f| f.message.contains("m6-reasoning")),
        "m6-reasoning must have zero panel_frozen warnings or errors: {frozen_findings:?}"
    );
    assert_eq!(
        uor_r4_harness::panels::noise_band(&panel.noise),
        Some(3.0),
        "noise band must parse as +-3 rows"
    );
    assert_eq!(panel.bars.len(), 4);
    assert_eq!(panel.bars[0].id, "overall-pass-240");
    assert_eq!(panel.bars[1].id, "compositions-80");
    assert_eq!(panel.bars[2].id, "altered-constraints-80");
    assert_eq!(panel.bars[3].id, "code-tasks-80");

    // Movement within +-3 of current (20 -> 22) is AT NOISE; 168 is PASS; 100 is FAIL.
    let at_noise = uor_r4_harness::panels::classify(panel, &panel.bars[0], 22.0);
    assert_eq!(at_noise.outcome, "AT NOISE");
    let fail = uor_r4_harness::panels::classify(panel, &panel.bars[0], 100.0);
    assert_eq!(fail.outcome, "FAIL");
    let pass = uor_r4_harness::panels::classify(panel, &panel.bars[0], 168.0);
    assert_eq!(pass.outcome, "PASS");
}
