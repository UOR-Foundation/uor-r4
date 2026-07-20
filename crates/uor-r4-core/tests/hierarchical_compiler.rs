use uor_r4_core::transformerless::compiler::{Corpus, induce_hierarchical_codes, STAGES};

#[test]
fn test_induce_hierarchical_codes() {
    let n = 20;
    let story = vec![1u32; n];
    let mut next = vec![0u32; n];
    
    // Pattern: 10 -> 20 -> 30 repeated 6 times
    for chunk in 0..6 {
        let base = chunk * 3;
        if base + 2 < n {
            next[base] = 10;
            next[base + 1] = 20;
            next[base + 2] = 30;
        }
    }
    
    let corpus = Corpus {
        n,
        stories: 1,
        story,
        input: vec![1u32; n],
        next,
        t_argmax: vec![0u32; n],
        top_tokens: vec![[0u32; 3]; n],
        top_weights: vec![[0u32; 3]; n],
    };

    let vocab = 100;
    let mut token_codes = vec![0u8; vocab * STAGES];
    token_codes[10 * STAGES] = 1;
    token_codes[10 * STAGES + 1] = 2;
    token_codes[10 * STAGES + 2] = 3;
    token_codes[10 * STAGES + 3] = 4;

    let hc = induce_hierarchical_codes(&token_codes, vocab, &corpus);

    // Verify stable type prefixes
    assert_eq!(hc.token_type_prefixes.get("10"), Some(&vec![1, 2, 3, 4]));
    
    // Verify relational prefixes (transition pair and triplet)
    assert!(!hc.relational_prefixes.is_empty(), "Relational prefixes should not be empty");
    
    let has_pair = hc.relational_prefixes.iter().any(|path| path == &[10, 20]);
    let has_triplet = hc.relational_prefixes.iter().any(|path| path == &[10, 20, 30]);
    assert!(has_pair, "Should contain transition pair [10, 20]");
    assert!(has_triplet, "Should contain transition triplet [10, 20, 30]");
}
