use super::m1_profiler::{M1EnergyModel, M1Profiler, TaskKind};
use super::{Config, Control, Document, Trainer};

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
fn native_profiler_reports_only_measured_scope() {
    let model = build_test_model();
    let report = M1Profiler::profile_task(
        &model,
        TaskKind::NarrativeProse,
        "The observatory stood silently",
        4,
        Control::Full,
    )
    .expect("diagnostic execution");
    assert!(report.stages.geometric_lookup_us.is_none());
    assert!(report.stages.operator_execution_us.is_none());
    assert!(!report.quality_verified);
    assert!(report.timing_scope.contains("excludes"));
    assert!(report.hardware.cpu_brand.is_none());
    assert!(report.hardware.physical_cores.is_none());
    assert!(report.hardware.total_memory_bytes.is_none());
    assert!(report.hardware.resident_memory_bytes.is_none());
    assert!(report.energy.estimated_power_mw.is_none());
    assert!(report.energy.thermal_status.is_none());
    assert_eq!(
        report.stages.total_us,
        report.stages.cold_load_us
            + report.stages.ingest_us
            + report.stages.prediction_us
            + report.stages.token_emission_us
            + report.stages.persistence_us
    );
}

#[test]
fn native_profiler_does_not_invent_power_or_thermals() {
    let report = M1EnergyModel::estimate(30 * 60 * 1_000_000, 10);
    assert_eq!(report.duration_us, 1_800_000_000);
    assert!(report.estimated_power_mw.is_none());
    assert!(report.energy_per_token_uj.is_none());
    assert!(report.total_energy_mj.is_none());
    assert!(report.thermal_status.is_none());
}
