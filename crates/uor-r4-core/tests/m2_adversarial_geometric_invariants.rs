//! Adversarial Geometric Invariants & Stress Harness for Milestone M2
//!
//! Author: teamwork_preview_challenger_m2_2
//! Role: Empirical Challenger & Specialist
//!
//! Stress-tests:
//! 1. Hopf fiber invertibility roundtrip S3 -> (S2, psi) -> S3 vs fiber loss sign collapse (q vs -q).
//! 2. Paired-H4 icosian exact ring arithmetic in Z[phi] vs Fibonacci companion matrix [[0, 1], [1, 1]] across large exponents.
//! 3. Prime route prefix collapse when exact sentence key I_S misses.

use std::num::{NonZeroU16, NonZeroUsize};

use uor_r4_core::native_geometric::hopf_metric::UnitS3;
use uor_r4_core::prime_route_attention::{
    compile_spin_manifest, GeometricAddress, ManifestProvenance, PhaseQ29, PrimeRegistry,
    PrimeRouteError, RouteSentence, SemanticAtom, SpinTorsionState, UnitS3Q30 as CoreUnitS3Q30,
    ZPhi, ZeroPowerBridge,
};
use uor_r4_core::prime_route_geometric_attention::{
    AttentionControl, AttentionRowKey, AttentionRowSource, CausalAttentionState,
    GeometricAttentionArtifact,
};

// ============================================================================
// 1. ADVERSARIAL CHALLENGE: HOPF FIBER INVERTIBILITY & ANTIPODAL SIGN COLLAPSE
// ============================================================================

#[test]
fn test_adversarial_hopf_fiber_exact_roundtrip_continuous() {
    // Test all orthant corners, axes, poles, and arbitrary pseudo-random unit quaternions
    let mut test_quats = vec![
        UnitS3::IDENTITY,
        UnitS3::I,
        UnitS3::J,
        UnitS3::K,
        // Antipodal points of basis
        UnitS3::new(-1.0, 0.0, 0.0, 0.0).unwrap(),
        UnitS3::new(0.0, -1.0, 0.0, 0.0).unwrap(),
        UnitS3::new(0.0, 0.0, -1.0, 0.0).unwrap(),
        UnitS3::new(0.0, 0.0, 0.0, -1.0).unwrap(),
        // South pole of S2: z1 = 0, z2 = c + di => z = -1
        UnitS3::new(0.0, 0.0, 1.0, 0.0).unwrap(),
        UnitS3::new(0.0, 0.0, -1.0, 0.0).unwrap(),
        UnitS3::new(0.0, 0.0, 0.0, 1.0).unwrap(),
        UnitS3::new(0.0, 0.0, 0.0, -1.0).unwrap(),
        UnitS3::new(0.0, 0.0, 1.0 / 2.0f64.sqrt(), 1.0 / 2.0f64.sqrt()).unwrap(),
        // Equator points of S2: |z1|^2 = |z2|^2 = 1/2 => z = 0
        UnitS3::new(0.5, 0.5, 0.5, 0.5).unwrap(),
        UnitS3::new(-0.5, -0.5, -0.5, -0.5).unwrap(),
        UnitS3::new(0.5, -0.5, 0.5, -0.5).unwrap(),
        UnitS3::new(-0.5, 0.5, -0.5, 0.5).unwrap(),
    ];

    // Generate 64 arbitrary unit quaternions across all 16 sign orthants
    for a_sgn in [-1.0, 1.0] {
        for b_sgn in [-1.0, 1.0] {
            for c_sgn in [-1.0, 1.0] {
                for d_sgn in [-1.0, 1.0] {
                    let a = a_sgn * 0.316227766;
                    let b = b_sgn * 0.447213595;
                    let c = c_sgn * 0.632455532;
                    let d = d_sgn * 0.547722557;
                    if let Ok(q) = UnitS3::new(a, b, c, d) {
                        test_quats.push(q);
                    }
                }
            }
        }
    }

    println!(
        "Testing Hopf continuous roundtrip on {} distinct unit quaternions...",
        test_quats.len()
    );

    for (idx, q) in test_quats.iter().enumerate() {
        let pt = q.hopf_fiber_point();

        // Base must lie on S2 (unit sphere)
        let s2_norm_sq = pt.base.x * pt.base.x + pt.base.y * pt.base.y + pt.base.z * pt.base.z;
        assert!(
            (s2_norm_sq - 1.0).abs() < 1e-12,
            "Quat {idx}: S2 base not normalized: {s2_norm_sq}"
        );

        // Phasor must lie on S1 (unit circle)
        let u1_norm_sq = pt.fiber_u1[0] * pt.fiber_u1[0] + pt.fiber_u1[1] * pt.fiber_u1[1];
        assert!(
            (u1_norm_sq - 1.0).abs() < 1e-12,
            "Quat {idx}: U1 phasor not normalized: {u1_norm_sq}"
        );

        // Reconstruct from (base, phase)
        let recon = pt.to_unit_s3().expect("valid reconstruction");

        // Coordinates on S3 must match identically within precision
        let err_a = (recon.a - q.a).abs();
        let err_b = (recon.b - q.b).abs();
        let err_c = (recon.c - q.c).abs();
        let err_d = (recon.d - q.d).abs();
        let max_err = err_a.max(err_b).max(err_c).max(err_d);

        assert!(
            max_err < 1e-10,
            "Quat {idx}: Reconstruction error {max_err:.2e} exceeded tolerance! Original: {:?}, Recon: {:?}",
            (q.a, q.b, q.c, q.d),
            (recon.a, recon.b, recon.c, recon.d)
        );

        // Metric distance between q and recon must be 0
        let recon_pt = recon.hopf_fiber_point();
        assert!(
            pt.metric_distance(&recon_pt) < 1e-10,
            "Quat {idx}: Metric distance non-zero: {:.2e}",
            pt.metric_distance(&recon_pt)
        );
    }
}

#[test]
fn test_adversarial_hopf_fiber_loss_causes_complete_sign_collapse() {
    // Verify that dropping psi causes complete loss of invertibility and collapses antipodal signs (q vs -q)
    let test_pairs = [
        (
            UnitS3::IDENTITY,
            UnitS3::new(-1.0, 0.0, 0.0, 0.0).unwrap(),
            "Identity vs -Identity",
        ),
        (
            UnitS3::I,
            UnitS3::new(0.0, -1.0, 0.0, 0.0).unwrap(),
            "I vs -I",
        ),
        (
            UnitS3::J,
            UnitS3::new(0.0, 0.0, -1.0, 0.0).unwrap(),
            "J vs -J",
        ),
        (
            UnitS3::K,
            UnitS3::new(0.0, 0.0, 0.0, -1.0).unwrap(),
            "K vs -K",
        ),
        (
            UnitS3::new(0.5, 0.5, 0.5, 0.5).unwrap(),
            UnitS3::new(-0.5, -0.5, -0.5, -0.5).unwrap(),
            "Diagonal (+1/2) vs Diagonal (-1/2)",
        ),
        (
            UnitS3::new(0.2, -0.4, 0.6, -0.663324958).unwrap(),
            UnitS3::new(-0.2, 0.4, -0.6, 0.663324958).unwrap(),
            "Arbitrary Q vs -Q",
        ),
        (
            UnitS3::new(0.0, 0.0, 1.0, 0.0).unwrap(),
            UnitS3::new(0.0, 0.0, -1.0, 0.0).unwrap(),
            "South Pole J vs -J",
        ),
    ];

    for (pos_q, neg_q, label) in test_pairs {
        // 1. Distance on S3 between q and -q is maximal (diameter of S3 = 2.0)
        let s3_dist = ((pos_q.a - neg_q.a).powi(2)
            + (pos_q.b - neg_q.b).powi(2)
            + (pos_q.c - neg_q.c).powi(2)
            + (pos_q.d - neg_q.d).powi(2))
        .sqrt();
        assert!(
            (s3_dist - 2.0).abs() < 1e-10,
            "{label}: Antipodal distance on S3 must be exactly 2.0, got {s3_dist}"
        );

        // 2. Continuous Hopf projections on S2: pi(q) and pi(-q)
        let base_pos = pos_q.hopf_map();
        let base_neg = neg_q.hopf_map();

        let s2_dist = ((base_pos.x - base_neg.x).powi(2)
            + (base_pos.y - base_neg.y).powi(2)
            + (base_pos.z - base_neg.z).powi(2))
        .sqrt();

        assert!(
            s2_dist < 1e-14,
            "{label}: Hopf projection MUST project q and -q to IDENTICAL base point on S2! Got distance {s2_dist}"
        );

        // 3. Fiber phase psi: psi(q) and psi(-q) must differ by exactly pi
        let psi_pos = pos_q.fiber_phase();
        let psi_neg = neg_q.fiber_phase();
        let mut d_psi = (psi_pos - psi_neg).abs();
        if d_psi > std::f64::consts::PI {
            d_psi = 2.0 * std::f64::consts::PI - d_psi;
        }
        assert!(
            (d_psi - std::f64::consts::PI).abs() < 1e-10,
            "{label}: Fiber phase difference must be exactly pi radians! Got {d_psi}"
        );

        // 4. FALSIFY / ATTACK: If psi is dropped and an observer reconstructs from S2 alone
        // using an arbitrary fixed section (e.g. psi = 0):
        let blind_recon = UnitS3::from_hopf_fiber(&base_pos, 0.0).unwrap();
        let blind_recon_neg = UnitS3::from_hopf_fiber(&base_neg, 0.0).unwrap();

        // Both reconstruct to the identical point!
        assert_eq!(
            blind_recon, blind_recon_neg,
            "{label}: Dropping fiber phase causes total sign collapse: q and -q reconstruct to identical quat!"
        );

        // For at least one of (pos_q, neg_q), the blind reconstruction is completely wrong
        let err_pos = ((blind_recon.a - pos_q.a).powi(2)
            + (blind_recon.b - pos_q.b).powi(2)
            + (blind_recon.c - pos_q.c).powi(2)
            + (blind_recon.d - pos_q.d).powi(2))
        .sqrt();
        let err_neg = ((blind_recon.a - neg_q.a).powi(2)
            + (blind_recon.b - neg_q.b).powi(2)
            + (blind_recon.c - neg_q.c).powi(2)
            + (blind_recon.d - neg_q.d).powi(2))
        .sqrt();

        // One of them must have distance >= 1.0 from the blind reconstruction!
        assert!(
            err_pos.max(err_neg) >= 1.0,
            "{label}: Blind reconstruction failed to exhibit expected sign error! err_pos: {err_pos}, err_neg: {err_neg}"
        );

        // 5. Fixed-Point Q1.30 Verification:
        let pos_q30 = pos_q.to_q30();
        let neg_q30 = neg_q.to_q30();
        let pt_pos_q30 = pos_q30.hopf_fiber_project();
        let pt_neg_q30 = neg_q30.hopf_fiber_project();

        // Bases are identical in Q30
        assert_eq!(
            pt_pos_q30.base.0, pt_neg_q30.base.0,
            "{label}: Q30 base must be identical for q and -q!"
        );

        // Fiber phasors are antipodal in Q30: u1(q) = -u1(-q)
        for i in 0..2 {
            let sum = pt_pos_q30.fiber_u1[i] as i64 + pt_neg_q30.fiber_u1[i] as i64;
            assert!(
                sum.abs() <= 2,
                "{label}: Q30 fiber phasor must be opposite sign for q and -q! pos: {:?}, neg: {:?}",
                pt_pos_q30.fiber_u1,
                pt_neg_q30.fiber_u1
            );
        }
    }
}

// ============================================================================
// 2. ADVERSARIAL CHALLENGE: PAIRED-H4 ICOSIAN EXACT Z[phi] RING ARITHMETIC
// ============================================================================

/// 2x2 integer matrix in i128 to represent exact ring elements in Z[phi]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Mat2([[i128; 2]; 2]);

impl Mat2 {
    const IDENTITY: Self = Self([[1, 0], [0, 1]]);
    // Companion matrix F = [[0, 1], [1, 1]]
    const FIBONACCI_COMPANION: Self = Self([[0, 1], [1, 1]]);

    fn mul(&self, other: &Self) -> Self {
        let mut res = [[0i128; 2]; 2];
        for i in 0..2 {
            for j in 0..2 {
                res[i][j] = self.0[i][0] * other.0[0][j] + self.0[i][1] * other.0[1][j];
            }
        }
        Self(res)
    }

    /// Construct matrix representation of (a + b*phi) as a*I + b*F = [[a, b], [b, a + b]]
    fn from_zphi(z: ZPhi) -> Self {
        let a = z.a as i128;
        let b = z.b as i128;
        Self([[a, b], [b, a + b]])
    }
}

#[test]
fn test_adversarial_fibonacci_matrix_vs_zphi_large_exponents() {
    // Stress test the Fibonacci matrix recurrence [[0, 1], [1, 1]] across large exponents
    // up to the exact i64 overflow boundary (n = 92).

    let mut mat_power = Mat2::IDENTITY;
    let mut zphi_power = ZPhi::new(1, 0); // 1 = phi^0
    let phi = ZPhi::new(0, 1); // phi = phi^1

    println!("Stress-testing Z[phi] ring multiplication against Fibonacci companion matrix...");

    for n in 0..=92 {
        // 1. Invariant check: matrix power matches zphi power
        // Mat2 power has form: [[F_{n-1}, F_n], [F_n, F_{n+1}]]
        // zphi_power has form: a = F_{n-1}, b = F_n
        let mat_a = mat_power.0[0][0];
        let mat_b = mat_power.0[0][1];
        let mat_b_alt = mat_power.0[1][0];
        let mat_c = mat_power.0[1][1];

        assert_eq!(
            mat_b, mat_b_alt,
            "Symmetry broken in Fibonacci matrix at n={n}!"
        );
        assert_eq!(
            mat_c,
            mat_a + mat_b,
            "Fibonacci recurrence violated in matrix at n={n}!"
        );

        assert_eq!(
            zphi_power.a as i128, mat_a,
            "ZPhi.a mismatch at n={n}! zphi: {}, mat: {mat_a}",
            zphi_power.a
        );
        assert_eq!(
            zphi_power.b as i128, mat_b,
            "ZPhi.b mismatch at n={n}! zphi: {}, mat: {mat_b}",
            zphi_power.b
        );

        // 2. Compare against times_phi() method
        if n > 0 {
            // Verify that times_phi() gives identical result
            let mut step_by_times_phi = ZPhi::new(1, 0);
            for _ in 0..n {
                step_by_times_phi = step_by_times_phi.times_phi().unwrap();
            }
            assert_eq!(
                step_by_times_phi, zphi_power,
                "times_phi() deviated from checked_mul at n={n}!"
            );

            // Verify times_phi_inverse() inverts times_phi()
            let inverted = zphi_power.times_phi_inverse().unwrap();
            let mut prev_power = ZPhi::new(1, 0);
            for _ in 0..(n - 1) {
                prev_power = prev_power.times_phi().unwrap();
            }
            assert_eq!(
                inverted, prev_power,
                "times_phi_inverse() failed to invert at n={n}!"
            );
        }

        // 3. Binary exponentiation in Z[phi]
        let bin_exp = zphi_pow(phi, n).expect("bin_exp within range");
        if n == 0 {
            assert_eq!(bin_exp, ZPhi::new(1, 0));
        } else {
            assert_eq!(
                bin_exp, zphi_power,
                "Binary exponentiation deviated at n={n}!"
            );
        }

        // Advance to n+1 if not at boundary
        if n < 92 {
            mat_power = mat_power.mul(&Mat2::FIBONACCI_COMPANION);
            zphi_power = zphi_power.checked_mul(phi).expect("should not overflow <= 92");
        }
    }

    println!("Successfully verified exact isomorphism up to n = 92!");
    println!(
        "F_91 = {}, F_92 = {}",
        zphi_power.a, zphi_power.b
    );
    assert_eq!(zphi_power.b, 7_540_113_804_746_346_429i64); // Known F_92

    // 4. OVERFLOW BOUNDARY AT n = 93:
    // F_93 = F_91 + F_92 = 4660046610375530309 + 7540113804746346429 = 12200160415121876738 > i64::MAX
    let overflow_mul = zphi_power.checked_mul(phi);
    assert_eq!(
        overflow_mul,
        Err(PrimeRouteError::ArithmeticOverflow),
        "ZPhi::checked_mul MUST cleanly return ArithmeticOverflow at n=93!"
    );

    let overflow_step = zphi_power.times_phi();
    assert_eq!(
        overflow_step,
        Err(PrimeRouteError::ArithmeticOverflow),
        "ZPhi::times_phi MUST cleanly return ArithmeticOverflow at n=93!"
    );
}

fn zphi_pow(base: ZPhi, mut exp: u32) -> Result<ZPhi, PrimeRouteError> {
    let mut res = ZPhi::new(1, 0);
    let mut cur = base;
    while exp > 0 {
        if exp & 1 == 1 {
            res = res.checked_mul(cur)?;
        }
        if exp > 1 {
            cur = cur.checked_mul(cur)?;
        }
        exp >>= 1;
    }
    Ok(res)
}

#[test]
fn test_adversarial_zphi_ring_isomorphism_arbitrary_elements() {
    // Test that for ANY arbitrary elements in Z[phi],
    // ZPhi::checked_mul is strictly isomorphic to 2x2 matrix multiplication:
    // M(z1 * z2) == M(z1) * M(z2)
    let test_elements = [
        ZPhi::new(3, -5),
        ZPhi::new(-17, 42),
        ZPhi::new(100, 250),
        ZPhi::new(-1000, -3000),
        ZPhi::new(54321, -12345),
        ZPhi::new(0, 0),
        ZPhi::new(1, 0),
        ZPhi::new(0, 1),
        ZPhi::new(-1, -1),
    ];

    for z1 in &test_elements {
        for z2 in &test_elements {
            let product_zphi = z1.checked_mul(*z2).unwrap();

            let m1 = Mat2::from_zphi(*z1);
            let m2 = Mat2::from_zphi(*z2);
            let m_prod = m1.mul(&m2);

            let expected_m = Mat2::from_zphi(product_zphi);
            assert_eq!(
                m_prod, expected_m,
                "Matrix isomorphism failed for {:?} * {:?}",
                z1, z2
            );

            // Test Galois conjugation
            // sigma(z) = (a + b) - b*phi
            // Companion = phi * sigma(z) = -b + a*phi (exact quarter turn [[0, -1], [1, 0]])
            let conj = z1.golden_conjugate().unwrap();
            assert_eq!(conj.a, z1.a + z1.b);
            assert_eq!(conj.b, -z1.b);

            let companion = conj.times_phi().unwrap();
            assert_eq!(
                companion.a, -z1.b,
                "Galois companion a must be -b! Got {}, expected {}",
                companion.a, -z1.b
            );
            assert_eq!(
                companion.b, z1.a,
                "Galois companion b must be a! Got {}, expected {}",
                companion.b, z1.a
            );
        }
    }
}

// ============================================================================
// 3. ADVERSARIAL CHALLENGE: PRIME ROUTE PREFIX COLLAPSE EMPIRICAL REPRODUCTION
// ============================================================================

fn label(seed: &str) -> String {
    format!("blake3:{}", blake3::hash(seed.as_bytes()).to_hex())
}

fn provenance() -> ManifestProvenance {
    ManifestProvenance {
        tokenizer_cid: label("attention-tokenizer"),
        corpus_cid: label("attention-corpus"),
        compiler_cid: label("attention-compiler"),
        cost_profile_cid: label("attention-cost-profile"),
    }
}

fn registry() -> PrimeRegistry {
    PrimeRegistry::compile(
        &["a", "b", "c", "d", "e", "p", "q", "x", "y", "z"]
            .into_iter()
            .map(|id| SemanticAtom {
                semantic_atom_id: id.to_owned(),
                payload_cid: label(&format!("payload-{id}")),
            })
            .collect::<Vec<_>>(),
    )
    .unwrap()
}

fn address(
    registry: &PrimeRegistry,
    id: &str,
    torsion: f64,
    spin_lane: usize,
    radial_lane: i64,
) -> GeometricAddress {
    let binding = registry.binding_for_id(id).unwrap();
    let r4 = match spin_lane % 4 {
        0 => [1.0, 0.0, 0.0, 0.0],
        1 => [0.0, 1.0, 0.0, 0.0],
        2 => [0.0, 0.0, 1.0, 0.0],
        _ => [0.0, 0.0, 0.0, 1.0],
    };
    GeometricAddress {
        atom: binding.atom,
        spin: SpinTorsionState::new(
            CoreUnitS3Q30::from_r4(r4).unwrap(),
            PhaseQ29::from_radians(0.0).unwrap(),
            PhaseQ29::from_radians(torsion).unwrap(),
        )
        .unwrap(),
        radial: ZPhi::new(radial_lane, radial_lane + 1),
        payload_cid: binding.payload_cid.clone(),
    }
}

#[allow(dead_code)]
#[derive(Clone)]
struct FixtureRoutes {
    a: GeometricAddress,
    b: GeometricAddress,
    c0: GeometricAddress,
    c1: GeometricAddress,
    d: GeometricAddress,
    e: GeometricAddress,
    p: GeometricAddress,
    q: GeometricAddress,
    x: GeometricAddress,
    y: GeometricAddress,
    z: GeometricAddress,
}

fn fixture(maximum_candidates: u16) -> (GeometricAttentionArtifact, FixtureRoutes) {
    let registry = registry();
    let routes = FixtureRoutes {
        a: address(&registry, "a", 0.0, 0, 1),
        b: address(&registry, "b", 0.05, 0, 2),
        c0: address(&registry, "c", 0.1, 0, 3),
        c1: address(&registry, "c", 0.6, 0, 4),
        d: address(&registry, "d", -0.05, 1, 5),
        e: address(&registry, "e", -0.1, 1, 6),
        p: address(&registry, "p", 0.15, 0, 7),
        q: address(&registry, "q", -0.15, 1, 8),
        x: address(&registry, "x", 0.2, 0, 9),
        y: address(&registry, "y", -0.2, 2, 10),
        z: address(&registry, "z", 0.3, 3, 11),
    };
    let sentences = vec![
        RouteSentence {
            sentence_id: "ordered-a".to_owned(),
            routes: vec![
                routes.a.clone(),
                routes.b.clone(),
                routes.c0.clone(),
                routes.x.clone(),
            ],
        },
        RouteSentence {
            sentence_id: "ordered-d".to_owned(),
            routes: vec![
                routes.d.clone(),
                routes.b.clone(),
                routes.c0.clone(),
                routes.y.clone(),
            ],
        },
        RouteSentence {
            sentence_id: "last-two-e".to_owned(),
            routes: vec![routes.e.clone(), routes.c0.clone(), routes.z.clone()],
        },
        RouteSentence {
            sentence_id: "last-one-p".to_owned(),
            routes: vec![routes.p.clone(), routes.x.clone()],
        },
        RouteSentence {
            sentence_id: "last-one-q".to_owned(),
            routes: vec![routes.q.clone(), routes.y.clone()],
        },
    ];
    let compilation = compile_spin_manifest(
        &sentences,
        registry,
        ZeroPowerBridge::ContinuousNull,
        provenance(),
        NonZeroU16::new(maximum_candidates).unwrap(),
        NonZeroUsize::new(1).unwrap(),
    )
    .unwrap();
    let attention =
        GeometricAttentionArtifact::compile_from_manifest_witnesses(&compilation.manifest).unwrap();
    (attention, routes)
}

fn state(
    attention: &GeometricAttentionArtifact,
    history: &[&GeometricAddress],
) -> CausalAttentionState {
    attention
        .causal_state_from_history(
            &history
                .iter()
                .map(|address| (*address).clone())
                .collect::<Vec<_>>(),
        )
        .unwrap()
}

#[test]
fn test_adversarial_prime_route_prefix_collapse_empirical_reproduction() {
    let (attention, routes) = fixture(8);

    // Test multiple distinct global prefix entity tokens (p vs q vs a)
    // All sharing the exact same local suffix: (b, c0)
    let state_p = state(&attention, &[&routes.p, &routes.b, &routes.c0]);
    let state_q = state(&attention, &[&routes.q, &routes.b, &routes.c0]);
    let state_e = state(&attention, &[&routes.e, &routes.b, &routes.c0]);

    let mut query_p = attention
        .query(&state_p, AttentionControl::RealGeometry)
        .expect("query p");
    let mut query_q = attention
        .query(&state_q, AttentionControl::RealGeometry)
        .expect("query q");
    let mut query_e = attention
        .query(&state_e, AttentionControl::RealGeometry)
        .expect("query e");

    // 1. Verify that the distinct prefixes produce distinct sentence hash keys
    let row_p = &query_p.rows_read[2];
    let row_q = &query_q.rows_read[2];
    let row_e = &query_e.rows_read[2];

    assert_eq!(row_p.source, AttentionRowSource::OrderedSentence);
    assert_eq!(row_q.source, AttentionRowSource::OrderedSentence);
    assert_eq!(row_e.source, AttentionRowSource::OrderedSentence);

    // None of these unseen sentences hit the training table
    assert!(!row_p.hit, "Unseen sentence p must not hit!");
    assert!(!row_q.hit, "Unseen sentence q must not hit!");
    assert!(!row_e.hit, "Unseen sentence e must not hit!");

    // All three have distinct sentence keys
    assert_ne!(row_p.key, row_q.key);
    assert_ne!(row_p.key, row_e.key);
    assert_ne!(row_q.key, row_e.key);

    // 2. FALSIFY / CRITICAL COLLAPSE:
    // Despite completely different prefix entities (e.g. "p", "q", "e"),
    // because IS missed, the candidate sets and selected predictions COLLAPSE IDENTICALLY!
    assert_eq!(
        query_p.candidates, query_q.candidates,
        "Candidate sets must collapse identically when IS misses!"
    );
    assert_eq!(
        query_p.candidates, query_e.candidates,
        "Candidate sets must collapse identically when IS misses!"
    );
    assert_eq!(
        query_p.selected, query_q.selected,
        "Selected prediction must collapse identically when IS misses!"
    );
    assert_eq!(
        query_p.selected, query_e.selected,
        "Selected prediction must collapse identically when IS misses!"
    );

    // Masking the unseen hash keys leaves the queries 100% bitwise identical
    query_p.rows_read[2].key = AttentionRowKey::OrderedSentence("masked".to_string());
    query_q.rows_read[2].key = AttentionRowKey::OrderedSentence("masked".to_string());
    query_e.rows_read[2].key = AttentionRowKey::OrderedSentence("masked".to_string());

    assert_eq!(
        query_p, query_q,
        "Query trace must be completely identical after masking unseen IS key!"
    );
    assert_eq!(
        query_p, query_e,
        "Query trace must be completely identical after masking unseen IS key!"
    );

    println!("Empirically reproduced and confirmed Unseen Global Prefix Collapse failure mode!");
}
