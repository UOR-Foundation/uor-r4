use uor_r4_core::transformerless::cd_space::{
    CayleyDicksonVector, ComplexNumber, Octonion, Quaternion,
};
use uor_r4_core::transformerless::endomorphism::EndomorphismAlgebra;

#[test]
fn test_clifford_anticommutator_relations() {
    for i in 1..=6 {
        let g_i = EndomorphismAlgebra::clifford_generator(i);
        // g_i^2 == -I (for 8x8 block)
        let g_i_sq = g_i.mul(&g_i);

        // Check top-left 8x8 block is -1
        for row in 0..8 {
            assert!(
                (g_i_sq.matrix[row * 16 + row] - (-1.0)).abs() < 1e-5,
                "Clifford generator {} squared must equal -I in 8x8 block",
                i
            );
        }

        for j in (i + 1)..=6 {
            let g_j = EndomorphismAlgebra::clifford_generator(j);
            let anticomm = g_i.anticommutator(&g_j);

            // Anticommutator {g_i, g_j} == 0 for i != j in top-left 8x8 block
            for row in 0..8 {
                for col in 0..8 {
                    assert!(
                        anticomm.matrix[row * 16 + col].abs() < 1e-5,
                        "Clifford generators {}, {} must anticommute: {{g_i, g_j}} == 0",
                        i,
                        j
                    );
                }
            }
        }
    }
}

#[test]
fn test_clifford_volume_element_identity() {
    let g1 = EndomorphismAlgebra::clifford_generator(1);
    let g2 = EndomorphismAlgebra::clifford_generator(2);
    let g3 = EndomorphismAlgebra::clifford_generator(3);
    let g4 = EndomorphismAlgebra::clifford_generator(4);
    let g5 = EndomorphismAlgebra::clifford_generator(5);
    let g6 = EndomorphismAlgebra::clifford_generator(6);

    // \omega = \gamma_1 \gamma_2 \gamma_3 \gamma_4 \gamma_5 \gamma_6
    let prod = g1.mul(&g2).mul(&g3).mul(&g4).mul(&g5).mul(&g6);
    let vol = EndomorphismAlgebra::volume_element();

    for i in 0..256 {
        assert!(
            (prod.matrix[i] - vol.matrix[i]).abs() < 1e-4,
            "Product of Clifford generators g1..g6 must match volume element L_e7"
        );
    }
}

#[test]
fn test_volume_element_centralizer() {
    let vol = EndomorphismAlgebra::volume_element();
    assert!(
        vol.commutes_with_volume(),
        "Volume element must commute with itself"
    );

    let eye = EndomorphismAlgebra::identity();
    assert!(
        eye.commutes_with_volume(),
        "Identity matrix must commute with volume element"
    );
}

#[test]
fn test_endomorphism_application_on_vector() {
    let e1 = Octonion::imaginary(1);
    let e2 = Octonion::imaginary(2);
    let quat = Quaternion::default();
    let comp = ComplexNumber::default();

    let op = EndomorphismAlgebra::left_octonion(&e1);
    let vec = CayleyDicksonVector::embed(&e2, &quat, &comp, 0.0, 0.0);

    let res = op.apply(&vec);
    let res_oct = res.project_octonion();

    // L_e1(e2) = e1 * e2 = e4
    assert_eq!(res_oct, Octonion::imaginary(4));
}
