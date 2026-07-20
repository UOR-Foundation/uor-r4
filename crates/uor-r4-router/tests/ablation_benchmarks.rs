use uor_r4_router::UorR4Router;
use uor_r4_router::geometry::TypedObject;
use uor_r4_router::benchmark::run_ablation_benchmark;

#[test]
fn test_ablation_benchmarks() {
    let router = UorR4Router::new(0.5);

    // Queries: (Object, Target Ground Truth ID)
    let queries = vec![
        (
            TypedObject {
                object_type: "document".to_string(),
                content: "borehole depth gambling".to_string(),
            },
            1, // GT target axis/id
        ),
        (
            TypedObject {
                object_type: "document".to_string(),
                content: "temporal aquifer dry season".to_string(),
            },
            2, // GT target axis/id
        ),
    ];

    let results = run_ablation_benchmark(&router, &queries);

    assert_eq!(results.len(), 2);

    // Verify Spectral results
    let spectral = &results[0];
    assert_eq!(spectral.geometry_name, "Spectral Heuristic");
    assert!(spectral.recall_at_3 >= 0.0);
    assert!(spectral.hits_at_3 >= 0.0);
    assert_eq!(spectral.migration_agreement, 0.98);

    // Verify VSA results
    let vsa = &results[1];
    assert_eq!(vsa.geometry_name, "VSA Grounded");
    assert!(vsa.recall_at_3 >= 0.0);
    assert!(vsa.hits_at_3 >= 0.0);
    assert!(vsa.unlearning_time_ns > 0);
}
