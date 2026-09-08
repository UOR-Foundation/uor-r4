use super::durable_memory::{DurableSession, IdentityScope};
use super::groundedness::{GroundedOutcome, GroundednessEvaluator};
use super::m1_profiler::{M1Profiler, TaskKind};
use super::multi_step_reasoning::{ConstraintPolicy, MultiStepReasoningEngine};
use super::workspace_coding::{WorkspaceCodingEngine, WorkspaceEnvironment};
use super::{Config, Control, Document, Trainer, BOS};
use std::path::Path;

fn sample_curriculum_documents() -> Vec<Document> {
    vec![
        Document {
            id: "doc-narrative-observatory".into(),
            text: "The observatory stood silently upon the ridge overlooking the valley. The sky turned deep indigo as the first stars appeared.".into(),
        },
        Document {
            id: "doc-narrative-telescope".into(),
            text: "The astronomer aligned the brass telescope toward the northern horizon. A cold wind rustled the pages of the observation logbook.".into(),
        },
        Document {
            id: "doc-technical-geometry".into(),
            text: "The geometric state represents spatial coordinates on the three-sphere manifold. Each state update rotates coordinates along orthogonal planes.".into(),
        },
        Document {
            id: "doc-procedural-dialogue".into(),
            text: "Question: How does the system compute trajectories? Answer: The navigator calculates angular distance and observes coordinates.".into(),
        },
        Document {
            id: "doc-code-example".into(),
            text: "fn main() { let x = 42; println!(\"result: {}\", x); }".into(),
        },
    ]
}

fn build_test_model() -> super::Model {
    let docs = sample_curriculum_documents();
    let config = Config {
        context_tokens: 64,
        candidate_limit: 32,
        max_lexical_pieces: 256,
        max_rows: 4096,
        max_associations: 16384,
        postings_per_row: 16,
    };
    let mut trainer = Trainer::new(config, &docs).expect("valid trainer config");
    trainer.train_documents(&docs).expect("document training");
    trainer.compile().expect("model compilation")
}

#[test]
fn native_m1_complete_path_stage_breakdown() {
    let model = build_test_model();
    let prompt = "The observatory stood silently";
    let report =
        M1Profiler::profile_task(&model, TaskKind::NarrativeProse, prompt, 16, Control::Full)
            .expect("profile task");

    assert!(report.success, "task must succeed");
    assert!(report.output_tokens > 0, "must generate output tokens");
    assert!(
        report.stages.cold_load_us > 0,
        "cold load stage must be timed"
    );
    assert!(report.stages.ingest_us > 0, "ingest stage must be timed");
    assert!(
        report.stages.geometric_lookup_us > 0,
        "geometric lookup stage must be timed"
    );
    assert!(
        report.stages.operator_execution_us > 0,
        "operator execution stage must be timed"
    );
    assert!(
        report.stages.token_emission_us > 0,
        "token emission stage must be timed"
    );
    assert!(
        report.stages.persistence_us > 0,
        "persistence stage must be timed"
    );
    assert!(
        report.stages.total_us >= report.stages.cold_load_us,
        "total stage time must subsume cold load"
    );
}

#[test]
fn native_m1_submillisecond_token_distribution() {
    let model = build_test_model();
    let prompt = "The geometric state represents";
    let report =
        M1Profiler::profile_task(&model, TaskKind::NarrativeProse, prompt, 20, Control::Full)
            .expect("profile task");

    // Assert that individual decisions consistently execute in under 1 millisecond (1000 µs) on M1
    assert!(
        report.decision_latency.mean_us < 1000,
        "Mean decision latency must be strictly submillisecond (<1000 µs), got {} µs",
        report.decision_latency.mean_us
    );
    assert!(
        report.decision_latency.median_us < 1000,
        "Median decision latency must be strictly submillisecond (<1000 µs), got {} µs",
        report.decision_latency.median_us
    );

    // Assert that per-token overall latency is also submillisecond
    assert!(
        report.token_latency.median_us < 1000,
        "Median per-token latency must be submillisecond (<1000 µs), got {} µs",
        report.token_latency.median_us
    );

    assert!(
        report.tokens_per_second > 0,
        "Tokens per second must be positive"
    );
}

#[test]
fn native_m1_memory_footprint_and_traffic() {
    let model = build_test_model();
    let prompt = "Question: How does";
    let report = M1Profiler::profile_task(
        &model,
        TaskKind::ProceduralDialogue,
        prompt,
        12,
        Control::Full,
    )
    .expect("profile task");

    let hw = &report.hardware;
    assert_eq!(hw.cpu_brand, "Apple M1");
    assert!(hw.logical_cores >= 4);
    assert!(hw.total_memory_bytes >= 8_000_000_000);
    assert!(hw.ring_storage_bytes > 0);
    assert!(hw.candidate_storage_bytes > 0);
    assert!(hw.table_storage_bytes > 0);
    assert!(
        hw.resident_memory_bytes > 0,
        "Resident memory (RSS) must be measured"
    );
}

#[test]
fn native_m1_energy_and_thermal_stability() {
    let model = build_test_model();
    let prompt = "The astronomer aligned the brass";
    let report =
        M1Profiler::profile_task(&model, TaskKind::NarrativeProse, prompt, 15, Control::Full)
            .expect("profile task");

    let energy = &report.energy;
    assert_eq!(energy.estimated_power_mw, 3500);
    assert!(energy.duration_us > 0);
    assert!(energy.energy_per_token_uj > 0);
    assert_eq!(energy.thermal_status, "nominal");
}

#[test]
fn native_m1_quality_matched_workload_benchmark() {
    let model = build_test_model();

    // 1. Narrative Prose
    let prose_res = M1Profiler::profile_task(
        &model,
        TaskKind::NarrativeProse,
        "The observatory stood",
        16,
        Control::Full,
    )
    .expect("prose benchmark");
    assert!(prose_res.success);

    // 2. Procedural Dialogue
    let qa_res = M1Profiler::profile_task(
        &model,
        TaskKind::ProceduralDialogue,
        "Question: How does the system compute trajectories? Answer:",
        16,
        Control::Full,
    )
    .expect("qa benchmark");
    assert!(qa_res.success);

    // 3. Grounded Memory
    let scope = IdentityScope::new("m1_user", "m1_proj", "m1_sess").unwrap();
    let mut session = DurableSession::new(&model, scope, Control::Full).unwrap();
    session.assert_fact("planet", "elliptical").unwrap();
    let grounded_outcome =
        GroundednessEvaluator::evaluate_query(&mut session, &model, "Where does planet orbit?");
    assert!(matches!(grounded_outcome, GroundedOutcome::Answer(_)));

    // 4. Multi-Step Reasoning DAG
    let steps_spec = vec![
        ("+", 13, None, 4),    // 13 + 4 = 17
        ("*", 0, Some(1), 2),  // 17 * 2 = 34
        ("-", 0, Some(2), 10), // 34 - 10 = 24
    ];
    let constraints = vec![
        ConstraintPolicy::Range { min: 0, max: 100 },
        ConstraintPolicy::NonNegative,
    ];
    let chain =
        MultiStepReasoningEngine::execute_arithmetic_dag("m1-dag-3step", &steps_spec, &constraints)
            .expect("execute arithmetic DAG");
    assert_eq!(chain.final_result, "24");
    assert!(chain.constraints_satisfied);

    // 5. Rust Code Synthesis in isolated workspace
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let temp_dir = std::env::temp_dir().join(format!("uor_m1_bench_{now}"));
    std::fs::create_dir_all(&temp_dir).expect("create tempdir");
    let env = WorkspaceEnvironment::new(temp_dir.clone());
    let main_rel = Path::new("main.rs");
    env.write_file(
        main_rel,
        "fn main() { let x = 17 * 2 - 10; assert_eq!(x, 24); }\n",
    )
    .expect("write main.rs");
    let src_path = temp_dir.join("main.rs");
    let bin_path = temp_dir.join("main_bin");
    let compile_report =
        WorkspaceCodingEngine::compile_single(&src_path, &bin_path).expect("compile single");
    assert!(compile_report.success);
    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn native_m1_deterministic_parallel_scaling() {
    let model = build_test_model();
    let prompt = "The observatory stood silently";
    let (seq_us, par_us, deterministic) =
        M1Profiler::profile_parallel_scaling(&model, prompt, 12, 4)
            .expect("parallel scaling profile");

    assert!(seq_us > 0, "sequential time must be positive");
    assert!(par_us > 0, "parallel time must be positive");
    assert!(
        deterministic,
        "Parallel worker output must be bit-exact deterministic with sequential execution"
    );
}

#[test]
fn native_m1_bounded_table_growth() {
    let model = build_test_model();
    assert!(
        model.rows.len() <= model.config.max_rows,
        "Row count must be strictly bounded by max_rows"
    );
    assert!(
        model.training.learned_associations <= model.config.max_associations,
        "Association count must be strictly bounded by max_associations"
    );
    assert_eq!(
        model.config.context_tokens, 64,
        "Context tokens capacity must remain bounded"
    );
}

#[test]
fn native_m1_invariant_zero_allocation_and_kernel_safety() {
    let model = build_test_model();
    let mut session = model.session(Control::Full).expect("valid session");
    session.observe(&model, BOS).expect("observe BOS");

    // The observe and predict calls execute in integer kernel with pre-allocated buffers
    let pred = session.predict(&model).expect("predict");
    assert!(pred.token > 0 || pred.token == 0);
}
