use uor_r4_core::transformerless::{
    performance_certificate::{PerformanceCertificate, PerformanceProfiler},
    runtime::OpKernel,
};

#[test]
fn test_performance_profiler_and_threshold_verification() {
    let ops = OpKernel {
        adds: 100,
        compares: 64,
        candidate_scans: 32,
        ..Default::default()
    };

    let cert = PerformanceProfiler::profile(2048, 4, ops, 4096);

    assert_eq!(cert.metrics.bytes_read, 2048);
    assert_eq!(cert.metrics.cache_miss_estimate, (2048 / 64) + 4);
    assert_eq!(cert.metrics.branch_miss_estimate, (64 / 32) + (32 / 16));
    assert!(cert.baseline_threshold_passed);
    assert!(cert.verify_cid());
}

#[test]
fn test_performance_certificate_threshold_failure() {
    let ops = OpKernel::default();
    let cert = PerformanceProfiler::profile(8192, 10, ops, 4096); // 8192 > 4096 limit

    assert!(!cert.baseline_threshold_passed);
    assert!(cert.verify_cid());
}

#[test]
fn test_performance_certificate_cbor_roundtrip() {
    let ops = OpKernel::default();
    let cert = PerformanceProfiler::profile(1024, 2, ops, 2048);

    let cbor_bytes = cert.to_cbor_bytes().expect("serialize CBOR");
    let decoded = PerformanceCertificate::from_cbor_bytes(&cbor_bytes).expect("deserialize CBOR");

    assert_eq!(cert, decoded);
    assert!(decoded.verify_cid());
}
