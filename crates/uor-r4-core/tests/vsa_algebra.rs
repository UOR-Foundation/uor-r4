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

#[test]
fn test_encode_statement_and_role_swapping() {
    use uor_r4_core::semantic::{encode_statement, expand_atom};

    let space = "space_1";
    let statement_1 = encode_statement("paris_cid", "capital_of", "france_cid", space);
    let statement_2_swapped = encode_statement("france_cid", "capital_of", "paris_cid", space);

    // Swapped statement should be distinguishable (similarity < 0.80) to original
    let sim_swapped = statement_1.similarity(&statement_2_swapped);
    assert!(sim_swapped < 0.80, "Role swapped similarity: {}", sim_swapped);

    // Verify component presence by unbinding
    let r_subj = expand_atom("role", "subject", space);
    let h_paris = expand_atom("entity", "paris_cid", space);

    // Unbind subject role from statement_1. It should have high similarity to Paris entity
    let retrieved_subj = statement_1.unbind(&r_subj);
    let sim_retrieved = retrieved_subj.similarity(&h_paris);
    assert!(sim_retrieved > 0.60, "Retrieved subject similarity: {}", sim_retrieved);
}

#[test]
fn test_encode_event_and_graph_edge_directionality() {
    use uor_r4_core::semantic::{encode_event, encode_graph_edge};

    let space = "space_1";
    let event = encode_event("alice_cid", "visited", "t_2026", "london_cid", space);
    assert_ne!(event.0, [0u64; 16]);

    let edge_a_to_b = encode_graph_edge("node_a", "points_to", "node_b", space);
    let edge_b_to_a = encode_graph_edge("node_b", "points_to", "node_a", space);

    // Directed target edge swapping must be distinguishable (similarity < 0.65) due to permutation shift
    let sim_edge = edge_a_to_b.similarity(&edge_b_to_a);
    assert!(sim_edge < 0.65, "Directed edge similarity: {}", sim_edge);
}

