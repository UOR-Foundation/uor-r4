use uor_r4_core::semantic::{encode_event, encode_graph_edge, encode_statement, expand_atom};

#[test]
fn test_vsa_algebra_identities() {
    let space = "test_space";
    let a = expand_atom("entity", "a", space);
    let b = expand_atom("entity", "b", space);

    // 1. Self-Inverse (Hamming similarity of bind(A, bind(A, B)) with B is 1.0)
    let bound = a.bind(&b);
    let unbound = a.unbind(&bound);
    assert_eq!(
        unbound.similarity(&b),
        1.0,
        "unbind should recover original hypervector"
    );

    // 2. Commutativity of Bind: bind(A, B) == bind(B, A)
    let bound_ba = b.bind(&a);
    assert_eq!(
        bound.similarity(&bound_ba),
        1.0,
        "bind should be commutative"
    );

    // 3. Permutation circular shifts: permute(A, 1024) == A
    let perm_zero = a.permute(0);
    assert_eq!(perm_zero.similarity(&a), 1.0);
    let perm_full = a.permute(1024);
    assert_eq!(perm_full.similarity(&a), 1.0);
}

#[test]
fn test_vsa_grounding_conformance_signatures() {
    let space = "test_space";

    // Statements
    let stmt = encode_statement("Paris", "capital_of", "France", space);
    // Events
    let event = encode_event("Alice", "visited", "2026-07-20", "Paris", space);
    // Graph edge
    let edge = encode_graph_edge("NodeA", "links_to", "NodeB", space);

    // Verify non-zero and stable dimension size
    assert_eq!(stmt.0.len(), 16);
    assert_eq!(event.0.len(), 16);
    assert_eq!(edge.0.len(), 16);

    // Asserts deterministic hash expansion - bit conformance
    let paris_entity = expand_atom("entity", "Paris", space);
    let paris_entity_again = expand_atom("entity", "Paris", space);
    assert_eq!(paris_entity.similarity(&paris_entity_again), 1.0);
}
