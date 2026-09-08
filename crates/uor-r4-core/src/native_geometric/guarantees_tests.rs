use super::guarantees::*;
use super::SCHEMA;

#[test]
fn test_serving_operation_census_allowed_ops() {
    let mut census = ServingOperationCensus::new();

    // Record allowed hot-path operations
    census.record_op(AllowedOp::BitwiseXor);
    census.record_op(AllowedOp::BitwiseAnd);
    census.record_op(AllowedOp::ShiftLeft);
    census.record_op(AllowedOp::Popcount);
    census.record_op(AllowedOp::CheckedAdd);
    census.record_op(AllowedOp::TableLookup);
    census.record_op(AllowedOp::ShiftAddProduct);

    // Verify decomposed multiplication without hardware multiply
    let prod = ServingOperationCensus::shift_add_product(13, 17);
    assert_eq!(prod, 221);

    let report = census.audit().expect("Census audit should succeed");
    assert_eq!(report.total_allowed_ops, 7);
    assert!(report.is_kernel_pure);
    assert!(report.no_std_compliant);
    assert!(report.zero_float_verified);
    assert!(report.zero_matrix_product_verified);
    assert!(report.forbidden_detected.is_empty());
}

#[test]
fn test_serving_operation_census_rejects_forbidden_ops() {
    let mut census = ServingOperationCensus::new();
    census.record_op(AllowedOp::TableLookup);
    census.record_forbidden(ForbiddenOp::FloatingPointOp);

    let err = census.audit().unwrap_err();
    assert!(err.0.contains("detected forbidden operations"));

    let mut census2 = ServingOperationCensus::new();
    census2.record_forbidden(ForbiddenOp::DirectHardwareMultiply);
    let err2 = census2.audit().unwrap_err();
    assert!(err2.0.contains("detected forbidden operations"));
}

#[test]
fn test_zphi_ring_exact_arithmetic() {
    let x = ZPhi::new(1, 1); // 1 + phi
    let y = ZPhi::new(2, -1); // 2 - phi

    // Addition: (1 + phi) + (2 - phi) = 3 + 0*phi
    let sum = x.add(&y);
    assert_eq!(sum, ZPhi::new(3, 0));

    // Subtraction: (1 + phi) - (2 - phi) = -1 + 2*phi
    let diff = x.sub(&y);
    assert_eq!(diff, ZPhi::new(-1, 2));

    // Multiplication: (1 + phi)^2 = 1 + 2*phi + phi^2 = 1 + 2*phi + (phi + 1) = 2 + 3*phi
    let x_sq = x.mul(&x);
    assert_eq!(x_sq, ZPhi::new(2, 3));

    // Algebraic norm: N(a + b*phi) = a^2 + ab - b^2
    // N(1 + phi) = 1 + 1 - 1 = 1 (unit)
    assert_eq!(x.norm(), 1);
    // N(2 + 3*phi) = 4 + 6 - 9 = 1 (unit)
    assert_eq!(x_sq.norm(), 1);
    // N(2 + phi) = 4 + 2 - 1 = 5
    assert_eq!(ZPhi::new(2, 1).norm(), 5);

    // Associativity: (x * y) * z == x * (y * z)
    let z = ZPhi::new(3, 2);
    let lhs = x.mul(&y).mul(&z);
    let rhs = x.mul(&y.mul(&z));
    assert_eq!(lhs, rhs);

    // Distributivity: x * (y + z) == x * y + x * z
    let dist_lhs = x.mul(&y.add(&z));
    let dist_rhs = x.mul(&y).add(&x.mul(&z));
    assert_eq!(dist_lhs, dist_rhs);
}

#[test]
fn test_zphi_fibonacci_recurrence_and_inverse() {
    // Start with phi: (0, 1)
    let mut step = ZPhi::PHI;
    let expected_fib = [
        (1, 1), // F_2, F_3
        (1, 2), // F_3, F_4
        (2, 3), // F_4, F_5
        (3, 5), // F_5, F_6
        (5, 8), // F_6, F_7
    ];

    for exp in expected_fib {
        step = step.fibonacci_step();
        assert_eq!(step, ZPhi::new(exp.0, exp.1));
    }

    // Exact inverse Fibonacci stepping
    for exp in expected_fib.iter().rev().skip(1) {
        step = step.fibonacci_step_inv();
        assert_eq!(step, ZPhi::new(exp.0, exp.1));
    }
    step = step.fibonacci_step_inv();
    assert_eq!(step, ZPhi::PHI);

    // Inversion identity: f_inv(f(x)) == x
    let arbitrary = ZPhi::new(17, -42);
    assert_eq!(arbitrary.fibonacci_step().fibonacci_step_inv(), arbitrary);
}

#[test]
fn test_icosian_quaternion_inverse_witness() {
    // Identity quaternion
    let q_id = IcosianQuaternion::IDENTITY;
    assert!(q_id.is_unit());
    assert!(q_id.verify_inverse_witness().is_ok());

    // Basic basis quaternions i, j, k
    let q_i = IcosianQuaternion::new(ZPhi::ZERO, ZPhi::ONE, ZPhi::ZERO, ZPhi::ZERO);
    let q_j = IcosianQuaternion::new(ZPhi::ZERO, ZPhi::ZERO, ZPhi::ONE, ZPhi::ZERO);
    let q_k = IcosianQuaternion::new(ZPhi::ZERO, ZPhi::ZERO, ZPhi::ZERO, ZPhi::ONE);

    assert!(q_i.is_unit());
    assert!(q_i.verify_inverse_witness().is_ok());
    assert!(q_j.is_unit());
    assert!(q_j.verify_inverse_witness().is_ok());
    assert!(q_k.is_unit());
    assert!(q_k.verify_inverse_witness().is_ok());

    // Check Hamiltonian quaternion multiplication: i * j = k
    let ij = q_i.mul(&q_j);
    assert_eq!(ij, q_k);

    // Check i^2 = -1
    let ii = q_i.mul(&q_i);
    assert_eq!(
        ii,
        IcosianQuaternion::new(ZPhi::new(-1, 0), ZPhi::ZERO, ZPhi::ZERO, ZPhi::ZERO)
    );

    // Non-unit quaternion must fail inverse witness
    let non_unit = IcosianQuaternion::new(ZPhi::new(2, 0), ZPhi::ZERO, ZPhi::ZERO, ZPhi::ZERO);
    assert!(!non_unit.is_unit());
    assert!(non_unit.verify_inverse_witness().is_err());

    // Paired-H4 Icosian representation (E8 = H4 x H4)
    let paired = PairedH4Icosian::new(q_i, q_j);
    assert!(paired.verify_paired_witnesses().is_ok());
}

#[test]
fn test_euler_hopf_bridge_and_chart_adapters() {
    let mut bridge_null = EulerHopfBridge::new(BridgeBoundary::ContinuousNull);
    assert_eq!(bridge_null.boundary_value(), 0);

    let bridge_disc = EulerHopfBridge::new(BridgeBoundary::DiscreteEmptyProduct);
    assert_eq!(bridge_disc.boundary_value(), 1);

    // Orientation updates without float trig
    bridge_null.update_orientation(5, 12);
    assert_eq!(bridge_null.chirality, 1);
    assert_eq!(bridge_null.polarity, 1);

    bridge_null.update_orientation(-8, 15);
    assert_eq!(bridge_null.chirality, -1);
    assert_eq!(bridge_null.polarity, 1);

    // Quarter-turn phase shift: 0 + 64 = 64
    let shifted = bridge_null.quarter_turn_phase_shift(0);
    assert_eq!(shifted, 64);
    assert_eq!(bridge_null.quarter_turn_shifts, 1);

    // Chart adapters
    let witness_euc = ChartAdapter::select_least_cost(false);
    assert_eq!(witness_euc.chart, ChartKind::EuclideanSqrt2);
    assert_eq!(witness_euc.error_bound_ppm, 0);
    assert!(witness_euc.preserves_orientation);
    assert!(witness_euc.has_inverse_witness);

    let witness_riem = ChartAdapter::select_least_cost(true);
    assert_eq!(witness_riem.chart, ChartKind::RiemannianInterval);
    assert_eq!(witness_riem.error_bound_ppm, 0);
}

#[test]
fn test_artifact_integrity_witness_and_codec_separation() {
    let cid = "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi";
    let cfg_hash = "config_hash_abc123";
    let witness = ArtifactIntegrityWitness::create(cid, cfg_hash);

    assert_eq!(witness.schema_version, SCHEMA);
    assert!(witness.verify(cid).is_ok());

    // Mismatched CID
    assert!(witness.verify("wrong_cid").is_err());

    // Codec role distinction
    let c_lex = CodecRole::LexicalCodec;
    let kappa = CodecRole::ContentIdentity;
    assert_ne!(c_lex, kappa);
}

#[test]
fn test_formal_claim_dossier_vocabulary_and_statuses() {
    let dossier = FormalClaimDossier::new();
    let report = dossier
        .verify_dossier()
        .expect("Dossier verification should pass");

    assert_eq!(report.total_claims, 9);
    assert_eq!(report.guarantee_count, 6);
    assert_eq!(report.witnessed_count, 2);
    assert_eq!(report.structural_count, 5);
    assert_eq!(report.empirical_count, 1);
    assert_eq!(report.assumed_count, 1);
    assert!(report.is_valid);
    assert!(report.prohibited_phrases_found.is_empty());
}

#[test]
fn test_prohibited_wording_scanner() {
    let mut dossier = FormalClaimDossier::new();
    // Register a claim that uses prohibited wording without disavowal
    dossier.register(FormalClaim {
        id: "INVALID-CLAIM-01".into(),
        title: "Exact Teacher Equivalence".into(),
        class: ClaimClass::Guarantee,
        status: ClaimStatus::Structural,
        vocabulary_section: "§2.1".into(),
        code_binding: "test".into(),
        disavowal_note: "Claiming exact teacher equivalence here.".into(),
    });

    let err = dossier.verify_dossier().unwrap_err();
    assert!(err.0.contains("prohibited wording"));
}
