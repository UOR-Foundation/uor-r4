use uor_r4_router::UorR4Router;

#[test]
fn test_live_selective_backoff_and_intersection() {
    let mut router = UorR4Router::new(0.5);
    router.set_geometry_type("vsa");
    router.clear_corpus(); // Clear default corpus to avoid collisions

    // Index a sentence
    let doc = "The quick brown fox jumps over the lazy dog.";
    router.index_sentence(doc, "shared");

    println!("Facet Store type index keys: {:?}", router.facet_store.type_index.keys());
    println!("Facet Store entity index keys: {:?}", router.facet_store.entity_index.keys());

    // Query matching the sentence in VSA mode via backoff
    let resonances = router.get_top_resonances_native("fox jumps", "shared", 5);
    println!("Resonances returned count: {}", resonances.len());
    for r in &resonances {
        println!("Resonance: sentence='{}', relevance={}", r.sentence, r.relevance);
    }
    assert!(!resonances.is_empty(), "Should match indexed sentence");
    assert!(resonances[0].sentence.contains("fox jumps"), "Should contain the correct text");

    // Retrieve Merkle root of MultiFacetStore
    let epoch_root = router.get_store_epoch_root();
    assert!(epoch_root.starts_with("blake3:"), "Epoch root must be a Blake3 CID");

    // Retrieve Merkle proof for "entity" facet path [100, 200]
    let proof_val = router.get_store_inclusion_proof_native("entity", "100,200");
    assert!(proof_val.is_some(), "Proof must be generated successfully");
}
