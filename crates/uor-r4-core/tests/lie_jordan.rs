use uor_r4_core::transformerless::endomorphism::EndomorphismAlgebra;
use uor_r4_core::transformerless::lie_jordan::{
    universal_product, universal_product_u8, LieJordanSplit,
};

#[test]
fn test_lie_jordan_decomposition() {
    let g1 = EndomorphismAlgebra::clifford_generator(1);
    let split = LieJordanSplit::decompose(&g1);

    // Clifford generator g1 is anti-Hermitian (Lie)
    assert!(
        LieJordanSplit::is_anti_hermitian(&split.lie),
        "Lie component must be anti-Hermitian"
    );
    assert!(
        LieJordanSplit::is_hermitian(&split.jordan),
        "Jordan component must be Hermitian"
    );

    // Reconstructed operator matches original A = u + h
    let rec = split.reconstruct();
    for (orig, res) in g1.matrix.iter().zip(&rec.matrix) {
        assert!((orig - res).abs() < 1e-5);
    }
}

#[test]
fn test_universal_product_properties() {
    let g1 = EndomorphismAlgebra::clifford_generator(1);
    let g2 = EndomorphismAlgebra::clifford_generator(2);

    let u1 = LieJordanSplit::decompose(&g1).lie;
    let u2 = LieJordanSplit::decompose(&g2).lie;

    // For strictly anti-Hermitian u1, u2: m(u1, u2) = u1*u2 + u2*u1^\dagger = u1*u2 - u2*u1 = [u1, u2]
    let m_prod = universal_product(&u1, &u2);
    let comm = u1.commutator(&u2);

    for (a, b) in m_prod.matrix.iter().zip(&comm.matrix) {
        assert!(
            (a - b).abs() < 1e-4,
            "Universal product on anti-Hermitian elements must equal Lie commutator"
        );
    }
}

#[test]
fn test_universal_product_u8_integer_kernel() {
    let a: u8 = 0b1100_1010;
    let b: u8 = 0b1010_1100;

    let res_lie = universal_product_u8(a, b, true);
    let res_jordan = universal_product_u8(a, b, false);

    assert_eq!(res_jordan, a & b);
    assert_ne!(res_lie, res_jordan);
}

#[test]
fn test_no_float_no_multiply_in_lie_jordan_core() {
    let source = include_str!("../src/transformerless/lie_jordan.rs");
    let kernel_start = source
        .find("pub fn universal_product_u8")
        .expect("universal_product_u8 function must exist");
    let kernel_code = &source[kernel_start..];

    assert!(
        !kernel_code.contains("f32") && !kernel_code.contains("f64"),
        "universal_product_u8 hot path must contain zero floating-point types"
    );
    assert!(
        !kernel_code.contains(" * ") && !kernel_code.contains(" / "),
        "universal_product_u8 hot path must contain zero multiplication or division operators"
    );
}
