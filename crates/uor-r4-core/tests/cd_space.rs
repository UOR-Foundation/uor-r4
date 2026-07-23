use uor_r4_core::transformerless::cd_space::{
    CayleyDicksonVector, ComplexNumber, Octonion, Quaternion,
};

#[test]
fn test_octonion_multiplication_fano_plane() {
    let e1 = Octonion::imaginary(1);
    let e2 = Octonion::imaginary(2);
    let e3 = Octonion::imaginary(3);

    // e1 * e2 = e4
    let e1_e2 = e1.mul(&e2);
    assert_eq!(e1_e2, Octonion::imaginary(4));

    // Non-associativity across different Fano lines:
    // (e1 * e2) * e3 = e4 * e3 = -e6
    // e1 * (e2 * e3) = e1 * e5 = +e6
    let e2_e3 = e2.mul(&e3);
    let lhs = e1_e2.mul(&e3);
    let rhs = e1.mul(&e2_e3);

    assert_ne!(
        lhs, rhs,
        "Octonion multiplication across different lines must be non-associative"
    );
    assert_eq!(lhs.r[6], -1.0);
    assert_eq!(rhs.r[6], 1.0);
}

#[test]
fn test_quaternion_multiplication_associativity() {
    let q1 = Quaternion::new([0.0, 1.0, 0.0, 0.0]); // \epsilon_1
    let q2 = Quaternion::new([0.0, 0.0, 1.0, 0.0]); // \epsilon_2
    let q3 = Quaternion::new([0.0, 0.0, 0.0, 1.0]); // \epsilon_3

    // (\epsilon_1 \epsilon_2) \epsilon_3 == \epsilon_1 (\epsilon_2 \epsilon_3)
    let lhs = q1.mul(&q2).mul(&q3);
    let rhs = q1.mul(&q2.mul(&q3));

    assert_eq!(lhs, rhs, "Quaternion multiplication must be associative");
    assert_eq!(lhs.r[0], -1.0, "i * j * k == -1");
}

#[test]
fn test_complex_multiplication_and_conjugate() {
    let c = ComplexNumber::new([3.0, 4.0]);
    let c_conj = c.conjugate();

    let prod = c.mul(&c_conj);
    assert_eq!(prod.r[0], 25.0);
    assert_eq!(prod.r[1], 0.0);
    assert_eq!(c.norm_squared(), 25.0);
}

#[test]
fn test_cayley_dickson_nested_embeddings() {
    let oct = Octonion::new([1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0]);
    let quat = Quaternion::new([9.0, 10.0, 11.0, 12.0]);
    let comp = ComplexNumber::new([13.0, 14.0]);
    let real = 15.0;
    let scalar = 16.0;

    let vec = CayleyDicksonVector::embed(&oct, &quat, &comp, real, scalar);

    assert_eq!(vec.project_octonion(), oct);
    assert_eq!(vec.project_quaternion(), quat);
    assert_eq!(vec.project_complex(), comp);
    assert_eq!(vec.project_real(), real);
    assert_eq!(vec.project_scalar(), scalar);
}

#[test]
fn test_norm_multiplicativity() {
    let o1 = Octonion::new([1.0, -2.0, 3.0, 0.5, 0.0, 1.5, -1.0, 2.0]);
    let o2 = Octonion::new([0.5, 1.0, -1.5, 2.0, 3.0, -0.5, 1.0, 0.0]);

    let o12 = o1.mul(&o2);
    let norm_product = o1.norm() * o2.norm();
    let product_norm = o12.norm();

    assert!(
        (product_norm - norm_product).abs() < 1e-4,
        "Octonion norm must be multiplicative: ||o1 * o2|| == ||o1|| * ||o2||"
    );
}
