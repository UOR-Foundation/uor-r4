use uor_r4_core::semantic::{Hypervector, expand_atom};

#[test]
fn test_vsa_binding_and_unbinding() {
    let a = expand_atom("entity", "cid_a", "space_1");
    let b = expand_atom("role", "cid_b", "space_1");

    // Bind A and B
    let bound = a.bind(&b);
    assert_ne!(bound, a);
    assert_ne!(bound, b);

    // Unbind B to retrieve A
    let retrieved_a = bound.unbind(&b);
    assert_eq!(retrieved_a, a);

    // Unbind A to retrieve B
    let retrieved_b = bound.unbind(&a);
    assert_eq!(retrieved_b, b);
}

#[test]
fn test_vsa_bundling_majority() {
    let a = expand_atom("entity", "cid_a", "space_1");
    let b = expand_atom("entity", "cid_b", "space_1");
    let c = expand_atom("entity", "cid_c", "space_1");

    // Bundle A, B, C
    let bundle = Hypervector::bundle(&[a, b, c]);

    // Check similarity (majority bundle should share higher similarity with members than random vectors)
    let sim_a = bundle.similarity(&a);
    let sim_b = bundle.similarity(&b);
    let sim_c = bundle.similarity(&c);

    let random = expand_atom("entity", "cid_random", "space_1");
    let sim_rand = bundle.similarity(&random);

    assert!(sim_a > 0.55);
    assert!(sim_b > 0.55);
    assert!(sim_c > 0.55);
    assert!(sim_rand < 0.55);
}

#[test]
fn test_vsa_permutation_and_shifts() {
    let a = expand_atom("entity", "cid_a", "space_1");

    // Shift by 0 is identical
    assert_eq!(a.permute(0), a);
    assert_eq!(a.permute(1024), a);

    // Shift by 64
    let shifted_64 = a.permute(64);
    assert_ne!(shifted_64, a);

    // Double shift equals single shift of 128
    assert_eq!(shifted_64.permute(64), a.permute(128));
}

#[test]
fn test_vsa_similarity_orthogonality() {
    let a = expand_atom("entity", "cid_a", "space_1");
    let a_identical = expand_atom("entity", "cid_a", "space_1");
    let b = expand_atom("entity", "cid_b", "space_1");

    // Identical vector similarity is 1.0
    assert_eq!(a.similarity(&a_identical), 1.0);

    // Quasi-orthogonal random vectors similarity is around 0.5 (Hamming distance of random vectors)
    let sim_ab = a.similarity(&b);
    assert!((sim_ab - 0.5).abs() < 0.08, "Similarity was {}", sim_ab);
}
