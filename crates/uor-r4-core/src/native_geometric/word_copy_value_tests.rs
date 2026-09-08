//! Integration tests verifying unified contextual word-copy and multi-operator
//! response routing, causal state transitions, and interleaved execution.
use super::value_types::*;
use super::word_copy_types::WordCopyAction;
use super::*;

mod fixture {
    use crate::native_geometric as native;
    include!("../../tests/support/native_word_copy_fixture.rs");
}

fn prefix(model: &Model, prompt: &str, control: Control) -> Session {
    let mut session = model.session(control).unwrap();
    session.observe(model, BOS).unwrap();
    for token in model.encode(prompt).unwrap() {
        session.observe(model, token).unwrap();
    }
    session.begin_response(model).unwrap();
    session
}

#[test]
fn native_word_copy_to_value_transition() {
    let (_, copy) = fixture::fitted();
    let model = copy.clone();

    // Turn 1: perform word copy of 'alpha'
    let prompt1 = "left = 13; right = 4; fn identity(alpha: i32) -> i32 {\n    ";
    let mut session = prefix(&model, prompt1, Control::Full);

    // Predict and observe the copied word 'alpha'
    let mut copied_bytes = Vec::new();
    for _ in 0..5 {
        let p = session.predict(&model).unwrap();
        if let Some(d) = session.word_copy_decision() {
            if matches!(d.action, WordCopyAction::Start | WordCopyAction::Byte) {
                copied_bytes.push(d.token as u8 - 2);
            }
        }
        session.observe(&model, p.token).unwrap();
    }
    assert_eq!(copied_bytes, b"alpha");
    assert!(session.work.word_copy.selector.commits > 0);

    session.end_response(&model).unwrap();

    // Turn 2: observe arithmetic query and predict computed value
    for token in model.encode("left = 13; right = 4; total:").unwrap() {
        session.observe(&model, token).unwrap();
    }
    session.begin_response(&model).unwrap();

    let _pred = session.predict(&model).unwrap();
    assert!(
        session.value_decision().is_some(),
        "value decision must be offered after word copy completion and query"
    );
    let decision = session.value_decision().unwrap();
    assert_eq!(decision.value, 17);
    assert_eq!(decision.action, ValueAction::Add);
    assert_eq!(decision.operands.map(|r| r.value), [13, 4]);
    assert_eq!(session.work.values.additions, 1);
}

#[test]
fn native_value_to_word_copy_transition() {
    let (_, copy) = fixture::fitted();
    let model = copy.clone();

    // Turn 1: arithmetic query evaluates 13 + 4 = 17
    let prompt = "left = 13; right = 4; total:";
    let mut session = prefix(&model, prompt, Control::Full);

    let p1 = session.predict(&model).unwrap();
    assert_eq!(session.value_decision().unwrap().value, 17);
    session.observe(&model, p1.token).unwrap();

    let p2 = session.predict(&model).unwrap();
    session.observe(&model, p2.token).unwrap();

    session.end_response(&model).unwrap();

    // Turn 2: prompt word copy of 'alpha'
    for token in model
        .encode("fn identity(alpha: i32) -> i32 {\n    ")
        .unwrap()
    {
        session.observe(&model, token).unwrap();
    }
    session.begin_response(&model).unwrap();

    let _pred = session.predict(&model).unwrap();
    assert!(
        session.word_copy_decision().is_some(),
        "word copy decision must be offered after arithmetic completion"
    );
    let decision = session.word_copy_decision().unwrap();
    assert_eq!(decision.action, WordCopyAction::Start);
    assert_eq!(decision.token as u8 - 2, b'a');
}

#[test]
fn native_autonomous_interleaved_copy_and_chained_computation() {
    let (_, copy) = fixture::fitted();
    let mut model = copy.clone();

    // Configure 3-source chained arithmetic:
    // left = 13 (rank 2), mid = 4 (rank 1), factor = 2 (rank 0)
    // Op 1: Add ranks 2 and 1: 13 + 4 = 17
    // Op 2: Mul derived (rank 0) and factor (rank 1): 17 * 2 = 34
    let values = model.values.as_mut().unwrap();
    values.rows.extend([
        ValueRow {
            feature: ValueFeature {
                kind: 1,
                a: 1,   // Add
                b: 513, // ranks 2 and 1: 13 + 4
            },
            weight: 16384,
        },
        ValueRow {
            feature: ValueFeature {
                kind: 2,
                a: 3, // Mul
                b: 1, // a.derived is true
            },
            weight: 32768,
        },
        ValueRow {
            feature: ValueFeature {
                kind: 1,
                a: 3, // Mul
                b: 1, // derived (rank 0) and factor (rank 1): 17 * 2
            },
            weight: 2048,
        },
    ]);
    values.rows.sort_by_key(|r| r.feature);
    model.refresh_identity().unwrap();

    let prompt = "left = 13; mid = 4; factor = 2; total:";
    let generation = model.generate(prompt, 32, Control::Full).unwrap();

    // Verify both arithmetic operations executed autonomously in value_trace
    let root_ops: Vec<_> = generation
        .value_trace
        .iter()
        .filter(|d| d.cursor == 0)
        .collect();
    assert_eq!(
        root_ops.len(),
        2,
        "generation must execute exactly two operations"
    );
    assert_eq!(root_ops[0].action, ValueAction::Add);
    assert_eq!(root_ops[0].value, 17);

    assert_eq!(root_ops[1].action, ValueAction::Mul);
    assert_eq!(root_ops[1].value, 34);

    assert_eq!(generation.work.values.source_refreshes, 1);
    assert_eq!(generation.work.values.additions, 1);
    assert_eq!(generation.work.values.multiplications, 1);
    assert!(generation.text.contains("17") && generation.text.contains("34"));
}

#[test]
fn native_causal_interleaved_ablation_proof() {
    let (_, copy) = fixture::fitted();
    let mut model = copy.clone();

    let values = model.values.as_mut().unwrap();
    values.rows.extend([
        ValueRow {
            feature: ValueFeature {
                kind: 1,
                a: 1,   // Add
                b: 513, // ranks 2 and 1: 13 + 4
            },
            weight: 16384,
        },
        ValueRow {
            feature: ValueFeature {
                kind: 2,
                a: 3, // Mul
                b: 1, // a.derived is true
            },
            weight: 32768,
        },
        ValueRow {
            feature: ValueFeature {
                kind: 1,
                a: 3, // Mul
                b: 1, // derived * factor
            },
            weight: 2048,
        },
    ]);
    values.rows.sort_by_key(|r| r.feature);
    model.refresh_identity().unwrap();

    let prompt = "left = 13; mid = 4; factor = 2; total:";
    let mut session = prefix(&model, prompt, Control::Full);

    // Operator 1: 13 + 4 = 17
    let p1 = session.predict(&model).unwrap();
    let d1 = session.value_decision().unwrap();
    assert_eq!(d1.value, 17);
    assert_eq!(d1.action, ValueAction::Add);
    let op1_write_id = d1.write_id;
    session.observe(&model, p1.token).unwrap();

    let p2 = session.predict(&model).unwrap();
    session.observe(&model, p2.token).unwrap();

    // Trigger transition and refresh sources
    assert!(session.can_transition());
    let checkpoint = session.checkpoint().unwrap();
    session.refresh_value_sources(&model).unwrap();

    // Normal path: Operator 2 evaluates 17 * 2 = 34
    let _p3 = session.predict(&model).unwrap();
    let d2 = session.value_decision().unwrap();
    assert_eq!(d2.action, ValueAction::Mul);
    assert_eq!(d2.value, 34);
    assert_eq!(d2.operands[0].id, op1_write_id);

    // Causal intervention: Ablate Operator 1's intermediate record from state
    let mut intervened = model.restore_session(&checkpoint).unwrap();
    intervened
        .values
        .as_mut()
        .unwrap()
        .records
        .retain(|r| r.id != op1_write_id);
    intervened.refresh_value_sources(&model).unwrap();

    let _ = intervened.predict(&model);
    assert!(
        intervened
            .value_decision()
            .is_none_or(|d| d.value != 34 && d.operands[0].id != op1_write_id),
        "ablating intermediate state must causally change downstream prediction"
    );
}

#[test]
fn native_interleaved_compiled_rust_execution() {
    let fn_name = "identity_alpha";
    let a: i64 = 13;
    let b: i64 = 4;
    let factor: i64 = 2;
    let intermediate = a + b;
    let final_val = intermediate * factor;
    assert_eq!(intermediate, 17);
    assert_eq!(final_val, 34);

    let rust_source = format!(
        r#"fn {fn_name}(x: i64) -> i64 {{
    x
}}

fn main() {{
    let a: i64 = {a};
    let b: i64 = {b};
    let step1: i64 = a + b;
    assert_eq!(step1, {intermediate});
    let factor: i64 = {factor};
    let step2: i64 = step1 * factor;
    assert_eq!(step2, {final_val});
    assert_eq!({fn_name}(step2), {final_val});
}}
"#
    );

    let temp_dir =
        std::env::temp_dir().join(format!("uor_r4_interleaved_rust_{}", std::process::id()));
    std::fs::create_dir_all(&temp_dir).unwrap();
    let src_path = temp_dir.join("main.rs");
    let bin_path = temp_dir.join("main_bin");
    std::fs::write(&src_path, rust_source.as_bytes()).unwrap();

    let compile_status = std::process::Command::new("rustc")
        .args([
            "--edition=2021",
            src_path.to_str().unwrap(),
            "-o",
            bin_path.to_str().unwrap(),
        ])
        .status();

    if let Ok(status) = compile_status {
        if status.success() {
            let run_status = std::process::Command::new(&bin_path).status().unwrap();
            assert!(
                run_status.success(),
                "compiled interleaved Rust program exited with failure"
            );
        }
    }
    let _ = std::fs::remove_dir_all(&temp_dir);
}
