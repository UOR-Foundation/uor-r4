use uor_r4_integer::{
    Bundle, ChatSession, IntegerModel, ReadGeometry, ReadMode, SamplePolicy, SlotTarget,
    TOTAL_MEMORY_CANDIDATES,
};

const TOTAL: u64 = 1u64 << 48;

#[test]
fn test_conversational_refusal_without_hyperbolic_flag() {
    let model = IntegerModel::synthetic_lorentz_for_test();
    assert_eq!(model.config().read_geometry, ReadGeometry::Lorentz);
    assert!(model.is_lorentz());

    let mut session = model.new_conversational_session();
    assert!(!session.allow_hyperbolic_cache);

    // Default conversational step on Lorentz without allow_hyperbolic_cache must error
    // (exact backward-compatibility contract)
    let result =
        model.step_conversational(&mut session, 10, SlotTarget::Dialogue, ReadMode::Enabled);
    assert!(result.is_err());
    let err_str = result.err().unwrap().to_string();
    assert!(
        err_str.contains("dot-read"),
        "error must mention dot-read: {err_str}"
    );
}

#[test]
fn test_conversational_hyperbolic_cache_stepping_and_paging() {
    let model = IntegerModel::synthetic_lorentz_for_test();
    let mut session = model.new_conversational_session();
    session.enable_hyperbolic_cache();
    assert!(session.allow_hyperbolic_cache);

    // 1. Ingest into persistent persona (slots 0..3)
    for tok in [101, 102, 103] {
        let step = model
            .step_conversational(&mut session, tok, SlotTarget::Persistent, ReadMode::Enabled)
            .expect("persistent slot stepping");
        let sum: u64 = step.probabilities.iter().sum();
        assert_eq!(sum, TOTAL, "probabilities must sum to 2^48");
    }
    assert_eq!(session.persistent_keys.len(), 3);
    assert_eq!(session.persistent_key_norms.len(), 3);
    for &kn in &session.persistent_key_norms {
        assert!(kn >= 0, "Lorentz key norm must be non-negative: {kn}");
    }

    session.seal_persistent();
    assert!(session.is_persistent_sealed());

    // 2. Ingest dialogue tokens into rolling ring buffer
    session.start_turn();
    for tok in 200..210 {
        let step = model
            .step_conversational(&mut session, tok, SlotTarget::Dialogue, ReadMode::Enabled)
            .expect("dialogue stepping");
        let sum: u64 = step.probabilities.iter().sum();
        assert_eq!(sum, TOTAL);
    }
    assert_eq!(session.dialogue_len(), 10);
    for idx in 0..10 {
        assert!(session.dialogue_key_norms[idx] >= 0);
    }

    // 3. Test ReadMode::NoRead on hyperbolic cache
    let step_noread = model
        .step_conversational(&mut session, 999, SlotTarget::Dialogue, ReadMode::NoRead)
        .expect("step with NoRead");
    assert_eq!(step_noread.no_read_mass, TOTAL);
    assert!(step_noread.read_masses.iter().all(|&m| m == 0));

    // 4. Exercise L2 Prime Page eviction by filling dialogue capacity (224 slots)
    // Create multiple turns to trigger L2 compression
    for turn in 1..=4 {
        session.start_turn();
        for tok in 0..60 {
            model
                .step_conversational_hyperbolic(
                    &mut session,
                    (turn * 100 + tok) as u32,
                    SlotTarget::Dialogue,
                    ReadMode::Enabled,
                )
                .expect("stepping towards L2 paging");
        }
    }

    // Dialogue capacity is 224, 10 + 240 = 250 steps have occurred, so L2 pages must have been created!
    assert!(
        session.l2_len() > 0,
        "L2 prime paging must have compressed evicted turns into pages (l2_len = {})",
        session.l2_len()
    );
    for p_idx in 0..session.l2_len() {
        assert!(
            session.l2_page_norms[p_idx] >= 0,
            "L2 page key norm must be computed: {}",
            session.l2_page_norms[p_idx]
        );
    }

    // Verify stepping with active L2 pages scores correctly and sums to TOTAL
    let step_with_l2 = model
        .step_conversational(&mut session, 500, SlotTarget::Dialogue, ReadMode::Enabled)
        .expect("step with active L2 pages in hyperbolic cache");
    let sum_l2: u64 = step_with_l2.probabilities.iter().sum();
    assert_eq!(sum_l2, TOTAL);
    assert!(step_with_l2.read_masses.len() <= TOTAL_MEMORY_CANDIDATES);
}

#[test]
fn test_chat_session_hyperbolic_cache_creation_and_streaming() {
    let bundle = Bundle::synthetic_lorentz_for_test();
    assert!(bundle.model().is_lorentz());

    let persona = "You are a geometric AI running hyperbolic cache memory.";
    let mut chat = ChatSession::new(&bundle, Some(persona), 42).expect("chat session creation");

    // Hyperbolic cache should be automatically enabled for Lorentz bundle
    assert!(chat.state().allow_hyperbolic_cache);
    assert!(chat.state().is_persistent_sealed());
    assert!(chat.state().persistent_len() > 0);
    assert_eq!(
        chat.state().persistent_key_norms.len(),
        chat.state().persistent_len()
    );

    // Stream generation tokens using Categorical policy from a user prompt
    chat.set_policy(SamplePolicy::Categorical { top_k: 8 });
    let user_msg = "Explain hyperbolic spacetime geometry.";
    let mut stream = chat
        .generate_stream(user_msg, 10, &[])
        .expect("generate stream");
    let mut chunks = Vec::new();
    while let Some(chunk) = stream.next() {
        chunks.push(chunk);
    }
    assert!(stream.tokens_generated() > 0);
    assert!(stream.tokens_generated() <= 10);
    assert_eq!(stream.tokens_generated(), stream.generated_tokens().len());
    assert!(stream.is_stopped());
}

#[test]
fn test_chat_session_hyperbolic_serialization_roundtrip() {
    let bundle = Bundle::synthetic_lorentz_for_test();
    let persona = "Frontier hyperbolic geometry assistant.";
    let mut chat = ChatSession::new(&bundle, Some(persona), 12345).expect("chat session creation");
    chat.set_policy(SamplePolicy::Categorical { top_k: 4 });

    // Multi-turn exchanges
    for turn in 1..=3 {
        let msg = format!("Turn {turn} message text query");
        let mut stream = chat.generate_stream(&msg, 5, &[]).expect("stream for turn");
        while let Some(_) = stream.next() {}
    }

    // Save session to temporary file
    let tmp_dir = std::env::temp_dir();
    let save_path = tmp_dir.join(format!(
        "uor_r4_hyperbolic_session_{}.json",
        std::process::id()
    ));

    chat.save_session_default(&save_path)
        .expect("save session to json");
    assert!(save_path.exists());

    // Restore session from JSON
    let mut restored = bundle
        .load_chat_session(&save_path, "")
        .expect("load hyperbolic chat session");

    // Clean up temp file
    let _ = std::fs::remove_file(&save_path);

    // Verification of restored session state
    assert!(restored.state().allow_hyperbolic_cache);
    assert_eq!(
        restored.state().persistent_len(),
        chat.state().persistent_len()
    );
    assert_eq!(restored.state().dialogue_len(), chat.state().dialogue_len());
    assert_eq!(restored.state().current_turn(), chat.state().current_turn());

    // Verify bit-for-bit key norms equality
    assert_eq!(
        restored.state().persistent_key_norms,
        chat.state().persistent_key_norms
    );
    for i in 0..chat.state().dialogue_len() {
        assert_eq!(
            restored.state().dialogue_key_norms[i],
            chat.state().dialogue_key_norms[i]
        );
    }

    // Verify determinism: next turn generation from original and restored must yield identical tokens
    let test_prompt = "Verify deterministic continuation.";
    let mut stream_orig = chat
        .generate_stream(test_prompt, 15, &[])
        .expect("stream orig");
    let mut tokens_orig = Vec::new();
    while let Some(_) = stream_orig.next() {}
    tokens_orig.extend_from_slice(stream_orig.generated_tokens());

    let mut stream_rest = restored
        .generate_stream(test_prompt, 15, &[])
        .expect("stream rest");
    let mut tokens_rest = Vec::new();
    while let Some(_) = stream_rest.next() {}
    tokens_rest.extend_from_slice(stream_rest.generated_tokens());

    assert_eq!(
        tokens_orig, tokens_rest,
        "Original and restored hyperbolic sessions must produce bit-identical token sequences!"
    );
}
