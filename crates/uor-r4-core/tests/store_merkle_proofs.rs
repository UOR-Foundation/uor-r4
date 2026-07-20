use uor_r4_core::semantic::merkle::{compute_merkle_root_and_proof, verify_merkle_proof};

#[test]
fn test_merkle_root_and_proof_verification() {
    let leaves: Vec<&[u8]> = vec![
        b"type:window1:1,2",
        b"entity:window2:10,20",
        b"relation:window3:99",
        b"temporal:window4:100",
    ];

    // Compute root and proof for target leaf at index 1
    let target_idx = 1;
    let (root, proof) = compute_merkle_root_and_proof(&leaves, target_idx).unwrap();

    // Verify proof
    let is_valid = verify_merkle_proof(&root, leaves[target_idx], &proof, target_idx);
    assert!(is_valid, "Merkle proof verification failed");

    // Verify with invalid leaf
    let is_valid_bad = verify_merkle_proof(&root, b"invalid leaf content", &proof, target_idx);
    assert!(!is_valid_bad, "Merkle proof should reject invalid leaf");
}

#[test]
fn test_single_leaf_merkle() {
    let leaves: Vec<&[u8]> = vec![b"single leaf"];
    let (root, proof) = compute_merkle_root_and_proof(&leaves, 0).unwrap();
    assert_eq!(proof.len(), 0);

    let is_valid = verify_merkle_proof(&root, leaves[0], &proof, 0);
    assert!(is_valid);
}
