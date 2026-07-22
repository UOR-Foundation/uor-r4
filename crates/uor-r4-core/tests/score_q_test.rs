use uor_r4_core::transformerless::score_q::{ScoreQ, StorageDescriptor};

#[test]
fn test_score_q_conversions_and_precision() {
    let logprobs = [-0.01f32, -0.5f32, -2.3f32, -10.0f32, 0.0f32, 1.25f32];
    for &lp in &logprobs {
        let q = ScoreQ::from_logprob(lp);
        let back = q.to_logprob();
        let diff = (lp - back).abs();
        assert!(
            diff < 1.5e-4,
            "lp {} -> ScoreQ {:?} -> back {} (diff {})",
            lp,
            q,
            back,
            diff
        );
    }
}

#[test]
fn test_score_q_saturating_arithmetic() {
    let a = ScoreQ::from_logprob(-1.5f32);
    let b = ScoreQ::from_logprob(-0.5f32);
    let c = a + b;
    let expected = -2.0;
    assert!((c.to_logprob() - expected).abs() < 1e-4);

    let max = ScoreQ::MAX;
    let overflow = max + ScoreQ::from_raw(100);
    assert_eq!(overflow, ScoreQ::MAX, "saturating add caps at MAX");

    let min = ScoreQ::MIN;
    let underflow = min - ScoreQ::from_raw(100);
    assert_eq!(underflow, ScoreQ::MIN, "saturating sub caps at MIN");
}

#[test]
fn test_storage_descriptor_decode() {
    let desc = StorageDescriptor::new(16, 8, 0);
    // raw = 256 -> centered = 256 -> shifted left 8 bits = 65536 = ScoreQ(65536) = 1.0 logprob
    let q = desc.decode(256);
    assert_eq!(q.raw(), 65536);
    assert!((q.to_logprob() - 1.0).abs() < 1e-4);
}
